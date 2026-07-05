use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// 获取默认解压目录（与压缩包同名，不含扩展名）
///
/// 当目标目录与已有文件冲突时（常见于无扩展名的文件，如 `特典`），
/// 自动添加 `_` 后缀 + 循环递增避免冲突：
///   - 第 1 次冲突 → `特典_`
///   - 第 2 次冲突 → `特典_2`
///   - 第 3 次冲突 → `特典_3`
///   - ...
pub fn default_dest(path: &Path) -> PathBuf {
    let stem = path.file_stem().unwrap_or_default();
    let parent = path.parent().unwrap_or(Path::new("."));
    let dest = parent.join(&stem);

    // 目标路径不冲突，直接返回
    if !dest.exists() || dest.is_dir() {
        return dest;
    }

    // 目标路径已存在且是文件 → 尝试添加后缀避免冲突
    let stem_str = stem.to_string_lossy();
    let mut counter = 0u32;

    loop {
        let alt_name = if counter == 0 {
            format!("{}_", stem_str)
        } else {
            // counter 从 1 开始（0 已用），显示为 counter+1
            format!("{}_{}", stem_str, counter + 1)
        };
        let alt = parent.join(&alt_name);

        // 找到不存在的路径或已是目录时使用
        if !alt.exists() || alt.is_dir() {
            return alt;
        }
        counter += 1;
    }
}

/// 生成不冲突的文件路径
///
/// 如果 `path` 对应的文件已存在，则在文件名末尾（扩展名之前）添加 `_1`、`_2`……后缀：
///   - `/dir/file.txt` 已存在 → `/dir/file_1.txt`
///   - `/dir/file_1.txt` 也已存在 → `/dir/file_2.txt`
///   - `/dir/no_ext` 已存在 → `/dir/no_ext_1`
///
/// 如果是目录路径，则直接返回（目录不在此函数的处理范围内）。
pub fn resolve_conflict_path(path: &Path) -> PathBuf {
    // 如果路径不存在或是一个目录，无需处理
    if !path.exists() || path.is_dir() {
        return path.to_path_buf();
    }

    let parent = path.parent().unwrap_or(Path::new("."));
    let filename = path.file_name().unwrap_or_default().to_string_lossy();

    // 分离文件名和扩展名
    // 对于 `.gitignore` 这类以点开头的文件，整个都是 stem；`archive.tar.gz` 取 `archive.tar` 为 stem
    let (stem, ext) = match filename.rfind('.') {
        Some(pos) if pos > 0 => {
            let s = filename[..pos].to_string();
            let e = filename[pos..].to_string();
            (s, e)
        }
        _ => (filename.to_string(), String::new()),
    };

    let mut counter = 1u32;
    loop {
        let alt_name = format!("{}_{}{}", stem, counter, ext);
        let alt = parent.join(&alt_name);
        if !alt.exists() || alt.is_dir() {
            return alt;
        }
        counter += 1;
    }
}

/// 确保父目录存在（用于写入文件前）
pub(crate) fn ensure_parent_dir(out_path: &Path) -> io::Result<()> {
    if let Some(parent) = out_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }
    Ok(())
}
