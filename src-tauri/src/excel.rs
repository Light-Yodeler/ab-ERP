use crate::optimizer::{RoomAnomaly, RoomConflict, TransactionItem};
use calamine::{open_workbook_auto, Data, DataType, Reader};
use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::path::Path;
use umya_spreadsheet::structs::{DataValidation, DataValidationValues, DataValidations};
use umya_spreadsheet::Workbook;

pub const INDONESIAN_REGIONS: &[&str] = &[
    "Mataram",
    "Praya",
    "Selong",
    "Lombok Barat",
    "Lombok Tengah",
    "Lombok Timur",
    "Lombok Utara",
    "Sumbawa",
    "Bima",
    "Nusa Tenggara Barat",
    "Nusa Tenggara Timur",
    "Bali",
    "Denpasar",
    "Jakarta",
    "DKI Jakarta",
    "Bogor",
    "Depok",
    "Tangerang",
    "Bekasi",
    "Bandung",
    "Jawa Barat",
    "Banten",
    "Semarang",
    "Solo",
    "Jawa Tengah",
    "DI Yogyakarta",
    "Surabaya",
    "Malang",
    "Jawa Timur",
    "Aceh",
    "Medan",
    "Sumatera Utara",
    "Sumatera Barat",
    "Riau",
    "Kepulauan Riau",
    "Jambi",
    "Sumatera Selatan",
    "Kepulauan Bangka Belitung",
    "Bengkulu",
    "Lampung",
    "Kalimantan Barat",
    "Kalimantan Tengah",
    "Kalimantan Selatan",
    "Kalimantan Timur",
    "Kalimantan Utara",
    "Makassar",
    "Sulawesi Selatan",
    "Sulawesi Utara",
    "Gorontalo",
    "Sulawesi Tengah",
    "Sulawesi Barat",
    "Sulawesi Tenggara",
    "Maluku",
    "Maluku Utara",
    "Papua",
    "Papua Barat",
    "Papua Barat Daya",
    "Papua Selatan",
    "Papua Tengah",
    "Papua Pegunungan",
];

