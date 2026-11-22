use crate::auth::seed_default_users;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionRow {
    pub id: i64,
    pub unit: String,
    pub date: String,
    pub amount: f64,
    pub notes: Option<String>,
    pub source_file: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedTransactions {
    pub items: Vec<TransactionRow>,
    pub total_count: usize,
    pub total_amount: f64,
    pub page: usize,
    pub page_size: usize,
    pub total_pages: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedRoomReservations {
    pub items: Vec<RoomReservationRow>,
    pub total_count: usize,
    pub total_person: i64,
    pub total_rooms: i64,
    pub total_nights: i64,
    pub total_payment: f64,
    pub page: usize,
    pub page_size: usize,
    pub total_pages: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportLogRow {
    pub id: i64,
    pub unit: Option<String>,
    pub file_name: Option<String>,
    pub records_count: i64,
    pub total_amount: f64,
    pub period: Option<String>,
    pub file_hash: Option<String>,
    pub imported_at: Option<String>,
    pub stored_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomReservationRow {
    pub id: i64,
    pub unit: String,
    pub transaction_id: Option<i64>,
    pub date: String,
    pub guest_name: String,
    pub total_person: Option<i64>,
    pub booking_number: Option<String>,
    pub nationality: Option<String>,
    pub room_number: Option<String>,
    pub total_rooms: i64,
    pub check_in_date: String,
    pub check_out_date: String,
    pub total_nights: Option<i64>,
    pub rate: Option<f64>,
    pub total_payment: f64,
    pub remarks: Option<String>,
    pub source_file: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomReservationUpdate {
    pub id: i64,
    pub date: String,
    pub guest_name: String,
    pub total_person: i64,
    pub booking_number: Option<String>,
    pub nationality: Option<String>,
    pub room_number: Option<String>,
    pub total_rooms: i64,
    pub check_in_date: String,
    pub check_out_date: String,
    pub total_payment: f64,
    pub remarks: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitRow {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub tax_rate: f64,
    pub address: Option<String>,
    pub npwpd: Option<String>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageInfo {
    pub db_path: String,
    pub db_size_bytes: u64,
    pub archive_dir: String,
    pub archive_size_bytes: u64,
    pub total_transactions: usize,
    pub total_units: usize,
    pub total_import_logs: usize,
}

/// Resolves standard app data and archive directories.
pub fn get_default_paths() -> (PathBuf, PathBuf) {
    let base_dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("com.andabungalows.erp");
    let archive_dir = base_dir.join("arsip_excel");
    let db_path = base_dir.join("laporan_bapenda.sqlite");
    (db_path, archive_dir)
}

/// Discovers legacy databases from previous Electron installations if present.
pub fn find_legacy_database() -> Option<PathBuf> {
    let candidate_dirs = ["tax-report-app", "laporan-bapenda-dispenda", "Laporan dispenda"];
    if let Some(data_dir) = dirs::data_dir() {
        for app in candidate_dirs {
            let candidate = data_dir.join(app).join("laporan_bapenda.sqlite");
            if candidate.exists() && candidate.is_file() {
                if let Ok(meta) = fs::metadata(&candidate) {
                    if meta.len() > 0 {
                        return Some(candidate);
                    }
                }
            }
        }
    }
    None
}

/// Opens connection and sets WAL PRAGMAs.
pub fn open_connection(db_path: &Path) -> Result<Connection, rusqlite::Error> {
    if let Some(parent) = db_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let conn = Connection::open(db_path)?;
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;
         PRAGMA foreign_keys = ON;
         PRAGMA busy_timeout = 5000;",
    )?;
    Ok(conn)
}

/// Inits database, runs schema migrations, seeds default users, units, and settings.
pub fn init_db(db_path: &Path) -> Result<Connection, String> {
    init_db_with_migration(db_path, false)
}

/// Inits database with optional auto-migration from legacy Electron data directories.
pub fn init_db_with_migration(db_path: &Path, auto_migrate: bool) -> Result<Connection, String> {
    // Check if target DB exists; if not, attempt WAL-safe auto-migration from legacy installation
    if auto_migrate && !db_path.exists() {
        if let Some(legacy_path) = find_legacy_database() {
            if let Some(parent) = db_path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let migrated = match open_connection(&legacy_path) {
                Ok(legacy_conn) => backup_database(&legacy_conn, db_path).is_ok(),
                Err(_) => false,
            };
            if !migrated {
                let _ = fs::copy(&legacy_path, db_path);
            }
        }
    }

    let conn = open_connection(db_path).map_err(|e| format!("Gagal membuka koneksi database: {}", e))?;

    // Schema Migrations
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS units (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            code TEXT UNIQUE NOT NULL,
            name TEXT NOT NULL,
            tax_rate REAL DEFAULT 0.10,
            address TEXT,
            npwpd TEXT,
            is_active INTEGER DEFAULT 1
        );

        CREATE TABLE IF NOT EXISTS transactions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            unit TEXT NOT NULL,
            date TEXT NOT NULL,
            amount REAL NOT NULL,
            notes TEXT,
            source_file TEXT,
            created_at DATETIME DEFAULT (datetime('now', 'localtime'))
        );

        CREATE INDEX IF NOT EXISTS idx_trans_unit_date ON transactions(unit, date);
        CREATE INDEX IF NOT EXISTS idx_trans_date ON transactions(date);

        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT
        );

        CREATE TABLE IF NOT EXISTS import_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            unit TEXT,
            file_name TEXT,
            records_count INTEGER,
            total_amount REAL,
            period TEXT,
            file_hash TEXT,
            imported_at DATETIME DEFAULT (datetime('now', 'localtime')),
            stored_path TEXT
        );

        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT UNIQUE NOT NULL,
            pin_hash TEXT NOT NULL,
            salt TEXT NOT NULL,
            full_name TEXT NOT NULL,
            role TEXT NOT NULL CHECK(role IN ('admin', 'staff')),
            created_at DATETIME DEFAULT (datetime('now', 'localtime')),
            last_login DATETIME
        );

        CREATE TABLE IF NOT EXISTS room_reservations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            unit TEXT NOT NULL DEFAULT 'Bungalows',
            transaction_id INTEGER REFERENCES transactions(id) ON DELETE CASCADE,
            date TEXT NOT NULL,
            guest_name TEXT NOT NULL,
            total_person INTEGER,
            booking_number TEXT,
            nationality TEXT,
            room_number TEXT,
            total_rooms INTEGER NOT NULL DEFAULT 1,
            check_in_date TEXT NOT NULL,
            check_out_date TEXT NOT NULL,
            total_nights INTEGER,
            rate REAL,
            total_payment REAL NOT NULL,
            remarks TEXT,
            source_file TEXT,
            created_at DATETIME DEFAULT (datetime('now', 'localtime'))
        );

        CREATE INDEX IF NOT EXISTS idx_room_res_date ON room_reservations(date);
        CREATE INDEX IF NOT EXISTS idx_room_res_checkin ON room_reservations(check_in_date);
        CREATE INDEX IF NOT EXISTS idx_room_res_unit_date ON room_reservations(unit, date);
        CREATE INDEX IF NOT EXISTS idx_room_res_trans_id ON room_reservations(transaction_id);

        CREATE TABLE IF NOT EXISTS training_certificates (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            certificate_no TEXT NOT NULL UNIQUE COLLATE NOCASE,
            recipient_name TEXT NOT NULL,
            duration_text TEXT NOT NULL,
            program_name TEXT NOT NULL DEFAULT 'hospitality training program',
            start_date TEXT NOT NULL,
            end_date TEXT NOT NULL,
            issued_date TEXT NOT NULL,
            department TEXT,
            institution TEXT,
            achievement_text TEXT,
            notes TEXT,
            signer_name TEXT NOT NULL DEFAULT 'Agus Sumardi Wijaya',
            signer_title TEXT NOT NULL DEFAULT 'General Manager',
            paper_size TEXT NOT NULL DEFAULT 'F4' CHECK(paper_size IN ('F4', 'A4')),
            status TEXT NOT NULL DEFAULT 'active' CHECK(status IN ('active', 'revoked')),
            department_code TEXT,
            program_type_code TEXT,
            issued_city TEXT,
            body_template TEXT,
            created_by TEXT,
            created_at DATETIME DEFAULT (datetime('now', 'localtime')),
            updated_at DATETIME DEFAULT (datetime('now', 'localtime'))
        );

        CREATE INDEX IF NOT EXISTS idx_cert_issued_date ON training_certificates(issued_date);
        CREATE INDEX IF NOT EXISTS idx_cert_status ON training_certificates(status);
        CREATE INDEX IF NOT EXISTS idx_cert_recipient ON training_certificates(recipient_name);

        CREATE TABLE IF NOT EXISTS departments (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            code TEXT NOT NULL UNIQUE COLLATE NOCASE,
            name TEXT NOT NULL,
            program_description TEXT NOT NULL DEFAULT '',
            competencies TEXT NOT NULL DEFAULT '',
            is_active INTEGER NOT NULL DEFAULT 1,
            sort_order INTEGER NOT NULL DEFAULT 0,
            created_at DATETIME DEFAULT (datetime('now', 'localtime'))
        );

        CREATE TABLE IF NOT EXISTS program_types (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            code TEXT NOT NULL UNIQUE COLLATE NOCASE,
            name TEXT NOT NULL,
            default_duration_months INTEGER NOT NULL DEFAULT 3,
            description TEXT NOT NULL DEFAULT '',
            is_active INTEGER NOT NULL DEFAULT 1,
            sort_order INTEGER NOT NULL DEFAULT 0,
            created_at DATETIME DEFAULT (datetime('now', 'localtime'))
        );

        CREATE TABLE IF NOT EXISTS certificate_sequences (
            year INTEGER PRIMARY KEY,
            last_seq INTEGER NOT NULL DEFAULT 0
        );",
    )
    .map_err(|e| format!("Gagal membuat skema database: {}", e))?;
    // Auto-migration for existing databases
    let _ = conn.execute("ALTER TABLE import_logs ADD COLUMN period TEXT", []);
    let _ = conn.execute("ALTER TABLE import_logs ADD COLUMN file_hash TEXT", []);
    let _ = conn.execute("ALTER TABLE training_certificates ADD COLUMN achievement_text TEXT", []);
    let _ = conn.execute("ALTER TABLE training_certificates ADD COLUMN department_code TEXT", []);
    let _ = conn.execute("ALTER TABLE training_certificates ADD COLUMN program_type_code TEXT", []);
    let _ = conn.execute("ALTER TABLE training_certificates ADD COLUMN issued_city TEXT", []);
    let _ = conn.execute("ALTER TABLE training_certificates ADD COLUMN body_template TEXT", []);
    // Backfill period for legacy import_logs from transactions
    let _ = conn.execute(
        "UPDATE import_logs 
         SET period = (
             SELECT GROUP_CONCAT(DISTINCT strftime('%Y-%m', t.date)) 
             FROM transactions t 
             WHERE t.source_file = import_logs.file_name AND t.unit = import_logs.unit
         )
         WHERE (period IS NULL OR period = '') 
           AND EXISTS (
             SELECT 1 FROM transactions t 
             WHERE t.source_file = import_logs.file_name AND t.unit = import_logs.unit
         )",
        [],
    );

    // Fallback backfill from imported_at
    let _ = conn.execute(
        "UPDATE import_logs 
         SET period = strftime('%Y-%m', imported_at) 
         WHERE (period IS NULL OR period = '') AND imported_at IS NOT NULL",
        [],
    );
    // Synchronize legacy UTC timestamps in import_logs with local timestamps from stored_path
    if let Ok(mut stmt) = conn.prepare("SELECT id, stored_path, imported_at FROM import_logs WHERE stored_path IS NOT NULL") {
        let rows: Vec<(i64, String, Option<String>)> = stmt
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
            .ok()
            .map(|mapped| mapped.filter_map(|res| res.ok()).collect())
            .unwrap_or_default();

        for (log_id, path_str, existing_imported_at) in rows {
            if let Some(file_name) = std::path::Path::new(&path_str).file_name().and_then(|f| f.to_str()) {
                let parts: Vec<&str> = file_name.split('_').collect();
                if parts.len() >= 2 && parts[0].len() == 8 && parts[1].len() == 6 {
                    let d = parts[0];
                    let t = parts[1];
                    let local_ts = format!(
                        "{}-{}-{} {}:{}:{}",
                        &d[0..4], &d[4..6], &d[6..8],
                        &t[0..2], &t[2..4], &t[4..6]
                    );
                    if existing_imported_at.as_deref() != Some(local_ts.as_str()) {
                        let _ = conn.execute(
                            "UPDATE import_logs SET imported_at = ?1 WHERE id = ?2",
                            rusqlite::params![local_ts, log_id],
                        );
                    }
                }
            }
        }
    }



    // Seed default units
    let unit_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM units", [], |r| r.get(0))
        .unwrap_or(0);
    if unit_count == 0 {
        conn.execute_batch(
            "INSERT OR IGNORE INTO units (code, name, tax_rate, is_active) VALUES
                ('Bungalows', 'ANDA BUNGALOWS', 0.10, 1),
                ('Restaurant', 'ANDA RESTAURANT', 0.10, 1);",
        )
        .map_err(|e| format!("Gagal inisialisasi default units: {}", e))?;
    }

    // Seed default settings
    let default_settings = [
        ("businessName", "ANDA BUNGALOWS & RESTAURANT"),
        (
            "businessAddress",
            "Jalan Pariwisata Pantai Kuta, Kecamatan Pujut, Lombok Tengah, NTB",
        ),
        ("contactNumber", "HP/WhatsApp: +62818-0578-3810"),
        ("npwpd", "P.2.0001234.01.23"),
        ("taxRate", "0.10"),
        ("signPosition", "Wajib Pajak Daerah"),
        ("signName", "Pimpinan / Pengelola"),
        ("officerName", "Petugas Bapenda Lombok Tengah"),
        ("officerNip", "NIP. 19800101 200501 1 001"),
        ("certSignerName", "Agus Sumardi Wijaya"),
        ("certSignerTitle", "General Manager"),
        ("certProgramName", "hospitality training program"),
        ("certPaperSize", "F4"),
        ("certIssuedCity", crate::certificate::DEFAULT_ISSUED_CITY),
        ("certBodyTemplate", crate::certificate::DEFAULT_BODY_TEMPLATE),
        ("certShowSignature", "0"),
        ("certLogoFile", ""),
        ("certSignatureFile", ""),
    ];

    for (k, v) in default_settings.iter() {
        let exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM settings WHERE key = ?1",
                params![k],
                |r| r.get(0),
            )
            .unwrap_or(0);
        if exists == 0 {
            let _ = conn.execute(
                "INSERT INTO settings (key, value) VALUES (?1, ?2)",
                params![k, v],
            );
        }
    }

    // Seed default users (admin & staff)
    let _ = seed_default_users(&conn);
    // Seed initial hospitality departments and program types when empty
    let _ = seed_default_catalog(&conn);

    Ok(conn)
}

