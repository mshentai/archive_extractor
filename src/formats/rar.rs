use std::fs;
use std::path::Path;

/// 解压 RAR 文件
pub(crate) fn extract_rar(path: &Path, dest: &Path, password: Option<&str>) {
    println!("正在解压 RAR: {} -> {}", path.display(), dest.display());

    // unrar 的底层 C 库要求目标目录已存在，否则解压会静默失败
    if let Err(e) = fs::create_dir_all(dest) {
        eprintln!("创建目标目录 {} 失败: {}", dest.display(), e);
        return;
    }

    let path_str = path.to_string_lossy().into_owned();
    let dest_str = dest.to_string_lossy().into_owned();

    let archive = match password {
        Some(pwd) => unrar::Archive::with_password(path_str, pwd.to_string()),
        None => unrar::Archive::new(path_str),
    };

    match archive.extract_to(dest_str) {
        Ok(_) => println!("RAR 解压完成: {}", dest.display()),
        Err(e) => {
            let hint = if password.is_none() {
                "（可能已加密，请使用 -p/--password 提供密码）"
            } else {
                "（密码错误？）"
            };
            eprintln!("解压 RAR 文件 {} 失败: {} {}", path.display(), e, hint);
        }
    }
}
