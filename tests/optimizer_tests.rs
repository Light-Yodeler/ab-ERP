use excel_subset_lib::excel::{analyze_excel_file, export_solution_to_excel};
use excel_subset_lib::optimizer::{analyze_room_issues, solve_subset, SolverOptions};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

fn create_synthetic_2col_sample(tag: &str) -> PathBuf {
    let sample_path = std::env::temp_dir().join(format!("test_synthetic_contoh_{tag}.xlsx"));
    if sample_path.exists() {
        let _ = fs::remove_file(&sample_path);
    }
    let mut book = umya_spreadsheet::new_file();
    let sheet = book.sheet_mut(0).expect("sheet 0");
    sheet.cell_mut("A1").set_value("tgl");
    sheet.cell_mut("B1").set_value("Total");

    let mut running_sum = 0.0_f64;
    for i in 0..88_usize {
        let row = (i + 2) as u32;
        let amt = 650_000.0
            + ((i % 19) as f64) * 206_000.0
            + (((i * 29) % 103) as f64) * 19.0
            + ((i % 13) as f64) * 3.0;
        running_sum += amt;
        let date_cell = sheet.cell_mut((1, row));
        date_cell.set_value_number(46000.0 + (i as f64));
        sheet.cell_mut((2, row)).set_value_number(amt);
    }
    let last_amt = 223_376_237.0 - running_sum;
    sheet.cell_mut((1, 90)).set_value_number(46088.0);
    sheet.cell_mut((2, 90)).set_value_number(last_amt);

    sheet.cell_mut("A91").set_value("Jumlah");
    let tot_cell = sheet.cell_mut("B91");
    tot_cell.set_formula("SUM(B2:B90)");
    tot_cell.set_formula_result_number(223_376_237.0);

    sheet.cell_mut("B92").set_value_number(111_688_118.5);
    umya_spreadsheet::writer::xlsx::write(&book, &sample_path)
        .expect("Failed to write synthetic sample");
    sample_path
}

#[test]
fn test_analyze_sample_excel() {
    let sample_path = create_synthetic_2col_sample("analyze");
    assert!(sample_path.exists(), "Sample file must exist at {}", sample_path.display());

    let analysis = analyze_excel_file(&sample_path).expect("Should parse sample excel successfully");
    println!("Parsed sheet: {}", analysis.sheet_name);
    println!("Data start row: {}, end row: {}", analysis.data_start_row, analysis.data_end_row);
    println!("Total rows: {}", analysis.total_rows);
    println!("Total sum: {}", analysis.total_sum);
    println!("Target detected: {}", analysis.target_detected);

    assert_eq!(analysis.data_start_row, 2);
    assert_eq!(analysis.data_end_row, 90);
    assert_eq!(analysis.total_rows, 89);
    assert!((analysis.total_sum - 223376237.0).abs() < 1e-3);
    assert!((analysis.target_detected - 111688118.5).abs() < 1e-3);
    let _ = fs::remove_file(&sample_path);
}

#[test]
fn test_solver_speed_and_accuracy() {
    let sample_path = create_synthetic_2col_sample("speed");
    let analysis = analyze_excel_file(&sample_path).expect("Should parse sample excel");
    let options = SolverOptions {
        target: analysis.target_detected,
        tolerance: 100.0,
        max_trajectories: Some(64),
        seed: Some(42),
        exclude_signatures: None,
    };

    let result = solve_subset(&analysis.items, &options).expect("Should find subset solution");

    println!("Solver duration: {:.2} ms", result.duration_ms);
    println!("Target: {:.1}, Total selected: {:.1}, Difference: {:.1}",
        result.target_amount, result.total_amount, result.difference);
    println!("Selected items count: {}", result.selected_indices.len());

    // Criteria: Speed < 20 ms, Difference <= 100
    let max_allowed = if cfg!(debug_assertions) { 100.0 } else { 20.0 };
    assert!(result.duration_ms < max_allowed, "Execution time must be < {} ms, got {:.2} ms", max_allowed, result.duration_ms);
    assert!(result.difference <= 100.0, "Difference must be <= 100, got {:.1}", result.difference);
    // On this dataset difference is close to zero (between 0.5 and a few rupiahs)
    assert!(result.difference <= 50.0, "Difference expected <= 50.0, got {:.1}", result.difference);

    // Verify chronological order: original_rows must be strictly strictly increasing
    for i in 1..result.selected_rows.len() {
        assert!(
            result.selected_rows[i] > result.selected_rows[i - 1],
            "Selected rows must be in strictly increasing chronological order: {} then {}",
            result.selected_rows[i - 1],
            result.selected_rows[i]
        );
    }
    let _ = fs::remove_file(&sample_path);
}

#[test]
fn test_solver_randomized_variations() {
    let sample_path = create_synthetic_2col_sample("variations");
    let analysis = analyze_excel_file(&sample_path).expect("Should parse sample excel");
    let mut seen_signatures = Vec::new();
    let mut solutions: Vec<HashSet<usize>> = Vec::new();

    for run in 0..5 {
        let options = SolverOptions {
            target: analysis.target_detected,
            tolerance: 100.0,
            max_trajectories: Some(64),
            seed: Some(1000 + run as u64 * 777),
            exclude_signatures: Some(seen_signatures.clone()),
        };

        let result = solve_subset(&analysis.items, &options).expect("Should find solution");
        seen_signatures.push(result.signature);
        let row_set: HashSet<usize> = result.selected_rows.iter().copied().collect();

        println!("Run #{}: diff={:.1}, count={}, sig={}",
            run + 1, result.difference, result.selected_rows.len(), result.signature);

        assert!(result.difference <= 100.0, "Difference in run {} must be <= 100", run + 1);

        // Check difference with prior solutions
        for (prev_idx, prev_set) in solutions.iter().enumerate() {
            let symmetric_diff: usize = row_set.symmetric_difference(prev_set).count();
            println!("  Symmetric difference with run #{}: {} rows", prev_idx + 1, symmetric_diff);
            assert!(
                symmetric_diff >= 4,
                "Consecutive regenerate runs should vary by at least 4 rows, got {}",
                symmetric_diff
            );
        }

        solutions.push(row_set);
    }
    let _ = fs::remove_file(&sample_path);
}

