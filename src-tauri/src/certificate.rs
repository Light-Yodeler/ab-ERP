use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use chrono::Datelike;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use svg2pdf::usvg;

const DEFAULT_LOGO_PNG: &[u8] = include_bytes!("../assets/cert_logo_default.png");

const FONT_NUNITO_SANS_CONDENSED: &[u8] =
    include_bytes!("../../src/assets/fonts/NunitoSansCondensed-Regular.ttf");
const FONT_SOURCE_SERIF_REGULAR: &[u8] =
    include_bytes!("../../src/assets/fonts/SourceSerifPro-Regular.ttf");
const FONT_SOURCE_SERIF_BOLD: &[u8] =
    include_bytes!("../../src/assets/fonts/SourceSerifPro-Bold.ttf");
const FONT_LIBRE_BASKERVILLE: &[u8] =
    include_bytes!("../../src/assets/fonts/LibreBaskerville-Regular.ttf");
const FONT_ROBOTO_REGULAR: &[u8] =
    include_bytes!("../../src/assets/fonts/Roboto-Regular.ttf");

const MM_PER_PT: f64 = 25.4 / 72.0;
const MAX_ASSET_BYTES: u64 = 5 * 1024 * 1024;
const ALLOWED_LOGO_FILES: &[&str] = &["custom_logo.png", "custom_logo.jpg"];
const ALLOWED_SIGNATURE_FILES: &[&str] = &["custom_signature.png", "custom_signature.jpg"];

const ENGLISH_MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificatePreviewInput {
    pub certificate_no: Option<String>,
    pub recipient_name: Option<String>,
    pub duration_text: Option<String>,
    pub program_name: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub issued_date: Option<String>,
    #[serde(default)]
    pub department: Option<String>,
    #[serde(default)]
    pub achievement_text: Option<String>,
    pub signer_name: Option<String>,
    pub signer_title: Option<String>,
    pub paper_size: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateSettingsPayload {
    pub signer_name: String,
    pub signer_title: String,
    pub program_name: String,
    pub paper_size: String,
    pub show_signature: bool,
    pub has_custom_logo: bool,
    pub has_custom_signature: bool,
    pub logo_data_uri: String,
    pub signature_data_uri: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CertificateRenderData {
    pub certificate_no: String,
    pub recipient_name: String,
    pub duration_text: String,
    pub program_name: String,
    pub issued_date: String,
    pub date_range_display: String,
    pub show_awarded_line: bool,
    pub department: Option<String>,
    pub achievement_text: Option<String>,
    pub signer_name: String,
    pub signer_title: String,
    pub paper_size: String,
    pub logo_data_uri: String,
    pub signature_data_uri: Option<String>,
    pub show_signature: bool,
    pub is_revoked: bool,
}

impl CertificateRenderData {
    /// Builds render-ready certificate data from live form input and stored settings.
    pub fn from_preview_input(
        input: &CertificatePreviewInput,
        cert_assets_dir: &Path,
        settings: &HashMap<String, String>,
    ) -> Self {
        let non_empty = |opt: &Option<String>, fallback: &str| -> String {
            opt.as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .unwrap_or(fallback)
                .to_string()
        };
        let clean_opt = |opt: &Option<String>| -> Option<String> {
            opt.as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty() && *s != "-")
                .map(|s| s.to_string())
        };

        let default_program = settings
            .get("certProgramName")
            .map(String::as_str)
            .filter(|s| !s.trim().is_empty())
            .unwrap_or("hospitality training program");
        let default_signer = settings
            .get("certSignerName")
            .map(String::as_str)
            .filter(|s| !s.trim().is_empty())
            .unwrap_or("Agus Sumardi Wijaya");
        let default_title = settings
            .get("certSignerTitle")
            .map(String::as_str)
            .filter(|s| !s.trim().is_empty())
            .unwrap_or("General Manager");
        let default_paper = settings
            .get("certPaperSize")
            .map(String::as_str)
            .filter(|s| *s == "F4" || *s == "A4")
            .unwrap_or("F4");

        let paper_size = match input.paper_size.as_deref().map(str::trim) {
            Some("A4") | Some("a4") => "A4".to_string(),
            Some("F4") | Some("f4") => "F4".to_string(),
            _ => default_paper.to_string(),
        };

        let show_signature = settings
            .get("certShowSignature")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);

        let logo_data_uri = load_logo_data_uri(
            cert_assets_dir,
            settings.get("certLogoFile").map(String::as_str),
        );
        let signature_data_uri = if show_signature {
            load_signature_data_uri(
                cert_assets_dir,
                settings.get("certSignatureFile").map(String::as_str),
            )
        } else {
            None
        };

        let issued_date = match input.issued_date.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
            Some(raw) => format_english_date(raw).unwrap_or_else(|_| raw.to_string()),
            None => "[Date]".to_string(),
        };

        let parsed_end = input
            .end_date
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .and_then(crate::excel_importer::parse_date_string);
        let parsed_issued = input
            .issued_date
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .and_then(crate::excel_importer::parse_date_string);

        let show_awarded_line = match (parsed_issued, parsed_end) {
            (Some(iss), Some(end)) => iss != end,
            (Some(_), None) => true,
            (None, _) => false,
        };

        Self {
            certificate_no: non_empty(&input.certificate_no, "[000/ABR/YYYY]"),
            recipient_name: non_empty(&input.recipient_name, "[Recipient Name]"),
            duration_text: non_empty(&input.duration_text, "[Duration]"),
            program_name: non_empty(&input.program_name, default_program),
            issued_date,
            date_range_display: format_english_date_range(
                input.start_date.as_deref(),
                input.end_date.as_deref(),
            ),
            show_awarded_line,
            department: clean_opt(&input.department),
            achievement_text: clean_opt(&input.achievement_text),
            signer_name: non_empty(&input.signer_name, default_signer),
            signer_title: non_empty(&input.signer_title, default_title),
            paper_size,
            logo_data_uri,
            signature_data_uri,
            show_signature,
            is_revoked: input
                .status
                .as_deref()
                .map(|s| s.eq_ignore_ascii_case("revoked"))
                .unwrap_or(false),
        }
    }
}

