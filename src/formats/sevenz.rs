use std::path::Path;

/// 解压 7z 文件
pub(crate) fn extract_7z(path: &Path, dest: &Path, password: Option<&str>) {
    println!("正在解压 7z: {} -> {}", path.display(), dest.display());

    let result = match password {
        Some(pwd) => {
            let pwd_obj = sevenz_rust::Password::from(pwd);
            sevenz_rust::decompress_file_with_password(path, dest, pwd_obj)
        }
        None => sevenz_rust::decompress_file(path, dest),
    };

    match result {
        Ok(_) => println!("7z 解压完成: {}", dest.display()),
        Err(e) => {
            let hint = if password.is_none() {
                "（可能已加密，请使用 -p/--password 提供密码）"
            } else {
                "（密码错误？）"
            };
            eprintln!("解压 7z 文件 {} 失败: {} {}", path.display(), e, hint);
        }
    }
}
