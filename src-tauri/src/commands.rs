use crate::auth::{self, UserProfile};
use crate::db::{
    self, ImportLogRow, PaginatedRoomReservations, PaginatedTransactions, RoomReservationUpdate,
    StorageInfo, UnitRow,
};
use crate::excel::{
    analyze_excel_file, export_room_reservations_to_excel, export_solution_to_excel,
    ExcelAnalysis, ALL_WORLD_COUNTRIES, INDONESIAN_REGIONS,
};
use crate::excel_importer::{import_excel, preview_excel_import, ImportOptions, ImportPreview, ImportResult};
use crate::optimizer::{solve_subset as run_solver, SolverOptions, SolverResult, TransactionItem};
use crate::tax_report::{
    generate_tax_recap, generate_tourism_report, TaxRecapReport, TourismFormReport,
};
use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;

pub struct AppState {
    pub db: Mutex<rusqlite::Connection>,
    pub db_path: PathBuf,
    pub archive_dir: PathBuf,
    pub current_user: Mutex<Option<UserProfile>>,
}

fn require_admin_id(state: &State<'_, AppState>) -> Result<i64, String> {
    let user_guard = state.current_user.lock().map_err(|e| e.to_string())?;
    match user_guard.as_ref() {
        Some(u) if u.role == "admin" => Ok(u.id),
        _ => Err("Akses ditolak: tindakan ini hanya dapat dilakukan oleh Administrator.".to_string()),
    }
}

fn require_admin(state: &State<'_, AppState>) -> Result<(), String> {
    require_admin_id(state).map(|_| ())
}

fn sanitize_filename_segment(raw: &str) -> String {
    let cleaned: String = raw
        .trim()
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' | ' ' => '_',
            _ => c,
        })
        .collect();
    if cleaned.is_empty() {
        "Data".to_string()
    } else {
        cleaned
    }
}

// ==========================================
// File Dialogs & Optimizer Commands
// ==========================================

#[tauri::command]
pub fn pick_excel_file() -> Result<Option<String>, String> {
    let file = rfd::FileDialog::new()
        .add_filter("Excel Workbook (*.xlsx, *.xls, *.xlsm)", &["xlsx", "xls", "xlsm"])
        .add_filter("Semua Berkas (*.*)", &["*"])
        .pick_file();
    Ok(file.map(|f| f.to_string_lossy().to_string()))
}

#[tauri::command]
pub fn pick_save_file(default_name: Option<String>) -> Result<Option<String>, String> {
    let mut dialog = rfd::FileDialog::new()
        .add_filter("Excel Workbook (*.xlsx)", &["xlsx"]);
    if let Some(name) = default_name {
        dialog = dialog.set_file_name(&name);
    }
    let file = dialog.save_file();
    Ok(file.map(|f| f.to_string_lossy().to_string()))
}

#[tauri::command]
pub fn analyze_excel(file_path: String) -> Result<ExcelAnalysis, String> {
    analyze_excel_file(&file_path)
}

#[tauri::command]
pub fn solve_subset(
    items: Vec<TransactionItem>,
    options: SolverOptions,
) -> Result<SolverResult, String> {
    run_solver(&items, &options)
        .ok_or_else(|| "Gagal menemukan kombinasi subset yang memenuhi kriteria.".to_string())
}

#[tauri::command]
pub fn export_excel(
    source_path: String,
    destination_path: String,
    items: Vec<TransactionItem>,
    selected_indices: Vec<usize>,
) -> Result<String, String> {
    export_solution_to_excel(&source_path, &destination_path, &items, &selected_indices)
}

// ==========================================
// Authentication Commands
// ==========================================

#[tauri::command]
pub fn auth_login(
    state: State<'_, AppState>,
    username: String,
    pin: String,
) -> Result<UserProfile, String> {
    let profile = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        auth::authenticate(&conn, &username, &pin)?
    };
    let mut current_user = state.current_user.lock().map_err(|e| e.to_string())?;
    *current_user = Some(profile.clone());
    Ok(profile)
}

#[tauri::command]
pub fn auth_logout(state: State<'_, AppState>) -> Result<(), String> {
    let mut current_user = state.current_user.lock().map_err(|e| e.to_string())?;
    *current_user = None;
    Ok(())
}

#[tauri::command]
pub fn auth_list_users(state: State<'_, AppState>) -> Result<Vec<UserProfile>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    auth::list_users(&conn)
}

#[tauri::command]
pub fn auth_create_user(
    state: State<'_, AppState>,
    username: String,
    pin: String,
    full_name: String,
    role: String,
) -> Result<UserProfile, String> {
    require_admin(&state)?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    auth::create_user(&conn, &username, &pin, &full_name, &role)
}

