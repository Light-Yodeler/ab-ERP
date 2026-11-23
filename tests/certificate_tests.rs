use excel_subset_lib::certificate::{
    self, CertificatePreviewInput, CertificateRenderData,
};
use excel_subset_lib::db::{
    self, DepartmentInput, ProgramTypeInput, TrainingCertificateInput,
};
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

    assert_eq!(
        db::get_next_certificate_number(&conn, 2026, 9, Some("FO")).unwrap(),
        "001/ABR-FO/IX/2026"
    );
    assert_eq!(
        db::get_next_certificate_number(&conn, 2025, 11, Some("FO")).unwrap(),
        "001/ABR-FO/XI/2025"
    );

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
            department_code: Some("FO".to_string()),
            program_type_code: Some("INT".to_string()),
            issued_city: Some("Kuta, Lombok".to_string()),
            body_template: None,
        },
        Some("admin"),
    )
    .expect("create c1");
    assert_eq!(c1.certificate_no, "001/ABR-FO/IX/2026");
    assert_eq!(c1.paper_size, "F4");
    assert_eq!(c1.signer_name, "Agus Sumardi Wijaya");
    assert_eq!(c1.signer_title, "General Manager");
    assert_eq!(c1.department_code.as_deref(), Some("FO"));
    assert_eq!(c1.program_type_code.as_deref(), Some("INT"));
    assert_eq!(c1.issued_city.as_deref(), Some("Kuta, Lombok"));
    assert_eq!(
        c1.achievement_text.as_deref(),
        Some("Achieving an Excellent rating in hospitality service and professionalism.")
    );

    assert_eq!(
        db::get_next_certificate_number(&conn, 2026, 9, Some("HK")).unwrap(),
        "002/ABR-HK/IX/2026"
    );

    // Manual jump to 010/ABR-HK/IX/2026 -> next becomes 011/ABR-HK/IX/2026
    let c2 = db::create_certificate(
        &conn,
        &TrainingCertificateInput {
            certificate_no: Some("010/ABR-HK/IX/2026".to_string()),
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
            department_code: Some("HK".to_string()),
            program_type_code: None,
            issued_city: None,
            body_template: None,
        },
        Some("staff"),
    )
    .expect("create c2");
    assert_eq!(c2.certificate_no, "010/ABR-HK/IX/2026");
    assert_eq!(
        db::get_next_certificate_number(&conn, 2026, 9, None).unwrap(),
        "011/ABR/IX/2026"
    );

    // Deleting highest number does NOT reuse sequence 10
    db::delete_certificate(&conn, c2.id).expect("delete c2");
    assert_eq!(
        db::get_next_certificate_number(&conn, 2026, 11, Some("KIT")).unwrap(),
        "011/ABR-KIT/XI/2026"
    );

    // Year reset in 2027
    assert_eq!(
        db::get_next_certificate_number(&conn, 2027, 1, Some("FO")).unwrap(),
        "001/ABR-FO/I/2027"
    );

    // Duplicate number fails cleanly (even different case via COLLATE NOCASE)
    let dup = db::create_certificate(
        &conn,
        &TrainingCertificateInput {
            certificate_no: Some("001/abr-fo/ix/2026".to_string()),
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
            department_code: None,
            program_type_code: None,
            issued_city: None,
            body_template: None,
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

    // Verify idempotent migration on legacy DB missing v1.8.0 columns
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
    let migrated_cols: Vec<String> = migrated_conn
        .prepare("PRAGMA table_info(training_certificates)")
        .unwrap()
        .query_map([], |r| r.get::<_, String>(1))
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    for expected_col in [
        "achievement_text",
        "department_code",
        "program_type_code",
        "issued_city",
        "body_template",
    ] {
        assert!(
            migrated_cols.iter().any(|c| c == expected_col),
            "{} column should be auto-migrated",
            expected_col
        );
    }

    let row1 = db::create_certificate(
        &conn,
        &TrainingCertificateInput {
            certificate_no: None,
            recipient_name: "BAIQ RINA HARTINI".to_string(),
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
            department_code: None,
            program_type_code: Some("OJT".to_string()),
            issued_city: None,
            body_template: None,
        },
        Some("staff"),
    )
    .expect("create row1");
    assert_eq!(row1.certificate_no, "001/ABR-HK/VII/2026");
    assert_eq!(row1.recipient_name, "Baiq Rina Hartini");
    assert_eq!(row1.start_date, "2026-05-01");
    assert_eq!(row1.end_date, "2026-07-31");
    assert_eq!(row1.issued_date, "2026-07-31");
    assert_eq!(row1.status, "active");

    let row2 = db::create_certificate(
        &conn,
        &TrainingCertificateInput {
            certificate_no: None,
            recipient_name: "lalu ahmad fauzan".to_string(),
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
            department_code: None,
            program_type_code: None,
            issued_city: Some("Praya, Lombok Tengah".to_string()),
            body_template: None,
        },
        Some("admin"),
    )
    .expect("create row2");
    assert_eq!(row2.certificate_no, "001/ABR-ROT/VII/2027");
    assert_eq!(row2.recipient_name, "Lalu Ahmad Fauzan");
    assert_eq!(row2.issued_city.as_deref(), Some("Praya, Lombok Tengah"));

    // Certificate number is permanently locked after issuance
    let reject_no_change = db::update_certificate(
        &conn,
        row1.id,
        &TrainingCertificateInput {
            certificate_no: Some("005/ABR-FO/IX/2026".to_string()),
            recipient_name: "baiq rina hartini, a.md.par.".to_string(),
            duration_text: "four-month".to_string(),
            program_name: Some("hospitality training program".to_string()),
            start_date: "2026-05-01".to_string(),
            end_date: "2026-08-31".to_string(),
            issued_date: "2026-09-01".to_string(),
            department: Some("Front Office (2 months); Kitchen (2 months)".to_string()),
            institution: Some("SMKN 1 Pujut".to_string()),
            achievement_text: Some("Excellent Hospitality Rating".to_string()),
            notes: Some("Revised".to_string()),
            signer_name: Some("Agus Sumardi Wijaya".to_string()),
            signer_title: Some("General Manager".to_string()),
            paper_size: Some("F4".to_string()),
            department_code: Some("ROT".to_string()),
            program_type_code: Some("INT".to_string()),
            issued_city: Some("Kuta, Lombok".to_string()),
            body_template: None,
        },
    );
    assert!(
        reject_no_change.as_ref().unwrap_err().contains("terkunci"),
        "expected locked number error, got: {:?}",
        reject_no_change
    );

    // Editing a 1-department certificate into a rotation keeps original ABR-HK number
    let updated = db::update_certificate(
        &conn,
        row1.id,
        &TrainingCertificateInput {
            certificate_no: Some(row1.certificate_no.clone()),
            recipient_name: "baiq rina hartini, a.md.par.".to_string(),
            duration_text: "four-month".to_string(),
            program_name: Some("hospitality training program".to_string()),
            start_date: "2026-05-01".to_string(),
            end_date: "2026-08-31".to_string(),
            issued_date: "2026-09-01".to_string(),
            department: Some("Front Office (2 months); Kitchen (2 months)".to_string()),
            institution: Some("SMKN 1 Pujut".to_string()),
            achievement_text: Some("Excellent Hospitality Rating".to_string()),
            notes: Some("Revised".to_string()),
            signer_name: Some("Agus Sumardi Wijaya".to_string()),
            signer_title: Some("General Manager".to_string()),
            paper_size: Some("F4".to_string()),
            department_code: Some("ROT".to_string()),
            program_type_code: Some("INT".to_string()),
            issued_city: Some("Kuta, Lombok".to_string()),
            body_template: None,
        },
    )
    .expect("update keeping locked certificate_no");
    assert_eq!(updated.certificate_no, "001/ABR-HK/VII/2026");
    assert_eq!(updated.department_code.as_deref(), Some("ROT"));
    assert_eq!(updated.recipient_name, "Baiq Rina Hartini, A.Md.Par.");
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
    assert_eq!(certificate::format_divisions_phrase(None), None);
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

    // Case 1: Single division + Issued on [Date], [City] line rendered below body
    let single_same_day = CertificateRenderData::from_preview_input(
        &CertificatePreviewInput {
            certificate_no: Some("001/ABR-FO/XI/2026".to_string()),
            recipient_name: Some("M. Syahid & Partners <VIP>".to_string()),
            duration_text: Some("six-month".to_string()),
            program_name: None,
            start_date: Some("2026-06-01".to_string()),
            end_date: Some("2026-11-21".to_string()),
            issued_date: Some("2026-11-21".to_string()),
            issued_city: Some("Kuta, Lombok".to_string()),
            body_template: None,
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
    assert_eq!(
        certificate::format_issued_line(&single_same_day).as_deref(),
        Some("Issued in Kuta, Lombok, on November 21, 2026")
    );

    let f4_svg = certificate::render_certificate_svg(&single_same_day);
    assert!(f4_svg.contains("width=\"330mm\" height=\"215mm\" viewBox=\"0 0 330.00 215.00\""));
    assert!(f4_svg.contains("width=\"310.00\" height=\"195.00\" fill=\"none\" stroke=\"#C67D24\""));
    assert!(f4_svg.contains("Certificate No. 001/ABR-FO/XI/2026"));
    assert!(f4_svg.contains("M. Syahid &amp; Partners &lt;Vip&gt;"));
    assert!(f4_svg.contains("for successfully completing the six-month hospitality training program in the Front Office Department"));
    assert!(f4_svg.contains("at Anda Bungalows &amp; Restaurant, from June 1, 2026 to November 21, 2026."));
    assert!(f4_svg.contains("Achieving an Excellent rating in hospitality service and professionalism."));
    assert!(f4_svg.contains("Issued in Kuta, Lombok, on November 21, 2026"));
    assert!(!f4_svg.contains("Awarded on"));

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

    // Case 2: 2-division rotation + Issued in rendered cleanly
    let rotation_diff_day = CertificateRenderData::from_preview_input(
        &CertificatePreviewInput {
            certificate_no: Some("002/ABR-ROT/XI/2026".to_string()),
            recipient_name: Some("Ni Wayan Sari Putri".to_string()),
            duration_text: Some("six-month".to_string()),
            program_name: None,
            start_date: Some("2026-05-01".to_string()),
            end_date: Some("2026-10-31".to_string()),
            issued_date: Some("2026-11-21".to_string()),
            issued_city: Some("Kuta, Lombok".to_string()),
            body_template: None,
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
    let rot_svg = certificate::render_certificate_svg(&rotation_diff_day);
    assert!(rot_svg.contains("in the Housekeeping Department (3 months) and the Food &amp; Beverage Department (3 months)"));
    assert!(rot_svg.contains("at Anda Bungalows &amp; Restaurant, from May 1, 2026 to October 31, 2026."));
    assert!(rot_svg.contains("Achieving a Very Good rating in hospitality service and professionalism."));
    assert!(rot_svg.contains("Issued in Kuta, Lombok, on November 21, 2026"));

    // Case 3: 3-division rotation readability on both F4 and A4
    let three_div_f4 = CertificateRenderData::from_preview_input(
        &CertificatePreviewInput {
            certificate_no: Some("003/ABR-ROT/XI/2026".to_string()),
            recipient_name: Some("Ni Kadek Ayu Sari".to_string()),
            duration_text: Some("six-month".to_string()),
            program_name: Some("internship program".to_string()),
            start_date: Some("2026-05-01".to_string()),
            end_date: Some("2026-10-31".to_string()),
            issued_date: Some("2026-11-21".to_string()),
            issued_city: Some("Kuta, Lombok".to_string()),
            body_template: None,
            department: Some("Housekeeping (2 months); Kitchen (2 months); Front Office (2 months)".to_string()),
            achievement_text: None,
            signer_name: None,
            signer_title: None,
            paper_size: Some("F4".to_string()),
            status: None,
        },
        &dir,
        &settings,
    );
    let three_div_lines = certificate::build_body_lines(&three_div_f4);
    let three_joined = three_div_lines.join(" ");
    assert!(three_joined.contains("for successfully completing the six-month internship program"));
    assert!(three_joined.contains("in the Housekeeping Department (2 months), the Kitchen Department (2 months), and the Front Office Department (2 months)"));
    assert!(three_joined.contains("at Anda Bungalows & Restaurant, from May 1, 2026 to October 31, 2026."));

    let mut three_div_a4 = three_div_f4.clone();
    three_div_a4.paper_size = "A4".to_string();
    assert!(certificate::render_certificate_svg(&three_div_a4).contains("Issued in Kuta, Lombok, on November 21, 2026"));

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
            certificate_no: Some("001/ABR-FO/XI/2026".to_string()),
            recipient_name: Some("Baiq Rina Hartini".to_string()),
            duration_text: Some("six-month".to_string()),
            program_name: Some("hospitality training program".to_string()),
            start_date: Some("2026-06-01".to_string()),
            end_date: Some("2026-11-21".to_string()),
            issued_date: Some("2026-11-21".to_string()),
            issued_city: None,
            body_template: None,
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
            certificate_no: Some("002/ABR-HK/XI/2026".to_string()),
            recipient_name: Some("Ni Wayan Sari Putri".to_string()),
            duration_text: Some("six-month".to_string()),
            program_name: Some("hospitality training program".to_string()),
            start_date: Some("2026-05-01".to_string()),
            end_date: Some("2026-10-31".to_string()),
            issued_date: Some("2026-11-21".to_string()),
            issued_city: None,
            body_template: None,
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
            certificate_no: Some("001/ABR-HK/XI/2026".to_string()),
            recipient_name: Some("Maulana".to_string()),
            duration_text: Some("six-month".to_string()),
            program_name: Some("hospitality training program".to_string()),
            start_date: Some("2026-06-01".to_string()),
            end_date: Some("2026-11-22".to_string()),
            issued_date: Some("2026-11-22".to_string()),
            issued_city: None,
            body_template: None,
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
            department_code: None,
            program_type_code: None,
            issued_city: None,
            body_template: None,
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
            department_code: None,
            program_type_code: None,
            issued_city: None,
            body_template: None,
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
            department_code: None,
            program_type_code: None,
            issued_city: None,
            body_template: None,
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
            department_code: None,
            program_type_code: None,
            issued_city: None,
            body_template: None,
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
            department_code: None,
            program_type_code: None,
            issued_city: None,
            body_template: None,
        },
    );
    assert!(empty_update_no.is_err());

    let missing_update = db::update_certificate(
        &conn,
        99999,
        &TrainingCertificateInput {
            certificate_no: Some("999/ABR/I/2026".to_string()),
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
            department_code: None,
            program_type_code: None,
            issued_city: None,
            body_template: None,
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
            department_code: None,
            program_type_code: None,
            issued_city: None,
            body_template: None,
        },
        Some("legacy_admin"),
    )
    .expect("create_certificate works right after restore");
    assert_eq!(post_restore_cert.certificate_no, "001/ABR-FO/IX/2026");

    // 4. Multi-line (>= 5 lines) + Issued on line + digital signature does not overlap last text
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
            certificate_no: Some("009/ABR-HK/VII/2026".to_string()),
            recipient_name: Some("Ni Luh Putu Sri Wahyuni".to_string()),
            duration_text: Some("six-month".to_string()),
            program_name: Some("international hospitality & culinary training program".to_string()),
            start_date: Some("2026-01-01".to_string()),
            end_date: Some("2026-06-30".to_string()),
            issued_date: Some("2026-07-15".to_string()),
            issued_city: Some("Kuta, Lombok".to_string()),
            body_template: None,
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

    let issued_marker = ">Issued in Kuta, Lombok, on July 15, 2026</text>";
    let issued_pos = svg.find(issued_marker).expect("issued line in svg");
    let issued_tag_start = svg[..issued_pos].rfind("<text ").unwrap();
    let issued_y = svg[issued_tag_start..issued_pos]
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
    let sig_tag = &svg[sig_img_start..sig_img_end];
    let sig_y = sig_tag
        .split("y=\"")
        .nth(1)
        .unwrap()
        .split('"')
        .next()
        .unwrap()
        .parse::<f64>()
        .unwrap();
    let sig_h = sig_tag
        .split("height=\"")
        .nth(1)
        .unwrap()
        .split('"')
        .next()
        .unwrap()
        .parse::<f64>()
        .unwrap();

    assert!(
        (181.90 - issued_y) >= 24.0,
        "expected >= 24mm signature clearance above signer top, got {} (issued_y={})",
        181.90 - issued_y,
        issued_y
    );
    assert!(
        sig_y > issued_y,
        "signature top y={} must be strictly below Issued on baseline y={}",
        sig_y,
        issued_y
    );
    assert!(
        sig_y + sig_h <= 185.90 - 4.0 + 0.05,
        "signature bottom {} must stay above signer top",
        sig_y + sig_h
    );

    // M5: Ultra-dense >=9 visual lines A4 keeps signature within signer clearance or omits cleanly
    let mut ultra_dense = dense_data.clone();
    ultra_dense.achievement_text = Some("Achieving an Excellent rating in hospitality operations, international guest relations, culinary standards, housekeeping audits, front desk reservations, and supervisory leadership across three rotating hotel departments.".to_string());
    let ultra_svg = certificate::render_certificate_svg(&ultra_dense);
    assert!(certificate::build_body_lines(&ultra_dense).len() >= 7);
    let u_first_img = ultra_svg.find("<image ").unwrap();
    if let Some(rel_sig) = ultra_svg[u_first_img + 7..].find("<image ") {
        let u_start = u_first_img + 7 + rel_sig;
        let u_end = ultra_svg[u_start..].find("/>").unwrap() + u_start;
        let u_tag = &ultra_svg[u_start..u_end];
        let u_y: f64 = u_tag.split("y=\"").nth(1).unwrap().split('"').next().unwrap().parse().unwrap();
        let u_h: f64 = u_tag.split("height=\"").nth(1).unwrap().split('"').next().unwrap().parse().unwrap();
        assert!(u_y + u_h <= 181.95, "ultra-dense signature overlapped signer: {} + {}", u_y, u_h);
    }

    // H4: Manual certificate_no without '/' must NOT corrupt certificate_sequences for 2026
    let seq_conn = db::init_db(&dir.join("seq_guard.sqlite")).expect("seq db");
    db::create_certificate(
        &seq_conn,
        &TrainingCertificateInput {
            certificate_no: Some("2026".to_string()),
            recipient_name: "Manual Format".to_string(),
            duration_text: "one-month".to_string(),
            program_name: None,
            start_date: "2026-01-01".to_string(),
            end_date: "2026-01-31".to_string(),
            issued_date: "2026-02-01".to_string(),
            department: None,
            institution: None,
            achievement_text: None,
            notes: None,
            signer_name: None,
            signer_title: None,
            paper_size: None,
            department_code: None,
            program_type_code: None,
            issued_city: None,
            body_template: None,
        },
        Some("admin"),
    )
    .expect("create manual without slash");
    assert_eq!(
        db::get_next_certificate_number(&seq_conn, 2026, 2, Some("FO")).unwrap(),
        "001/ABR-FO/II/2026"
    );
    // 3-division rotation automatically assigns ROT code
    let rot3 = db::create_certificate(
        &seq_conn,
        &TrainingCertificateInput {
            certificate_no: None,
            recipient_name: "Peserta Rotasi".to_string(),
            duration_text: "six-month".to_string(),
            program_name: None,
            start_date: "2026-01-01".to_string(),
            end_date: "2026-06-30".to_string(),
            issued_date: "2026-07-01".to_string(),
            department: Some("Housekeeping (2 months); Kitchen (2 months); Bar (2 months)".to_string()),
            institution: None,
            achievement_text: None,
            notes: None,
            signer_name: None,
            signer_title: None,
            paper_size: None,
            department_code: None,
            program_type_code: Some("MT".to_string()),
            issued_city: None,
            body_template: None,
        },
        Some("admin"),
    )
    .expect("rot3");
    assert_eq!(rot3.certificate_no, "001/ABR-ROT/VII/2026");

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_catalog_departments_and_program_types_crud() {
    let dir = temp_test_dir("catalog");
    let db_path = dir.join("cert_catalog.sqlite");
    let conn = db::init_db(&db_path).expect("init_db");

    // 10 seeded departments & 7 seeded program types (complete English copy)
    let depts = db::get_departments(&conn).expect("get_departments");
    assert_eq!(depts.len(), 10);
    let kit = depts.iter().find(|d| d.code == "KIT").expect("KIT seeded");
    let fo = depts.iter().find(|d| d.code == "FO").expect("FO seeded");
    assert!(kit.program_description.starts_with("The program covered "));
    assert!(kit.competencies.contains("Basic cooking techniques"));
    assert!(fo.competencies.contains("Guest registration"));
    assert_ne!(kit.program_description, fo.program_description);
    assert_ne!(kit.competencies, fo.competencies);

    let progs = db::get_program_types(&conn).expect("get_program_types");
    assert_eq!(progs.len(), 7);
    let ojt = progs.iter().find(|p| p.code == "OJT").expect("OJT");
    assert!(progs.iter().any(|p| p.code == "INT"));
    assert!(progs.iter().any(|p| p.code == "WS"));

    // H1: Updating KIT or OJT with sort_order: None must preserve sort_order == 10
    let edited_kit = db::save_department(
        &conn,
        &DepartmentInput {
            id: Some(kit.id),
            code: kit.code.clone(),
            name: kit.name.clone(),
            program_description: kit.program_description.clone(),
            competencies: kit.competencies.clone(),
            is_active: None,
            sort_order: None,
        },
    )
    .expect("edit kit preserves sort_order");
    assert_eq!(edited_kit.sort_order, 10);
    let edited_ojt = db::save_program_type(
        &conn,
        &ProgramTypeInput {
            id: Some(ojt.id),
            code: ojt.code.clone(),
            name: ojt.name.clone(),
            default_duration_months: ojt.default_duration_months,
            description: ojt.description.clone(),
            is_active: None,
            sort_order: None,
        },
    )
    .expect("edit ojt preserves sort_order");
    assert_eq!(edited_ojt.sort_order, 10);

    // Upgrade early v1.8.0 "Training in..." / Indonesian seed without overwriting custom edits
    let v180_legacy_path = dir.join("legacy_v180_seed.sqlite");
    {
        let lconn = db::init_db(&v180_legacy_path).expect("init v180");
        lconn.execute(
            "UPDATE departments SET program_description = 'Training in food preparation, kitchen hygiene, and menu production', competencies = 'Teknik memasak dasar\nFood safety (HACCP)\nPenanganan bahan\nPlating' WHERE code = 'KIT'",
            [],
        ).unwrap();
        lconn.execute(
            "UPDATE departments SET program_description = 'Custom FO description kept intact' WHERE code = 'FO'",
            [],
        ).unwrap();
    }
    let upgraded_conn = db::init_db(&v180_legacy_path).expect("upgrade seed");
    let up_depts = db::get_departments(&upgraded_conn).unwrap();
    assert_eq!(
        up_depts.iter().find(|d| d.code == "KIT").unwrap().program_description,
        "The program covered food preparation, kitchen hygiene, and menu production."
    );
    assert_eq!(
        up_depts.iter().find(|d| d.code == "FO").unwrap().program_description,
        "Custom FO description kept intact"
    );

    // Create, update, duplicate check, and delete department
    let created_dept = db::save_department(
        &conn,
        &DepartmentInput {
            id: None,
            code: "bqt".to_string(),
            name: "Banquet & Events".to_string(),
            program_description: "The program covered banquet setup and event coordination.".to_string(),
            competencies: "Banquet setup\nEvent coordination".to_string(),
            is_active: Some(true),
            sort_order: None,
        },
    )
    .expect("create dept");
    assert_eq!(created_dept.code, "BQT");

    let dup_dept = db::save_department(
        &conn,
        &DepartmentInput {
            id: None,
            code: "BQT".to_string(),
            name: "Duplicate".to_string(),
            program_description: String::new(),
            competencies: String::new(),
            is_active: None,
            sort_order: None,
        },
    );
    assert!(dup_dept.is_err());

    let updated_dept = db::save_department(
        &conn,
        &DepartmentInput {
            id: Some(created_dept.id),
            code: "BQT".to_string(),
            name: "Banquet Operations".to_string(),
            program_description: "Updated desc".to_string(),
            competencies: "Stage setup".to_string(),
            is_active: Some(true),
            sort_order: Some(110),
        },
    )
    .expect("update dept");
    assert_eq!(updated_dept.name, "Banquet Operations");

    db::delete_department(&conn, created_dept.id).expect("delete dept");
    assert_eq!(db::get_departments(&conn).unwrap().len(), 10);

    // Program Type CRUD
    let created_pt = db::save_program_type(
        &conn,
        &ProgramTypeInput {
            id: None,
            code: "voc".to_string(),
            name: "vocational hospitality program".to_string(),
            default_duration_months: 4,
            description: "Magang SMK 4 bulan".to_string(),
            is_active: Some(true),
            sort_order: None,
        },
    )
    .expect("create pt");
    assert_eq!(created_pt.code, "VOC");
    assert_eq!(created_pt.default_duration_months, 4);

    db::delete_program_type(&conn, created_pt.id).expect("delete pt");
    assert_eq!(db::get_program_types(&conn).unwrap().len(), 7);

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_recipient_name_normalization_and_body_template() {
    assert_eq!(certificate::normalize_recipient_name("maulana"), "Maulana");
    assert_eq!(
        certificate::normalize_recipient_name("NI LUH PUTU SRI WAHYUNI"),
        "Ni Luh Putu Sri Wahyuni"
    );
    assert_eq!(
        certificate::normalize_recipient_name("dr. i made putra"),
        "dr. I Made Putra"
    );
    assert_eq!(
        certificate::normalize_recipient_name("Dr. i made putra, S.Tr.Par."),
        "Dr. I Made Putra, S.Tr.Par."
    );
    assert_eq!(
        certificate::normalize_recipient_name("prof. dr. hj. r.a. kartini, s.e., m.m."),
        "Prof. Dr. Hj. R.A. Kartini, S.E., M.M."
    );
    assert_eq!(
        certificate::normalize_recipient_name("ahmad al-farizi bin hasan, a.md.par."),
        "Ahmad Al-Farizi bin Hasan, A.Md.Par."
    );
    assert_eq!(
        certificate::normalize_recipient_name("john o'connor, mba"),
        "John O'Connor, MBA"
    );
    assert_eq!(
        certificate::normalize_recipient_name("k.h. ma'ruf amin"),
        "K.H. Ma'ruf Amin"
    );
    assert_eq!(
        certificate::normalize_recipient_name("H. la ode budi"),
        "H. La Ode Budi"
    );
    assert_eq!(
        certificate::normalize_recipient_name("dr. budi santoso, Sp.PD"),
        "dr. Budi Santoso, Sp.PD"
    );

    let dir = temp_test_dir("tpl");
    let settings = HashMap::new();

    // 1. Workshop single-day + no department collapses "in the {departemen}" cleanly
    let workshop_data = CertificateRenderData::from_preview_input(
        &CertificatePreviewInput {
            certificate_no: Some("004/ABR-FO/XI/2025".to_string()),
            recipient_name: Some("budi santoso, s.par.".to_string()),
            duration_text: Some("one-day".to_string()),
            program_name: Some("short course / workshop".to_string()),
            start_date: Some("2025-11-12".to_string()),
            end_date: Some("2025-11-12".to_string()),
            issued_date: Some("2025-11-12".to_string()),
            issued_city: Some("Kuta, Lombok".to_string()),
            body_template: Some(certificate::DEFAULT_BODY_TEMPLATE.to_string()),
            department: None,
            achievement_text: None,
            signer_name: None,
            signer_title: None,
            paper_size: Some("F4".to_string()),
            status: None,
        },
        &dir,
        &settings,
    );
    assert_eq!(workshop_data.recipient_name, "Budi Santoso, S.Par.");
    let ws_rendered = certificate::render_body_template(&workshop_data);
    assert_eq!(
        ws_rendered,
        "for successfully completing the one-day short course / workshop at Anda Bungalows & Restaurant, on November 12, 2025."
    );

    // 2. Custom variable template with {tanggal_mulai}/{tanggal_selesai}
    let custom_data = CertificateRenderData::from_preview_input(
        &CertificatePreviewInput {
            certificate_no: Some("004/ABR-FO/XI/2025".to_string()),
            recipient_name: Some("siti aisyah".to_string()),
            duration_text: Some("three-month".to_string()),
            program_name: Some("internship program".to_string()),
            start_date: Some("2025-03-01".to_string()),
            end_date: Some("2025-05-31".to_string()),
            issued_date: Some("2025-06-01".to_string()),
            issued_city: Some("Mataram, Lombok".to_string()),
            body_template: Some(
                "has completed the {durasi} {jenis_program} in the {departemen} at {properti} ({tanggal_mulai} - {tanggal_selesai})."
                    .to_string(),
            ),
            department: Some("Front Office".to_string()),
            achievement_text: None,
            signer_name: None,
            signer_title: None,
            paper_size: Some("A4".to_string()),
            status: None,
        },
        &dir,
        &settings,
    );
    let custom_lines = certificate::build_body_lines(&custom_data);
    assert!(custom_lines
        .join(" ")
        .contains("has completed the three-month internship program in the Front Office Department at Anda Bungalows & Restaurant (March 1, 2025 - May 31, 2025)."));
    let custom_svg = certificate::render_certificate_svg(&custom_data);
    assert!(custom_svg.contains("Issued in Mataram, Lombok, on June 1, 2025"));

    // 3. Business name in settings drives SVG header brand & {properti} template variable
    let mut mixed_settings = HashMap::new();
    mixed_settings.insert("businessName".to_string(), "Kuta Sunset Resort".to_string());
    let mixed_prop_data = CertificateRenderData::from_preview_input(
        &CertificatePreviewInput {
            certificate_no: Some("005/ABR-FO/XI/2025".to_string()),
            recipient_name: Some("Siti Aisyah".to_string()),
            duration_text: Some("three-month".to_string()),
            program_name: Some("internship program".to_string()),
            start_date: Some("2025-03-01".to_string()),
            end_date: Some("2025-05-31".to_string()),
            issued_date: Some("2025-06-01".to_string()),
            issued_city: Some("Kuta, Lombok".to_string()),
            body_template: None,
            department: Some("Front Office".to_string()),
            achievement_text: None,
            signer_name: None,
            signer_title: None,
            paper_size: Some("F4".to_string()),
            status: None,
        },
        &dir,
        &mixed_settings,
    );
    let mixed_svg = certificate::render_certificate_svg(&mixed_prop_data);
    assert!(mixed_svg.contains(">KUTA SUNSET RESORT</text>"));
    assert!(mixed_svg.contains("at Kuta Sunset Resort, from March 1, 2025 to May 31, 2025."));

    let mut caps_settings = HashMap::new();
    caps_settings.insert("businessName".to_string(), "PT BALI PARADISE RESORT".to_string());
    let caps_prop_data = CertificateRenderData::from_preview_input(
        &CertificatePreviewInput {
            certificate_no: Some("006/ABR-FO/XI/2025".to_string()),
            recipient_name: Some("Siti Aisyah".to_string()),
            duration_text: Some("three-month".to_string()),
            program_name: Some("internship program".to_string()),
            start_date: Some("2025-03-01".to_string()),
            end_date: Some("2025-05-31".to_string()),
            issued_date: Some("2025-06-01".to_string()),
            issued_city: Some("Kuta, Lombok".to_string()),
            body_template: None,
            department: Some("Front Office".to_string()),
            achievement_text: None,
            signer_name: None,
            signer_title: None,
            paper_size: Some("F4".to_string()),
            status: None,
        },
        &dir,
        &caps_settings,
    );
    let caps_svg = certificate::render_certificate_svg(&caps_prop_data);
    assert!(caps_svg.contains(">PT BALI PARADISE RESORT</text>"));
    assert!(caps_svg.contains("at PT Bali Paradise Resort, from March 1, 2025 to May 31, 2025."));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_certificate_number_lock_and_division_duration_policy() {
    let dir = temp_test_dir("lock_dur_policy");
    let db_path = dir.join("cert_policy.sqlite");
    let conn = db::init_db(&db_path).expect("init_db");

    // 1. Issue single-department certificate (ABR-FO)
    let issued = db::create_certificate(
        &conn,
        &TrainingCertificateInput {
            certificate_no: None,
            recipient_name: "Andi Pratama".to_string(),
            duration_text: "six-month".to_string(),
            program_name: None,
            start_date: "2026-01-01".to_string(),
            end_date: "2026-06-30".to_string(),
            issued_date: "2026-06-30".to_string(),
            department: Some("Front Office".to_string()),
            institution: None,
            achievement_text: None,
            notes: None,
            signer_name: None,
            signer_title: None,
            paper_size: Some("F4".to_string()),
            department_code: Some("FO".to_string()),
            program_type_code: Some("OJT".to_string()),
            issued_city: None,
            body_template: None,
        },
        Some("admin"),
    )
    .expect("issue single-dept certificate");
    assert_eq!(issued.certificate_no, "001/ABR-FO/VI/2026");

    // Attempting to mutate certificate_no to ABR-ROT after issuance fails
    let mut rot_attempt = TrainingCertificateInput {
        certificate_no: Some("001/ABR-ROT/VI/2026".to_string()),
        recipient_name: "Andi Pratama".to_string(),
        duration_text: "six-month".to_string(),
        program_name: None,
        start_date: "2026-01-01".to_string(),
        end_date: "2026-06-30".to_string(),
        issued_date: "2026-06-30".to_string(),
        department: Some("Front Office (3 months); Kitchen (3 months)".to_string()),
        institution: None,
        achievement_text: None,
        notes: None,
        signer_name: None,
        signer_title: None,
        paper_size: Some("F4".to_string()),
        department_code: Some("ROT".to_string()),
        program_type_code: Some("OJT".to_string()),
        issued_city: None,
        body_template: None,
    };
    let err_rot_no = db::update_certificate(&conn, issued.id, &rot_attempt).unwrap_err();
    assert!(err_rot_no.contains("terkunci"));

    // Updating into a 2-division rotation with certificate_no: None or unchanged keeps ABR-FO
    rot_attempt.certificate_no = None;
    let updated_rot_none = db::update_certificate(&conn, issued.id, &rot_attempt)
        .expect("update to rotation with None cert_no");
    assert_eq!(updated_rot_none.certificate_no, "001/ABR-FO/VI/2026");
    assert_eq!(updated_rot_none.department_code.as_deref(), Some("ROT"));

    // 2. Exceeding main period blocks both create and update
    let mut over_input = rot_attempt.clone();
    over_input.certificate_no = None;
    over_input.department = Some("Front Office (4 months); Kitchen (4 months)".to_string());
    let over_create_err = db::create_certificate(&conn, &over_input, Some("admin")).unwrap_err();
    assert!(
        over_create_err.contains("melebihi periode utama"),
        "expected duration block on create, got: {}",
        over_create_err
    );

    let single_over_err = db::create_certificate(
        &conn,
        &TrainingCertificateInput {
            department: Some("Kitchen (8 months)".to_string()),
            department_code: Some("KIT".to_string()),
            ..over_input.clone()
        },
        Some("admin"),
    )
    .unwrap_err();
    assert!(single_over_err.contains("melebihi periode utama"));

    over_input.certificate_no = Some(issued.certificate_no.clone());
    let over_update_err = db::update_certificate(&conn, issued.id, &over_input).unwrap_err();
    assert!(over_update_err.contains("melebihi periode utama"));

    // 3. Under-total division durations remain allowed (warning-only on frontend)
    let under_ok = db::create_certificate(
        &conn,
        &TrainingCertificateInput {
            certificate_no: None,
            department: Some("Front Office (2 months); Kitchen (2 months)".to_string()),
            ..over_input.clone()
        },
        Some("admin"),
    )
    .expect("under-total divisions should be allowed");
    assert_eq!(under_ok.certificate_no, "002/ABR-ROT/VI/2026");

    // Equivalent week units (26 weeks ~= 6.0 months) within tolerance succeed
    let weeks_ok = db::create_certificate(
        &conn,
        &TrainingCertificateInput {
            certificate_no: None,
            department: Some("Front Office (13 weeks); Kitchen (13 weeks)".to_string()),
            ..over_input.clone()
        },
        Some("admin"),
    )
    .expect("equivalent week durations should succeed");
    assert_eq!(weeks_ok.certificate_no, "003/ABR-ROT/VI/2026");

    // Nested parentheses in division duration must still be parsed & blocked
    let nested_paren_over = db::create_certificate(
        &conn,
        &TrainingCertificateInput {
            certificate_no: None,
            department: Some("Kitchen (4 months (full-time)); Front Office (4 months)".to_string()),
            ..over_input.clone()
        },
        Some("admin"),
    )
    .unwrap_err();
    assert!(nested_paren_over.contains("melebihi periode utama"));

    // Oversized or malformed manual certificate_no must NOT overflow or hijack 2026 sequence
    db::create_certificate(
        &conn,
        &TrainingCertificateInput {
            certificate_no: Some("4294967295/ABR-FO/XI/2026".to_string()),
            recipient_name: "Oversized Manual".to_string(),
            department: Some("Front Office".to_string()),
            department_code: Some("FO".to_string()),
            ..over_input.clone()
        },
        Some("admin"),
    )
    .expect("oversized manual stored as literal without poisoning sequence");
    db::create_certificate(
        &conn,
        &TrainingCertificateInput {
            certificate_no: Some("2026/2026".to_string()),
            recipient_name: "Two Seg Manual".to_string(),
            department: Some("Front Office".to_string()),
            department_code: Some("FO".to_string()),
            ..over_input
        },
        Some("admin"),
    )
    .expect("two-segment manual without corrupting sequence");
    assert_eq!(
        db::get_next_certificate_number(&conn, 2026, 11, Some("FO")).unwrap(),
        "004/ABR-FO/XI/2026"
    );

    // 4. Legacy English competency strings ("Plating and presentation", "Garden and pool upkeep")
    // upgrade idempotently without touching custom rows
    let custom_fo_comp = "Custom FO skill\nVIP Check-in";
    conn.execute(
        "UPDATE departments SET competencies = ?1 WHERE code = 'KIT'",
        rusqlite::params!["Basic cooking techniques\nFood safety (HACCP)\nIngredient handling\nPlating and presentation"],
    )
    .unwrap();
    conn.execute(
        "UPDATE departments SET competencies = ?1 WHERE code = 'MNT'",
        rusqlite::params!["Electrical maintenance\nPlumbing repairs\nAir conditioning maintenance\nGarden and pool upkeep"],
    )
    .unwrap();
    conn.execute(
        "UPDATE departments SET competencies = ?1 WHERE code = 'FO'",
        rusqlite::params![custom_fo_comp],
    )
    .unwrap();
    drop(conn);

    let reinit = db::init_db(&db_path).expect("re-init_db");
    let upgraded_depts = db::get_departments(&reinit).expect("depts after reinit");
    let kit = upgraded_depts.iter().find(|d| d.code == "KIT").unwrap();
    let mnt = upgraded_depts.iter().find(|d| d.code == "MNT").unwrap();
    let fo = upgraded_depts.iter().find(|d| d.code == "FO").unwrap();
    assert!(kit.competencies.ends_with("Food plating"));
    assert!(mnt.competencies.ends_with("Pool & garden upkeep"));
    assert_eq!(fo.competencies, custom_fo_comp);

    let _ = fs::remove_dir_all(&dir);
}
