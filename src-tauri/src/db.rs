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
    
    Ok(conn)
}

pub fn init_database(app: &tauri::AppHandle) -> Result<(), String> {
    let conn = get_db_connection(app)?;
    
    // 设置 WAL 模式以提高并发性能和稳定性
    // 设置同步模式为 NORMAL，在保证安全的同时提高写入性能
    // 使用 execute_batch 避免 PRAGMA 返回结果导致的 execute 错误
    conn.execute_batch("
        PRAGMA journal_mode = WAL;
        PRAGMA synchronous = NORMAL;
    ").map_err(|e| e.to_string())?;
    
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
            created_at INTEGER,
            notes TEXT,
            is_pinned INTEGER DEFAULT 0,
            UNIQUE(category, path)
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

    conn.execute(
        "CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )",
        []
    ).map_err(|e| e.to_string())?;

    // 检查并手动进行数据库迁移
    // 获取当前 files 表的所有列名
    let mut stmt = conn.prepare("PRAGMA table_info(files)").map_err(|e| e.to_string())?;
    let columns: Vec<String> = stmt.query_map([], |row| {
        Ok(row.get(1)?) // 获取列名
    }).map_err(|e| e.to_string())?
    .filter_map(|result| result.ok())
    .collect();
    
    // 如果category列不存在，则添加它
    if !columns.contains(&"category".to_string()) {
        println!("Adding category column to files table...");
        conn.execute(
            "ALTER TABLE files ADD COLUMN category TEXT NOT NULL DEFAULT 'main'",
            []
        ).map_err(|e| e.to_string())?;
    }
    
    // 如果open_count列不存在，则添加它
    if !columns.contains(&"open_count".to_string()) {
        println!("Adding open_count column to files table...");
        conn.execute(
            "ALTER TABLE files ADD COLUMN open_count INTEGER DEFAULT 0",
            []
        ).map_err(|e| e.to_string())?;
    }

    // 如果content列不存在，则添加它
    if !columns.contains(&"content".to_string()) {
        println!("Adding content column to files table...");
        conn.execute(
            "ALTER TABLE files ADD COLUMN content TEXT",
            []
        ).map_err(|e| e.to_string())?;
    }

    // 如果display_name列不存在，则添加它
    if !columns.contains(&"display_name".to_string()) {
        println!("Adding display_name column to files table...");
        conn.execute(
            "ALTER TABLE files ADD COLUMN display_name TEXT",
            []
        ).map_err(|e| e.to_string())?;
    }
    
    // 始终确保 display_name 不为 NULL（如果之前列存在但有 NULL 值）
    conn.execute(
        "UPDATE files SET display_name = name WHERE display_name IS NULL OR display_name = ''",
        []
    ).map_err(|e| e.to_string())?;

    // 如果created_at列不存在，则添加它
    if !columns.contains(&"created_at".to_string()) {
        println!("Adding created_at column to files table...");
        conn.execute(
            "ALTER TABLE files ADD COLUMN created_at INTEGER",
            []
        ).map_err(|e| e.to_string())?;
    }
    
    // 如果notes列不存在，则添加它
    if !columns.contains(&"notes".to_string()) {
        println!("Adding notes column to files table...");
        conn.execute(
            "ALTER TABLE files ADD COLUMN notes TEXT",
            []
        ).map_err(|e| e.to_string())?;
    }

    if !columns.contains(&"is_pinned".to_string()) {
        println!("Adding is_pinned column to files table...");
        conn.execute(
            "ALTER TABLE files ADD COLUMN is_pinned INTEGER DEFAULT 0",
            []
        ).map_err(|e| e.to_string())?;
    }

    // 检查是否仍存在旧的 UNIQUE(path) 约束
    let mut stmt = conn.prepare("PRAGMA index_list(files)").map_err(|e| e.to_string())?;
    let indexes: Vec<(String, bool)> = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(1)?, row.get::<_, bool>(2)?))
    }).map_err(|e| e.to_string())?
    .filter_map(|result| result.ok())
    .collect();

    let mut has_global_unique_path = false;
    for (idx_name, is_unique) in indexes {
        if is_unique {
            let mut info_stmt = conn.prepare(&format!("PRAGMA index_info({})", idx_name)).map_err(|e| e.to_string())?;
            let columns: Vec<String> = info_stmt.query_map([], |row| {
                Ok(row.get::<_, String>(2)?)
            }).map_err(|e| e.to_string())?
            .filter_map(|result| result.ok())
            .collect();

            if columns == vec!["path".to_string()] {
                has_global_unique_path = true;
                break;
            }
        }
    }

    if has_global_unique_path {
        let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;

        tx.execute(
            "ALTER TABLE files RENAME TO files_old",
            []
        ).map_err(|e| e.to_string())?;

        tx.execute(
            "CREATE TABLE files (
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
                created_at INTEGER,
                notes TEXT,
                is_pinned INTEGER DEFAULT 0,
                UNIQUE(category, path)
            )",
            []
        ).map_err(|e| e.to_string())?;

        tx.execute(
            "INSERT OR IGNORE INTO files (
                id, name, display_name, path, size, type, icon, content, category, open_count, created_at, notes, is_pinned
            )
            SELECT
                id,
                name,
                COALESCE(NULLIF(display_name, ''), name),
                path,
                size,
                COALESCE(type, ''),
                COALESCE(icon, ''),
                content,
                COALESCE(category, 'main'),
                COALESCE(open_count, 0),
                created_at,
                notes,
                0
            FROM files_old",
            []
        ).map_err(|e| e.to_string())?;

        tx.execute("DROP TABLE files_old", []).map_err(|e| e.to_string())?;
        tx.commit().map_err(|e| e.to_string())?;
    }

    // 检查并手动进行 categories 表的数据库迁移
    let mut cat_stmt = conn.prepare("PRAGMA table_info(categories)").map_err(|e| e.to_string())?;
    let cat_columns: Vec<String> = cat_stmt.query_map([], |row| {
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
    
    Ok(())
}