#[tauri::command]
pub fn auth_update_pin(
    state: State<'_, AppState>,
    user_id: i64,
    current_pin: Option<String>,
    new_pin: String,
    is_admin_override: bool,
) -> Result<(), String> {
    let caller = state
        .current_user
        .lock()
        .map_err(|e| e.to_string())?
        .clone()
        .ok_or_else(|| "Sesi belum masuk: silakan login terlebih dahulu.".to_string())?;
    let is_admin = caller.role == "admin";
    if !is_admin && caller.id != user_id {
        return Err("Akses ditolak: Anda hanya dapat mengubah PIN akun Anda sendiri.".to_string());
    }
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    auth::update_user_pin(
        &conn,
        user_id,
        current_pin.as_deref(),
        &new_pin,
        is_admin_override && is_admin,
    )
}

#[tauri::command]
pub fn auth_delete_user(
    state: State<'_, AppState>,
    target_user_id: i64,
    current_user_id: Option<i64>,
) -> Result<(), String> {
    let _ = current_user_id;
    let admin_id = require_admin_id(&state)?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    auth::delete_user(&conn, target_user_id, admin_id)
}

// ==========================================
// Transactions & Ledger Commands
// ==========================================

#[tauri::command]
pub fn get_transactions(
    state: State<'_, AppState>,
    unit: Option<String>,
    month: Option<String>,
    search: Option<String>,
    page: Option<usize>,
    page_size: Option<usize>,
) -> Result<PaginatedTransactions, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::get_paginated_transactions(
        &conn,
        unit.as_deref(),
        month.as_deref(),
        search.as_deref(),
        page.unwrap_or(1),
        page_size.unwrap_or(25),
    )
}

#[tauri::command]
pub fn get_room_reservations(
    state: State<'_, AppState>,
    unit: Option<String>,
    month: Option<String>,
    search: Option<String>,
    page: Option<usize>,
    page_size: Option<usize>,
) -> Result<PaginatedRoomReservations, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::get_paginated_room_reservations(
        &conn,
        unit.as_deref(),
        month.as_deref(),
        search.as_deref(),
        page.unwrap_or(1),
        page_size.unwrap_or(25),
    )
}

#[tauri::command]
pub fn add_transaction(
    state: State<'_, AppState>,
    unit: String,
    date: String,
    amount: f64,
    notes: Option<String>,
    source_file: Option<String>,
) -> Result<i64, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::add_transaction(
        &conn,
        &unit,
        &date,
        amount,
        notes.as_deref(),
        source_file.as_deref(),
    )
}

#[tauri::command]
pub fn update_transaction(
    state: State<'_, AppState>,
    id: i64,
    unit: String,
    date: String,
    amount: f64,
    notes: Option<String>,
) -> Result<(), String> {
    require_admin(&state)?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::update_transaction(&conn, id, &unit, &date, amount, notes.as_deref())
}

#[tauri::command]
pub fn delete_transaction(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    require_admin(&state)?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::delete_transaction(&conn, id)
}

#[tauri::command]
pub fn update_room_reservation(
    state: State<'_, AppState>,
    payload: RoomReservationUpdate,
) -> Result<(), String> {
    require_admin(&state)?;
    let mut conn = state.db.lock().map_err(|e| e.to_string())?;
    db::update_room_reservation(&mut conn, &payload)
}

#[tauri::command]
pub fn delete_room_reservation(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    require_admin(&state)?;
    let mut conn = state.db.lock().map_err(|e| e.to_string())?;
    db::delete_room_reservation(&mut conn, id)
}

#[tauri::command]
pub fn export_room_reservations_excel(
    state: State<'_, AppState>,
    unit: Option<String>,
    month: Option<String>,
) -> Result<String, String> {
    let unit_filter = unit
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty() && *s != "all")
        .map(|s| s.to_string());
    let month_filter = month
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty() && *s != "all")
        .map(|s| s.to_string());

    let rows = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        db::get_room_reservations(&conn, unit_filter.as_deref(), month_filter.as_deref())?
    };

    if rows.is_empty() {
        return Err("Tidak ada data reservasi 13 kolom untuk diekspor pada filter ini.".to_string());
    }

    let unit_label = sanitize_filename_segment(unit_filter.as_deref().unwrap_or("Semua_Unit"));
    let month_label = sanitize_filename_segment(month_filter.as_deref().unwrap_or("Semua_Periode"));
    let default_name = format!("Rekap_Tamu_13Kolom_{}_{}.xlsx", unit_label, month_label);

    let dialog = rfd::FileDialog::new()
        .add_filter("Excel Workbook (*.xlsx)", &["xlsx"])
        .set_file_name(&default_name)
        .save_file();

    let target_path = match dialog {
        Some(p) => p,
        None => return Ok("Ekspor dibatalkan oleh pengguna.".to_string()),
    };

    export_room_reservations_to_excel(&rows, &target_path)
}

