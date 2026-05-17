use std::path::Path;
use winreg::RegKey;
use winreg::enums::*;

const MENU_NAME: &str = "AE_Extract";
const MENU_DISPLAY: &str = "使用 AE 解压";

const MENU_NAME_FLAT: &str = "AE_Extract_Flat";
const MENU_DISPLAY_FLAT: &str = "使用 AE 解压（平铺模式）";

/// 在注册表指定路径下创建右键菜单项
fn register_menu(menu_name: &str, display: &str, command: &str) -> Result<(), String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let classes = hkcu
        .open_subkey_with_flags("Software\\Classes", KEY_CREATE_SUB_KEY)
        .map_err(|e| format!("无法打开注册表键: {}", e))?;

    // 文件右键菜单
    let (file_key, _) = classes
        .create_subkey(&format!("*\\shell\\{}", menu_name))
        .map_err(|e| format!("无法创建注册表键: {}", e))?;
    file_key
        .set_value("", &display)
        .map_err(|e| format!("无法设置注册表值: {}", e))?;
    let (cmd_key, _) = classes
        .create_subkey(&format!("*\\shell\\{}\\command", menu_name))
        .map_err(|e| format!("无法创建注册表键: {}", e))?;
    cmd_key
        .set_value("", &command)
        .map_err(|e| format!("无法设置注册表值: {}", e))?;

    // 目录右键菜单
    let (dir_key, _) = classes
        .create_subkey(&format!("Directory\\shell\\{}", menu_name))
        .map_err(|e| format!("无法创建注册表键: {}", e))?;
    dir_key
        .set_value("", &display)
        .map_err(|e| format!("无法设置注册表值: {}", e))?;
    let (cmd_key2, _) = classes
        .create_subkey(&format!("Directory\\shell\\{}\\command", menu_name))
        .map_err(|e| format!("无法创建注册表键: {}", e))?;
    cmd_key2
        .set_value("", &command)
        .map_err(|e| format!("无法设置注册表值: {}", e))?;

    Ok(())
}

/// 删除注册表路径下的右键菜单项
fn unregister_menu(menu_name: &str) -> Result<(), String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let classes = hkcu
        .open_subkey_with_flags("Software\\Classes", KEY_CREATE_SUB_KEY)
        .map_err(|e| format!("无法打开注册表键: {}", e))?;

    let _ = classes.delete_subkey_all(&format!("*\\shell\\{}", menu_name));
    let _ = classes.delete_subkey_all(&format!("Directory\\shell\\{}", menu_name));

    Ok(())
}

/// 检查指定右键菜单项是否已注册
fn check_registered(menu_name: &str) -> bool {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    hkcu.open_subkey(&format!("Software\\Classes\\*\\shell\\{}", menu_name))
        .is_ok()
}

/// 注册右键菜单（普通模式）
pub fn register(exe_path: &Path) -> Result<(), String> {
    let exe_str = exe_path.to_string_lossy().replace('/', "\\");
    let command = format!("\"{}\" \"%1\"", exe_str);
    register_menu(MENU_NAME, MENU_DISPLAY, &command)
}

/// 注册右键菜单（平铺模式）
pub fn register_flat(exe_path: &Path) -> Result<(), String> {
    let exe_str = exe_path.to_string_lossy().replace('/', "\\");
    let command = format!("\"{}\" --flat \"%1\"", exe_str);
    register_menu(MENU_NAME_FLAT, MENU_DISPLAY_FLAT, &command)
}

/// 取消注册所有右键菜单
pub fn unregister() -> Result<(), String> {
    unregister_menu(MENU_NAME)?;
    unregister_menu(MENU_NAME_FLAT)?;
    Ok(())
}

/// 检查右键菜单是否已注册（普通模式）
pub fn is_registered() -> bool {
    check_registered(MENU_NAME)
}

/// 检查右键菜单是否已注册（平铺模式）
pub fn is_flat_registered() -> bool {
    check_registered(MENU_NAME_FLAT)
}
