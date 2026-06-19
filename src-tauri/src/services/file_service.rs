use std::fs;
use std::path::Path;
use std::process::Command;

use crate::db::get_db_connection;
use crate::error::AppError;
use crate::icon::get_file_icon_base64;
use crate::models::FileInfo;
use crate::utils::{resolve_shortcut, to_abs_path};
use rusqlite::params;

pub fn list_directory(app: &tauri::AppHandle, path: String) -> Result<Vec<FileInfo>, AppError> {
    let mut results = Vec::new();

    // Get cached folder sizes if possible
    let cached_sizes = if let Ok(conn) = get_db_connection(app) {
        let stmt = conn.prepare("SELECT path, size FROM folder_sizes").ok();
        stmt.and_then(|mut s| {
            s.query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })
            .ok()
            .map(|iter| {
                iter.filter_map(|r| r.ok())
                    .collect::<std::collections::HashMap<String, i64>>()
            })
        })
    } else {
        None
    };

    // 如果路径为空或为 "This PC"，列出 Windows 盘符
    if path.is_empty() || path == "This PC" || path == "此电脑" {
        #[cfg(target_os = "windows")]
        {
            use windows::Win32::Storage::FileSystem::GetLogicalDrives;
            let drives = unsafe { GetLogicalDrives() };
            for i in 0..26 {
                if (drives >> i) & 1 == 1 {
                    let drive_letter = (b'A' + i as u8) as char;
                    let drive_path = format!("{}:\\", drive_letter);
                    let icon = get_file_icon_base64(Path::new(&drive_path)).unwrap_or_default();

                    results.push(FileInfo {
                        id: format!("drive-{}", drive_letter),
                        name: drive_path.clone(),
                        display_name: format!("本地磁盘 ({}:)", drive_letter),
                        path: drive_path,
                        size: 0,
                        r#type: "directory".to_string(),
                        icon,
                        content: None,
                        category: None,
                        open_count: None,
                        created_at: None,
                        modified_at: None,
                        notes: None,
                        is_pinned: Some(false),
                        dir_size_calculated: Some(false),
                        is_reparse_point: Some(false),
                    });
                }
            }
            return Ok(results);
        }
        #[cfg(not(target_os = "windows"))]
        {
            let p = Path::new("/");
            results.push(FileInfo {
                id: "root".to_string(),
                name: "/".to_string(),
                display_name: "根目录".to_string(),
                path: "/".to_string(),
                size: 0,
                r#type: "directory".to_string(),
                icon: get_file_icon_base64(p).unwrap_or_default(),
                content: None,
                category: None,
                open_count: None,
                created_at: None,
                modified_at: None,
                notes: None,
                is_pinned: Some(false),
                dir_size_calculated: Some(false),
                is_reparse_point: Some(false),
            });
            return Ok(results);
        }
    }

    let abs_path = to_abs_path(&path)?;
    let p = Path::new(&abs_path);
    if !p.is_dir() {
        return Err(AppError::NotFound(format!("Not a directory: {}", abs_path)));
    }

    for entry in fs::read_dir(p)? {
        let entry = entry?;
        let entry_path = entry.path();
        let name = entry_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        if name.starts_with('$') || name.starts_with('.') {
            continue;
        }

        let metadata = entry.metadata()?;
        let is_dir = metadata.is_dir();
        let is_reparse_point = metadata.file_type().is_symlink() || {
            #[cfg(target_os = "windows")]
            {
                use std::os::windows::fs::MetadataExt;
                (metadata.file_attributes() & 0x400) != 0 // FILE_ATTRIBUTE_REPARSE_POINT
            }
            #[cfg(not(target_os = "windows"))]
            {
                false
            }
        };

        let extension = if is_dir {
            "directory".to_string()
        } else {
            entry_path
                .extension()
                .map(|e| e.to_string_lossy().to_string())
                .unwrap_or_default()
        };

        // 移除图标同步获取，改为前端按需请求
        let icon = String::new();
        let path_str = entry_path.to_string_lossy().to_string();

        let (size, dir_size_calculated) = if is_dir {
            if let Some(&s) = cached_sizes.as_ref().and_then(|m| m.get(&path_str)) {
                (s as u64, Some(true))
            } else {
                (0, Some(false))
            }
        } else {
            (metadata.len(), Some(true))
        };

        results.push(FileInfo {
            id: generate_id(),
            name: name.clone(),
            display_name: name,
            path: path_str,
            size,
            r#type: extension,
            icon,
            content: None,
            category: None,
            open_count: None,
            created_at: metadata
                .created()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_millis() as i64),
            modified_at: metadata
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_millis() as i64),
            notes: None,
            is_pinned: Some(false),
            dir_size_calculated,
            is_reparse_point: Some(is_reparse_point),
        });
    }

    // 排序：目录在前，名称排序
    results.sort_by(|a, b| {
        let a_is_dir = a.r#type == "directory";
        let b_is_dir = b.r#type == "directory";
        if a_is_dir != b_is_dir {
            return b_is_dir.cmp(&a_is_dir);
        }
        a.name.to_lowercase().cmp(&b.name.to_lowercase())
    });

    Ok(results)
}