/// Retrieves paginated transactions with optional unit, month, and text search filter.
pub fn get_paginated_transactions(
    conn: &Connection,
    unit: Option<&str>,
    month: Option<&str>,
    search: Option<&str>,
    page: usize,
    page_size: usize,
) -> Result<PaginatedTransactions, String> {
    let p = if page == 0 { 1 } else { page };
    let size = if page_size == 0 { 25 } else { page_size };
    let offset = (p - 1) * size;

    let mut where_clauses: Vec<String> = Vec::new();
    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(u) = unit {
        if !u.is_empty() && u != "all" {
            where_clauses.push(format!("unit = ?{}", params_vec.len() + 1));
            params_vec.push(Box::new(u.to_string()));
        }
    }

    if let Some(m) = month {
        if !m.is_empty() && m != "all" {
            where_clauses.push(format!("date LIKE ?{}", params_vec.len() + 1));
            params_vec.push(Box::new(format!("{}%", m)));
        }
    }

    if let Some(s) = search {
        let clean = s.trim();
        if !clean.is_empty() {
            let idx = params_vec.len() + 1;
            where_clauses.push(format!(
                "(notes LIKE ?{0} OR source_file LIKE ?{0} OR date LIKE ?{0})",
                idx
            ));
            params_vec.push(Box::new(format!("%{}%", clean)));
        }
    }

    let where_str = if where_clauses.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", where_clauses.join(" AND "))
    };

    // Count and Sum query
    let count_sql = format!(
        "SELECT COUNT(*), COALESCE(SUM(amount), 0.0) FROM transactions {}",
        where_str
    );

    let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|b| b.as_ref()).collect();

    let (total_count, total_amount): (i64, f64) = conn
        .query_row(&count_sql, params_refs.as_slice(), |row| {
            Ok((row.get(0)?, row.get(1)?))
        })
        .map_err(|e| format!("Gagal menghitung transaksi: {}", e))?;

    // Items query
    let limit_param_idx = params_vec.len() + 1;
    let offset_param_idx = params_vec.len() + 2;

    let items_sql = format!(
        "SELECT id, unit, date, amount, notes, source_file, created_at FROM transactions {} ORDER BY date DESC, id DESC LIMIT ?{} OFFSET ?{}",
        where_str, limit_param_idx, offset_param_idx
    );

    let mut query_params: Vec<Box<dyn rusqlite::ToSql>> = params_vec;
    query_params.push(Box::new(size as i64));
    query_params.push(Box::new(offset as i64));

    let query_refs: Vec<&dyn rusqlite::ToSql> = query_params.iter().map(|b| b.as_ref()).collect();

    let mut stmt = conn
        .prepare(&items_sql)
        .map_err(|e| format!("Gagal mempersiapkan query transaksi: {}", e))?;

    let rows = stmt
        .query_map(query_refs.as_slice(), |row| {
            Ok(TransactionRow {
                id: row.get(0)?,
                unit: row.get(1)?,
                date: row.get(2)?,
                amount: row.get(3)?,
                notes: row.get(4)?,
                source_file: row.get(5)?,
                created_at: row.get(6)?,
            })
        })
        .map_err(|e| format!("Gagal menjalankan query transaksi: {}", e))?;

    let mut items = Vec::new();
    for r in rows {
        items.push(r.map_err(|e| e.to_string())?);
    }

    let total_pages = if total_count == 0 {
        1
    } else {
        ((total_count as usize) + size - 1) / size
    };

    Ok(PaginatedTransactions {
        items,
        total_count: total_count as usize,
        total_amount,
        page: p,
        page_size: size,
        total_pages,
    })
}

fn normalize_iso_date(input: &str, field_label: &str) -> Result<(chrono::NaiveDate, String), String> {
    let trimmed = input.trim();
    let d = crate::excel_importer::parse_date_string(trimmed).ok_or_else(|| {
        format!(
            "Format {} tidak valid (gunakan YYYY-MM-DD, DD/MM/YY, atau DD/MM/YYYY).",
            field_label
        )
    })?;
    Ok((d, d.format("%Y-%m-%d").to_string()))
}

/// Adds a single transaction manual row.
pub fn add_transaction(
    conn: &Connection,
    unit: &str,
    date: &str,
    amount: f64,
    notes: Option<&str>,
    source_file: Option<&str>,
) -> Result<i64, String> {
    let unit_trimmed = unit.trim();
    if unit_trimmed.is_empty() {
        return Err("Unit bisnis wajib dipilih.".to_string());
    }
    let (_, date_str) = normalize_iso_date(date, "tanggal transaksi")?;
    if !amount.is_finite() || amount < 0.0 {
        return Err("Nominal transaksi tidak valid atau bernilai negatif.".to_string());
    }

    let now_local = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    conn.execute(
        "INSERT INTO transactions (unit, date, amount, notes, source_file, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![unit_trimmed, date_str, amount, notes, source_file, now_local],
    )
    .map_err(|e| format!("Gagal menambah transaksi: {}", e))?;

    Ok(conn.last_insert_rowid())
}

/// Updates an existing transaction and synchronizes any linked 13-column reservation.
pub fn update_transaction(
    conn: &Connection,
    id: i64,
    unit: &str,
    date: &str,
    amount: f64,
    notes: Option<&str>,
) -> Result<(), String> {
    let unit_trimmed = unit.trim();
    if unit_trimmed.is_empty() {
        return Err("Unit bisnis wajib dipilih.".to_string());
    }
    let (new_date_parsed, date_str) = normalize_iso_date(date, "tanggal transaksi")?;
    if !amount.is_finite() || amount < 0.0 {
        return Err("Nominal transaksi tidak valid atau bernilai negatif.".to_string());
    }

    let tx = conn
        .unchecked_transaction()
        .map_err(|e| format!("Gagal memulai transaksi database: {}", e))?;
    let affected = tx
        .execute(
            "UPDATE transactions SET unit = ?1, date = ?2, amount = ?3, notes = ?4 WHERE id = ?5",
            params![unit_trimmed, date_str, amount, notes, id],
        )
        .map_err(|e| format!("Gagal memperbarui transaksi: {}", e))?;
    if affected == 0 {
        return Err("Transaksi tidak ditemukan.".to_string());
    }

    let linked_rows: Vec<(i64, String, String, String, Option<i64>)> = {
        let mut stmt = tx
            .prepare("SELECT id, date, check_in_date, check_out_date, total_nights FROM room_reservations WHERE transaction_id = ?1")
            .map_err(|e| format!("Gagal mempersiapkan sinkronisasi reservasi terkait: {}", e))?;
        let mapped = stmt
            .query_map(params![id], |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
            })
            .map_err(|e| format!("Gagal membaca rincian reservasi terkait: {}", e))?;
        let mut rows = Vec::new();
        for item in mapped {
            rows.push(item.map_err(|e| e.to_string())?);
        }
        rows
    };

    for (res_id, old_date_s, old_ci_s, old_co_s, nights_opt) in linked_rows {
        let old_date = crate::excel_importer::parse_date_string(&old_date_s);
        let old_ci = crate::excel_importer::parse_date_string(&old_ci_s);
        let old_co = crate::excel_importer::parse_date_string(&old_co_s);

        if let Some(od) = old_date {
            if od != new_date_parsed {
                let delta = new_date_parsed - od;
                let new_ci = old_ci.map(|ci| ci + delta).unwrap_or(new_date_parsed);
                let fallback_nights = nights_opt.unwrap_or(1).max(1);
                let new_co = match (old_ci, old_co) {
                    (Some(ci), Some(co)) if co > ci => new_ci + (co - ci),
                    _ => new_ci + chrono::Duration::days(fallback_nights),
                };
                let new_ci_str = new_ci.format("%Y-%m-%d").to_string();
                let new_co_str = new_co.format("%Y-%m-%d").to_string();
                tx.execute(
                    "UPDATE room_reservations
                     SET unit = ?1,
                         date = ?2,
                         check_in_date = ?3,
                         check_out_date = ?4,
                         total_payment = ?5,
                         rate = CAST(?5 AS REAL) / (MAX(COALESCE(total_nights, 1), 1) * MAX(total_rooms, 1))
                     WHERE id = ?6",
                    params![unit_trimmed, date_str, new_ci_str, new_co_str, amount, res_id],
                )
                .map_err(|e| format!("Gagal menyinkronkan tanggal reservasi terkait: {}", e))?;
                continue;
            }
        }

        tx.execute(
            "UPDATE room_reservations
             SET unit = ?1,
                 date = ?2,
                 total_payment = ?3,
                 rate = CAST(?3 AS REAL) / (MAX(COALESCE(total_nights, 1), 1) * MAX(total_rooms, 1))
             WHERE id = ?4",
            params![unit_trimmed, date_str, amount, res_id],
        )
        .map_err(|e| format!("Gagal menyinkronkan rincian reservasi terkait: {}", e))?;
    }

    tx.commit()
        .map_err(|e| format!("Gagal menyimpan pembaruan transaksi: {}", e))?;
    Ok(())
}

