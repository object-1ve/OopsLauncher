use std::fs;
use std::path::Path;
use std::process::Command;
use crate::models::FileInfo;
use crate::db::get_db_connection;
use crate::utils::{to_abs_path, resolve_shortcut};
use crate::icon::get_file_icon_base64;
use rusqlite::params;

// 保存文件列表到SQLite数据库
#[tauri::command]
pub fn save_files_to_db(app: tauri::AppHandle, files: Vec<FileInfo>) -> Result<(), String> {
    println!("Saving {} files to database...", files.len());
    let mut conn = get_db_connection(&app)?;
    
    // 开始事务
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    
    // 清空现有数据
    tx.execute("DELETE FROM files", []).map_err(|e| e.to_string())?;
    
    // 插入新数据
    let mut stmt = tx.prepare(
        "INSERT OR REPLACE INTO files (id, name, display_name, path, size, type, icon, content, category, open_count, created_at, notes, is_pinned) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    ).map_err(|e| e.to_string())?;

    for file in files {
        // 直接使用前端传来的路径，不再调用 to_abs_path 重新解析
        // get_file_info 已经在获取时规范化过路径，重复调用可能因 canonicalize 失败而导致跳过
        let path = file.path.trim().to_string();
        if path.is_empty() {
            println!("Skipping file with empty path: {}", file.name);
            continue;
        }

        // 获取分类 ID，如果没有则默认为 main
        let category_id = match file.category.as_ref() {
            Some(id) => id,
            None => "main"
        };
        
        // 获取当前时间作为创建时间（如果前端没传）
        let created_at = file.created_at.unwrap_or_else(|| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as i64
        });
        
        // 尝试执行插入
        stmt.execute(
            params![
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
            ]
        ).map_err(|e| {
            println!("Failed to save file {} to DB: {}", file.name, e);
            format!("Failed to save file {}: {}", file.name, e)
        })?;
    }
    
    // 释放 statement
    drop(stmt);

    // 提交事务
    tx.commit().map_err(|e| {
        println!("Failed to commit transaction: {}", e);
        e.to_string()
    })?;
    
    println!("Successfully saved all files to database.");
    Ok(())
}

