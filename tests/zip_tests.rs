mod common;

use archive_extractor::extract;
use archive_extractor::extract_to;
use archive_extractor::formats::zip::is_password_required_error;

use common::{create_test_zip, read_file_to_string, temp_dir, write_temp_file};

// ── 回归测试：现有功能 ───────────────────────────────────

#[test]
fn test_extract_zip_basic() {
    let dir = temp_dir("zip_basic");
    let zip_data = create_test_zip(&[("hello.txt", b"Hello, World!")], None);
    let zip_path = write_temp_file(&dir, "test.zip", &zip_data);

    let dest = dir.join("out");
    let _ = extract_to(&zip_path, &dest, None);

    let content = read_file_to_string(&dest.join("hello.txt"));
    assert_eq!(content, "Hello, World!");
}

#[test]
fn test_extract_zip_with_subdirs() {
    let dir = temp_dir("zip_subdirs");
    let zip_data = create_test_zip(
        &[
            ("dir1/file1.txt", b"file1"),
            ("dir2/file2.txt", b"file2"),
            ("root.txt", b"root"),
        ],
        None,
    );
    let zip_path = write_temp_file(&dir, "test.zip", &zip_data);

    let dest = dir.join("out");
    let _ = extract_to(&zip_path, &dest, None);

    assert_eq!(read_file_to_string(&dest.join("root.txt")), "root");
    assert_eq!(read_file_to_string(&dest.join("dir1/file1.txt")), "file1");
    assert_eq!(read_file_to_string(&dest.join("dir2/file2.txt")), "file2");
}

#[test]
fn test_extract_zip_path_traversal_safety() {
    let dir = temp_dir("zip_traversal");
    let zip_data = create_test_zip(
        &[
            ("safe.txt", b"safe"),
            ("../unsafe.txt", b"unsafe"),
            ("subdir/../../evil.txt", b"evil"),
        ],
        None,
    );
    let zip_path = write_temp_file(&dir, "test.zip", &zip_data);

    let dest = dir.join("out");
    let _ = extract_to(&zip_path, &dest, None);

    // safe.txt 应该被解压
    assert!(dest.join("safe.txt").exists(), "safe.txt 应被解压");
    // 路径遍历的条目应被跳过
    assert!(!dest.join("../unsafe.txt").exists(), "路径遍历条目应被跳过");
    assert!(!dest.join("evil.txt").exists(), "路径遍历条目应被跳过");
}

#[test]
fn test_extract_zip_empty() {
    let dir = temp_dir("zip_empty");
    let zip_data = create_test_zip(&[], None);
    let zip_path = write_temp_file(&dir, "empty.zip", &zip_data);

    let dest = dir.join("out");
    let _ = extract_to(&zip_path, &dest, None);

    assert!(dest.exists());
    assert!(dest.read_dir().unwrap().next().is_none());
}

#[test]
fn test_extract_zip_nonexistent_file() {
    let dir = temp_dir("zip_nonexist");
    let fake_path = dir.join("no_such_file.zip");
    // 不应 panic
    let _ = extract(&fake_path, None);
}

#[test]
fn test_extract_zip_not_a_zip() {
    let dir = temp_dir("zip_not_zip");
    let fake_zip = write_temp_file(&dir, "not_a_zip.bin", b"not a zip file content");
    // 不应 panic
    let _ = extract(&fake_zip, None);
}

#[test]
fn test_extract_to_custom_dest() {
    let dir = temp_dir("custom_dest");
    let zip_data = create_test_zip(&[("hello.txt", b"Hello!")], None);
    let zip_path = write_temp_file(&dir, "test.zip", &zip_data);

    let custom_dest = dir.join("custom_output");
    let _ = extract_to(&zip_path, &custom_dest, None);

    assert!(custom_dest.join("hello.txt").exists());
    assert_eq!(
        read_file_to_string(&custom_dest.join("hello.txt")),
        "Hello!"
    );
}

// ── 密码功能测试 ────────────────────────────────────────

#[test]
fn test_extract_zip_with_password_correct() {
    let dir = temp_dir("zip_pwd_correct");
    let zip_data = create_test_zip(&[("secret.txt", b"Top Secret Data")], Some("mypassword"));
    let zip_path = write_temp_file(&dir, "protected.zip", &zip_data);

    let dest = dir.join("out");
    let _ = extract_to(&zip_path, &dest, Some("mypassword"));

    let content = read_file_to_string(&dest.join("secret.txt"));
    assert_eq!(content, "Top Secret Data");
}

#[test]
fn test_extract_zip_without_password_encrypted() {
    let dir = temp_dir("zip_no_pwd");
    let zip_data = create_test_zip(&[("secret.txt", b"Secret")], Some("mypassword"));
    let zip_path = write_temp_file(&dir, "protected.zip", &zip_data);

    let dest = dir.join("out");
    // 不提供密码，应返回 PasswordRequired
    let result = extract_to(&zip_path, &dest, None);

    // 应返回 PasswordRequired 错误
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        archive_extractor::ExtractError::PasswordRequired
    ));
}

#[test]
fn test_extract_zip_wrong_password() {
    let dir = temp_dir("zip_wrong_pwd");
    let zip_data = create_test_zip(&[("secret.txt", b"Secret")], Some("correct_pwd"));
    let zip_path = write_temp_file(&dir, "protected.zip", &zip_data);

    let dest = dir.join("out");
    // 提供错误密码
    let _ = extract_to(&zip_path, &dest, Some("wrong_pwd"));

    assert!(dest.exists());
    // 密码错误，解压应失败（或条目被跳过）
    assert!(!dest.join("secret.txt").exists());
}

