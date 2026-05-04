pub(crate) mod rar;
pub(crate) mod sevenz;
pub mod zip;

use std::path::Path;

/// 根据文件签名检测类型并分发到对应格式的解压函数
pub(crate) fn dispatch_format(path: &Path, data: &[u8], dest: &Path, password: Option<&str>) {
    let kind = match infer::get(data) {
        Some(k) => k,
        None => {
            println!("未知文件类型，跳过: {}", path.display());
            return;
        }
    };

    match kind.mime_type() {
        "application/zip" => zip::extract_zip(path, data, dest, password),
        "application/x-7z-compressed" => sevenz::extract_7z(path, dest, password),
        "application/vnd.rar" => rar::extract_rar(path, dest, password),
        _ => println!(
            "不是支持的压缩格式 ({}), 跳过: {}",
            kind.mime_type(),
            path.display()
        ),
    }
}