/// Deletes a transaction by id along with any linked 13-column reservation.
pub fn delete_transaction(conn: &Connection, id: i64) -> Result<(), String> {
    let tx = conn
        .unchecked_transaction()
        .map_err(|e| format!("Gagal memulai transaksi database: {}", e))?;
    tx.execute(
        "DELETE FROM room_reservations WHERE transaction_id = ?1",
        params![id],
    )
    .map_err(|e| format!("Gagal menghapus rincian reservasi terkait: {}", e))?;
    tx.execute("DELETE FROM transactions WHERE id = ?1", params![id])
        .map_err(|e| format!("Gagal menghapus transaksi: {}", e))?;
    tx.commit()
        .map_err(|e| format!("Gagal menyelesaikan penghapusan transaksi: {}", e))?;
    Ok(())
}

fn clean_optional_field(val: &Option<String>) -> Option<String> {
    val.as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty() && *s != "-")
        .map(|s| s.to_string())
}

/// Atomically updates a 13-column room reservation and its linked summary transaction.
pub fn update_room_reservation(
    conn: &mut Connection,
    payload: &RoomReservationUpdate,
) -> Result<(), String> {
    let guest_name = payload.guest_name.trim();
    if guest_name.is_empty() {
        return Err("Nama tamu wajib diisi.".to_string());
    }

    let (_, date_str) = normalize_iso_date(&payload.date, "DATE")?;
    let (ci, ci_str) = normalize_iso_date(&payload.check_in_date, "Date Check In")?;
    let (co, co_str) = normalize_iso_date(&payload.check_out_date, "Date Check Out")?;

    if co <= ci {
        return Err("Tanggal Check Out harus setelah tanggal Check In.".to_string());
    }
    if payload.total_person < 1 {
        return Err("Total person minimal 1 orang.".to_string());
    }
    if payload.total_rooms < 1 {
        return Err("Total Kamar minimal 1 kamar.".to_string());
    }
    if !payload.total_payment.is_finite() || payload.total_payment < 0.0 {
        return Err("Total Payment tidak valid atau bernilai negatif.".to_string());
    }

    let nights = (co - ci).num_days().max(1);
    let rate = payload.total_payment / (nights as f64 * payload.total_rooms as f64);

    let booking_number = clean_optional_field(&payload.booking_number);
    let nationality = clean_optional_field(&payload.nationality);
    let room_number = clean_optional_field(&payload.room_number);
    let remarks = clean_optional_field(&payload.remarks);

    let tx = conn
        .transaction()
        .map_err(|e| format!("Gagal memulai transaksi database: {}", e))?;

    let trans_id: Option<i64> = tx
        .query_row(
            "SELECT transaction_id FROM room_reservations WHERE id = ?1",
            params![payload.id],
            |r| r.get(0),
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => "Data reservasi tidak ditemukan.".to_string(),
            other => format!("Gagal membaca data reservasi: {}", other),
        })?;

    tx.execute(
        "UPDATE room_reservations
         SET date = ?1, guest_name = ?2, total_person = ?3, booking_number = ?4,
             nationality = ?5, room_number = ?6, total_rooms = ?7, check_in_date = ?8,
             check_out_date = ?9, total_nights = ?10, rate = ?11, total_payment = ?12,
             remarks = ?13
         WHERE id = ?14",
        params![
            date_str,
            guest_name,
            payload.total_person,
            booking_number,
            nationality,
            room_number,
            payload.total_rooms,
            ci_str,
            co_str,
            nights,
            rate,
            payload.total_payment,
            remarks,
            payload.id
        ],
    )
    .map_err(|e| format!("Gagal memperbarui rincian reservasi: {}", e))?;

    if let Some(tid) = trans_id {
        let formatted_notes = match (&room_number, &remarks) {
            (Some(rm), Some(rem)) => format!("{} ({}) - Kamar {}", guest_name, rem, rm),
            (None, Some(rem)) => format!("{} ({})", guest_name, rem),
            (Some(rm), None) => format!("{} - Kamar {}", guest_name, rm),
            (None, None) => guest_name.to_string(),
        };
        tx.execute(
            "UPDATE transactions SET date = ?1, amount = ?2, notes = ?3 WHERE id = ?4",
            params![date_str, payload.total_payment, formatted_notes, tid],
        )
        .map_err(|e| format!("Gagal menyinkronkan transaksi pendapatan: {}", e))?;
    }

    tx.commit()
        .map_err(|e| format!("Gagal menyimpan pembaruan reservasi: {}", e))?;
    Ok(())
}

/// Atomically deletes a 13-column room reservation and its linked summary transaction.
pub fn delete_room_reservation(conn: &mut Connection, id: i64) -> Result<(), String> {
    let tx = conn
        .transaction()
        .map_err(|e| format!("Gagal memulai transaksi database: {}", e))?;

    let trans_id: Option<i64> = tx
        .query_row(
            "SELECT transaction_id FROM room_reservations WHERE id = ?1",
            params![id],
            |r| r.get(0),
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => "Data reservasi tidak ditemukan.".to_string(),
            other => format!("Gagal membaca data reservasi: {}", other),
        })?;

    tx.execute("DELETE FROM room_reservations WHERE id = ?1", params![id])
        .map_err(|e| format!("Gagal menghapus data reservasi: {}", e))?;

    if let Some(tid) = trans_id {
        tx.execute("DELETE FROM transactions WHERE id = ?1", params![tid])
            .map_err(|e| format!("Gagal menghapus transaksi pendapatan terkait: {}", e))?;
    }

    tx.commit()
        .map_err(|e| format!("Gagal menyelesaikan penghapusan reservasi: {}", e))?;
    Ok(())
}

/// Deletes all transactions for a specific unit in a date range.
pub fn delete_period_transactions(
    conn: &Connection,
    unit: &str,
    start_date: &str,
    end_date: &str,
) -> Result<usize, String> {
    let (start_d, start_iso) = normalize_iso_date(start_date, "tanggal mulai")?;
    let (end_d, end_iso) = normalize_iso_date(end_date, "tanggal akhir")?;
    if end_d < start_d {
        return Err("Tanggal akhir periode tidak boleh lebih awal dari tanggal mulai.".to_string());
    }

    let tx = conn
        .unchecked_transaction()
        .map_err(|e| format!("Gagal memulai transaksi penghapusan periode: {}", e))?;
    tx.execute(
        "DELETE FROM room_reservations WHERE unit = ?1 AND date BETWEEN ?2 AND ?3",
        params![unit, start_iso, end_iso],
    )
    .map_err(|e| format!("Gagal menghapus data reservasi periode: {}", e))?;
    let affected = tx
        .execute(
            "DELETE FROM transactions WHERE unit = ?1 AND date BETWEEN ?2 AND ?3",
            params![unit, start_iso, end_iso],
        )
        .map_err(|e| format!("Gagal menghapus periode transaksi: {}", e))?;
    tx.commit()
        .map_err(|e| format!("Gagal menyelesaikan penghapusan periode: {}", e))?;
    Ok(affected)
}

fn map_room_reservation_row(r: &rusqlite::Row) -> rusqlite::Result<RoomReservationRow> {
    Ok(RoomReservationRow {
        id: r.get(0)?,
        unit: r.get(1)?,
        transaction_id: r.get(2)?,
        date: r.get(3)?,
        guest_name: r.get(4)?,
        total_person: r.get(5)?,
        booking_number: r.get(6)?,
        nationality: r.get(7)?,
        room_number: r.get(8)?,
        total_rooms: r.get(9)?,
        check_in_date: r.get(10)?,
        check_out_date: r.get(11)?,
        total_nights: r.get(12)?,
        rate: r.get(13)?,
        total_payment: r.get(14)?,
        remarks: r.get(15)?,
        source_file: r.get(16)?,
        created_at: r.get(17)?,
    })
}

/// Retrieves room reservations with optional unit and month filter.
pub fn get_room_reservations(
    conn: &Connection,
    unit: Option<&str>,
    month: Option<&str>,
) -> Result<Vec<RoomReservationRow>, String> {
    let mut sql = "SELECT id, unit, transaction_id, date, guest_name, total_person, booking_number, nationality, room_number, total_rooms, check_in_date, check_out_date, total_nights, rate, total_payment, remarks, source_file, created_at FROM room_reservations WHERE 1=1".to_string();
    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(u) = unit {
        sql.push_str(" AND unit = ?");
        params_vec.push(Box::new(u.to_string()));
    }
    if let Some(m) = month {
        sql.push_str(" AND date BETWEEN ? AND ?");
        params_vec.push(Box::new(format!("{}-01", m)));
        params_vec.push(Box::new(format!("{}-31", m)));
    }
    sql.push_str(" ORDER BY date ASC, id ASC");

    let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|b| b.as_ref()).collect();
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(&params_refs[..], map_room_reservation_row)
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(rows)
}

