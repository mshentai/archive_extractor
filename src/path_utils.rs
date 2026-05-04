use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// 获取默认解压目录（与压缩包同名，不含扩展名）
pub fn default_dest(path: &Path) -> PathBuf {
    let stem = path.file_stem().unwrap_or_default();
    path.parent().unwrap_or(Path::new(".")).join(stem)
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
