pub mod models;
pub mod db;
pub mod utils;
pub mod icon;
pub mod error;
pub mod tray;
pub mod services;
pub mod commands;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .invoke_handler(tauri::generate_handler![
            commands::file::save_files_to_db,
            commands::file::load_files_from_db,
            commands::file::get_file_info,
            commands::file::open_path,
            commands::file::open_file_location,
            commands::file::open_with_dialog,
            commands::file::open_terminal,
            commands::category::save_categories_to_db,
            commands::category::load_categories_from_db,
            commands::category::rename_category_in_db,
            commands::category::delete_category_from_db,
            commands::notification::send_notification_custom,
            commands::app::get_app_version,
            commands::app::set_skip_taskbar,
            commands::app::check_is_minimized,
            commands::app::remove_window_animation,
            commands::app::exit_app,
            commands::app::disable_settings_system_menu,
            commands::settings::save_launcher_state_to_db,
            commands::settings::load_launcher_state_from_db,
            commands::settings::save_settings_to_json,
            commands::settings::load_settings_from_json,
        ])
        .setup(|app| {
            db::init_database(app.handle())?;

            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            tray::build_tray(app)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                // 如果是主窗口，隐藏而不是退出
                if window.label() == "main" {
                    let _ = window.hide();
                    api.prevent_close();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
