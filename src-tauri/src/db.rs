use rusqlite::Connection;
use std::fs;
use tauri::Manager;

// 获取数据库连接
pub fn get_db_connection(app: &tauri::AppHandle) -> Result<Connection, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let db_path = dir.join("oopslauncher.db");
    
    let conn = Connection::open(db_path)
        .map_err(|e| e.to_string())?;
    
    // 初始化数据库表结构
    init_database(&conn)?;
    
    Ok(conn)
}

// 初始化数据库表结构
pub fn init_database(conn: &Connection) -> Result<(), String> {
    // 创建文件表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS files (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            display_name TEXT NOT NULL,
            path TEXT NOT NULL,
            size INTEGER,
            type TEXT,
            icon TEXT,
            content TEXT,
            category TEXT NOT NULL DEFAULT 'main',
            open_count INTEGER DEFAULT 0,
            created_at INTEGER
        )",
        []
    ).map_err(|e| e.to_string())?;

    // 创建分类表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS categories (
            id TEXT PRIMARY KEY,
            parent_id TEXT,
            name TEXT NOT NULL,
            icon TEXT,
            sort_order INTEGER NOT NULL DEFAULT 0
        )",
        []
    ).map_err(|e| e.to_string())?;
    
    // 检查并同步分类表结构
    let mut stmt = conn.prepare("PRAGMA table_info(categories)").map_err(|e| e.to_string())?;
    let cat_columns: Vec<String> = stmt.query_map([], |row| {
        Ok(row.get(1)?)
    }).map_err(|e| e.to_string())?
    .filter_map(|result| result.ok())
    .collect();

    if !cat_columns.contains(&"parent_id".to_string()) {
        conn.execute("ALTER TABLE categories ADD COLUMN parent_id TEXT", []).map_err(|e| e.to_string())?;
    }
    if !cat_columns.contains(&"icon".to_string()) {
        conn.execute("ALTER TABLE categories ADD COLUMN icon TEXT", []).map_err(|e| e.to_string())?;
    }
    if !cat_columns.contains(&"sort_order".to_string()) {
        conn.execute("ALTER TABLE categories ADD COLUMN sort_order INTEGER NOT NULL DEFAULT 0", []).map_err(|e| e.to_string())?;
    }

    // 检查并添加category列（如果不存在）
    // SQLite不支持ALTER TABLE中的IF NOT EXISTS，所以需要先检查列是否存在
    let mut stmt = conn.prepare("PRAGMA table_info(files)").map_err(|e| e.to_string())?;
    let columns: Vec<String> = stmt.query_map([], |row| {
        Ok(row.get(1)?) // 获取列名
    }).map_err(|e| e.to_string())?
    .filter_map(|result| result.ok())
    .collect();
    
    // 如果category列不存在，则添加它
    if !columns.contains(&"category".to_string()) {
        conn.execute(
            "ALTER TABLE files ADD COLUMN category TEXT NOT NULL DEFAULT 'main'",
            []
        ).map_err(|e| e.to_string())?;
    }
    
    // 如果open_count列不存在，则添加它
    if !columns.contains(&"open_count".to_string()) {
        conn.execute(
            "ALTER TABLE files ADD COLUMN open_count INTEGER DEFAULT 0",
            []
        ).map_err(|e| e.to_string())?;
    }

    // 如果content列不存在，则添加它
    if !columns.contains(&"content".to_string()) {
        conn.execute(
            "ALTER TABLE files ADD COLUMN content TEXT",
            []
        ).map_err(|e| e.to_string())?;
    }
    
    // 如果display_name列不存在，则添加它
    if !columns.contains(&"display_name".to_string()) {
        conn.execute(
            "ALTER TABLE files ADD COLUMN display_name TEXT NOT NULL DEFAULT ''",
            []
        ).map_err(|e| e.to_string())?;
    }
    
    // 如果created_at列不存在，则添加它
    if !columns.contains(&"created_at".to_string()) {
        conn.execute(
            "ALTER TABLE files ADD COLUMN created_at INTEGER",
            []
        ).map_err(|e| e.to_string())?;
    }
    
    Ok(())
}
