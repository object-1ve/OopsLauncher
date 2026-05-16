use std::fs;
use std::path::Path;
use std::process::Command;

use crate::db::get_db_connection;
use crate::error::AppError;
use crate::icon::get_file_icon_base64;
use crate::models::FileInfo;
use crate::utils::{resolve_shortcut, to_abs_path};
use rusqlite::params;

pub fn save_files(app: &tauri::AppHandle, files: Vec<FileInfo>) -> Result<(), AppError> {
    let mut conn = get_db_connection(app)?;
    let tx = conn.transaction()?;

    tx.execute("DELETE FROM files", [])?;

    let mut stmt = tx.prepare(
        "INSERT OR REPLACE INTO files (id, name, display_name, path, size, type, icon, content, category, open_count, created_at, notes, is_pinned) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )?;

    for file in &files {
        let path = file.path.trim().to_string();
        if path.is_empty() {
            println!("Skipping file with empty path: {}", file.name);
            continue;
        }

        let category_id = file.category.as_deref().unwrap_or("main");
        let created_at = file.created_at.unwrap_or_else(|| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as i64
        });

        stmt.execute(params![
            &file.id,
            &file.name,
            &file.display_name,
            &path,
            file.size as i64,
            &file.r#type,
            &file.icon,
            &file.content,
            category_id,
            file.open_count.unwrap_or(0) as i64,
            created_at,
            &file.notes,
            if file.is_pinned.unwrap_or(false) { 1 } else { 0 }
        ])
        .map_err(|e| {
            println!("Failed to save file {} to DB: {}", file.name, e);
            AppError::Database(format!("Failed to save file {}: {}", file.name, e))
        })?;
    }

    drop(stmt);
    tx.commit()?;
    println!("Successfully saved {} files to database.", files.len());
    Ok(())
}

pub fn load_files(app: &tauri::AppHandle) -> Result<Vec<FileInfo>, AppError> {
    let conn = get_db_connection(app)?;

    let mut stmt = conn
        .prepare(
            "SELECT id, name, display_name, path, size, type, icon, content, category, open_count, created_at, notes, is_pinned FROM files ORDER BY open_count DESC",
        )?;

    let files_iter = stmt.query_map([], |row| {
        Ok(FileInfo {
            id: row.get(0)?,
            name: row.get(1)?,
            display_name: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
            path: row.get(3)?,
            size: row.get::<_, i64>(4).unwrap_or(0) as u64,
            r#type: row.get::<_, String>(5).unwrap_or_default(),
            icon: row.get::<_, String>(6).unwrap_or_default(),
            content: row.get(7).ok(),
            category: row.get(8).ok(),
            open_count: Some(row.get::<_, i64>(9).unwrap_or(0) as u64),
            created_at: row.get(10).ok(),
            notes: row.get(11).ok(),
            is_pinned: Some(row.get::<_, i64>(12).unwrap_or(0) != 0),
        })
    })?;

    let mut files = Vec::new();
    for file_result in files_iter {
        match file_result {
            Ok(file) => files.push(file),
            Err(e) => println!("Error mapping file row: {}", e),
        }
    }

    println!("Successfully loaded {} files from database.", files.len());
    Ok(files)
}

pub fn get_file_info(path: String) -> Result<FileInfo, AppError> {
    let abs_path = to_abs_path(&path)?;
    let original_p = Path::new(&abs_path);
    let original_name = original_p
        .file_name()
        .ok_or_else(|| AppError::NotFound("Invalid file name".into()))?
        .to_string_lossy()
        .to_string();

    let target_path = resolve_shortcut(&abs_path);
    let is_shortcut = target_path != abs_path;
    let p = Path::new(&target_path);

    if !p.exists() {
        return Err(AppError::NotFound(format!(
            "File not found: {}",
            target_path
        )));
    }

    let metadata = fs::metadata(p)?;
    let name = p
        .file_name()
        .ok_or_else(|| AppError::NotFound("Invalid file name".into()))?
        .to_string_lossy()
        .to_string();

    let extension = p
        .extension()
        .map(|e| e.to_string_lossy().to_string())
        .unwrap_or_default();

    let icon = match get_file_icon_base64(p) {
        Ok(icon) => icon,
        Err(e) => {
            println!("Failed to get icon: {}", e);
            String::new()
        }
    };

    let mut content = None;
    if metadata.is_file() && metadata.len() < 1024 * 1024 {
        let text_extensions = [
            "txt", "md", "json", "js", "ts", "html", "css", "py", "rs", "c", "cpp", "h", "hpp",
            "go", "sql", "yml", "yaml", "toml", "xml",
        ];
        if text_extensions.contains(&extension.to_lowercase().as_str()) {
            if let Ok(c) = fs::read_to_string(p) {
                content = Some(c);
            }
        }
    }

    let display_name_source = if is_shortcut {
        &original_name
    } else {
        &name
    };

    let display_name = if let Some(last_dot_idx) = display_name_source.rfind('.') {
        let ext = &display_name_source[last_dot_idx + 1..].to_lowercase();
        let common_extensions = [
            "exe", "lnk", "js", "ts", "html", "css", "py", "rs", "c", "cpp", "h", "hpp", "go",
            "sql", "yml", "yaml", "toml", "xml", "txt", "md", "json",
        ];
        if common_extensions.contains(&ext.as_str()) {
            display_name_source[..last_dot_idx].to_string()
        } else {
            display_name_source.clone()
        }
    } else {
        display_name_source.clone()
    };

    let created_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64;

    Ok(FileInfo {
        id: String::new(),
        name,
        display_name,
        path: target_path,
        size: metadata.len(),
        r#type: extension,
        icon,
        content,
        category: None,
        open_count: None,
        created_at: Some(created_at),
        notes: None,
        is_pinned: Some(false),
    })
}

