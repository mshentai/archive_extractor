use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::ExtractError;
use crate::formats;
use crate::path_utils::{default_dest, resolve_conflict_path};

/// 解压到默认目录（与压缩包同目录，同名文件夹）
pub fn extract(path: &Path, password: Option<&str>) -> Result<(), ExtractError> {
    let dest = default_dest(path);
    extract_to(path, &dest, password)
}

/// 解压到指定目录
pub fn extract_to(path: &Path, dest: &Path, password: Option<&str>) -> Result<(), ExtractError> {
    // 确保目标目录存在
    fs::create_dir_all(dest)
        .map_err(|e| ExtractError::ExtractFailed(format!("创建目标目录失败: {}", e)))?;

    // 先解压到临时暂存区，再逐文件迁移到目标目录（自动处理冲突）
    let staging = create_staging_dir(dest)?;
    let result = formats::dispatch_format(path, &staging, password);

    match result {
        Ok(()) => {
            // 将暂存区文件迁移到目标目录，处理冲突
            if let Err(e) = move_with_conflict_resolution(&staging, dest) {
                let _ = fs::remove_dir_all(&staging);
                return Err(ExtractError::ExtractFailed(format!(
                    "迁移文件时出错: {}",
                    e
                )));
            }
            // 清理暂存区
            let _ = fs::remove_dir_all(&staging);
            Ok(())
        }
        Err(e) => {
            // 解压失败，清理暂存区
            let _ = fs::remove_dir_all(&staging);
            Err(e)
        }
    }
}

// ---------------------------------------------------------------------------
// 内部辅助函数
// ---------------------------------------------------------------------------

/// 在 `dest` 下创建一个唯一的临时暂存目录
fn create_staging_dir(dest: &Path) -> Result<PathBuf, ExtractError> {
    let mut counter = 0u32;
    loop {
        let staging = if counter == 0 {
            dest.join(".ae_staging")
        } else {
            dest.join(format!(".ae_staging_{}", counter))
        };
        // 如果目录不存在，创建并返回
        if !staging.exists() {
            fs::create_dir_all(&staging)
                .map_err(|e| ExtractError::ExtractFailed(format!("创建暂存目录失败: {}", e)))?;
            return Ok(staging);
        }
        counter += 1;
    }
}

/// 递归遍历 `src` 目录，将每个文件迁移到 `dest` 目录，
/// 遇到同名文件时自动添加 `_1`、`_2`……后缀避免冲突。
fn move_with_conflict_resolution(src: &Path, dest: &Path) -> io::Result<()> {
    // 采用先收集再处理的方式，避免在遍历时修改目录结构
    let mut entries = Vec::new();
    collect_entries(src, src, &mut entries)?;

    for rel_path in &entries {
        let src_path = src.join(rel_path);
        let target_path = dest.join(rel_path);

        if src_path.is_dir() {
            fs::create_dir_all(&target_path)?;
        } else {
            // 确保父目录存在
            if let Some(parent) = target_path.parent() {
                fs::create_dir_all(parent)?;
            }
            // 使用冲突检测获取安全路径
            let actual_target = resolve_conflict_path(&target_path);
            if actual_target != target_path {
                eprintln!(
                    "  文件冲突，重命名: {} -> {}",
                    target_path.display(),
                    actual_target.display()
                );
            }
            // 重命名（同一文件系统，原子操作）
            fs::rename(&src_path, &actual_target)?;
        }
    }

    Ok(())
}

/// 递归收集 `base` 下所有文件和目录的相对路径
fn collect_entries(base: &Path, dir: &Path, entries: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let rel = path.strip_prefix(base).unwrap_or(&path).to_path_buf();
        entries.push(rel);
        if path.is_dir() {
            collect_entries(base, &path, entries)?;
        }
    }
    Ok(())
}
