use std::path::Path;
use winreg::RegKey;
use winreg::enums::*;

const MENU_NAME: &str = "AE_Extract";
const MENU_DISPLAY: &str = "使用 AE 解压";

/// 注册右键菜单（文件和目录）
pub fn register(exe_path: &Path) -> Result<(), String> {
    let exe_str = exe_path.to_string_lossy().replace('/', "\\");
    let command = format!("\"{}\" \"%1\"", exe_str);

    // 使用 HKEY_CURRENT_USER\Software\Classes 而非 HKEY_CLASSES_ROOT
    // 前者是用户级路径，不需要管理员权限即可写入
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let classes = hkcu
        .open_subkey_with_flags("Software\\Classes", KEY_CREATE_SUB_KEY)
        .map_err(|e| format!("无法打开注册表键: {}", e))?;

    // --- 文件右键菜单 ---
    let (file_key, _) = classes
        .create_subkey(&format!("*\\shell\\{}", MENU_NAME))
        .map_err(|e| format!("无法创建注册表键: {}", e))?;
    file_key
        .set_value("", &MENU_DISPLAY)
        .map_err(|e| format!("无法设置注册表值: {}", e))?;

    let (cmd_key, _) = classes
        .create_subkey(&format!("*\\shell\\{}\\command", MENU_NAME))
        .map_err(|e| format!("无法创建注册表键: {}", e))?;
    cmd_key
        .set_value("", &command)
        .map_err(|e| format!("无法设置注册表值: {}", e))?;

    // --- 目录右键菜单 ---
    let (dir_key, _) = classes
        .create_subkey(&format!("Directory\\shell\\{}", MENU_NAME))
        .map_err(|e| format!("无法创建注册表键: {}", e))?;
    dir_key
        .set_value("", &MENU_DISPLAY)
        .map_err(|e| format!("无法设置注册表值: {}", e))?;

    let (cmd_key2, _) = classes
        .create_subkey(&format!("Directory\\shell\\{}\\command", MENU_NAME))
        .map_err(|e| format!("无法创建注册表键: {}", e))?;
    cmd_key2
        .set_value("", &command)
        .map_err(|e| format!("无法设置注册表值: {}", e))?;

    Ok(())
}

/// 取消注册右键菜单
pub fn unregister() -> Result<(), String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let classes = hkcu
        .open_subkey_with_flags("Software\\Classes", KEY_CREATE_SUB_KEY)
        .map_err(|e| format!("无法打开注册表键: {}", e))?;

    let _ = classes.delete_subkey_all(&format!("*\\shell\\{}", MENU_NAME));
    let _ = classes.delete_subkey_all(&format!("Directory\\shell\\{}", MENU_NAME));

    Ok(())
}

/// 检查右键菜单是否已注册
pub fn is_registered() -> bool {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    hkcu.open_subkey(&format!("Software\\Classes\\*\\shell\\{}", MENU_NAME))
        .is_ok()
}