/// Loads all 5 bundled OpenType/TrueType certificate fonts into an isolated fontdb.
pub fn load_certificate_fontdb() -> usvg::fontdb::Database {
    let mut db = usvg::fontdb::Database::new();
    db.load_font_data(FONT_NUNITO_SANS_CONDENSED.to_vec());
    db.load_font_data(FONT_SOURCE_SERIF_REGULAR.to_vec());
    db.load_font_data(FONT_SOURCE_SERIF_BOLD.to_vec());
    db.load_font_data(FONT_LIBRE_BASKERVILLE.to_vec());
    db.load_font_data(FONT_ROBOTO_REGULAR.to_vec());
    db
}

/// Verifies that every required Canva certificate font face resolves inside `fontdb`.
pub fn check_bundled_fonts() -> Result<Vec<String>, String> {
    let db = load_certificate_fontdb();
    let checks = [
        ("Nunito Sans Condensed", 400u16, usvg::fontdb::Stretch::Condensed),
        ("Source Serif Pro", 400u16, usvg::fontdb::Stretch::Normal),
        ("Source Serif Pro", 700u16, usvg::fontdb::Stretch::Normal),
        ("Libre Baskerville", 400u16, usvg::fontdb::Stretch::Normal),
        ("Roboto", 400u16, usvg::fontdb::Stretch::Normal),
    ];

    let mut resolved = Vec::with_capacity(checks.len());
    for (family, weight, stretch) in checks {
        let query = usvg::fontdb::Query {
            families: &[usvg::fontdb::Family::Name(family)],
            weight: usvg::fontdb::Weight(weight),
            stretch,
            style: usvg::fontdb::Style::Normal,
        };
        let id = db
            .query(&query)
            .ok_or_else(|| format!("Font bundel tidak ditemukan: {} ({})", family, weight))?;
        let face = db
            .face(id)
            .ok_or_else(|| format!("Metadata font hilang: {} ({})", family, weight))?;
        let fam_name = face
            .families
            .first()
            .map(|(name, _)| name.clone())
            .unwrap_or_else(|| family.to_string());
        resolved.push(format!("{} (w={})", fam_name, face.weight.0));
    }
    Ok(resolved)
}

/// Formats any supported input date into English `"Month Day, Year"`.
pub fn format_english_date(raw: &str) -> Result<String, String> {
    let d = crate::excel_importer::parse_date_string(raw.trim()).ok_or_else(|| {
        format!(
            "Tanggal '{}' tidak valid (gunakan YYYY-MM-DD atau DD/MM/YY).",
            raw
        )
    })?;
    let month_idx = (d.month() as usize).saturating_sub(1);
    let month_name = ENGLISH_MONTHS.get(month_idx).copied().unwrap_or("January");
    Ok(format!("{} {}, {}", month_name, d.day(), d.year()))
}