/// Retrieves paginated 13-column room reservations with unit, month, search filters and summary metrics.
pub fn get_paginated_room_reservations(
    conn: &Connection,
    unit: Option<&str>,
    month: Option<&str>,
    search: Option<&str>,
    page: usize,
    page_size: usize,
) -> Result<PaginatedRoomReservations, String> {
    let p = if page == 0 { 1 } else { page };
    let size = if page_size == 0 { 25 } else { page_size };
    let offset = (p - 1) * size;

    let mut where_clauses: Vec<String> = Vec::new();
    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(u) = unit {
        if !u.is_empty() && u != "all" {
            where_clauses.push(format!("unit = ?{}", params_vec.len() + 1));
            params_vec.push(Box::new(u.to_string()));
        }
    }

    if let Some(m) = month {
        if !m.is_empty() && m != "all" {
            where_clauses.push(format!("date LIKE ?{}", params_vec.len() + 1));
            params_vec.push(Box::new(format!("{}%", m)));
        }
    }

    if let Some(s) = search {
        let clean = s.trim();
        if !clean.is_empty() {
            let idx = params_vec.len() + 1;
            where_clauses.push(format!(
                "(guest_name LIKE ?{0} OR COALESCE(booking_number, '') LIKE ?{0} OR COALESCE(nationality, '') LIKE ?{0} OR COALESCE(room_number, '') LIKE ?{0} OR COALESCE(remarks, '') LIKE ?{0} OR date LIKE ?{0} OR check_in_date LIKE ?{0} OR check_out_date LIKE ?{0})",
                idx
            ));
            params_vec.push(Box::new(format!("%{}%", clean)));
        }
    }

    let where_str = if where_clauses.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", where_clauses.join(" AND "))
    };

    let count_sql = format!(
        "SELECT COUNT(*), COALESCE(SUM(total_person), 0), COALESCE(SUM(total_rooms), 0), COALESCE(SUM(total_nights), 0), COALESCE(SUM(total_payment), 0.0) FROM room_reservations {}",
        where_str
    );

    let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|b| b.as_ref()).collect();
    let (total_count, total_person, total_rooms, total_nights, total_payment): (i64, i64, i64, i64, f64) = conn
        .query_row(&count_sql, params_refs.as_slice(), |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?))
        })
        .map_err(|e| format!("Gagal menghitung data reservasi: {}", e))?;

    let limit_idx = params_vec.len() + 1;
    let offset_idx = params_vec.len() + 2;
    let items_sql = format!(
        "SELECT id, unit, transaction_id, date, guest_name, total_person, booking_number, nationality, room_number, total_rooms, check_in_date, check_out_date, total_nights, rate, total_payment, remarks, source_file, created_at FROM room_reservations {} ORDER BY date DESC, id DESC LIMIT ?{} OFFSET ?{}",
        where_str, limit_idx, offset_idx
    );

    let mut query_params: Vec<Box<dyn rusqlite::ToSql>> = params_vec;
    query_params.push(Box::new(size as i64));
    query_params.push(Box::new(offset as i64));
    let query_refs: Vec<&dyn rusqlite::ToSql> = query_params.iter().map(|b| b.as_ref()).collect();

    let mut stmt = conn
        .prepare(&items_sql)
        .map_err(|e| format!("Gagal mempersiapkan query reservasi: {}", e))?;
    let items = stmt
        .query_map(query_refs.as_slice(), map_room_reservation_row)
        .map_err(|e| format!("Gagal menjalankan query reservasi: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let total_pages = if total_count == 0 {
        1
    } else {
        ((total_count as usize) + size - 1) / size
    };

    Ok(PaginatedRoomReservations {
        items,
        total_count: total_count as usize,
        total_person,
        total_rooms,
        total_nights,
        total_payment,
        page: p,
        page_size: size,
        total_pages,
    })
}

/// Cleans duplicate transactions having identical unit, date, amount, and notes.
pub fn clean_duplicate_transactions(conn: &Connection, unit: Option<&str>) -> Result<usize, String> {
    let sql = match unit {
        Some(u) if !u.is_empty() && u != "all" => {
            "DELETE FROM transactions WHERE id NOT IN (
                SELECT MIN(id) FROM transactions WHERE unit = ?1 GROUP BY unit, date, amount, COALESCE(notes, '')
            ) AND unit = ?1;"
        }
        _ => {
            "DELETE FROM transactions WHERE id NOT IN (
                SELECT MIN(id) FROM transactions GROUP BY unit, date, amount, COALESCE(notes, '')
            );"
        }
    };

    let affected = match unit {
        Some(u) if !u.is_empty() && u != "all" => conn.execute(sql, params![u]),
        _ => conn.execute(sql, []),
    }
    .map_err(|e| format!("Gagal membersihkan transaksi duplikat: {}", e))?;

    Ok(affected)
}

/// Returns a list of all distinct available months (YYYY-MM) in descending order.
pub fn get_available_months(conn: &Connection, unit: Option<&str>) -> Result<Vec<String>, String> {
    let has_unit = unit.map(|u| !u.is_empty() && u != "all").unwrap_or(false);
    let unit_filter = unit.unwrap_or("");
    let mut stmt = conn
        .prepare(
            "SELECT DISTINCT month FROM (
                SELECT substr(date, 1, 7) AS month FROM transactions WHERE (?1 = 0 OR unit = ?2) AND length(date) >= 7
                UNION
                SELECT substr(date, 1, 7) AS month FROM room_reservations WHERE (?1 = 0 OR unit = ?2) AND length(date) >= 7
                UNION
                SELECT substr(check_in_date, 1, 7) AS month FROM room_reservations WHERE (?1 = 0 OR unit = ?2) AND length(check_in_date) >= 7
                UNION
                SELECT substr(check_out_date, 1, 7) AS month FROM room_reservations WHERE (?1 = 0 OR unit = ?2) AND length(check_out_date) >= 7
            ) WHERE length(month) = 7 ORDER BY month DESC",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![if has_unit { 1_i64 } else { 0_i64 }, unit_filter], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    let mut months = Vec::new();
    for r in rows {
        let m: String = r.map_err(|e| e.to_string())?;
        if !m.is_empty() {
            months.push(m);
        }
    }
    Ok(months)
}

