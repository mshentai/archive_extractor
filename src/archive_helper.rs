use std::fs;
use std::path::Path;

use crate::formats;
use crate::path_utils::default_dest;

/// 解压到默认目录（与压缩包同目录，同名文件夹）
pub fn extract(path: &Path, password: Option<&str>) {
    let dest = default_dest(path);
    extract_to(path, &dest, password);
}

/// 解压到指定目录
pub fn extract_to(path: &Path, dest: &Path, password: Option<&str>) {
    // 确保目标目录存在
    if let Err(e) = fs::create_dir_all(dest) {
        eprintln!("创建目标目录 {} 失败: {}", dest.display(), e);
        return;
    }

    // 由 formats::dispatch_format 内部自行管理文件读取和类型检测
    formats::dispatch_format(path, dest, password);
}