/// Formats a start-to-end date span formally with year on both start and end dates.
pub fn format_english_date_range(start_raw: Option<&str>, end_raw: Option<&str>) -> String {
    let parse_opt = |opt: Option<&str>| {
        opt.map(str::trim)
            .filter(|s| !s.is_empty())
            .and_then(crate::excel_importer::parse_date_string)
    };
    let month_of = |d: chrono::NaiveDate| {
        ENGLISH_MONTHS
            .get((d.month() as usize).saturating_sub(1))
            .copied()
            .unwrap_or("January")
    };

    match (parse_opt(start_raw), parse_opt(end_raw)) {
        (Some(start), Some(end)) => format!(
            "{} {}, {} to {} {}, {}",
            month_of(start),
            start.day(),
            start.year(),
            month_of(end),
            end.day(),
            end.year()
        ),
        _ => {
            let start_str = start_raw
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(|s| format_english_date(s).unwrap_or_else(|_| s.to_string()))
                .unwrap_or_else(|| "[Start Date]".to_string());
            let end_str = end_raw
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(|s| format_english_date(s).unwrap_or_else(|_| s.to_string()))
                .unwrap_or_else(|| "[End Date]".to_string());
            format!("{} to {}", start_str, end_str)
        }
    }
}

fn format_single_department(segment: &str) -> String {
    let trimmed = segment.trim();
    let (raw_name, opt_dur) = if trimmed.ends_with(')') {
        if let Some(open_idx) = trimmed.rfind('(') {
            let name_part = trimmed[..open_idx].trim();
            let dur_part = trimmed[open_idx + 1..trimmed.len() - 1].trim();
            if !name_part.is_empty() && !dur_part.is_empty() {
                (name_part, Some(dur_part))
            } else {
                (trimmed, None)
            }
        } else {
            (trimmed, None)
        }
    } else {
        (trimmed, None)
    };

    let without_the = if raw_name.to_ascii_lowercase().starts_with("the ") {
        raw_name[4..].trim()
    } else {
        raw_name
    };
    let lower = without_the.to_ascii_lowercase();
    let base = if lower == "department"
        || lower.ends_with(" department")
        || lower.ends_with(" dept")
        || lower.ends_with(" dept.")
    {
        without_the.to_string()
    } else {
        format!("{} Department", without_the)
    };

    match opt_dur {
        Some(dur) => format!("the {} ({})", base, dur),
        None => format!("the {}", base),
    }
}

/// Formats `; `-separated training divisions into formal hotel department phrasing.
pub fn format_divisions_phrase(department_raw: Option<&str>) -> Option<String> {
    let raw = department_raw?.trim();
    if raw.is_empty() {
        return None;
    }
    let items: Vec<String> = raw
        .split(';')
        .map(str::trim)
        .filter(|s| !s.is_empty() && *s != "-")
        .map(format_single_department)
        .collect();
    match items.len() {
        0 => None,
        1 => Some(format!("in {}", items[0])),
        2 => Some(format!("in {} and {}", items[0], items[1])),
        _ => {
            let (last, head) = items.split_last()?;
            Some(format!("in {}, and {}", head.join(", "), last))
        }
    }
}

fn ensure_trailing_period(text: &str) -> String {
    let trimmed = text.trim();
    if trimmed.ends_with('.') || trimmed.ends_with('!') || trimmed.ends_with('?') {
        trimmed.to_string()
    } else {
        format!("{}.", trimmed)
    }
}

