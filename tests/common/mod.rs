use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

/// 创建一个临时目录并返回路径
pub fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("archive_extractor_test_{}", name));
    let _ = fs::remove_dir_all(&dir); // 清理旧的
    fs::create_dir_all(&dir).unwrap();
    dir
}

/// 用 zip crate 动态创建一个测试 ZIP 文件，返回字节数据
pub fn create_test_zip(entries: &[(&str, &[u8])], password: Option<&str>) -> Vec<u8> {
    use zip::ZipWriter;
    use zip::write::FileOptions;

    let buf = std::io::Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(buf);

    for (name, content) in entries {
        let mut options: FileOptions<'_, ()> = FileOptions::default();
        if let Some(pwd) = password {
            options = options.with_aes_encryption(zip::AesMode::Aes256, pwd);
        }
        zip.start_file(name, options).unwrap();
        zip.write_all(content).unwrap();
    }

    zip.finish().unwrap().into_inner()
}

/// 将字节写入临时文件，返回文件路径
pub fn write_temp_file(dir: &Path, name: &str, data: &[u8]) -> PathBuf {
    let path = dir.join(name);
    fs::write(&path, data).unwrap();
    path
}

/// 读取文件内容为字符串
pub fn read_file_to_string(path: &Path) -> String {
    fs::read_to_string(path).unwrap()
}
