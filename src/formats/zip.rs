use std::fs;
use std::io::Cursor;
use std::path::Path;

use crate::path_utils::ensure_parent_dir;

/// 解压 ZIP 文件
pub(crate) fn extract_zip(path: &Path, data: &[u8], dest: &Path, password: Option<&str>) {
    println!("正在解压 ZIP: {} -> {}", path.display(), dest.display());

    let mut archive = match zip::ZipArchive::new(Cursor::new(data)) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("无法打开 ZIP 文件 {}: {}", path.display(), e);
            return;
        }
    };

    for i in 0..archive.len() {
        // 尝试读取条目（带密码或不带密码）
        let entry_result = if let Some(pwd) = password {
            archive.by_index_decrypt(i, pwd.as_bytes())
        } else {
            archive.by_index(i)
        };

        // 如果无密码但条目加密，给用户明确提示
        if let Err(ref e) = entry_result {
            if is_password_required_error(e) {
                eprintln!("  条目 #{} 已加密，请使用 -p/--password 提供密码", i);
                continue;
            }
        }

        let mut entry = match entry_result {
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
            if let Err(e) = ensure_parent_dir(&out_path) {
                eprintln!("  创建目录 {} 失败: {}", out_path.display(), e);
                continue;
            }

            if let Err(e) = write_entry(&mut entry, &out_path) {
                eprintln!("  写入文件 {} 失败: {}", out_path.display(), e);
            }
        }
    }

    println!("ZIP 解压完成: {}", dest.display());
}

/// 检查是否为「需要密码」的 ZIP 错误
pub fn is_password_required_error(e: &zip::result::ZipError) -> bool {
    matches!(
        e,
        zip::result::ZipError::UnsupportedArchive(msg)
            if *msg == zip::result::ZipError::PASSWORD_REQUIRED
    )
}

/// 将 ZIP 条目写入磁盘
pub(crate) fn write_entry<R: std::io::Read>(
    reader: &mut R,
    out_path: &Path,
) -> std::io::Result<()> {
    let mut out_file = fs::File::create(out_path)?;
    std::io::copy(reader, &mut out_file)?;
    Ok(())
}
