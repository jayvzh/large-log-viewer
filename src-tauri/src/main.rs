#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use log_viewer_lib::commands::AppState;
use log_viewer_lib::Database;

fn main() {
    let db = Database::new().expect("Failed to initialize database");
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_os::init())
        .manage(AppState::new(db))
        .invoke_handler(tauri::generate_handler![
            log_viewer_lib::commands::open_file,
            log_viewer_lib::commands::parse_log,
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
            log_viewer_lib::commands::get_templates,
            log_viewer_lib::commands::create_template,
            log_viewer_lib::commands::update_template,
            log_viewer_lib::commands::delete_template,
            log_viewer_lib::commands::test_template,
            log_viewer_lib::commands::detect_template,
            log_viewer_lib::commands::export_templates,
            log_viewer_lib::commands::import_templates,
            log_viewer_lib::commands::export_logs,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
