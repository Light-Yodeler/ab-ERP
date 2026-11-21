use calamine::{open_workbook_auto, Data, Reader};
use chrono::{Datelike, NaiveDate};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::fs;
use std::io::Read;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportOptions {
    pub target_unit: Option<String>,
    pub mode: String, // "replace_period", "replace_all_unit", "skip_duplicates"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    pub unit: String,
    pub detected_unit: String,
    pub file_name: String,
    pub imported_records: usize,
    pub skipped_records: usize,
    pub total_amount: f64,
    pub months_affected: Vec<String>,
    pub period: String,
    pub file_hash: String,
    pub stored_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportPreview {
    pub file_name: String,
    pub file_size_bytes: u64,
    pub file_hash: String,
    pub detected_unit: String,
    pub target_sheet: String,
    pub total_sheet_rows: usize,
    pub valid_rows_count: usize,
    pub total_amount: f64,
    pub periods_detected: Vec<String>,
    pub period_string: String,
    pub is_duplicate_file: bool,
    pub duplicate_log: Option<crate::db::ImportLogRow>,
    pub existing_period_transactions_count: usize,
}

#[derive(Debug, Clone)]
pub struct ParsedReservation {
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
}

#[derive(Debug, Clone)]
pub struct ParsedRow {
    pub date: String,
    pub amount: f64,
    pub notes: Option<String>,
    pub reservation: Option<ParsedReservation>,
}

struct SheetParseResult {
    rows_count: usize,
    parsed_rows: Vec<ParsedRow>,
    affected_months: HashSet<String>,
}

/// Computes SHA-256 hash of a file for integrity check and duplicate detection.
pub fn compute_file_hash(path: &Path) -> Result<String, String> {
    let mut file = fs::File::open(path)
        .map_err(|e| format!("Gagal membaca berkas untuk verifikasi hash: {}", e))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 16384];
    loop {
        let count = file
            .read(&mut buffer)
            .map_err(|e| format!("Gagal menghitung hash berkas: {}", e))?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

/// Parses revenue data from a Calamine sheet range.
fn parse_revenue_range(range: &calamine::Range<Data>) -> Result<SheetParseResult, String> {
    let (rows_count, cols_count) = range.get_size();
    if rows_count == 0 || cols_count == 0 {
        return Err("Lembar kerja Excel kosong.".to_string());
    }

    let scan_limit = rows_count.min(25);
    let mut header_row_idx: Option<usize> = None;
    let mut is_reservation_sheet = false;

    let mut date_col_idx: Option<usize> = None;
    let mut name_col_idx: Option<usize> = None;
    let mut person_col_idx: Option<usize> = None;
    let mut booking_col_idx: Option<usize> = None;
    let mut nationality_col_idx: Option<usize> = None;
    let mut room_col_idx: Option<usize> = None;
    let mut total_rooms_col_idx: Option<usize> = None;
    let mut checkin_col_idx: Option<usize> = None;
    let mut checkout_col_idx: Option<usize> = None;
    let mut nights_col_idx: Option<usize> = None;
    let mut rate_col_idx: Option<usize> = None;
    let mut payment_col_idx: Option<usize> = None;
    let mut remarks_col_idx: Option<usize> = None;

    let mut legacy_amount_col_idx: Option<usize> = None;
    let mut legacy_notes_col_idx: Option<usize> = None;

    for r in 0..scan_limit {
        let mut d_col = None;
        let mut n_col = None;
        let mut p_col = None;
        let mut b_col = None;
        let mut nat_col = None;
        let mut rm_col = None;
        let mut tr_col = None;
        let mut ci_col = None;
        let mut co_col = None;
        let mut tn_col = None;
        let mut rt_col = None;
        let mut pay_col = None;
        let mut rem_col = None;

        let mut leg_amt_col = None;
        let mut leg_notes_col = None;

        for c in 0..cols_count {
            if let Some(cell) = range.get((r, c)) {
                let text = cell.to_string().to_lowercase();
                let clean = text.trim();

                if clean == "tanggal"
                    || clean == "tgl"
                    || clean == "date"
                    || clean == "waktu"
                    || (clean.contains("tanggal") && !clean.contains("check"))
                {
                    d_col = Some(c);
                } else if clean == "name"
                    || clean == "nama"
                    || clean == "guest"
                    || clean == "guest name"
                    || clean == "nama tamu"
                {
                    n_col = Some(c);
                } else if clean == "total person"
                    || clean == "person"
                    || clean == "pax"
                    || clean == "orang"
                    || clean == "tamu"
                    || clean == "jml tamu"
                {
                    p_col = Some(c);
                } else if clean == "booking number"
                    || clean == "booking no"
                    || clean == "no booking"
                    || clean == "kode booking"
                    || clean == "booking id"
                {
                    b_col = Some(c);
                } else if clean == "nationality"
                    || clean == "nationality / daerah asal"
                    || clean == "negara / daerah asal"
                    || clean == "negara/daerah asal"
                    || clean == "asal"
                    || clean == "daerah asal"
                    || clean == "warga negara"
                    || clean == "kebangsaan"
                    || clean == "negara"
                {
                    nat_col = Some(c);
                } else if clean == "room number"
                    || clean == "room no"
                    || clean == "no kamar"
                    || clean == "kamar"
                    || clean == "room"
                {
                    rm_col = Some(c);
                } else if clean == "total kamar"
                    || clean == "total room"
                    || clean == "jml kamar"
                {
                    tr_col = Some(c);
                } else if clean == "date check in"
                    || clean == "check in"
                    || clean == "checkin"
                    || clean == "tgl check in"
                    || clean == "tgl masuk"
                {
                    ci_col = Some(c);
                } else if clean == "date check out"
                    || clean == "check out"
                    || clean == "checkout"
                    || clean == "tgl check out"
                    || clean == "tgl keluar"
                {
                    co_col = Some(c);
                } else if clean == "total night"
                    || clean == "night"
                    || clean == "nights"
                    || clean == "malam"
                    || clean == "jml malam"
                {
                    tn_col = Some(c);
                } else if clean == "rate"
                    || clean == "tarif"
                    || clean == "harga per malam"
                    || clean == "room rate"
                {
                    rt_col = Some(c);
                } else if clean == "total payment"
                    || clean == "payment"
                    || clean == "total bayar"
                    || clean == "total bayaran"
                {
                    pay_col = Some(c);
                } else if clean == "remarks"
                    || clean == "channel"
                    || clean == "sumber"
                    || clean == "ota"
                {
                    rem_col = Some(c);
                }

                if clean == "total"
                    || clean == "jumlah"
                    || clean == "omset"
                    || clean == "omzet"
                    || clean == "pendapatan"
                    || clean == "amount"
                    || clean == "kredit"
                    || clean == "credit"
                    || clean == "nilai"
                {
                    leg_amt_col = Some(c);
                } else if clean == "keterangan"
                    || clean == "ket"
                    || clean == "notes"
                    || clean == "uraian"
                    || clean == "deskripsi"
                {
                    leg_notes_col = Some(c);
                }
            }
        }

        let res_score = [
            n_col.is_some(),
            rm_col.is_some(),
            ci_col.is_some(),
            co_col.is_some(),
            pay_col.is_some(),
            rt_col.is_some(),
            tn_col.is_some(),
        ]
        .iter()
        .filter(|&&b| b)
        .count();

        if d_col.is_some() && (pay_col.is_some() || leg_amt_col.is_some()) && res_score >= 3 {
            header_row_idx = Some(r);
            is_reservation_sheet = true;
            date_col_idx = d_col;
            name_col_idx = n_col;
            person_col_idx = p_col;
            booking_col_idx = b_col;
            nationality_col_idx = nat_col;
            room_col_idx = rm_col;
            total_rooms_col_idx = tr_col;
            checkin_col_idx = ci_col;
            checkout_col_idx = co_col;
            nights_col_idx = tn_col;
            rate_col_idx = rt_col;
            payment_col_idx = pay_col.or(leg_amt_col);
            remarks_col_idx = rem_col.or(leg_notes_col);
            break;
        } else if d_col.is_some() && leg_amt_col.is_some() {
            header_row_idx = Some(r);
            is_reservation_sheet = false;
            date_col_idx = d_col;
            legacy_amount_col_idx = leg_amt_col;
            legacy_notes_col_idx = leg_notes_col;
            break;
        }
    }

    let start_row = match header_row_idx {
        Some(hr) => hr + 1,
        None => {
            if rows_count > 1
                && parse_cell_date(range.get((1, 0)).unwrap_or(&Data::Empty)).is_some()
                && parse_cell_amount(range.get((1, 1)).unwrap_or(&Data::Empty)).is_some()
            {
                date_col_idx = Some(0);
                legacy_amount_col_idx = Some(1);
                legacy_notes_col_idx = if cols_count > 2 { Some(2) } else { None };
                is_reservation_sheet = false;
                1
            } else {
                return Err("Format tabel tidak sesuai standar. Kolom 'Tanggal' dan 'Total'/'Nominal' tidak ditemukan. Pastikan berkas menggunakan format tabel pendapatan yang sesuai.".to_string());
            }
        }
    };

    let date_col = date_col_idx.unwrap();
    let mut parsed_rows: Vec<ParsedRow> = Vec::new();
    let mut affected_months: HashSet<String> = HashSet::new();

    for r in start_row..rows_count {
        let date_cell = range.get((r, date_col));
        if date_cell.is_none() {
            continue;
        }
        let dc = date_cell.unwrap();

        let date_raw_str = dc.to_string().trim().to_lowercase();
        if date_raw_str.is_empty()
            || date_raw_str.starts_with("total")
            || date_raw_str.starts_with("jumlah")
            || date_raw_str.starts_with("grand")
            || date_raw_str.starts_with("saldo")
        {
            continue;
        }

        let parsed_d = match parse_cell_date(dc) {
            Some(d) => d,
            None => continue,
        };

        if is_reservation_sheet {
            let amount_cell = payment_col_idx.and_then(|c| range.get((r, c)));
            let amt = amount_cell.and_then(parse_cell_amount).unwrap_or(0.0);
            if amt <= 0.0 {
                continue;
            }

            let d_str = parsed_d.format("%Y-%m-%d").to_string();
            let m_str = format!("{:04}-{:02}", parsed_d.year(), parsed_d.month());
            affected_months.insert(m_str);

            let guest_name = name_col_idx
                .and_then(|c| range.get((r, c)))
                .map(|v| v.to_string().trim().to_string())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "Tamu".to_string());

            let total_person = person_col_idx
                .and_then(|c| range.get((r, c)))
                .and_then(|v| parse_cell_amount(v).map(|f| f as i64));

            let booking_number = booking_col_idx
                .and_then(|c| range.get((r, c)))
                .map(|v| v.to_string().trim().to_string())
                .filter(|s| !s.is_empty() && s != "-");

            let nationality = nationality_col_idx
                .and_then(|c| range.get((r, c)))
                .map(|v| v.to_string().trim().to_string())
                .filter(|s| !s.is_empty() && s != "-");

            let room_number = room_col_idx
                .and_then(|c| range.get((r, c)))
                .map(|v| v.to_string().trim().to_string())
                .filter(|s| !s.is_empty() && s != "-");

            let total_rooms = total_rooms_col_idx
                .and_then(|c| range.get((r, c)))
                .and_then(|v| parse_cell_amount(v).map(|f| f as i64))
                .unwrap_or(1)
                .max(1);

            let raw_nights = nights_col_idx
                .and_then(|c| range.get((r, c)))
                .and_then(|v| parse_cell_amount(v).map(|f| f as i64));

            let check_in_naive = checkin_col_idx
                .and_then(|c| range.get((r, c)))
                .and_then(parse_cell_date)
                .unwrap_or(parsed_d);

            let check_out_naive = match checkout_col_idx
                .and_then(|c| range.get((r, c)))
                .and_then(parse_cell_date)
            {
                Some(co) if co > check_in_naive => co,
                _ => {
                    let fallback_nights = raw_nights.filter(|&n| n > 0).unwrap_or(1);
                    check_in_naive + chrono::Duration::days(fallback_nights)
                }
            };

            let check_in_date = check_in_naive.format("%Y-%m-%d").to_string();
            let check_out_date = check_out_naive.format("%Y-%m-%d").to_string();
            let total_nights = raw_nights
                .filter(|&n| n > 0)
                .or_else(|| Some((check_out_naive - check_in_naive).num_days().max(1)));

            let rate = rate_col_idx
                .and_then(|c| range.get((r, c)))
                .and_then(parse_cell_amount)
                .or_else(|| {
                    if let Some(n) = total_nights {
                        if n > 0 && total_rooms > 0 {
                            Some(amt / (n as f64 * total_rooms as f64))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                });

            let remarks = remarks_col_idx
                .and_then(|c| range.get((r, c)))
                .map(|v| v.to_string().trim().to_string())
                .filter(|s| !s.is_empty() && s != "-");

            let formatted_notes = match (&room_number, &remarks) {
                (Some(rm), Some(rem)) => Some(format!("{} ({}) - Kamar {}", guest_name, rem, rm)),
                (None, Some(rem)) => Some(format!("{} ({})", guest_name, rem)),
                (Some(rm), None) => Some(format!("{} - Kamar {}", guest_name, rm)),
                (None, None) => Some(guest_name.clone()),
            };

            parsed_rows.push(ParsedRow {
                date: d_str,
                amount: amt,
                notes: formatted_notes,
                reservation: Some(ParsedReservation {
                    guest_name,
                    total_person,
                    booking_number,
                    nationality,
                    room_number,
                    total_rooms,
                    check_in_date,
                    check_out_date,
                    total_nights,
                    rate,
                    total_payment: amt,
                    remarks,
                }),
            });
        } else {
            let amount_cell = legacy_amount_col_idx.and_then(|c| range.get((r, c)));
            let notes_cell = legacy_notes_col_idx.and_then(|c| range.get((r, c)));

            if let Some(ac) = amount_cell {
                if let Some(amt) = parse_cell_amount(ac) {
                    if amt >= 0.0 {
                        let d_str = parsed_d.format("%Y-%m-%d").to_string();
                        let m_str = format!("{:04}-{:02}", parsed_d.year(), parsed_d.month());
                        affected_months.insert(m_str);

                        let notes = notes_cell
                            .map(|n| n.to_string().trim().to_string())
                            .filter(|s| !s.is_empty());

                        parsed_rows.push(ParsedRow {
                            date: d_str,
                            amount: amt,
                            notes,
                            reservation: None,
                        });
                    }
                }
            }
        }
    }

    if parsed_rows.is_empty() {
        return Err("Tidak ada baris data pendapatan yang valid ditemukan dalam berkas.".to_string());
    }

    Ok(SheetParseResult {
        rows_count,
        parsed_rows,
        affected_months,
    })
}

/// Detects the likely business unit based on filename and sheet names.
pub fn detect_unit_from_context(file_path: &Path, sheet_names: &[String]) -> String {
    let mut text_corpus = file_path
        .file_name()
        .map(|n| n.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    for sheet in sheet_names {
        text_corpus.push(' ');
        text_corpus.push_str(&sheet.to_lowercase());
    }

    if text_corpus.contains("resto")
        || text_corpus.contains("restaurant")
        || text_corpus.contains("makan")
        || text_corpus.contains("f&b")
        || text_corpus.contains("food")
    {
        "Restaurant".to_string()
    } else if text_corpus.contains("bungalow")
        || text_corpus.contains("kamar")
        || text_corpus.contains("villa")
        || text_corpus.contains("room")
        || text_corpus.contains("penginapan")
    {
        "Bungalows".to_string()
    } else {
        "Bungalows".to_string()
    }
}

/// Converts Excel serial date to NaiveDate (handling 1900 leap year bug).
fn excel_serial_to_date(serial: f64) -> Option<NaiveDate> {
    if !serial.is_finite() || !(1.0..=2958465.0).contains(&serial) {
        return None;
    }
    let days = serial.floor() as i64;
    // Excel erroneously treats 1900 as a leap year. Day 60 is 1900-02-29.
    // For serial >= 61, subtract 2 from 1900-01-01 base (which is 1899-12-30).
    let base = NaiveDate::from_ymd_opt(1899, 12, 30)?;
    base.checked_add_signed(chrono::Duration::days(days))
}

/// Normalizes dates from strings (DD/MM/YY, YYYY-MM-DD, DD/MM/YYYY, DD-MM-YYYY, ISO with time, or month names).
pub(crate) fn parse_date_string(s: &str) -> Option<NaiveDate> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Strip optional trailing time part (e.g. "2026-08-02T14:30:00Z", "02/08/26 00:00:00")
    let clean = if let Some(t_idx) = trimmed.find('T') {
        let prefix = trimmed[..t_idx].trim();
        let suffix = &trimmed[t_idx + 1..];
        let ends_with_digit = prefix.chars().last().is_some_and(|c| c.is_ascii_digit());
        let starts_with_digit = suffix.chars().next().is_some_and(|c| c.is_ascii_digit());
        if ends_with_digit && starts_with_digit && suffix.contains(':') {
            prefix
        } else {
            trimmed
        }
    } else if let Some((head, tail)) = trimmed.rsplit_once(' ') {
        if tail.contains(':') {
            head.trim()
        } else {
            trimmed
        }
    } else {
        trimmed
    };

    if clean.is_empty() {
        return None;
    }

    // Accept text-stored Excel serial numbers (e.g. "46235")
    if !clean.contains('/') && !clean.contains('-') && !clean.chars().any(|c| c.is_alphabetic()) {
        if let Ok(f) = clean.parse::<f64>() {
            if (20_000.0..=100_000.0).contains(&f) {
                if let Some(d) = excel_serial_to_date(f) {
                    return Some(d);
                }
            }
        }
    }

    // Direct 4-digit year formats first (require year >= 1000 so 2-digit '26' isn't read as year 0026)
    for fmt in [
        "%Y-%m-%d",
        "%d/%m/%Y",
        "%d-%m-%Y",
        "%d.%m.%Y",
        "%Y/%m/%d",
        "%Y.%m.%d",
        "%d/%m-%Y",
        "%d-%m/%Y",
    ] {
        if let Ok(d) = NaiveDate::parse_from_str(clean, fmt) {
            if chrono::Datelike::year(&d) >= 1000 {
                return Some(d);
            }
        }
    }
    for fmt in [
        "%d/%m-%y",
        "%d-%m/%y",
        "%d/%m/%y",
        "%d-%m-%y",
        "%d.%m.%y",
    ] {
        if let Ok(d) = NaiveDate::parse_from_str(clean, fmt) {
            return Some(d);
        }
    }

    // Month name mapping (Indonesian & English variants)
    let months = [
        ("januari", 1),
        ("january", 1),
        ("jan", 1),
        ("februari", 2),
        ("february", 2),
        ("febr", 2),
        ("feb", 2),
        ("maret", 3),
        ("march", 3),
        ("mar", 3),
        ("april", 4),
        ("apr", 4),
        ("mei", 5),
        ("may", 5),
        ("juni", 6),
        ("june", 6),
        ("jun", 6),
        ("juli", 7),
        ("july", 7),
        ("jul", 7),
        ("agustus", 8),
        ("august", 8),
        ("agust", 8),
        ("agu", 8),
        ("agt", 8),
        ("aug", 8),
        ("september", 9),
        ("sept", 9),
        ("sep", 9),
        ("oktober", 10),
        ("october", 10),
        ("okto", 10),
        ("okt", 10),
        ("oct", 10),
        ("november", 11),
        ("nopember", 11),
        ("nov", 11),
        ("nop", 11),
        ("desember", 12),
        ("december", 12),
        ("dese", 12),
        ("des", 12),
        ("dec", 12),
    ];

    let lower = clean.to_lowercase();
    let parts: Vec<&str> = lower
        .split(|c: char| c == ' ' || c == '-' || c == '/' || c == '.' || c == ',')
        .filter(|p| !p.is_empty())
        .collect();

    if parts.len() == 3 {
        if let (Ok(p0), Ok(p1), Ok(p2)) = (
            parts[0].parse::<i32>(),
            parts[1].parse::<u32>(),
            parts[2].parse::<i32>(),
        ) {
            if p0 > 1000 && p2 >= 1 {
                return NaiveDate::from_ymd_opt(p0, p1, p2 as u32);
            }
            if p0 >= 1 && p2 >= 0 {
                let full_year = if p2 < 70 {
                    2000 + p2
                } else if p2 < 100 {
                    1900 + p2
                } else {
                    p2
                };
                if let Some(d) = NaiveDate::from_ymd_opt(full_year, p1, p0 as u32) {
                    return Some(d);
                }
                if p1 > 12 && p0 <= 12 {
                    return NaiveDate::from_ymd_opt(full_year, p0 as u32, p1);
                }
            }
            return None;
        }

        let mut day_opt: Option<u32> = None;
        let mut month_opt: Option<u32> = None;
        let mut year_opt: Option<i32> = None;

        for part in &parts {
            if let Ok(num) = part.parse::<i32>() {
                if (1900..2100).contains(&num) && year_opt.is_none() {
                    year_opt = Some(num);
                } else if (1..=31).contains(&num) && day_opt.is_none() {
                    day_opt = Some(num as u32);
                } else if (0..100).contains(&num) && year_opt.is_none() {
                    year_opt = Some(if num < 70 { 2000 + num } else { 1900 + num });
                }
            } else {
                for &(m_name, m_num) in &months {
                    if *part == m_name {
                        month_opt = Some(m_num);
                        break;
                    }
                }
            }
        }

        if let (Some(y), Some(m), Some(d)) = (year_opt, month_opt, day_opt) {
            return NaiveDate::from_ymd_opt(y, m, d);
        }
    }

    None
}

/// Parses a cell into NaiveDate.
fn parse_cell_date(cell: &Data) -> Option<NaiveDate> {
    match cell {
        Data::Int(v) => excel_serial_to_date(*v as f64),
        Data::Float(v) => excel_serial_to_date(*v),
        Data::String(s) => parse_date_string(s),
        Data::DateTime(dt) => {
            let serial = dt.as_f64();
            excel_serial_to_date(serial)
        }
        Data::DateTimeIso(iso) => {
            let prefix: String = iso.chars().take(10).collect();
            NaiveDate::parse_from_str(&prefix, "%Y-%m-%d").ok()
        }
        _ => None,
    }
}

/// Parses a cell into an f64 amount.
fn parse_cell_amount(cell: &Data) -> Option<f64> {
    match cell {
        Data::Int(v) => Some(*v as f64),
        Data::Float(v) => Some(*v),
        Data::String(s) => {
            let mut clean = s.trim().to_string();
            if clean.is_empty() {
                return None;
            }

            // Remove Rp, rp, IDR, spaces
            clean = clean
                .replace("Rp", "")
                .replace("rp", "")
                .replace("IDR", "")
                .replace("idr", "")
                .replace(' ', "");

            let is_negative = clean.starts_with('-') || (clean.starts_with('(') && clean.ends_with(')'));
            clean = clean.replace('(', "").replace(')', "").replace('-', "");

            // Indonesian format: 1.500.000,00 -> 1500000.00
            // Or English format: 1,500,000.00 -> 1500000.00
            let has_comma = clean.contains(',');
            let has_dot = clean.contains('.');

            let normalized = if has_comma && has_dot {
                let comma_pos = clean.rfind(',').unwrap();
                let dot_pos = clean.rfind('.').unwrap();
                if comma_pos > dot_pos {
                    // Indonesian: dot is thousand, comma is decimal
                    clean.replace('.', "").replace(',', ".")
                } else {
                    // English: comma is thousand, dot is decimal
                    clean.replace(',', "")
                }
            } else if has_comma {
                // If comma followed by 2 digits at end, treat as decimal, else thousands
                let comma_pos = clean.rfind(',').unwrap();
                if clean.len() - comma_pos == 3 {
                    clean.replace(',', ".")
                } else {
                    clean.replace(',', "")
                }
            } else if has_dot {
                // If dot followed by 3 digits, usually thousand separator
                let dot_pos = clean.rfind('.').unwrap();
                if clean.len() - dot_pos == 4 && clean.matches('.').count() >= 1 {
                    clean.replace('.', "")
                } else {
                    clean
                }
            } else {
                clean
            };

            if let Ok(mut val) = normalized.parse::<f64>() {
                if is_negative {
                    val = -val;
                }
                Some(val)
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Imports and processes an Excel file into the SQLite database.
pub fn import_excel(
    conn: &mut Connection,
    file_path_str: &str,
    archive_dir: &Path,
    options: ImportOptions,
) -> Result<ImportResult, String> {
    let src_path = Path::new(file_path_str);
    if !src_path.exists() {
        return Err(format!("Berkas Excel tidak ditemukan: {}", file_path_str));
    }

    let ext = src_path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();
    if ext != "xlsx" && ext != "xls" && ext != "xlsm" {
        return Err(format!(
            "Format berkas '{}' tidak didukung. Sistem hanya mendukung berkas Excel (.xlsx, .xls, .xlsm).",
            if ext.is_empty() { "tanpa ekstensi" } else { &ext }
        ));
    }

    let file_hash = compute_file_hash(src_path)?;

    let file_name = src_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "import.xlsx".to_string());
    let mut workbook = open_workbook_auto(src_path)
        .map_err(|e| format!("Gagal membuka berkas Excel: {}", e))?;

    let sheet_names: Vec<String> = workbook.sheet_names().to_vec();
    if sheet_names.is_empty() {
        return Err("Berkas Excel tidak memiliki lembar kerja (sheet).".to_string());
    }

    let detected_unit = detect_unit_from_context(src_path, &sheet_names);
    let target_unit = options
        .target_unit
        .clone()
        .unwrap_or_else(|| detected_unit.clone());

    // Find best sheet
    let target_sheet_name = if sheet_names.len() == 1 {
        sheet_names[0].clone()
    } else {
        // Find sheet matching target unit
        let keyword = target_unit.to_lowercase();
        sheet_names
            .iter()
            .find(|s| s.to_lowercase().contains(&keyword))
            .cloned()
            .unwrap_or_else(|| sheet_names[0].clone())
    };

    let range = workbook
        .worksheet_range(&target_sheet_name)
        .map_err(|e| format!("Gagal membaca sheet '{}': {}", target_sheet_name, e))?;

    let parse_res = parse_revenue_range(&range)?;
    let parsed_rows = parse_res.parsed_rows;
    let affected_months = parse_res.affected_months;

    // Physical File Archiving
    fs::create_dir_all(archive_dir)
        .map_err(|e| format!("Gagal membuat direktori arsip Excel: {}", e))?;
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    let safe_unit = target_unit.replace(['/', '\\', ' '], "_");
    let safe_file_name = file_name.replace(['/', '\\', ' '], "_");
    let archive_file_name = format!("{}_{}_{}", timestamp, safe_unit, safe_file_name);
    let archive_path = archive_dir.join(&archive_file_name);

    fs::copy(src_path, &archive_path)
        .map_err(|e| format!("Gagal mengarsipkan berkas Excel ke '{}': {}", archive_path.display(), e))?;
    let stored_path = archive_path.to_string_lossy().to_string();

    // Execute SQLite Transactions
    let tx = conn
        .transaction()
        .map_err(|e| format!("Gagal memulai transaksi database: {}", e))?;

    // Apply Import Mode
    match options.mode.as_str() {
        "replace_all_unit" => {
            tx.execute("DELETE FROM room_reservations WHERE unit = ?1", params![target_unit])
                .map_err(|e| format!("Gagal menghapus seluruh data reservasi unit: {}", e))?;
            tx.execute("DELETE FROM transactions WHERE unit = ?1", params![target_unit])
                .map_err(|e| format!("Gagal menghapus seluruh data unit: {}", e))?;
        }
        "skip_duplicates" => {
            // Do not delete anything beforehand
        }
        _ => {
            // Default: "replace_period"
            for m in &affected_months {
                let start_date = format!("{}-01", m);
                let end_date = format!("{}-31", m);
                tx.execute(
                    "DELETE FROM room_reservations WHERE unit = ?1 AND date BETWEEN ?2 AND ?3",
                    params![target_unit, start_date, end_date],
                )
                .map_err(|e| format!("Gagal mengganti periode reservasi {}: {}", m, e))?;
                tx.execute(
                    "DELETE FROM transactions WHERE unit = ?1 AND date BETWEEN ?2 AND ?3",
                    params![target_unit, start_date, end_date],
                )
                .map_err(|e| format!("Gagal mengganti periode {}: {}", m, e))?;
            }
        }
    }

    let mut imported_count = 0;
    let mut skipped_count = 0;
    let mut total_amount = 0.0;

    let now_local = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    {
        let mut insert_stmt = tx
            .prepare(
                "INSERT INTO transactions (unit, date, amount, notes, source_file, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            )
            .map_err(|e| e.to_string())?;
        let mut check_stmt = tx
            .prepare(
                "SELECT COUNT(*) FROM transactions WHERE unit = ?1 AND date = ?2 AND amount = ?3 AND COALESCE(notes, '') = ?4",
            )
            .map_err(|e| e.to_string())?;
        let mut insert_res_stmt = tx
            .prepare(
                "INSERT INTO room_reservations (
                    unit, transaction_id, date, guest_name, total_person, booking_number,
                    nationality, room_number, total_rooms, check_in_date, check_out_date,
                    total_nights, rate, total_payment, remarks, source_file, created_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
            )
            .map_err(|e| e.to_string())?;

        for row in &parsed_rows {
            if options.mode == "skip_duplicates" {
                let notes_val = row.notes.as_deref().unwrap_or("");
                let exists: i64 = check_stmt
                    .query_row(
                        params![target_unit, row.date, row.amount, notes_val],
                        |r| r.get(0),
                    )
                    .unwrap_or(0);
                if exists > 0 {
                    skipped_count += 1;
                    continue;
                }
            }

            insert_stmt
                .execute(params![
                    target_unit,
                    row.date,
                    row.amount,
                    row.notes,
                    file_name,
                    now_local
                ])
                .map_err(|e| format!("Gagal menyisipkan transaksi: {}", e))?;
            let trans_id = tx.last_insert_rowid();

            if let Some(res) = &row.reservation {
                insert_res_stmt
                    .execute(params![
                        target_unit,
                        trans_id,
                        row.date,
                        res.guest_name,
                        res.total_person,
                        res.booking_number,
                        res.nationality,
                        res.room_number,
                        res.total_rooms,
                        res.check_in_date,
                        res.check_out_date,
                        res.total_nights,
                        res.rate,
                        res.total_payment,
                        res.remarks,
                        file_name,
                        now_local
                    ])
                    .map_err(|e| format!("Gagal menyisipkan reservasi kamar: {}", e))?;
            }
            imported_count += 1;
            total_amount += row.amount;
        }
    }

    // Record import log
    let mut months_vec: Vec<String> = affected_months.into_iter().collect();
    months_vec.sort();
    let period_str = months_vec.join(", ");

    // Record import log with period, file_hash, and local system timestamp
    tx.execute(
        "INSERT INTO import_logs (unit, file_name, records_count, total_amount, period, file_hash, stored_path, imported_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            target_unit,
            file_name,
            imported_count as i64,
            total_amount,
            period_str,
            file_hash,
            stored_path,
            now_local
        ],
    )
    .map_err(|e| format!("Gagal mencatat log impor: {}", e))?;

    tx.commit()
        .map_err(|e| format!("Gagal menyimpan transaksi ke database: {}", e))?;

    Ok(ImportResult {
        unit: target_unit,
        detected_unit,
        file_name,
        imported_records: imported_count,
        skipped_records: skipped_count,
        total_amount,
        months_affected: months_vec,
        period: period_str,
        file_hash,
        stored_path,
    })
}

/// Analyzes and validates an Excel file prior to execution, detecting structure, periods, and duplicates.
pub fn preview_excel_import(
    conn: &Connection,
    file_path_str: &str,
    target_unit: Option<String>,
) -> Result<ImportPreview, String> {
    let src_path = Path::new(file_path_str);
    if !src_path.exists() {
        return Err(format!("Berkas Excel tidak ditemukan: {}", file_path_str));
    }

    let ext = src_path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();
    if ext != "xlsx" && ext != "xls" && ext != "xlsm" {
        return Err(format!(
            "Format berkas '{}' tidak didukung. Sistem hanya mendukung berkas Excel (.xlsx, .xls, .xlsm).",
            if ext.is_empty() { "tanpa ekstensi" } else { &ext }
        ));
    }

    let file_size_bytes = fs::metadata(src_path).map(|m| m.len()).unwrap_or(0);
    let file_hash = compute_file_hash(src_path)?;

    let file_name = src_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "import.xlsx".to_string());

    let mut workbook = open_workbook_auto(src_path)
        .map_err(|e| format!("Gagal membuka berkas Excel: {}. Pastikan berkas tidak rusak atau terenkripsi.", e))?;

    let sheet_names: Vec<String> = workbook.sheet_names().to_vec();
    if sheet_names.is_empty() {
        return Err("Berkas Excel tidak memiliki lembar kerja (sheet).".to_string());
    }

    let detected_unit = detect_unit_from_context(src_path, &sheet_names);
    let effective_unit = target_unit.unwrap_or_else(|| detected_unit.clone());

    let target_sheet_name = if sheet_names.len() == 1 {
        sheet_names[0].clone()
    } else {
        let keyword = effective_unit.to_lowercase();
        sheet_names
            .iter()
            .find(|s| s.to_lowercase().contains(&keyword))
            .cloned()
            .unwrap_or_else(|| sheet_names[0].clone())
    };

    let range = workbook
        .worksheet_range(&target_sheet_name)
        .map_err(|e| format!("Gagal membaca sheet '{}': {}", target_sheet_name, e))?;

    let parse_res = parse_revenue_range(&range)?;

    let mut periods_vec: Vec<String> = parse_res.affected_months.into_iter().collect();
    periods_vec.sort();
    let period_str = periods_vec.join(", ");

    let total_amount: f64 = parse_res.parsed_rows.iter().map(|r| r.amount).sum();
    let valid_rows_count = parse_res.parsed_rows.len();

    // Check for duplicate import log in database
    let mut duplicate_log: Option<crate::db::ImportLogRow> = None;
    let mut is_duplicate_file = false;

    let mut check_stmt = conn
        .prepare(
            "SELECT id, unit, file_name, records_count, total_amount, period, file_hash, imported_at, stored_path FROM import_logs WHERE file_hash = ?1 ORDER BY id DESC LIMIT 1"
        )
        .map_err(|e| e.to_string())?;

    let mut rows = check_stmt
        .query_map(params![file_hash], |r| {
            Ok(crate::db::ImportLogRow {
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

    if let Some(Ok(log)) = rows.next() {
        is_duplicate_file = true;
        duplicate_log = Some(log);
    } else {
        let mut check_name_stmt = conn
            .prepare(
                "SELECT id, unit, file_name, records_count, total_amount, period, file_hash, imported_at, stored_path FROM import_logs WHERE file_name = ?1 AND records_count = ?2 AND ABS(total_amount - ?3) < 0.01 ORDER BY id DESC LIMIT 1"
            )
            .map_err(|e| e.to_string())?;
        let mut name_rows = check_name_stmt
            .query_map(params![file_name, valid_rows_count as i64, total_amount], |r| {
                Ok(crate::db::ImportLogRow {
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
        if let Some(Ok(log)) = name_rows.next() {
            is_duplicate_file = true;
            duplicate_log = Some(log);
        }
    }

    // Count existing transactions in DB for affected periods
    let mut existing_period_transactions_count = 0;
    for m in &periods_vec {
        let start_date = format!("{}-01", m);
        let end_date = format!("{}-31", m);
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM transactions WHERE unit = ?1 AND date BETWEEN ?2 AND ?3",
                params![effective_unit, start_date, end_date],
                |r| r.get(0),
            )
            .unwrap_or(0);
        existing_period_transactions_count += count as usize;
    }

    Ok(ImportPreview {
        file_name,
        file_size_bytes,
        file_hash,
        detected_unit,
        target_sheet: target_sheet_name,
        total_sheet_rows: parse_res.rows_count,
        valid_rows_count,
        total_amount,
        periods_detected: periods_vec,
        period_string: period_str,
        is_duplicate_file,
        duplicate_log,
        existing_period_transactions_count,
    })
}
