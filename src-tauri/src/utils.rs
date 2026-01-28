use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn resolve_shortcut(path: &str) -> String {
    #[cfg(target_os = "windows")]
    {
        if path.to_lowercase().ends_with(".lnk") {
            let output = Command::new("powershell")
                .args([
                    "-NoProfile",
                    "-Command",
                    &format!("$shell = New-Object -ComObject WScript.Shell; $shortcut = $shell.CreateShortcut('{}'); $shortcut.TargetPath", path)
                ])
                .output();

            if let Ok(out) = output {
                let target = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if !target.is_empty() && Path::new(&target).exists() {
                    return target;
                }
            }
        }
    }
    path.to_string()
}

pub fn to_abs_path(path: &str) -> Result<String, String> {
    let p = Path::new(path);
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
