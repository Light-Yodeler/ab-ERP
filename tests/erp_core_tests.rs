use excel_subset_lib::auth::{authenticate, hash_pin, verify_pin};
use excel_subset_lib::db::{
    add_transaction, backup_database, clean_duplicate_transactions, delete_period_transactions,
    delete_transaction, get_available_months, get_paginated_transactions, get_settings, get_units,
    init_db, restore_database, update_transaction,
};
use excel_subset_lib::tax_report::{generate_tax_recap, generate_tourism_report};
use std::fs;

#[test]
fn test_auth_pbkdf2_legacy_compatibility() {
    println!("=== Testing PBKDF2 HMAC-SHA512 Legacy Compatibility ===");
    let salt = "1b90e03b2784cb3ce769aed861872e45";
    let pin = "123456";
    let expected_hash = "435fa6d6175c000396e8b7c618dbe9c87f9f32d5d6a977b34eba183f777400ef5b634c527a512e0b258fa75bf9e4073b318cb96eaef29c3685cd0b081c3de76d";

    let computed = hash_pin(pin, salt);
    assert_eq!(computed, expected_hash, "Hash must exactly match legacy database hash");
    assert!(verify_pin(pin, salt, expected_hash));
    assert!(!verify_pin("wrong_pin", salt, expected_hash));
    println!("✓ PBKDF2 HMAC-SHA512 100% matched legacy hash!");
}

#[test]
fn test_db_lifecycle_and_crud() {
    println!("=== Testing DB Lifecycle, Seeds, and CRUD ===");
    let temp_db_path = std::env::temp_dir().join("test_anda_erp_lifecycle.sqlite");
    if temp_db_path.exists() {
        let _ = fs::remove_file(&temp_db_path);
    }

    let conn = init_db(&temp_db_path).expect("Failed to initialize test database");

    // 1. Verify Seed Units
    let units = get_units(&conn).expect("Failed to get units");
    assert_eq!(units.len(), 2);
    assert_eq!(units[0].code, "Bungalows");
    assert_eq!(units[1].code, "Restaurant");

    // 2. Verify Seed Settings
    let settings = get_settings(&conn).expect("Failed to get settings");
    assert_eq!(settings.get("businessName").unwrap(), "ANDA BUNGALOWS & RESTAURANT");
    assert_eq!(settings.get("taxRate").unwrap(), "0.10");

    // 3. Verify Default Users & Auth
    let admin_profile = authenticate(&conn, "admin", "123456").expect("Admin login must succeed");
    assert_eq!(admin_profile.role, "admin");

    let staff_profile = authenticate(&conn, "staff", "1234").expect("Staff login must succeed");
    assert_eq!(staff_profile.role, "staff");

    assert!(authenticate(&conn, "admin", "999999").is_err(), "Wrong PIN must fail");

    // 4. Test CRUD Transactions
    let id1 = add_transaction(&conn, "Bungalows", "2026-08-01", 3500000.0, Some("Kamar 01"), Some("manual"))
        .expect("Failed to add transaction 1");
    let id2 = add_transaction(&conn, "Bungalows", "2026-08-02", 4200000.0, Some("Kamar 02"), Some("manual"))
        .expect("Failed to add transaction 2");
    let _id3 = add_transaction(&conn, "Restaurant", "2026-08-01", 1250000.0, Some("Dinner"), Some("manual"))
        .expect("Failed to add transaction 3");

    // Pagination query
    let paged = get_paginated_transactions(&conn, Some("Bungalows"), Some("2026-08"), None, 1, 10)
        .expect("Failed to get paginated transactions");
    assert_eq!(paged.total_count, 2);
    assert_eq!(paged.total_amount, 7700000.0);
    assert_eq!(paged.items.len(), 2);

    // Update & validation checks
    assert!(add_transaction(&conn, "Bungalows", "2026-02-30", 100000.0, None, None).is_err());
    assert!(add_transaction(&conn, "Bungalows", "2026-08-01", f64::NAN, None, None).is_err());
    assert!(update_transaction(&conn, id1, "Bungalows", "2026-99-99", 100000.0, None).is_err());

    update_transaction(&conn, id1, "Bungalows", "2026-08-01", 3800000.0, Some("Kamar 01 Updated"))
        .expect("Failed to update transaction");

    let paged_updated = get_paginated_transactions(&conn, Some("Bungalows"), Some("2026-08"), None, 1, 10).unwrap();
    assert_eq!(paged_updated.total_amount, 8000000.0);

    // Delete
    delete_transaction(&conn, id2).expect("Failed to delete transaction");
    let paged_after_del = get_paginated_transactions(&conn, Some("Bungalows"), Some("2026-08"), None, 1, 10).unwrap();
    assert_eq!(paged_after_del.total_count, 1);

    // Clean duplicates test
    let _dup1 = add_transaction(&conn, "Bungalows", "2026-08-10", 1000000.0, Some("Dup"), None).unwrap();
    let _dup2 = add_transaction(&conn, "Bungalows", "2026-08-10", 1000000.0, Some("Dup"), None).unwrap();
    let cleaned = clean_duplicate_transactions(&conn, Some("Bungalows")).expect("Failed to clean duplicates");
    assert_eq!(cleaned, 1, "Must remove exactly 1 duplicate row");

    println!("✓ Database lifecycle and CRUD tests passed!");
    let _ = fs::remove_file(&temp_db_path);
}