#[test]
fn test_export_to_excel_and_verify_structure() {
    let sample_path = create_synthetic_2col_sample("export");
    let analysis = analyze_excel_file(&sample_path).expect("Should parse sample excel");
    let options = SolverOptions {
        target: analysis.target_detected,
        tolerance: 10.0,
        max_trajectories: Some(64),
        seed: Some(999),
        exclude_signatures: None,
    };

    let result = solve_subset(&analysis.items, &options).expect("Should find subset solution");
    let temp_output = std::env::temp_dir().join("contoh_hasil_test.xlsx");
    if temp_output.exists() {
        let _ = fs::remove_file(&temp_output);
    }

    let msg = export_solution_to_excel(&sample_path, &temp_output, &analysis.items, &result.selected_indices)
        .expect("Export should succeed");
    println!("Export result: {}", msg);

    assert!(temp_output.exists(), "Output file must exist");

    // Read back exported workbook with umya_spreadsheet to verify structure
    let book = umya_spreadsheet::reader::xlsx::read(&temp_output).expect("Should re-open exported excel");
    let sheet = book.sheet(0).expect("Should get sheet 0");

    // Check E1 and F1 headers
    let e1 = sheet.cell_value("E1").value();
    let f1 = sheet.cell_value("F1").value();
    assert_eq!(e1, "tgl", "E1 must be 'tgl'");
    assert_eq!(f1, "Total", "F1 must be 'Total'");

    let k = result.selected_indices.len() as u32;
    // Check first data row E2, F2
    let e2 = sheet.cell_value("E2").value();
    let f2 = sheet.cell_value("F2").value();
    assert!(!e2.is_empty(), "E2 must contain date value");
    assert!(!f2.is_empty(), "F2 must contain total value");

    // Check summary row at k + 2
    let sum_row = k + 2;
    let sum_label = sheet.cell_value(format!("E{sum_row}")).value();
    let sum_cell = sheet.cell_value(format!("F{sum_row}"));
    let sum_formula = sum_cell.formula();
    let sum_val: f64 = sum_cell.value().parse().unwrap_or(0.0);

    println!("Exported summary row {}: E='{}', formula='{}', evaluated={}",
        sum_row, sum_label, sum_formula, sum_val);

    assert_eq!(sum_label, "Jumlah", "Summary label must be 'Jumlah'");
    let expected_formula = format!("SUM(F2:F{})", k + 1);
    assert_eq!(sum_formula, expected_formula, "Formula must match SUM range");
    assert!(
        (sum_val - result.total_amount).abs() < 1e-3,
        "Evaluated sum value in F{} ({}) must equal solver result ({})",
        sum_row, sum_val, result.total_amount
    );

    // Clean up
    let _ = fs::remove_file(&temp_output);
    let _ = fs::remove_file(&sample_path);
}

