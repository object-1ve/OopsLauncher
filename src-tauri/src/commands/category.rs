use crate::{error::AppError, models::Category, services};

#[tauri::command]
pub fn save_categories_to_db(app: tauri::AppHandle, categories: Vec<Category>) -> Result<(), AppError> {
    services::category_service::save_categories(&app, categories)
}

#[tauri::command]
pub fn load_categories_from_db(app: tauri::AppHandle) -> Result<Vec<Category>, AppError> {
    services::category_service::load_categories(&app)
}

#[tauri::command]
pub fn rename_category_in_db(app: tauri::AppHandle, id: String, new_name: String) -> Result<(), AppError> {
    services::category_service::rename_category(&app, id, new_name)
}

#[tauri::command]
pub fn delete_category_from_db(app: tauri::AppHandle, id: String) -> Result<(), AppError> {
    services::category_service::delete_category(&app, id)
}
