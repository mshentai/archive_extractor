use std::fs;
use std::io::{Read, Seek};
use std::path::Path;

use crate::ExtractError;
use crate::path_utils::ensure_parent_dir;

/// 解压 ZIP 文件（流式读取，无需全量加载到内存）
pub(crate) fn extract_zip<R: Read + Seek>(
    path: &Path,
    reader: R,
    dest: &Path,
    password: Option<&str>,
) -> Result<(), ExtractError> {
    println!("正在解压 ZIP: {} -> {}", path.display(), dest.display());

    let mut archive = zip::ZipArchive::new(reader)
        .map_err(|e| ExtractError::ExtractFailed(format!("无法打开 ZIP 文件: {}", e)))?;

    // 预检查：无密码时尝试读取第一个条目，如果因加密失败则返回 PasswordRequired
    if password.is_none() && archive.len() > 0 {
        match archive.by_index(0) {
            Err(ref e) if is_password_required_error(e) => {
                return Err(ExtractError::PasswordRequired);
            }
            _ => {} // 无加密或其它错误，继续处理
        }
    }

    for i in 0..archive.len() {
        // 尝试读取条目（带密码或不带密码）
        let entry_result = if let Some(pwd) = password {
            archive.by_index_decrypt(i, pwd.as_bytes())
        } else {
            archive.by_index(i)
        };

        // 如果无密码但条目加密，返回密码所需错误
        if let Err(ref e) = entry_result {
            if is_password_required_error(e) {
                return Err(ExtractError::PasswordRequired);
            }
        }

        let mut entry = entry_result.map_err(|e| {
            ExtractError::ExtractFailed(format!("读取 ZIP 条目 #{} 失败: {}", i, e))
        })?;

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
    Ok(())
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
/// 如果目标文件已存在，打印警告
pub(crate) fn write_entry<R: std::io::Read>(
    reader: &mut R,
    out_path: &Path,
) -> std::io::Result<()> {
    if out_path.exists() {
        eprintln!("  警告: 文件已存在，将被覆盖: {}", out_path.display());
    }
    let mut out_file = fs::File::create(out_path)?;
    std::io::copy(reader, &mut out_file)?;
    Ok(())
}