#[test]
fn test_analyze_and_optimize_13_column_reservation_template() {
    let test_file = std::env::temp_dir().join("test_synthetic_13col.xlsx");
    if test_file.exists() {
        let _ = fs::remove_file(&test_file);
    }
    {
        let mut book = umya_spreadsheet::new_file();
        book.sheet_mut(0).unwrap().set_name("Rekap Tamu");
        let sheet = book.sheet_by_name_mut("Rekap Tamu").unwrap();
        let headers = [
            ("A1", "Tanggal"), ("B1", "NAME"), ("C1", "Total person"), ("D1", "Booking Number"),
            ("E1", "Nationality"), ("F1", "Room Number"), ("G1", "Total Kamar"),
            ("H1", "Date Check in"), ("I1", "Date Check out"), ("J1", "Total Night"),
            ("K1", "Rate"), ("L1", "Total Payment"), ("M1", "Remarks"),
        ];
        for (coord, text) in headers {
            sheet.cell_mut(coord).set_value(text);
        }

        let mut running_sum = 0.0_f64;
        for i in 0..76_usize {
            let row = (i + 2) as u32;
            let amt = if i < 75 {
                let v = 1_500_000.0 + ((i as f64) * 16_835.0) + (((i * 29) % 73) as f64) * 500.0;
                running_sum += v;
                v
            } else {
                163_214_747.0 - running_sum
            };
            let serial_ci = 46296.0 + ((i / 3) as f64);
            let serial_co = serial_ci + 2.0;

            let c_date = sheet.cell_mut((1, row));
            c_date.set_value_number(serial_ci);
            c_date.style_mut().number_format_mut().set_format_code("dd/mm/yy");

            sheet.cell_mut((2, row)).set_value(if i == 0 { "Paul Hutson".to_string() } else { format!("Guest {:02}", i + 1) });
            sheet.cell_mut((3, row)).set_value_number(2.0);
            sheet.cell_mut((4, row)).set_value(if i % 3 == 0 { "-" } else { "BK1000" });
            sheet.cell_mut((5, row)).set_value(if i % 4 == 0 { "Mataram" } else { "Australia" });
            sheet.cell_mut((6, row)).set_value(format!("{:02}", (i % 14) + 1));
            sheet.cell_mut((7, row)).set_value_number(1.0);

            let c_ci = sheet.cell_mut((8, row));
            c_ci.set_formula(format!("A{row}"));
            c_ci.set_formula_result_number(serial_ci);
            c_ci.style_mut().number_format_mut().set_format_code("dd/mm/yy");

            let c_co = sheet.cell_mut((9, row));
            c_co.set_formula(format!("H{row}+J{row}"));
            c_co.set_formula_result_number(serial_co);
            c_co.style_mut().number_format_mut().set_format_code("dd/mm/yy");

            sheet.cell_mut((10, row)).set_value_number(2.0);
            let c_rate = sheet.cell_mut((11, row));
            c_rate.set_formula(format!("IF(J{row}*G{row}>0,L{row}/J{row}/G{row},0)"));
            c_rate.set_formula_result_number(amt / 2.0);
            sheet.cell_mut((12, row)).set_value_number(amt);
            sheet.cell_mut((13, row)).set_value(if i % 2 == 0 { "Booking.com" } else { "Walk in" });
        }

        sheet.cell_mut("A78").set_value("TOTAL");
        let tot_cell = sheet.cell_mut("L78");
        tot_cell.set_formula("SUM(L2:L77)");
        tot_cell.set_formula_result_number(163_214_747.0);
        sheet.cell_mut("L79").set_value_number(81_607_373.5);

        umya_spreadsheet::writer::xlsx::write(&book, &test_file)
            .expect("Failed to write synthetic 13-col fixture");
    }

    println!("=== Testing 13-Column Reservation Analysis & Optimization ===");
    let analysis = analyze_excel_file(&test_file).expect("Must parse 13-column template");

    assert_eq!(analysis.format_type, "RoomReservation13Col");
    assert_eq!(analysis.total_rows, 76, "Must detect 76 transaction items");
    assert!((analysis.total_sum - 163214747.0).abs() < 1.0, "Total sum must match Rp 163.214.747");
    assert!((analysis.target_detected - 81607373.5).abs() < 1.0, "Target must be 50% (Rp 81.607.373,5)");
    assert_eq!(analysis.items[0].guest_name.as_deref(), Some("Paul Hutson"));
    let first_det = analysis.items[0].details.as_ref().expect("Details must be parsed");
    assert!(
        first_det.check_in_date.as_deref().map(|s| s.contains('/')).unwrap_or(false),
        "Check-in date must be formatted as date string, got {:?}",
        first_det.check_in_date
    );

    // Solve subset
    let options = SolverOptions {
        target: analysis.target_detected,
        tolerance: 100.0,
        max_trajectories: Some(64),
        seed: Some(42),
        exclude_signatures: None,
    };

    let result = solve_subset(&analysis.items, &options).expect("Should find subset solution");
    println!("13-Col Solver duration: {:.2} ms, diff: {:.1}, count: {}",
        result.duration_ms, result.difference, result.selected_indices.len());
    assert!(result.difference <= 100.0, "Difference must be <= tolerance");

    // Test in-place export to 13-column format
    let temp_copy = std::env::temp_dir().join("test_13col_export_result.xlsx");
    if temp_copy.exists() { let _ = fs::remove_file(&temp_copy); }
    fs::copy(&test_file, &temp_copy).expect("Failed to copy sample file");

    let export_msg = export_solution_to_excel(&temp_copy, &temp_copy, &analysis.items, &result.selected_indices)
        .expect("Export to 13-column format must succeed");
    println!("Export result: {}", export_msg);
    assert!(export_msg.contains("kolom O-AA"));

    // Verify exported workbook structure using umya-spreadsheet
    let book = umya_spreadsheet::reader::xlsx::read(&temp_copy).expect("Must re-read exported file");
    let sheet = book.sheet(0).expect("Must get sheet 0");

    // Check Header in Col O and Col Z
    let o1 = sheet.cell("O1").map(|c| c.value().to_string()).unwrap_or_default();
    let z1 = sheet.cell("Z1").map(|c| c.value().to_string()).unwrap_or_default();
    assert_eq!(o1, "Tanggal", "Col O header must mirror source A1 ('Tanggal')");
    assert_eq!(z1, "Total Payment", "Col Z header must be Total Payment");

    // Check Result Sheet exists
    assert!(book.sheet_by_name("Rekap Tamu (Hasil Acak)").is_ok(), "Result sheet 'Rekap Tamu (Hasil Acak)' must be created");

    let _ = fs::remove_file(&temp_copy);

    // Test Clean Export: Export to a new clean standalone file
    let clean_output_file = std::env::temp_dir().join("test_13col_clean_export.xlsx");
    if clean_output_file.exists() { let _ = fs::remove_file(&clean_output_file); }

    let clean_export_msg = export_solution_to_excel(&test_file, &clean_output_file, &analysis.items, &result.selected_indices)
        .expect("Export to clean 13-column file must succeed");
    println!("Clean export result: {}", clean_export_msg);
    assert!(clean_output_file.exists(), "Clean exported file must exist");

    let clean_book = umya_spreadsheet::reader::xlsx::read(&clean_output_file).expect("Clean output must open");
    assert!(clean_book.sheet_by_name("Referensi & Petunjuk").is_ok(), "Clean 13-col export must include Referensi & Petunjuk sheet");
    let clean_sheet = clean_book.sheet_by_name("Rekap Tamu").unwrap();
    let a2_fmt = clean_sheet.cell("A2").unwrap().style().number_format().map(|nf| nf.format_code().to_string()).unwrap_or_default();
    assert_eq!(a2_fmt, "dd/mm/yy", "Date column in clean export must preserve dd/mm/yy number format");
    assert_eq!(clean_sheet.cell("H2").unwrap().formula(), "A2");
    assert_eq!(clean_sheet.cell("I2").unwrap().formula(), "H2+J2");
    assert_eq!(clean_sheet.cell("K2").unwrap().formula(), "IF(J2*G2>0,L2/J2/G2,0)");

    // Verify round-trip: analyze_excel_file must parse the clean exported file as RoomReservation13Col!
    let clean_analysis = analyze_excel_file(&clean_output_file)
        .expect("Clean exported file must be recognized by analyzer");
    assert_eq!(clean_analysis.format_type, "RoomReservation13Col");
    assert_eq!(clean_analysis.total_rows, result.selected_indices.len());
    assert!((clean_analysis.total_sum - result.total_amount).abs() < 1.0);
    assert!((clean_analysis.target_detected - result.total_amount / 2.0).abs() < 1.0);
    println!("✓ Clean exported file parsed back into analyzer with 100% round-trip fidelity!");

    let _ = fs::remove_file(&clean_output_file);
    let _ = fs::remove_file(&test_file);
    println!("✓ 13-column reservation optimizer analysis, solver, and export passed successfully!");
}