fn generate_id() -> String {
    use std::time::SystemTime;
    let now = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{:x}", now)
}

pub fn save_files(app: &tauri::AppHandle, files: Vec<FileInfo>) -> Result<(), AppError> {
    let mut conn = get_db_connection(app)?;
    let tx = conn.transaction()?;

    tx.execute("DELETE FROM files", [])?;

    let mut stmt = tx.prepare(
        "INSERT OR REPLACE INTO files (id, name, display_name, path, size, type, icon, content, category, open_count, created_at, modified_at, notes, is_pinned) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
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
        let modified_at = file.modified_at.unwrap_or(created_at);

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
            modified_at,
            &file.notes,
            if file.is_pinned.unwrap_or(false) {
                1
            } else {
                0
            }
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
            "SELECT id, name, display_name, path, size, type, icon, content, category, open_count, created_at, modified_at, notes, is_pinned FROM files ORDER BY open_count DESC",
        )?;

    let files_iter = stmt.query_map([], |row| {
        let file_type = row.get::<_, String>(5).unwrap_or_default();
        Ok(FileInfo {
            id: row.get(0)?,
            name: row.get(1)?,
            display_name: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
            path: row.get(3)?,
            size: row.get::<_, i64>(4).unwrap_or(0) as u64,
            r#type: file_type.clone(),
            icon: row.get::<_, String>(6).unwrap_or_default(),
            content: row.get::<_, Option<String>>(7).unwrap_or(None),
            category: row.get::<_, Option<String>>(8).unwrap_or(None),
            open_count: Some(row.get::<_, i64>(9).unwrap_or(0) as u64),
            created_at: row.get::<_, Option<i64>>(10).unwrap_or(None),
            modified_at: row.get::<_, Option<i64>>(11).unwrap_or(None),
            notes: row.get::<_, Option<String>>(12).unwrap_or(None),
            is_pinned: Some(row.get::<_, i64>(13).unwrap_or(0) != 0),
            dir_size_calculated: Some(file_type != "directory"),
            is_reparse_point: Some(false),
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

pub fn check_paths_exist(paths: Vec<String>) -> Result<Vec<bool>, AppError> {
    let mut results = Vec::new();
    for path in paths {
        let abs_path = to_abs_path(&path).unwrap_or(path.clone());
        results.push(Path::new(&abs_path).exists());
    }
    Ok(results)
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

    let display_name_source = if is_shortcut { &original_name } else { &name };

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

    let modified_at = metadata
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as i64)
        .unwrap_or(created_at);

    let is_reparse_point = metadata.file_type().is_symlink() || {
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::fs::MetadataExt;
            (metadata.file_attributes() & 0x400) != 0 // FILE_ATTRIBUTE_REPARSE_POINT
        }
        #[cfg(not(target_os = "windows"))]
        {
            false
        }
    };

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
        modified_at: Some(modified_at),
        notes: None,
        is_pinned: Some(false),
        dir_size_calculated: Some(metadata.is_file()),
        is_reparse_point: Some(is_reparse_point),
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
        let p = Path::new(&path);
        // 如果是可执行文件，设置工作目录为其所在文件夹并直接启动
        if p.is_file()
            && p.extension()
                .map_or(false, |ext| ext.to_string_lossy().to_lowercase() == "exe")
        {
            if let Some(parent) = p.parent() {
                Command::new(&path)
                    .current_dir(parent)
                    .spawn()
                    .map_err(|e| AppError::Platform(format!("Failed to launch exe: {}", e)))?;
                return Ok(());
            }
        }

        // 其他情况（文件夹、文档等）使用 explorer.exe 打开
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
                AppError::NotFound(format!(
                    "cannot resolve parent directory: {}",
                    resolved_path
                ))
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

pub fn delete_to_trash(path: String) -> Result<(), AppError> {
    let path = to_abs_path(&path)?;
    trash::delete(&path).map_err(|e| AppError::Other(format!("Failed to move to trash: {}", e)))?;
    Ok(())
}

pub fn copy_file_or_dir(src: String, dst_dir: String) -> Result<(), AppError> {
    let src_path = Path::new(&src);
    let dst_parent = Path::new(&dst_dir);

    if !src_path.exists() {
        return Err(AppError::NotFound(format!(
            "Source path does not exist: {}",
            src
        )));
    }

    if src_path.is_dir() {
        let mut options = fs_extra::dir::CopyOptions::new();
        options.overwrite = true;
        options.copy_inside = true;
        fs_extra::dir::copy(src_path, dst_parent, &options)
            .map_err(|e| AppError::Other(format!("Failed to copy directory: {}", e)))?;
    } else {
        let file_name = src_path
            .file_name()
            .ok_or_else(|| AppError::Other("Invalid source file name".into()))?;
        let dest_path = dst_parent.join(file_name);
        fs::copy(src_path, dest_path)
            .map_err(|e| AppError::Other(format!("Failed to copy file: {}", e)))?;
    }
    Ok(())
}

pub fn move_file_or_dir(src: String, dst_dir: String) -> Result<(), AppError> {
    let src_path = Path::new(&src);
    let dst_parent = Path::new(&dst_dir);

    if !src_path.exists() {
        return Err(AppError::NotFound(format!(
            "Source path does not exist: {}",
            src
        )));
    }

    let file_name = src_path
        .file_name()
        .ok_or_else(|| AppError::Other("Invalid source name".into()))?;
    let dest_path = dst_parent.join(file_name);

    // Try rename first (fastest, but fails across partitions)
    if fs::rename(src_path, &dest_path).is_ok() {
        return Ok(());
    }

    // Fallback to copy and delete if rename fails
    if src_path.is_dir() {
        let mut options = fs_extra::dir::CopyOptions::new();
        options.overwrite = true;
        options.copy_inside = true;
        fs_extra::dir::move_dir(src_path, dst_parent, &options)
            .map_err(|e| AppError::Other(format!("Failed to move directory: {}", e)))?;
    } else {
        let mut options = fs_extra::file::CopyOptions::new();
        options.overwrite = true;
        fs_extra::file::move_file(src_path, dest_path, &options)
            .map_err(|e| AppError::Other(format!("Failed to move file: {}", e)))?;
    }
    Ok(())
}

pub fn get_dir_size(path: &Path) -> u64 {
    let mut total_size = 0;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let metadata = entry.metadata().ok();
            if let Some(meta) = metadata {
                if meta.is_dir() {
                    total_size += get_dir_size(&entry.path());
                } else {
                    total_size += meta.len();
                }
            }
        }
    }
    total_size
}

pub fn calculate_dir_sizes(
    paths: Vec<String>,
) -> Result<std::collections::HashMap<String, u64>, AppError> {
    let mut results = std::collections::HashMap::new();
    for path_str in paths {
        let path = Path::new(&path_str);
        if path.is_dir() {
            let size = get_dir_size(path);
            results.insert(path_str, size);
        }
    }
    Ok(results)
}

pub fn calculate_dir_size_single(path: String) -> Result<u64, AppError> {
    let path_buf = Path::new(&path);
    if !path_buf.is_dir() {
        return Err(AppError::NotFound(format!("Not a directory: {}", path)));
    }
    Ok(get_dir_size(path_buf))
}
