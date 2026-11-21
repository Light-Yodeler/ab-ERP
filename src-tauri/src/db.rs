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
        CREATE INDEX IF NOT EXISTS idx_room_res_trans_id ON room_reservations(transaction_id);",
    )
    .map_err(|e| format!("Gagal membuat skema database: {}", e))?;
    // Auto-migration for import_logs
    let _ = conn.execute("ALTER TABLE import_logs ADD COLUMN period TEXT", []);
    let _ = conn.execute("ALTER TABLE import_logs ADD COLUMN file_hash TEXT", []);
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
