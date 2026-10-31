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
pub fn open_path(path: String, args: Option<String>) -> Result<(), AppError> {
    services::file_service::open_path(path, args)
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

/// 列出目录。声明为 async：内部要 read_dir + 读 SQLite 目录大小缓存，
/// 大目录同步执行会阻塞 WebView 主线程（切目录时卡住）。
#[tauri::command(async)]
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

/// 批量取图标（资源管理器当前可见项）。声明为 async：命令体在 Tauri 异步运行时线程池执行，
/// 不会阻塞 WebView 主线程；图标提取要走 Shell COM，放在主线程会直接卡住界面。
#[tauri::command(async)]
pub fn get_file_icons(
    paths: Vec<String>,
    size: Option<u32>,
) -> Result<crate::models::FileIcons, AppError> {
    let size = size.unwrap_or(crate::icon::DEFAULT_ICON_SIZE);
    services::file_service::get_file_icons(paths, size)
}

#[tauri::command]
pub async fn search_windows_index(query: String) -> Result<Vec<FileInfo>, AppError> {
    services::file_service::search_windows_index(query)
}

#[tauri::command]
pub fn create_file(parent_dir: String, file_name: String) -> Result<String, AppError> {
    services::file_service::create_file(parent_dir, file_name)
}

#[tauri::command]
pub fn create_folder(parent_dir: String, folder_name: String) -> Result<String, AppError> {
    services::file_service::create_folder(parent_dir, folder_name)
}

#[tauri::command]
pub fn rename_path(path: String, new_name: String) -> Result<String, AppError> {
    services::file_service::rename_path(path, new_name)
}

#[tauri::command]
pub fn scan_start_menu_programs(app: tauri::AppHandle) -> Result<Vec<FileInfo>, AppError> {
    services::file_service::scan_start_menu_programs(&app)
}

#[tauri::command]
pub fn add_favorite(app: tauri::AppHandle, path: String, name: String) -> Result<services::file_service::Favorite, AppError> {
    services::file_service::add_favorite(&app, path, name)
}

#[tauri::command]
pub fn remove_favorite(app: tauri::AppHandle, path: String) -> Result<(), AppError> {
    services::file_service::remove_favorite(&app, path)
}

#[tauri::command]
pub fn get_favorites(app: tauri::AppHandle) -> Result<Vec<services::file_service::Favorite>, AppError> {
    services::file_service::get_favorites(&app)
}

#[tauri::command]
pub fn is_favorite(app: tauri::AppHandle, path: String) -> Result<bool, AppError> {
    services::file_service::is_favorite(&app, path)
}

#[tauri::command]
pub fn increment_open_count(app: tauri::AppHandle, path: String) -> Result<(), AppError> {
    services::file_service::increment_open_count(&app, path)
}
