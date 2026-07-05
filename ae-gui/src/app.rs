use std::path::PathBuf;
use std::sync::mpsc;

use eframe::egui::{self, Button, Color32, Frame, Margin, RichText};

use archive_extractor::ExtractError;

use crate::context_menu;
use crate::extract::{self, WorkerCommand, WorkerMessage};

// ---------------------------------------------------------------------------
// 应用状态
// ---------------------------------------------------------------------------

pub struct AeGuiApp {
    // --- 文件路径 ---
    file_path: Option<PathBuf>,

    // --- 解压模式 ---
    flat: bool,

    // --- 解压状态 ---
    is_extracting: bool,
    status_text: String,
    log_messages: Vec<String>,

    // --- 静默模式（右键菜单启动） ---
    silent_file: Option<PathBuf>,
    should_start_extract: bool,

    // --- 密码弹窗 ---
    show_password_dialog: bool,
    password_input: String,
    password_error: Option<String>,
    pending_path: Option<PathBuf>,

    // --- 工作线程通信 ---
    cmd_tx: Option<mpsc::Sender<WorkerCommand>>,
    msg_rx: Option<mpsc::Receiver<WorkerMessage>>,

    // --- 右键菜单状态 ---
    context_menu_registered: bool,
    context_menu_flat_registered: bool,
    context_menu_message: Option<String>,

    // --- 程序退出 ---
    should_exit: bool,
}

impl AeGuiApp {
    /// 创建新应用实例
    ///
    /// `silent_file` — 从右键菜单启动时传入的文件路径
    /// `flat` — 是否使用平铺模式解压
    pub fn new(silent_file: Option<PathBuf>, flat: bool) -> Self {
        let (cmd_tx, msg_rx) = extract::spawn_worker();

        Self {
            file_path: silent_file.clone(),
            flat,
            is_extracting: false,
            status_text: String::new(),
            log_messages: Vec::new(),
            silent_file,
            should_start_extract: false,
            show_password_dialog: false,
            password_input: String::new(),
            password_error: None,
            pending_path: None,
            cmd_tx: Some(cmd_tx),
            msg_rx: Some(msg_rx),
            context_menu_registered: context_menu::is_registered(),
            context_menu_flat_registered: context_menu::is_flat_registered(),
            context_menu_message: None,
            should_exit: false,
        }
    }

    /// 设置为静默模式自动开始解压
    pub fn set_auto_start(&mut self) {
        self.should_start_extract = true;
    }

    /// 启动解压
    fn start_extract(&mut self, path: PathBuf, ctx: &egui::Context) {
        self.is_extracting = true;
        let mode = if self.flat {
            "平铺模式"
        } else {
            "标准模式"
        };
        self.status_text = format!("{} 正在解压: {} ...", mode, path.display());
        self.log_messages.clear();
        self.password_error = None;

        if let Some(tx) = &self.cmd_tx {
            let _ = tx.send(WorkerCommand::Extract {
                path,
                flat: self.flat,
                ctx: ctx.clone(),
            });
        }
    }

    /// 处理来自工作线程的消息
    fn poll_worker(&mut self) {
        while let Ok(msg) = self.msg_rx.as_ref().unwrap().try_recv() {
            match msg {
                WorkerMessage::Log(text) => {
                    self.log_messages.push(text);
                }
                WorkerMessage::Finished(result) => {
                    self.is_extracting = false;
                    match result {
                        Ok(()) => {
                            self.status_text = "✅ 解压完成！".to_string();
                            // 静默模式下解压成功后退出
                            if self.silent_file.is_some() {
                                self.should_exit = true;
                            }
                        }
                        Err(ExtractError::ExtractFailed(msg)) => {
                            self.status_text = format!("❌ 解压失败: {}", msg);
                        }
                        // PasswordRequired / WrongPassword 现在通过独立消息处理，不再走 Finished
                        _ => {}
                    }
                }
                WorkerMessage::Progress { current, total } => {
                    self.status_text = format!("解压进度: {}/{}", current, total);
                }
                WorkerMessage::PasswordRequired {
                    file_path,
                    current,
                    total,
                    is_wrong_password,
                } => {
                    self.status_text = format!("🔒 [{}/{}] 需要密码", current, total);
                    self.pending_path = Some(file_path);
                    self.show_password_dialog = true;
                    self.password_input.clear();
                    if is_wrong_password {
                        self.password_error = Some("密码错误，请重新输入".to_string());
                    } else {
                        self.password_error = None;
                    }
                }
            }
        }
    }

