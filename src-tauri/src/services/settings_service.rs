use std::fs;

use crate::db::get_db_connection;
use crate::error::AppError;
use crate::models::LauncherState;
use tauri::{Emitter, Manager};

fn sanitize_launcher_state(state: LauncherState) -> LauncherState {
    let current_category = if state.current_category.trim().is_empty() {
        "all_files".to_string()
    } else {
        state.current_category
    };
    let sort_method = match state.sort_method.as_str() {
        "name" | "openCount" | "created_at" => state.sort_method,
        _ => "openCount".to_string(),
    };
    let sort_order = match state.sort_order.as_str() {
        "asc" | "desc" => state.sort_order,
        _ => "desc".to_string(),
    };
    let classify_method = match state.classify_method.as_str() {
        "none" | "type" => state.classify_method,
        _ => "none".to_string(),
    };
    LauncherState {
        current_category,
        sort_method,
        sort_order,
        classify_method,
        explorer_path: state.explorer_path,
    }
}

pub fn save_launcher_state(app: &tauri::AppHandle, state: LauncherState) -> Result<(), AppError> {
    let dir = app.path().app_data_dir()?;
    fs::create_dir_all(&dir)?;
    let path = dir.join("launcher_state.json");

    let safe_state = sanitize_launcher_state(state);
    let state_json = serde_json::to_string_pretty(&safe_state)?;
    fs::write(path, state_json)?;
    Ok(())
}

pub fn load_launcher_state(app: &tauri::AppHandle) -> Result<LauncherState, AppError> {
    let dir = app.path().app_data_dir()?;
    let path = dir.join("launcher_state.json");

    if path.exists() {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(state) = serde_json::from_str::<LauncherState>(&content) {
                return Ok(sanitize_launcher_state(state));
            }
        }
    }

    // Fallback: try loading from SQLite settings table (migration)
    let db_result = (|| -> Result<LauncherState, AppError> {
        let conn = get_db_connection(app)?;
        let saved_json: Option<String> = conn
            .query_row(
                "SELECT value FROM settings WHERE key = 'launcher_state' LIMIT 1",
                [],
                |row| row.get(0),
            )
            .ok();

        if let Some(content) = saved_json {
            if let Ok(state) = serde_json::from_str::<LauncherState>(&content) {
                return Ok(sanitize_launcher_state(state));
            }
        }
        Err(AppError::NotFound("Not found in DB".into()))
    })();

    if let Ok(state) = db_result {
        return Ok(state);
    }

    Ok(LauncherState {
        current_category: "all_files".to_string(),
        sort_method: "openCount".to_string(),
        sort_order: "desc".to_string(),
        classify_method: "none".to_string(),
        explorer_path: None,
    })
}

pub fn save_settings(app: &tauri::AppHandle, settings: serde_json::Value) -> Result<(), AppError> {
    let dir = app.path().app_data_dir()?;
    fs::create_dir_all(&dir)?;
    let path = dir.join("settings.json");

    let content = serde_json::to_string_pretty(&settings)?;
    fs::write(path, content)?;

    // Notify all windows about the settings change
    let _ = app.emit("settings-changed", settings);

    Ok(())
}

pub fn load_settings(app: &tauri::AppHandle) -> Result<serde_json::Value, AppError> {
    let dir = app.path().app_data_dir()?;
    let path = dir.join("settings.json");

    if !path.exists() {
        return Ok(serde_json::Value::Null);
    }

    let content = fs::read_to_string(path)?;
    let settings = serde_json::from_str(&content)?;
    Ok(settings)
}
