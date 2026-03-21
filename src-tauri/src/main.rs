#![cfg_attr(not(target_os = "windows"), windows_subsystem = "windows")]

use log_viewer_lib::commands::AppState;
use log_viewer_lib::Database;
use std::env;
use tauri::Emitter;

fn main() {
    let db = Database::new().expect("Failed to initialize database");

    let args: Vec<String> = env::args().collect();
    let initial_file = if args.len() > 1 {
        Some(args[1].clone())
    } else {
        None
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_os::init())
        .manage(AppState::new(db))
        .invoke_handler(tauri::generate_handler![
            log_viewer_lib::commands::open_file,
            log_viewer_lib::commands::parse_log,
            log_viewer_lib::commands::get_templates,
            log_viewer_lib::commands::save_template,
            log_viewer_lib::commands::delete_template,
            log_viewer_lib::commands::preview_template,
            log_viewer_lib::commands::detect_template_for_lines,
            log_viewer_lib::commands::get_parse_session,
            log_viewer_lib::commands::get_entries,
            log_viewer_lib::commands::get_entry_detail,
            log_viewer_lib::commands::get_stats,
            log_viewer_lib::commands::filter_by_level,
            log_viewer_lib::commands::filter_by_time,
            log_viewer_lib::commands::search,
            log_viewer_lib::commands::filter_logs,
            log_viewer_lib::commands::get_current_file,
            log_viewer_lib::commands::clear_cache,
            log_viewer_lib::commands::get_cache_info,
            log_viewer_lib::commands::get_settings,
            log_viewer_lib::commands::save_settings,
            log_viewer_lib::commands::set_file_association,
            log_viewer_lib::commands::set_file_associations,
            log_viewer_lib::commands::remove_file_association,
            log_viewer_lib::commands::remove_file_associations,
            log_viewer_lib::commands::check_file_association,
            log_viewer_lib::commands::check_file_associations,
        ])
        .setup(move |app| {
            if let Some(file_path) = initial_file {
                let app_handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    let _ = app_handle.emit("open-file-argument", file_path);
                });
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