#[test]
fn test_optimizer_prevents_room_date_conflicts_and_detects_anomalies() {
    let test_file = std::env::temp_dir().join("test_room_conflicts_13col.xlsx");
    let clean_out = std::env::temp_dir().join("test_room_conflicts_clean_out.xlsx");
    for p in [&test_file, &clean_out] {
        if p.exists() {
            let _ = fs::remove_file(p);
        }
    }

    // Excel serial helper for August 2026 (2026-08-01 = 46235)
    let aug_serial = |day: u32| -> f64 { 46234.0 + (day as f64) };

    struct SpecRow {
        row: u32,
        date_day: u32,
        ci_day: u32,
        nights: u32,
        room: &'static str,
        total_rooms: f64,
        amount: f64,
        guest: &'static str,
    }

    let mut specs: Vec<SpecRow> = Vec::new();
    // Build sparse non-conflicting background reservations across rooms 2..14 in Aug 2026
    // Rows 2..5
    specs.push(SpecRow { row: 2, date_day: 1, ci_day: 5, nights: 2, room: "2", total_rooms: 1.0, amount: 1_400_000.0, guest: "Early Booker Distinct CI" });
    specs.push(SpecRow { row: 3, date_day: 1, ci_day: 1, nights: 1, room: "4", total_rooms: 1.0, amount: 750_000.0, guest: "SameDay CheckOut R4" });
    specs.push(SpecRow { row: 4, date_day: 2, ci_day: 2, nights: 1, room: "4", total_rooms: 1.0, amount: 760_000.0, guest: "SameDay CheckIn R4" });
    specs.push(SpecRow { row: 5, date_day: 2, ci_day: 2, nights: 1, room: "6", total_rooms: 1.0, amount: 710_000.0, guest: "Normal R6" });

    // Conflict 1: Room 7 on 2 Aug (rows 6 and 7)
    specs.push(SpecRow { row: 6, date_day: 2, ci_day: 2, nights: 1, room: "7", total_rooms: 1.0, amount: 716_099.0, guest: "Conflict R7 A" });
    specs.push(SpecRow { row: 7, date_day: 2, ci_day: 2, nights: 1, room: "7", total_rooms: 1.0, amount: 708_070.0, guest: "Conflict R7 B" });

    // Rows 8..18 (non-conflicting)
    for r in 8..=18_u32 {
        let rm = match (r - 8) % 5 {
            0 => "8",
            1 => "9",
            2 => "10",
            3 => "11",
            _ => "12",
        };
        let day = 3 + (r - 8) / 5 * 2;
        specs.push(SpecRow {
            row: r,
            date_day: day,
            ci_day: day,
            nights: 1,
            room: rm,
            total_rooms: if r == 8 { 2.0 } else { 1.0 }, // row 8 triggers room-count anomaly ('8' vs 2)
            amount: 720_000.0 + ((r as f64) * 3_117.0),
            guest: if r == 8 { "Anomaly SingleRoomFor2" } else { "Filler Early" },
        });
    }

    // Conflict 2: Room 1 on 9 Aug (row 19 checks in Aug 8 for 2 nights, row 25 checks in Aug 9 for 1 night)
    specs.push(SpecRow { row: 19, date_day: 8, ci_day: 8, nights: 2, room: "1", total_rooms: 1.0, amount: 1_450_680.0, guest: "Conflict R1 A" });
    for r in 20..=24_u32 {
        let rm = match r - 20 {
            0 => "2",
            1 => "4",
            2 => "5",
            3 => "6",
            _ => "14",
        };
        specs.push(SpecRow {
            row: r,
            date_day: 8,
            ci_day: 8,
            nights: 1,
            room: rm,
            total_rooms: 1.0,
            amount: 780_000.0 + ((r as f64) * 2_503.0),
            guest: "Filler Mid",
        });
    }
    specs.push(SpecRow { row: 25, date_day: 9, ci_day: 9, nights: 1, room: "1", total_rooms: 1.0, amount: 800_000.0, guest: "Conflict R1 B" });

    // Multi-room "07, 13" vs "13" on Aug 12 (rows 26 & 27) and decimal "5.1" (row 28)
    specs.push(SpecRow { row: 26, date_day: 12, ci_day: 12, nights: 1, room: "07, 13", total_rooms: 2.0, amount: 1_300_000.0, guest: "MultiRoom 07+13" });
    specs.push(SpecRow { row: 27, date_day: 12, ci_day: 12, nights: 1, room: "13", total_rooms: 1.0, amount: 650_000.0, guest: "Conflict R13" });
    specs.push(SpecRow { row: 28, date_day: 13, ci_day: 13, nights: 1, room: "5.1", total_rooms: 2.0, amount: 1_600_000.0, guest: "Decimal Room 5.1" });

    // Rows 29..53 non-conflicting spread across Aug 14..20
    for r in 29..=53_u32 {
        let idx = r - 29;
        let rm = match idx % 6 {
            0 => "2",
            1 => "6",
            2 => "8",
            3 => "9",
            4 => "10",
            _ => "11",
        };
        let day = 14 + (idx / 6) * 2;
        specs.push(SpecRow {
            row: r,
            date_day: day,
            ci_day: day,
            nights: 1,
            room: rm,
            total_rooms: 1.0,
            amount: 680_000.0 + ((r as f64) * 4_219.0),
            guest: "Filler Spread",
        });
    }

    // Conflict 3: Room 3 on 22 Aug (rows 54, 55, 56)
    specs.push(SpecRow { row: 54, date_day: 21, ci_day: 21, nights: 2, room: "3", total_rooms: 1.0, amount: 1_884_000.0, guest: "Conflict R3 A" });
    specs.push(SpecRow { row: 55, date_day: 22, ci_day: 22, nights: 1, room: "3", total_rooms: 1.0, amount: 800_000.0, guest: "Conflict R3 B" });
    specs.push(SpecRow { row: 56, date_day: 22, ci_day: 22, nights: 1, room: "3", total_rooms: 1.0, amount: 800_500.0, guest: "Conflict R3 C" });

    // Rows 57..62 tail non-conflicting
    for (k, r) in (57..=62_u32).enumerate() {
        let rm = match k {
            0 => "12",
            1 => "14",
            2 => "2",
            3 => "4",
            4 => "6",
            _ => "9",
        };
        specs.push(SpecRow {
            row: r,
            date_day: 24 + (k as u32 / 2),
            ci_day: 24 + (k as u32 / 2),
            nights: 1,
            room: rm,
            total_rooms: 1.0,
            amount: 740_000.0 + ((r as f64) * 1_901.0),
            guest: "Tail NonConflict",
        });
    }

    {
        let mut book = umya_spreadsheet::new_file();
        book.sheet_mut(0).unwrap().set_name("Rekap Tamu");
        let sheet = book.sheet_by_name_mut("Rekap Tamu").unwrap();
        let headers = [
            ("A1", "DATE"), ("B1", "NAME"), ("C1", "Total person"), ("D1", "Booking Number"),
            ("E1", "Nationality"), ("F1", "Room Number"), ("G1", "Total Kamar"),
            ("H1", "Date Check in"), ("I1", "Date Check out"), ("J1", "Total Night"),
            ("K1", "Rate"), ("L1", "Total Payment"), ("M1", "Remarks"),
        ];
        for (coord, text) in headers {
            sheet.cell_mut(coord).set_value(text);
        }

        let mut total_sum = 0.0_f64;
        for sp in &specs {
            total_sum += sp.amount;
            let row = sp.row;
            let s_date = aug_serial(sp.date_day);
            let s_ci = aug_serial(sp.ci_day);
            let s_co = aug_serial(sp.ci_day + sp.nights);

            let c_date = sheet.cell_mut((1, row));
            c_date.set_value_number(s_date);
            c_date.style_mut().number_format_mut().set_format_code("dd/mm/yy");

            sheet.cell_mut((2, row)).set_value(sp.guest);
            sheet.cell_mut((3, row)).set_value_number(2.0);
            sheet.cell_mut((4, row)).set_value(format!("BK{row}"));
            sheet.cell_mut((5, row)).set_value("Australia");
            sheet.cell_mut((6, row)).set_value(sp.room);
            sheet.cell_mut((7, row)).set_value_number(sp.total_rooms);

            let c_ci = sheet.cell_mut((8, row));
            if sp.ci_day == sp.date_day {
                c_ci.set_formula(format!("A{row}"));
                c_ci.set_formula_result_number(s_ci);
            } else {
                c_ci.set_value_number(s_ci);
            }
            c_ci.style_mut().number_format_mut().set_format_code("dd/mm/yy");

            let c_co = sheet.cell_mut((9, row));
            c_co.set_formula(format!("H{row}+J{row}"));
            c_co.set_formula_result_number(s_co);
            c_co.style_mut().number_format_mut().set_format_code("dd/mm/yy");

            sheet.cell_mut((10, row)).set_value_number(sp.nights as f64);
            let c_rate = sheet.cell_mut((11, row));
            c_rate.set_formula(format!("IF(J{row}*G{row}>0,L{row}/J{row}/G{row},0)"));
            c_rate.set_formula_result_number(sp.amount / (sp.nights as f64 * sp.total_rooms));
            sheet.cell_mut((12, row)).set_value_number(sp.amount);
            sheet.cell_mut((13, row)).set_value("Booking.com");
        }

        sheet.cell_mut("A63").set_value("TOTAL");
        let tot = sheet.cell_mut("L63");
        tot.set_formula("SUM(L2:L62)");
        tot.set_formula_result_number(total_sum);
        sheet.cell_mut("L64").set_value_number((total_sum / 2.0).round());

        umya_spreadsheet::writer::xlsx::write(&book, &test_file).expect("write conflict test workbook");
    }

    let analysis = analyze_excel_file(&test_file).expect("analyze_excel_file must succeed");
    assert_eq!(analysis.format_type, "RoomReservation13Col");

    // 1) Verify reported conflicts on Room 7 @ 02/08, Room 1 @ 09/08, Room 3 @ 22/08, and Room 13 @ 12/08
    assert!(
        analysis.room_conflicts.iter().any(|c| c.room_number == "7" && c.date == "02/08/2026" && c.rows == vec![6, 7]),
        "Expected conflict on Room 7 @ 02/08/2026 rows [6, 7], got {:?}",
        analysis.room_conflicts
    );
    assert!(
        analysis.room_conflicts.iter().any(|c| c.room_number == "1" && c.date == "09/08/2026" && c.rows == vec![19, 25]),
        "Expected conflict on Room 1 @ 09/08/2026 rows [19, 25], got {:?}",
        analysis.room_conflicts
    );
    assert!(
        analysis.room_conflicts.iter().any(|c| c.room_number == "3" && c.date == "22/08/2026" && c.rows == vec![54, 55, 56]),
        "Expected conflict on Room 3 @ 22/08/2026 rows [54, 55, 56], got {:?}",
        analysis.room_conflicts
    );
    assert!(
        analysis.room_conflicts.iter().any(|c| c.room_number == "13" && c.date == "12/08/2026" && c.rows == vec![26, 27]),
        "Expected multi-room conflict on Room 13 @ 12/08/2026 rows [26, 27], got {:?}",
        analysis.room_conflicts
    );
    // Same-day check-out / check-in on Room 4 MUST NOT conflict
    assert!(
        !analysis.room_conflicts.iter().any(|c| c.room_number == "4"),
        "Same-day checkout/checkin on Room 4 must not conflict, got {:?}",
        analysis.room_conflicts
    );
    // Verify anomalies: decimal "5.1" (row 28) & room count mismatch "8" vs 2 (row 8)
    assert!(
        analysis.room_anomalies.iter().any(|a| a.row == 28 && a.room_number == "5.1"),
        "Expected decimal anomaly on row 28 ('5.1'), got {:?}",
        analysis.room_anomalies
    );
    assert!(
        analysis.room_anomalies.iter().any(|a| a.row == 8 && a.room_number == "8"),
        "Expected room-count mismatch anomaly on row 8 ('8' for 2 rooms), got {:?}",
        analysis.room_anomalies
    );

    // 2) Run solver across 15 seeds + exclude_signatures: every result must stay <= tolerance AND have 0 room conflicts
    let mut seen_sigs: Vec<u64> = Vec::new();
    for seed_idx in 0..15_u64 {
        let res = solve_subset(
            &analysis.items,
            &SolverOptions {
                target: analysis.target_detected,
                tolerance: 100.0,
                max_trajectories: Some(64),
                seed: Some(700 + seed_idx * 97),
                exclude_signatures: Some(seen_sigs.clone()),
            },
        )
        .expect("Solver must find conflict-free solution");

        seen_sigs.push(res.signature);
        assert!(
            res.difference <= 100.0,
            "Seed {seed_idx} diff {} exceeded tolerance",
            res.difference
        );

        let selected_items: Vec<_> = res
            .selected_indices
            .iter()
            .map(|&idx| analysis.items[idx].clone())
            .collect();
        let (sel_conflicts, _) = analyze_room_issues(&selected_items);
        assert!(
            sel_conflicts.is_empty(),
            "Seed {seed_idx} produced conflicting rooms in output: {:?}",
            sel_conflicts
        );
    }

    // 3) Verify clean export preserves distinct Check-In != DATE (row 2) and original room numbers
    let export_indices: Vec<usize> = vec![0, 4, 24, 26]; // includes row 2 (idx 0, distinct CI Aug 5 != Aug 1), row 6 ("7"), row 26 ("07, 13"), and row 28 ("5.1")
    export_solution_to_excel(&test_file, &clean_out, &analysis.items, &export_indices)
        .expect("Clean export must succeed");
    let out_book = umya_spreadsheet::reader::xlsx::read(&clean_out).expect("Read clean out");
    let out_sheet = out_book.sheet_by_name("Rekap Tamu").expect("Rekap Tamu sheet");
    // Row 2 in exported sheet corresponds to index 0 (original row 2: DATE = Aug 1, Check in = Aug 5)
    assert!(!out_sheet.cell("H2").unwrap().is_formula(), "Distinct Check in != DATE must not become =A2");
    let h2_val = out_sheet.cell("H2").unwrap().value_number().expect("H2 numeric serial");
    assert!((h2_val - aug_serial(5)).abs() < 0.5, "H2 serial must preserve Aug 5, got {h2_val}");
    assert_eq!(out_sheet.cell("F2").unwrap().value().to_string(), "2");
    assert_eq!(out_sheet.cell("F4").unwrap().value().to_string(), "07, 13");
    assert_eq!(out_sheet.cell("F5").unwrap().value().to_string(), "5.1");

    let _ = fs::remove_file(&clean_out);
    let _ = fs::remove_file(&test_file);
}

