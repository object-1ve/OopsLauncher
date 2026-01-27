use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tauri::Manager;

// 用于图像处理
use image::RgbImage;
use base64::Engine as _;
use base64::engine::general_purpose;

// 用于数据库存储
use rusqlite::Connection;

// Windows平台特定的图标获取
#[cfg(target_os = "windows")]
mod win_icon {
    use super::*;
    use winapi::um::shellapi::{SHGetFileInfoW, SHGFI_ICON, SHGFI_LARGEICON, SHFILEINFOW};
    use winapi::um::wingdi::{CreateCompatibleDC, SelectObject, DeleteDC, DIB_RGB_COLORS, CreateSolidBrush};
    use winapi::um::winuser::{DestroyIcon, GetSystemMetrics, SM_CXICON, SM_CYICON, ReleaseDC, GetDC, DrawIcon, FillRect};
    use winapi::shared::windef::RECT;
    use winapi::ctypes;
    use std::ptr;
    use std::mem;
    use std::io::Cursor;

    pub fn get_file_icon_base64(path: &Path) -> Result<String, String> {
        let path_str = path.to_str().ok_or("Invalid path")?;
        let wide_path: Vec<u16> = path_str.encode_utf16().chain(std::iter::once(0)).collect();

        let mut shfi: SHFILEINFOW = unsafe { mem::zeroed() };
        let flags = SHGFI_ICON | SHGFI_LARGEICON;

        let hresult = unsafe {
            SHGetFileInfoW(
                wide_path.as_ptr(),
                0,
                &mut shfi,
                mem::size_of::<SHFILEINFOW>() as u32,
                flags,
            )
        };

        if hresult == 0 {
            return Err("Failed to get file info".to_string());
        }

        let hicon = shfi.hIcon;
        if hicon.is_null() {
            return Err("Failed to get icon".to_string());
        }

        let icon_width = unsafe { GetSystemMetrics(SM_CXICON) };
        let icon_height = unsafe { GetSystemMetrics(SM_CYICON) };

        let hdc = unsafe { GetDC(ptr::null_mut()) };
        let hmemdc = unsafe { CreateCompatibleDC(hdc) };

        // 创建一个兼容的位图
        let hbmp = unsafe {
            winapi::um::wingdi::CreateCompatibleBitmap(
                hdc,
                icon_width,
                icon_height
            )
        };

        // 选择位图到内存DC
        let old_bmp = unsafe { SelectObject(hmemdc, hbmp as _) };

        // 填充白色背景
        let white_brush = unsafe {
            CreateSolidBrush(
                0xFFFFFF // 白色
            )
        };
        
        let mut rect = RECT {
            left: 0,
            top: 0,
            right: icon_width,
            bottom: icon_height
        };
        
        unsafe {
            FillRect(hmemdc, &mut rect, white_brush);
            winapi::um::wingdi::DeleteObject(white_brush as _);
        }

        // 绘制图标到内存DC
        unsafe {
            DrawIcon(
                hmemdc,
                0,
                0,
                hicon
            );
        }

        // 准备像素缓冲区
        let mut pixels: Vec<u8> = vec![0; (icon_width * icon_height * 3) as usize];

        // 获取位图信息
        let mut bmi = winapi::um::wingdi::BITMAPINFO {
            bmiHeader: winapi::um::wingdi::BITMAPINFOHEADER {
                biSize: std::mem::size_of::<winapi::um::wingdi::BITMAPINFOHEADER>() as u32,
                biWidth: icon_width,
                biHeight: -icon_height, // 负高度表示从上到下
                biPlanes: 1,
                biBitCount: 24,
                biCompression: winapi::um::wingdi::BI_RGB,
                biSizeImage: 0,
                biXPelsPerMeter: 0,
                biYPelsPerMeter: 0,
                biClrUsed: 0,
                biClrImportant: 0,
            },
            bmiColors: [winapi::um::wingdi::RGBQUAD { rgbBlue: 0, rgbGreen: 0, rgbRed: 0, rgbReserved: 0 }],
        };

        // 获取位图数据
        unsafe {
            winapi::um::wingdi::GetDIBits(
                hmemdc,
                hbmp,
                0,
                icon_height as u32,
                pixels.as_mut_ptr() as *mut ctypes::c_void,
                &mut bmi as *mut winapi::um::wingdi::BITMAPINFO,
                DIB_RGB_COLORS
            );
        }

        // 清理资源
        unsafe {
            SelectObject(hmemdc, old_bmp);
            winapi::um::wingdi::DeleteObject(hbmp as _);
            ReleaseDC(ptr::null_mut(), hdc);
            DeleteDC(hmemdc);
            DestroyIcon(hicon);
        }

        // 创建图像并转换为base64
        let img = RgbImage::from_raw(icon_width as u32, icon_height as u32, pixels)
            .ok_or("Failed to create image".to_string())?;

        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);
        img.write_to(&mut cursor, image::ImageOutputFormat::Png)
            .map_err(|e| e.to_string())?;

