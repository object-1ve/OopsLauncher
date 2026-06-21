pub mod commands;
pub mod db;
pub mod error;
pub mod icon;
pub mod models;
pub mod services;
pub mod tray;
pub mod utils;

#[cfg(not(debug_assertions))]
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // debug 构建下不会启用单实例插件，builder 不会被重新赋值，故抑制 unused_mut 警告
    #[cfg_attr(debug_assertions, allow(unused_mut))]
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ));

    // 仅在发布版启用单实例；dev 下禁用，避免旧进程把新启动的进程"劫持"、
    // 唤起旧窗口，导致 tauri.conf.json / Rust 改动看似不生效。
    #[cfg(not(debug_assertions))]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            log::warn!("[single-instance] 已有实例在运行，唤起现有窗口（新进程已退出）");
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }));
    }

    builder
        .invoke_handler(tauri::generate_handler![
            commands::file::save_files_to_db,
            commands::file::load_files_from_db,
            commands::file::get_file_info,
            commands::file::check_paths_exist,
            commands::file::open_path,
            commands::file::open_file_location,
            commands::file::open_with_dialog,
            commands::file::open_terminal,
            commands::file::list_directory,
            commands::file::delete_to_trash,
            commands::file::copy_file_or_dir,
            commands::file::move_file_or_dir,
            commands::file::calculate_dir_sizes,
            commands::file::calculate_dir_size,
            commands::file::get_file_icon,
            commands::file::search_windows_index,
            commands::file::scan_start_menu_programs,
            commands::folder_size_task::start_folder_size_task,
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
            // 启动横幅：每次「真正新建的进程」才会打印。
            // 看不到这行 = 你启动的进程被 single-instance 拦截、唤起了旧窗口。
            println!(
                "[startup] OopsLauncher v{} 进程已启动（PID {}）",
                env!("CARGO_PKG_VERSION"),
                std::process::id()
            );
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
