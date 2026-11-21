use excel_subset_lib::certificate::{
    self, CertificatePreviewInput, CertificateRenderData,
};
use excel_subset_lib::db::{self, TrainingCertificateInput};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

fn temp_test_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "anda_erp_cert_test_{}_{}",
        tag,
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create temp test dir");
    dir
}

#[test]
fn test_certificate_auto_numbering_and_year_reset() {
    let dir = temp_test_dir("num");
    let db_path = dir.join("cert_num.sqlite");
    let conn = db::init_db(&db_path).expect("init_db");

    assert_eq!(db::get_next_certificate_number(&conn, 2026).unwrap(), "001/ABR/2026");

    let c1 = db::create_certificate(
        &conn,
        &TrainingCertificateInput {
            certificate_no: None,
            recipient_name: "Ni Putu Ayu Lestari".to_string(),
            duration_text: "three-month".to_string(),
            program_name: None,
            start_date: "2026-06-01".to_string(),
            end_date: "2026-08-31".to_string(),
            issued_date: "2026-09-01".to_string(),
            department: Some("Front Office".to_string()),
            institution: Some("Poltekpar Lombok".to_string()),
            achievement_text: Some("Achieving an Excellent rating in hospitality service and professionalism.".to_string()),
            notes: None,
            signer_name: None,
            signer_title: None,
            paper_size: None,
        },
        Some("admin"),
    )
    .expect("create c1");
    assert_eq!(c1.certificate_no, "001/ABR/2026");
    assert_eq!(c1.paper_size, "F4");
    assert_eq!(c1.signer_name, "Agus Sumardi Wijaya");
    assert_eq!(c1.signer_title, "General Manager");
    assert_eq!(
        c1.achievement_text.as_deref(),
        Some("Achieving an Excellent rating in hospitality service and professionalism.")
    );

    assert_eq!(db::get_next_certificate_number(&conn, 2026).unwrap(), "002/ABR/2026");

    // Manual jump to 010/ABR/2026 -> next becomes 011/ABR/2026
    let c2 = db::create_certificate(
        &conn,
        &TrainingCertificateInput {
            certificate_no: Some("010/ABR/2026".to_string()),
            recipient_name: "I Made Putra".to_string(),
            duration_text: "six-month".to_string(),
            program_name: None,
            start_date: "01/03/26".to_string(),
            end_date: "31/08/26".to_string(),
            issued_date: "05/09/26".to_string(),
            department: Some("Housekeeping (3 months); Food & Beverage Service (3 months)".to_string()),
            institution: None,
            achievement_text: None,
            notes: None,
            signer_name: None,
            signer_title: None,
            paper_size: Some("A4".to_string()),
        },
        Some("staff"),
    )
    .expect("create c2");
    assert_eq!(c2.certificate_no, "010/ABR/2026");
    assert_eq!(db::get_next_certificate_number(&conn, 2026).unwrap(), "011/ABR/2026");

    // Year reset in 2027
    assert_eq!(db::get_next_certificate_number(&conn, 2027).unwrap(), "001/ABR/2027");

    // Duplicate number fails cleanly (even different case via COLLATE NOCASE)
    let dup = db::create_certificate(
        &conn,
        &TrainingCertificateInput {
            certificate_no: Some("010/abr/2026".to_string()),
            recipient_name: "Duplikat".to_string(),
            duration_text: "three-month".to_string(),
            program_name: None,
            start_date: "2026-01-01".to_string(),
            end_date: "2026-03-31".to_string(),
            issued_date: "2026-04-01".to_string(),
            department: None,
            institution: None,
            achievement_text: None,
            notes: None,
            signer_name: None,
            signer_title: None,
            paper_size: None,
        },
        Some("admin"),
    );
    assert!(dup.is_err());
    assert!(dup.unwrap_err().contains("sudah terdaftar"));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_certificate_crud_flexible_dates_and_pagination() {
    let dir = temp_test_dir("crud");
    let db_path = dir.join("cert_crud.sqlite");
    let conn = db::init_db(&db_path).expect("init_db");

    // Verify idempotent migration on DB missing achievement_text column
    let legacy_db_path = dir.join("legacy_v170.sqlite");
    {
        let raw_conn = rusqlite::Connection::open(&legacy_db_path).expect("open raw");
        raw_conn
            .execute_batch(
                "CREATE TABLE training_certificates (
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
                    notes TEXT,
                    signer_name TEXT NOT NULL DEFAULT 'Agus Sumardi Wijaya',
                    signer_title TEXT NOT NULL DEFAULT 'General Manager',
                    paper_size TEXT NOT NULL DEFAULT 'F4',
                    status TEXT NOT NULL DEFAULT 'active',
                    created_by TEXT,
                    created_at DATETIME,
                    updated_at DATETIME
                );",
            )
            .expect("legacy table");
    }
    let migrated_conn = db::init_db(&legacy_db_path).expect("migrate legacy");
    let has_col: bool = migrated_conn
        .prepare("PRAGMA table_info(training_certificates)")
        .unwrap()
        .query_map([], |r| r.get::<_, String>(1))
        .unwrap()
        .filter_map(Result::ok)
        .any(|name| name == "achievement_text");
    assert!(has_col, "achievement_text column should be auto-migrated");

    let row1 = db::create_certificate(
        &conn,
        &TrainingCertificateInput {
            certificate_no: None,
            recipient_name: "Baiq Rina Hartini".to_string(),
            duration_text: "three-month".to_string(),
            program_name: None,
            start_date: "01/05/26".to_string(),
            end_date: "31/07/26".to_string(),
            issued_date: "31/07/26".to_string(),
            department: Some("Housekeeping".to_string()),
            institution: Some("SMKN 1 Pujut".to_string()),
            achievement_text: Some("Achieving a Very Good rating in hospitality service and professionalism.".to_string()),
            notes: Some("Batch 1".to_string()),
            signer_name: None,
            signer_title: None,
            paper_size: None,
        },
        Some("staff"),
    )
    .expect("create row1");
    assert_eq!(row1.start_date, "2026-05-01");
    assert_eq!(row1.end_date, "2026-07-31");
    assert_eq!(row1.issued_date, "2026-07-31");
    assert_eq!(row1.status, "active");

    let row2 = db::create_certificate(
        &conn,
        &TrainingCertificateInput {
            certificate_no: None,
            recipient_name: "Lalu Ahmad Fauzan".to_string(),
            duration_text: "six-month".to_string(),
            program_name: None,
            start_date: "2027-01-01".to_string(),
            end_date: "2027-06-30".to_string(),
            issued_date: "2027-07-02".to_string(),
            department: Some("Housekeeping (3 months); Kitchen (3 months)".to_string()),
            institution: Some("STP Mataram".to_string()),
            achievement_text: None,
            notes: None,
            signer_name: None,
            signer_title: None,
            paper_size: Some("A4".to_string()),
        },
        Some("admin"),
    )
    .expect("create row2");
    assert_eq!(row2.certificate_no, "001/ABR/2027");

    let updated = db::update_certificate(
        &conn,
        row1.id,
        &TrainingCertificateInput {
            certificate_no: Some("005/ABR/2026".to_string()),
            recipient_name: "Baiq Rina Hartini, A.Md".to_string(),
            duration_text: "four-month".to_string(),
            program_name: Some("hospitality training program".to_string()),
            start_date: "2026-05-01".to_string(),
            end_date: "2026-08-31".to_string(),
            issued_date: "2026-09-01".to_string(),
            department: Some("Front Office".to_string()),
            institution: Some("SMKN 1 Pujut".to_string()),
            achievement_text: Some("Excellent Hospitality Rating".to_string()),
            notes: Some("Revised".to_string()),
            signer_name: Some("Agus Sumardi Wijaya".to_string()),
            signer_title: Some("General Manager".to_string()),
            paper_size: Some("F4".to_string()),
        },
    )
    .expect("update");
    assert_eq!(updated.certificate_no, "005/ABR/2026");
    assert_eq!(updated.achievement_text.as_deref(), Some("Excellent Hospitality Rating"));

    let revoked = db::set_certificate_status(&conn, row1.id, "revoked").expect("revoke");
    assert_eq!(revoked.status, "revoked");

    let years = db::get_certificate_years(&conn).expect("years");
    assert_eq!(years, vec!["2027".to_string(), "2026".to_string()]);

    let page_all = db::get_paginated_certificates(&conn, None, None, None, 1, 25).expect("all");
    assert_eq!(page_all.total_count, 2);
    assert_eq!(page_all.active_count, 1);
    assert_eq!(page_all.revoked_count, 1);

    let page_search = db::get_paginated_certificates(&conn, Some("2026"), Some("revoked"), Some("Rating"), 1, 25)
        .expect("filtered");
    assert_eq!(page_search.total_count, 1);
    assert_eq!(page_search.items[0].id, row1.id);

    db::delete_certificate(&conn, row1.id).expect("delete");
    let page_after = db::get_paginated_certificates(&conn, None, None, None, 1, 25).expect("after");
    assert_eq!(page_after.total_count, 1);

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_certificate_fontdb_and_svg_geometry() {
    let fonts = certificate::check_bundled_fonts().expect("all 5 bundled fonts resolve");
    assert_eq!(fonts.len(), 5);
    assert_eq!(
        certificate::format_english_date("2026-11-21").unwrap(),
        "November 21, 2026"
    );
    assert_eq!(
        certificate::format_english_date("05/01/26").unwrap(),
        "January 5, 2026"
    );
    assert_eq!(
        certificate::format_english_date_range(Some("2026-06-01"), Some("2026-11-21")),
        "June 1, 2026 to November 21, 2026"
    );
    assert_eq!(
        certificate::format_english_date_range(Some("2025-11-01"), Some("2026-04-30")),
        "November 1, 2025 to April 30, 2026"
    );
    assert_eq!(
        certificate::format_divisions_phrase(None),
        None
    );
    assert_eq!(
        certificate::format_divisions_phrase(Some("Front Office")).as_deref(),
        Some("in the Front Office Department")
    );
    assert_eq!(
        certificate::format_divisions_phrase(Some("the Front Office Department")).as_deref(),
        Some("in the Front Office Department")
    );
    assert_eq!(
        certificate::format_divisions_phrase(Some("Housekeeping (3 months); Food & Beverage (3 months)")).as_deref(),
        Some("in the Housekeeping Department (3 months) and the Food & Beverage Department (3 months)")
    );
    assert_eq!(
        certificate::format_divisions_phrase(Some("Front Office; Housekeeping; Kitchen")).as_deref(),
        Some("in the Front Office Department, the Housekeeping Department, and the Kitchen Department")
    );

    let dir = temp_test_dir("svg");
    let settings = HashMap::new();

    // Case 1: Single division + issued_date == end_date (Awarded on suppressed to avoid repetition)
    let single_same_day = CertificateRenderData::from_preview_input(
        &CertificatePreviewInput {
            certificate_no: Some("001/ABR/2026".to_string()),
            recipient_name: Some("M. Syahid & Partners <VIP>".to_string()),
            duration_text: Some("six-month".to_string()),
            program_name: None,
            start_date: Some("2026-06-01".to_string()),
            end_date: Some("2026-11-21".to_string()),
            issued_date: Some("2026-11-21".to_string()),
            department: Some("Front Office".to_string()),
            achievement_text: Some("Achieving an Excellent rating in hospitality service and professionalism.".to_string()),
            signer_name: None,
            signer_title: None,
            paper_size: Some("F4".to_string()),
            status: None,
        },
        &dir,
        &settings,
    );
    assert!(!single_same_day.show_awarded_line);

    let f4_svg = certificate::render_certificate_svg(&single_same_day);
    assert!(f4_svg.contains("width=\"330mm\" height=\"215mm\" viewBox=\"0 0 330.00 215.00\""));
    assert!(f4_svg.contains("width=\"310.00\" height=\"195.00\" fill=\"none\" stroke=\"#C67D24\""));
    assert!(f4_svg.contains("Certificate No. 001/ABR/2026"));
    assert!(f4_svg.contains("M. Syahid &amp; Partners &lt;VIP&gt;"));
    assert!(f4_svg.contains("for successfully completing the six-month hospitality training program in the Front Office Department"));
    assert!(f4_svg.contains("at Anda Bungalows &amp; Restaurant, from June 1, 2026 to November 21, 2026."));
    assert!(f4_svg.contains("Achieving an Excellent rating in hospitality service and professionalism."));
    assert!(!f4_svg.contains("Awarded on November 21, 2026."));

    let lines = certificate::build_body_lines(&single_same_day);
    assert_eq!(lines.len(), 3);
    assert_eq!(
        lines[0],
        "for successfully completing the six-month hospitality training program in the Front Office Department"
    );
    assert_eq!(
        lines[1],
        "at Anda Bungalows & Restaurant, from June 1, 2026 to November 21, 2026."
    );

    // Case 2: 2-division rotation + issued_date != end_date (Awarded on kept explicitly)
    let rotation_diff_day = CertificateRenderData::from_preview_input(
        &CertificatePreviewInput {
            certificate_no: Some("002/ABR/2026".to_string()),
            recipient_name: Some("Ni Wayan Sari Putri".to_string()),
            duration_text: Some("six-month".to_string()),
            program_name: None,
            start_date: Some("2026-05-01".to_string()),
            end_date: Some("2026-10-31".to_string()),
            issued_date: Some("2026-11-21".to_string()),
            department: Some("Housekeeping (3 months); Food & Beverage (3 months)".to_string()),
            achievement_text: Some("Achieving a Very Good rating in hospitality service and professionalism".to_string()),
            signer_name: None,
            signer_title: None,
            paper_size: Some("F4".to_string()),
            status: None,
        },
        &dir,
        &settings,
    );
    assert!(rotation_diff_day.show_awarded_line);
    let rot_svg = certificate::render_certificate_svg(&rotation_diff_day);
    assert!(rot_svg.contains("in the Housekeeping Department (3 months) and the Food &amp; Beverage Department (3 months)"));
    assert!(rot_svg.contains("at Anda Bungalows &amp; Restaurant, from May 1, 2026 to October 31, 2026."));
    assert!(rot_svg.contains("Achieving a Very Good rating in hospitality service and professionalism."));
    assert!(rot_svg.contains("Awarded on November 21, 2026."));

    let mut a4_data = rotation_diff_day.clone();
    a4_data.paper_size = "A4".to_string();
    a4_data.recipient_name = "ABCDEFGHIJKLMNOPQRSTUVWXYZ ABCDEFGHIJKLMNOPQRSTUVWXYZ".to_string();
    let a4_svg = certificate::render_certificate_svg(&a4_data);
    assert!(a4_svg.contains("width=\"297mm\" height=\"210mm\" viewBox=\"0 0 297.00 210.00\""));
    assert!(a4_svg.contains("width=\"277.00\" height=\"190.00\" fill=\"none\" stroke=\"#C67D24\""));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_certificate_pdf_and_png_generation() {
    let dir = temp_test_dir("export");
    let settings = HashMap::new();
    let sample_single_f4 = CertificateRenderData::from_preview_input(
        &CertificatePreviewInput {
            certificate_no: Some("001/ABR/2026".to_string()),
            recipient_name: Some("Baiq Rina Hartini".to_string()),
            duration_text: Some("six-month".to_string()),
            program_name: Some("hospitality training program".to_string()),
            start_date: Some("2026-06-01".to_string()),
            end_date: Some("2026-11-21".to_string()),
            issued_date: Some("2026-11-21".to_string()),
            department: Some("Front Office".to_string()),
            achievement_text: Some("Achieving an Excellent rating in hospitality service and professionalism.".to_string()),
            signer_name: Some("Agus Sumardi Wijaya".to_string()),
            signer_title: Some("General Manager".to_string()),
            paper_size: Some("F4".to_string()),
            status: Some("active".to_string()),
        },
        &dir,
        &settings,
    );

    let f4_pdf = certificate::render_certificate_pdf_bytes(&sample_single_f4).expect("f4 pdf");
    assert!(f4_pdf.starts_with(b"%PDF-"));
    let f4_pdf_str = String::from_utf8_lossy(&f4_pdf);
    assert!(f4_pdf_str.contains("/MediaBox [0 0 935.4"), "unexpected F4 MediaBox in PDF");
    assert!(f4_pdf_str.contains("609.4"), "unexpected F4 height in PDF");

    let f4_png = certificate::render_certificate_png_bytes(&sample_single_f4).expect("f4 png");
    assert!(f4_png.starts_with(b"\x89PNG\r\n\x1a\n"));
    let f4_w = u32::from_be_bytes(f4_png[16..20].try_into().unwrap());
    let f4_h = u32::from_be_bytes(f4_png[20..24].try_into().unwrap());
    assert!((3895..=3901).contains(&f4_w), "unexpected F4 300DPI PNG width {}", f4_w);
    assert!((2536..=2542).contains(&f4_h), "unexpected F4 300DPI PNG height {}", f4_h);

    let sample_rot_f4 = CertificateRenderData::from_preview_input(
        &CertificatePreviewInput {
            certificate_no: Some("002/ABR/2026".to_string()),
            recipient_name: Some("Ni Wayan Sari Putri".to_string()),
            duration_text: Some("six-month".to_string()),
            program_name: Some("hospitality training program".to_string()),
            start_date: Some("2026-05-01".to_string()),
            end_date: Some("2026-10-31".to_string()),
            issued_date: Some("2026-11-21".to_string()),
            department: Some("Housekeeping (3 months); Food & Beverage (3 months)".to_string()),
            achievement_text: Some("Achieving a Very Good rating in hospitality service and professionalism.".to_string()),
            signer_name: Some("Agus Sumardi Wijaya".to_string()),
            signer_title: Some("General Manager".to_string()),
            paper_size: Some("F4".to_string()),
            status: Some("active".to_string()),
        },
        &dir,
        &settings,
    );
    let rot_png = certificate::render_certificate_png_bytes(&sample_rot_f4).expect("rot png");
    assert!(rot_png.starts_with(b"\x89PNG\r\n\x1a\n"));

    let mut sample_a4 = sample_rot_f4;
    sample_a4.paper_size = "A4".to_string();
    let a4_pdf = certificate::render_certificate_pdf_bytes(&sample_a4).expect("a4 pdf");
    let a4_pdf_str = String::from_utf8_lossy(&a4_pdf);
    assert!(a4_pdf_str.contains("/MediaBox [0 0 841.8"), "unexpected A4 MediaBox in PDF");
    assert!(a4_pdf_str.contains("595.2"), "unexpected A4 height in PDF");

    let a4_png = certificate::render_certificate_png_bytes(&sample_a4).expect("a4 png");
    let a4_w = u32::from_be_bytes(a4_png[16..20].try_into().unwrap());
    let a4_h = u32::from_be_bytes(a4_png[20..24].try_into().unwrap());
    assert!((3505..=3511).contains(&a4_w), "unexpected A4 300DPI PNG width {}", a4_w);
    assert!((2478..=2484).contains(&a4_h), "unexpected A4 300DPI PNG height {}", a4_h);

    let sample_maulana_f4 = CertificateRenderData::from_preview_input(
        &CertificatePreviewInput {
            certificate_no: Some("001/ABR/2026".to_string()),
            recipient_name: Some("Maulana".to_string()),
            duration_text: Some("six-month".to_string()),
            program_name: Some("hospitality training program".to_string()),
            start_date: Some("2026-06-01".to_string()),
            end_date: Some("2026-11-22".to_string()),
            issued_date: Some("2026-11-22".to_string()),
            department: Some("Housekeeping".to_string()),
            achievement_text: None,
            signer_name: Some("Agus Sumardi Wijaya".to_string()),
            signer_title: Some("General Manager".to_string()),
            paper_size: Some("F4".to_string()),
            status: Some("active".to_string()),
        },
        &dir,
        &settings,
    );
    assert_eq!(certificate::build_body_lines(&sample_maulana_f4).len(), 2);

    let mut sample_maulana_a4 = sample_maulana_f4;
    sample_maulana_a4.paper_size = "A4".to_string();
    assert_eq!(certificate::build_body_lines(&sample_maulana_a4).len(), 2);

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_certificate_asset_validation() {
    let dir = temp_test_dir("asset");

    let bad_path = dir.join("fake.png");
    fs::write(&bad_path, b"not a valid image").unwrap();
    assert!(certificate::validate_and_save_asset(&dir, "logo", &bad_path).is_err());

    let big_path = dir.join("big.png");
    let mut big_bytes = b"\x89PNG\r\n\x1a\n".to_vec();
    big_bytes.resize(5 * 1024 * 1024 + 64, 0u8);
    fs::write(&big_path, big_bytes).unwrap();
    assert!(certificate::validate_and_save_asset(&dir, "logo", &big_path).is_err());

    let sample_png = dir.join("real_sig.png");
    fs::write(
        &sample_png,
        fs::read("src-tauri/assets/cert_logo_default.png")
            .or_else(|_| fs::read("assets/cert_logo_default.png"))
            .expect("read bundled default logo"),
    )
    .unwrap();
    let saved = certificate::validate_and_save_asset(&dir, "signature", &sample_png).expect("save sig");
    assert_eq!(saved, "custom_signature.png");
    assert!(certificate::load_signature_data_uri(&dir, Some(&saved)).is_some());

    // Preserves existing custom logo when a subsequent invalid upload fails
    let saved_logo = certificate::validate_and_save_asset(&dir, "logo", &sample_png).expect("save logo");
    assert_eq!(saved_logo, "custom_logo.png");
    assert!(certificate::validate_and_save_asset(&dir, "logo", &bad_path).is_err());
    assert!(dir.join("custom_logo.png").exists());

    certificate::remove_custom_asset(&dir, "signature").expect("remove");
    assert!(certificate::load_signature_data_uri(&dir, Some(&saved)).is_none());
    assert!(certificate::load_logo_data_uri(&dir, None).starts_with("data:image/png;base64,"));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_certificate_regression_audit_cases() {
    let dir = temp_test_dir("reg");
    let db_path = dir.join("cert_reg.sqlite");
    let conn = db::init_db(&db_path).expect("init_db");

    // 1. Negative validation cases
    let empty_name = db::create_certificate(
        &conn,
        &TrainingCertificateInput {
            certificate_no: None,
            recipient_name: "   ".to_string(),
            duration_text: "three-month".to_string(),
            program_name: None,
            start_date: "2026-06-01".to_string(),
            end_date: "2026-08-31".to_string(),
            issued_date: "2026-09-01".to_string(),
            department: None,
            institution: None,
            achievement_text: None,
            notes: None,
            signer_name: None,
            signer_title: None,
            paper_size: None,
        },
        Some("admin"),
    );
    assert!(empty_name.is_err());

    let bad_dates = db::create_certificate(
        &conn,
        &TrainingCertificateInput {
            certificate_no: None,
            recipient_name: "Test Peserta".to_string(),
            duration_text: "three-month".to_string(),
            program_name: None,
            start_date: "2026-09-01".to_string(),
            end_date: "2026-06-01".to_string(),
            issued_date: "2026-09-02".to_string(),
            department: None,
            institution: None,
            achievement_text: None,
            notes: None,
            signer_name: None,
            signer_title: None,
            paper_size: None,
        },
        Some("admin"),
    );
    assert!(bad_dates.is_err());

    let bad_paper = db::create_certificate(
        &conn,
        &TrainingCertificateInput {
            certificate_no: None,
            recipient_name: "Test Peserta".to_string(),
            duration_text: "three-month".to_string(),
            program_name: None,
            start_date: "2026-06-01".to_string(),
            end_date: "2026-08-31".to_string(),
            issued_date: "2026-09-01".to_string(),
            department: None,
            institution: None,
            achievement_text: None,
            notes: None,
            signer_name: None,
            signer_title: None,
            paper_size: Some("Letter".to_string()),
        },
        Some("admin"),
    );
    assert!(bad_paper.is_err());

    assert!(db::set_certificate_status(&conn, 99999, "revoked").is_err());
    assert!(db::delete_certificate(&conn, 99999).is_err());

    // Empty result pagination clamp
    let empty_page = db::get_paginated_certificates(&conn, None, None, None, 5, 25).expect("empty");
    assert_eq!(empty_page.page, 1);
    assert_eq!(empty_page.total_pages, 1);

    let created = db::create_certificate(
        &conn,
        &TrainingCertificateInput {
            certificate_no: None,
            recipient_name: "Peserta Satu".to_string(),
            duration_text: "six-month".to_string(),
            program_name: None,
            start_date: "2026-05-01".to_string(),
            end_date: "2026-10-31".to_string(),
            issued_date: "2026-11-21".to_string(),
            department: Some("Housekeeping".to_string()),
            institution: None,
            achievement_text: None,
            notes: None,
            signer_name: None,
            signer_title: None,
            paper_size: Some("F4".to_string()),
        },
        Some("admin"),
    )
    .expect("create one");

    assert!(db::set_certificate_status(&conn, created.id, "invalid").is_err());
    let empty_update_no = db::update_certificate(
        &conn,
        created.id,
        &TrainingCertificateInput {
            certificate_no: Some("   ".to_string()),
            recipient_name: "Peserta Satu".to_string(),
            duration_text: "six-month".to_string(),
            program_name: None,
            start_date: "2026-05-01".to_string(),
            end_date: "2026-10-31".to_string(),
            issued_date: "2026-11-21".to_string(),
            department: None,
            institution: None,
            achievement_text: None,
            notes: None,
            signer_name: None,
            signer_title: None,
            paper_size: None,
        },
    );
    assert!(empty_update_no.is_err());

    let missing_update = db::update_certificate(
        &conn,
        99999,
        &TrainingCertificateInput {
            certificate_no: Some("999/ABR/2026".to_string()),
            recipient_name: "Ghost".to_string(),
            duration_text: "six-month".to_string(),
            program_name: None,
            start_date: "2026-05-01".to_string(),
            end_date: "2026-10-31".to_string(),
            issued_date: "2026-11-21".to_string(),
            department: None,
            institution: None,
            achievement_text: None,
            notes: None,
            signer_name: None,
            signer_title: None,
            paper_size: None,
        },
    );
    assert!(missing_update.is_err());

    // 2. Non-empty pagination clamp when page exceeds total_pages
    let clamped = db::get_paginated_certificates(&conn, None, None, None, 99, 25).expect("clamped");
    assert_eq!(clamped.total_pages, 1);
    assert_eq!(clamped.page, 1);
    assert_eq!(clamped.items.len(), 1);
    drop(conn);

    // 3. Restoring pre-v1.7.0 backup + init_db migrates training_certificates & preserves data
    let legacy_backup = dir.join("pre_v170_backup.sqlite");
    {
        let bconn = rusqlite::Connection::open(&legacy_backup).expect("open backup");
        bconn
            .execute_batch(
                "CREATE TABLE units (id INTEGER PRIMARY KEY, code TEXT UNIQUE NOT NULL, name TEXT NOT NULL, tax_rate REAL DEFAULT 0.1, address TEXT, npwpd TEXT, is_active INTEGER DEFAULT 1);
                 CREATE TABLE transactions (id INTEGER PRIMARY KEY, unit TEXT NOT NULL, date TEXT NOT NULL, amount REAL NOT NULL, notes TEXT, source_file TEXT, created_at TEXT);
                 CREATE TABLE settings (key TEXT PRIMARY KEY, value TEXT NOT NULL);
                 CREATE TABLE users (id INTEGER PRIMARY KEY, username TEXT UNIQUE NOT NULL, pin_hash TEXT NOT NULL, salt TEXT NOT NULL, full_name TEXT NOT NULL, role TEXT NOT NULL, last_login TEXT);
                 INSERT INTO users (username, pin_hash, salt, full_name, role) VALUES ('legacy_admin', 'h', 's', 'Legacy Admin', 'admin');
                 INSERT INTO transactions (unit, date, amount) VALUES ('Bungalows', '2026-05-01', 1500000.0);",
            )
            .expect("seed legacy backup");
    }
    db::restore_database(&db_path, &legacy_backup).expect("restore");
    let restored_conn = db::init_db(&db_path).expect("init restored db");
    let user_preserved: String = restored_conn
        .query_row("SELECT username FROM users WHERE username = 'legacy_admin'", [], |r| r.get(0))
        .expect("legacy_admin preserved");
    assert_eq!(user_preserved, "legacy_admin");
    let tx_preserved: i64 = restored_conn
        .query_row("SELECT COUNT(*) FROM transactions", [], |r| r.get(0))
        .expect("transactions preserved");
    assert_eq!(tx_preserved, 1);
    let post_restore_cert = db::create_certificate(
        &restored_conn,
        &TrainingCertificateInput {
            certificate_no: None,
            recipient_name: "Pasca Restore".to_string(),
            duration_text: "three-month".to_string(),
            program_name: None,
            start_date: "2026-06-01".to_string(),
            end_date: "2026-08-31".to_string(),
            issued_date: "2026-09-01".to_string(),
            department: Some("Front Office".to_string()),
            institution: None,
            achievement_text: None,
            notes: None,
            signer_name: None,
            signer_title: None,
            paper_size: None,
        },
        Some("legacy_admin"),
    )
    .expect("create_certificate works right after restore");
    assert_eq!(post_restore_cert.certificate_no, "001/ABR/2026");

    // 4. Multi-line (>= 5 lines) + digital signature does not overlap last body text baseline
    let sig_sample = dir.join("sig.png");
    fs::write(
        &sig_sample,
        fs::read("src-tauri/assets/cert_logo_default.png")
            .or_else(|_| fs::read("assets/cert_logo_default.png"))
            .expect("read png"),
    )
    .unwrap();
    let sig_file = certificate::validate_and_save_asset(&dir, "signature", &sig_sample).expect("sig");
    let mut sig_settings = HashMap::new();
    sig_settings.insert("certShowSignature".to_string(), "1".to_string());
    sig_settings.insert("certSignatureFile".to_string(), sig_file);

    let dense_data = CertificateRenderData::from_preview_input(
        &CertificatePreviewInput {
            certificate_no: Some("009/ABR/2026".to_string()),
            recipient_name: Some("Ni Luh Putu Sri Wahyuni".to_string()),
            duration_text: Some("six-month".to_string()),
            program_name: Some("international hospitality & culinary training program".to_string()),
            start_date: Some("2026-01-01".to_string()),
            end_date: Some("2026-06-30".to_string()),
            issued_date: Some("2026-07-15".to_string()),
            department: Some("Housekeeping (2 months); Food & Beverage Service (2 months); Front Office (2 months)".to_string()),
            achievement_text: Some("Achieving an Excellent rating in hospitality service, guest relations, and operational excellence.".to_string()),
            signer_name: Some("Agus Sumardi Wijaya".to_string()),
            signer_title: Some("General Manager".to_string()),
            paper_size: Some("A4".to_string()),
            status: Some("active".to_string()),
        },
        &dir,
        &sig_settings,
    );
    let body_lines = certificate::build_body_lines(&dense_data);
    assert!(body_lines.len() >= 5, "expected >= 5 lines, got {}", body_lines.len());
    let svg = certificate::render_certificate_svg(&dense_data);

    let last_line = body_lines.last().unwrap();
    let line_marker = format!(">{}</text>", last_line.replace('&', "&amp;"));
    let text_pos = svg.find(&line_marker).expect("last body line in svg");
    let text_tag_start = svg[..text_pos].rfind("<text ").unwrap();
    let y_attr = svg[text_tag_start..text_pos]
        .split("y=\"")
        .nth(1)
        .unwrap()
        .split('"')
        .next()
        .unwrap()
        .parse::<f64>()
        .unwrap();

    let first_img = svg.find("<image ").unwrap();
    let sig_img_start = svg[first_img + 7..].find("<image ").unwrap() + first_img + 7;
    let sig_img_end = svg[sig_img_start..].find("/>").unwrap() + sig_img_start;
    let sig_y = svg[sig_img_start..sig_img_end]
        .split("y=\"")
        .nth(1)
        .unwrap()
        .split('"')
        .next()
        .unwrap()
        .parse::<f64>()
        .unwrap();

    assert!(
        sig_y > y_attr,
        "signature top y={} must be strictly below last body baseline y={}",
        sig_y,
        y_attr
    );

    let _ = fs::remove_dir_all(&dir);
}
