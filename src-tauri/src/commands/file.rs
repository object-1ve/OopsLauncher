use crate::{error::AppError, models::FileInfo, services};

#[tauri::command]
pub fn save_files_to_db(app: tauri::AppHandle, files: Vec<FileInfo>) -> Result<(), AppError> {
    services::file_service::save_files(&app, files)
}

#[tauri::command]
pub fn load_files_from_db(app: tauri::AppHandle) -> Result<Vec<FileInfo>, AppError> {
    services::file_service::load_files(&app)
}

#[tauri::command]
pub fn get_file_info(path: String) -> Result<FileInfo, AppError> {
    services::file_service::get_file_info(path)
}

#[tauri::command]
pub fn open_path(path: String) -> Result<(), AppError> {
    services::file_service::open_path(path)
}

#[tauri::command]
pub fn open_file_location(path: String) -> Result<(), AppError> {
    services::file_service::open_file_location(path)
}

#[tauri::command]
pub fn open_with_dialog(path: String) -> Result<(), AppError> {
    services::file_service::open_with_dialog(path)
}

#[tauri::command]
pub fn open_terminal(path: String) -> Result<(), AppError> {
    services::file_service::open_terminal(path)
}
