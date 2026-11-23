use chrono::{Datelike, NaiveDate, Weekday};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyRecapItem {
    pub day: u32,
    pub date: String,
    pub day_name: String,
    pub revenue: f64,
    pub is_active: bool,
    pub cumulative_revenue: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxRecapReport {
    pub unit: String,
    pub unit_name: String,
    pub month: String, // "YYYY-MM"
    pub month_name: String, // e.g. "Agustus 2026"
    pub year: i32,
    pub month_number: u32,
    pub days_in_month: u32,
    pub total_revenue: f64,
    pub tax_rate: f64,
    pub tax_due: f64,
    pub net_revenue: f64,
    pub active_days: usize,
    pub zero_days: usize,
    pub max_daily_revenue: f64,
    pub min_daily_revenue_non_zero: f64,
    pub avg_per_day: f64,
    pub document_number: String,
    pub npwpd: String,
    pub business_name: String,
    pub business_address: String,
    pub business_contact: String,
    pub sign_position: String,
    pub sign_name: String,
    pub officer_name: String,
    pub officer_nip: String,
    pub generated_at: String,
    pub daily_items: Vec<DailyRecapItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TourismWeeklyRow {
    pub week_number: u32,
    pub label: String,
    pub days_count: i64,
    pub domestic_check_in: i64,
    pub domestic_check_out: i64,
    pub foreign_check_in: i64,
    pub foreign_check_out: i64,
    pub room_occupied: i64,
    pub room_vacant: i64,
    pub bed_check_out: i64,
    pub bed_occupied: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TourismOriginItem {
    pub rank: usize,
    pub origin_name: String,
    pub guest_count: i64,
    pub reservation_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TourismFormReport {
    pub unit: String,
    pub accommodation_name: String,
    pub accommodation_type: String,
    pub room_capacity: i64,
    pub month: String,
    pub month_name: String,
    pub year: i32,
    pub days_in_month: u32,
    pub total_reservations: usize,
    pub total_guests: i64,
    pub domestic_guests: i64,
    pub foreign_guests: i64,
    pub total_room_nights: i64,
    pub total_bed_nights: i64,
    pub occupancy_rate_pct: f64,
    pub avg_stay_nights: f64,
    pub stay_category: String,
    pub weekly_rows: Vec<TourismWeeklyRow>,
    pub weekly_total: TourismWeeklyRow,
    pub foreign_origins: Vec<TourismOriginItem>,
    pub domestic_origins: Vec<TourismOriginItem>,
}

fn indonesian_day_name(weekday: Weekday) -> &'static str {
    match weekday {
        Weekday::Mon => "Senin",
        Weekday::Tue => "Selasa",
        Weekday::Wed => "Rabu",
        Weekday::Thu => "Kamis",
        Weekday::Fri => "Jumat",
        Weekday::Sat => "Sabtu",
        Weekday::Sun => "Minggu",
    }
}

fn indonesian_month_name(month: u32) -> &'static str {
    match month {
        1 => "Januari",
        2 => "Februari",
        3 => "Maret",
        4 => "April",
        5 => "Mei",
        6 => "Juni",
        7 => "Juli",
        8 => "Agustus",
        9 => "September",
        10 => "Oktober",
        11 => "November",
        12 => "Desember",
        _ => "",
    }
}

fn days_in_month(year: i32, month: u32) -> u32 {
    let Some(first_of_this) = NaiveDate::from_ymd_opt(year, month, 1) else {
        return 31;
    };
    let next_month_date = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    };
    if let Some(next) = next_month_date {
        (next - first_of_this).num_days() as u32
    } else {
        31
    }
}

fn parse_year_month(month_str: &str) -> Result<(i32, u32), String> {
    let parts: Vec<&str> = month_str.split('-').collect();
    if parts.len() != 2 {
        return Err("Format bulan harus YYYY-MM (contoh: 2026-08)".to_string());
    }
    let year: i32 = parts[0]
        .parse()
        .map_err(|_| "Tahun tidak valid".to_string())?;
    let month_num: u32 = parts[1]
        .parse()
        .map_err(|_| "Bulan tidak valid".to_string())?;

    if !(1900..=2100).contains(&year) {
        return Err("Tahun harus antara 1900 dan 2100".to_string());
    }
    if !(1..=12).contains(&month_num) {
        return Err("Nomor bulan harus antara 01 dan 12".to_string());
    }
    Ok((year, month_num))
}

pub fn generate_tax_recap(
    conn: &Connection,
    unit: &str,
    month_str: &str, // "YYYY-MM"
) -> Result<TaxRecapReport, String> {
    let (year, month_num) = parse_year_month(month_str)?;

    let total_days = days_in_month(year, month_num);
    let start_date = format!("{}-{:02}-01", year, month_num);
    let end_date = format!("{}-{:02}-{:02}", year, month_num, total_days);

    // Fetch transactions grouped by day
    let mut stmt = conn
        .prepare(
            "SELECT date, SUM(amount) FROM transactions WHERE unit = ?1 AND date BETWEEN ?2 AND ?3 GROUP BY date ORDER BY date ASC",
        )
        .map_err(|e| format!("Query transaksi gagal: {}", e))?;

    let rows = stmt
        .query_map(params![unit, start_date, end_date], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, f64>(1)?))
        })
        .map_err(|e| format!("Eksekusi transaksi gagal: {}", e))?;

    let mut day_revenue_map: HashMap<String, f64> = HashMap::new();
    for r in rows {
        let (d, amt) = r.map_err(|e| e.to_string())?;
        day_revenue_map.insert(d, amt);
    }

    // Fetch Unit details
    let unit_info: (String, f64, Option<String>, Option<String>) = conn
        .query_row(
            "SELECT name, tax_rate, address, npwpd FROM units WHERE code = ?1",
            params![unit],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .unwrap_or_else(|_| (format!("ANDA {}", unit.to_uppercase()), 0.10, None, None));

    let (unit_name, tax_rate, unit_address, unit_npwpd) = unit_info;

    // Fetch Global Settings
    let mut settings_map: HashMap<String, String> = HashMap::new();
    if let Ok(mut set_stmt) = conn.prepare("SELECT key, value FROM settings") {
        if let Ok(s_rows) = set_stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?))) {
            for sr in s_rows.flatten() {
                settings_map.insert(sr.0, sr.1);
            }
        }
    }

    let business_name = settings_map
        .get("businessName")
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "ANDA BUNGALOWS & RESTAURANT".to_string());
    let business_address = unit_address.unwrap_or_else(|| {
        settings_map
            .get("businessAddress")
            .cloned()
            .unwrap_or_else(|| "Jalan Pariwisata Pantai Kuta, Lombok Tengah".to_string())
    });
    let business_contact = settings_map
        .get("contactNumber")
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "HP/WhatsApp: +62818-0578-3810".to_string());
    let npwpd = unit_npwpd.unwrap_or_else(|| {
        settings_map
            .get("npwpd")
            .cloned()
            .unwrap_or_else(|| "P.2.0001234.01.23".to_string())
    });
    let sign_position = settings_map
        .get("signPosition")
        .cloned()
        .unwrap_or_else(|| "Wajib Pajak Daerah".to_string());
    let sign_name = settings_map
        .get("signName")
        .cloned()
        .unwrap_or_else(|| "Pimpinan / Pengelola".to_string());
    let officer_name = settings_map
        .get("officerName")
        .cloned()
        .unwrap_or_else(|| "Petugas Bapenda Lombok Tengah".to_string());
    let officer_nip = settings_map
        .get("officerNip")
        .cloned()
        .unwrap_or_else(|| "NIP. 19800101 200501 1 001".to_string());

    // Build 1..total_days daily items
    let mut daily_items: Vec<DailyRecapItem> = Vec::with_capacity(total_days as usize);
    let mut cumulative_revenue = 0.0;
    let mut active_days = 0;
    let mut zero_days = 0;
    let mut max_daily_revenue: f64 = 0.0;
    let mut min_daily_revenue_non_zero: f64 = f64::MAX;

    for day in 1..=total_days {
        let date_obj = NaiveDate::from_ymd_opt(year, month_num, day).unwrap();
        let date_str = date_obj.format("%Y-%m-%d").to_string();
        let day_name = indonesian_day_name(date_obj.weekday()).to_string();

        let rev = day_revenue_map.get(&date_str).copied().unwrap_or(0.0);
        cumulative_revenue += rev;

        if rev > 0.0 {
            active_days += 1;
            if rev > max_daily_revenue {
                max_daily_revenue = rev;
            }
            if rev < min_daily_revenue_non_zero {
                min_daily_revenue_non_zero = rev;
            }
        } else {
            zero_days += 1;
        }

        daily_items.push(DailyRecapItem {
            day,
            date: date_str,
            day_name,
            revenue: rev,
            is_active: rev > 0.0,
            cumulative_revenue,
        });
    }

    if min_daily_revenue_non_zero == f64::MAX {
        min_daily_revenue_non_zero = 0.0;
    }

    let total_revenue = cumulative_revenue;
    let tax_due = (total_revenue * tax_rate).round();
    let net_revenue = total_revenue - tax_due;
    let avg_per_day = if total_days > 0 {
        total_revenue / (total_days as f64)
    } else {
        0.0
    };

    // Deterministic document number hash matching legacy format: No: ANDA-BGW/2026-09/XXXX
    let fallback_code: String = unit.chars().take(3).collect();
    let unit_code = match unit.to_lowercase().as_str() {
        "bungalows" | "bungalow" | "bgw" => "BGW",
        "restaurant" | "resto" | "rst" => "RST",
        _ => fallback_code.as_str(),
    };

    let doc_hash_src = format!("{}-{}-{}-{}", unit_code, year, month_num, total_revenue as u64);
    let doc_hash = {
        let mut h: u32 = 5381;
        for b in doc_hash_src.bytes() {
            h = ((h << 5).wrapping_add(h)).wrapping_add(b as u32);
        }
        format!("{:04X}", h & 0xFFFF)
    };

    let document_number = format!(
        "No: ANDA-{}/{}-{:02}/{}",
        unit_code,
        year,
        month_num,
        doc_hash
    );
    let month_name = format!("{} {}", indonesian_month_name(month_num), year);
    let generated_at = chrono::Local::now().format("%d-%m-%Y %H:%M:%S").to_string();

    Ok(TaxRecapReport {
        unit: unit.to_string(),
        unit_name,
        month: month_str.to_string(),
        month_name,
        year,
        month_number: month_num,
        days_in_month: total_days,
        total_revenue,
        tax_rate,
        tax_due,
        net_revenue,
        active_days,
        zero_days,
        max_daily_revenue,
        min_daily_revenue_non_zero,
        avg_per_day,
        document_number,
        npwpd,
        business_name,
        business_address,
        business_contact,
        sign_position,
        sign_name,
        officer_name,
        officer_nip,
        generated_at,
        daily_items,
    })
}

