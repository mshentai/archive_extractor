mod archive_helper;
pub mod formats;
pub mod path_utils;

use std::fmt;
use std::path::Path;

// ---------------------------------------------------------------------------
// 错误类型
// ---------------------------------------------------------------------------

/// 解压过程中可能出现的错误
#[derive(Debug)]
pub enum ExtractError {
    /// 压缩包需要密码，但未提供
    PasswordRequired,
    /// 解压失败（IO 错误、格式错误等）
    ExtractFailed(String),
}

impl fmt::Display for ExtractError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExtractError::PasswordRequired => write!(f, "需要密码"),
            ExtractError::ExtractFailed(msg) => write!(f, "解压失败: {}", msg),
        }
    }
}

impl std::error::Error for ExtractError {}

// ---------------------------------------------------------------------------
// 公开 API
// ---------------------------------------------------------------------------

/// 解压到默认目录（与压缩包同目录，同名文件夹）
pub fn extract(path: &Path, password: Option<&str>) -> Result<(), ExtractError> {
    archive_helper::extract(path, password)
}

/// 解压到指定目录
pub fn extract_to(path: &Path, dest: &Path, password: Option<&str>) -> Result<(), ExtractError> {
    archive_helper::extract_to(path, dest, password)
}