#[test]
fn test_tax_recap_report_computation() {
    println!("=== Testing Tax Recap Monthly Calendar Aggregation ===");
    let temp_db_path = std::env::temp_dir().join("test_anda_erp_recap.sqlite");
    if temp_db_path.exists() {
        let _ = fs::remove_file(&temp_db_path);
    }

    let conn = init_db(&temp_db_path).expect("Failed to initialize database");

    // Add transactions across August 2026 (31 days)
    // Day 1: 3,000,000
    // Day 15: 5,000,000
    // Day 31: 2,000,000
    // Total Revenue = 10,000,000
    add_transaction(&conn, "Bungalows", "2026-08-01", 3000000.0, None, None).unwrap();
    add_transaction(&conn, "Bungalows", "2026-08-15", 5000000.0, None, None).unwrap();
    add_transaction(&conn, "Bungalows", "2026-08-31", 2000000.0, None, None).unwrap();

    let recap = generate_tax_recap(&conn, "Bungalows", "2026-08").expect("Failed to generate tax recap");

    assert_eq!(recap.days_in_month, 31, "August must have 31 days");
    assert_eq!(recap.daily_items.len(), 31, "Calendar items must include all 31 days");

    assert_eq!(recap.total_revenue, 10000000.0);
    assert_eq!(recap.tax_rate, 0.10);
    assert_eq!(recap.tax_due, 1000000.0); // 10%
    assert_eq!(recap.net_revenue, 9000000.0);

    assert_eq!(recap.active_days, 3);
    assert_eq!(recap.zero_days, 28);
    assert_eq!(recap.max_daily_revenue, 5000000.0);
    assert_eq!(recap.min_daily_revenue_non_zero, 2000000.0);

    assert!(recap.document_number.starts_with("No: ANDA-BGW/2026-08/"));

    let months = get_available_months(&conn, Some("Bungalows")).unwrap();
    assert_eq!(months, vec!["2026-08"]);

    assert!(generate_tax_recap(&conn, "Bungalows", "300000-01").is_err(), "Out-of-range year must return Err instead of panicking");

    println!("✓ Tax recapitulation calendar and 10% tax math passed!");
    let _ = fs::remove_file(&temp_db_path);
}

#[test]
fn test_backup_and_disaster_recovery() {
    println!("=== Testing Online Backup & Disaster Recovery ===");
    let temp_db_path = std::env::temp_dir().join("test_anda_erp_backup_src.sqlite");
    let backup_path = std::env::temp_dir().join("test_anda_erp_backup_dst.sqlite");

    if temp_db_path.exists() { let _ = fs::remove_file(&temp_db_path); }
    if backup_path.exists() { let _ = fs::remove_file(&backup_path); }

    let conn = init_db(&temp_db_path).expect("Failed to initialize src db");
    add_transaction(&conn, "Bungalows", "2026-08-01", 5000000.0, Some("Initial"), None).unwrap();

    // Perform Backup
    backup_database(&conn, &backup_path).expect("Failed to backup database");
    assert!(backup_path.exists(), "Backup file must be created on disk");

    // Add extra transaction to source that shouldn't be in backup
    add_transaction(&conn, "Bungalows", "2026-08-02", 9999999.0, Some("Extra"), None).unwrap();
    let count_before_restore: i64 = conn.query_row("SELECT COUNT(*) FROM transactions", [], |r| r.get(0)).unwrap();
    assert_eq!(count_before_restore, 2);

    // Drop connection to allow file restore
    drop(conn);

    // Restore
    restore_database(&temp_db_path, &backup_path).expect("Failed to restore database");

    // Verify .pre_restore.bak was created
    let pre_bak_path = temp_db_path.with_extension("sqlite.pre_restore.bak");
    assert!(pre_bak_path.exists(), "Pre-restore emergency backup must exist");

    // Reopen restored DB and verify it only has 1 transaction (5,000,000)
    let restored_conn = init_db(&temp_db_path).expect("Failed to reopen restored db");
    let count_after_restore: i64 = restored_conn.query_row("SELECT COUNT(*) FROM transactions", [], |r| r.get(0)).unwrap();
    assert_eq!(count_after_restore, 1, "Restored database must match backup state exactly");

    let total: f64 = restored_conn.query_row("SELECT SUM(amount) FROM transactions", [], |r| r.get(0)).unwrap();
    assert_eq!(total, 5000000.0);

    println!("✓ Backup, disaster recovery, and pre-restore safety file passed!");

    let _ = fs::remove_file(&temp_db_path);
    let _ = fs::remove_file(&backup_path);
    let _ = fs::remove_file(&pre_bak_path);
}