fn wrap_balanced_text(text: &str, max_chars: usize) -> Vec<String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }
    if trimmed.chars().count() <= max_chars || max_chars == 0 {
        return vec![trimmed.to_string()];
    }

    let words: Vec<&str> = trimmed.split_whitespace().collect();
    if words.len() <= 1 {
        return vec![trimmed.to_string()];
    }

    let total_chars: usize = words.iter().map(|w| w.chars().count()).sum::<usize>() + words.len() - 1;
    let n_lines = (total_chars + max_chars - 1) / max_chars;

    if n_lines == 2 {
        let mut best_split: Option<(usize, usize)> = None;
        for i in 1..words.len() {
            let left = words[..i].join(" ");
            let right = words[i..].join(" ");
            let lc = left.chars().count();
            let rc = right.chars().count();
            if lc <= max_chars && rc <= max_chars {
                let diff = lc.abs_diff(rc);
                if best_split.map_or(true, |(_, d)| diff < d) {
                    best_split = Some((i, diff));
                }
            }
        }
        if let Some((idx, _)) = best_split {
            return vec![words[..idx].join(" "), words[idx..].join(" ")];
        }
    }

    let target = (total_chars + n_lines - 1) / n_lines;
    let mut lines = Vec::with_capacity(n_lines);
    let mut current = String::new();
    for word in words {
        let word_len = word.chars().count();
        if current.is_empty() {
            current.push_str(word);
        } else {
            let cur_len = current.chars().count();
            let candidate_len = cur_len + 1 + word_len;
            if candidate_len <= max_chars
                && (candidate_len <= target || (candidate_len - target) < (target.saturating_sub(cur_len)))
            {
                current.push(' ');
                current.push_str(word);
            } else {
                lines.push(current);
                current = word.to_string();
            }
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

fn count_divisions(department_raw: Option<&str>) -> usize {
    let Some(raw) = department_raw.map(str::trim).filter(|s| !s.is_empty()) else {
        return 0;
    };
    raw.split(';')
        .map(str::trim)
        .filter(|s| !s.is_empty() && *s != "-")
        .count()
}

/// Builds balanced, non-redundant body lines for the certificate text block.
pub fn build_body_lines(data: &CertificateRenderData) -> Vec<String> {
    let is_a4 = data.paper_size.eq_ignore_ascii_case("A4");
    let max_chars = if is_a4 { 82 } else { 96 };
    let single_dept_max = if is_a4 { 108 } else { 120 };

    let mut lines = Vec::new();
    let prog_line = format!(
        "for successfully completing the {} {}",
        data.duration_text, data.program_name
    );
    let place_period = format!(
        "at Anda Bungalows & Restaurant, from {}.",
        data.date_range_display
    );

    if let Some(div_phrase) = format_divisions_phrase(data.department.as_deref()) {
        if count_divisions(data.department.as_deref()) == 1 {
            let merged = format!("{} {}", prog_line, div_phrase);
            if merged.chars().count() <= single_dept_max {
                lines.push(merged);
            } else {
                lines.extend(wrap_balanced_text(&merged, max_chars));
            }
            lines.extend(wrap_balanced_text(&place_period, max_chars));
        } else {
            lines.extend(wrap_balanced_text(&prog_line, max_chars));
            lines.extend(wrap_balanced_text(&div_phrase, max_chars));
            lines.extend(wrap_balanced_text(&place_period, max_chars));
        }
    } else {
        lines.extend(wrap_balanced_text(&prog_line, max_chars));
        lines.extend(wrap_balanced_text(&place_period, max_chars));
    }

    if let Some(ach) = data
        .achievement_text
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        let normalized = ensure_trailing_period(ach);
        lines.extend(wrap_balanced_text(&normalized, max_chars));
    }

    if data.show_awarded_line {
        lines.push(format!("Awarded on {}.", data.issued_date));
    }

    lines
}

fn escape_xml(input: &str) -> String {
    let mut out = String::with_capacity(input.len() + 8);
    for ch in input.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(ch),
        }
    }
    out
}

fn read_safe_asset_data_uri(
    cert_assets_dir: &Path,
    file_name: Option<&str>,
    allowed: &[&str],
) -> Option<String> {
    let clean = file_name?.trim();
    if !allowed.contains(&clean) {
        return None;
    }
    let path = cert_assets_dir.join(clean);
    let bytes = fs::read(path).ok()?;
    let mime = if clean.ends_with(".jpg") {
        "image/jpeg"
    } else {
        "image/png"
    };
    Some(format!("data:{};base64,{}", mime, BASE64.encode(&bytes)))
}

/// Returns a `data:` URI for either a custom uploaded logo or the bundled default logo PNG.
pub fn load_logo_data_uri(cert_assets_dir: &Path, custom_file: Option<&str>) -> String {
    read_safe_asset_data_uri(cert_assets_dir, custom_file, ALLOWED_LOGO_FILES)
        .unwrap_or_else(|| format!("data:image/png;base64,{}", BASE64.encode(DEFAULT_LOGO_PNG)))
}

/// Returns a `data:` URI for a custom uploaded signature/stamp if present and allowlisted.
pub fn load_signature_data_uri(
    cert_assets_dir: &Path,
    custom_file: Option<&str>,
) -> Option<String> {
    read_safe_asset_data_uri(cert_assets_dir, custom_file, ALLOWED_SIGNATURE_FILES)
}

