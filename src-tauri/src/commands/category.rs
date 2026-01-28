use crate::models::Category;
use crate::db::get_db_connection;

// 保存分类列表到SQLite数据库
#[tauri::command]
pub fn save_categories_to_db(app: tauri::AppHandle, categories: Vec<Category>) -> Result<(), String> {
    let mut conn = get_db_connection(&app)?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    
    tx.execute("DELETE FROM categories", []).map_err(|e| e.to_string())?;
    
    let mut stmt = tx.prepare(
        "INSERT OR REPLACE INTO categories (id, parent_id, name, icon, sort_order) VALUES (?, ?, ?, ?, ?)"
    ).map_err(|e| e.to_string())?;

    for category in categories {
        stmt.execute(rusqlite::params![
            &category.id, 
            &category.parent_id, 
            &category.name, 
            &category.icon, 
            category.sort_order
        ]).map_err(|e| e.to_string())?;
    }
    
    drop(stmt);
    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

// 从SQLite数据库读取分类列表
#[tauri::command]
pub fn load_categories_from_db(app: tauri::AppHandle) -> Result<Vec<Category>, String> {
    let conn = get_db_connection(&app)?;
    let mut stmt = conn.prepare("SELECT id, parent_id, name, icon, sort_order FROM categories ORDER BY sort_order ASC").map_err(|e| e.to_string())?;
    
    let categories_iter = stmt.query_map([], |row| {
        Ok(Category {
            id: row.get(0)?,
            parent_id: row.get(1)?,
            name: row.get(2)?,
            icon: row.get(3)?,
            sort_order: row.get(4)?,
        })
    }).map_err(|e| e.to_string())?;
    
    let mut categories = Vec::new();
    for category in categories_iter {
        categories.push(category.map_err(|e| e.to_string())?);
    }
    
    Ok(categories)
}

// 更新分类名称
#[tauri::command]
pub fn rename_category_in_db(app: tauri::AppHandle, id: String, new_name: String) -> Result<(), String> {
    let conn = get_db_connection(&app)?;
    conn.execute(
        "UPDATE categories SET name = ? WHERE id = ?",
        [&new_name, &id]
    ).map_err(|e| e.to_string())?;
    Ok(())
}

// 删除分类及其关联的文件
#[tauri::command]
pub fn delete_category_from_db(app: tauri::AppHandle, id: String) -> Result<(), String> {
    let mut conn = get_db_connection(&app)?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    
    // 删除分类
    tx.execute("DELETE FROM categories WHERE id = ?", [&id]).map_err(|e| e.to_string())?;
    
    // 删除该分类下的所有文件
    tx.execute("DELETE FROM files WHERE category = ?", [&id]).map_err(|e| e.to_string())?;
    
    tx.commit().map_err(|e| e.to_string())?;
    println!("Category {} and its files deleted from DB.", id);
    Ok(())
}
