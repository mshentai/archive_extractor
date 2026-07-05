use std::fs;
use std::path::Path;

use archive_extractor::path_utils::{default_dest, resolve_conflict_path};

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

// ---------------------------------------------------------------------------
// resolve_conflict_path 测试
// ---------------------------------------------------------------------------

#[test]
fn test_resolve_conflict_path_no_conflict() {
    // 不存在的路径应原样返回
    let path = Path::new("C:\\nonexistent\\file.txt");
    assert_eq!(resolve_conflict_path(path), path);
}

#[test]
fn test_resolve_conflict_path_with_extension() {
    let dir = std::env::temp_dir().join("ae_test_resolve_ext");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    // 创建 file.txt
    let path = dir.join("file.txt");
    fs::write(&path, b"original").unwrap();

    let resolved = resolve_conflict_path(&path);
    assert_eq!(resolved, dir.join("file_1.txt"), "应自动添加 _1 后缀");

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_resolve_conflict_path_no_extension() {
    let dir = std::env::temp_dir().join("ae_test_resolve_noext");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    // 创建无扩展名文件 "README"
    let path = dir.join("README");
    fs::write(&path, b"content").unwrap();

    let resolved = resolve_conflict_path(&path);
    assert_eq!(resolved, dir.join("README_1"), "无扩展名应添加 _1 后缀");

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_resolve_conflict_path_multiple_conflicts() {
    let dir = std::env::temp_dir().join("ae_test_resolve_multi");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    // 创建 data.txt 和 data_1.txt
    let path = dir.join("data.txt");
    fs::write(&path, b"v0").unwrap();
    fs::write(&dir.join("data_1.txt"), b"v1").unwrap();

    let resolved = resolve_conflict_path(&path);
    assert_eq!(
        resolved,
        dir.join("data_2.txt"),
        "跳过 data_1.txt，使用 data_2.txt"
    );

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_resolve_conflict_path_double_extension() {
    let dir = std::env::temp_dir().join("ae_test_resolve_double_ext");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    // 创建 archive.tar.gz
    let path = dir.join("archive.tar.gz");
    fs::write(&path, b"content").unwrap();

    let resolved = resolve_conflict_path(&path);
    // 后缀应加在最后一个扩展名之前：archive.tar_1.gz
    assert_eq!(
        resolved,
        dir.join("archive.tar_1.gz"),
        "双扩展名应在最后一个点前插入 _1"
    );

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_resolve_conflict_path_dir_no_conflict() {
    // 目录路径应原样返回
    let dir = std::env::temp_dir().join("ae_test_resolve_dir");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    let path = dir.join("subdir");
    fs::create_dir_all(&path).unwrap();

    let resolved = resolve_conflict_path(&path);
    assert_eq!(resolved, path, "目录不应被重命名");

    let _ = fs::remove_dir_all(&dir);
}
