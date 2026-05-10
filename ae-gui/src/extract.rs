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
    Progress { current: usize, total: usize },
    Finished(Result<(), ExtractError>),
    Log(String),
}

/// UI 线程 → 工作线程的命令
#[derive(Debug)]
pub enum WorkerCommand {
    Extract {
        path: PathBuf,
        password: Option<String>,
    },
}

// ---------------------------------------------------------------------------
// 工作线程管理
// ---------------------------------------------------------------------------

/// 启动后台解压工作线程，返回命令发送端和消息接收端
pub fn spawn_worker() -> (mpsc::Sender<WorkerCommand>, mpsc::Receiver<WorkerMessage>) {
    let (cmd_tx, cmd_rx) = mpsc::channel::<WorkerCommand>();
    let (msg_tx, msg_rx) = mpsc::channel::<WorkerMessage>();

    thread::spawn(move || {
        while let Ok(cmd) = cmd_rx.recv() {
            match cmd {
                WorkerCommand::Extract { path, password } => {
                    handle_extract(path, password, &msg_tx);
                }
            }
        }
    });

    (cmd_tx, msg_rx)
}

// ---------------------------------------------------------------------------
// 核心处理逻辑
// ---------------------------------------------------------------------------

/// 处理解压命令：文件直接解压，目录则递归扫描后逐个解压
fn handle_extract(path: PathBuf, password: Option<String>, msg_tx: &mpsc::Sender<WorkerMessage>) {
    // 1. 收集所有待解压的文件路径
    let paths: Vec<PathBuf> = if path.is_file() {
        vec![path.clone()]
    } else if path.is_dir() {
        let mut files = Vec::new();
        scan_directory(&path, 0, 1, &mut files, msg_tx);
        files
    } else {
        vec![]
    };

    if paths.is_empty() {
        let _ = msg_tx.send(WorkerMessage::Log("⚠ 没有找到可解压的文件".to_string()));
        let _ = msg_tx.send(WorkerMessage::Finished(Ok(())));
        return;
    }

    // 2. 逐个解压
    let total = paths.len();
    let mut has_error = false;

    for (i, file_path) in paths.iter().enumerate() {
        let _ = msg_tx.send(WorkerMessage::Progress {
            current: i + 1,
            total,
        });
        let _ = msg_tx.send(WorkerMessage::Log(format!(
            "[{}/{}] 正在解压: {}",
            i + 1,
            total,
            file_path.display()
        )));

        let result = archive_extractor::extract(file_path, password.as_deref());

        match &result {
            Ok(()) => {
                let _ = msg_tx.send(WorkerMessage::Log(format!(
                    "✓ [{}/{}] 解压完成: {}",
                    i + 1,
                    total,
                    file_path.display()
                )));
            }
            Err(ExtractError::PasswordRequired) => {
                let _ = msg_tx.send(WorkerMessage::Log(format!(
                    "🔒 [{}/{}] 需要密码，跳过: {}",
                    i + 1,
                    total,
                    file_path.display()
                )));
                has_error = true;
            }
            Err(ExtractError::ExtractFailed(msg)) => {
                let _ = msg_tx.send(WorkerMessage::Log(format!(
                    "✗ [{}/{}] 解压失败: {}",
                    i + 1,
                    total,
                    msg
                )));
                has_error = true;
            }
        }
    }

    let final_result = if has_error {
        Err(ExtractError::ExtractFailed("部分文件解压失败".to_string()))
    } else {
        Ok(())
    };
    let _ = msg_tx.send(WorkerMessage::Finished(final_result));
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