pub const ALL_WORLD_COUNTRIES: &[&str] = &[
    "Afganistan",
    "Afrika Selatan",
    "Afrika Tengah",
    "Albania",
    "Aljazair",
    "Amerika Serikat",
    "Andorra",
    "Angola",
    "Antigua dan Barbuda",
    "Arab Saudi",
    "Argentina",
    "Armenia",
    "Australia",
    "Austria",
    "Azerbaijan",
    "Bahama",
    "Bahrain",
    "Bangladesh",
    "Barbados",
    "Belanda",
    "Belarus",
    "Belgia",
    "Belize",
    "Benin",
    "Bhutan",
    "Bolivia",
    "Bosnia dan Herzegovina",
    "Botswana",
    "Brasil",
    "Brunei Darussalam",
    "Bulgaria",
    "Burkina Faso",
    "Burundi",
    "Ceko",
    "Chad",
    "Chili",
    "Denmark",
    "Djibouti",
    "Dominika",
    "Ekuador",
    "El Salvador",
    "Eritrea",
    "Estonia",
    "Eswatini",
    "Etiopia",
    "Fiji",
    "Filipina",
    "Finlandia",
    "Gabon",
    "Gambia",
    "Georgia",
    "Ghana",
    "Grenada",
    "Guatemala",
    "Guinea",
    "Guinea Khatulistiwa",
    "Guinea-Bissau",
    "Guyana",
    "Haiti",
    "Honduras",
    "Hong Kong",
    "Hungaria",
    "India",
    "Inggris",
    "Irak",
    "Iran",
    "Irlandia",
    "Islandia",
    "Israel",
    "Italia",
    "Jamaika",
    "Jepang",
    "Jerman",
    "Kamboja",
    "Kamerun",
    "Kanada",
    "Kazakhstan",
    "Kenya",
    "Kepulauan Marshall",
    "Kepulauan Solomon",
    "Kirgizstan",
    "Kiribati",
    "Kolombia",
    "Komoro",
    "Kongo",
    "Korea Selatan",
    "Korea Utara",
    "Kosovo",
    "Kosta Rika",
    "Kroasia",
    "Kuba",
    "Kuwait",
    "Laos",
    "Latvia",
    "Lebanon",
    "Lesotho",
    "Liberia",
    "Libya",
    "Liechtenstein",
    "Lituania",
    "Luksemburg",
    "Madagaskar",
    "Makau",
    "Makedonia Utara",
    "Maladewa",
    "Malawi",
    "Malaysia",
    "Mali",
    "Malta",
    "Maroko",
    "Mauritania",
    "Mauritius",
    "Meksiko",
    "Mesir",
    "Mikronesia",
    "Moldova",
    "Monako",
    "Mongolia",
    "Montenegro",
    "Mozambik",
    "Myanmar",
    "Namibia",
    "Nauru",
    "Nepal",
    "Niger",
    "Nigeria",
    "Nikaragua",
    "Norwegia",
    "Oman",
    "Pakistan",
    "Palau",
    "Palestina",
    "Panama",
    "Pantai Gading",
    "Papua Nugini",
    "Paraguay",
    "Peru",
    "Polandia",
    "Portugal",
    "Prancis",
    "Qatar",
    "Republik Demokratik Kongo",
    "Republik Dominika",
    "Rumania",
    "Rusia",
    "Rwanda",
    "Saint Kitts dan Nevis",
    "Saint Lucia",
    "Saint Vincent dan Grenadine",
    "Samoa",
    "San Marino",
    "Sao Tome dan Principe",
    "Selandia Baru",
    "Senegal",
    "Serbia",
    "Seychelles",
    "Sierra Leone",
    "Singapura",
    "Siprus",
    "Slovakia",
    "Slovenia",
    "Somalia",
    "Spanyol",
    "Sri Lanka",
    "Sudan",
    "Sudan Selatan",
    "Suriah",
    "Suriname",
    "Swedia",
    "Swiss",
    "Taiwan",
    "Tajikistan",
    "Tanjung Verde",
    "Tanzania",
    "Thailand",
    "Timor Leste",
    "Tiongkok",
    "Togo",
    "Tonga",
    "Trinidad dan Tobago",
    "Tunisia",
    "Turki",
    "Turkmenistan",
    "Tuvalu",
    "Uganda",
    "Ukraina",
    "Uni Emirat Arab",
    "Uruguay",
    "Uzbekistan",
    "Vanuatu",
    "Vatikan",
    "Venezuela",
    "Vietnam",
    "Yaman",
    "Yordania",
    "Yunani",
    "Zambia",
    "Zimbabwe",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExcelAnalysis {
    pub file_path: String,
    pub sheet_name: String,
    pub data_start_row: usize,
    pub data_end_row: usize,
    pub total_rows: usize,
    pub total_sum: f64,
    pub target_detected: f64,
    pub target_formula: Option<String>,
    pub items: Vec<TransactionItem>,
    #[serde(default)]
    pub format_type: String,
    #[serde(default)]
    pub room_conflicts: Vec<RoomConflict>,
    #[serde(default)]
    pub room_anomalies: Vec<RoomAnomaly>,
}

fn naive_date_to_excel_serial(d: NaiveDate) -> Option<f64> {
    let epoch = NaiveDate::from_ymd_opt(1899, 12, 30)?;
    Some((d - epoch).num_days() as f64)
}

fn format_naive_dd_mm_yy(d: NaiveDate) -> String {
    format!("{:02}/{:02}/{:02}", d.day(), d.month(), d.year().rem_euclid(100))
}

fn excel_serial_to_date_str(serial: f64) -> Option<String> {
    if !serial.is_finite() || serial < 1.0 {
        return None;
    }
    let days = serial.floor() as i64;
    let adjusted_days = if days > 60 { days - 2 } else { days - 1 };
    let epoch = NaiveDate::from_ymd_opt(1900, 1, 1)?;
    let date = epoch.checked_add_signed(chrono::Duration::days(adjusted_days))?;
    Some(format_naive_dd_mm_yy(date))
}

fn format_calamine_date_cell(cell: Option<&Data>) -> Option<String> {
    let c = cell?;
    let formatted = match c {
        Data::DateTime(dt) => {
            excel_serial_to_date_str(dt.as_f64()).unwrap_or_else(|| c.to_string())
        }
        Data::Float(f) => excel_serial_to_date_str(*f).unwrap_or_else(|| format!("{f}")),
        Data::Int(i) => excel_serial_to_date_str(*i as f64).unwrap_or_else(|| format!("{i}")),
        Data::String(s) => {
            let trimmed = s.trim();
            crate::excel_importer::parse_date_string(trimmed)
                .map(format_naive_dd_mm_yy)
                .unwrap_or_else(|| trimmed.to_string())
        }
        Data::DateTimeIso(iso) => {
            let trimmed = iso.trim();
            crate::excel_importer::parse_date_string(trimmed)
                .map(format_naive_dd_mm_yy)
                .unwrap_or_else(|| trimmed.to_string())
        }
        _ => c.to_string(),
    };
    let clean = formatted.trim();
    if clean.is_empty() || clean == "-" {
        None
    } else {
        Some(clean.to_string())
    }
}

fn is_13_col_header(col_a: &str, col_b: &str, col_l: &str) -> bool {
    (col_a.contains("date") || col_a.contains("tgl") || col_a.contains("tanggal"))
        && (col_b.contains("name") || col_b.contains("nama"))
        && (col_l.contains("payment") || col_l.contains("total") || col_l.contains("bayar"))
}

pub fn analyze_excel_file<P: AsRef<Path>>(path: P) -> Result<ExcelAnalysis, String> {
    let p = path.as_ref();
    let mut workbook = open_workbook_auto(p).map_err(|e| format!("Gagal membuka file Excel: {e}"))?;
    let sheet_names = workbook.sheet_names();
    if sheet_names.is_empty() {
        return Err("File Excel tidak memiliki lembar kerja (worksheet).".to_string());
    }
    let sheet_name = sheet_names[0].clone();
    let range = workbook
        .worksheet_range(&sheet_name)
        .map_err(|e| format!("Gagal membaca sheet '{sheet_name}': {e}"))?;

    let (max_r, max_c) = range.get_size();
    if max_r == 0 || max_c == 0 {
        return Err("Lembar kerja kosong.".to_string());
    }

    // Header inspection on row 0 (Excel Row 1) to determine template format
    let is_13_col = if max_c >= 12 {
        let col_a = range.get((0, 0)).map(|d| d.to_string().trim().to_lowercase()).unwrap_or_default();
        let col_b = range.get((0, 1)).map(|d| d.to_string().trim().to_lowercase()).unwrap_or_default();
        let col_l = range.get((0, 11)).map(|d| d.to_string().trim().to_lowercase()).unwrap_or_default();
        is_13_col_header(&col_a, &col_b, &col_l)
    } else {
        false
    };

    let (date_col, amount_col, name_col, format_type) = if is_13_col {
        (0, 11, Some(1), "RoomReservation13Col".to_string())
    } else {
        (0, 1, None, "Legacy2Col".to_string())
    };

    let mut data_start_row = 2;
    let mut data_end_row = 1;
    let mut items = Vec::new();
    let mut total_from_sum_cell: Option<f64> = None;
    let mut target_detected: Option<f64> = None;
    let mut target_formula: Option<String> = None;
    let mut prev_row_was_total = false;

    for r in 0..max_r {
        let excel_row = r + 1;
        let cell_a = range.get((r, date_col));
        let cell_amt = range.get((r, amount_col));
        let cell_name = name_col.and_then(|c| range.get((r, c)));

        if excel_row == 1 {
            continue;
        }

        let a_str = match cell_a {
            Some(Data::String(s)) => s.trim().to_lowercase(),
            _ => String::new(),
        };

        if a_str.contains("jumlah dibagi") || a_str.contains("target") {
            if let Some(b) = cell_amt {
                if let Some(val) = b.as_f64() {
                    target_detected = Some(val);
                    target_formula = Some("Jumlah dibagi 2".to_string());
                }
            }
            prev_row_was_total = false;
            continue;
        }

        if a_str == "jumlah" || a_str == "total" || a_str.contains("subtotal") {
            if let Some(b) = cell_amt {
                if let Some(val) = b.as_f64() {
                    total_from_sum_cell = Some(val);
                }
            }
            prev_row_was_total = true;
            continue;
        }

        // Row 79 footer pattern: previous row was TOTAL and this row Col A is empty, but Col amount has numeric value
        if prev_row_was_total && a_str.is_empty() {
            if let Some(b) = cell_amt {
                if let Some(val) = b.as_f64() {
                    if let Some(tot) = total_from_sum_cell {
                        if (val - tot / 2.0).abs() < 1.0 {
                            target_detected = Some(val);
                            target_formula = Some("=TOTAL / 2".to_string());
                        }
                    } else {
                        target_detected = Some(val);
                    }
                }
            }
            prev_row_was_total = false;
            continue;
        }
        prev_row_was_total = false;

        let row_amount: Option<f64> = cell_amt.and_then(|b| b.as_f64());

        if let Some(amount) = row_amount {
            if amount != 0.0 {
                let mut raw_date: Option<f64> = None;
                let mut date_str = String::new();

                if let Some(a) = cell_a {
                    match a {
                        Data::DateTime(dt) => {
                            raw_date = Some(dt.as_f64());
                            if let Some(s) = raw_date.and_then(excel_serial_to_date_str) {
                                date_str = s;
                            } else {
                                date_str = a.to_string();
                            }
                        }
                        Data::Float(f) => {
                            raw_date = Some(*f);
                            if let Some(s) = excel_serial_to_date_str(*f) {
                                date_str = s;
                            } else {
                                date_str = format!("{f}");
                            }
                        }
                        Data::Int(i) => {
                            raw_date = Some(*i as f64);
                            if let Some(s) = excel_serial_to_date_str(*i as f64) {
                                date_str = s;
                            } else {
                                date_str = format!("{i}");
                            }
                        }
                        Data::String(s) => {
                            let trimmed = s.trim();
                            if let Some(d) = crate::excel_importer::parse_date_string(trimmed) {
                                raw_date = naive_date_to_excel_serial(d);
                                date_str = format_naive_dd_mm_yy(d);
                            } else {
                                date_str = trimmed.to_string();
                            }
                        }
                        Data::DateTimeIso(iso) => {
                            let trimmed = iso.trim();
                            if let Some(d) = crate::excel_importer::parse_date_string(trimmed) {
                                raw_date = naive_date_to_excel_serial(d);
                                date_str = format_naive_dd_mm_yy(d);
                            } else {
                                date_str = trimmed.to_string();
                            }
                        }
                        _ => {
                            date_str = a.to_string();
                        }
                    }
                }

                if date_str.is_empty() {
                    date_str = format!("Baris {excel_row}");
                }

                let guest_name = cell_name.map(|c| c.to_string().trim().to_string()).filter(|s| !s.is_empty());
                let details = if is_13_col {
                    let total_person = range.get((r, 2)).and_then(|c| c.as_f64()).map(|f| f as i64);
                    let booking_number = range.get((r, 3)).map(|c| c.to_string().trim().to_string()).filter(|s| !s.is_empty() && s != "-");
                    let nationality = range.get((r, 4)).map(|c| c.to_string().trim().to_string()).filter(|s| !s.is_empty() && s != "-");
                    let room_number = range.get((r, 5)).map(|c| c.to_string().trim().to_string()).filter(|s| !s.is_empty() && s != "-");
                    let total_rooms = range.get((r, 6)).and_then(|c| c.as_f64()).map(|f| f as i64).or(Some(1));
                    let check_in_date = format_calamine_date_cell(range.get((r, 7)));
                    let check_out_date = format_calamine_date_cell(range.get((r, 8)));
                    let total_nights = range.get((r, 9)).and_then(|c| c.as_f64()).map(|f| f as i64);
                    let rate = range.get((r, 10)).and_then(|c| c.as_f64());
                    let remarks = range.get((r, 12)).map(|c| c.to_string().trim().to_string()).filter(|s| !s.is_empty() && s != "-");

                    Some(crate::optimizer::ReservationDetails {
                        total_person,
                        booking_number,
                        nationality,
                        room_number,
                        total_rooms,
                        check_in_date,
                        check_out_date,
                        total_nights,
                        rate,
                        remarks,
                    })
                } else {
                    None
                };

                if items.is_empty() {
                    data_start_row = excel_row;
                }
                data_end_row = excel_row;

                items.push(TransactionItem {
                    original_row: excel_row,
                    date_str,
                    raw_date,
                    amount,
                    guest_name,
                    details,
                });
            }
        }
    }

    if items.is_empty() {
        return Err("Tidak ditemukan baris transaksi valid dalam berkas.".to_string());
    }

    let calculated_sum: f64 = items.iter().map(|item| item.amount).sum();
    let final_total = total_from_sum_cell.unwrap_or(calculated_sum);
    let final_target = target_detected.unwrap_or(final_total / 2.0);
    let final_formula = target_formula.or_else(|| Some("TOTAL / 2 (50%)".to_string()));
    let (room_conflicts, room_anomalies) = if is_13_col {
        crate::optimizer::analyze_room_issues(&items)
    } else {
        (Vec::new(), Vec::new())
    };

    Ok(ExcelAnalysis {
        file_path: p.to_string_lossy().to_string(),
        sheet_name,
        data_start_row,
        data_end_row,
        total_rows: items.len(),
        total_sum: final_total,
        target_detected: final_target,
        target_formula: final_formula,
        items,
        format_type,
        room_conflicts,
        room_anomalies,
    })
}

/// Checks if the destination file is locked by Excel or another process
fn check_file_writable<P: AsRef<Path>>(path: P) -> Result<(), String> {
    let p = path.as_ref();
    if p.exists() {
        match OpenOptions::new().write(true).open(p) {
            Ok(_) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => Err(
                "File sedang dibuka di Microsoft Excel atau aplikasi lain. Harap tutup file tersebut terlebih dahulu atau simpan dengan nama baru.".to_string(),
            ),
            Err(e) => Err(format!("Gagal mengakses file: {e}")),
        }
    } else {
        Ok(())
    }
}

pub fn export_solution_to_excel<P: AsRef<Path>, Q: AsRef<Path>>(
    source_path: P,
    destination_path: Q,
    items: &[TransactionItem],
    selected_indices: &[usize],
) -> Result<String, String> {
    let src = source_path.as_ref();
    let dest = destination_path.as_ref();

    if selected_indices.is_empty() {
        return Err("Tidak ada baris yang dipilih untuk diekspor.".to_string());
    }

    check_file_writable(dest)?;

    let mut book = umya_spreadsheet::reader::xlsx::read(src)
        .map_err(|e| format!("Gagal memuat workbook dengan umya-spreadsheet: {e}"))?;

    let is_13_col = {
        let sheet = book
            .sheet(0)
            .map_err(|e| format!("Tidak dapat menemukan sheet pertama di workbook: {e}"))?;
        let col_a = sheet.cell("A1").map(|c| c.value().trim().to_lowercase()).unwrap_or_default();
        let col_b = sheet.cell("B1").map(|c| c.value().trim().to_lowercase()).unwrap_or_default();
        let col_l = sheet.cell("L1").map(|c| c.value().trim().to_lowercase()).unwrap_or_default();
        is_13_col_header(&col_a, &col_b, &col_l)
    };

    if !is_13_col {
        let sheet = book
            .sheet_mut(0)
            .map_err(|e| format!("Tidak dapat menemukan sheet pertama di workbook: {e}"))?;

        sheet.cell_mut("E1").set_value("tgl");
        sheet.cell_mut("F1").set_value("Total");

        let clear_limit = (items.len() as u32 + 10).max(300);
        for r in 2..=clear_limit {
            sheet.cell_mut(format!("E{r}")).set_value("");
            sheet.cell_mut(format!("F{r}")).set_value("");
        }

        let mut out_row: u32 = 2;
        let mut written_rows: usize = 0;
        let mut sum_eval: f64 = 0.0;

        for &idx in selected_indices {
            if idx >= items.len() {
                continue;
            }
            let item = &items[idx];
            sum_eval += item.amount;

            let cell_e = sheet.cell_mut(format!("E{out_row}"));
            if let Some(raw) = item.raw_date {
                cell_e.set_value_number(raw);
                cell_e.style_mut().number_format_mut().set_format_code("dd/mm/yy");
            } else {
                cell_e.set_value(&item.date_str);
            }

            let cell_f = sheet.cell_mut(format!("F{out_row}"));
            cell_f.set_value_number(item.amount);
            cell_f.style_mut().number_format_mut().set_format_code("#,##0");
            out_row += 1;
            written_rows += 1;
        }

        if written_rows == 0 {
            return Err("Tidak ada baris valid untuk diekspor.".to_string());
        }

        let last_data_row = out_row - 1;

        let cell_sum_label = sheet.cell_mut(format!("E{out_row}"));
        cell_sum_label.set_value("Jumlah");
        cell_sum_label.style_mut().font_mut().set_bold(true);

        let sum_formula = format!("SUM(F2:F{last_data_row})");
        let cell_sum_val = sheet.cell_mut(format!("F{out_row}"));
        cell_sum_val.set_formula(sum_formula);
        cell_sum_val.set_formula_result_number(sum_eval);
        cell_sum_val.style_mut().number_format_mut().set_format_code("#,##0");
        cell_sum_val.style_mut().font_mut().set_bold(true);

        umya_spreadsheet::writer::xlsx::write(&book, dest)
            .map_err(|e| format!("Gagal menyimpan file Excel: {e}"))?;

        Ok(format!(
            "Berhasil menulis {} baris ke kolom E & F pada {}",
            written_rows,
            dest.to_string_lossy()
        ))
    } else {
        // Each snapshot cell stores (display_or_text, parsed_numeric_value)
        let mut snapshot_rows: Vec<Vec<(String, Option<f64>)>> = Vec::new();
        {
            let sheet = book.sheet(0).map_err(|e| e.to_string())?;
            for &idx in selected_indices {
                if idx >= items.len() {
                    continue;
                }
                let orig_r = items[idx].original_row as u32;
                let mut row_cells = Vec::new();
                for c in 1..=13 {
                    let cell = sheet.cell((c, orig_r));
                    let val_str = cell.map(|cell_ref| cell_ref.value().to_string()).unwrap_or_default();
                    let mut val_num = cell.and_then(|cell_ref| cell_ref.value_number());
                    if val_num.is_none() {
                        if matches!(c, 1 | 8 | 9) && !val_str.trim().is_empty() {
                            val_num = crate::excel_importer::parse_date_string(&val_str)
                                .and_then(naive_date_to_excel_serial);
                        } else if matches!(c, 3 | 7 | 10 | 11 | 12) {
                            val_num = val_str.replace(',', "").trim().parse::<f64>().ok();
                        }
                    }
                    row_cells.push((val_str, val_num));
                }
                snapshot_rows.push(row_cells);
            }
        }
        if snapshot_rows.is_empty() {
            return Err("Tidak ada baris valid untuk diekspor.".to_string());
        }

        let extract_row_num = |cell: &(String, Option<f64>)| -> f64 {
            cell.1.unwrap_or_else(|| {
                cell.0.replace(',', "").trim().parse::<f64>().unwrap_or(0.0)
            })
        };

        let write_13col_snapshot = |sheet: &mut umya_spreadsheet::Worksheet,
                                    header_labels: &[String],
                                    col_offset: u32| {
            let col_letter = |offset_1based: u32| -> String {
                let c = offset_1based + col_offset;
                let idx = (c - 1) as usize;
                if idx < 26 {
                    ((b'A' + idx as u8) as char).to_string()
                } else {
                    format!(
                        "{}{}",
                        (b'A' + (idx / 26 - 1) as u8) as char,
                        (b'A' + (idx % 26) as u8) as char
                    )
                }
            };

            let date_col = col_letter(1);
            let pax_col = col_letter(3);
            let rms_col = col_letter(7);
            let ci_col = col_letter(8);
            let nts_col = col_letter(10);
            let pay_col = col_letter(12);

            for (idx, text) in header_labels.iter().enumerate() {
                let col_idx = (idx as u32) + 1 + col_offset;
                let cell = sheet.cell_mut((col_idx, 1));
                cell.set_value(text);
                cell.style_mut().font_mut().set_bold(true);
            }

            let mut out_r: u32 = 2;
            let mut sum_pax: f64 = 0.0;
            let mut sum_rooms: f64 = 0.0;
            let mut sum_nights: f64 = 0.0;
            let mut sum_eval: f64 = 0.0;

            for row_data in &snapshot_rows {
                let date_num = row_data[0].1;
                let rooms_num = extract_row_num(&row_data[6]).max(1.0);
                let ci_num = row_data[7].1;
                let co_num = row_data[8].1;
                let raw_nights = extract_row_num(&row_data[9]);
                let nights_num = if raw_nights > 0.0 {
                    raw_nights
                } else if let (Some(base_ci), Some(co_n)) = (ci_num.or(date_num), co_num) {
                    if co_n > base_ci {
                        (co_n - base_ci).round().max(1.0)
                    } else {
                        0.0
                    }
                } else {
                    0.0
                };
                let pay_num = extract_row_num(&row_data[11]);

                for (i, (val_str, val_num)) in row_data.iter().enumerate() {
                    let col_idx = (i as u32) + 1 + col_offset;
                    let cell = sheet.cell_mut((col_idx, out_r));

                    if i == 7 {
                        // Col H (Date Check in): use live formula =A{row} unless check-in differs from booking DATE
                        let matches_booking_date = date_num
                            .map(|d| {
                                val_num
                                    .map(|v| (v - d).abs() < 0.5)
                                    .unwrap_or_else(|| val_str.trim().is_empty())
                            })
                            .unwrap_or(false);
                        if let (true, Some(d_num)) = (matches_booking_date, date_num) {
                            cell.set_formula(format!("{date_col}{out_r}"));
                            cell.set_formula_result_number(d_num);
                            cell.style_mut().number_format_mut().set_format_code("dd/mm/yy");
                            continue;
                        }
                    } else if i == 8 {
                        // Col I (Date Check out): use live formula =H{row}+J{row} when consistent
                        let eff_ci = ci_num.or(date_num);
                        if let (Some(base_ci), true) = (eff_ci, nights_num > 0.0) {
                            let expected_co = base_ci + nights_num;
                            let matches_sum = val_num
                                .map(|v| (v - expected_co).abs() < 0.5)
                                .unwrap_or_else(|| val_str.trim().is_empty());
                            if matches_sum {
                                cell.set_formula(format!("{ci_col}{out_r}+{nts_col}{out_r}"));
                                cell.set_formula_result_number(expected_co);
                                cell.style_mut().number_format_mut().set_format_code("dd/mm/yy");
                                continue;
                            }
                        }
                    } else if i == 9 {
                        // Col J (Total Night): fill inferred numeric nights if blank/non-positive
                        if val_num.unwrap_or(0.0) <= 0.0 && nights_num > 0.0 {
                            cell.set_value_number(nights_num);
                            continue;
                        }
                    } else if i == 10 {
                        // Col K (Rate): keep live formula =IF(J{r}*G{r}>0,L{r}/J{r}/G{r},0)
                        let denom = nights_num * rooms_num;
                        let rate_val = if denom > 0.0 {
                            pay_num / denom
                        } else {
                            val_num.unwrap_or(0.0)
                        };
                        cell.set_formula(format!(
                            "IF({nts_col}{out_r}*{rms_col}{out_r}>0,{pay_col}{out_r}/{nts_col}{out_r}/{rms_col}{out_r},0)"
                        ));
                        cell.set_formula_result_number(rate_val);
                        cell.style_mut().number_format_mut().set_format_code("#,##0");
                        continue;
                    }

                    if let Some(num) = val_num {
                        cell.set_value_number(*num);
                        if i == 0 || i == 7 || i == 8 {
                            cell.style_mut()
                                .number_format_mut()
                                .set_format_code("dd/mm/yy");
                        } else if i == 11 {
                            cell.style_mut()
                                .number_format_mut()
                                .set_format_code("#,##0");
                        }
                    } else {
                        cell.set_value(val_str);
                    }
                }
                sum_pax += extract_row_num(&row_data[2]);
                sum_rooms += extract_row_num(&row_data[6]);
                sum_nights += nights_num;
                sum_eval += pay_num;
                out_r += 1;
            }

            let last_r = out_r - 1;

            let cell_tot = sheet.cell_mut((1 + col_offset, out_r));
            cell_tot.set_value("TOTAL");
            cell_tot.style_mut().font_mut().set_bold(true);

            let c_pax = sheet.cell_mut((3 + col_offset, out_r));
            c_pax.set_formula(format!("SUM({pax_col}2:{pax_col}{last_r})"));
            c_pax.set_formula_result_number(sum_pax);
            c_pax.style_mut().font_mut().set_bold(true);

            let c_rms = sheet.cell_mut((7 + col_offset, out_r));
            c_rms.set_formula(format!("SUM({rms_col}2:{rms_col}{last_r})"));
            c_rms.set_formula_result_number(sum_rooms);
            c_rms.style_mut().font_mut().set_bold(true);

            let c_nts = sheet.cell_mut((10 + col_offset, out_r));
            c_nts.set_formula(format!("SUM({nts_col}2:{nts_col}{last_r})"));
            c_nts.set_formula_result_number(sum_nights);
            c_nts.style_mut().font_mut().set_bold(true);

            let cell_l = sheet.cell_mut((12 + col_offset, out_r));
            cell_l.set_formula(format!("SUM({pay_col}2:{pay_col}{last_r})"));
            cell_l.set_formula_result_number(sum_eval);
            cell_l.style_mut().number_format_mut().set_format_code("#,##0");
            cell_l.style_mut().font_mut().set_bold(true);

            let cell_t = sheet.cell_mut((12 + col_offset, out_r + 1));
            cell_t.set_formula(format!("{pay_col}{out_r}/2"));
            cell_t.set_formula_result_number(sum_eval / 2.0);
            cell_t.style_mut().number_format_mut().set_format_code("#,##0");
            cell_t.style_mut().font_mut().set_bold(true);
        };

        let standard_headers: Vec<String> = [
            "DATE",
            "NAME",
            "Total person",
            "Booking Number",
            "Nationality",
            "Room Number",
            "Total Kamar",
            "Date Check in",
            "Date Check out",
            "Total Night",
            "Rate",
            "Total Payment",
            "Remarks",
        ]
        .iter()
        .map(|s| (*s).to_string())
        .collect();

        if src != dest {
            let mut clean_book = umya_spreadsheet::new_file();
            let sheet_name = "Rekap Tamu";
            clean_book.sheet_mut(0).map_err(|e| e.to_string())?.set_name(sheet_name);
            {
                let sheet_res = clean_book
                    .sheet_by_name_mut(sheet_name)
                    .map_err(|e| e.to_string())?;
                write_13col_snapshot(sheet_res, &standard_headers, 0);
            }
            add_reference_sheet_and_dropdown(&mut clean_book, sheet_name, snapshot_rows.len())?;

            umya_spreadsheet::writer::xlsx::write(&clean_book, dest)
                .map_err(|e| format!("Gagal menyimpan file Excel: {e}"))?;

            return Ok(format!(
                "Berhasil mengekspor {} baris hasil rekonsiliasi ke berkas template 13 kolom: {}",
                snapshot_rows.len(),
                dest.to_string_lossy()
            ));
        }

        {
            let sheet = book.sheet_mut(0).map_err(|e| e.to_string())?;
            let source_headers: Vec<String> = (1..=13)
                .map(|c| {
                    sheet
                        .cell((c, 1))
                        .map(|cell_ref| cell_ref.value().to_string())
                        .unwrap_or_default()
                })
                .collect();

            let clear_limit = (items.len() as u32 + 10).max(300);
            for r in 2..=clear_limit {
                for c in 15..=27 {
                    sheet.cell_mut((c, r)).set_value("");
                }
            }

            write_13col_snapshot(sheet, &source_headers, 14);
        }

        let result_sheet_name = "Rekap Tamu (Hasil Acak)";
        if book.sheet_by_name(result_sheet_name).is_err() {
            let _ = book.new_sheet(result_sheet_name);
        }

        if let Ok(sheet_res) = book.sheet_by_name_mut(result_sheet_name) {
            write_13col_snapshot(sheet_res, &standard_headers, 0);
        }

        umya_spreadsheet::writer::xlsx::write(&book, dest)
            .map_err(|e| format!("Gagal menyimpan file Excel: {e}"))?;

        Ok(format!(
            "Berhasil menulis {} baris acak ke kolom O-AA dan sheet 'Rekap Tamu (Hasil Acak)' pada {}",
            snapshot_rows.len(),
            dest.to_string_lossy()
        ))
    }
}

fn iso_date_to_excel_serial(date_str: &str) -> Option<f64> {
    crate::excel_importer::parse_date_string(date_str).and_then(naive_date_to_excel_serial)
}

fn add_reference_sheet_and_dropdown(
    book: &mut Workbook,
    main_sheet_name: &str,
    data_row_count: usize,
) -> Result<(), String> {
    let ref_sheet_name = "Referensi & Petunjuk";
    if book.sheet_by_name(ref_sheet_name).is_err() {
        let _ = book.new_sheet(ref_sheet_name);
    }
    let last_dropdown_row = 9 + INDONESIAN_REGIONS.len() + ALL_WORLD_COUNTRIES.len();
    {
        let ref_sheet = book.sheet_by_name_mut(ref_sheet_name).map_err(|e| e.to_string())?;
        ref_sheet.cell_mut("A1").set_value("PETUNJUK PENGISIAN TEMPLATE 13 KOLOM (FORM 1 DINAS PARIWISATA)");
        ref_sheet.cell_mut("A1").style_mut().font_mut().set_bold(true);
        ref_sheet.cell_mut("A3").set_value("1. Dropdown Kolom E (Nationality) memuat 38 provinsi/kota utama Indonesia (Domestik) dan seluruh ~198 negara/wilayah dunia (Mancanegara).");
        ref_sheet.cell_mut("A4").set_value("2. Input Bebas Diperbolehkan: Jika daerah/negara tamu belum ada di dropdown, Anda tetap dapat mengetik langsung (gunakan 'IDN - Nama Kota' untuk kota kecil domestik, atau ketik langsung nama negaranya untuk mancanegara).");
        ref_sheet.cell_mut("A5").set_value("3. Hindari menulis 'Indonesia' saja agar Tabel 2 Form 1 (10 Besar Daerah Asal Nusantara) menampilkan kota/provinsi asal yang spesifik.");
        ref_sheet.cell_mut("A6").set_value("4. Pastikan Kolom C (Total person), G (Total Kamar), H (Date Check in), I (Date Check out), dan J (Total Night) selalu terisi agar Form 1 menghitung jumlah wisatawan & hunian dengan akurat.");

        ref_sheet.cell_mut("A9").set_value("Daerah Asal Domestik (Nusantara)");
        ref_sheet.cell_mut("A9").style_mut().font_mut().set_bold(true);
        ref_sheet.cell_mut("C9").set_value("Semua Negara Asal Mancanegara (INTL)");
        ref_sheet.cell_mut("C9").style_mut().font_mut().set_bold(true);
        ref_sheet.cell_mut("E9").set_value("Daftar Gabungan Dropdown (Col E)");
        ref_sheet.cell_mut("E9").style_mut().font_mut().set_bold(true);

        for (idx, region) in INDONESIAN_REGIONS.iter().enumerate() {
            let r_num = (10 + idx) as u32;
            ref_sheet.cell_mut((1, r_num)).set_value(*region);
            ref_sheet.cell_mut((5, r_num)).set_value(*region);
        }

        let offset_country = 10 + INDONESIAN_REGIONS.len();
        for (idx, country) in ALL_WORLD_COUNTRIES.iter().enumerate() {
            ref_sheet.cell_mut((3, (10 + idx) as u32)).set_value(*country);
            ref_sheet.cell_mut((5, (offset_country + idx) as u32)).set_value(*country);
        }
    }

    {
        let sheet = book.sheet_by_name_mut(main_sheet_name).map_err(|e| e.to_string())?;
        let mut dv = DataValidation::default();
        dv.set_type(DataValidationValues::List);
        dv.set_allow_blank(true);
        dv.set_show_input_message(true);
        dv.set_show_error_message(false);
        dv.set_prompt_title("Asal Wisatawan (Domestik / Mancanegara)");
        dv.set_prompt("Pilih daerah Indonesia (Domestik) atau negara dunia (Mancanegara). Boleh ketik bebas bila belum ada di daftar.");
        let max_dd_row = (data_row_count as u32 + 200).max(1000);
        dv.sequence_of_references_mut().set_sqref(format!("E2:E{max_dd_row}"));
        dv.set_formula1(format!("='Referensi & Petunjuk'!$E$10:$E${last_dropdown_row}"));

        let mut dvs = DataValidations::default();
        dvs.add_data_validation_list(dv);
        sheet.set_data_validations(dvs);
    }

    Ok(())
}

/// Exports 13-column room reservations from the database into an import-ready .xlsx workbook.
pub fn export_room_reservations_to_excel<P: AsRef<Path>>(
    reservations: &[crate::db::RoomReservationRow],
    dest_path: P,
) -> Result<String, String> {
    if reservations.is_empty() {
        return Err("Tidak ada data reservasi untuk diekspor.".to_string());
    }

    let dest = dest_path.as_ref();
    check_file_writable(dest)?;
    let mut book = umya_spreadsheet::new_file();
    let sheet_name = "Rekap Tamu";
    book.sheet_mut(0).map_err(|e| e.to_string())?.set_name(sheet_name);

    {
        let sheet = book.sheet_by_name_mut(sheet_name).map_err(|e| e.to_string())?;
        let headers = [
            ("A1", "DATE"),
            ("B1", "NAME"),
            ("C1", "Total person"),
            ("D1", "Booking Number"),
            ("E1", "Nationality"),
            ("F1", "Room Number"),
            ("G1", "Total Kamar"),
            ("H1", "Date Check in"),
            ("I1", "Date Check out"),
            ("J1", "Total Night"),
            ("K1", "Rate"),
            ("L1", "Total Payment"),
            ("M1", "Remarks"),
        ];
        for (coord, text) in headers {
            let cell = sheet.cell_mut(coord);
            cell.set_value(text);
            cell.style_mut().font_mut().set_bold(true);
        }

        let mut sum_pax: i64 = 0;
        let mut sum_rooms: i64 = 0;
        let mut sum_nights: i64 = 0;
        let mut sum_payment: f64 = 0.0;

        for (idx, r) in reservations.iter().enumerate() {
            let row_num = (idx + 2) as u32;
            let date_serial = iso_date_to_excel_serial(&r.date);
            let ci_serial = iso_date_to_excel_serial(&r.check_in_date);
            let co_serial = iso_date_to_excel_serial(&r.check_out_date);

            if let Some(serial) = date_serial {
                let c = sheet.cell_mut((1, row_num));
                c.set_value_number(serial);
                c.style_mut().number_format_mut().set_format_code("dd/mm/yy");
            } else {
                sheet.cell_mut((1, row_num)).set_value_string(&r.date);
            }

            sheet.cell_mut((2, row_num)).set_value_string(&r.guest_name);

            let pax = r.total_person.unwrap_or(1).max(1);
            sheet.cell_mut((3, row_num)).set_value_number(pax as f64);

            sheet
                .cell_mut((4, row_num))
                .set_value_string(r.booking_number.as_deref().unwrap_or("-"));
            sheet
                .cell_mut((5, row_num))
                .set_value_string(r.nationality.as_deref().unwrap_or(""));
            sheet
                .cell_mut((6, row_num))
                .set_value_string(r.room_number.as_deref().unwrap_or(""));

            let rooms = r.total_rooms.max(1);
            sheet.cell_mut((7, row_num)).set_value_number(rooms as f64);

            let nights = r
                .total_nights
                .filter(|&n| n > 0)
                .unwrap_or_else(|| {
                    match (
                        NaiveDate::parse_from_str(r.check_in_date.trim(), "%Y-%m-%d"),
                        NaiveDate::parse_from_str(r.check_out_date.trim(), "%Y-%m-%d"),
                    ) {
                        (Ok(ci), Ok(co)) if co > ci => (co - ci).num_days(),
                        _ => 1,
                    }
                })
                .max(1);

            if r.check_in_date.trim() == r.date.trim() {
                if let Some(serial) = date_serial {
                    let c = sheet.cell_mut((8, row_num));
                    c.set_formula(format!("A{row_num}"));
                    c.set_formula_result_number(serial);
                    c.style_mut().number_format_mut().set_format_code("dd/mm/yy");
                } else {
                    sheet.cell_mut((8, row_num)).set_value_string(&r.check_in_date);
                }
            } else if let Some(serial) = ci_serial {
                let c = sheet.cell_mut((8, row_num));
                c.set_value_number(serial);
                c.style_mut().number_format_mut().set_format_code("dd/mm/yy");
            } else {
                sheet.cell_mut((8, row_num)).set_value_string(&r.check_in_date);
            }

            let eff_ci = ci_serial.or(date_serial);
            if let (Some(cis), Some(cos)) = (eff_ci, co_serial) {
                let c = sheet.cell_mut((9, row_num));
                if (cos - (cis + nights as f64)).abs() < 0.5 {
                    c.set_formula(format!("H{row_num}+J{row_num}"));
                    c.set_formula_result_number(cos);
                } else {
                    c.set_value_number(cos);
                }
                c.style_mut().number_format_mut().set_format_code("dd/mm/yy");
            } else if let Some(serial) = co_serial {
                let c = sheet.cell_mut((9, row_num));
                c.set_value_number(serial);
                c.style_mut().number_format_mut().set_format_code("dd/mm/yy");
            } else {
                sheet.cell_mut((9, row_num)).set_value_string(&r.check_out_date);
            }

            sheet.cell_mut((10, row_num)).set_value_number(nights as f64);

            let rate_val = r
                .rate
                .unwrap_or_else(|| r.total_payment / (nights as f64 * rooms as f64));
            let c_rate = sheet.cell_mut((11, row_num));
            c_rate.set_formula(format!("IF(J{row_num}*G{row_num}>0,L{row_num}/J{row_num}/G{row_num},0)"));
            c_rate.set_formula_result_number(rate_val);
            c_rate.style_mut().number_format_mut().set_format_code("#,##0");

            let c_pay = sheet.cell_mut((12, row_num));
            c_pay.set_value_number(r.total_payment);
            c_pay.style_mut().number_format_mut().set_format_code("#,##0");

            sheet
                .cell_mut((13, row_num))
                .set_value_string(r.remarks.as_deref().unwrap_or(""));

            sum_pax += pax;
            sum_rooms += rooms;
            sum_nights += nights;
            sum_payment += r.total_payment;
        }

        let last_r = (reservations.len() + 1) as u32;
        let tot_r = last_r + 1;

        sheet.cell_mut((1, tot_r)).set_value("TOTAL");
        sheet.cell_mut((1, tot_r)).style_mut().font_mut().set_bold(true);

        let c_pax = sheet.cell_mut((3, tot_r));
        c_pax.set_formula(format!("SUM(C2:C{last_r})"));
        c_pax.set_formula_result_number(sum_pax as f64);
        c_pax.style_mut().font_mut().set_bold(true);

        let c_rms = sheet.cell_mut((7, tot_r));
        c_rms.set_formula(format!("SUM(G2:G{last_r})"));
        c_rms.set_formula_result_number(sum_rooms as f64);
        c_rms.style_mut().font_mut().set_bold(true);

        let c_nts = sheet.cell_mut((10, tot_r));
        c_nts.set_formula(format!("SUM(J2:J{last_r})"));
        c_nts.set_formula_result_number(sum_nights as f64);
        c_nts.style_mut().font_mut().set_bold(true);

        let c_pay_tot = sheet.cell_mut((12, tot_r));
        c_pay_tot.set_formula(format!("SUM(L2:L{last_r})"));
        c_pay_tot.set_formula_result_number(sum_payment);
        c_pay_tot.style_mut().number_format_mut().set_format_code("#,##0");
        c_pay_tot.style_mut().font_mut().set_bold(true);
    }

    add_reference_sheet_and_dropdown(&mut book, sheet_name, reservations.len())?;
    umya_spreadsheet::writer::xlsx::write(&book, dest)
        .map_err(|e| format!("Gagal menyimpan ekspor Excel 13 kolom: {e}"))?;

    Ok(format!(
        "Berhasil mengekspor {} data reservasi (13 kolom) ke: {}",
        reservations.len(),
        dest.display()
    ))
}

/// Generates an official import template Excel workbook (.xlsx) for the specified unit.
pub fn generate_import_template<P: AsRef<Path>>(unit: &str, dest_path: P) -> Result<(), String> {
    let dest = dest_path.as_ref();
    check_file_writable(dest)?;
    let mut book = umya_spreadsheet::new_file();
    let is_bungalows = unit.to_lowercase().contains("bungalow") || unit.to_lowercase() == "bgw";

    if is_bungalows {
        let sheet_name = "Rekap Tamu";
        book.sheet_mut(0).map_err(|e| e.to_string())?.set_name(sheet_name);
        {
            let sheet = book.sheet_by_name_mut(sheet_name).map_err(|e| e.to_string())?;

            let headers = [
                ("A1", "DATE"),
                ("B1", "NAME"),
                ("C1", "Total person"),
                ("D1", "Booking Number"),
                ("E1", "Nationality"),
                ("F1", "Room Number"),
                ("G1", "Total Kamar"),
                ("H1", "Date Check in"),
                ("I1", "Date Check out"),
                ("J1", "Total Night"),
                ("K1", "Rate"),
                ("L1", "Total Payment"),
                ("M1", "Remarks"),
            ];

            for (coord, text) in headers {
                let cell = sheet.cell_mut(coord);
                cell.set_value(text);
                cell.style_mut().font_mut().set_bold(true);
            }

            let sample_rows = [
                ("2026-10-01", "Paul Hutson", 2.0, "-", "Australia", "02", 1.0, 2.0, 2500000.0, "Walk in"),
                ("2026-10-02", "Jane Smith", 1.0, "5278381900", "Prancis", "05", 1.0, 2.0, 1850000.0, "Booking.com"),
                ("2026-10-03", "Ahmad Fauzi", 4.0, "2055347556", "Mataram", "06, 07", 2.0, 2.0, 3200000.0, "Agoda"),
            ];

            for (idx, (iso_dt, name, pax, bno, nat, rm, rms, nights, pay, rem)) in sample_rows.iter().enumerate() {
                let row_num = (idx + 2) as u32;
                let serial = iso_date_to_excel_serial(iso_dt).unwrap_or(46296.0 + idx as f64);

                let c_dt = sheet.cell_mut((1, row_num));
                c_dt.set_value_number(serial);
                c_dt.style_mut().number_format_mut().set_format_code("dd/mm/yy");

                sheet.cell_mut((2, row_num)).set_value_string(*name);
                sheet.cell_mut((3, row_num)).set_value_number(*pax);
                sheet.cell_mut((4, row_num)).set_value_string(*bno);
                sheet.cell_mut((5, row_num)).set_value_string(*nat);
                sheet.cell_mut((6, row_num)).set_value_string(*rm);
                sheet.cell_mut((7, row_num)).set_value_number(*rms);

                let c_ci = sheet.cell_mut((8, row_num));
                c_ci.set_formula(format!("A{row_num}"));
                c_ci.set_formula_result_number(serial);
                c_ci.style_mut().number_format_mut().set_format_code("dd/mm/yy");

                let c_co = sheet.cell_mut((9, row_num));
                c_co.set_formula(format!("H{row_num}+J{row_num}"));
                c_co.set_formula_result_number(serial + *nights);
                c_co.style_mut().number_format_mut().set_format_code("dd/mm/yy");

                sheet.cell_mut((10, row_num)).set_value_number(*nights);

                let rate_val = *pay / (*nights * *rms);
                let c_rate = sheet.cell_mut((11, row_num));
                c_rate.set_formula(format!("IF(J{row_num}*G{row_num}>0,L{row_num}/J{row_num}/G{row_num},0)"));
                c_rate.set_formula_result_number(rate_val);
                c_rate.style_mut().number_format_mut().set_format_code("#,##0");

                let c_pay = sheet.cell_mut((12, row_num));
                c_pay.set_value_number(*pay);
                c_pay.style_mut().number_format_mut().set_format_code("#,##0");

                sheet.cell_mut((13, row_num)).set_value_string(*rem);
            }

            // Summary row 6
            let cell_a6 = sheet.cell_mut("A6");
            cell_a6.set_value("TOTAL");
            cell_a6.style_mut().font_mut().set_bold(true);

            let cell_c6 = sheet.cell_mut("C6");
            cell_c6.set_formula("SUM(C2:C4)");
            cell_c6.set_formula_result_number(7.0);
            cell_c6.style_mut().font_mut().set_bold(true);

            let cell_g6 = sheet.cell_mut("G6");
            cell_g6.set_formula("SUM(G2:G4)");
            cell_g6.set_formula_result_number(4.0);
            cell_g6.style_mut().font_mut().set_bold(true);

            let cell_j6 = sheet.cell_mut("J6");
            cell_j6.set_formula("SUM(J2:J4)");
            cell_j6.set_formula_result_number(6.0);
            cell_j6.style_mut().font_mut().set_bold(true);

            let cell_l6 = sheet.cell_mut("L6");
            cell_l6.set_formula("SUM(L2:L4)");
            cell_l6.set_formula_result_number(7550000.0);
            cell_l6.style_mut().number_format_mut().set_format_code("#,##0");
            cell_l6.style_mut().font_mut().set_bold(true);
        }

        add_reference_sheet_and_dropdown(&mut book, sheet_name, 3)?;
    } else {
        let sheet_name = format!("Input {}", unit);
        book.sheet_mut(0).map_err(|e| e.to_string())?.set_name(&sheet_name);
        let sheet = book.sheet_by_name_mut(&sheet_name).map_err(|e| e.to_string())?;

        // Header row 1
        let cell_a1 = sheet.cell_mut("A1");
        cell_a1.set_value("Tanggal");
        cell_a1.style_mut().font_mut().set_bold(true);

        let cell_b1 = sheet.cell_mut("B1");
        cell_b1.set_value("Total Pendapatan (Rp)");
        cell_b1.style_mut().font_mut().set_bold(true);

        let cell_c1 = sheet.cell_mut("C1");
        cell_c1.set_value("Keterangan");
        cell_c1.style_mut().font_mut().set_bold(true);

        // Sample row 2
        let cell_a2 = sheet.cell_mut("A2");
        cell_a2.set_value("2026-08-01");

        let cell_b2 = sheet.cell_mut("B2");
        cell_b2.set_value_number(2500000.0);
        cell_b2.style_mut().number_format_mut().set_format_code("#,##0");

        let cell_c2 = sheet.cell_mut("C2");
        cell_c2.set_value("Penjualan F&B Meja 05");

        // Sample row 3
        let cell_a3 = sheet.cell_mut("A3");
        cell_a3.set_value("2026-08-02");

        let cell_b3 = sheet.cell_mut("B3");
        cell_b3.set_value_number(1850000.0);
        cell_b3.style_mut().number_format_mut().set_format_code("#,##0");

        let cell_c3 = sheet.cell_mut("C3");
        cell_c3.set_value("Penjualan F&B Bar & Lounge");

        // Sample row 4
        let cell_a4 = sheet.cell_mut("A4");
        cell_a4.set_value("2026-08-03");

        let cell_b4 = sheet.cell_mut("B4");
        cell_b4.set_value_number(3200000.0);
        cell_b4.style_mut().number_format_mut().set_format_code("#,##0");

        let cell_c4 = sheet.cell_mut("C4");
        cell_c4.set_value("Reservasi Buffet Group");

        // Summary row 6
        let cell_a6 = sheet.cell_mut("A6");
        cell_a6.set_value("Jumlah");
        cell_a6.style_mut().font_mut().set_bold(true);

        let cell_b6 = sheet.cell_mut("B6");
        cell_b6.set_formula("SUM(B2:B4)");
        cell_b6.set_formula_result_number(7550000.0);
        cell_b6.style_mut().number_format_mut().set_format_code("#,##0");
        cell_b6.style_mut().font_mut().set_bold(true);
    }

    umya_spreadsheet::writer::xlsx::write(&book, dest)
        .map_err(|e| format!("Gagal menyimpan template Excel: {e}"))?;

    Ok(())
}
