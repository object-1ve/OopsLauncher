use crate::db::get_db_connection;
use crate::models::LauncherState;
use std::fs;
use tauri::Manager;

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
    }
}

#[tauri::command]
pub fn save_launcher_state_to_db(app: tauri::AppHandle, state: LauncherState) -> Result<(), String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join("launcher_state.json");
    
    let safe_state = sanitize_launcher_state(state);
    let state_json = serde_json::to_string_pretty(&safe_state).map_err(|e| e.to_string())?;
    fs::write(path, state_json).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn load_launcher_state_from_db(app: tauri::AppHandle) -> Result<LauncherState, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let path = dir.join("launcher_state.json");
    
    if path.exists() {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(state) = serde_json::from_str::<LauncherState>(&content) {
                return Ok(sanitize_launcher_state(state));
            }
        }
    }

    // 降级尝试从数据库加载 (迁移用)
    let db_result = (|| -> Result<LauncherState, String> {
        let conn = get_db_connection(&app)?;
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
        Err("Not found in DB".to_string())
    })();

    if let Ok(state) = db_result {
        return Ok(state);
    }

    Ok(LauncherState {
        current_category: "all_files".to_string(),
        sort_method: "openCount".to_string(),
        sort_order: "desc".to_string(),
        classify_method: "none".to_string(),
    })
}

// ================== JSON Settings ==================

#[tauri::command]
pub fn save_settings_to_json(app: tauri::AppHandle, settings: serde_json::Value) -> Result<(), String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join("settings.json");
    
    let content = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    fs::write(path, content).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn load_settings_from_json(app: tauri::AppHandle) -> Result<serde_json::Value, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let path = dir.join("settings.json");
    
    if !path.exists() {
        return Ok(serde_json::Value::Null);
    }
    
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let settings = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    Ok(settings)
}
