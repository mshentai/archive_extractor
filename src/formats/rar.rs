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

    // extract_to() 返回 OpenArchive（迭代器），
    // 必须通过 .process() 或迭代来触发实际的 RARProcessFile 调用，
    // 否则直接 Ok(_) 丢弃 OpenArchive 会导致没有文件被解压。
    match archive.extract_to(dest_str) {
        Ok(mut open_archive) => match open_archive.process() {
            Ok(entries) => {
                println!(
                    "RAR 解压完成: {} ({} 个条目)",
                    dest.display(),
                    entries.len()
                );
            }
            Err(e) => {
                let hint = if password.is_none() {
                    "（可能已加密，请使用 -p/--password 提供密码）"
                } else {
                    "（密码错误？）"
                };
                // 即使 process() 返回错误，部分文件可能已成功解压
                let extracted_count = e.data.as_ref().map_or(0, |v| v.len());
                if extracted_count > 0 {
                    println!(
                        "RAR 部分解压: {} (成功 {} 个条目) {}",
                        dest.display(),
                        extracted_count,
                        hint
                    );
                } else {
                    eprintln!("解压 RAR 文件 {} 失败: {} {}", path.display(), e, hint);
                }
            }
        },
        Err(e) => {
            let hint = if password.is_none() {
                "（可能已加密，请使用 -p/--password 提供密码）"
            } else {
                "（密码错误？）"
            };
            eprintln!("打开 RAR 文件 {} 失败: {} {}", path.display(), e, hint);
        }
    }
}
