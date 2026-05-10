use std::path::PathBuf;
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
                    let _ =
                        msg_tx.send(WorkerMessage::Log(format!("开始解压: {}", path.display())));

                    let result = archive_extractor::extract(&path, password.as_deref());

                    match &result {
                        Ok(()) => {
                            let _ = msg_tx.send(WorkerMessage::Log(format!(
                                "✓ 解压完成: {}",
                                path.display()
                            )));
                        }
                        Err(ExtractError::PasswordRequired) => {
                            let _ = msg_tx
                                .send(WorkerMessage::Log("🔒 文件已加密，需要密码".to_string()));
                        }
                        Err(ExtractError::ExtractFailed(msg)) => {
                            let _ = msg_tx.send(WorkerMessage::Log(format!("✗ 解压失败: {}", msg)));
                        }
                    }

                    let _ = msg_tx.send(WorkerMessage::Finished(result));
                }
            }
        }
    });

    (cmd_tx, msg_rx)
}
