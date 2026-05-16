pub mod migrations;

pub use migrations::init_database;

use rusqlite::Connection;
use std::fs;
use tauri::Manager;

pub fn get_db_connection(app: &tauri::AppHandle) -> Result<Connection, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let db_path = dir.join("oopslauncher.db");

    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    Ok(conn)
}