pub fn open_path(path: String) -> Result<(), AppError> {
    let path = path.trim().to_string();
    if path.is_empty() {
        return Err(AppError::Other("path is empty".into()));
    }

    let path = to_abs_path(&path)?;
    let p = Path::new(&path);
    if !p.exists() {
        return Err(AppError::NotFound(format!("path does not exist: {}", path)));
    }

    #[cfg(target_os = "windows")]
    {
        Command::new("explorer.exe")
            .arg(&path)
            .spawn()
            .map_err(|e| AppError::Platform(e.to_string()))?;
        return Ok(());
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(&path)
            .spawn()
            .map_err(|e| AppError::Platform(e.to_string()))?;
        return Ok(());
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(|e| AppError::Platform(e.to_string()))?;
        return Ok(());
    }
}

pub fn open_file_location(path: String) -> Result<(), AppError> {
    let path = path.trim().to_string();
    if path.is_empty() {
        return Err(AppError::Other("path is empty".into()));
    }

    let path = to_abs_path(&path)?;
    let p = Path::new(&path);
    if !p.exists() {
        return Err(AppError::NotFound(format!("path does not exist: {}", path)));
    }

    #[cfg(target_os = "windows")]
    {
        Command::new("explorer.exe")
            .args(["/select,", &path])
            .spawn()
            .map_err(|e| AppError::Platform(e.to_string()))?;
        return Ok(());
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .args(["-R", &path])
            .spawn()
            .map_err(|e| AppError::Platform(e.to_string()))?;
        return Ok(());
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        if let Some(parent) = p.parent() {
            Command::new("xdg-open")
                .arg(parent)
                .spawn()
                .map_err(|e| AppError::Platform(e.to_string()))?;
        } else {
            Command::new("xdg-open")
                .arg(&path)
                .spawn()
                .map_err(|e| AppError::Platform(e.to_string()))?;
        }
        return Ok(());
    }
}

pub fn open_with_dialog(path: String) -> Result<(), AppError> {
    let path = path.trim().to_string();
    if path.is_empty() {
        return Err(AppError::Other("path is empty".into()));
    }

    let path = to_abs_path(&path)?;
    let p = Path::new(&path);
    if !p.exists() {
        return Err(AppError::NotFound(format!("path does not exist: {}", path)));
    }

    #[cfg(target_os = "windows")]
    {
        Command::new("rundll32.exe")
            .args(["shell32.dll,OpenAs_RunDLL", &path])
            .spawn()
            .map_err(|e| AppError::Platform(e.to_string()))?;
        return Ok(());
    }

    #[cfg(not(target_os = "windows"))]
    {
        return Err(AppError::Platform(
            "当前平台暂不支持【打开方式】对话框".into(),
        ));
    }
}

pub fn open_terminal(path: String) -> Result<(), AppError> {
    let path = path.trim().to_string();
    if path.is_empty() {
        return Err(AppError::Other("path is empty".into()));
    }

    let path = to_abs_path(&path)?;
    let resolved_path = resolve_shortcut(&path);
    let p = Path::new(&resolved_path);

    if !p.exists() {
        return Err(AppError::NotFound(format!(
            "path does not exist: {}",
            resolved_path
        )));
    }

    let target_dir = if p.is_dir() {
        p.to_path_buf()
    } else {
        p.parent()
            .ok_or_else(|| {
                AppError::NotFound(format!("cannot resolve parent directory: {}", resolved_path))
            })?
            .to_path_buf()
    };

    #[cfg(target_os = "windows")]
    {
        let target_str = target_dir.to_string_lossy();
        let escaped_path = target_str.replace('\'', "''");
        let ps_command = format!("Set-Location -LiteralPath '{}'", escaped_path);

        Command::new("cmd")
            .args([
                "/c",
                "start",
                "powershell",
                "-NoExit",
                "-Command",
                &ps_command,
            ])
            .current_dir(&target_dir)
            .spawn()
            .map_err(|e| AppError::Platform(format!("无法启动终端窗口: {}", e)))?;

        return Ok(());
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .args(["-a", "Terminal", target_dir.to_string_lossy().as_ref()])
            .current_dir(&target_dir)
            .spawn()
            .map_err(|e| AppError::Platform(format!("无法启动终端: {}", e)))?;
        return Ok(());
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let dir = target_dir.to_string_lossy().to_string();
        let candidates: [(&str, Vec<&str>); 4] = [
            ("x-terminal-emulator", vec!["--working-directory", &dir]),
            ("gnome-terminal", vec!["--working-directory", &dir]),
            ("konsole", vec!["--workdir", &dir]),
            ("xfce4-terminal", vec!["--working-directory", &dir]),
        ];

        for (cmd, args) in &candidates {
            if Command::new(cmd)
                .args(args)
                .current_dir(&target_dir)
                .spawn()
                .is_ok()
            {
                return Ok(());
            }
        }
        return Err(AppError::Platform("未找到可用终端程序".into()));
    }
}
