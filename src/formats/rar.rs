use std::fs;
use std::path::Path;

use crate::ExtractError;

/// 解压 RAR 文件
pub(crate) fn extract_rar(
    path: &Path,
    dest: &Path,
    password: Option<&str>,
) -> Result<(), ExtractError> {
    println!("正在解压 RAR: {} -> {}", path.display(), dest.display());

    // unrar 的底层 C 库要求目标目录已存在，否则解压会静默失败
    fs::create_dir_all(dest)
        .map_err(|e| ExtractError::ExtractFailed(format!("创建目标目录失败: {}", e)))?;

    let path_str = path.to_string_lossy().into_owned();
    let dest_str = dest.to_string_lossy().into_owned();

    let archive = match password {
        Some(pwd) => unrar::Archive::with_password(path_str, pwd.to_string()),
        None => unrar::Archive::new(path_str),
    };

    // extract_to() 返回 OpenArchive，必须通过 .process() 触发实际解压
    match archive.extract_to(dest_str) {
        Ok(mut open_archive) => match open_archive.process() {
            Ok(entries) => {
                println!(
                    "RAR 解压完成: {} ({} 个条目)",
                    dest.display(),
                    entries.len()
                );
                Ok(())
            }
            Err(e) => {
                let err_msg = e.to_string();
                // 无密码且遇到加密相关错误
                if password.is_none() && is_rar_password_error(&err_msg) {
                    // 检查是否有部分文件被解压
                    let extracted_count = e.data.as_ref().map_or(0, |v| v.len());
                    if extracted_count > 0 {
                        println!(
                            "RAR 部分解压: {} (成功 {} 个条目)",
                            dest.display(),
                            extracted_count
                        );
                    }
                    Err(ExtractError::PasswordRequired)
                } else {
                    Err(ExtractError::ExtractFailed(format!(
                        "解压 RAR 文件失败: {}",
                        err_msg
                    )))
                }
            }
        },
        Err(e) => {
            let err_msg = e.to_string();
            if password.is_none() && is_rar_password_error(&err_msg) {
                Err(ExtractError::PasswordRequired)
            } else {
                Err(ExtractError::ExtractFailed(format!(
                    "打开 RAR 文件失败: {}",
                    err_msg
                )))
            }
        }
    }
}

/// 判断 RAR 错误信息是否与密码相关
fn is_rar_password_error(msg: &str) -> bool {
    let keywords = [
        "password",
        "encrypted",
        "Missing password",
        "Wrong password",
        "ERR_MISSING_PASSWORD",
        "ERR_WRONG_PASSWORD",
        "encryption",
    ];
    let lower = msg.to_lowercase();
    keywords.iter().any(|k| lower.contains(&k.to_lowercase()))
}
