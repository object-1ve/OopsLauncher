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
pub fn check_paths_exist(paths: Vec<String>) -> Result<Vec<bool>, AppError> {
    services::file_service::check_paths_exist(paths)
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

#[tauri::command]
pub fn list_directory(app: tauri::AppHandle, path: String) -> Result<Vec<FileInfo>, AppError> {
    services::file_service::list_directory(&app, path)
}

#[tauri::command]
pub fn delete_to_trash(path: String) -> Result<(), AppError> {
    services::file_service::delete_to_trash(path)
}

#[tauri::command]
pub fn copy_file_or_dir(src: String, dst: String) -> Result<(), AppError> {
    services::file_service::copy_file_or_dir(src, dst)
}

#[tauri::command]
pub fn move_file_or_dir(src: String, dst: String) -> Result<(), AppError> {
    services::file_service::move_file_or_dir(src, dst)
}

#[tauri::command]
pub fn calculate_dir_sizes(
    paths: Vec<String>,
) -> Result<std::collections::HashMap<String, u64>, AppError> {
    services::file_service::calculate_dir_sizes(paths)
}

#[tauri::command]
pub fn calculate_dir_size(path: String) -> Result<u64, AppError> {
    services::file_service::calculate_dir_size_single(path)
}

#[tauri::command]
pub fn get_file_icon(path: String) -> Result<String, AppError> {
    use std::path::Path;
    crate::icon::get_file_icon_base64(Path::new(&path)).map_err(|e| AppError::Other(e.to_string()))
}

#[tauri::command]
pub async fn search_windows_index(query: String) -> Result<Vec<FileInfo>, AppError> {
    services::file_service::search_windows_index(query)
}

#[tauri::command]
pub fn scan_start_menu_programs(app: tauri::AppHandle) -> Result<Vec<FileInfo>, AppError> {
    services::file_service::scan_start_menu_programs(&app)
}
