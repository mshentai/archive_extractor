use std::fs;
use std::path::{Path, PathBuf};

use clap::Parser;

use archive_extractor::extract;
use archive_extractor::extract_to;

// ---------------------------------------------------------------------------
// CLI 参数定义
// ---------------------------------------------------------------------------

/// 多功能解压工具 — 支持 ZIP / 7z / RAR 格式
#[derive(Parser, Debug)]
#[command(name = "archive_extractor", version, about)]
struct Cli {
    /// 压缩文件路径 / 目录路径 / 列表文件路径（当使用 --list 时）
    path: PathBuf,

    /// 将 PATH 视为路径列表文件（每行一个路径）
    #[arg(short = 'l', long)]
    list: bool,

    /// 目录扫描深度（仅扫描目录本身时填 1，递归子目录叠加）
    #[arg(short = 'd', long, default_value = "1")]
    depth: u32,

    /// 可选的输出根目录；不指定时解压到压缩文件同级目录下
    #[arg(short = 'o', long)]
    output: Option<PathBuf>,

    /// 平铺模式：跳过 <文件名>/ 子目录，直接解压到输出根目录
    #[arg(short = 'f', long)]
    flat: bool,

    /// 解压密码（适用于加密的压缩包）
    #[arg(short = 'p', long)]
    password: Option<String>,
}

// ---------------------------------------------------------------------------
// 入口
// ---------------------------------------------------------------------------

fn main() {
    let args = Cli::parse();

    // 1. 解析输入 → 得到路径列表
    let paths = match resolve_inputs(&args) {
        Ok(list) => list,
        Err(e) => {
            eprintln!("错误: {}", e);
            std::process::exit(1);
        }
    };

    if paths.is_empty() {
        println!("没有待处理的路径。");
        return;
    }

    // 2. 遍历处理每个路径
    let output_dir = args.output.as_deref();
    let flat = args.flat;
    let password = args.password.as_deref();
    let mut has_error = false;
    for path in &paths {
        if let Err(e) = process_path(path, args.depth, output_dir, flat, password) {
            eprintln!("处理失败 [{}]: {}", path.display(), e);
            has_error = true;
        }
    }

    if has_error {
        std::process::exit(1);
    }
}

// ---------------------------------------------------------------------------
// 输入解析
// ---------------------------------------------------------------------------

/// 将 CLI 参数解析为一个路径列表
fn resolve_inputs(args: &Cli) -> Result<Vec<PathBuf>, String> {
    if args.list {
        // 读取列表文件，每行一个路径
        let content = fs::read_to_string(&args.path)
            .map_err(|e| format!("无法读取列表文件 '{}': {}", args.path.display(), e))?;

        let paths: Vec<PathBuf> = content
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .map(PathBuf::from)
            .collect();

        if paths.is_empty() {
            return Err(format!(
                "列表文件 '{}' 中没有有效的路径条目",
                args.path.display()
            ));
        }

        Ok(paths)
    } else {
        // 单路径，包装为 Vec
        Ok(vec![args.path.clone()])
    }
}

// ---------------------------------------------------------------------------
// 路径处理
// ---------------------------------------------------------------------------

/// 处理单个路径：文件直接解压，目录递归扫描
fn process_path(
    path: &Path,
    depth: u32,
    output_dir: Option<&Path>,
    flat: bool,
    password: Option<&str>,
) -> Result<(), archive_extractor::ExtractError> {
    if !path.exists() {
        eprintln!("路径不存在，跳过: {}", path.display());
        return Ok(());
    }

    if path.is_file() {
        // 直接解压该文件
        extract_single(path, output_dir, flat, password)
    } else if path.is_dir() {
        // 扫描目录，收集压缩文件
        let mut archives = Vec::new();
        scan_directory(path, depth, 1, &mut archives);

        if archives.is_empty() {
            println!("目录 '{}' 中没有找到压缩文件。", path.display());
            return Ok(());
        }

        println!(
            "目录 '{}' 中找到 {} 个压缩文件，开始解压...",
            path.display(),
            archives.len()
        );
        for archive in &archives {
            extract_single(archive, output_dir, flat, password)?;
        }
        Ok(())
    } else {
        Ok(())
    }
}

/// 解压单个文件（可指定输出基目录和密码）
///
/// `flat=false`（默认）：输出到 `<root>/<file_stem>/`
/// `flat=true`：输出到 `<root>/`（跳过 file_stem 子目录）
fn extract_single(
    path: &Path,
    output_dir: Option<&Path>,
    flat: bool,
    password: Option<&str>,
) -> Result<(), archive_extractor::ExtractError> {
    match (output_dir, flat) {
        // 无 -o + 无 --flat（默认）：解压到压缩包同级目录下的同名子目录
        (None, false) => extract(path, password),
        // 无 -o + --flat：直接解压到压缩包所在目录
        (None, true) => {
            let dest = path.parent().unwrap_or(Path::new("."));
            extract_to(path, dest, password)
        }
        // 有 -o + 无 --flat（默认）：解压到 <OUTPUT>/<file_stem>/
        (Some(base), false) => {
            let stem = path.file_stem().unwrap_or_default();
            let dest = base.join(stem);
            extract_to(path, &dest, password)
        }
        // 有 -o + --flat：直接解压到 <OUTPUT>/
        (Some(base), true) => extract_to(path, base, password),
    }
}

// ---------------------------------------------------------------------------
// 目录扫描
// ---------------------------------------------------------------------------

/// 按指定深度扫描目录，收集所有文件路径
fn scan_directory(dir: &Path, max_depth: u32, cur_depth: u32, results: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("无法读取目录 '{}': {}", dir.display(), e);
            return;
        }
    };

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                eprintln!("  读取目录条目失败: {}", e);
                continue;
            }
        };

        let entry_path = entry.path();

        if entry_path.is_file() {
            results.push(entry_path);
        } else if entry_path.is_dir() && cur_depth < max_depth {
            // 递归扫描子目录
            scan_directory(&entry_path, max_depth, cur_depth + 1, results);
        }
    }
}