/// Builds the full certificate settings state plus inline preview URIs.
pub fn build_certificate_settings_payload(
    cert_assets_dir: &Path,
    settings: &HashMap<String, String>,
) -> CertificateSettingsPayload {
    let logo_file = settings.get("certLogoFile").map(String::as_str);
    let sig_file = settings.get("certSignatureFile").map(String::as_str);
    let custom_logo = read_safe_asset_data_uri(cert_assets_dir, logo_file, ALLOWED_LOGO_FILES);
    let custom_sig =
        read_safe_asset_data_uri(cert_assets_dir, sig_file, ALLOWED_SIGNATURE_FILES);

    let paper = settings
        .get("certPaperSize")
        .map(String::as_str)
        .filter(|s| *s == "F4" || *s == "A4")
        .unwrap_or("F4")
        .to_string();

    CertificateSettingsPayload {
        signer_name: settings
            .get("certSignerName")
            .cloned()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| "Agus Sumardi Wijaya".to_string()),
        signer_title: settings
            .get("certSignerTitle")
            .cloned()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| "General Manager".to_string()),
        program_name: settings
            .get("certProgramName")
            .cloned()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| "hospitality training program".to_string()),
        paper_size: paper,
        show_signature: settings
            .get("certShowSignature")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false),
        has_custom_logo: custom_logo.is_some(),
        has_custom_signature: custom_sig.is_some(),
        logo_data_uri: custom_logo
            .unwrap_or_else(|| format!("data:image/png;base64,{}", BASE64.encode(DEFAULT_LOGO_PNG))),
        signature_data_uri: custom_sig,
    }
}

/// Validates PNG/JPEG bytes and size (<= 5 MB), then stores strictly inside `cert_assets_dir`.
pub fn validate_and_save_asset(
    cert_assets_dir: &Path,
    kind: &str,
    source_path: &Path,
) -> Result<String, String> {
    let prefix = match kind {
        "logo" => "custom_logo",
        "signature" => "custom_signature",
        _ => return Err("Jenis aset sertifikat tidak valid.".to_string()),
    };

    let meta = fs::metadata(source_path)
        .map_err(|e| format!("Gagal membaca metadata berkas gambar: {}", e))?;
    if !meta.is_file() {
        return Err("Jalur yang dipilih bukan berkas yang valid.".to_string());
    }
    if meta.len() == 0 || meta.len() > MAX_ASSET_BYTES {
        return Err("Ukuran gambar maksimal 5 MB dan tidak boleh kosong.".to_string());
    }

    let bytes =
        fs::read(source_path).map_err(|e| format!("Gagal membaca berkas gambar: {}", e))?;
    if bytes.is_empty() || (bytes.len() as u64) > MAX_ASSET_BYTES {
        return Err("Ukuran gambar maksimal 5 MB dan tidak boleh kosong.".to_string());
    }
    let ext = if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        "png"
    } else if bytes.len() >= 3 && bytes[0..3] == [0xFF, 0xD8, 0xFF] {
        "jpg"
    } else {
        return Err("Format gambar tidak didukung. Unggah berkas PNG atau JPEG.".to_string());
    };

    fs::create_dir_all(cert_assets_dir)
        .map_err(|e| format!("Gagal membuat direktori aset sertifikat: {}", e))?;

    let filename = format!("{}.{}", prefix, ext);
    let tmp_path = cert_assets_dir.join(format!("{}.tmp", filename));
    let dest = cert_assets_dir.join(&filename);
    fs::write(&tmp_path, &bytes)
        .map_err(|e| format!("Gagal menyimpan berkas aset sertifikat: {}", e))?;
    if let Err(e) = fs::rename(&tmp_path, &dest) {
        let _ = fs::remove_file(&tmp_path);
        return Err(format!("Gagal menyimpan berkas aset sertifikat: {}", e));
    }

    let other_ext = if ext == "png" { "jpg" } else { "png" };
    let stale_path = cert_assets_dir.join(format!("{}.{}", prefix, other_ext));
    if stale_path.exists() {
        let _ = fs::remove_file(stale_path);
    }

    Ok(filename)
}