fn split_origin_prefix(raw: &str) -> (Option<bool>, &str) {
    let trimmed = raw.trim();
    const PREFIXES: &[(&str, bool)] = &[
        ("idn", true),
        ("domestik", true),
        ("domestic", true),
        ("wni", true),
        ("lokal", true),
        ("nusantara", true),
        ("intl", false),
        ("mancanegara", false),
        ("wna", false),
        ("foreign", false),
        ("asing", false),
    ];

    for (pfx, is_dom) in PREFIXES {
        if let Some(head) = trimmed.get(..pfx.len()) {
            if head.eq_ignore_ascii_case(pfx) {
                let rest = trimmed[pfx.len()..].trim_start();
                if let Some(after_sep) = rest.strip_prefix(['-', ':', '/', '|']) {
                    return (Some(*is_dom), after_sep.trim());
                }
            }
        }
    }
    (None, trimmed)
}

fn is_domestic_origin(origin: &str) -> bool {
    let (prefix_dom, stripped) = split_origin_prefix(origin);
    if let Some(dom) = prefix_dom {
        return dom;
    }
    let lower = stripped.to_lowercase();
    if lower.is_empty() || lower == "-" {
        return false;
    }
    const FOREIGN_OVERRIDES: &[&str] = &[
        "papua nugini",
        "papua new guinea",
    ];
    if FOREIGN_OVERRIDES.iter().any(|fw| lower.contains(fw)) {
        return false;
    }
    const DOMESTIC_KEYWORDS: &[&str] = &[
        "indonesia",
        "idn",
        "wni",
        "lokal",
        "domestik",
        "nusantara",
        "mataram",
        "praya",
        "selong",
        "lombok",
        "sumbawa",
        "bima",
        "dompu",
        "taliwang",
        "ntb",
        "ntt",
        "nusa tenggara",
        "kupang",
        "flores",
        "sumba",
        "bali",
        "denpasar",
        "badung",
        "gianyar",
        "ubud",
        "kuta",
        "singaraja",
        "jakarta",
        "bogor",
        "depok",
        "tangerang",
        "bekasi",
        "bandung",
        "cirebon",
        "tasikmalaya",
        "sukabumi",
        "karawang",
        "jawa",
        "semarang",
        "solo",
        "surakarta",
        "magelang",
        "purwokerto",
        "tegal",
        "pekalongan",
        "yogyakarta",
        "jogja",
        "sleman",
        "bantul",
        "surabaya",
        "malang",
        "sidoarjo",
        "gresik",
        "banyuwangi",
        "kediri",
        "banten",
        "serang",
        "cilegon",
        "aceh",
        "medan",
        "sumatera",
        "sumatra",
        "padang",
        "pekanbaru",
        "riau",
        "batam",
        "jambi",
        "bengkulu",
        "palembang",
        "bangka",
        "belitung",
        "lampung",
        "pontianak",
        "palangkaraya",
        "banjarmasin",
        "samarinda",
        "balikpapan",
        "tarakan",
        "kalimantan",
        "makassar",
        "manado",
        "palu",
        "kendari",
        "mamuju",
        "gorontalo",
        "sulawesi",
        "ambon",
        "maluku",
        "ternate",
        "papua",
        "jayapura",
        "sorong",
        "merauke",
        "nabire",
        "timika",
    ];
    let tokens: Vec<&str> = lower
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .collect();
    DOMESTIC_KEYWORDS.iter().any(|kw| {
        if kw.len() <= 4 {
            tokens.contains(kw)
        } else {
            lower.contains(kw)
        }
    })
}

