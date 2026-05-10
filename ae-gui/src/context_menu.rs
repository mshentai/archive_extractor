use std::path::Path;
use winreg::RegKey;
use winreg::enums::*;

const MENU_NAME: &str = "AE_Extract";
const MENU_DISPLAY: &str = "使用 AE 解压";

/// 注册右键菜单（文件和目录）
pub fn register(exe_path: &Path) -> Result<(), String> {
    let exe_str = exe_path.to_string_lossy().replace('/', "\\");
    let command = format!("\"{}\" \"%1\"", exe_str);

    let hkcr = RegKey::predef(HKEY_CLASSES_ROOT);

    // --- 文件右键菜单 ---
    let (file_key, _) = hkcr
        .create_subkey(&format!("*\\shell\\{}", MENU_NAME))
        .map_err(|e| format!("无法创建注册表键: {}", e))?;
    file_key
        .set_value("", &MENU_DISPLAY)
        .map_err(|e| format!("无法设置注册表值: {}", e))?;

    let (cmd_key, _) = hkcr
        .create_subkey(&format!("*\\shell\\{}\\command", MENU_NAME))
        .map_err(|e| format!("无法创建注册表键: {}", e))?;
    cmd_key
        .set_value("", &command)
        .map_err(|e| format!("无法设置注册表值: {}", e))?;

    // --- 目录右键菜单 ---
    let (dir_key, _) = hkcr
        .create_subkey(&format!("Directory\\shell\\{}", MENU_NAME))
        .map_err(|e| format!("无法创建注册表键: {}", e))?;
    dir_key
        .set_value("", &MENU_DISPLAY)
        .map_err(|e| format!("无法设置注册表值: {}", e))?;

    let (cmd_key2, _) = hkcr
        .create_subkey(&format!("Directory\\shell\\{}\\command", MENU_NAME))
        .map_err(|e| format!("无法创建注册表键: {}", e))?;
    cmd_key2
        .set_value("", &command)
        .map_err(|e| format!("无法设置注册表值: {}", e))?;

    Ok(())
}

/// 取消注册右键菜单
pub fn unregister() -> Result<(), String> {
    let hkcr = RegKey::predef(HKEY_CLASSES_ROOT);

    let _ = hkcr.delete_subkey_all(&format!("*\\shell\\{}", MENU_NAME));
    let _ = hkcr.delete_subkey_all(&format!("Directory\\shell\\{}", MENU_NAME));

    Ok(())
}

/// 检查右键菜单是否已注册
pub fn is_registered() -> bool {
    let hkcr = RegKey::predef(HKEY_CLASSES_ROOT);
    hkcr.open_subkey(&format!("*\\shell\\{}", MENU_NAME))
        .is_ok()
}