    /// 处理密码提交
    fn submit_password(&mut self) {
        let pwd = self.password_input.trim().to_string();
        if pwd.is_empty() {
            self.password_error = Some("密码不能为空".to_string());
            return;
        }
        self.show_password_dialog = false;
        self.password_error = None;
        // 将密码发送给工作线程，无需重新启动解压
        if let Some(tx) = &self.cmd_tx {
            let _ = tx.send(WorkerCommand::ProvidePassword(pwd));
        }
    }

    /// 选择文件
    fn pick_file(&mut self) {
        if let Some(files) = rfd::FileDialog::new()
            .add_filter("压缩文件", &["zip", "7z", "rar"])
            .pick_files()
        {
            if let Some(file) = files.first() {
                self.file_path = Some(file.clone());
                self.status_text = format!("已选择: {}", file.display());
            }
        }
    }

    /// 选择目录
    fn pick_directory(&mut self) {
        if let Some(dir) = rfd::FileDialog::new().pick_folder() {
            self.file_path = Some(dir.clone());
            self.status_text = format!("已选择目录: {}", dir.display());
        }
    }
}

impl eframe::App for AeGuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // --- 轮询工作线程消息 ---
        self.poll_worker();

        // --- 静默模式下解压成功后自动关闭窗口 ---
        if self.should_exit {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        // --- 静默模式下自动开始解压（仅在第一次 update 时） ---
        if self.should_start_extract {
            self.should_start_extract = false;
            if let Some(path) = self.silent_file.clone() {
                self.start_extract(path, ctx);
            }
        }

        // =====================================================================
        // 主界面布局
        // =====================================================================

        egui::CentralPanel::default().show(ctx, |ui| {
            // 标题
            ui.heading("📦 AE - Archive Extractor");
            ui.separator();
            ui.add_space(8.0);

            // =================================================================
            // 文件选择区域
            // =================================================================
            ui.horizontal(|ui| {
                if ui.button("📂 选择文件").clicked() {
                    self.pick_file();
                }
                if ui.button("📁 选择目录").clicked() {
                    self.pick_directory();
                }

                // 当前文件路径显示
                if let Some(path) = &self.file_path {
                    ui.label(path.to_string_lossy().as_ref());
                }
            });

            ui.add_space(4.0);

            // =================================================================
            // 解压模式切换 + 操作按钮
            // =================================================================
            ui.horizontal(|ui| {
                // 平铺模式复选框
                ui.checkbox(&mut self.flat, "📂 平铺模式（直接解压到当前目录）");
            });

            ui.horizontal(|ui| {
                let can_extract = self.file_path.is_some() && !self.is_extracting;
                let extract_btn = Button::new(RichText::new("🚀 解压").size(16.0));
                let response = if can_extract {
                    ui.add_enabled(true, extract_btn)
                } else {
                    ui.add_enabled(false, extract_btn)
                };
                if response.clicked() {
                    if let Some(path) = self.file_path.clone() {
                        self.start_extract(path, ctx);
                    }
                }

                // 状态文字
                ui.label(&self.status_text);
            });

            // 显示当前模式提示
            if self.flat {
                ui.colored_label(
                    Color32::LIGHT_BLUE,
                    "💡 平铺模式：压缩包内容将直接解压到当前目录，不创建子文件夹",
                );
            }

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(4.0);

            // =================================================================
            // 日志信息区域
            // =================================================================
            Frame::group(ui.style())
                .inner_margin(Margin::symmetric(8, 4))
                .show(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .max_height(200.0)
                        .auto_shrink([false, true])
                        .show(ui, |ui| {
                            for msg in &self.log_messages {
                                ui.label(msg);
                            }
                            if self.log_messages.is_empty() {
                                ui.colored_label(
                                    Color32::GRAY,
                                    "暂无日志，选择文件后点击「解压」开始",
                                );
                            }
                        });
                });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(4.0);

            // =================================================================
            // 右键菜单管理
            // =================================================================
            ui.heading("⚙ 右键菜单管理");

            // --- 普通模式 ---
            ui.horizontal(|ui| {
                ui.label("标准模式: ");
                let registered = self.context_menu_registered;

                let register_btn = Button::new("📝 注册");
                if registered {
                    ui.add_enabled(false, register_btn);
                } else {
                    if ui.add(register_btn).clicked() {
                        let exe_path = std::env::current_exe().unwrap_or_default();
                        match context_menu::register(&exe_path) {
                            Ok(()) => {
                                self.context_menu_registered = true;
                                self.context_menu_message =
                                    Some("✅ 标准模式右键菜单注册成功".to_string());
                            }
                            Err(e) => {
                                self.context_menu_message = Some(format!("❌ 注册失败: {}", e));
                            }
                        }
                    }
                }

                if registered {
                    ui.colored_label(Color32::GREEN, "● 已注册");
                } else {
                    ui.colored_label(Color32::GRAY, "○ 未注册");
                }
            });

            // --- 平铺模式 ---
            ui.horizontal(|ui| {
                ui.label("平铺模式: ");
                let flat_registered = self.context_menu_flat_registered;

                let register_btn = Button::new("📝 注册");
                if flat_registered {
                    ui.add_enabled(false, register_btn);
                } else {
                    if ui.add(register_btn).clicked() {
                        let exe_path = std::env::current_exe().unwrap_or_default();
                        match context_menu::register_flat(&exe_path) {
                            Ok(()) => {
                                self.context_menu_flat_registered = true;
                                self.context_menu_message =
                                    Some("✅ 平铺模式右键菜单注册成功".to_string());
                            }
                            Err(e) => {
                                self.context_menu_message = Some(format!("❌ 注册失败: {}", e));
                            }
                        }
                    }
                }

                if flat_registered {
                    ui.colored_label(Color32::GREEN, "● 已注册");
                } else {
                    ui.colored_label(Color32::GRAY, "○ 未注册");
                }
            });

            // --- 取消注册 ---
            ui.horizontal(|ui| {
                let any_registered =
                    self.context_menu_registered || self.context_menu_flat_registered;
                let unregister_btn = Button::new("🗑 取消注册全部");
                if any_registered {
                    if ui.add(unregister_btn).clicked() {
                        match context_menu::unregister() {
                            Ok(()) => {
                                self.context_menu_registered = false;
                                self.context_menu_flat_registered = false;
                                self.context_menu_message =
                                    Some("✅ 所有右键菜单已取消注册".to_string());
                            }
                            Err(e) => {
                                self.context_menu_message = Some(format!("❌ 取消注册失败: {}", e));
                            }
                        }
                    }
                } else {
                    ui.add_enabled(false, unregister_btn);
                }
            });

            // 显示右键菜单状态消息
            if let Some(msg) = &self.context_menu_message {
                ui.label(msg);
            }
        });

        // =====================================================================
        // 密码弹窗（模态对话框）
        // =====================================================================
        if self.show_password_dialog {
            // 创建一个模态窗口
            egui::Window::new("🔑 输入密码")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label("此文件已加密，请输入解压密码：");
                    ui.add_space(4.0);

                    // 显示文件名
                    if let Some(path) = &self.pending_path {
                        ui.colored_label(
                            Color32::LIGHT_BLUE,
                            format!(
                                "文件: {}",
                                path.file_name().unwrap_or_default().to_string_lossy()
                            ),
                        );
                    }
                    ui.add_space(8.0);

                    // 在 TextEdit 之前拦截 Enter 键（TextEdit 会消费掉 Enter 事件）
                    if ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter)) {
                        self.submit_password();
                    }

                    // 密码输入框（明文）
                    let response = ui.add_sized(
                        [250.0, 20.0],
                        egui::TextEdit::singleline(&mut self.password_input)
                            .hint_text("请输入密码"),
                    );
                    // 自动聚焦到密码输入框
                    response.request_focus();

                    // 密码错误提示
                    if let Some(err) = &self.password_error {
                        ui.colored_label(Color32::RED, err);
                    }

                    ui.add_space(8.0);

                    ui.horizontal(|ui| {
                        if ui.button("确定").clicked() {
                            self.submit_password();
                        }
                        if ui.button("取消").clicked()
                            || ui.input(|i| i.key_pressed(egui::Key::Escape))
                        {
                            self.show_password_dialog = false;
                            self.password_error = None;
                            self.pending_path = None;
                            if let Some(tx) = &self.cmd_tx {
                                let _ = tx.send(WorkerCommand::Cancel);
                            }
                            self.status_text = "已取消解压".to_string();
                        }
                    });
                });

            // 请求焦点，使密码对话框获得输入焦点
            ctx.request_repaint();
        }
    }
}