#[test]
fn test_extract_zip_mixed_encrypted_and_plain() {
    let dir = temp_dir("zip_mixed");
    use std::io::Write;
    use zip::ZipWriter;
    use zip::write::FileOptions;

    let buf = std::io::Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(buf);

    // 未加密条目
    zip.start_file("public.txt", FileOptions::<()>::default())
        .unwrap();
    zip.write_all(b"Public Data").unwrap();

    // 加密条目
    let encrypted_opts: FileOptions<'_, ()> =
        FileOptions::default().with_aes_encryption(zip::AesMode::Aes256, "secret");
    zip.start_file("private.txt", encrypted_opts).unwrap();
    zip.write_all(b"Private Data").unwrap();

    let zip_data = zip.finish().unwrap().into_inner();
    let zip_path = write_temp_file(&dir, "mixed.zip", &zip_data);

    // 不提供密码 → 应返回 PasswordRequired
    let dest = dir.join("out");
    let result = extract_to(&zip_path, &dest, None);

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        archive_extractor::ExtractError::PasswordRequired
    ));
}

#[test]
fn test_extract_zip_mixed_with_password() {
    let dir = temp_dir("zip_mixed_pwd");
    use std::io::Write;
    use zip::ZipWriter;
    use zip::write::FileOptions;

    let buf = std::io::Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(buf);

    zip.start_file("public.txt", FileOptions::<()>::default())
        .unwrap();
    zip.write_all(b"Public Data").unwrap();

    let encrypted_opts: FileOptions<'_, ()> =
        FileOptions::default().with_aes_encryption(zip::AesMode::Aes256, "secret");
    zip.start_file("private.txt", encrypted_opts).unwrap();
    zip.write_all(b"Private Data").unwrap();

    let zip_data = zip.finish().unwrap().into_inner();
    let zip_path = write_temp_file(&dir, "mixed.zip", &zip_data);

    // 提供密码 → 两者都解压
    let dest = dir.join("out");
    let _ = extract_to(&zip_path, &dest, Some("secret"));

    assert_eq!(read_file_to_string(&dest.join("public.txt")), "Public Data");
    assert_eq!(
        read_file_to_string(&dest.join("private.txt")),
        "Private Data"
    );
}

#[test]
fn test_is_password_required_error() {
    let err = zip::result::ZipError::UnsupportedArchive(zip::result::ZipError::PASSWORD_REQUIRED);
    assert!(is_password_required_error(&err));

    let other_err = zip::result::ZipError::FileNotFound;
    assert!(!is_password_required_error(&other_err));
}

// ── Flat 模式（等价 --flat 行为）测试 ────────────────────

#[test]
fn test_extract_zip_flat_mode_no_subdir() {
    // --flat 的等价行为：直接用 extract_to 解压到目标目录
    // 验证不会多出一层 <file_stem>/ 子目录
    let dir = temp_dir("zip_flat_no_subdir");
    let zip_data = create_test_zip(&[("hello.txt", b"Hello, Flat!")], None);
    let zip_path = write_temp_file(&dir, "test.zip", &zip_data);

    // flat 模式：直接解压到目标目录（不拼接 file_stem）
    let dest = dir.join("flat_out");
    let _ = extract_to(&zip_path, &dest, None);

    // 文件应直接在 dest 下，没有 test/ 这层子目录
    assert!(dest.join("hello.txt").exists());
    assert_eq!(read_file_to_string(&dest.join("hello.txt")), "Hello, Flat!");

    // 验证 dest 下没有 test/ 子目录（即不存在 test/hello.txt）
    assert!(!dest.join("test").join("hello.txt").exists());
}

#[test]
fn test_extract_zip_flat_with_subdirs() {
    // flat 模式下，压缩包内子目录结构应保留
    let dir = temp_dir("zip_flat_subdirs");
    let zip_data = create_test_zip(
        &[
            ("dir1/file1.txt", b"file1"),
            ("dir2/file2.txt", b"file2"),
            ("root.txt", b"root"),
        ],
        None,
    );
    let zip_path = write_temp_file(&dir, "archive.zip", &zip_data);

    // flat 模式：直接解压到目标目录
    let dest = dir.join("flat_out");
    let _ = extract_to(&zip_path, &dest, None);

    // 验证目录结构是 dest/dir1/file1.txt，不是 dest/archive/dir1/file1.txt
    assert_eq!(read_file_to_string(&dest.join("root.txt")), "root");
    assert_eq!(read_file_to_string(&dest.join("dir1/file1.txt")), "file1");
    assert_eq!(read_file_to_string(&dest.join("dir2/file2.txt")), "file2");

    // 验证没有多余的 file_stem 子目录
    assert!(!dest.join("archive").exists());
}

#[test]
fn test_extract_zip_flat_conflict_overwrite() {
    // flat 模式下，两个压缩包解压到同目录：后覆盖先
    let dir = temp_dir("zip_flat_conflict");

    // 第一个压缩包：version1.txt
    let zip1 = create_test_zip(&[("data.txt", b"version1")], None);
    let zip1_path = write_temp_file(&dir, "a.zip", &zip1);

    // 第二个压缩包：也有 data.txt，内容不同
    let zip2 = create_test_zip(&[("data.txt", b"version2")], None);
    let zip2_path = write_temp_file(&dir, "b.zip", &zip2);

    let dest = dir.join("out");
    // 先解压 a.zip
    let _ = extract_to(&zip1_path, &dest, None);
    assert_eq!(read_file_to_string(&dest.join("data.txt")), "version1");

    // 再解压 b.zip（覆盖 a.zip 的 data.txt）
    let _ = extract_to(&zip2_path, &dest, None);

    // 验证被覆盖为 version2
    assert_eq!(read_file_to_string(&dest.join("data.txt")), "version2");
}
