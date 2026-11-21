pub mod auth;
pub mod commands;
pub mod db;
pub mod excel;
pub mod excel_importer;
pub mod optimizer;
pub mod tax_report;

use commands::*;
use std::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let (db_path, archive_dir) = db::get_default_paths();
    let conn = db::init_db_with_migration(&db_path, true).expect("failed to initialize sqlite database");

    let app_state = AppState {
        db: Mutex::new(conn),
        db_path,
        archive_dir,
        current_user: Mutex::new(None),
    };

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            // Optimizer
            pick_excel_file,
            pick_save_file,
            analyze_excel,
            solve_subset,
            export_excel,
            // Auth
            auth_login,
            auth_logout,
            auth_list_users,
            auth_create_user,
            auth_update_pin,
            auth_delete_user,
            // Transactions & Ledger
            get_transactions,
            get_room_reservations,
            add_transaction,
            update_transaction,
            delete_transaction,
            update_room_reservation,
            delete_room_reservation,
            export_room_reservations_excel,
            get_origin_reference_list,
            delete_period_transactions,
            clean_duplicate_transactions,
            get_available_months,
            // Tax Report
            get_tax_recap,
            get_tourism_report,
            print_report,
            set_window_title,
            preview_import_file,
            import_excel_file,
            get_import_logs,
            download_excel_template,
            // Units
            get_units,
            add_unit,
            update_unit,
            // Settings
            get_settings,
            update_settings,
            // Storage & Backup
            get_storage_info,
            backup_database_dialog,
            restore_database_dialog,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