/// Gets all unit records.
pub fn get_units(conn: &Connection) -> Result<Vec<UnitRow>, String> {
    let mut stmt = conn
        .prepare("SELECT id, code, name, tax_rate, address, npwpd, is_active FROM units ORDER BY id ASC")
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |r| {
            Ok(UnitRow {
                id: r.get(0)?,
                code: r.get(1)?,
                name: r.get(2)?,
                tax_rate: r.get(3)?,
                address: r.get(4)?,
                npwpd: r.get(5)?,
                is_active: r.get::<_, i64>(6)? != 0,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut units = Vec::new();
    for r in rows {
        units.push(r.map_err(|e| e.to_string())?);
    }
    Ok(units)
}

/// Adds a new unit.
pub fn add_unit(
    conn: &Connection,
    code: &str,
    name: &str,
    tax_rate: f64,
    address: Option<&str>,
    npwpd: Option<&str>,
) -> Result<i64, String> {
    conn.execute(
        "INSERT INTO units (code, name, tax_rate, address, npwpd, is_active) VALUES (?1, ?2, ?3, ?4, ?5, 1)",
        params![code, name, tax_rate, address, npwpd],
    )
    .map_err(|e| format!("Gagal menambah unit: {}", e))?;
    Ok(conn.last_insert_rowid())
}

/// Updates an existing unit.
pub fn update_unit(
    conn: &Connection,
    id: i64,
    name: &str,
    tax_rate: f64,
    address: Option<&str>,
    npwpd: Option<&str>,
    is_active: bool,
) -> Result<(), String> {
    conn.execute(
        "UPDATE units SET name = ?1, tax_rate = ?2, address = ?3, npwpd = ?4, is_active = ?5 WHERE id = ?6",
        params![name, tax_rate, address, npwpd, if is_active { 1 } else { 0 }, id],
    )
    .map_err(|e| format!("Gagal mengubah unit: {}", e))?;
    Ok(())
}

/// Retrieves all settings as key-value pairs.
pub fn get_settings(conn: &Connection) -> Result<HashMap<String, String>, String> {
    let mut stmt = conn
        .prepare("SELECT key, value FROM settings")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
        .map_err(|e| e.to_string())?;

    let mut map = HashMap::new();
    for r in rows {
        let (k, v) = r.map_err(|e| e.to_string())?;
        map.insert(k, v);
    }
    Ok(map)
}

/// Updates multiple settings keys.
pub fn update_settings(conn: &Connection, settings: &HashMap<String, String>) -> Result<(), String> {
    for (k, v) in settings.iter() {
        conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = ?2",
            params![k, v],
        )
        .map_err(|e| format!("Gagal menyimpan pengaturan {}: {}", k, e))?;
    }
    Ok(())
}

/// Retrieves import history logs.
pub fn get_import_logs(conn: &Connection, unit: Option<&str>) -> Result<Vec<ImportLogRow>, String> {
    let mut logs = Vec::new();
    if let Some(u) = unit {
        if !u.is_empty() && u != "all" {
            let mut stmt = conn
                .prepare("SELECT id, unit, file_name, records_count, total_amount, period, file_hash, imported_at, stored_path FROM import_logs WHERE unit = ?1 ORDER BY id DESC LIMIT 100")
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map(params![u], |r| {
                    Ok(ImportLogRow {
                        id: r.get(0)?,
                        unit: r.get(1)?,
                        file_name: r.get(2)?,
                        records_count: r.get(3)?,
                        total_amount: r.get(4)?,
                        period: r.get(5)?,
                        file_hash: r.get(6)?,
                        imported_at: r.get(7)?,
                        stored_path: r.get(8)?,
                    })
                })
                .map_err(|e| e.to_string())?;
            for r in rows {
                logs.push(r.map_err(|e| e.to_string())?);
            }
            return Ok(logs);
        }
    }

    let mut stmt = conn
        .prepare("SELECT id, unit, file_name, records_count, total_amount, period, file_hash, imported_at, stored_path FROM import_logs ORDER BY id DESC LIMIT 100")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |r| {
            Ok(ImportLogRow {
                id: r.get(0)?,
                unit: r.get(1)?,
                file_name: r.get(2)?,
                records_count: r.get(3)?,
                total_amount: r.get(4)?,
                period: r.get(5)?,
                file_hash: r.get(6)?,
                imported_at: r.get(7)?,
                stored_path: r.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?;
    for r in rows {
        logs.push(r.map_err(|e| e.to_string())?);
    }
    Ok(logs)
}

/// Collects database file size, archive directory size, and count metrics.
pub fn get_storage_info(
    conn: &Connection,
    db_path: &Path,
    archive_dir: &Path,
) -> Result<StorageInfo, String> {
    let db_size_bytes = fs::metadata(db_path).map(|m| m.len()).unwrap_or(0);

    let mut archive_size_bytes: u64 = 0;
    if let Ok(entries) = fs::read_dir(archive_dir) {
        for entry in entries.flatten() {
            if let Ok(m) = entry.metadata() {
                if m.is_file() {
                    archive_size_bytes += m.len();
                }
            }
        }
    }

    let total_transactions: i64 = conn
        .query_row("SELECT COUNT(*) FROM transactions", [], |r| r.get(0))
        .unwrap_or(0);

    let total_units: i64 = conn
        .query_row("SELECT COUNT(*) FROM units", [], |r| r.get(0))
        .unwrap_or(0);

    let total_import_logs: i64 = conn
        .query_row("SELECT COUNT(*) FROM import_logs", [], |r| r.get(0))
        .unwrap_or(0);

    Ok(StorageInfo {
        db_path: db_path.to_string_lossy().to_string(),
        db_size_bytes,
        archive_dir: archive_dir.to_string_lossy().to_string(),
        archive_size_bytes,
        total_transactions: total_transactions as usize,
        total_units: total_units as usize,
        total_import_logs: total_import_logs as usize,
    })
}

/// Backs up the live database safely to target_path using SQLite's online backup API.
pub fn backup_database(conn: &Connection, target_path: &Path) -> Result<String, String> {
    if let Some(parent) = target_path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let mut dst = Connection::open(target_path)
        .map_err(|e| format!("Gagal membuka file tujuan backup: {}", e))?;

    let backup = rusqlite::backup::Backup::new(conn, &mut dst)
        .map_err(|e| format!("Gagal menginisialisasi online backup: {}", e))?;

    backup
        .run_to_completion(100, std::time::Duration::from_millis(50), None)
        .map_err(|e| format!("Proses backup gagal: {}", e))?;

    Ok(target_path.to_string_lossy().to_string())
}

/// Restores the database from a backup file:
/// 1. Verifies the backup is a valid SQLite DB containing 'transactions' table.
/// 2. Creates a safety copy 'laporan_bapenda.sqlite.pre_restore.bak'.
/// 3. Replaces active database file.
pub fn restore_database(db_path: &Path, source_backup_path: &Path) -> Result<String, String> {
    if !source_backup_path.exists() {
        return Err("File cadangan tidak ditemukan.".to_string());
    }

    // 1. Verify backup validity
    {
        let test_conn = Connection::open(source_backup_path)
            .map_err(|_| "File bukan database SQLite yang valid.".to_string())?;
        let count: Result<i64, _> = test_conn.query_row(
            "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='transactions'",
            [],
            |r| r.get(0),
        );
        match count {
            Ok(c) if c > 0 => {}
            _ => {
                return Err(
                    "File cadangan tidak memiliki tabel 'transactions' yang sesuai.".to_string(),
                )
            }
        }
    }

    // 2. Create emergency WAL-safe backup of current DB before closing connection
    if db_path.exists() {
        let bak_path = db_path.with_extension("sqlite.pre_restore.bak");
        let live_conn = open_connection(db_path)
            .map_err(|e| format!("Gagal membuka database aktif untuk cadangan darurat: {}", e))?;
        backup_database(&live_conn, &bak_path)
            .map_err(|e| format!("Gagal membuat salinan darurat (.pre_restore.bak): {}", e))?;
    }

    // 3. Overwrite current DB transactionally via SQLite Online Backup API (WAL-safe)
    let src_conn = Connection::open(source_backup_path)
        .map_err(|e| format!("Gagal membuka file cadangan: {}", e))?;
    let mut dst_conn = open_connection(db_path)
        .map_err(|e| format!("Gagal membuka database tujuan pemulihan: {}", e))?;
    let backup = rusqlite::backup::Backup::new(&src_conn, &mut dst_conn)
        .map_err(|e| format!("Gagal menginisialisasi pemulihan database: {}", e))?;
    backup
        .run_to_completion(100, std::time::Duration::from_millis(50), None)
        .map_err(|e| format!("Proses pemulihan database gagal: {}", e))?;

    Ok(db_path.to_string_lossy().to_string())
}

// ==========================================
// Training Certificates CRUD & Pagination
// ==========================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingCertificateRow {
    pub id: i64,
    pub certificate_no: String,
    pub recipient_name: String,
    pub duration_text: String,
    pub program_name: String,
    pub start_date: String,
    pub end_date: String,
    pub issued_date: String,
    pub department: Option<String>,
    pub institution: Option<String>,
    pub achievement_text: Option<String>,
    pub notes: Option<String>,
    pub signer_name: String,
    pub signer_title: String,
    pub paper_size: String,
    pub status: String,
    pub department_code: Option<String>,
    pub program_type_code: Option<String>,
    pub issued_city: Option<String>,
    pub body_template: Option<String>,
    pub created_by: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingCertificateInput {
    pub certificate_no: Option<String>,
    pub recipient_name: String,
    pub duration_text: String,
    pub program_name: Option<String>,
    pub start_date: String,
    pub end_date: String,
    pub issued_date: String,
    pub department: Option<String>,
    pub institution: Option<String>,
    #[serde(default)]
    pub achievement_text: Option<String>,
    pub notes: Option<String>,
    pub signer_name: Option<String>,
    pub signer_title: Option<String>,
    pub paper_size: Option<String>,
    #[serde(default)]
    pub department_code: Option<String>,
    #[serde(default)]
    pub program_type_code: Option<String>,
    #[serde(default)]
    pub issued_city: Option<String>,
    #[serde(default)]
    pub body_template: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedCertificates {
    pub items: Vec<TrainingCertificateRow>,
    pub total_count: usize,
    pub active_count: usize,
    pub revoked_count: usize,
    pub page: usize,
    pub page_size: usize,
    pub total_pages: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepartmentRow {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub program_description: String,
    pub competencies: String,
    pub is_active: bool,
    pub sort_order: i64,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepartmentInput {
    pub id: Option<i64>,
    pub code: String,
    pub name: String,
    pub program_description: String,
    pub competencies: String,
    pub is_active: Option<bool>,
    pub sort_order: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramTypeRow {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub default_duration_months: i64,
    pub description: String,
    pub is_active: bool,
    pub sort_order: i64,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramTypeInput {
    pub id: Option<i64>,
    pub code: String,
    pub name: String,
    pub default_duration_months: i64,
    pub description: String,
    pub is_active: Option<bool>,
    pub sort_order: Option<i64>,
}

struct ResolvedCertificateFields {
    recipient_name: String,
    duration_text: String,
    program_name: String,
    start_iso: String,
    end_iso: String,
    issued_iso: String,
    issued_year: i32,
    issued_month: u32,
    department: Option<String>,
    department_code: Option<String>,
    program_type_code: Option<String>,
    issued_city: Option<String>,
    body_template: Option<String>,
    institution: Option<String>,
    achievement_text: Option<String>,
    notes: Option<String>,
    signer_name: String,
    signer_title: String,
    paper_size: String,
}

fn sanitize_catalog_code(raw: &str) -> Result<String, String> {
    let cleaned: String = raw
        .trim()
        .to_ascii_uppercase()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .take(12)
        .collect();
    if cleaned.is_empty() {
        return Err("Kode tidak boleh kosong dan hanya boleh berisi huruf/angka/tanda hubung.".to_string());
    }
    Ok(cleaned)
}

fn sanitize_optional_dept_code(code: Option<&str>) -> Option<String> {
    let raw = code?.trim();
    if raw.is_empty() || raw == "-" {
        return None;
    }
    let cleaned: String = raw
        .to_ascii_uppercase()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .take(12)
        .collect();
    if cleaned.is_empty() {
        None
    } else {
        Some(cleaned)
    }
}

fn strip_department_suffix(segment: &str) -> String {
    let without_paren = if segment.ends_with(')') {
        if let Some(open_idx) = segment.rfind('(') {
            segment[..open_idx].trim()
        } else {
            segment.trim()
        }
    } else {
        segment.trim()
    };
    let lower = without_paren.to_ascii_lowercase();
    if let Some(stripped) = lower.strip_suffix(" department") {
        let end = stripped.len();
        without_paren[..end].trim().to_string()
    } else {
        without_paren.to_string()
    }
}

fn resolve_department_code(
    conn: &Connection,
    explicit_code: Option<&str>,
    division_text: Option<&str>,
) -> Option<String> {
    if let Some(code) = sanitize_optional_dept_code(explicit_code) {
        return Some(code);
    }
    let divs: Vec<&str> = division_text?
        .split(';')
        .map(str::trim)
        .filter(|s| !s.is_empty() && *s != "-")
        .collect();
    if divs.is_empty() {
        return None;
    }
    if divs.len() >= 2 {
        return Some("ROT".to_string());
    }
    let first_div = divs[0];
    let base_name = strip_department_suffix(first_div);
    conn.query_row(
        "SELECT code FROM departments
         WHERE LOWER(name) = LOWER(?1) OR LOWER(code) = LOWER(?2)
         LIMIT 1",
        params![base_name, first_div],
        |r| r.get::<_, String>(0),
    )
    .ok()
    .and_then(|c| sanitize_optional_dept_code(Some(&c)))
}

fn parse_duration_to_months(raw: &str) -> Option<f64> {
    let text = raw.trim().to_ascii_lowercase();
    if text.is_empty() {
        return None;
    }
    let mut parts = text.split(|c: char| c.is_whitespace() || c == '-').filter(|s| !s.is_empty());
    let num_tok = parts.next()?;
    let unit_tok = parts
        .next()?
        .trim_end_matches(|c: char| matches!(c, '.' | ',' | ';' | ')'));

    let num = match num_tok {
        "one" => 1.0,
        "two" => 2.0,
        "three" => 3.0,
        "four" => 4.0,
        "five" => 5.0,
        "six" => 6.0,
        "seven" => 7.0,
        "eight" => 8.0,
        "nine" => 9.0,
        "ten" => 10.0,
        "eleven" => 11.0,
        "twelve" => 12.0,
        other => other.replace(',', ".").parse::<f64>().ok()?,
    };
    if !num.is_finite() || num <= 0.0 {
        return None;
    }

    match unit_tok {
        "year" | "years" | "yr" | "yrs" | "tahun" | "thn" => Some(num * 12.0),
        "month" | "months" | "mo" | "mos" | "bulan" | "bln" => Some(num),
        "week" | "weeks" | "wk" | "wks" | "minggu" | "mgg" => Some(num / 4.33),
        "day" | "days" | "hari" => Some(num / 30.0),
        _ => None,
    }
}

fn format_months_summary(val: f64) -> String {
    let rounded = (val * 10.0).round() / 10.0;
    if (rounded - rounded.round()).abs() < 0.05 {
        format!("{} bulan", rounded.round() as i64)
    } else {
        format!("{:.1} bulan", rounded)
    }
}

fn validate_division_durations(
    department: Option<&str>,
    start_d: chrono::NaiveDate,
    end_d: chrono::NaiveDate,
    duration_text: &str,
) -> Result<(), String> {
    let Some(raw_dept) = department else {
        return Ok(());
    };
    let mut sum_months = 0.0_f64;
    let mut parsed_count = 0usize;

    for seg in raw_dept.split(';').map(str::trim).filter(|s| !s.is_empty() && *s != "-") {
        if let Some(without_close) = seg.strip_suffix(')') {
            let mut depth = 1_usize;
            let mut open_idx = None;
            for (idx, ch) in without_close.char_indices().rev() {
                if ch == ')' {
                    depth += 1;
                } else if ch == '(' {
                    depth -= 1;
                    if depth == 0 {
                        open_idx = Some(idx);
                        break;
                    }
                }
            }
            if let Some(open_pos) = open_idx.or_else(|| without_close.rfind('(')) {
                let inner = without_close[open_pos + 1..].trim();
                if let Some(months) = parse_duration_to_months(inner) {
                    sum_months += months;
                    parsed_count += 1;
                }
            }
        }
    }
    if parsed_count == 0 {
        return Ok(());
    }

    let period_months = parse_duration_to_months(duration_text)
        .unwrap_or_else(|| ((end_d - start_d).num_days().max(1) as f64) / 30.4375);

    if sum_months > period_months + 0.35 {
        return Err(format!(
            "Total durasi divisi ({}) melebihi periode utama ({}). Sesuaikan durasi per divisi atau rentang tanggal pelatihan.",
            format_months_summary(sum_months),
            format_months_summary(period_months)
        ));
    }
    Ok(())
}

fn resolve_certificate_fields(
    conn: &Connection,
    input: &TrainingCertificateInput,
) -> Result<ResolvedCertificateFields, String> {
    let raw_recipient = input.recipient_name.trim();
    if raw_recipient.is_empty() {
        return Err("Nama penerima sertifikat wajib diisi.".to_string());
    }
    let recipient_name = crate::certificate::normalize_recipient_name(raw_recipient);
    let duration_text = input.duration_text.trim();
    if duration_text.is_empty() {
        return Err("Durasi pelatihan wajib diisi.".to_string());
    }

    let (start_d, start_iso) = normalize_iso_date(&input.start_date, "tanggal mulai")?;
    let (end_d, end_iso) = normalize_iso_date(&input.end_date, "tanggal selesai")?;
    if end_d < start_d {
        return Err("Tanggal selesai pelatihan tidak boleh lebih awal dari tanggal mulai.".to_string());
    }
    let (issued_d, issued_iso) = normalize_iso_date(&input.issued_date, "tanggal terbit")?;

    let settings = get_settings(conn)?;
    let setting_or = |key: &str, fallback: &str| -> String {
        settings
            .get(key)
            .map(|v| v.trim())
            .filter(|v| !v.is_empty())
            .unwrap_or(fallback)
            .to_string()
    };
    let clean_or_default = |opt: &Option<String>, fallback: &str| -> String {
        opt.as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or(fallback)
            .to_string()
    };

    let default_program = setting_or("certProgramName", "hospitality training program");
    let default_signer = setting_or("certSignerName", "Agus Sumardi Wijaya");
    let default_title = setting_or("certSignerTitle", "General Manager");
    let default_paper = setting_or("certPaperSize", "F4");
    let default_city = setting_or("certIssuedCity", crate::certificate::DEFAULT_ISSUED_CITY);
    let default_template = setting_or(
        "certBodyTemplate",
        crate::certificate::DEFAULT_BODY_TEMPLATE,
    );

    let department = clean_optional_field(&input.department);
    validate_division_durations(department.as_deref(), start_d, end_d, duration_text)?;
    let department_code = resolve_department_code(
        conn,
        input.department_code.as_deref(),
        department.as_deref(),
    );
    let program_type_code = sanitize_optional_dept_code(input.program_type_code.as_deref());
    let issued_city = clean_optional_field(&input.issued_city).or(Some(default_city));
    let body_template = clean_optional_field(&input.body_template).or(Some(default_template));

    Ok(ResolvedCertificateFields {
        recipient_name,
        duration_text: duration_text.to_string(),
        program_name: clean_or_default(&input.program_name, &default_program),
        start_iso,
        end_iso,
        issued_iso,
        issued_year: chrono::Datelike::year(&issued_d),
        issued_month: chrono::Datelike::month(&issued_d),
        department,
        department_code,
        program_type_code,
        issued_city,
        body_template,
        institution: clean_optional_field(&input.institution),
        achievement_text: clean_optional_field(&input.achievement_text),
        notes: clean_optional_field(&input.notes),
        signer_name: clean_or_default(&input.signer_name, &default_signer),
        signer_title: clean_or_default(&input.signer_title, &default_title),
        paper_size: normalize_paper_size(input.paper_size.as_deref(), &default_paper)?,
    })
}

fn normalize_paper_size(raw: Option<&str>, fallback: &str) -> Result<String, String> {
    match raw.map(str::trim).filter(|s| !s.is_empty()) {
        None => Ok(fallback.to_string()),
        Some(s) if s.eq_ignore_ascii_case("F4") => Ok("F4".to_string()),
        Some(s) if s.eq_ignore_ascii_case("A4") => Ok("A4".to_string()),
        Some(other) => Err(format!(
            "Ukuran kertas '{}' tidak didukung (gunakan F4 atau A4).",
            other
        )),
    }
}

fn map_cert_db_error(err: rusqlite::Error, cert_no: &str, op_label: &str) -> String {
    if let rusqlite::Error::SqliteFailure(ffi_err, _) = &err {
        if ffi_err.extended_code == 2067 || ffi_err.code == rusqlite::ErrorCode::ConstraintViolation {
            return format!(
                "Nomor sertifikat '{}' sudah terdaftar. Gunakan nomor lain.",
                cert_no
            );
        }
    }
    let msg = err.to_string();
    if msg.contains("UNIQUE constraint failed") {
        format!(
            "Nomor sertifikat '{}' sudah terdaftar. Gunakan nomor lain.",
            cert_no
        )
    } else {
        format!("Gagal {}: {}", op_label, msg)
    }
}

fn map_certificate_row(r: &rusqlite::Row) -> rusqlite::Result<TrainingCertificateRow> {
    Ok(TrainingCertificateRow {
        id: r.get(0)?,
        certificate_no: r.get(1)?,
        recipient_name: r.get(2)?,
        duration_text: r.get(3)?,
        program_name: r.get(4)?,
        start_date: r.get(5)?,
        end_date: r.get(6)?,
        issued_date: r.get(7)?,
        department: r.get(8)?,
        institution: r.get(9)?,
        achievement_text: r.get(10)?,
        notes: r.get(11)?,
        signer_name: r.get(12)?,
        signer_title: r.get(13)?,
        paper_size: r.get(14)?,
        status: r.get(15)?,
        department_code: r.get(16)?,
        program_type_code: r.get(17)?,
        issued_city: r.get(18)?,
        body_template: r.get(19)?,
        created_by: r.get(20)?,
        created_at: r.get(21)?,
        updated_at: r.get(22)?,
    })
}

/// Converts a calendar month (1..=12) into uppercase Roman numerals for certificate numbering.
pub fn month_to_roman(month: u32) -> &'static str {
    match month {
        1 => "I",
        2 => "II",
        3 => "III",
        4 => "IV",
        5 => "V",
        6 => "VI",
        7 => "VII",
        8 => "VIII",
        9 => "IX",
        10 => "X",
        11 => "XI",
        12 => "XII",
        _ => "I",
    }
}

fn format_certificate_no(seq: u32, dept_code: Option<&str>, month: u32, year: i32) -> String {
    let roman = month_to_roman(month);
    match sanitize_optional_dept_code(dept_code) {
        Some(code) => format!("{:03}/ABR-{}/{}/{}", seq, code, roman, year),
        None => format!("{:03}/ABR/{}/{}", seq, roman, year),
    }
}

fn parse_seq_and_year_from_cert_no(cert_no: &str) -> Option<(u32, i32)> {
    let parts: Vec<&str> = cert_no.split('/').map(str::trim).collect();
    if parts.len() < 3 {
        return None;
    }
    let first = parts.first()?.parse::<u32>().ok()?;
    if first == 0 || first > 999_999 {
        return None;
    }
    let last = parts.last()?.parse::<i32>().ok()?;
    Some((first, last))
}

fn compute_next_sequence(conn: &Connection, year: i32) -> Result<u32, String> {
    let seq_table_val: u32 = conn
        .query_row(
            "SELECT last_seq FROM certificate_sequences WHERE year = ?1",
            params![year],
            |r| r.get::<_, i64>(0),
        )
        .ok()
        .and_then(|v| u32::try_from(v).ok())
        .unwrap_or(0);

    let pattern = format!("%/{}", year);
    let mut stmt = conn
        .prepare("SELECT certificate_no FROM training_certificates WHERE certificate_no LIKE ?1")
        .map_err(|e| format!("Gagal membaca urutan nomor sertifikat: {}", e))?;
    let rows = stmt
        .query_map(params![pattern], |r| r.get::<_, String>(0))
        .map_err(|e| format!("Gagal memindai nomor sertifikat: {}", e))?;

    let mut max_seq = seq_table_val;
    for res in rows {
        let cert_no = res.map_err(|e| e.to_string())?;
        if let Some((num, parsed_year)) = parse_seq_and_year_from_cert_no(&cert_no) {
            if parsed_year == year && num > max_seq {
                max_seq = num;
            }
        }
    }
    max_seq
        .checked_add(1)
        .ok_or_else(|| "Urutan nomor sertifikat melampaui batas maksimum.".to_string())
}

fn upsert_sequence_high_water(conn: &Connection, year: i32, seq: u32) -> Result<(), String> {
    conn.execute(
        "INSERT INTO certificate_sequences (year, last_seq) VALUES (?1, ?2)
         ON CONFLICT(year) DO UPDATE SET last_seq = MAX(certificate_sequences.last_seq, excluded.last_seq)",
        params![year, i64::from(seq)],
    )
    .map_err(|e| format!("Gagal memperbarui urutan sertifikat: {}", e))?;
    Ok(())
}

/// Computes the next available `NNN/ABR-{DEPT}/{ROMAN}/{YYYY}` certificate number for preview.
pub fn get_next_certificate_number(
    conn: &Connection,
    year: i32,
    month: u32,
    dept_code: Option<&str>,
) -> Result<String, String> {
    let next_seq = compute_next_sequence(conn, year)?;
    Ok(format_certificate_no(next_seq, dept_code, month, year))
}

/// Fetches a single training certificate record by primary key ID.
pub fn get_certificate_by_id(conn: &Connection, id: i64) -> Result<TrainingCertificateRow, String> {
    conn.query_row(
        "SELECT id, certificate_no, recipient_name, duration_text, program_name,
                start_date, end_date, issued_date, department, institution, achievement_text, notes,
                signer_name, signer_title, paper_size, status,
                department_code, program_type_code, issued_city, body_template,
                created_by, created_at, updated_at
         FROM training_certificates WHERE id = ?1",
        params![id],
        map_certificate_row,
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => "Data sertifikat tidak ditemukan.".to_string(),
        other => format!("Gagal membaca data sertifikat: {}", other),
    })
}

/// Validates input fields and creates a new training certificate record in SQLite atomically.
pub fn create_certificate(
    conn: &Connection,
    input: &TrainingCertificateInput,
    created_by: Option<&str>,
) -> Result<TrainingCertificateRow, String> {
    let tx = conn
        .unchecked_transaction()
        .map_err(|e| format!("Gagal memulai transaksi sertifikat: {}", e))?;

    let fields = resolve_certificate_fields(&tx, input)?;
    let certificate_no = match input
        .certificate_no
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        Some(manual) => {
            if let Some((seq, yr)) = parse_seq_and_year_from_cert_no(manual) {
                if yr >= 1900 && yr <= 9999 {
                    upsert_sequence_high_water(&tx, yr, seq)?;
                }
            }
            manual.to_string()
        }
        None => {
            let next_seq = compute_next_sequence(&tx, fields.issued_year)?;
            upsert_sequence_high_water(&tx, fields.issued_year, next_seq)?;
            format_certificate_no(
                next_seq,
                fields.department_code.as_deref(),
                fields.issued_month,
                fields.issued_year,
            )
        }
    };
    let creator = created_by.map(str::trim).filter(|s| !s.is_empty());

    tx.execute(
        "INSERT INTO training_certificates
         (certificate_no, recipient_name, duration_text, program_name, start_date, end_date,
          issued_date, department, institution, achievement_text, notes, signer_name, signer_title,
          paper_size, status, department_code, program_type_code, issued_city, body_template, created_by)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, 'active', ?15, ?16, ?17, ?18, ?19)",
        params![
            certificate_no,
            fields.recipient_name,
            fields.duration_text,
            fields.program_name,
            fields.start_iso,
            fields.end_iso,
            fields.issued_iso,
            fields.department,
            fields.institution,
            fields.achievement_text,
            fields.notes,
            fields.signer_name,
            fields.signer_title,
            fields.paper_size,
            fields.department_code,
            fields.program_type_code,
            fields.issued_city,
            fields.body_template,
            creator,
        ],
    )
    .map_err(|e| map_cert_db_error(e, &certificate_no, "menyimpan sertifikat"))?;

    let inserted_id = tx.last_insert_rowid();
    tx.commit()
        .map_err(|e| format!("Gagal mengesahkan transaksi sertifikat: {}", e))?;

    get_certificate_by_id(conn, inserted_id)
}

/// Validates input fields and updates an existing training certificate record.
/// Certificate numbers are permanently locked after issuance and cannot be modified on update.
pub fn update_certificate(
    conn: &Connection,
    id: i64,
    input: &TrainingCertificateInput,
) -> Result<TrainingCertificateRow, String> {
    let existing_no: String = conn
        .query_row(
            "SELECT certificate_no FROM training_certificates WHERE id = ?1",
            params![id],
            |r| r.get(0),
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => "Data sertifikat tidak ditemukan.".to_string(),
            other => format!("Gagal membaca data sertifikat: {}", other),
        })?;

    if let Some(raw_no) = input.certificate_no.as_deref() {
        let req_no = raw_no.trim();
        if req_no.is_empty() {
            return Err(
                "Nomor sertifikat sudah terkunci setelah diterbitkan dan tidak dapat dikosongkan."
                    .to_string(),
            );
        }
        if !existing_no.eq_ignore_ascii_case(req_no) {
            return Err(format!(
                "Nomor sertifikat '{}' sudah terkunci setelah diterbitkan dan tidak dapat diubah.",
                existing_no
            ));
        }
    }

    let fields = resolve_certificate_fields(conn, input)?;

    let affected = conn
        .execute(
            "UPDATE training_certificates
             SET recipient_name = ?1,
                 duration_text = ?2,
                 program_name = ?3,
                 start_date = ?4,
                 end_date = ?5,
                 issued_date = ?6,
                 department = ?7,
                 institution = ?8,
                 achievement_text = ?9,
                 notes = ?10,
                 signer_name = ?11,
                 signer_title = ?12,
                 paper_size = ?13,
                 department_code = ?14,
                 program_type_code = ?15,
                 issued_city = ?16,
                 body_template = ?17,
                 updated_at = datetime('now', 'localtime')
             WHERE id = ?18",
            params![
                fields.recipient_name,
                fields.duration_text,
                fields.program_name,
                fields.start_iso,
                fields.end_iso,
                fields.issued_iso,
                fields.department,
                fields.institution,
                fields.achievement_text,
                fields.notes,
                fields.signer_name,
                fields.signer_title,
                fields.paper_size,
                fields.department_code,
                fields.program_type_code,
                fields.issued_city,
                fields.body_template,
                id,
            ],
        )
        .map_err(|e| map_cert_db_error(e, &existing_no, "memperbarui sertifikat"))?;

    if affected == 0 {
        return Err("Data sertifikat tidak ditemukan.".to_string());
    }
    get_certificate_by_id(conn, id)
}

/// Sets a training certificate's lifecycle status (`"active"` or `"revoked"`).
pub fn set_certificate_status(
    conn: &Connection,
    id: i64,
    status: &str,
) -> Result<TrainingCertificateRow, String> {
    let clean_status = status.trim();
    if clean_status != "active" && clean_status != "revoked" {
        return Err("Status sertifikat tidak valid (gunakan 'active' atau 'revoked').".to_string());
    }
    let affected = conn
        .execute(
            "UPDATE training_certificates SET status = ?1, updated_at = datetime('now', 'localtime') WHERE id = ?2",
            params![clean_status, id],
        )
        .map_err(|e| format!("Gagal mengubah status sertifikat: {}", e))?;
    if affected == 0 {
        return Err("Data sertifikat tidak ditemukan.".to_string());
    }
    get_certificate_by_id(conn, id)
}

/// Permanently removes a training certificate record from SQLite.
pub fn delete_certificate(conn: &Connection, id: i64) -> Result<(), String> {
    let affected = conn
        .execute("DELETE FROM training_certificates WHERE id = ?1", params![id])
        .map_err(|e| format!("Gagal menghapus sertifikat: {}", e))?;
    if affected == 0 {
        return Err("Data sertifikat tidak ditemukan.".to_string());
    }
    Ok(())
}

/// Queries training certificates with SQL year, status, search filters and pagination.
pub fn get_paginated_certificates(
    conn: &Connection,
    year: Option<&str>,
    status: Option<&str>,
    search: Option<&str>,
    page: usize,
    page_size: usize,
) -> Result<PaginatedCertificates, String> {
    let requested_page = if page == 0 { 1 } else { page };
    let size = if page_size == 0 { 25 } else { page_size.clamp(1, 100) };

    let mut where_clauses: Vec<String> = Vec::new();
    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(y) = year.map(str::trim) {
        if !y.is_empty() && y != "all" {
            where_clauses.push(format!("substr(issued_date, 1, 4) = ?{}", params_vec.len() + 1));
            params_vec.push(Box::new(y.to_string()));
        }
    }

    if let Some(st) = status.map(str::trim) {
        if !st.is_empty() && st != "all" {
            where_clauses.push(format!("status = ?{}", params_vec.len() + 1));
            params_vec.push(Box::new(st.to_string()));
        }
    }

    if let Some(s) = search.map(str::trim) {
        if !s.is_empty() {
            let idx = params_vec.len() + 1;
            where_clauses.push(format!(
                "(certificate_no LIKE ?{0} OR recipient_name LIKE ?{0} OR COALESCE(department, '') LIKE ?{0} OR COALESCE(institution, '') LIKE ?{0} OR COALESCE(achievement_text, '') LIKE ?{0} OR program_name LIKE ?{0} OR COALESCE(notes, '') LIKE ?{0})",
                idx
            ));
            params_vec.push(Box::new(format!("%{}%", s)));
        }
    }

    let where_str = if where_clauses.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", where_clauses.join(" AND "))
    };

    let count_sql = format!(
        "SELECT COUNT(*),
                COALESCE(SUM(CASE WHEN status = 'active' THEN 1 ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN status = 'revoked' THEN 1 ELSE 0 END), 0)
         FROM training_certificates {}",
        where_str
    );

    let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|b| b.as_ref()).collect();
    let (total_count, active_count, revoked_count): (i64, i64, i64) = conn
        .query_row(&count_sql, params_refs.as_slice(), |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })
        .map_err(|e| format!("Gagal menghitung riwayat sertifikat: {}", e))?;

    let total_pages = if total_count == 0 {
        1
    } else {
        ((total_count as usize) + size - 1) / size
    };
    let p = requested_page.min(total_pages);
    let offset = (p - 1) * size;

    let limit_idx = params_vec.len() + 1;
    let offset_idx = params_vec.len() + 2;
    let items_sql = format!(
        "SELECT id, certificate_no, recipient_name, duration_text, program_name,
                start_date, end_date, issued_date, department, institution, achievement_text, notes,
                signer_name, signer_title, paper_size, status,
                department_code, program_type_code, issued_city, body_template,
                created_by, created_at, updated_at
         FROM training_certificates {}
         ORDER BY issued_date DESC, id DESC
         LIMIT ?{} OFFSET ?{}",
        where_str, limit_idx, offset_idx
    );

    let mut query_params: Vec<Box<dyn rusqlite::ToSql>> = params_vec;
    query_params.push(Box::new(size as i64));
    query_params.push(Box::new(offset as i64));
    let query_refs: Vec<&dyn rusqlite::ToSql> = query_params.iter().map(|b| b.as_ref()).collect();

    let mut stmt = conn
        .prepare(&items_sql)
        .map_err(|e| format!("Gagal mempersiapkan query sertifikat: {}", e))?;
    let items = stmt
        .query_map(query_refs.as_slice(), map_certificate_row)
        .map_err(|e| format!("Gagal membaca daftar sertifikat: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(PaginatedCertificates {
        items,
        total_count: total_count as usize,
        active_count: active_count as usize,
        revoked_count: revoked_count as usize,
        page: p,
        page_size: size,
        total_pages,
    })
}

/// Lists all distinct issuance years (`YYYY`) present in `training_certificates` descending.
pub fn get_certificate_years(conn: &Connection) -> Result<Vec<String>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT DISTINCT substr(issued_date, 1, 4) AS yr
             FROM training_certificates
             WHERE length(issued_date) >= 4
             ORDER BY yr DESC",
        )
        .map_err(|e| format!("Gagal mengambil daftar tahun sertifikat: {}", e))?;
    let rows = stmt
        .query_map([], |r| r.get::<_, String>(0))
        .map_err(|e| e.to_string())?;
    let mut years = Vec::new();
    for r in rows {
        let y = r.map_err(|e| e.to_string())?;
        if !y.is_empty() {
            years.push(y);
        }
    }
    Ok(years)
}