#[test]
fn test_generate_and_import_excel_template() {
    println!("=== Testing Excel Template Generation and Calamine Import Round-Trip ===");
    let template_path = std::env::temp_dir().join("test_generated_template.xlsx");
    let archive_dir = std::env::temp_dir().join("test_template_archive");
    let db_path = std::env::temp_dir().join("test_template_db.sqlite");

    if template_path.exists() { let _ = fs::remove_file(&template_path); }
    if db_path.exists() { let _ = fs::remove_file(&db_path); }
    let _ = fs::create_dir_all(&archive_dir);

    // 1. Generate template
    excel_subset_lib::excel::generate_import_template("Bungalows", &template_path)
        .expect("Failed to generate excel template");
    assert!(template_path.exists(), "Template file must be written to disk");

    // 2. Import generated template into database
    let mut conn = init_db(&db_path).expect("Failed to init database");
    let options = excel_subset_lib::excel_importer::ImportOptions {
        target_unit: Some("Bungalows".to_string()),
        mode: "replace_period".to_string(),
    };

    let result = excel_subset_lib::excel_importer::import_excel(
        &mut conn,
        &template_path.to_string_lossy(),
        &archive_dir,
        options,
    )
    .expect("Generated template must be 100% compatible with importer");

    println!("Import result: {} records imported, total: {}, period: {}, hash: {}", result.imported_records, result.total_amount, result.period, result.file_hash);
    assert_eq!(result.imported_records, 3, "Sample template must provide 3 sample rows");
    assert!((result.total_amount - 7550000.0).abs() < 1e-3, "Total must match sample values (2.5M + 1.85M + 3.2M)");
    assert!(!result.period.is_empty(), "Period string must be recorded");
    assert!(!result.file_hash.is_empty(), "SHA-256 hash must be recorded");

    // Verify import_logs in database
    let logs = excel_subset_lib::db::get_import_logs(&conn, Some("Bungalows")).expect("Failed to get import logs");
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].period.as_deref(), Some(result.period.as_str()), "Database log must store period");
    assert_eq!(logs[0].file_hash.as_deref(), Some(result.file_hash.as_str()), "Database log must store file_hash");
    println!("✓ Template generation and import round-trip passed!");

    let _ = fs::remove_file(&template_path);
    let _ = fs::remove_file(&db_path);
    let _ = fs::remove_dir_all(&archive_dir);
}

#[test]
fn test_import_validation_preview_and_duplicate_safeguard() {
    println!("=== Testing Import Validation, Preview, and Duplicate Safeguard ===");
    let template_path = std::env::temp_dir().join("test_preview_template.xlsx");
    let wrong_format_path = std::env::temp_dir().join("test_invalid_file.csv");
    let archive_dir = std::env::temp_dir().join("test_preview_archive");
    let db_path = std::env::temp_dir().join("test_preview_db.sqlite");

    if template_path.exists() { let _ = fs::remove_file(&template_path); }
    if wrong_format_path.exists() { let _ = fs::remove_file(&wrong_format_path); }
    if db_path.exists() { let _ = fs::remove_file(&db_path); }
    let _ = fs::create_dir_all(&archive_dir);

    // Generate template and invalid file
    excel_subset_lib::excel::generate_import_template("Bungalows", &template_path)
        .expect("Failed to generate excel template");
    fs::write(&wrong_format_path, "tanggal,total\n2026-08-01,1000000\n").unwrap();

    let mut conn = init_db(&db_path).expect("Failed to init database");

    // 1. Format validation test: wrong file extension must be rejected with Err
    let invalid_res = excel_subset_lib::excel_importer::preview_excel_import(
        &conn,
        &wrong_format_path.to_string_lossy(),
        Some("Bungalows".to_string()),
    );
    assert!(invalid_res.is_err(), "Invalid format (.csv) must be rejected");
    let err_msg = invalid_res.err().unwrap();
    assert!(err_msg.contains("tidak didukung"), "Error message must state unsupported format: {}", err_msg);

    // 2. Pre-import analysis preview on valid Excel file
    let preview = excel_subset_lib::excel_importer::preview_excel_import(
        &conn,
        &template_path.to_string_lossy(),
        Some("Bungalows".to_string()),
    ).expect("Valid excel file must preview successfully");

    assert_eq!(preview.valid_rows_count, 3);
    assert!((preview.total_amount - 7550000.0).abs() < 1e-3);
    assert!(!preview.periods_detected.is_empty(), "Must detect period");
    assert_eq!(preview.is_duplicate_file, false, "First time preview must not be duplicate");
    assert_eq!(preview.existing_period_transactions_count, 0, "No transactions exist yet");

    // 3. Execute first import
    let options = excel_subset_lib::excel_importer::ImportOptions {
        target_unit: Some("Bungalows".to_string()),
        mode: "replace_period".to_string(),
    };
    let import_res = excel_subset_lib::excel_importer::import_excel(
        &mut conn,
        &template_path.to_string_lossy(),
        &archive_dir,
        options,
    ).expect("Import must succeed");
    assert_eq!(import_res.imported_records, 3);

    // 4. Duplicate Safeguard: Previewing the exact same file again must flag duplicate!
    let duplicate_preview = excel_subset_lib::excel_importer::preview_excel_import(
        &conn,
        &template_path.to_string_lossy(),
        Some("Bungalows".to_string()),
    ).expect("Preview must succeed");

    assert_eq!(duplicate_preview.is_duplicate_file, true, "Must flag identical file as duplicate");
    assert!(duplicate_preview.duplicate_log.is_some(), "Must attach previous import log");
    let dup_log = duplicate_preview.duplicate_log.unwrap();
    assert_eq!(dup_log.records_count, 3);
    assert_eq!(dup_log.unit.as_deref(), Some("Bungalows"));
    assert_eq!(duplicate_preview.existing_period_transactions_count, 3, "Must count 3 existing transactions");

    println!("✓ Import validation, preview, period tracking, and duplicate safeguard tests passed!");

    let _ = fs::remove_file(&template_path);
    let _ = fs::remove_file(&wrong_format_path);
    let _ = fs::remove_file(&db_path);
    let _ = fs::remove_dir_all(&archive_dir);
}