        let base64_icon = general_purpose::STANDARD.encode(&buffer);
        Ok(format!("data:image/png;base64,{}", base64_icon))
    }
}

// 非Windows平台的默认实现
#[cfg(not(target_os = "windows"))]
pub fn get_file_icon_base64(_path: &Path) -> Result<String, String> {
    // 在非Windows平台上，返回默认图标
    Ok("".to_string())
}

#[cfg(target_os = "windows")]
pub fn get_file_icon_base64(path: &Path) -> Result<String, String> {
    win_icon::get_file_icon_base64(path)
}

// 文件信息结构体
#[derive(Debug, Serialize, Deserialize)]
pub struct FileInfo {
    id: String,
    name: String,
    path: String,
    size: u64,
    r#type: String,
    icon: String,
    category: Option<String>,
}

// 获取数据库连接
fn get_db_connection(app: &tauri::AppHandle) -> Result<Connection, String> {
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
fn init_database(conn: &Connection) -> Result<(), String> {
    // 创建表（如果不存在）
    conn.execute(
        "CREATE TABLE IF NOT EXISTS files (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            path TEXT NOT NULL,
            size INTEGER,
            type TEXT,
            icon TEXT,
            category TEXT NOT NULL DEFAULT 'main'
        )",
        []
    ).map_err(|e| e.to_string())?;
    
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
    
    Ok(())
}

fn to_abs_path(path: &str) -> Result<String, String> {
    let p = Path::new(path);
    let abs: PathBuf = if p.is_absolute() {
        p.to_path_buf()
    } else {
        env::current_dir()
            .map_err(|e| e.to_string())?
            .join(p)
    };

    let abs = abs.canonicalize().unwrap_or(abs);
    
    // 在Windows上移除 \\?\ 前缀
    #[cfg(target_os = "windows")]
    let abs_str = {
        let s = abs.to_string_lossy().to_string();
        if s.starts_with(r"\\?\") {
            s[4..].to_string()
        } else {
            s
        }
    };

    #[cfg(not(target_os = "windows"))]
    let abs_str = abs.to_string_lossy().to_string();

    Ok(abs_str)
}

// 保存文件列表到SQLite数据库
#[tauri::command]
pub fn save_files_to_db(app: tauri::AppHandle, files: Vec<FileInfo>) -> Result<(), String> {
    let mut conn = get_db_connection(&app)?;
    
    // 开始事务
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    
    // 清空现有数据
    // 注意：如果我们只想覆盖特定category，可以加 WHERE category = ? 条件
    // 但目前的逻辑是前端传入所有数据，所以全量覆盖是合理的
    // 为了支持部分更新，我们可以在这里做优化，但当前保持全量更新简单可靠
    tx.execute("DELETE FROM files", []).map_err(|e| e.to_string())?;
    
    // 插入新数据
    let mut stmt = tx.prepare(
        "INSERT INTO files (id, name, path, size, type, icon, category) VALUES (?, ?, ?, ?, ?, ?, ?)"
    ).map_err(|e| e.to_string())?;

    for mut file in files {
        file.path = to_abs_path(&file.path)?;
        
        // 获取分类信息，如果没有则默认为main
        let category = match file.category.as_ref() {
            Some(cat) => cat,
            None => "main"
        };
        
        stmt.execute(
            [
                &file.id,
                &file.name,
                &file.path,
                &file.size.to_string(),
                &file.r#type,
                &file.icon,
                category
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
    
    let mut stmt = conn.prepare("SELECT id, name, path, size, type, icon, category FROM files").map_err(|e| e.to_string())?;
    
    let files_iter = stmt.query_map([], |row| {
        Ok(FileInfo {
            id: row.get(0)?,
            name: row.get(1)?,
            path: row.get(2)?,
            size: row.get::<_, i64>(3)? as u64,
            r#type: row.get(4)?,
            icon: row.get(5)?,
            category: Some(row.get(6)?)  // 添加category字段
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
    let p = Path::new(&abs_path);
    
    if !p.exists() {
        return Err(format!("File not found: {}", abs_path));
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

    Ok(FileInfo {
        id: "".to_string(),
        name,
        path: abs_path,
        size: metadata.len(),
        r#type: extension,
        icon,
        category: None,
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