// 从SQLite数据库读取文件列表
#[tauri::command]
pub fn load_files_from_db(app: tauri::AppHandle) -> Result<Vec<FileInfo>, String> {
    println!("Loading files from database...");
    let conn = get_db_connection(&app)?;
    
    let mut stmt = conn.prepare("SELECT id, name, display_name, path, size, type, icon, content, category, open_count, created_at, notes, is_pinned FROM files ORDER BY open_count DESC")
        .map_err(|e| {
            println!("Failed to prepare select statement: {}", e);
            e.to_string()
        })?;
    
    let files_iter = stmt.query_map([], |row| {
        let name: String = row.get(1)?;
        let display_name: Option<String> = row.get(2)?;
        let path: String = row.get(3)?;
        let size: i64 = row.get(4).unwrap_or(0);
        let r#type: String = row.get(5).unwrap_or_else(|_| "".to_string());
        let icon: String = row.get(6).unwrap_or_else(|_| "".to_string());
        let content: Option<String> = row.get(7).ok();
        let category: Option<String> = row.get(8).ok();
        let open_count: i64 = row.get(9).unwrap_or(0);
        let created_at: Option<i64> = row.get(10).ok();
        let notes: Option<String> = row.get(11).ok();
        let is_pinned: i64 = row.get(12).unwrap_or(0);

        Ok(FileInfo {
            id: row.get(0)?,
            name,
            display_name: display_name.unwrap_or_else(|| "".to_string()),
            path,
            size: size as u64,
            r#type,
            icon,
            content,
            category,
            open_count: Some(open_count as u64),
            created_at,
            notes,
            is_pinned: Some(is_pinned != 0),
        })
    }).map_err(|e| {
        println!("Failed to query files: {}", e);
        e.to_string()
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

#[tauri::command]
pub fn get_file_info(path: String) -> Result<FileInfo, String> {
    let abs_path = to_abs_path(&path)?;
    let original_p = Path::new(&abs_path);
    let original_name = original_p.file_name()
        .ok_or("Invalid file name")?
        .to_string_lossy()
        .to_string();

    // 解析快捷方式
    let target_path = resolve_shortcut(&abs_path);
    let is_shortcut = target_path != abs_path;
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

    // 生成 display_name
    // 如果是快捷方式，使用快捷方式的名称
    // 如果不是，使用目标文件的名称，并去掉常见后缀
    let display_name_source = if is_shortcut { &original_name } else { &name };
    
    let display_name = if let Some(last_dot_idx) = display_name_source.rfind('.') {
        let ext = &display_name_source[last_dot_idx + 1..].to_lowercase();
        let common_extensions = ["exe", "lnk", "js", "ts", "html", "css", "py", "rs", "c", "cpp", "h", "hpp", "go", "sql", "yml", "yaml", "toml", "xml", "txt", "md", "json"];
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
        created_at: Some(created_at),
        notes: None,
        is_pinned: Some(false),
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
        Command::new("explorer.exe")
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

#[tauri::command]
pub fn open_with_dialog(path: String) -> Result<(), String> {
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
        Command::new("rundll32.exe")
            .args(["shell32.dll,OpenAs_RunDLL", &path])
            .spawn()
            .map_err(|e| e.to_string())?;
        return Ok(());
    }

    #[cfg(not(target_os = "windows"))]
    {
        return Err("当前平台暂不支持“打开方式”对话框".to_string());
    }
}

#[tauri::command]
pub fn open_terminal(path: String) -> Result<(), String> {
    let path = path.trim();
    if path.is_empty() {
        return Err("path is empty".to_string());
    }

    let path = to_abs_path(path)?;
    // 解析快捷方式，确保如果是指向目录的快捷方式，能在目标目录打开
    let resolved_path = resolve_shortcut(&path);
    let p = Path::new(&resolved_path);
    
    if !p.exists() {
        return Err(format!("path does not exist: {}", resolved_path));
    }

    let target_dir = if p.is_dir() {
        p.to_path_buf()
    } else {
        p.parent()
            .ok_or_else(|| format!("cannot resolve parent directory: {}", resolved_path))?
            .to_path_buf()
    };

    #[cfg(target_os = "windows")]
    {
        // 使用 cmd /c start 来确保在 Windows 上打开一个全新的独立终端窗口
        // 这样可以避免在开发环境下占用当前控制台，并确保路径跳转正确
        let target_str = target_dir.to_string_lossy();
        let escaped_path = target_str.replace('\'', "''");
        
        // 构建 powershell 命令字符串，使用 -NoExit 保持窗口开启，并使用 Set-Location 跳转目录
        let ps_command = format!("Set-Location -LiteralPath '{}'", escaped_path);
        
        Command::new("cmd")
            .args(["/c", "start", "powershell", "-NoExit", "-Command", &ps_command])
            .current_dir(&target_dir)
            .spawn()
            .map_err(|e| format!("无法启动终端窗口: {}", e))?;
        
        return Ok(());
    }

    #[cfg(target_os = "macos")]
    {
        // macOS 下 Terminal 可以直接接受目录作为参数
        Command::new("open")
            .args(["-a", "Terminal", target_dir.to_string_lossy().as_ref()])
            .current_dir(&target_dir)
            .spawn()
            .map_err(|e| format!("无法启动终端: {}", e))?;
        return Ok(());
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let dir = target_dir.to_string_lossy().to_string();
        let candidates = [
            ("x-terminal-emulator", vec!["--working-directory", &dir]),
            ("gnome-terminal", vec!["--working-directory", &dir]),
            ("konsole", vec!["--workdir", &dir]),
            ("xfce4-terminal", vec!["--working-directory", &dir]),
        ];

        for (cmd, args) in candidates {
            if Command::new(cmd).args(&args).current_dir(&target_dir).spawn().is_ok() {
                return Ok(());
            }
        }
        return Err("未找到可用终端程序".to_string());
    }
}
