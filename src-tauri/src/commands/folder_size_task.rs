use crate::{error::AppError, services};

/// Starts a background folder size task and returns the task id.
#[tauri::command]
pub fn start_folder_size_task(
    app: tauri::AppHandle,
    paths: Vec<String>,
) -> Result<String, AppError> {
    services::folder_size_task_service::start_folder_size_task(app, paths)
}