#[test]
fn test_room_reservations_13_col_import_and_book1() {
    println!("=== Testing 13-Column Room Reservations & Dual-Write Invariant ===");
    let temp_db_path = std::env::temp_dir().join("test_room_res_db.sqlite");
    let archive_dir = std::env::temp_dir().join("test_room_res_archive");
    let tpl_13_path = std::env::temp_dir().join("test_tpl_13col.xlsx");
    let tpl_3_path = std::env::temp_dir().join("test_tpl_3col.xlsx");

    if temp_db_path.exists() { let _ = fs::remove_file(&temp_db_path); }
    if tpl_13_path.exists() { let _ = fs::remove_file(&tpl_13_path); }
    if tpl_3_path.exists() { let _ = fs::remove_file(&tpl_3_path); }
    let _ = fs::create_dir_all(&archive_dir);

    let mut conn = init_db(&temp_db_path).expect("Failed to init database");

    // 1. Generate 13-column template for Bungalows and import it
    excel_subset_lib::excel::generate_import_template("Bungalows", &tpl_13_path)
        .expect("Failed to generate 13-col template");

    // Verify Referensi & Petunjuk sheet + non-blocking dropdown DataValidation on Rekap Tamu!E2:E1000
    let tpl_book = umya_spreadsheet::reader::xlsx::read(&tpl_13_path)
        .expect("Failed to open generated Bungalows template with umya_spreadsheet");
    let ref_sheet = tpl_book.sheet_by_name("Referensi & Petunjuk")
        .expect("Generated Bungalows template must include 'Referensi & Petunjuk' sheet");
    assert_eq!(ref_sheet.value("A10"), "Mataram");
    assert_eq!(ref_sheet.value("C10"), "Afganistan");
    let main_sheet = tpl_book.sheet_by_name("Rekap Tamu")
        .expect("Generated Bungalows template must include 'Rekap Tamu' sheet");
    assert_eq!(main_sheet.cell("H2").unwrap().formula(), "A2");
    assert_eq!(main_sheet.cell("I2").unwrap().formula(), "H2+J2");
    assert_eq!(main_sheet.cell("K2").unwrap().formula(), "IF(J2*G2>0,L2/J2/G2,0)");
    let dvs = main_sheet.data_validations()
        .expect("Rekap Tamu sheet must define dropdown data validations");
    assert_eq!(dvs.data_validation_list().len(), 1);
    let dv = &dvs.data_validation_list()[0];
    assert_eq!(dv.sequence_of_references().get_sqref(), "E2:E1000");
    assert_eq!(dv.show_error_message(), false, "Dropdown must allow free typing for unlisted regions/countries");
    assert!(dv.formula1().contains("'Referensi & Petunjuk'!$E$10:$E$"));

    let opt_bungalow = excel_subset_lib::excel_importer::ImportOptions {
        target_unit: Some("Bungalows".to_string()),
        mode: "replace_period".to_string(),
    };

    let res_13 = excel_subset_lib::excel_importer::import_excel(
        &mut conn,
        &tpl_13_path.to_string_lossy(),
        &archive_dir,
        opt_bungalow,
    ).expect("13-col template import must succeed");

    assert_eq!(res_13.imported_records, 3);
    assert!((res_13.total_amount - 7550000.0).abs() < 1e-3);

    // Verify room_reservations table has exactly 3 records
    let res_rows = excel_subset_lib::db::get_room_reservations(&conn, Some("Bungalows"), None)
        .expect("Failed to query room_reservations");
    assert_eq!(res_rows.len(), 3);
    assert_eq!(res_rows[0].guest_name, "Paul Hutson");
    assert_eq!(res_rows[0].room_number.as_deref(), Some("02"));
    assert_eq!(res_rows[0].remarks.as_deref(), Some("Walk in"));
    assert!((res_rows[0].total_payment - 2500000.0).abs() < 1e-3);

    // Verify paginated room_reservations query & search metrics
    let res_paged = excel_subset_lib::db::get_paginated_room_reservations(
        &conn,
        Some("Bungalows"),
        None,
        Some("Paul"),
        1,
        25,
    )
    .expect("Failed to query paginated room_reservations");
    assert_eq!(res_paged.total_count, 1);
    assert_eq!(res_paged.total_person, 2);
    assert_eq!(res_paged.total_rooms, 1);
    assert_eq!(res_paged.total_nights, 2);
    assert!((res_paged.total_payment - 2500000.0).abs() < 1e-3);

    // Verify Tourism Form 1 report generation on October 2026 sample template
    let tourism = excel_subset_lib::tax_report::generate_tourism_report(&conn, "Bungalows", "2026-10", 14)
        .expect("Failed to generate tourism Form 1 report");
    assert_eq!(tourism.month_name, "OKTOBER");
    assert_eq!(tourism.days_in_month, 31);
    assert_eq!(tourism.weekly_rows.len(), 4);
    assert_eq!(tourism.total_reservations, 3);
    assert_eq!(tourism.total_guests, 7); // 2 + 1 + 4
    assert_eq!(tourism.domestic_guests, 4); // Ahmad Fauzi (Mataram)
    assert_eq!(tourism.foreign_guests, 3); // Paul Hutson (2, Australia) + Jane Smith (1, Prancis)
    assert_eq!(tourism.weekly_total.room_occupied, 8); // (1*2) + (1*2) + (2*2)
    assert_eq!(tourism.weekly_total.room_vacant, 14 * 31 - 8);
    assert_eq!(tourism.weekly_total.bed_occupied, 14); // (2*2) + (1*2) + (4*2)
    assert_eq!(tourism.weekly_total.bed_check_out, 7);
    assert_eq!(tourism.foreign_origins[0].origin_name, "Australia");
    assert_eq!(tourism.foreign_origins[0].guest_count, 2);
    assert_eq!(tourism.domestic_origins[0].origin_name, "Mataram");
    assert_eq!(tourism.domestic_origins[0].guest_count, 4);
    assert!((tourism.avg_stay_nights - 2.0).abs() < 1e-3);
    assert_eq!(tourism.stay_category, "1-2 MALAM");

    // 2. Generate 3-column template for Restaurant and import it (Option 1 verification!)
    excel_subset_lib::excel::generate_import_template("Restaurant", &tpl_3_path)
        .expect("Failed to generate 3-col template");

    let opt_resto = excel_subset_lib::excel_importer::ImportOptions {
        target_unit: Some("Restaurant".to_string()),
        mode: "replace_period".to_string(),
    };

    let res_3 = excel_subset_lib::excel_importer::import_excel(
        &mut conn,
        &tpl_3_path.to_string_lossy(),
        &archive_dir,
        opt_resto,
    ).expect("3-col template import must succeed");

    assert_eq!(res_3.imported_records, 3);
    // Verify room_reservations for Restaurant has 0 rows (Option 1 verified!)
    let resto_res = excel_subset_lib::db::get_room_reservations(&conn, Some("Restaurant"), None)
        .expect("Failed to query room_reservations");
    assert_eq!(resto_res.len(), 0, "3-column format must NOT insert dummy rows into room_reservations");

    // 3. Self-contained Form 1 edge cases:
    //    - Nationality boundary matching ("Solomon Islands" -> Foreign vs "Solo", "Bali", "Lombok Tengah" -> Domestic)
    //    - Fallback when check_in_date == check_out_date while total_nights > 0
    //    - Cross-month reservation detection in get_available_months
    conn.execute(
        "INSERT INTO room_reservations (unit, date, guest_name, total_person, booking_number, nationality, room_number, total_rooms, check_in_date, check_out_date, total_nights, rate, total_payment, remarks, source_file)
         VALUES
         ('Bungalows', '2026-10-10', 'Solomon Guest', 2, 'BK-SOL', 'Solomon Islands', '08', 1, '2026-10-10', '2026-10-10', 2, 700000.0, 1400000.0, 'Direct', 'fixture'),
         ('Bungalows', '2026-10-11', 'Solo Guest', 1, 'BK-SLO', 'Solo', '09', 1, '2026-10-11', '2026-10-12', 1, 600000.0, 600000.0, 'Direct', 'fixture'),
         ('Bungalows', '2026-10-12', 'Bali Guest', 2, 'BK-BAL', 'Bali', '10', 1, '2026-10-12', '2026-10-13', 1, 650000.0, 650000.0, 'Direct', 'fixture'),
         ('Bungalows', '2026-10-13', 'Lombok Guest', 3, 'BK-LOM', 'Lombok Tengah', '11', 1, '2026-10-13', '2026-10-14', 1, 650000.0, 650000.0, 'Direct', 'fixture'),
         ('Bungalows', '2026-10-14', 'Prefix IDN Guest', 2, 'BK-PIDN', 'IDN - Surabaya', '13', 1, '2026-10-14', '2026-10-15', 1, 700000.0, 700000.0, 'Direct', 'fixture'),
         ('Bungalows', '2026-10-15', 'Prefix INTL Guest', 2, 'BK-PINTL', 'INTL - Australia', '14', 1, '2026-10-15', '2026-10-16', 1, 750000.0, 750000.0, 'Booking.com', 'fixture'),
         ('Bungalows', '2026-11-29', 'Cross Month Guest', 2, 'BK-XM', 'Germany', '12', 1, '2026-11-29', '2026-12-02', 3, 900000.0, 2700000.0, 'Booking.com', 'fixture')",
        [],
    ).expect("Failed to insert Form 1 edge-case fixtures");

    let tourism_edge = excel_subset_lib::tax_report::generate_tourism_report(&conn, "Bungalows", "2026-10", 14)
        .expect("Failed to generate tourism Form 1 report with edge cases");
    assert_eq!(tourism_edge.foreign_guests, 7); // Initial 3 + Solomon Guest (2) + INTL - Australia (2)
    assert_eq!(tourism_edge.domestic_guests, 12); // Initial 4 + Solo (1) + Bali (2) + Lombok Tengah (3) + IDN - Surabaya (2)
    assert!(
        tourism_edge.foreign_origins.iter().any(|o| o.origin_name == "Solomon Islands" && o.guest_count == 2),
        "Solomon Islands must be classified as Mancanegara, not matched by substring 'solo'"
    );
    assert!(
        tourism_edge.foreign_origins.iter().any(|o| o.origin_name == "Australia" && o.guest_count == 4),
        "Prefix 'INTL - Australia' (2) must normalize and aggregate into 'Australia' (2 + 2 = 4)"
    );
    assert!(
        tourism_edge.domestic_origins.iter().any(|o| o.origin_name == "Solo" && o.guest_count == 1),
        "Exact domestic city 'Solo' must still be classified as Nusantara"
    );
    assert!(
        tourism_edge.domestic_origins.iter().any(|o| o.origin_name == "Surabaya" && o.guest_count == 2),
        "Prefix 'IDN - Surabaya' must normalize into 'Surabaya' under Nusantara"
    );
    // Initial room_occupied (8) + Solomon same-date fallback (1 room * 2 nights = 2) + 5 * (1 * 1) = 15
    assert_eq!(tourism_edge.weekly_total.room_occupied, 15);
    // Initial bed_occupied (14) + Solomon fallback (4) + Solo (1) + Bali (2) + Lombok Tengah (3) + IDN - Surabaya (2) + INTL - Australia (2) = 28
    assert_eq!(tourism_edge.weekly_total.bed_occupied, 28);

    let avail_months = get_available_months(&conn, Some("Bungalows")).expect("Failed to list available months");
    assert!(avail_months.contains(&"2026-11".to_string()));
    assert!(avail_months.contains(&"2026-12".to_string()), "Cross-month check_out_date period in room_reservations must appear in get_available_months");

    // 4. Test atomic update_room_reservation (validation + dual-sync) and 13-col Excel export round-trip
    let ahmad_row = res_rows
        .iter()
        .find(|r| r.guest_name == "Ahmad Fauzi")
        .expect("Ahmad Fauzi reservation must exist");

    // Reject invalid Check Out <= Check In
    let invalid_dates = excel_subset_lib::db::RoomReservationUpdate {
        id: ahmad_row.id,
        date: ahmad_row.date.clone(),
        guest_name: "Ahmad Fauzi".to_string(),
        total_person: 4,
        booking_number: ahmad_row.booking_number.clone(),
        nationality: Some("Surabaya".to_string()),
        room_number: Some("06, 07".to_string()),
        total_rooms: 2,
        check_in_date: "2026-10-03".to_string(),
        check_out_date: "2026-10-03".to_string(),
        total_payment: 3200000.0,
        remarks: Some("Agoda".to_string()),
    };
    assert!(
        excel_subset_lib::db::update_room_reservation(&mut conn, &invalid_dates).is_err(),
        "update_room_reservation must reject check_out_date <= check_in_date"
    );

    // Valid atomic update updating nationality, guests, dates, and payment
    let valid_update = excel_subset_lib::db::RoomReservationUpdate {
        id: ahmad_row.id,
        date: ahmad_row.date.clone(),
        guest_name: "Ahmad Fauzi".to_string(),
        total_person: 5,
        booking_number: ahmad_row.booking_number.clone(),
        nationality: Some("Surabaya".to_string()),
        room_number: Some("06, 07".to_string()),
        total_rooms: 2,
        check_in_date: "2026-10-03".to_string(),
        check_out_date: "2026-10-06".to_string(),
        total_payment: 4500000.0,
        remarks: Some("Direct".to_string()),
    };
    excel_subset_lib::db::update_room_reservation(&mut conn, &valid_update)
        .expect("Valid update_room_reservation must succeed");

    let updated_res = excel_subset_lib::db::get_room_reservations(&conn, Some("Bungalows"), Some("2026-10"))
        .expect("Failed to reload reservations");
    let updated_ahmad = updated_res
        .iter()
        .find(|r| r.id == ahmad_row.id)
        .expect("Updated row must exist");
    assert_eq!(updated_ahmad.total_person, Some(5));
    assert_eq!(updated_ahmad.nationality.as_deref(), Some("Surabaya"));
    assert_eq!(updated_ahmad.total_nights, Some(3));
    assert!((updated_ahmad.rate.unwrap() - 750000.0).abs() < 1e-3);
    assert!((updated_ahmad.total_payment - 4500000.0).abs() < 1e-3);

    // Verify linked transactions row was atomically synchronized
    let linked_tx_id = updated_ahmad.transaction_id.expect("Reservation must stay linked to transaction");
    let (tx_amt, tx_notes): (f64, String) = conn
        .query_row(
            "SELECT amount, COALESCE(notes, '') FROM transactions WHERE id = ?1",
            rusqlite::params![linked_tx_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .expect("Linked transaction must exist");
    assert!((tx_amt - 4500000.0).abs() < 1e-3);
    assert_eq!(tx_notes, "Ahmad Fauzi (Direct) - Kamar 06, 07");

    // Verify update_transaction recalculates room_reservations.rate atomically
    update_transaction(
        &conn,
        linked_tx_id,
        "Bungalows",
        &ahmad_row.date,
        6000000.0,
        Some("Ahmad Fauzi (Direct) - Kamar 06, 07"),
    )
    .expect("update_transaction must sync total_payment and recalculate rate");
    let rate_after_tx_update: f64 = conn
        .query_row(
            "SELECT rate FROM room_reservations WHERE id = ?1",
            rusqlite::params![ahmad_row.id],
            |r| r.get(0),
        )
        .unwrap();
    assert!((rate_after_tx_update - 1000000.0).abs() < 1e-3, "rate must update to 6000000 / (3 * 2) = 1000000");

    // Restore updated payment back to 4500000 for export test below
    update_transaction(
        &conn,
        linked_tx_id,
        "Bungalows",
        &ahmad_row.date,
        4500000.0,
        Some("Ahmad Fauzi (Direct) - Kamar 06, 07"),
    )
    .unwrap();
    let updated_res = excel_subset_lib::db::get_room_reservations(&conn, Some("Bungalows"), Some("2026-10"))
        .expect("Failed to reload reservations");

    // Verify case-insensitive origin merge in generate_tourism_report
    conn.execute(
        "INSERT INTO room_reservations (unit, date, guest_name, total_person, nationality, total_rooms, check_in_date, check_out_date, total_nights, rate, total_payment, source_file)
         VALUES ('Bungalows', '2026-10-16', 'Upper Case Aussie', 1, 'AUSTRALIA', 1, '2026-10-16', '2026-10-17', 1, 500000.0, 500000.0, 'fixture')",
        [],
    ).unwrap();
    let tourism_case = generate_tourism_report(&conn, "Bungalows", "2026-10", 14).unwrap();
    let aus_count: Vec<_> = tourism_case
        .foreign_origins
        .iter()
        .filter(|o| o.origin_name.eq_ignore_ascii_case("Australia"))
        .collect();
    assert_eq!(aus_count.len(), 1, "AUSTRALIA and Australia must merge into one row");
    assert_eq!(aus_count[0].origin_name, "Australia");
    assert_eq!(aus_count[0].guest_count, 5);

    // 5. Export October 2026 reservations to 13-col Excel and verify re-import into fresh DB
    let export_13_path = std::env::temp_dir().join("test_exported_13col.xlsx");
    let roundtrip_db_path = std::env::temp_dir().join("test_export_roundtrip_db.sqlite");
    if export_13_path.exists() { let _ = fs::remove_file(&export_13_path); }
    if roundtrip_db_path.exists() { let _ = fs::remove_file(&roundtrip_db_path); }

    let export_msg = excel_subset_lib::excel::export_room_reservations_to_excel(&updated_res, &export_13_path)
        .expect("export_room_reservations_to_excel must succeed");
    assert!(export_msg.contains("Berhasil mengekspor"));
    assert!(export_13_path.exists());

    let exported_book = umya_spreadsheet::reader::xlsx::read(&export_13_path)
        .expect("Failed to open exported 13-col Excel");
    assert!(exported_book.sheet_by_name("Referensi & Petunjuk").is_ok());
    let exported_sheet = exported_book.sheet_by_name("Rekap Tamu").unwrap();
    assert_eq!(exported_sheet.cell("H2").unwrap().formula(), "A2");
    assert_eq!(exported_sheet.cell("I2").unwrap().formula(), "H2+J2");
    assert_eq!(exported_sheet.cell("K2").unwrap().formula(), "IF(J2*G2>0,L2/J2/G2,0)");
    let exported_dv = exported_sheet
        .data_validations()
        .expect("Exported Excel must retain E2:E1000 dropdown validation");
    assert_eq!(exported_dv.data_validation_list()[0].show_error_message(), false);

    let mut rt_conn = init_db(&roundtrip_db_path).expect("Failed to init round-trip DB");
    let rt_res = excel_subset_lib::excel_importer::import_excel(
        &mut rt_conn,
        &export_13_path.to_string_lossy(),
        &archive_dir,
        excel_subset_lib::excel_importer::ImportOptions {
            target_unit: Some("Bungalows".to_string()),
            mode: "replace_period".to_string(),
        },
    )
    .expect("Re-importing exported 13-col Excel must succeed");
    let expected_oct_sum: f64 = updated_res.iter().map(|r| r.total_payment).sum();
    assert_eq!(rt_res.imported_records, updated_res.len());
    assert!((rt_res.total_amount - expected_oct_sum).abs() < 1e-2);

    // Verify delete_room_reservation removes both reservation and linked transaction atomically
    let tx_count_before: i64 = conn
        .query_row("SELECT COUNT(*) FROM transactions WHERE unit = 'Bungalows'", [], |r| r.get(0))
        .unwrap();
    excel_subset_lib::db::delete_room_reservation(&mut conn, ahmad_row.id)
        .expect("delete_room_reservation must succeed");
    let tx_count_after: i64 = conn
        .query_row("SELECT COUNT(*) FROM transactions WHERE unit = 'Bungalows'", [], |r| r.get(0))
        .unwrap();
    assert_eq!(tx_count_after, tx_count_before - 1);
    let residual_res: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM room_reservations WHERE id = ?1",
            rusqlite::params![ahmad_row.id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(residual_res, 0);

    let _ = fs::remove_file(&export_13_path);
    let _ = fs::remove_file(&roundtrip_db_path);

    // Verify delete_period_transactions removes both transactions and reservations atomically
    let del_cnt = delete_period_transactions(&conn, "Bungalows", "2026-11-01", "2026-12-31")
        .expect("delete_period_transactions must succeed");
    assert_eq!(del_cnt, 0); // fixture Cross Month Guest had no transaction_id
    let remaining_nov: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM room_reservations WHERE unit = 'Bungalows' AND date BETWEEN '2026-11-01' AND '2026-12-31'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(remaining_nov, 0, "delete_period_transactions must remove room_reservations in range too");

    println!("✓ All 13-column room reservations, atomic edit/delete, export round-trip, Form 1 edge cases, and dual-write invariant tests passed!");
    let _ = fs::remove_file(&temp_db_path);
    let _ = fs::remove_file(&tpl_13_path);
    let _ = fs::remove_file(&tpl_3_path);
    let _ = fs::remove_dir_all(&archive_dir);
}

#[test]
fn test_flexible_date_inputs_and_update_transaction_syncs_reservation_dates() {
    let temp_db = std::env::temp_dir().join("test_flex_dates_sync.sqlite");
    if temp_db.exists() {
        let _ = fs::remove_file(&temp_db);
    }
    let mut conn = init_db(&temp_db).expect("init_db");

    // 1) add_transaction accepts DD/MM/YY, DD/MM-YY, Indo month names, MM/DD/YY (when day > 12), and ISO datetime
    let id_1 = add_transaction(&conn, "Bungalows", "02/08/26", 500_000.0, Some("dd/mm/yy"), None).unwrap();
    let id_2 = add_transaction(&conn, "Bungalows", "02/08-26", 500_000.0, Some("dd/mm-yy"), None).unwrap();
    let id_3 = add_transaction(&conn, "Bungalows", "2 Agustus 2026", 500_000.0, Some("indo"), None).unwrap();
    let id_4 = add_transaction(&conn, "Bungalows", "08/25/26", 500_000.0, Some("mm/dd/yy"), None).unwrap();
    let id_5 = add_transaction(&conn, "Bungalows", "2026-08-02 14:30:00", 500_000.0, Some("datetime"), None).unwrap();

    let get_date = |id: i64| -> String {
        conn.query_row("SELECT date FROM transactions WHERE id = ?1", [id], |r| r.get(0)).unwrap()
    };
    assert_eq!(get_date(id_1), "2026-08-02");
    assert_eq!(get_date(id_2), "2026-08-02");
    assert_eq!(get_date(id_3), "2026-08-02");
    assert_eq!(get_date(id_4), "2026-08-25");
    assert_eq!(get_date(id_5), "2026-08-02");

    // 2) update_transaction shifting date by +5 days must shift linked check_in_date & check_out_date atomically
    let linked_tx_id = add_transaction(&conn, "Bungalows", "01/08/26", 1_400_000.0, Some("Linked Stay"), None).unwrap();
    conn.execute(
        "INSERT INTO room_reservations (unit, transaction_id, date, guest_name, total_person, nationality, room_number, total_rooms, check_in_date, check_out_date, total_nights, rate, total_payment, source_file)
         VALUES ('Bungalows', ?1, '2026-08-01', 'Shift Test Guest', 2, 'Australia', '04', 1, '2026-08-03', '2026-08-05', 2, 700000.0, 1400000.0, 'fixture')",
        rusqlite::params![linked_tx_id],
    ).unwrap();

    update_transaction(&conn, linked_tx_id, "Bungalows", "06/08/26", 1_600_000.0, Some("Shifted +5d")).unwrap();
    let (res_date, res_ci, res_co, res_rate): (String, String, String, f64) = conn.query_row(
        "SELECT date, check_in_date, check_out_date, rate FROM room_reservations WHERE transaction_id = ?1",
        [linked_tx_id],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
    ).unwrap();
    assert_eq!(res_date, "2026-08-06");
    assert_eq!(res_ci, "2026-08-08", "check_in_date must shift by +5 days");
    assert_eq!(res_co, "2026-08-10", "check_out_date must shift by +5 days preserving 2 nights");
    assert!((res_rate - 800_000.0).abs() < 1e-2);

    // 3) update_room_reservation also accepts DD/MM/YY and DD/MM-YY and normalizes to YYYY-MM-DD
    let res_id: i64 = conn.query_row(
        "SELECT id FROM room_reservations WHERE transaction_id = ?1",
        [linked_tx_id],
        |r| r.get(0),
    ).unwrap();
    excel_subset_lib::db::update_room_reservation(
        &mut conn,
        &excel_subset_lib::db::RoomReservationUpdate {
            id: res_id,
            date: "10/08/26".to_string(),
            guest_name: "Shift Test Guest".to_string(),
            total_person: 2,
            booking_number: None,
            nationality: Some("Australia".to_string()),
            room_number: Some("04".to_string()),
            total_rooms: 1,
            check_in_date: "10/08-26".to_string(),
            check_out_date: "13 Agt 2026".to_string(),
            total_payment: 2_100_000.0,
            remarks: None,
        },
    ).expect("update_room_reservation with flexible dates must succeed");

    let (u_date, u_ci, u_co, u_nights): (String, String, String, i64) = conn.query_row(
        "SELECT date, check_in_date, check_out_date, total_nights FROM room_reservations WHERE id = ?1",
        [res_id],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
    ).unwrap();
    assert_eq!(u_date, "2026-08-10");
    assert_eq!(u_ci, "2026-08-10");
    assert_eq!(u_co, "2026-08-13");
    assert_eq!(u_nights, 3);

    let _ = fs::remove_file(&temp_db);
}
