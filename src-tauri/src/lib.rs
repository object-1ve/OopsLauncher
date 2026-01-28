pub mod models;
pub mod db;
pub mod utils;
pub mod icon;
pub mod commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .plugin(tauri_plugin_notification::init())
    .plugin(tauri_plugin_dialog::init())
    .plugin(tauri_plugin_global_shortcut::Builder::new().build())
    .plugin(tauri_plugin_clipboard_manager::init())
    .plugin(tauri_plugin_autostart::init(tauri_plugin_autostart::MacosLauncher::LaunchAgent, Some(vec!["--minimized"])))
    .invoke_handler(tauri::generate_handler![
        commands::file::save_files_to_db, 
        commands::file::load_files_from_db,
        commands::file::get_file_info,
        commands::file::open_path,
        commands::file::open_file_location,
        commands::category::save_categories_to_db, 
        commands::category::load_categories_from_db,
        commands::category::rename_category_in_db,
        commands::category::delete_category_from_db,
        commands::notification::send_notification_custom,
        commands::app::get_app_version,
    ])
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }
      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