fn normalize_origin_label(raw: &str, is_domestic: bool) -> String {
    let (prefix_dom, stripped) = split_origin_prefix(raw);
    if stripped.is_empty() || stripped == "-" {
        if prefix_dom == Some(true) {
            return "Indonesia".to_string();
        }
        return "Tidak Tercatat".to_string();
    }
    if is_domestic {
        let lower = stripped.to_lowercase();
        if lower == "idn"
            || lower == "wni"
            || lower == "lokal"
            || lower == "domestik"
            || lower == "indonesia"
        {
            return "Indonesia".to_string();
        }
        if let Some(canonical) = crate::excel::INDONESIAN_REGIONS
            .iter()
            .find(|r| r.eq_ignore_ascii_case(stripped))
        {
            return (*canonical).to_string();
        }
    } else if let Some(canonical) = crate::excel::ALL_WORLD_COUNTRIES
        .iter()
        .find(|c| c.eq_ignore_ascii_case(stripped))
    {
        return (*canonical).to_string();
    }
    stripped.to_string()
}

fn aggregate_origins(map: HashMap<String, (String, i64, usize)>) -> Vec<TourismOriginItem> {
    let mut list: Vec<(String, i64, usize)> = map.into_values().collect();
    list.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    list.into_iter()
        .enumerate()
        .map(|(idx, (origin_name, guest_count, reservation_count))| TourismOriginItem {
            rank: idx + 1,
            origin_name,
            guest_count,
            reservation_count,
        })
        .collect()
}

