mod archive_helper;
pub mod formats;
pub mod path_utils;

/// 解压到默认目录（与压缩包同目录，同名文件夹）
pub fn extract(path: &std::path::Path, password: Option<&str>) {
    archive_helper::extract(path, password)
}

/// 解压到指定目录
pub fn extract_to(path: &std::path::Path, dest: &std::path::Path, password: Option<&str>) {
    archive_helper::extract_to(path, dest, password)
}
