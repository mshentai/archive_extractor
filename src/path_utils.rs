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

/// 确保父目录存在（用于写入文件前）
pub(crate) fn ensure_parent_dir(out_path: &Path) -> io::Result<()> {
    if let Some(parent) = out_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }
    Ok(())
}
