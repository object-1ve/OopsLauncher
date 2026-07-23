use std::env;
use std::path::{Path, PathBuf};

#[cfg(target_os = "windows")]
use windows::{
    core::{HSTRING, PCWSTR, ComInterface},
    Win32::System::Com::{CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, IPersistFile, STGM},
    Win32::UI::Shell::{IShellLinkW, ShellLink},
    Win32::Foundation::MAX_PATH,
};

pub fn resolve_shortcut(path: &str) -> String {
    #[cfg(target_os = "windows")]
    {
        if path.to_lowercase().ends_with(".lnk") {
            unsafe {
                let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
                let result = (|| -> windows::core::Result<String> {
                    let shell_link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)?;
                    let persist_file: IPersistFile = shell_link.cast()?;
                    let h_path = HSTRING::from(path);
                    persist_file.Load(PCWSTR(h_path.as_ptr()), STGM(0))?;

                    let mut buffer = [0u16; MAX_PATH as usize];
                    shell_link.GetPath(&mut buffer, std::ptr::null_mut(), 0)?;
                    
                    let target = String::from_utf16_lossy(&buffer);
                    let target = target.trim_matches(char::from(0)).to_string();
                    Ok(target)
                })();
                CoUninitialize();
                if let Ok(target) = result {
                    if !target.is_empty() && Path::new(&target).exists() {
                        return target;
                    }
                }
            }
        }
    }
    path.to_string()
}

pub fn to_abs_path(path: &str) -> Result<String, String> {
    let path = path.trim();
    let p = Path::new(path);
    if path.is_empty() {
        return env::current_dir()
            .map(|d| d.to_string_lossy().to_string())
            .map_err(|e| format!("current_dir error: {}", e));
    }
    let abs: PathBuf = if p.is_absolute() {
        p.to_path_buf()
    } else {
        env::current_dir()
            .map_err(|e| e.to_string())?
            .join(p)
    };

    let abs = abs.canonicalize().unwrap_or(abs);
    
    // 在Windows上移除 \\?\ 前缀
    #[cfg(target_os = "windows")]
    let abs_str = {
        let s = abs.to_string_lossy().to_string();
        if s.starts_with(r"\\?\") {
            s[4..].to_string()
        } else {
            s
        }
    };

    #[cfg(not(target_os = "windows"))]
    let abs_str = abs.to_string_lossy().to_string();

    Ok(abs_str)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_abs_path_absolute() {
        #[cfg(target_os = "windows")]
        let path = "C:\\";
        #[cfg(not(target_os = "windows"))]
        let path = "/tmp";
        let result = to_abs_path(path);
        assert!(result.is_ok(), "to_abs_path('{}') should not error: {:?}", path, result.err());
        let p = result.unwrap();
        assert!(!p.is_empty(), "path should not be empty");
    }

    #[test]
    fn test_to_abs_path_trim_whitespace() {
        #[cfg(target_os = "windows")]
        let input = "  C:\\  ";
        #[cfg(not(target_os = "windows"))]
        let input = "  /tmp  ";
        let result = to_abs_path(input);
        assert!(result.is_ok(), "to_abs_path('{}') should not error: {:?}", input, result.err());
    }

    #[test]
    fn test_to_abs_path_empty() {
        let result = to_abs_path("");
        assert!(result.is_ok(), "to_abs_path('') should not error: {:?}", result.err());
    }

    #[test]
    fn test_to_abs_path_dot() {
        let result = to_abs_path(".");
        assert!(result.is_ok(), "to_abs_path('.') should not error: {:?}", result.err());
    }
}
