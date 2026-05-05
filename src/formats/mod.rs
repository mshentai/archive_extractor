pub(crate) mod rar;
pub(crate) mod sevenz;
pub mod zip;

use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;

const HEADER_SIZE: usize = 1024; // 检测文件类型只需要前 1KB

/// 根据文件签名检测类型并分发到对应格式的解压函数
pub(crate) fn dispatch_format(path: &Path, dest: &Path, password: Option<&str>) {
    // 1. 打开文件并读取前 1KB 头部用于类型检测
    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("无法打开文件 {}: {}", path.display(), e);
            return;
        }
    };

    let mut reader = BufReader::new(file);
    let mut header = vec![0u8; HEADER_SIZE];
    let n = match reader.read(&mut header) {
        Ok(n) => n,
        Err(e) => {
            eprintln!("无法读取文件 {}: {}", path.display(), e);
            return;
        }
    };
    header.truncate(n);

    // 2. 用 infer 检测类型
    let kind = match infer::get(&header) {
        Some(k) => k,
        None => {
            println!("未知文件类型，跳过: {}", path.display());
            return;
        }
    };

    // 3. 根据类型分发
    match kind.mime_type() {
        "application/zip" => {
            // 回到文件开头，将 BufReader 传给 ZIP 解压（流式读取）
            if let Err(e) = reader.seek(SeekFrom::Start(0)) {
                eprintln!("无法定位文件 {}: {}", path.display(), e);
                return;
            }
            zip::extract_zip(path, reader, dest, password);
        }
        "application/x-7z-compressed" => {
            // 关闭文件句柄，让 sevenz_rust 自行管理
            drop(reader);
            sevenz::extract_7z(path, dest, password);
        }
        "application/vnd.rar" => {
            // 关闭文件句柄，让 unrar 自行管理
            drop(reader);
            rar::extract_rar(path, dest, password);
        }
        _ => println!(
            "不是支持的压缩格式 ({}), 跳过: {}",
            kind.mime_type(),
            path.display()
        ),
    }
}
