mod commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![
        commands::save_files_to_db, 
        commands::load_files_from_db,
        commands::get_file_info,
        commands::open_path
    ])
    .setup(|app| {
      app.handle().plugin(tauri_plugin_global_shortcut::Builder::new().build())?;
      app.handle().plugin(tauri_plugin_clipboard_manager::init())?;

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
