#[test]
fn test_format_detection_zip() {
    // ZIP 文件签名：PK\x03\x04
    let zip_header = [0x50, 0x4B, 0x03, 0x04, 0x00, 0x00];
    let kind = infer::get(&zip_header).unwrap();
    assert_eq!(kind.mime_type(), "application/zip");
}

#[test]
fn test_format_detection_7z() {
    // 7z 文件签名：7z\xBC\xAF\x27\x1C
    let header = [0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C];
    let kind = infer::get(&header).unwrap();
    assert_eq!(kind.mime_type(), "application/x-7z-compressed");
}

#[test]
fn test_format_detection_rar() {
    // RAR 文件签名：Rar!\x1A\x07\x00（至少7字节，第7字节为0x00或0x01）
    let header = [0x52, 0x61, 0x72, 0x21, 0x1A, 0x07, 0x00];
    let kind = infer::get(&header).unwrap();
    assert_eq!(kind.mime_type(), "application/vnd.rar");
}

#[test]
fn test_format_detection_unknown() {
    let data = [0x00, 0x01, 0x02, 0x03];
    assert!(infer::get(&data).is_none());
}
