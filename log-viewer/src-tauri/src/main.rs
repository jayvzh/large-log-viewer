#![cfg_attr(not(target_os = "windows"), windows_subsystem = "windows")]

use log_viewer_lib::commands::AppState;
use log_viewer_lib::Database;

fn main() {
    let db = Database::new().expect("Failed to initialize database");
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::new(db))
        .invoke_handler(tauri::generate_handler![
            log_viewer_lib::commands::open_file,
            log_viewer_lib::commands::parse_log,
            log_viewer_lib::commands::get_entries,
            log_viewer_lib::commands::get_entry_detail,
            log_viewer_lib::commands::get_stats,
            log_viewer_lib::commands::filter_by_level,
            log_viewer_lib::commands::search,
            log_viewer_lib::commands::get_current_file,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
