use std::path::Path;

use archive_extractor::path_utils::default_dest;

#[test]
fn test_default_dest_basic() {
    let p = Path::new("/home/user/archive.zip");
    assert_eq!(default_dest(p), Path::new("/home/user/archive"));
}

#[test]
fn test_default_dest_no_extension() {
    let p = Path::new("archive");
    assert_eq!(default_dest(p), Path::new("archive"));
}

#[test]
fn test_default_dest_multi_extension() {
    let p = Path::new("game.rar.bak");
    assert_eq!(default_dest(p), Path::new("game.rar"));
}

#[test]
fn test_default_dest_windows_path() {
    let p = Path::new("C:\\Downloads\\file.7z");
    assert_eq!(default_dest(p), Path::new("C:\\Downloads\\file"));
}
