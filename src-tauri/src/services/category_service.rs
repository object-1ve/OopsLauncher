use crate::db::get_db_connection;
use crate::error::AppError;
use crate::models::Category;

pub fn save_categories(app: &tauri::AppHandle, categories: Vec<Category>) -> Result<(), AppError> {
    let mut conn = get_db_connection(app)?;
    let tx = conn.transaction()?;

    tx.execute("DELETE FROM categories", [])?;

    let mut stmt = tx.prepare(
        "INSERT OR REPLACE INTO categories (id, parent_id, name, icon, sort_order) VALUES (?, ?, ?, ?, ?)",
    )?;

    for category in &categories {
        stmt.execute(rusqlite::params![
            &category.id,
            &category.parent_id,
            &category.name,
            &category.icon,
            category.sort_order
        ])?;
    }

    drop(stmt);
    tx.commit()?;
    Ok(())
}

pub fn load_categories(app: &tauri::AppHandle) -> Result<Vec<Category>, AppError> {
    let conn = get_db_connection(app)?;

    let mut stmt = conn.prepare(
        "SELECT id, parent_id, name, icon, sort_order FROM categories ORDER BY sort_order ASC",
    )?;

    let categories_iter = stmt.query_map([], |row| {
        Ok(Category {
            id: row.get(0)?,
            parent_id: row.get(1)?,
            name: row.get(2)?,
            icon: row.get(3)?,
            sort_order: row.get(4)?,
        })
    })?;

    let mut categories = Vec::new();
    for category in categories_iter {
        categories.push(category?);
    }

    Ok(categories)
}

pub fn rename_category(app: &tauri::AppHandle, id: String, new_name: String) -> Result<(), AppError> {
    let conn = get_db_connection(app)?;
    conn.execute("UPDATE categories SET name = ? WHERE id = ?", [&new_name, &id])?;
    Ok(())
}

pub fn delete_category(app: &tauri::AppHandle, id: String) -> Result<(), AppError> {
    let mut conn = get_db_connection(app)?;
    let tx = conn.transaction()?;

    tx.execute("DELETE FROM categories WHERE id = ?", [&id])?;
    tx.execute("DELETE FROM files WHERE category = ?", [&id])?;

    tx.commit()?;
    println!("Category {} and its files deleted from DB.", id);
    Ok(())
}