/// Removes any stored custom image asset for `"logo"` or `"signature"`.
pub fn remove_custom_asset(cert_assets_dir: &Path, kind: &str) -> Result<(), String> {
    let prefix = match kind {
        "logo" => "custom_logo",
        "signature" => "custom_signature",
        _ => return Err("Jenis aset sertifikat tidak valid.".to_string()),
    };
    for ext in ["png", "jpg"] {
        let p = cert_assets_dir.join(format!("{}.{}", prefix, ext));
        if p.exists() {
            fs::remove_file(&p)
                .map_err(|e| format!("Gagal menghapus berkas aset lama: {}", e))?;
        }
    }
    Ok(())
}

/// Renders the canonical vector SVG document using exact mm viewBox units.
pub fn render_certificate_svg(data: &CertificateRenderData) -> String {
    let is_a4 = data.paper_size.eq_ignore_ascii_case("A4");
    let (page_w, page_h, y_off) = if is_a4 {
        (297.0_f64, 210.0_f64, 0.0_f64)
    } else {
        (330.0_f64, 215.0_f64, 2.5_f64)
    };
    let frame_w = page_w - 20.0;
    let frame_h = page_h - 20.0;
    let cx = page_w / 2.0;

    let pt_to_mm = |pt: f64| pt * MM_PER_PT;

    let cert_no_fs = pt_to_mm(10.0);
    let cert_no_ls = cert_no_fs * 0.045;
    let cert_no_x = page_w - 20.0;
    let cert_no_y = 21.25;

    let logo_w = 58.49;
    let logo_h = 30.95;
    let logo_x = cx - (logo_w / 2.0);
    let logo_y = 13.63 + y_off;

    let brand_fs = pt_to_mm(17.0);
    let brand_ls = brand_fs * 0.09;
    let brand_y = 49.14 + y_off + 5.55;

    let title_fs = pt_to_mm(44.2);
    let title_y = 69.17 + y_off + 14.30;

    let sub_fs = pt_to_mm(15.0);
    let sub_y = 98.00 + y_off + 4.85;

    let name_chars = data.recipient_name.chars().count();
    let recip_pt = if name_chars <= 26 {
        38.0
    } else {
        (38.0 * 26.0 / (name_chars as f64)).max(22.0)
    };
    let recip_fs = pt_to_mm(recip_pt);
    let recip_y = 116.34 + y_off + 8.0 + (recip_fs * 0.32);

    let body_lines = build_body_lines(data);
    let line_count = body_lines.len();
    let (body_pt, body_step, body_first_y) = match line_count {
        0..=2 => (11.0_f64, pt_to_mm(11.0) * 1.62, 148.20 + y_off + 3.55),
        3 => (11.0_f64, pt_to_mm(11.0) * 1.54, 146.37 + y_off + 3.55),
        4 => (10.5_f64, 5.35_f64, 143.50 + y_off + 3.40),
        5 => (10.0_f64, 4.85_f64, 141.60 + y_off + 3.20),
        _ => (9.2_f64, 4.05_f64, 138.80 + y_off + 2.95),
    };
    let body_fs = pt_to_mm(body_pt);
    let body_bottom_y = if line_count == 0 {
        body_first_y
    } else {
        body_first_y + ((line_count - 1) as f64) * body_step + (body_fs * 0.30)
    };
    let body_markup: String = body_lines
        .iter()
        .enumerate()
        .map(|(i, line)| {
            let y = body_first_y + (i as f64) * body_step;
            format!(
                "<text x=\"{cx:.2}\" y=\"{y:.2}\" text-anchor=\"middle\" font-family=\"'Libre Baskerville', serif\" font-weight=\"400\" font-size=\"{body_fs:.3}\" fill=\"#000000\">{text}</text>",
                cx = cx,
                y = y,
                body_fs = body_fs,
                text = escape_xml(line),
            )
        })
        .collect();

    let signer_fs = pt_to_mm(13.4);
    let signer_ls = signer_fs * 0.025;
    let signer_y = 176.00 + y_off + 4.40;

    let divider_y = 184.06 + y_off;
    let divider_half = 50.26 / 2.0;

    let role_fs = pt_to_mm(7.8);
    let role_ls = role_fs * 0.08;
    let role_y = 185.64 + y_off + 2.55;

    let mut sig_markup = String::new();
    if data.show_signature {
        if let Some(sig_uri) = &data.signature_data_uri {
            let sig_w = 44.0;
            let signer_top_y = signer_y - 4.0;
            let max_sig_top = body_bottom_y + 1.5;
            let sig_h = (signer_top_y - max_sig_top).clamp(7.0, 14.0);
            let sig_y = (signer_top_y - sig_h).max(max_sig_top);
            let sig_x = cx - (sig_w / 2.0);
            sig_markup = format!(
                "<image x=\"{:.2}\" y=\"{:.2}\" width=\"{:.2}\" height=\"{:.2}\" preserveAspectRatio=\"xMidYMid meet\" href=\"{}\"/>",
                sig_x,
                sig_y,
                sig_w,
                sig_h,
                escape_xml(sig_uri)
            );
        }
    }

    let revoked_markup = if data.is_revoked {
        format!(
            "<text x=\"{cx:.2}\" y=\"{cy:.2}\" text-anchor=\"middle\" font-family=\"'Source Serif Pro', serif\" font-weight=\"700\" font-size=\"26\" letter-spacing=\"4\" fill=\"#B91C1C\" fill-opacity=\"0.16\" transform=\"rotate(-22 {cx:.2} {cy:.2})\">REVOKED</text>",
            cx = cx,
            cy = page_h / 2.0
        )
    } else {
        String::new()
    };

    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{page_w:.0}mm\" height=\"{page_h:.0}mm\" viewBox=\"0 0 {page_w:.2} {page_h:.2}\">\
<rect x=\"0\" y=\"0\" width=\"{page_w:.2}\" height=\"{page_h:.2}\" fill=\"#FFFFFF\"/>\
<rect x=\"10\" y=\"10\" width=\"{frame_w:.2}\" height=\"{frame_h:.2}\" fill=\"none\" stroke=\"#C67D24\" stroke-width=\"0.53\"/>\
<text x=\"{cert_no_x:.2}\" y=\"{cert_no_y:.2}\" text-anchor=\"end\" font-family=\"'Nunito Sans Condensed', sans-serif\" font-weight=\"400\" font-size=\"{cert_no_fs:.3}\" letter-spacing=\"{cert_no_ls:.3}\" fill=\"#545454\">Certificate No. {cert_no}</text>\
<image x=\"{logo_x:.2}\" y=\"{logo_y:.2}\" width=\"{logo_w:.2}\" height=\"{logo_h:.2}\" preserveAspectRatio=\"xMidYMid meet\" href=\"{logo_uri}\"/>\
<text x=\"{cx:.2}\" y=\"{brand_y:.2}\" text-anchor=\"middle\" font-family=\"'Nunito Sans Condensed', sans-serif\" font-weight=\"400\" font-size=\"{brand_fs:.3}\" letter-spacing=\"{brand_ls:.3}\" fill=\"#000000\">ANDA BUNGALOWS &amp; RESTAURANT</text>\
<text x=\"{cx:.2}\" y=\"{title_y:.2}\" text-anchor=\"middle\" font-family=\"'Source Serif Pro', serif\" font-weight=\"400\" font-size=\"{title_fs:.3}\" fill=\"#1F1F1F\">Certificate of Completion</text>\
<text x=\"{cx:.2}\" y=\"{sub_y:.2}\" text-anchor=\"middle\" font-family=\"'Libre Baskerville', serif\" font-weight=\"400\" font-size=\"{sub_fs:.3}\" fill=\"#000000\">This certificate is proudly presented to</text>\
<text x=\"{cx:.2}\" y=\"{recip_y:.2}\" text-anchor=\"middle\" font-family=\"'Source Serif Pro', serif\" font-weight=\"700\" font-size=\"{recip_fs:.3}\" fill=\"#000000\">{recipient}</text>\
{body_markup}\
{sig_markup}\
<text x=\"{cx:.2}\" y=\"{signer_y:.2}\" text-anchor=\"middle\" font-family=\"'Source Serif Pro', serif\" font-weight=\"700\" font-size=\"{signer_fs:.3}\" letter-spacing=\"{signer_ls:.3}\" fill=\"#1F1F1F\">{signer_name}</text>\
<line x1=\"{div_x1:.2}\" y1=\"{divider_y:.2}\" x2=\"{div_x2:.2}\" y2=\"{divider_y:.2}\" stroke=\"#C67D24\" stroke-width=\"0.35\"/>\
<circle cx=\"{div_x1:.2}\" cy=\"{divider_y:.2}\" r=\"0.55\" fill=\"#C67D24\"/>\
<circle cx=\"{div_x2:.2}\" cy=\"{divider_y:.2}\" r=\"0.55\" fill=\"#C67D24\"/>\
<text x=\"{cx:.2}\" y=\"{role_y:.2}\" text-anchor=\"middle\" font-family=\"'Roboto', sans-serif\" font-weight=\"400\" font-size=\"{role_fs:.3}\" letter-spacing=\"{role_ls:.3}\" fill=\"#545454\">{signer_title}</text>\
{revoked_markup}\
</svg>",
        page_w = page_w,
        page_h = page_h,
        frame_w = frame_w,
        frame_h = frame_h,
        cert_no_x = cert_no_x,
        cert_no_y = cert_no_y,
        cert_no_fs = cert_no_fs,
        cert_no_ls = cert_no_ls,
        cert_no = escape_xml(&data.certificate_no),
        logo_x = logo_x,
        logo_y = logo_y,
        logo_w = logo_w,
        logo_h = logo_h,
        logo_uri = escape_xml(&data.logo_data_uri),
        cx = cx,
        brand_y = brand_y,
        brand_fs = brand_fs,
        brand_ls = brand_ls,
        title_y = title_y,
        title_fs = title_fs,
        sub_y = sub_y,
        sub_fs = sub_fs,
        recip_y = recip_y,
        recip_fs = recip_fs,
        recipient = escape_xml(&data.recipient_name),
        body_markup = body_markup,
        sig_markup = sig_markup,
        signer_y = signer_y,
        signer_fs = signer_fs,
        signer_ls = signer_ls,
        signer_name = escape_xml(&data.signer_name),
        div_x1 = cx - divider_half,
        div_x2 = cx + divider_half,
        divider_y = divider_y,
        role_y = role_y,
        role_fs = role_fs,
        role_ls = role_ls,
        signer_title = escape_xml(&data.signer_title),
        revoked_markup = revoked_markup,
    )
}

