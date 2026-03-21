use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAssociationStatus {
    pub log: bool,
}

const SUPPORTED_EXTENSIONS: [&str; 2] = ["log", "log.1"];

#[cfg(target_os = "windows")]
pub mod windows {
    use super::SUPPORTED_EXTENSIONS;
    use std::collections::HashMap;
    use winreg::enums::*;
    use winreg::RegKey;

    const APP_NAME: &str = "LogViewer";

    fn get_exe_path() -> Result<String, String> {
        let exe_path = std::env::current_exe()
            .map_err(|e| format!("Failed to get exe path: {}", e))?;
        Ok(exe_path.to_string_lossy().to_string())
    }

    fn get_hkcu() -> RegKey {
        RegKey::predef(HKEY_CURRENT_USER)
    }

    pub fn set_file_association(extension: &str) -> Result<(), String> {
        let exe_path = get_exe_path()?;
        let hkcu = get_hkcu();

        let prog_id = format!("{}.{}", APP_NAME, normalize_extension(extension));
        let prog_id_key = hkcu
            .create_subkey(format!("Software\\Classes\\{}", prog_id))
            .map_err(|e| format!("Failed to create prog_id key: {}", e))?
            .0;

        prog_id_key
            .set_value("", &format!("{} File", APP_NAME))
            .map_err(|e| format!("Failed to set prog_id default: {}", e))?;

        let icon_key = prog_id_key
            .create_subkey("DefaultIcon")
            .map_err(|e| format!("Failed to create icon key: {}", e))?
            .0;
        icon_key
            .set_value("", &format!("{},0", exe_path))
            .map_err(|e| format!("Failed to set icon: {}", e))?;

        let command_key = prog_id_key
            .create_subkey("shell\\open\\command")
            .map_err(|e| format!("Failed to create command key: {}", e))?
            .0;
        command_key
            .set_value("", &format!("\"{}\" \"%1\"", exe_path))
            .map_err(|e| format!("Failed to set command: {}", e))?;

        let ext_key = hkcu
            .create_subkey(format!("Software\\Classes\\.{}", extension))
            .map_err(|e| format!("Failed to create extension key: {}", e))?
            .0;
        ext_key
            .set_value("", &prog_id)
            .map_err(|e| format!("Failed to set extension association: {}", e))?;

        let open_with_key = hkcu
            .create_subkey(format!(
                "Software\\Classes\\SystemFileAssociations\\.{}\\Shell\\{}",
                extension, APP_NAME
            ))
            .map_err(|e| format!("Failed to create open with key: {}", e))?
            .0;
        open_with_key
            .set_value("", &format!("用{}打开", APP_NAME))
            .map_err(|e| format!("Failed to set open with text: {}", e))?;

        let open_with_cmd_key = open_with_key
            .create_subkey("command")
            .map_err(|e| format!("Failed to create open with command key: {}", e))?
            .0;
        open_with_cmd_key
            .set_value("", &format!("\"{}\" \"%1\"", exe_path))
            .map_err(|e| format!("Failed to set open with command: {}", e))?;

        Ok(())
    }

    pub fn set_file_associations(extensions: &[String]) -> Result<(), String> {
        for ext in extensions {
            set_file_association(ext)?;
        }
        notify_shell_change();
        Ok(())
    }

    pub fn remove_file_association(extension: &str) -> Result<(), String> {
        let hkcu = get_hkcu();
        let prog_id = format!("{}.{}", APP_NAME, normalize_extension(extension));

        let _ = hkcu.delete_subkey_all(format!("Software\\Classes\\{}", prog_id));
        let _ = hkcu.delete_subkey(format!("Software\\Classes\\.{}", extension));
        let _ = hkcu.delete_subkey_all(format!(
            "Software\\Classes\\SystemFileAssociations\\.{}\\Shell\\{}",
            extension, APP_NAME
        ));

        Ok(())
    }

    pub fn remove_file_associations(extensions: &[String]) -> Result<(), String> {
        for ext in extensions {
            remove_file_association(ext)?;
        }
        notify_shell_change();
        Ok(())
    }

    pub fn check_file_association(extension: &str) -> Result<bool, String> {
        let hkcu = get_hkcu();
        let prog_id = format!("{}.{}", APP_NAME, normalize_extension(extension));

        let ext_key = hkcu
            .open_subkey(format!("Software\\Classes\\.{}", extension))
            .ok();

        if let Some(key) = ext_key {
            let value: Result<String, _> = key.get_value("");
            if let Ok(val) = value {
                if val == prog_id {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    pub fn check_file_associations() -> Result<HashMap<String, bool>, String> {
        let mut result = HashMap::new();
        for ext in SUPPORTED_EXTENSIONS {
            result.insert(ext.to_string(), check_file_association(ext)?);
        }
        Ok(result)
    }

    fn normalize_extension(extension: &str) -> String {
        extension.replace(".", "_").to_uppercase()
    }

    fn notify_shell_change() {
        unsafe {
            #[link(name = "shell32")]
            extern "system" {
                fn SHChangeNotify(
                    wEventId: u32,
                    uFlags: u32,
                    dwItem1: *const std::ffi::c_void,
                    dwItem2: *const std::ffi::c_void,
                );
            }
            const SHCNE_ASSOCCHANGED: u32 = 0x08000000;
            const SHCNF_IDLIST: u32 = 0x0000;
            SHChangeNotify(SHCNE_ASSOCCHANGED, SHCNF_IDLIST, std::ptr::null(), std::ptr::null());
        }
    }
}

#[cfg(not(target_os = "windows"))]
pub mod windows {
    use super::SUPPORTED_EXTENSIONS;
    use std::collections::HashMap;

    pub fn set_file_association(_extension: &str) -> Result<(), String> {
        Err("File association is only supported on Windows".to_string())
    }

    pub fn set_file_associations(_extensions: &[String]) -> Result<(), String> {
        Err("File association is only supported on Windows".to_string())
    }

    pub fn remove_file_association(_extension: &str) -> Result<(), String> {
        Err("File association is only supported on Windows".to_string())
    }

    pub fn remove_file_associations(_extensions: &[String]) -> Result<(), String> {
        Err("File association is only supported on Windows".to_string())
    }

    pub fn check_file_association(_extension: &str) -> Result<bool, String> {
        Ok(false)
    }

    pub fn check_file_associations() -> Result<HashMap<String, bool>, String> {
        let mut result = HashMap::new();
        for ext in SUPPORTED_EXTENSIONS {
            result.insert(ext.to_string(), false);
        }
        Ok(result)
    }
}