#[test]
fn test_optimizer_prevents_conflicts_with_dd_mm_dash_yy_text_dates() {
    let test_file = std::env::temp_dir().join("test_room_conflicts_dd_mm_dash_yy.xlsx");
    if test_file.exists() {
        let _ = fs::remove_file(&test_file);
    }

    struct TextRow {
        date: String,
        guest: String,
        room: String,
        total_rooms: f64,
        ci: String,
        co: String,
        nights: Option<f64>,
        amount: f64,
    }

    let mut rows: Vec<TextRow> = vec![
        // Row 2..4: Three conflicting guests on Oct 11..12 in Room "04" / "4" using varied date formats
        TextRow {
            date: "11/10/26".to_string(),
            guest: "Andrew Kasumich".to_string(),
            room: "04".to_string(),
            total_rooms: 1.0,
            ci: "11/10/26".to_string(),
            co: "12/10/26".to_string(),
            nights: Some(1.0),
            amount: 722_486.0,
        },
        TextRow {
            date: "11/10-26".to_string(),
            guest: "Eugeny Kusakov".to_string(),
            room: "4".to_string(),
            total_rooms: 1.0,
            ci: "11/10-26".to_string(),
            co: "12/10-26".to_string(),
            nights: Some(1.0),
            amount: 678_089.0,
        },
        TextRow {
            date: "11.10.2026".to_string(),
            guest: "Malena Robles".to_string(),
            room: "4".to_string(),
            total_rooms: 1.0,
            ci: "11.10.2026".to_string(),
            co: "12.10.2026".to_string(),
            nights: Some(1.0),
            amount: 731_440.0,
        },
        // Row 5: Birte Devillee on Room "04", mixed Indo month & ISO timestamp (NO conflict)
        TextRow {
            date: "11 Okt 2026".to_string(),
            guest: "Birte Devillee".to_string(),
            room: "04".to_string(),
            total_rooms: 1.0,
            ci: "2026-10-12 00:00:00".to_string(),
            co: "2026-10-13".to_string(),
            nights: Some(1.0),
            amount: 686_840.0,
        },
        // Row 6..7: Same-day checkout/checkin on Room 10 (NO conflict)
        TextRow {
            date: "07/10-26".to_string(),
            guest: "Jimmy Paule".to_string(),
            room: "10, 11".to_string(),
            total_rooms: 2.0,
            ci: "07/10-26".to_string(),
            co: "10/10-26".to_string(),
            nights: Some(3.0),
            amount: 3_702_530.0,
        },
        TextRow {
            date: "10/10-26".to_string(),
            guest: "Xu Songquan".to_string(),
            room: "10".to_string(),
            total_rooms: 1.0,
            ci: "10/10-26".to_string(),
            co: "12/10-26".to_string(),
            nights: Some(2.0),
            amount: 1_800_000.0,
        },
        // Row 8..9: Multi-room 1-night stay with Total Night = 3 (room-nights) vs next-day Room 03 (NO conflict)
        TextRow {
            date: "21/10-26".to_string(),
            guest: "Lokal MultiRoom".to_string(),
            room: "02, 03, 11".to_string(),
            total_rooms: 3.0,
            ci: "21/10-26".to_string(),
            co: "22/10-26".to_string(),
            nights: Some(3.0),
            amount: 2_400_000.0,
        },
        TextRow {
            date: "22/10-26".to_string(),
            guest: "Katharina Bogner".to_string(),
            room: "03".to_string(),
            total_rooms: 1.0,
            ci: "22/10-26".to_string(),
            co: "23/10-26".to_string(),
            nights: Some(1.0),
            amount: 648_609.0,
        },
        // Row 10: Missing Total Night (None), inferred from 12/10-26 .. 13/10-26
        TextRow {
            date: "12/10-26".to_string(),
            guest: "Bule NoNights".to_string(),
            room: "14".to_string(),
            total_rooms: 1.0,
            ci: "12/10-26".to_string(),
            co: "13/10-26".to_string(),
            nights: None,
            amount: 650_000.0,
        },
    ];

    // Add 54 non-conflicting October filler reservations across safe dates & varied date formats
    let safe_rooms = ["01", "02", "05", "06", "07", "08", "09", "12", "13"];
    let safe_days: [u32; 6] = [1, 3, 15, 17, 25, 28];
    for (k, &day) in safe_days.iter().enumerate() {
        for (ri, &rm) in safe_rooms.iter().enumerate() {
            let idx = k * safe_rooms.len() + ri;
            let (ci, co) = match day {
                1 => (format!("{:02} Okt 2026", day), format!("{:02} Okt 2026", day + 1)),
                15 => (format!("2026-10-{:02}", day), format!("2026-10-{:02}", day + 1)),
                25 => (format!("10/{:02}/26", day), format!("10/{:02}/26", day + 1)),
                _ => (format!("{:02}/10-26", day), format!("{:02}/10-26", day + 1)),
            };
            let amt = 610_000.0 + ((idx as f64) * 19_300.0) + (((idx * 29) % 197) as f64);
            rows.push(TextRow {
                date: ci.clone(),
                guest: format!("Filler Guest {:02}", idx + 1),
                room: rm.to_string(),
                total_rooms: 1.0,
                ci,
                co,
                nights: Some(1.0),
                amount: amt,
            });
        }
    }

    {
        let mut book = umya_spreadsheet::new_file();
        book.sheet_mut(0).unwrap().set_name("Rekap Tamu");
        let sheet = book.sheet_by_name_mut("Rekap Tamu").unwrap();
        let headers = [
            ("A1", "DATE"), ("B1", "NAME"), ("C1", "Total person"), ("D1", "Booking Number"),
            ("E1", "Nationality"), ("F1", "Room Number"), ("G1", "Total Kamar"),
            ("H1", "Date Check in"), ("I1", "Date Check out"), ("J1", "Total Night"),
            ("K1", "Rate"), ("L1", "Total Payment"), ("M1", "Remarks"),
        ];
        for (coord, text) in headers {
            sheet.cell_mut(coord).set_value(text);
        }

        let mut total_sum = 0.0_f64;
        for (i, item) in rows.iter().enumerate() {
            let r = (i + 2) as u32;
            total_sum += item.amount;
            sheet.cell_mut((1, r)).set_value_string(&item.date);
            sheet.cell_mut((2, r)).set_value_string(&item.guest);
            sheet.cell_mut((3, r)).set_value_number(2.0);
            sheet.cell_mut((4, r)).set_value_string("-");
            sheet.cell_mut((5, r)).set_value_string("Australia");
            sheet.cell_mut((6, r)).set_value_string(&item.room);
            sheet.cell_mut((7, r)).set_value_number(item.total_rooms);
            sheet.cell_mut((8, r)).set_value_string(&item.ci);
            sheet.cell_mut((9, r)).set_value_string(&item.co);
            if let Some(nts) = item.nights {
                sheet.cell_mut((10, r)).set_value_number(nts);
            }
            sheet.cell_mut((11, r)).set_value_number(item.amount / item.total_rooms);
            sheet.cell_mut((12, r)).set_value_number(item.amount);
            sheet.cell_mut((13, r)).set_value_string("Walk in");
        }

        let tot_row = (rows.len() + 2) as u32;
        sheet.cell_mut((1, tot_row)).set_value_string("TOTAL");
        let c_tot = sheet.cell_mut((12, tot_row));
        c_tot.set_formula(format!("SUM(L2:L{})", rows.len() + 1));
        c_tot.set_formula_result_number(total_sum);
        sheet.cell_mut((12, tot_row + 1)).set_value_number((total_sum / 2.0).round());

        umya_spreadsheet::writer::xlsx::write(&book, &test_file).expect("write DD/MM-YY conflict fixture");
    }

    let analysis = analyze_excel_file(&test_file).expect("analyze_excel_file must succeed on DD/MM-YY text dates");
    assert_eq!(analysis.format_type, "RoomReservation13Col");

    // Verify all mixed input formats normalize to canonical DD/MM/YY in analysis
    assert_eq!(analysis.items[0].date_str, "11/10/26");
    assert_eq!(analysis.items[1].date_str, "11/10/26");
    assert_eq!(analysis.items[2].date_str, "11/10/26");
    assert_eq!(analysis.items[3].date_str, "11/10/26");
    assert_eq!(analysis.items[3].details.as_ref().unwrap().check_in_date.as_deref(), Some("12/10/26"));
    assert_eq!(analysis.items[3].details.as_ref().unwrap().check_out_date.as_deref(), Some("13/10/26"));
    assert!(analysis.items.iter().any(|it| it.date_str == "25/10/26"), "MM/DD/YY day > 12 must normalize to 25/10/26");
    assert!(analysis.items.iter().any(|it| it.date_str == "15/10/26"), "ISO date must normalize to 15/10/26");
    assert!(analysis.items.iter().any(|it| it.date_str == "01/10/26"), "Indonesian month date must normalize to 01/10/26");

    // Verify Room 4 conflict on 11/10/2026 covers rows 2, 3, 4 (and NOT row 5 on Oct 12)
    assert!(
        analysis.room_conflicts.iter().any(|c| c.room_number == "4" && c.date == "11/10/2026" && c.rows == vec![2, 3, 4]),
        "Expected Room 4 conflict on 11/10/2026 rows [2, 3, 4], got {:?}",
        analysis.room_conflicts
    );
    assert!(
        !analysis.room_conflicts.iter().any(|c| c.room_number == "4" && c.date == "12/10/2026"),
        "Row 5 on 12/10/2026 must not conflict, got {:?}",
        analysis.room_conflicts
    );
    assert!(
        !analysis.room_conflicts.iter().any(|c| c.room_number == "10"),
        "Same-day checkout/checkin on Room 10 must not conflict, got {:?}",
        analysis.room_conflicts
    );
    assert!(
        !analysis.room_conflicts.iter().any(|c| c.room_number == "3"),
        "Multi-room 1-night stay (co - ci) must not create false Oct 22 conflict on Room 3, got {:?}",
        analysis.room_conflicts
    );
    // Verify anomalies for Total Night mismatch (row 8) and missing Total Night (row 10)
    assert!(
        analysis.room_anomalies.iter().any(|a| a.row == 8 && a.message.contains("Total Night (3)")),
        "Expected Total Night mismatch anomaly on row 8, got {:?}",
        analysis.room_anomalies
    );
    assert!(
        analysis.room_anomalies.iter().any(|a| a.row == 10 && a.message.contains("Total Night kosong")),
        "Expected missing Total Night anomaly on row 10, got {:?}",
        analysis.room_anomalies
    );

    // Run solver across 15 seeds + exclude_signatures: must hit tolerance and have 0 conflicts
    let mut seen_sigs: Vec<u64> = Vec::new();
    for seed_idx in 0..15_u64 {
        let res = solve_subset(
            &analysis.items,
            &SolverOptions {
                target: analysis.target_detected,
                tolerance: 100.0,
                max_trajectories: Some(64),
                seed: Some(900 + seed_idx * 131),
                exclude_signatures: Some(seen_sigs.clone()),
            },
        )
        .expect("Solver must find conflict-free subset for DD/MM-YY dates");

        seen_sigs.push(res.signature);
        assert!(
            res.difference <= 100.0,
            "Seed {seed_idx} diff {} exceeded tolerance",
            res.difference
        );

        let selected_items: Vec<_> = res
            .selected_indices
            .iter()
            .map(|&idx| analysis.items[idx].clone())
            .collect();
        let (sel_conflicts, _) = analyze_room_issues(&selected_items);
        assert!(
            sel_conflicts.is_empty(),
            "Seed {seed_idx} selected conflicting rooms: {:?}",
            sel_conflicts
        );
        let oct11_room4_count = res
            .selected_rows
            .iter()
            .filter(|&&r| matches!(r, 2 | 3 | 4))
            .count();
        assert!(
            oct11_room4_count <= 1,
            "At most 1 of rows [2, 3, 4] may be selected, got {:?}",
            res.selected_rows
        );
    }

    // Verify exporting text-dated DD/MM-YY workbook (including missing Total Night row idx 8) normalizes serials & formulas
    let clean_out = std::env::temp_dir().join("test_dd_mm_dash_yy_clean_export.xlsx");
    if clean_out.exists() {
        let _ = fs::remove_file(&clean_out);
    }
    export_solution_to_excel(&test_file, &clean_out, &analysis.items, &[0, 4, 8])
        .expect("Exporting DD/MM-YY workbook must succeed");
    let out_book = umya_spreadsheet::reader::xlsx::read(&clean_out).expect("Read clean DD/MM-YY export");
    let out_sheet = out_book.sheet_by_name("Rekap Tamu").expect("Rekap Tamu sheet");
    for coord in ["A2", "A3", "A4", "H2", "I2", "H4", "I4"] {
        let fmt = out_sheet.cell(coord).unwrap().style().number_format().map(|nf| nf.format_code().to_string()).unwrap_or_default();
        assert_eq!(fmt, "dd/mm/yy", "{coord} must use dd/mm/yy format");
    }
    assert!(out_sheet.cell("A4").unwrap().value_number().is_some(), "Text date in A4 must become real Excel date serial");
    assert_eq!(out_sheet.cell("H4").unwrap().formula(), "A4");
    assert_eq!(out_sheet.cell("J4").unwrap().value_number(), Some(1.0), "Missing Total Night must be inferred as 1");
    assert_eq!(out_sheet.cell("I4").unwrap().formula(), "H4+J4");
    assert_eq!(out_sheet.cell("K4").unwrap().formula(), "IF(J4*G4>0,L4/J4/G4,0)");
    let k4_cached: f64 = out_sheet.cell("K4").unwrap().value().parse().unwrap_or(0.0);
    assert!((k4_cached - 650_000.0).abs() < 1e-2, "Cached Rate in K4 must equal 650000, got {k4_cached}");

    // Verify Legacy 2-col export guards against all-out-of-bounds indices
    let legacy_sample = create_synthetic_2col_sample("oob_guard");
    let legacy_out = std::env::temp_dir().join("test_legacy_oob_out.xlsx");
    let legacy_analysis = analyze_excel_file(&legacy_sample).unwrap();
    let oob_res = export_solution_to_excel(&legacy_sample, &legacy_out, &legacy_analysis.items, &[999_999]);
    assert!(oob_res.is_err(), "Legacy 2-col export must reject all-out-of-bounds indices");

    let _ = fs::remove_file(&clean_out);
    let _ = fs::remove_file(&legacy_sample);
    let _ = fs::remove_file(&legacy_out);
    let _ = fs::remove_file(&test_file);
}