#[derive(Debug, Clone, Serialize)]
pub struct OriginReferenceList {
    pub domestic_regions: Vec<&'static str>,
    pub world_countries: Vec<&'static str>,
}

#[tauri::command]
pub fn get_origin_reference_list() -> OriginReferenceList {
    OriginReferenceList {
        domestic_regions: INDONESIAN_REGIONS.to_vec(),
        world_countries: ALL_WORLD_COUNTRIES.to_vec(),
    }
}

#[tauri::command]
pub fn delete_period_transactions(
    state: State<'_, AppState>,
    unit: String,
    start_date: String,
    end_date: String,
) -> Result<usize, String> {
    require_admin(&state)?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::delete_period_transactions(&conn, &unit, &start_date, &end_date)
}

#[tauri::command]
pub fn clean_duplicate_transactions(
    state: State<'_, AppState>,
    unit: Option<String>,
) -> Result<usize, String> {
    require_admin(&state)?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::clean_duplicate_transactions(&conn, unit.as_deref())
}

#[tauri::command]
pub fn get_available_months(
    state: State<'_, AppState>,
    unit: Option<String>,
) -> Result<Vec<String>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::get_available_months(&conn, unit.as_deref())
}

// ==========================================
// Tax Recapitulation Report Commands
// ==========================================

#[tauri::command]
pub fn get_tax_recap(
    state: State<'_, AppState>,
    unit: String,
    month: String,
) -> Result<TaxRecapReport, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    generate_tax_recap(&conn, &unit, &month)
}

#[tauri::command]
pub fn get_tourism_report(
    state: State<'_, AppState>,
    unit: String,
    month: String,
    room_capacity: Option<i64>,
) -> Result<TourismFormReport, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    generate_tourism_report(&conn, &unit, &month, room_capacity.unwrap_or(14))
}

// ==========================================
// Excel Importer & Log Commands
// ==========================================

#[tauri::command]
pub fn import_excel_file(
    state: State<'_, AppState>,
    file_path: String,
    options: ImportOptions,
) -> Result<ImportResult, String> {
    if options.mode == "skip_duplicates" {
        let has_session = state
            .current_user
            .lock()
            .map_err(|e| e.to_string())?
            .is_some();
        if !has_session {
            return Err("Sesi belum masuk: silakan login terlebih dahulu.".to_string());
        }
    } else {
        require_admin(&state)?;
    }
    let mut conn = state.db.lock().map_err(|e| e.to_string())?;
    import_excel(&mut conn, &file_path, &state.archive_dir, options)
}
#[tauri::command]
pub fn preview_import_file(
    state: State<'_, AppState>,
    file_path: String,
    target_unit: Option<String>,
) -> Result<ImportPreview, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    preview_excel_import(&conn, &file_path, target_unit)
}


#[tauri::command]
pub fn get_import_logs(
    state: State<'_, AppState>,
    unit: Option<String>,
) -> Result<Vec<ImportLogRow>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::get_import_logs(&conn, unit.as_deref())
}

#[tauri::command]
pub fn download_excel_template(unit: Option<String>) -> Result<String, String> {
    let u = sanitize_filename_segment(&unit.unwrap_or_else(|| "Bungalows".to_string()));
    let default_name = format!("Template_Import_Pendapatan_{}.xlsx", u);

    let dialog = rfd::FileDialog::new()
        .add_filter("Excel Workbook (*.xlsx)", &["xlsx"])
        .set_file_name(&default_name)
        .save_file();

    let target_path = match dialog {
        Some(p) => p,
        None => return Ok("Pengunduhan template dibatalkan oleh pengguna.".to_string()),
    };

    crate::excel::generate_import_template(&u, &target_path)?;

    Ok(format!(
        "Template Excel berhasil disimpan ke: {}",
        target_path.display()
    ))
}

