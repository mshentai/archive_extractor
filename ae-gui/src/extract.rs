use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;

use archive_extractor::ExtractError;

// ---------------------------------------------------------------------------
// 消息类型
// ---------------------------------------------------------------------------

/// 工作线程 → UI 线程的消息
#[derive(Debug)]
pub enum WorkerMessage {
    Progress {
        current: usize,
        total: usize,
    },
    Finished(Result<(), ExtractError>),
    Log(String),
    /// 当前文件需要密码
    PasswordRequired {
        file_path: PathBuf,
        current: usize,
        total: usize,
        is_wrong_password: bool,
    },
}

/// UI 线程 → 工作线程的命令
#[derive(Debug)]
pub enum WorkerCommand {
    /// 启动批处理解压（不再传 password，改为交互式）
    Extract { path: PathBuf },
    /// 用户提交的密码
    ProvidePassword(String),
    /// 用户取消当前批处理
    Cancel,
}

// ---------------------------------------------------------------------------
// 工作线程内部状态
// ---------------------------------------------------------------------------

/// 批处理解压的运行时状态
struct ExtractState {
    files: Vec<PathBuf>,
    current_index: usize,
    current_password: Option<String>,
    msg_tx: mpsc::Sender<WorkerMessage>,
    has_error: bool,
}

// ---------------------------------------------------------------------------
// 工作线程管理
// ---------------------------------------------------------------------------

/// 启动后台解压工作线程，返回命令发送端和消息接收端
pub fn spawn_worker() -> (mpsc::Sender<WorkerCommand>, mpsc::Receiver<WorkerMessage>) {
    let (cmd_tx, cmd_rx) = mpsc::channel::<WorkerCommand>();
    let (msg_tx, msg_rx) = mpsc::channel::<WorkerMessage>();

    thread::spawn(move || {
        let mut state: Option<ExtractState> = None;

        loop {
            let cmd = match cmd_rx.recv() {
                Ok(cmd) => cmd,
                Err(_) => break, // 通道关闭，线程退出
            };

            match cmd {
                WorkerCommand::Extract { path } => {
                    // 收集文件列表
                    let files = if path.is_file() {
                        vec![path]
                    } else if path.is_dir() {
                        let mut files = Vec::new();
                        scan_directory(&path, 0, 1, &mut files, &msg_tx);
                        files
                    } else {
                        vec![]
                    };

                    if files.is_empty() {
                        let _ =
                            msg_tx.send(WorkerMessage::Log("⚠ 没有找到可解压的文件".to_string()));
                        let _ = msg_tx.send(WorkerMessage::Finished(Ok(())));
                        continue;
                    }

                    state = Some(ExtractState {
                        files,
                        current_index: 0,
                        current_password: None,
                        msg_tx: msg_tx.clone(),
                        has_error: false,
                    });

                    // 开始处理
                    process_current(&mut state);
                }
                WorkerCommand::ProvidePassword(pwd) => {
                    if let Some(s) = &mut state {
                        s.current_password = Some(pwd);
                        process_current(&mut state);
                    }
                }
                WorkerCommand::Cancel => {
                    state = None;
                    let _ = msg_tx.send(WorkerMessage::Finished(Err(ExtractError::ExtractFailed(
                        "用户取消解压".to_string(),
                    ))));
                }
            }
        }
    });

    (cmd_tx, msg_rx)
}

// ---------------------------------------------------------------------------
// 核心处理逻辑
// ---------------------------------------------------------------------------

/// 处理当前状态：循环解压文件，直到需要密码交互或全部完成
fn process_current(state: &mut Option<ExtractState>) {
    let s = match state {
        Some(s) => s,
        None => return,
    };

    while s.current_index < s.files.len() {
        let file_path = &s.files[s.current_index];

        let _ = s.msg_tx.send(WorkerMessage::Progress {
            current: s.current_index + 1,
            total: s.files.len(),
        });
        let _ = s.msg_tx.send(WorkerMessage::Log(format!(
            "[{}/{}] 正在解压: {}",
            s.current_index + 1,
            s.files.len(),
            file_path.display()
        )));

        let result = archive_extractor::extract(file_path, s.current_password.as_deref());

        match &result {
            Ok(()) => {
                let _ = s.msg_tx.send(WorkerMessage::Log(format!(
                    "✓ [{}/{}] 解压完成: {}",
                    s.current_index + 1,
                    s.files.len(),
                    file_path.display()
                )));
                s.current_index += 1;
            }
            Err(ExtractError::PasswordRequired) => {
                // 需要密码 → 请求 UI 输入，暂停批处理
                let _ = s.msg_tx.send(WorkerMessage::PasswordRequired {
                    file_path: file_path.clone(),
                    current: s.current_index + 1,
                    total: s.files.len(),
                    is_wrong_password: false,
                });
                return; // 回到主循环等待 ProvidePassword 或 Cancel
            }
            Err(ExtractError::WrongPassword) => {
                // 密码错误 → 请求 UI 重新输入，暂停批处理
                let _ = s.msg_tx.send(WorkerMessage::PasswordRequired {
                    file_path: file_path.clone(),
                    current: s.current_index + 1,
                    total: s.files.len(),
                    is_wrong_password: true,
                });
                return; // 回到主循环等待新密码
            }
            Err(ExtractError::ExtractFailed(msg)) => {
                let _ = s.msg_tx.send(WorkerMessage::Log(format!(
                    "✗ [{}/{}] 解压失败: {}",
                    s.current_index + 1,
                    s.files.len(),
                    msg
                )));
                s.current_index += 1;
                s.has_error = true;
            }
        }
    }

    // 所有文件处理完毕
    let final_result = if s.has_error {
        Err(ExtractError::ExtractFailed("部分文件解压失败".to_string()))
    } else {
        Ok(())
    };
    let _ = s.msg_tx.send(WorkerMessage::Finished(final_result));

    // 重置状态
    *state = None;
}

// ---------------------------------------------------------------------------
// 目录扫描
// ---------------------------------------------------------------------------

/// 递归扫描目录，收集所有文件
///
/// - `max_depth=0` 表示不限制深度
/// - `cur_depth` 从 1 开始，递归时递增
fn scan_directory(
    dir: &Path,
    max_depth: u32,
    cur_depth: u32,
    results: &mut Vec<PathBuf>,
    msg_tx: &mpsc::Sender<WorkerMessage>,
) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) => {
            let _ = msg_tx.send(WorkerMessage::Log(format!(
                "⚠ 无法读取目录 '{}': {}",
                dir.display(),
                e
            )));
            return;
        }
    };

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let entry_path = entry.path();

        if entry_path.is_file() {
            results.push(entry_path);
        } else if entry_path.is_dir() && (max_depth == 0 || cur_depth < max_depth) {
            scan_directory(&entry_path, max_depth, cur_depth + 1, results, msg_tx);
        }
    }
}
