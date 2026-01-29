use tauri::{AppHandle, Window};

#[tauri::command]
pub fn get_app_version(app: AppHandle) -> String {
    app.package_info().version.to_string()
}

#[tauri::command]
pub fn set_skip_taskbar(window: Window, skip: bool) -> Result<(), String> {
    window.set_skip_taskbar(skip).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn remove_window_animation(window: Window) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::Foundation::HWND;
        use windows::Win32::Graphics::Dwm::{DwmSetWindowAttribute, DWMWA_TRANSITIONS_FORCEDISABLED};
        
        // 在 Tauri v2 中，window.hwnd() 返回的是 Result<HWND, Error>
        // 这里的 HWND 是 tauri 包装的类型，可以通过 .0 获取内部的 isize
        let hwnd_value = window.hwnd().map_err(|e| e.to_string())?;
        let hwnd = HWND(hwnd_value.0 as _);
        let value = 1i32;
        unsafe {
            let _ = DwmSetWindowAttribute(
                hwnd,
                DWMWA_TRANSITIONS_FORCEDISABLED,
                &value as *const _ as _,
                std::mem::size_of::<i32>() as u32,
            );
        }
    }
    Ok(())
}