#[tauri::command]
pub fn print_report(window: tauri::WebviewWindow, title: Option<String>) -> Result<(), String> {
    if let Some(t) = &title {
        let _ = window.set_title(t);
    }
    #[cfg(target_os = "macos")]
    {
        use objc2::msg_send;
        use objc2::runtime::{AnyClass, AnyObject};
        use std::ffi::CStr;

        if let Ok(cls_name) = CStr::from_bytes_with_nul(b"NSPrintInfo\0") {
            if let Some(cls) = AnyClass::get(cls_name) {
                // SAFETY: Running on macOS where AppKit's `NSPrintInfo` class exists;
                // `sharedPrintInfo` returns an optional shared instance whose `setOrientation:`
                // and margin selectors accept `isize` (NSInteger) and `f64` (CGFloat).
                unsafe {
                    let print_info: Option<&AnyObject> = msg_send![cls, sharedPrintInfo];
                    if let Some(info) = print_info {
                        // 1 = NSPaperOrientationLandscape (0 = Portrait)
                        let _: () = msg_send![info, setOrientation: 1isize];
                        let _: () = msg_send![info, setLeftMargin: 15.0f64];
                        let _: () = msg_send![info, setRightMargin: 15.0f64];
                        let _: () = msg_send![info, setTopMargin: 15.0f64];
                        let _: () = msg_send![info, setBottomMargin: 15.0f64];
                    }
                }
            }
        }
    }
    window.print().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_window_title(window: tauri::WebviewWindow, title: String) -> Result<(), String> {
    window.set_title(&title).map_err(|e| e.to_string())
}
// ==========================================
// Unit Management Commands
// ==========================================

#[tauri::command]
pub fn get_units(state: State<'_, AppState>) -> Result<Vec<UnitRow>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::get_units(&conn)
}

#[tauri::command]
pub fn add_unit(
    state: State<'_, AppState>,
    code: String,
    name: String,
    tax_rate: f64,
    address: Option<String>,
    npwpd: Option<String>,
) -> Result<i64, String> {
    require_admin(&state)?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::add_unit(
        &conn,
        &code,
        &name,
        tax_rate,
        address.as_deref(),
        npwpd.as_deref(),
    )
}

#[tauri::command]
pub fn update_unit(
    state: State<'_, AppState>,
    id: i64,
    name: String,
    tax_rate: f64,
    address: Option<String>,
    npwpd: Option<String>,
    is_active: bool,
) -> Result<(), String> {
    require_admin(&state)?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::update_unit(
        &conn,
        id,
        &name,
        tax_rate,
        address.as_deref(),
        npwpd.as_deref(),
        is_active,
    )
}

// ==========================================
// Settings Commands
// ==========================================

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<HashMap<String, String>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::get_settings(&conn)
}

#[tauri::command]
pub fn update_settings(
    state: State<'_, AppState>,
    settings: HashMap<String, String>,
) -> Result<(), String> {
    require_admin(&state)?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::update_settings(&conn, &settings)
}

// ==========================================
// Storage & Disaster Recovery (Backup/Restore)
// ==========================================

#[tauri::command]
pub fn get_storage_info(state: State<'_, AppState>) -> Result<StorageInfo, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::get_storage_info(&conn, &state.db_path, &state.archive_dir)
}

#[tauri::command]
pub fn backup_database_dialog(state: State<'_, AppState>) -> Result<String, String> {
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    let default_name = format!("laporan_bapenda_backup_{}.sqlite", timestamp);

    let save_target = rfd::FileDialog::new()
        .add_filter("SQLite Database (*.sqlite, *.bak)", &["sqlite", "bak", "db"])
        .set_file_name(&default_name)
        .save_file();

    let target_path = match save_target {
        Some(p) => p,
        None => return Ok("Pencadangan dibatalkan oleh pengguna.".to_string()),
    };

    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::backup_database(&conn, &target_path)?;

    Ok(format!(
        "Database berhasil dicadangkan ke: {}",
        target_path.display()
    ))
}

#[tauri::command]
pub fn restore_database_dialog(state: State<'_, AppState>) -> Result<String, String> {
    require_admin(&state)?;
    let pick_target = rfd::FileDialog::new()
        .add_filter("SQLite Database (*.sqlite, *.bak)", &["sqlite", "bak", "db"])
        .pick_file();

    let backup_file = match pick_target {
        Some(p) => p,
        None => return Ok("Pemulihan dibatalkan oleh pengguna.".to_string()),
    };

    let mut conn = state.db.lock().map_err(|e| e.to_string())?;

    // Perform file restore
    db::restore_database(&state.db_path, &backup_file)?;

    // Reopen SQLite connection to switch to restored DB cleanly
    let new_conn = db::open_connection(&state.db_path)
        .map_err(|e| format!("Gagal memuat ulang koneksi database: {}", e))?;
    *conn = new_conn;

    Ok("Database berhasil dipulihkan dari cadangan!".to_string())
}