/// Generates monthly tourism statistics for Dinas Pariwisata Form 1 from 13-column reservations.
pub fn generate_tourism_report(
    conn: &Connection,
    unit: &str,
    month_str: &str,
    room_capacity: i64,
) -> Result<TourismFormReport, String> {
    let (year, month_num) = parse_year_month(month_str)?;
    let cap = room_capacity.max(1);
    let total_days = days_in_month(year, month_num);
    let month_start = NaiveDate::from_ymd_opt(year, month_num, 1)
        .ok_or_else(|| "Tanggal awal bulan tidak valid".to_string())?;
    let month_end = NaiveDate::from_ymd_opt(year, month_num, total_days)
        .ok_or_else(|| "Tanggal akhir bulan tidak valid".to_string())?;
    let start_str = month_start.format("%Y-%m-%d").to_string();
    let end_str = month_end.format("%Y-%m-%d").to_string();

    let unit_name: String = conn
        .query_row(
            "SELECT name FROM units WHERE code = ?1",
            params![unit],
            |r| r.get(0),
        )
        .unwrap_or_else(|_| format!("ANDA {}", unit.to_uppercase()));

    let mut stmt = conn
        .prepare(
            "SELECT date, COALESCE(total_person, 1), nationality, total_rooms, check_in_date, check_out_date, total_nights
             FROM room_reservations
             WHERE unit = ?1
               AND ((date BETWEEN ?2 AND ?3) OR (check_in_date <= ?3 AND check_out_date > ?2))",
        )
        .map_err(|e| format!("Query reservasi wisata gagal: {}", e))?;

    struct ResItem {
        ci: NaiveDate,
        co: NaiveDate,
        pax: i64,
        rooms: i64,
        nights: i64,
        is_domestic: bool,
        origin_label: String,
    }

    let rows = stmt
        .query_map(params![unit, start_str, end_str], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, Option<String>>(2)?,
                r.get::<_, i64>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, String>(5)?,
                r.get::<_, Option<i64>>(6)?,
            ))
        })
        .map_err(|e| format!("Eksekusi query wisata gagal: {}", e))?;

    let mut reservations: Vec<ResItem> = Vec::new();
    for row_res in rows {
        let (date_s, raw_pax, nat_opt, raw_rooms, ci_s, co_s, nights_opt) =
            row_res.map_err(|e| e.to_string())?;

        let fallback_d = NaiveDate::parse_from_str(&date_s, "%Y-%m-%d").unwrap_or(month_start);
        let ci = NaiveDate::parse_from_str(&ci_s, "%Y-%m-%d").unwrap_or(fallback_d);
        let mut co = NaiveDate::parse_from_str(&co_s, "%Y-%m-%d").unwrap_or_else(|_| {
            let n = nights_opt.unwrap_or(1).max(1);
            ci + chrono::Duration::days(n)
        });
        if co <= ci {
            let n = nights_opt.unwrap_or(1).max(1);
            co = ci + chrono::Duration::days(n);
        }
        let nights = nights_opt
            .filter(|&n| n > 0)
            .unwrap_or_else(|| (co - ci).num_days().max(1))
            .max(1);
        let pax = raw_pax.max(1);
        let rooms = raw_rooms.max(1);
        let raw_nat = nat_opt.as_deref().unwrap_or("");
        let is_domestic = is_domestic_origin(raw_nat);
        let origin_label = normalize_origin_label(raw_nat, is_domestic);

        reservations.push(ResItem {
            ci,
            co,
            pax,
            rooms,
            nights,
            is_domestic,
            origin_label,
        });
    }

    let week_ranges: [(u32, u32, u32); 4] = [
        (1, 1, 7),
        (2, 8, 14),
        (3, 15, 21),
        (4, 22, total_days),
    ];

    let mut weekly_rows: Vec<TourismWeeklyRow> = Vec::with_capacity(4);
    let mut weekly_total = TourismWeeklyRow {
        week_number: 0,
        label: "TOTAL".to_string(),
        days_count: total_days as i64,
        domestic_check_in: 0,
        domestic_check_out: 0,
        foreign_check_in: 0,
        foreign_check_out: 0,
        room_occupied: 0,
        room_vacant: 0,
        bed_check_out: 0,
        bed_occupied: 0,
    };

    for (w_num, d_start, d_end) in week_ranges {
        let w_start = NaiveDate::from_ymd_opt(year, month_num, d_start).unwrap();
        let w_end = NaiveDate::from_ymd_opt(year, month_num, d_end).unwrap();
        let days_count = (d_end - d_start + 1) as i64;

        let mut dom_ci = 0_i64;
        let mut dom_co = 0_i64;
        let mut intl_ci = 0_i64;
        let mut intl_co = 0_i64;
        let mut room_occ = 0_i64;
        let mut bed_occ = 0_i64;

        for res in &reservations {
            if res.ci >= w_start && res.ci <= w_end {
                if res.is_domestic {
                    dom_ci += res.pax;
                } else {
                    intl_ci += res.pax;
                }
            }
            if res.co >= w_start && res.co <= w_end {
                if res.is_domestic {
                    dom_co += res.pax;
                } else {
                    intl_co += res.pax;
                }
            }
        }

        for d in d_start..=d_end {
            let cur = NaiveDate::from_ymd_opt(year, month_num, d).unwrap();
            for res in &reservations {
                if res.ci <= cur && cur < res.co {
                    room_occ += res.rooms;
                    bed_occ += res.pax;
                }
            }
        }

        let room_vac = (cap * days_count - room_occ).max(0);
        let bed_co = dom_co + intl_co;

        weekly_total.domestic_check_in += dom_ci;
        weekly_total.domestic_check_out += dom_co;
        weekly_total.foreign_check_in += intl_ci;
        weekly_total.foreign_check_out += intl_co;
        weekly_total.room_occupied += room_occ;
        weekly_total.room_vacant += room_vac;
        weekly_total.bed_check_out += bed_co;
        weekly_total.bed_occupied += bed_occ;

        weekly_rows.push(TourismWeeklyRow {
            week_number: w_num,
            label: format!("Minggu {} ({}-{})", w_num, d_start, d_end),
            days_count,
            domestic_check_in: dom_ci,
            domestic_check_out: dom_co,
            foreign_check_in: intl_ci,
            foreign_check_out: intl_co,
            room_occupied: room_occ,
            room_vacant: room_vac,
            bed_check_out: bed_co,
            bed_occupied: bed_occ,
        });
    }

    let mut total_reservations = 0_usize;
    let mut domestic_guests = 0_i64;
    let mut foreign_guests = 0_i64;
    let mut total_stay_nights_sum = 0_i64;
    let mut foreign_map: HashMap<String, (String, i64, usize)> = HashMap::new();
    let mut domestic_map: HashMap<String, (String, i64, usize)> = HashMap::new();

    for res in &reservations {
        if res.ci >= month_start && res.ci <= month_end {
            total_reservations += 1;
            total_stay_nights_sum += res.nights;
            let key = res.origin_label.to_lowercase();
            let target_map = if res.is_domestic {
                domestic_guests += res.pax;
                &mut domestic_map
            } else {
                foreign_guests += res.pax;
                &mut foreign_map
            };
            let entry = target_map
                .entry(key)
                .or_insert_with(|| (res.origin_label.clone(), 0, 0));
            entry.1 += res.pax;
            entry.2 += 1;
        }
    }

    let total_guests = domestic_guests + foreign_guests;
    let total_room_nights = weekly_total.room_occupied;
    let total_bed_nights = weekly_total.bed_occupied;
    let available_room_nights = (cap * total_days as i64) as f64;
    let occupancy_rate_pct = if available_room_nights > 0.0 {
        (total_room_nights as f64 / available_room_nights) * 100.0
    } else {
        0.0
    };
    let avg_stay_nights = if total_reservations > 0 {
        total_stay_nights_sum as f64 / total_reservations as f64
    } else {
        0.0
    };
    let stay_category = if total_reservations == 0 {
        "BELUM ADA DATA".to_string()
    } else if avg_stay_nights <= 2.0 {
        "1-2 MALAM".to_string()
    } else if avg_stay_nights <= 3.0 {
        "3 MALAM".to_string()
    } else {
        "> 3 MALAM".to_string()
    };

    Ok(TourismFormReport {
        unit: unit.to_string(),
        accommodation_name: unit_name,
        accommodation_type: "Hotel Non Bintang".to_string(),
        room_capacity: cap,
        month: month_str.to_string(),
        month_name: indonesian_month_name(month_num).to_uppercase(),
        year,
        days_in_month: total_days,
        total_reservations,
        total_guests,
        domestic_guests,
        foreign_guests,
        total_room_nights,
        total_bed_nights,
        occupancy_rate_pct,
        avg_stay_nights,
        stay_category,
        weekly_rows,
        weekly_total,
        foreign_origins: aggregate_origins(foreign_map),
        domestic_origins: aggregate_origins(domestic_map),
    })
}

