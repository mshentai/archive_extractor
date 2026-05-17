mod app;
mod context_menu;
mod extract;

use std::path::PathBuf;
use std::sync::Arc;

use eframe::egui::{FontDefinitions, FontFamily, ViewportBuilder};

fn main() -> Result<(), eframe::Error> {
    // 解析命令行参数
    let args: Vec<String> = std::env::args().collect();
    let mut silent_file: Option<PathBuf> = None;
    let mut flat = false;

    // 跳过第一个参数（程序路径），解析剩余参数
    for arg in args.iter().skip(1) {
        if arg == "--flat" || arg == "-f" {
            flat = true;
        } else {
            let path = PathBuf::from(arg);
            if path.exists() {
                silent_file = Some(path);
            } else {
                eprintln!("文件不存在: {}", arg);
            }
        }
    }

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
        Box::new(|cc| {
            // ================================================================
            // 配置中文字体（防止 egui 默认字体不含 CJK 导致乱码）
            // ================================================================
            let mut fonts = FontDefinitions::default();

            // 尝试从 Windows 系统字体目录加载中文字体
            // 必须将实际字体数据注册到 font_data 中，才能在 families 中引用
            let font_candidates: &[(&str, &str)] = &[
                ("msyh", r"C:\Windows\Fonts\msyh.ttc"),     // 微软雅黑
                ("msyhbd", r"C:\Windows\Fonts\msyhbd.ttc"), // 微软雅黑 Bold
                ("simhei", r"C:\Windows\Fonts\SIMHEI.TTF"), // 黑体
                ("simsun", r"C:\Windows\Fonts\SIMSUN.TTC"), // 宋体
            ];

            let mut loaded_any = false;
            for (name, path) in font_candidates {
                if let Ok(data) = std::fs::read(path) {
                    fonts
                        .font_data
                        .insert(name.to_string(), Arc::new(egui::FontData::from_owned(data)));
                    fonts
                        .families
                        .entry(FontFamily::Proportional)
                        .or_default()
                        .insert(0, name.to_string());
                    fonts
                        .families
                        .entry(FontFamily::Monospace)
                        .or_default()
                        .insert(0, name.to_string());
                    loaded_any = true;
                    break; // 找到一个可用的字体就够
                }
            }

            if !loaded_any {
                eprintln!("警告: 未能加载任何中文字体，中文可能显示为乱码");
            }

            cc.egui_ctx.set_fonts(fonts);
            // ================================================================

            let mut gui_app = app::AeGuiApp::new(silent_file, flat);
            if auto_start {
                gui_app.set_auto_start();
            }
            Ok(Box::new(gui_app))
        }),
    )
}
