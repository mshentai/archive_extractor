use std::path::Path;

use crate::ExtractError;

/// 解压 7z 文件
pub(crate) fn extract_7z(
    path: &Path,
    dest: &Path,
    password: Option<&str>,
) -> Result<(), ExtractError> {
    println!("正在解压 7z: {} -> {}", path.display(), dest.display());

    let result = match password {
        Some(pwd) => {
            let pwd_obj = sevenz_rust::Password::from(pwd);
            sevenz_rust::decompress_file_with_password(path, dest, pwd_obj)
        }
        None => sevenz_rust::decompress_file(path, dest),
    };

    match result {
        Ok(_) => {
            println!("7z 解压完成: {}", dest.display());
            Ok(())
        }
        Err(e) => {
            let err_msg = e.to_string();
            // 判断是否为密码相关错误
            if is_7z_password_error(&err_msg) {
                if password.is_some() {
                    Err(ExtractError::WrongPassword)
                } else {
                    Err(ExtractError::PasswordRequired)
                }
            } else {
                Err(ExtractError::ExtractFailed(format!(
                    "解压 7z 文件失败: {}",
                    err_msg
                )))
            }
        }
    }
}

/// 判断 7z 错误信息是否与密码相关
fn is_7z_password_error(msg: &str) -> bool {
    let keywords = [
        "encrypted",
        "password",
        "Wrong password",
        "Error as unknown",
        "Decoder error",
        "k_wrongPassword",
        "crypto",
    ];
    let lower = msg.to_lowercase();
    keywords.iter().any(|k| lower.contains(&k.to_lowercase()))
}
