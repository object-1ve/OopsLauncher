use rusqlite::Connection;

pub fn init_database(app: &tauri::AppHandle) -> Result<(), String> {
    let conn = super::get_db_connection(app)?;

    conn.execute_batch(
        "
        PRAGMA journal_mode = WAL;
        PRAGMA synchronous = NORMAL;
    ",
    )
    .map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS folder_sizes (
            path TEXT PRIMARY KEY,
            size INTEGER NOT NULL,
            last_updated INTEGER NOT NULL
        )",
        [],
    )
    .map_err(|e| e.to_string())?;

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
            modified_at INTEGER,
            notes TEXT,
            is_pinned INTEGER DEFAULT 0,
            UNIQUE(category, path)
        )",
        [],
    )
    .map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS categories (
            id TEXT PRIMARY KEY,
            parent_id TEXT,
            name TEXT NOT NULL,
            icon TEXT,
            sort_order INTEGER NOT NULL DEFAULT 0
        )",
        [],
    )
    .map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )",
        [],
    )
    .map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS favorites (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            path TEXT NOT NULL UNIQUE,
            sort_order INTEGER NOT NULL DEFAULT 0,
            open_count INTEGER NOT NULL DEFAULT 0
        )",
        [],
    )
    .map_err(|e| e.to_string())?;

    migrate_files_table(&conn)?;
    migrate_categories_table(&conn)?;
    migrate_favorites_table(&conn)?;

    Ok(())
}

fn migrate_files_table(conn: &Connection) -> Result<(), String> {
    let mut stmt = conn
        .prepare("PRAGMA table_info(files)")
        .map_err(|e| e.to_string())?;
    let columns: Vec<String> = stmt
        .query_map([], |row| Ok(row.get(1)?))
        .map_err(|e| e.to_string())?
        .filter_map(|result| result.ok())
        .collect();

    let add_column = |conn: &Connection, col: &str, def: &str| -> Result<(), String> {
        if !columns.contains(&col.to_string()) {
            println!("Adding {} column to files table...", col);
            conn.execute(&format!("ALTER TABLE files ADD COLUMN {} {}", col, def), [])
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    };

    add_column(conn, "category", "TEXT NOT NULL DEFAULT 'main'")?;
    add_column(conn, "open_count", "INTEGER DEFAULT 0")?;
    add_column(conn, "content", "TEXT")?;
    add_column(conn, "display_name", "TEXT")?;
    add_column(conn, "created_at", "INTEGER")?;
    add_column(conn, "modified_at", "INTEGER")?;
    add_column(conn, "notes", "TEXT")?;
    add_column(conn, "is_pinned", "INTEGER DEFAULT 0")?;

    conn.execute(
        "UPDATE files SET display_name = name WHERE display_name IS NULL OR display_name = ''",
        [],
    )
    .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("PRAGMA index_list(files)")
        .map_err(|e| e.to_string())?;
    let indexes: Vec<(String, bool)> = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(1)?, row.get::<_, bool>(2)?))
        })
        .map_err(|e| e.to_string())?
        .filter_map(|result| result.ok())
        .collect();

    let mut has_global_unique_path = false;
    for (idx_name, is_unique) in indexes {
        if is_unique {
            let mut info_stmt = conn
                .prepare(&format!("PRAGMA index_info({})", idx_name))
                .map_err(|e| e.to_string())?;
            let idx_columns: Vec<String> = info_stmt
                .query_map([], |row| Ok(row.get::<_, String>(2)?))
                .map_err(|e| e.to_string())?
                .filter_map(|result| result.ok())
                .collect();

            if idx_columns == vec!["path".to_string()] {
                has_global_unique_path = true;
                break;
            }
        }
    }

    if has_global_unique_path {
        let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;

        tx.execute("ALTER TABLE files RENAME TO files_old", [])
            .map_err(|e| e.to_string())?;

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
                modified_at INTEGER,
                notes TEXT,
                is_pinned INTEGER DEFAULT 0,
                UNIQUE(category, path)
            )",
            [],
        )
        .map_err(|e| e.to_string())?;

        tx.execute(
            "INSERT OR IGNORE INTO files (
                id, name, display_name, path, size, type, icon, content, category, open_count, created_at, modified_at, notes, is_pinned
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
                created_at,
                notes,
                0
            FROM files_old",
            [],
        )
        .map_err(|e| e.to_string())?;

        tx.execute("DROP TABLE files_old", [])
            .map_err(|e| e.to_string())?;
        tx.commit().map_err(|e| e.to_string())?;
    }

    Ok(())
}

fn migrate_categories_table(conn: &Connection) -> Result<(), String> {
    let mut stmt = conn
        .prepare("PRAGMA table_info(categories)")
        .map_err(|e| e.to_string())?;
    let cat_columns: Vec<String> = stmt
        .query_map([], |row| Ok(row.get(1)?))
        .map_err(|e| e.to_string())?
        .filter_map(|result| result.ok())
        .collect();

    if !cat_columns.contains(&"parent_id".to_string()) {
        conn.execute("ALTER TABLE categories ADD COLUMN parent_id TEXT", [])
            .map_err(|e| e.to_string())?;
    }
    if !cat_columns.contains(&"icon".to_string()) {
        conn.execute("ALTER TABLE categories ADD COLUMN icon TEXT", [])
            .map_err(|e| e.to_string())?;
    }
    if !cat_columns.contains(&"sort_order".to_string()) {
        conn.execute(
            "ALTER TABLE categories ADD COLUMN sort_order INTEGER NOT NULL DEFAULT 0",
            [],
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}

fn migrate_favorites_table(conn: &Connection) -> Result<(), String> {
    let mut stmt = conn
        .prepare("PRAGMA table_info(favorites)")
        .map_err(|e| e.to_string())?;
    let columns: Vec<String> = stmt
        .query_map([], |row| Ok(row.get(1)?))
        .map_err(|e| e.to_string())?
        .filter_map(|result| result.ok())
        .collect();

    if !columns.contains(&"open_count".to_string()) {
        conn.execute(
            "ALTER TABLE favorites ADD COLUMN open_count INTEGER NOT NULL DEFAULT 0",
            [],
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}