fn seed_default_catalog(conn: &Connection) -> Result<(), String> {
    let canonical_departments: [(&str, &str, &str, &str, &str, &[&str], i64); 10] = [
        (
            "KIT",
            "Kitchen",
            "The program covered food preparation, kitchen hygiene, and menu production.",
            "Basic cooking techniques\nFood safety (HACCP)\nIngredient handling\nFood plating",
            "Training in food preparation, kitchen hygiene, and menu production",
            &[
                "Teknik memasak dasar\nFood safety (HACCP)\nPenanganan bahan\nPlating",
                "Basic cooking techniques\nFood safety (HACCP)\nIngredient handling\nPlating and presentation",
            ],
            10_i64,
        ),
        (
            "FB",
            "Food & Beverage Service",
            "The program covered restaurant service, guest handling, and table setup.",
            "Table service\nMenu knowledge\nGuest complaint handling\nUpselling techniques",
            "Training in restaurant service, guest handling, and table setup",
            &["Table service\nPengetahuan menu\nPenanganan keluhan\nUpselling"],
            20,
        ),
        (
            "BAR",
            "Bar",
            "The program covered beverage preparation and bar operations.",
            "Beverage preparation\nProduct knowledge\nBar inventory control",
            "Training in beverage preparation and bar operations",
            &["Pembuatan minuman\nPengetahuan produk\nKontrol stok bar"],
            30,
        ),
        (
            "FO",
            "Front Office",
            "The program covered guest reception, reservations, and check-in/check-out procedures.",
            "Guest registration\nReservation systems\nGuest communication\nPayment handling",
            "Training in guest reception, reservations, and check-in/check-out procedures",
            &["Registrasi tamu\nSistem reservasi\nKomunikasi tamu\nPenanganan pembayaran"],
            40,
        ),
        (
            "HK",
            "Housekeeping",
            "The program covered room preparation, cleanliness standards, and laundry operations.",
            "Room cleanliness standards\nBed making\nLinen handling\nPublic area maintenance",
            "Training in room preparation, cleanliness standards, and laundry operations",
            &["Standar kebersihan kamar\nBed making\nPenanganan linen\nArea publik"],
            50,
        ),
        (
            "MNT",
            "Maintenance / Engineering",
            "The program covered facility upkeep and basic technical repairs.",
            "Electrical maintenance\nPlumbing repairs\nAir conditioning maintenance\nPool & garden upkeep",
            "Training in facility upkeep and basic technical repairs",
            &[
                "Perawatan listrik\nPlumbing\nPerawatan AC\nTaman dan kolam",
                "Electrical maintenance\nPlumbing repairs\nAir conditioning maintenance\nGarden and pool upkeep",
            ],
            60,
        ),
        (
            "SM",
            "Sales & Marketing",
            "The program covered promotional campaigns, online booking channels, and guest engagement.",
            "Social media management\nOTA channel management (Booking.com, Agoda)\nGuest inquiry handling",
            "Training in promotion, online booking channels, and guest engagement",
            &["Media sosial\nOTA (Booking.com, Agoda)\nPenanganan inquiry"],
            70,
        ),
        (
            "ADM",
            "Administration / Accounting",
            "The program covered hotel administration, cashiering, and basic bookkeeping.",
            "Transaction recording\nInvoicing\nDaily financial reporting",
            "Training in hotel administration, cashiering, and basic bookkeeping",
            &["Pencatatan transaksi\nInvoice\nLaporan harian"],
            80,
        ),
        (
            "SEC",
            "Security",
            "The program covered guest safety, emergency response, and property security procedures.",
            "Property patrols\nEmergency procedures\nAccess control",
            "Training in guest and property safety procedures",
            &["Patroli\nProsedur darurat\nKontrol akses"],
            90,
        ),
        (
            "SPA",
            "Spa & Recreation",
            "The program covered spa treatments and guest activity coordination.",
            "Spa treatment techniques\nEquipment sanitation\nGuest activity coordination",
            "Training in spa treatments and guest activity coordination",
            &["Teknik perawatan\nKebersihan alat\nKoordinasi aktivitas"],
            100,
        ),
    ];

    let dept_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM departments", [], |r| r.get(0))
        .unwrap_or(0);
    if dept_count == 0 {
        for (code, name, desc, comp, _, _, sort_order) in canonical_departments {
            let _ = conn.execute(
                "INSERT INTO departments (code, name, program_description, competencies, is_active, sort_order)
                 VALUES (?1, ?2, ?3, ?4, 1, ?5)",
                params![code, name, desc, comp, sort_order],
            );
        }
    } else {
        // Upgrade only unmodified earlier seed copies while leaving custom user edits intact
        for (code, _, desc, comp, legacy_desc, legacy_comps, _) in canonical_departments {
            let _ = conn.execute(
                "UPDATE departments SET program_description = ?1
                 WHERE UPPER(code) = ?2 AND program_description = ?3",
                params![desc, code, legacy_desc],
            );
            for legacy_comp in legacy_comps {
                let _ = conn.execute(
                    "UPDATE departments SET competencies = ?1
                     WHERE UPPER(code) = ?2 AND competencies = ?3",
                    params![comp, code, legacy_comp],
                );
            }
        }
    }

    let pt_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM program_types", [], |r| r.get(0))
        .unwrap_or(0);
    if pt_count == 0 {
        let default_program_types = [
            (
                "OJT",
                "on-the-job training program",
                6_i64,
                "Belajar langsung sambil bekerja di bawah supervisor (siswa SMK/mahasiswa pariwisata, karyawan baru)",
                10_i64,
            ),
            (
                "INT",
                "internship program",
                3,
                "Magang terstruktur, sering bagian dari kurikulum kampus (mahasiswa, 3-6 bulan)",
                20,
            ),
            (
                "APP",
                "apprenticeship program",
                6,
                "Pelatihan jangka panjang dengan jenjang keterampilan (calon staf tetap)",
                30,
            ),
            (
                "MT",
                "management trainee program",
                6,
                "Rotasi antar departemen untuk calon pemimpin (lulusan D3/S1)",
                40,
            ),
            (
                "WS",
                "short course / workshop",
                1,
                "Pelatihan singkat topik tertentu 1-5 hari (staf yang sudah ada)",
                50,
            ),
            (
                "REF",
                "refresher / skill upgrade program",
                1,
                "Penyegaran keterampilan atau standar baru (staf lama)",
                60,
            ),
            (
                "CERT",
                "certification course",
                1,
                "Pelatihan dengan ujian (food safety, first aid, fire safety)",
                70,
            ),
        ];
        for (code, name, months, desc, sort_order) in default_program_types {
            let _ = conn.execute(
                "INSERT INTO program_types (code, name, default_duration_months, description, is_active, sort_order)
                 VALUES (?1, ?2, ?3, ?4, 1, ?5)",
                params![code, name, months, desc, sort_order],
            );
        }
    }
    Ok(())
}