#[cfg(test)]
mod tests {
    use super::{is_domestic_origin, normalize_origin_label};

    #[test]
    fn test_world_countries_and_domestic_regions_detection() {
        for &country in crate::excel::ALL_WORLD_COUNTRIES {
            assert!(
                !is_domestic_origin(country),
                "Negara '{}' dari ALL_WORLD_COUNTRIES seharusnya terdeteksi sebagai Mancanegara",
                country
            );
        }

        let extra_foreign = [
            "Papua New Guinea",
            "Solomon Islands",
            "United States",
            "United Kingdom",
            "Netherlands",
            "France",
            "Germany",
            "New Zealand",
            "Timor-Leste",
            "Skotlandia",
            "Puerto Riko",
            "INTL - Papua",
        ];
        for country in extra_foreign {
            assert!(
                !is_domestic_origin(country),
                "Negara/wilayah '{}' seharusnya terdeteksi sebagai Mancanegara",
                country
            );
        }

        for &region in crate::excel::INDONESIAN_REGIONS {
            assert!(
                is_domestic_origin(region),
                "Daerah '{}' dari INDONESIAN_REGIONS seharusnya terdeteksi sebagai Domestik (Nusantara)",
                region
            );
        }

        let extra_domestic = ["IDN - Kota Baru", "Domestik - Kupang", "WNI - Surabaya"];
        for region in extra_domestic {
            assert!(
                is_domestic_origin(region),
                "Daerah '{}' seharusnya terdeteksi sebagai Domestik (Nusantara)",
                region
            );
        }

        assert_eq!(normalize_origin_label("IDN - Kota Baru", true), "Kota Baru");
        assert_eq!(normalize_origin_label("INTL - Papua Nugini", false), "Papua Nugini");
        assert_eq!(normalize_origin_label("Puerto Riko", false), "Puerto Riko");
        assert_eq!(normalize_origin_label("AUSTRALIA", false), "Australia");
        assert_eq!(normalize_origin_label("mataram", true), "Mataram");
    }
}
