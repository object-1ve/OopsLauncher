use crate::db::get_db_connection;
use crate::models::LauncherState;
use rusqlite::params;

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
    let conn = get_db_connection(&app)?;
    let safe_state = sanitize_launcher_state(state);
    let state_json = serde_json::to_string(&safe_state).map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO settings (key, value) VALUES ('launcher_state', ?1)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![state_json],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn load_launcher_state_from_db(app: tauri::AppHandle) -> Result<LauncherState, String> {
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

    Ok(LauncherState {
        current_category: "all_files".to_string(),
        sort_method: "openCount".to_string(),
        sort_order: "desc".to_string(),
        classify_method: "none".to_string(),
    })
}