fn parse_certificate_tree(svg: &str) -> Result<usvg::Tree, String> {
    let mut options = usvg::Options::default();
    options.dpi = 96.0;
    *options.fontdb_mut() = load_certificate_fontdb();
    usvg::Tree::from_str(svg, &options).map_err(|e| format!("Gagal memproses tata letak SVG: {}", e))
}

/// Converts rendered certificate SVG into a vector PDF buffer with exact mm page dimensions.
pub fn render_certificate_pdf_bytes(data: &CertificateRenderData) -> Result<Vec<u8>, String> {
    let svg = render_certificate_svg(data);
    let tree = parse_certificate_tree(&svg)?;
    svg2pdf::to_pdf(
        &tree,
        svg2pdf::ConversionOptions::default(),
        svg2pdf::PageOptions { dpi: 96.0 },
    )
    .map_err(|e| format!("Gagal menghasilkan berkas PDF sertifikat: {}", e))
}

/// Converts rendered certificate SVG into a 300 DPI PNG image buffer.
pub fn render_certificate_png_bytes(data: &CertificateRenderData) -> Result<Vec<u8>, String> {
    let svg = render_certificate_svg(data);
    let tree = parse_certificate_tree(&svg)?;
    let scale = 300.0_f32 / 96.0_f32;
    let width = (tree.size().width() * scale).round().max(1.0) as u32;
    let height = (tree.size().height() * scale).round().max(1.0) as u32;

    let mut pixmap = tiny_skia::Pixmap::new(width, height)
        .ok_or_else(|| "Gagal mengalokasikan kanvas gambar sertifikat.".to_string())?;
    pixmap.fill(tiny_skia::Color::WHITE);

    resvg::render(
        &tree,
        tiny_skia::Transform::from_scale(scale, scale),
        &mut pixmap.as_mut(),
    );

    pixmap
        .encode_png()
        .map_err(|e| format!("Gagal menyandikan berkas PNG sertifikat: {}", e))
}

/// Writes a vector PDF certificate file to `dest_path`.
pub fn export_certificate_pdf(data: &CertificateRenderData, dest_path: &Path) -> Result<(), String> {
    let bytes = render_certificate_pdf_bytes(data)?;
    fs::write(dest_path, bytes)
        .map_err(|e| format!("Gagal menyimpan berkas PDF sertifikat: {}", e))
}

/// Writes a 300 DPI PNG certificate file to `dest_path`.
pub fn export_certificate_png(data: &CertificateRenderData, dest_path: &Path) -> Result<(), String> {
    let bytes = render_certificate_png_bytes(data)?;
    fs::write(dest_path, bytes)
        .map_err(|e| format!("Gagal menyimpan berkas PNG sertifikat: {}", e))
}
