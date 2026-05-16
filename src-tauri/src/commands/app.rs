use crate::error::AppError;
use tauri::{AppHandle, Manager, Window};

#[tauri::command]
pub fn get_app_version(app: AppHandle) -> String {
    app.package_info().version.to_string()
}

#[tauri::command]
pub fn set_skip_taskbar(app: AppHandle, skip: bool) -> Result<(), AppError> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| AppError::Window("Main window not found".into()))?;
    window
        .set_skip_taskbar(skip)
        .map_err(|e| AppError::Window(e.to_string()))
}

#[tauri::command]
pub fn check_is_minimized() -> bool {
    std::env::args().any(|arg| arg == "--minimized")
}

#[tauri::command]
pub fn remove_window_animation(window: Window) -> Result<(), AppError> {
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::Foundation::HWND;
        use windows::Win32::Graphics::Dwm::{
            DwmSetWindowAttribute, DWMWA_TRANSITIONS_FORCEDISABLED,
        };

        let hwnd_value = window.hwnd().map_err(|e| AppError::Window(e.to_string()))?;
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

#[tauri::command]
pub fn exit_app(app: AppHandle) {
    app.exit(0);
}

#[tauri::command]
pub fn disable_settings_system_menu(window: Window) -> Result<(), AppError> {
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::UI::WindowsAndMessaging::{
            GetWindowLongPtrW, SetWindowLongPtrW, GWL_STYLE, WS_SYSMENU,
        };

        let hwnd_value = window.hwnd().map_err(|e| AppError::Window(e.to_string()))?;
        let hwnd = windows::Win32::Foundation::HWND(hwnd_value.0 as _);

        unsafe {
            let style = GetWindowLongPtrW(hwnd, GWL_STYLE);
            if style != 0 {
                let new_style = style & !(WS_SYSMENU.0 as isize);
                SetWindowLongPtrW(hwnd, GWL_STYLE, new_style);
            }
        }
    }
    Ok(())
}
