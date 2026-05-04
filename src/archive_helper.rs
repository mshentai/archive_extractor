use std::fs;
use std::io::Cursor;
use std::path::Path;

/// 解压到默认目录（与压缩包同目录，同名文件夹）
pub fn extract(path: &Path) {
    let dest = default_dest(path);
    extract_impl(path, &dest);
}

/// 解压到指定目录
pub fn extract_to(path: &Path, dest: &Path) {
    // 确保目标目录存在
    if let Err(e) = fs::create_dir_all(dest) {
        eprintln!("创建目标目录 {} 失败: {}", dest.display(), e);
        return;
    }
    extract_impl(path, dest);
}

/// 内部共享实现
fn extract_impl(path: &Path, dest: &Path) {
    // 1. 读取文件到缓冲区
    let data = match fs::read(path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("无法读取文件 {}: {}", path.display(), e);
            return;
        }
    };

    // 2. 用 infer 检测类型
    let kind = match infer::get(&data) {
        Some(k) => k,
        None => {
            println!("未知文件类型，跳过: {}", path.display());
            return;
        }
    };

    // 3. 根据 MIME 类型分发
    match kind.mime_type() {
        "application/zip" => extract_zip(path, &data, dest),
        "application/x-7z-compressed" => extract_7z(path, dest),
        "application/vnd.rar" => extract_rar(path, dest),
        _ => println!(
            "不是支持的压缩格式 ({}), 跳过: {}",
            kind.mime_type(),
            path.display()
        ),
    }
}

/// 获取默认解压目录（与压缩包同名，不含扩展名）
fn default_dest(path: &Path) -> std::path::PathBuf {
    let stem = path.file_stem().unwrap_or_default();
    path.parent().unwrap_or(Path::new(".")).join(stem)
}

/// 解压 ZIP 文件
fn extract_zip(path: &Path, data: &[u8], dest: &Path) {
    println!("正在解压 ZIP: {} -> {}", path.display(), dest.display());

    let mut archive = match zip::ZipArchive::new(Cursor::new(data)) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("无法打开 ZIP 文件 {}: {}", path.display(), e);
            return;
        }
    };

    for i in 0..archive.len() {
        let mut entry = match archive.by_index(i) {
            Ok(e) => e,
            Err(e) => {
                eprintln!("  读取 ZIP 条目 #{} 失败: {}", i, e);
                continue;
            }
        };

        // 安全性：跳过包含路径遍历的条目
        let entry_path = entry.name().replace('\\', "/");
        if entry_path.contains("..") {
            eprintln!("  跳过不安全的路径: {}", entry_path);
            continue;
        }

        let out_path = dest.join(&entry_path);

        if entry.is_dir() {
            if let Err(e) = fs::create_dir_all(&out_path) {
                eprintln!("  创建目录 {} 失败: {}", out_path.display(), e);
            }
        } else {
            // 确保父目录存在
            if let Some(parent) = out_path.parent() {
                if !parent.exists() {
                    if let Err(e) = fs::create_dir_all(parent) {
                        eprintln!("  创建目录 {} 失败: {}", parent.display(), e);
                        continue;
                    }
                }
            }

            if let Err(e) = write_entry(&mut entry, &out_path) {
                eprintln!("  写入文件 {} 失败: {}", out_path.display(), e);
            }
        }
    }

    println!("ZIP 解压完成: {}", dest.display());
}

/// 将 ZIP 条目写入磁盘
fn write_entry<R: std::io::Read>(reader: &mut R, out_path: &Path) -> std::io::Result<()> {
    let mut out_file = fs::File::create(out_path)?;
    std::io::copy(reader, &mut out_file)?;
    Ok(())
}

/// 解压 7z 文件
fn extract_7z(path: &Path, dest: &Path) {
    println!("正在解压 7z: {} -> {}", path.display(), dest.display());

    match sevenz_rust::decompress_file(path, dest) {
        Ok(_) => println!("7z 解压完成: {}", dest.display()),
        Err(e) => eprintln!("解压 7z 文件 {} 失败: {}", path.display(), e),
    }
}

/// 解压 RAR 文件
fn extract_rar(path: &Path, dest: &Path) {
    let path_str = path.to_string_lossy().into_owned();
    let dest_str = dest.to_string_lossy().into_owned();
    println!("正在解压 RAR: {} -> {}", path.display(), dest.display());

    // unrar 的底层 C 库要求目标目录已存在，否则解压会静默失败
    if let Err(e) = fs::create_dir_all(dest) {
        eprintln!("创建目标目录 {} 失败: {}", dest.display(), e);
        return;
    }

    match unrar::Archive::new(path_str).extract_to(dest_str) {
        Ok(_) => println!("RAR 解压完成: {}", dest.display()),
        Err(e) => eprintln!("解压 RAR 文件 {} 失败: {}", path.display(), e),
    }
}
