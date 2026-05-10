mod app;
mod context_menu;
mod extract;

use std::path::PathBuf;

use eframe::egui::ViewportBuilder;

fn main() -> Result<(), eframe::Error> {
    // 解析命令行参数
    let args: Vec<String> = std::env::args().collect();
    let silent_file: Option<PathBuf> = if args.len() > 1 {
        let path = PathBuf::from(&args[1]);
        if path.exists() {
            Some(path)
        } else {
            eprintln!("文件不存在: {}", args[1]);
            None
        }
    } else {
        None
    };

    // 静默模式下自动开始解压标记
    let auto_start = silent_file.is_some();

    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([600.0, 480.0])
            .with_min_inner_size([400.0, 300.0])
            .with_title("AE - Archive Extractor"),
        ..Default::default()
    };

    eframe::run_native(
        "AE - Archive Extractor",
        options,
        Box::new(|_cc| {
            let mut gui_app = app::AeGuiApp::new(silent_file);
            if auto_start {
                gui_app.set_auto_start();
            }
            Ok(Box::new(gui_app))
        }),
    )
}
