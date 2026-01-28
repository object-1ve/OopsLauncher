use std::fs;
use std::path::Path;
use std::process::Command;
use crate::models::FileInfo;
use crate::db::get_db_connection;
use crate::utils::{to_abs_path, resolve_shortcut};
use crate::icon::get_file_icon_base64;

// 保存文件列表到SQLite数据库
#[tauri::command]
pub fn save_files_to_db(app: tauri::AppHandle, files: Vec<FileInfo>) -> Result<(), String> {
    let mut conn = get_db_connection(&app)?;
    
    // 开始事务
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    
    // 清空现有数据
    tx.execute("DELETE FROM files", []).map_err(|e| e.to_string())?;
    
    // 插入新数据
    let mut stmt = tx.prepare(
        "INSERT OR REPLACE INTO files (id, name, display_name, path, size, type, icon, content, category, open_count) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    ).map_err(|e| e.to_string())?;

    for mut file in files {
        file.path = to_abs_path(&file.path)?;
        
        // 获取分类信息，如果没有则默认为main
        let category = match file.category.as_ref() {
            Some(cat) => cat,
            None => "main"
        };
        
        // 获取打开次数，如果没有则默认为0
        let open_count = file.open_count.unwrap_or(0).to_string();
        
        stmt.execute(
            [
                &file.id,
                &file.name,
                &file.display_name,
                &file.path,
                &file.size.to_string(),
                &file.r#type,
                &file.icon,
                &file.content.unwrap_or_default(),
                category,
                &open_count
            ]
        ).map_err(|e| e.to_string())?;
    }
    
    // 释放 statement
    drop(stmt);

    // 提交事务
    tx.commit().map_err(|e| e.to_string())?;
    
    Ok(())
}

// 从SQLite数据库读取文件列表
#[tauri::command]
pub fn load_files_from_db(app: tauri::AppHandle) -> Result<Vec<FileInfo>, String> {
    let conn = get_db_connection(&app)?;
    
    let mut stmt = conn.prepare("SELECT id, name, display_name, path, size, type, icon, content, category, open_count FROM files").map_err(|e| e.to_string())?;
    
    let files_iter = stmt.query_map([], |row| {
        Ok(FileInfo {
            id: row.get(0)?,
            name: row.get(1)?,
            display_name: row.get(2)?,
            path: row.get(3)?,
            size: row.get::<_, i64>(4)? as u64,
            r#type: row.get(5)?,
            icon: row.get(6)?,
            content: row.get(7)?,
            category: Some(row.get(8)?),
            open_count: row.get(9)?
        })
    }).map_err(|e| e.to_string())?;
    
    let mut files = Vec::new();
    for file in files_iter {
        files.push(file.map_err(|e| e.to_string())?);
    }
    
    Ok(files)
}

#[tauri::command]
pub fn get_file_info(path: String) -> Result<FileInfo, String> {
    let abs_path = to_abs_path(&path)?;
    
    // 解析快捷方式
    let target_path = resolve_shortcut(&abs_path);
    let p = Path::new(&target_path);
    
    if !p.exists() {
        return Err(format!("File not found: {}", target_path));
    }

    let metadata = fs::metadata(p).map_err(|e| e.to_string())?;
    let name = p.file_name()
        .ok_or("Invalid file name")?
        .to_string_lossy()
        .to_string();
    
    let extension = p.extension()
        .map(|e| e.to_string_lossy().to_string())
        .unwrap_or_default();

    // 获取文件图标
    let icon = match get_file_icon_base64(p) {
        Ok(icon) => icon,
        Err(e) => {
            println!("Failed to get icon: {}", e);
            "".to_string()
        }
    };

    // 读取文件内容 (如果是文本文件且大小小于1MB)
    let mut content = None;
    if metadata.is_file() && metadata.len() < 1024 * 1024 {
        // 常见的文本文件后缀
        let text_extensions = ["txt", "md", "json", "js", "ts", "html", "css", "py", "rs", "c", "cpp", "h", "hpp", "go", "sql", "yml", "yaml", "toml", "xml"];
        if text_extensions.contains(&extension.to_lowercase().as_str()) {
            if let Ok(c) = fs::read_to_string(p) {
                content = Some(c);
            }
        }
    }

    // 生成 display_name，去掉常见后缀
    let display_name = if let Some(last_dot_idx) = name.rfind('.') {
        let ext = &name[last_dot_idx + 1..].to_lowercase();
        let common_extensions = ["exe", "js", "ts", "html", "css", "py", "rs", "c", "cpp", "h", "hpp", "go", "sql", "yml", "yaml", "toml", "xml", "txt", "md", "json"];
        if common_extensions.contains(&ext.as_str()) {
            name[..last_dot_idx].to_string()
        } else {
            name.clone()
        }
    } else {
        name.clone()
    };

    Ok(FileInfo {
        id: "".to_string(),
        name,
        display_name,
        path: target_path,
        size: metadata.len(),
        r#type: extension,
        icon,
        content,
        category: None,
        open_count: None,
    })
}

#[tauri::command]
pub fn open_path(path: String) -> Result<(), String> {
    let path = path.trim();
    if path.is_empty() {
        return Err("path is empty".to_string());
    }

    let path = to_abs_path(path)?;
    let p = Path::new(&path);
    if !p.exists() {
        return Err(format!("path does not exist: {}", path));
    }

    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(["/C", "start", ""])
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
        return Ok(());
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
        return Ok(());
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
        return Ok(());
    }
}

#[tauri::command]
pub fn open_file_location(path: String) -> Result<(), String> {
    let path = path.trim();
    if path.is_empty() {
        return Err("path is empty".to_string());
    }

    let path = to_abs_path(path)?;
    let p = Path::new(&path);
    if !p.exists() {
        return Err(format!("path does not exist: {}", path));
    }

    #[cfg(target_os = "windows")]
    {
        // On Windows, use explorer /select to open folder and select file
        Command::new("explorer.exe")
            .args(["/select,", &path])
            .spawn()
            .map_err(|e| e.to_string())?;
        return Ok(());
    }

    #[cfg(target_os = "macos")]
    {
        // On macOS, use open -R to reveal file in Finder
        Command::new("open")
            .args(["-R", &path])
            .spawn()
            .map_err(|e| e.to_string())?;
        return Ok(());
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        // On Linux, use xdg-open on the parent directory
        if let Some(parent) = p.parent() {
            Command::new("xdg-open")
                .arg(parent)
                .spawn()
                .map_err(|e| e.to_string())?;
        } else {
            Command::new("xdg-open")
                .arg(&path)
                .spawn()
                .map_err(|e| e.to_string())?;
        }
        return Ok(());
    }
}
