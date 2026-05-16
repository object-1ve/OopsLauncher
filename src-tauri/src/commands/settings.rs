use crate::{error::AppError, models::LauncherState, services};

#[tauri::command]
pub fn save_launcher_state_to_db(app: tauri::AppHandle, state: LauncherState) -> Result<(), AppError> {
    services::settings_service::save_launcher_state(&app, state)
}

#[tauri::command]
pub fn load_launcher_state_from_db(app: tauri::AppHandle) -> Result<LauncherState, AppError> {
    services::settings_service::load_launcher_state(&app)
}

#[tauri::command]
pub fn save_settings_to_json(app: tauri::AppHandle, settings: serde_json::Value) -> Result<(), AppError> {
    services::settings_service::save_settings(&app, settings)
}

#[tauri::command]
pub fn load_settings_from_json(app: tauri::AppHandle) -> Result<serde_json::Value, AppError> {
    services::settings_service::load_settings(&app)
}
