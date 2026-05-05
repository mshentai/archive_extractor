use std::fs;
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

// ---------------------------------------------------------------------------
// 冲突处理测试（需要真实的文件系统状态）
// ---------------------------------------------------------------------------

#[test]
fn test_default_dest_conflict_with_file() {
    let dir = std::env::temp_dir().join("ae_test_conflict_file");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    // 创建一个无扩展名的文件 "特典"
    let file_path = dir.join("特典");
    fs::write(&file_path, b"dummy").unwrap();

    // default_dest 应该返回 "特典_" 而非 "特典"（因为 "特典" 已作为文件存在）
    let dest = default_dest(&file_path);
    assert_eq!(dest, dir.join("特典_"), "应与已有文件冲突，自动添加 _ 后缀");

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_default_dest_conflict_multiple_times() {
    let dir = std::env::temp_dir().join("ae_test_conflict_multi");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    // 创建无扩展名的文件 "archive"
    let file_path = dir.join("archive");
    fs::write(&file_path, b"dummy").unwrap();

    // 再创建 "archive_" 文件 → 第二次冲突
    let first_conflict = dir.join("archive_");
    fs::write(&first_conflict, b"dummy2").unwrap();

    // default_dest 应跳过 archive_，使用 archive_2
    let dest = default_dest(&file_path);
    assert_eq!(
        dest,
        dir.join("archive_2"),
        "存档_ 也冲突时应使用 archive_2"
    );

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_default_dest_no_conflict_after_cleanup() {
    // 验证：删除冲突文件后再次调用应回到默认路径
    let dir = std::env::temp_dir().join("ae_test_conflict_cleanup");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    let file_path = dir.join("data");
    fs::write(&file_path, b"dummy").unwrap();

    // 第一次调用：有冲突 → data_
    let dest = default_dest(&file_path);
    assert_eq!(dest, dir.join("data_"), "初次冲突应使用 data_");

    // 删除冲突文件后 → 回到 data
    fs::remove_file(&file_path).unwrap();
    let dest2 = default_dest(&file_path);
    assert_eq!(dest2, dir.join("data"), "冲突消除后应使用默认 data");

    let _ = fs::remove_dir_all(&dir);
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
