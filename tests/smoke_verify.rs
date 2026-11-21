use excel_subset_lib::excel::{analyze_excel_file, export_solution_to_excel};
use excel_subset_lib::optimizer::{solve_subset, SolverOptions};
use std::fs;
use std::path::PathBuf;

fn ensure_smoke_source() -> PathBuf {
    let sample_path = std::env::temp_dir().join("smoke_synthetic_contoh.xlsx");
    if !sample_path.exists() {
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
        sheet.cell_mut((1, row)).set_value_number(46000.0 + (i as f64));
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
            .expect("Failed to write synthetic smoke source");
    }
    sample_path
}

#[test]
fn test_smoke_verification() {
    println!("=== Smoke Verification of End-to-End Excel In-Place Modification ===");
    let original_path = ensure_smoke_source();
    let smoke_copy = std::env::temp_dir().join("smoke_contoh_actual.xlsx");

    // 1. Copy original file
    fs::copy(&original_path, &smoke_copy).expect("Failed to copy sample file");
    println!("1. Copied {} to {}", original_path.display(), smoke_copy.display());

    // 2. Analyze
    let analysis = analyze_excel_file(&smoke_copy).expect("Failed to analyze file");
    println!("2. Analysis successful: Sheet='{}', Data rows={}-{} (total {} rows), Target detected={}",
        analysis.sheet_name, analysis.data_start_row, analysis.data_end_row, analysis.total_rows, analysis.target_detected);

    // 3. Solve subset
    let options = SolverOptions {
        target: analysis.target_detected,
        tolerance: 0.5,
        max_trajectories: Some(64),
        seed: Some(12345),
        exclude_signatures: None,
    };
    let result = solve_subset(&analysis.items, &options).expect("Failed to solve subset");
    println!("3. Solver finished in {:.2} ms!", result.duration_ms);
    println!("   Target: {:.1}", result.target_amount);
    println!("   Selected Total: {:.1}", result.total_amount);
    println!("   Difference: {:.1}", result.difference);
    println!("   Selected count: {} items", result.selected_indices.len());

    let max_allowed = if cfg!(debug_assertions) { 100.0 } else { 20.0 };
    assert!(result.duration_ms < max_allowed, "Solver duration must be < {}ms, got {:.2}ms", max_allowed, result.duration_ms);
    assert!(result.difference <= 50.0, "Difference must be <= 50");

    // 4. Export in-place to columns E & F
    let msg = export_solution_to_excel(&smoke_copy, &smoke_copy, &analysis.items, &result.selected_indices)
        .expect("Failed to export in-place");
    println!("4. In-place export message: {}", msg);

    // 5. Verify written file structure using umya-spreadsheet
    let book = umya_spreadsheet::reader::xlsx::read(&smoke_copy).expect("Failed to re-read file");
    let sheet = book.sheet(0).expect("Failed to get sheet 0");

    let e1 = sheet.cell_value("E1").value();
    let f1 = sheet.cell_value("F1").value();
    println!("5. Header E1='{}', F1='{}'", e1, f1);
    assert_eq!(e1, "tgl");
    assert_eq!(f1, "Total");

    let k = result.selected_indices.len() as u32;
    let sum_row = k + 2;
    let sum_label = sheet.cell_value(format!("E{sum_row}")).value();
    let sum_cell = sheet.cell_value(format!("F{sum_row}"));
    let sum_formula = sum_cell.formula();
    let sum_val: f64 = sum_cell.value().parse().unwrap_or(0.0);

    println!("   Summary row {}: E='{}', F formula='{}', evaluated={}",
        sum_row, sum_label, sum_formula, sum_val);

    assert_eq!(sum_label, "Jumlah");
    assert_eq!(sum_formula, format!("SUM(F2:F{})", k + 1));
    assert!((sum_val - result.total_amount).abs() < 1e-3);

    let _ = fs::remove_file(&smoke_copy);
    let _ = fs::remove_file(&original_path);
    println!("=== All Smoke Verification Checks Passed Successfully! ===");
}