fn map_department_row(r: &rusqlite::Row) -> rusqlite::Result<DepartmentRow> {
    let active_flag: i64 = r.get(5)?;
    Ok(DepartmentRow {
        id: r.get(0)?,
        code: r.get(1)?,
        name: r.get(2)?,
        program_description: r.get(3)?,
        competencies: r.get(4)?,
        is_active: active_flag != 0,
        sort_order: r.get(6)?,
        created_at: r.get(7)?,
    })
}

fn map_program_type_row(r: &rusqlite::Row) -> rusqlite::Result<ProgramTypeRow> {
    let active_flag: i64 = r.get(5)?;
    Ok(ProgramTypeRow {
        id: r.get(0)?,
        code: r.get(1)?,
        name: r.get(2)?,
        default_duration_months: r.get(3)?,
        description: r.get(4)?,
        is_active: active_flag != 0,
        sort_order: r.get(6)?,
        created_at: r.get(7)?,
    })
}

/// Lists all training departments ordered by `sort_order`, then name.
pub fn get_departments(conn: &Connection) -> Result<Vec<DepartmentRow>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, code, name, program_description, competencies, is_active, sort_order, created_at
             FROM departments
             ORDER BY sort_order ASC, name ASC",
        )
        .map_err(|e| format!("Gagal membaca daftar departemen: {}", e))?;
    let items = stmt
        .query_map([], map_department_row)
        .map_err(|e| format!("Gagal memindai departemen: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(items)
}

/// Creates or updates a training department entry in SQLite.
pub fn save_department(conn: &Connection, input: &DepartmentInput) -> Result<DepartmentRow, String> {
    let code = sanitize_catalog_code(&input.code)?;
    let name = input.name.trim();
    if name.is_empty() {
        return Err("Nama departemen wajib diisi.".to_string());
    }
    let desc = input.program_description.trim();
    let comp = input.competencies.trim();

    let target_id = if let Some(id) = input.id {
        let active_opt = input.is_active.map(|b| if b { 1_i64 } else { 0 });
        let affected = conn
            .execute(
                "UPDATE departments
                 SET code = ?1, name = ?2, program_description = ?3, competencies = ?4,
                     is_active = COALESCE(?5, is_active), sort_order = COALESCE(?6, sort_order)
                 WHERE id = ?7",
                params![code, name, desc, comp, active_opt, input.sort_order, id],
            )
            .map_err(|e| {
                if e.to_string().contains("UNIQUE") {
                    format!("Kode departemen '{}' sudah digunakan.", code)
                } else {
                    format!("Gagal memperbarui departemen: {}", e)
                }
            })?;
        if affected == 0 {
            return Err("Departemen tidak ditemukan.".to_string());
        }
        id
    } else {
        let active_val: i64 = if input.is_active.unwrap_or(true) { 1 } else { 0 };
        let sort_order = input.sort_order.unwrap_or_else(|| {
            conn.query_row("SELECT COALESCE(MAX(sort_order), 0) + 10 FROM departments", [], |r| r.get(0))
                .unwrap_or(10)
        });
        conn.execute(
            "INSERT INTO departments (code, name, program_description, competencies, is_active, sort_order)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![code, name, desc, comp, active_val, sort_order],
        )
        .map_err(|e| {
            if e.to_string().contains("UNIQUE") {
                format!("Kode departemen '{}' sudah digunakan.", code)
            } else {
                format!("Gagal menambah departemen: {}", e)
            }
        })?;
        conn.last_insert_rowid()
    };

    conn.query_row(
        "SELECT id, code, name, program_description, competencies, is_active, sort_order, created_at
         FROM departments WHERE id = ?1",
        params![target_id],
        map_department_row,
    )
    .map_err(|e| format!("Gagal memuat ulang departemen: {}", e))
}

/// Permanently deletes a department catalog entry by ID.
pub fn delete_department(conn: &Connection, id: i64) -> Result<(), String> {
    let affected = conn
        .execute("DELETE FROM departments WHERE id = ?1", params![id])
        .map_err(|e| format!("Gagal menghapus departemen: {}", e))?;
    if affected == 0 {
        return Err("Departemen tidak ditemukan.".to_string());
    }
    Ok(())
}

/// Lists all certificate program types ordered by `sort_order`, then name.
pub fn get_program_types(conn: &Connection) -> Result<Vec<ProgramTypeRow>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, code, name, default_duration_months, description, is_active, sort_order, created_at
             FROM program_types
             ORDER BY sort_order ASC, name ASC",
        )
        .map_err(|e| format!("Gagal membaca daftar jenis program: {}", e))?;
    let items = stmt
        .query_map([], map_program_type_row)
        .map_err(|e| format!("Gagal memindai jenis program: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(items)
}

/// Creates or updates a certificate program type entry in SQLite.
pub fn save_program_type(
    conn: &Connection,
    input: &ProgramTypeInput,
) -> Result<ProgramTypeRow, String> {
    let code = sanitize_catalog_code(&input.code)?;
    let name = input.name.trim();
    if name.is_empty() {
        return Err("Nama jenis program wajib diisi.".to_string());
    }
    let months = input.default_duration_months.clamp(1, 36);
    let desc = input.description.trim();

    let target_id = if let Some(id) = input.id {
        let active_opt = input.is_active.map(|b| if b { 1_i64 } else { 0 });
        let affected = conn
            .execute(
                "UPDATE program_types
                 SET code = ?1, name = ?2, default_duration_months = ?3, description = ?4,
                     is_active = COALESCE(?5, is_active), sort_order = COALESCE(?6, sort_order)
                 WHERE id = ?7",
                params![code, name, months, desc, active_opt, input.sort_order, id],
            )
            .map_err(|e| {
                if e.to_string().contains("UNIQUE") {
                    format!("Kode jenis program '{}' sudah digunakan.", code)
                } else {
                    format!("Gagal memperbarui jenis program: {}", e)
                }
            })?;
        if affected == 0 {
            return Err("Jenis program tidak ditemukan.".to_string());
        }
        id
    } else {
        let active_val: i64 = if input.is_active.unwrap_or(true) { 1 } else { 0 };
        let sort_order = input.sort_order.unwrap_or_else(|| {
            conn.query_row("SELECT COALESCE(MAX(sort_order), 0) + 10 FROM program_types", [], |r| r.get(0))
                .unwrap_or(10)
        });
        conn.execute(
            "INSERT INTO program_types (code, name, default_duration_months, description, is_active, sort_order)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![code, name, months, desc, active_val, sort_order],
        )
        .map_err(|e| {
            if e.to_string().contains("UNIQUE") {
                format!("Kode jenis program '{}' sudah digunakan.", code)
            } else {
                format!("Gagal menambah jenis program: {}", e)
            }
        })?;
        conn.last_insert_rowid()
    };

    conn.query_row(
        "SELECT id, code, name, default_duration_months, description, is_active, sort_order, created_at
         FROM program_types WHERE id = ?1",
        params![target_id],
        map_program_type_row,
    )
    .map_err(|e| format!("Gagal memuat ulang jenis program: {}", e))
}

/// Permanently deletes a program type catalog entry by ID.
pub fn delete_program_type(conn: &Connection, id: i64) -> Result<(), String> {
    let affected = conn
        .execute("DELETE FROM program_types WHERE id = ?1", params![id])
        .map_err(|e| format!("Gagal menghapus jenis program: {}", e))?;
    if affected == 0 {
        return Err("Jenis program tidak ditemukan.".to_string());
    }
    Ok(())
}
