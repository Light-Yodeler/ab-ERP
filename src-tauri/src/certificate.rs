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

pub const DEFAULT_PROPERTY_NAME: &str = "Anda Bungalows & Restaurant";
pub const DEFAULT_ISSUED_CITY: &str = "Kuta, Lombok";
pub const DEFAULT_BODY_TEMPLATE: &str = "for successfully completing the {durasi} {jenis_program} in the {departemen} at {properti}, from {mulai} to {selesai}.";

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
    pub issued_city: Option<String>,
    #[serde(default)]
    pub body_template: Option<String>,
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
    pub issued_city: String,
    pub body_template: String,
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
    pub start_date_display: String,
    pub end_date_display: String,
    pub is_same_day_span: bool,
    pub issued_date: String,
    pub issued_city: Option<String>,
    pub body_template: String,
    pub property_heading: String,
    pub property_name: String,
    pub date_range_display: String,
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
        let default_city = settings
            .get("certIssuedCity")
            .map(String::as_str)
            .filter(|s| !s.trim().is_empty())
            .unwrap_or(DEFAULT_ISSUED_CITY);
        let default_template = settings
            .get("certBodyTemplate")
            .map(String::as_str)
            .filter(|s| !s.trim().is_empty())
            .unwrap_or(DEFAULT_BODY_TEMPLATE);

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

        let fmt_date_or = |raw_opt: &Option<String>, placeholder: &str| -> String {
            match raw_opt.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
                Some(raw) => format_english_date(raw).unwrap_or_else(|_| raw.to_string()),
                None => placeholder.to_string(),
            }
        };

        let start_date_display = fmt_date_or(&input.start_date, "[Start Date]");
        let end_date_display = fmt_date_or(&input.end_date, "[End Date]");
        let issued_trimmed = input
            .issued_date
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty());
        let issued_date = match issued_trimmed {
            Some(raw) => format_english_date(raw).unwrap_or_else(|_| raw.to_string()),
            None => "[Date]".to_string(),
        };

        let parsed_start = input
            .start_date
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .and_then(crate::excel_importer::parse_date_string);
        let parsed_end = input
            .end_date
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .and_then(crate::excel_importer::parse_date_string);
        let is_same_day_span = matches!((parsed_start, parsed_end), (Some(s), Some(e)) if s == e);

        let recipient_name = match input
            .recipient_name
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            Some(raw) => normalize_recipient_name(raw),
            None => "[Recipient Name]".to_string(),
        };

        let issued_city = match input
            .issued_city
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty() && *s != "-")
        {
            Some(c) => Some(c.to_string()),
            None => {
                let fallback = default_city.trim();
                if fallback.is_empty() || fallback == "-" {
                    None
                } else {
                    Some(fallback.to_string())
                }
            }
        };

        let raw_business_name = settings
            .get("businessName")
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .unwrap_or("ANDA BUNGALOWS & RESTAURANT");

        Self {
            certificate_no: non_empty(&input.certificate_no, "[000/ABR/I/YYYY]"),
            recipient_name,
            duration_text: non_empty(&input.duration_text, "[Duration]"),
            program_name: non_empty(&input.program_name, default_program),
            start_date_display,
            end_date_display,
            is_same_day_span,
            issued_date,
            issued_city,
            body_template: non_empty(&input.body_template, default_template),
            property_heading: raw_business_name.to_uppercase(),
            property_name: format_property_name_for_body(raw_business_name),
            date_range_display: format_english_date_range(
                input.start_date.as_deref(),
                input.end_date.as_deref(),
            ),
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

fn format_property_name_for_body(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return DEFAULT_PROPERTY_NAME.to_string();
    }
    let has_upper = trimmed.chars().any(|c| c.is_uppercase());
    let has_lower = trimmed.chars().any(|c| c.is_lowercase());
    if has_upper && !has_lower {
        return trimmed
            .split_whitespace()
            .map(|w| {
                let upper = w.to_ascii_uppercase();
                if matches!(
                    upper.as_str(),
                    "PT" | "PT." | "CV" | "CV." | "UD" | "UD." | "TBK" | "TBK." | "&"
                ) {
                    upper
                } else {
                    capitalize_word(w)
                }
            })
            .collect::<Vec<_>>()
            .join(" ");
    }
    trimmed.to_string()
}

fn capitalize_word(word: &str) -> String {
    let mut out = String::with_capacity(word.len());
    let mut capitalized = false;
    for ch in word.chars() {
        if !capitalized && ch.is_alphabetic() {
            for up in ch.to_uppercase() {
                out.push(up);
            }
            capitalized = true;
        } else {
            for lo in ch.to_lowercase() {
                out.push(lo);
            }
        }
    }
    out
}

fn normalize_apostrophe_segment(seg: &str) -> String {
    if let Some((head, tail)) = seg.split_once('\'') {
        if head.chars().count() == 1
            && tail.chars().count() >= 2
            && matches!(head.to_ascii_lowercase().as_str(), "d" | "o" | "l")
        {
            return format!("{}'{}", capitalize_word(head), capitalize_word(tail));
        }
        let lower_tail: String = tail.chars().flat_map(char::to_lowercase).collect();
        return format!("{}'{}", capitalize_word(head), lower_tail);
    }
    capitalize_word(seg)
}

fn is_prefix_title_token(token: &str) -> bool {
    let cleaned = token.trim().trim_end_matches('.').to_ascii_lowercase();
    matches!(
        cleaned.as_str(),
        "prof"
            | "dr"
            | "drg"
            | "ir"
            | "drs"
            | "dra"
            | "h"
            | "hj"
            | "k.h"
            | "kh"
            | "tgh"
            | "r"
            | "r.a"
            | "ra"
            | "mr"
            | "mrs"
            | "ms"
    )
}

fn normalize_name_token(token: &str, is_first_word: bool, after_prof: bool) -> String {
    if token.is_empty() {
        return String::new();
    }
    if token == "dr." && !after_prof {
        return "dr.".to_string();
    }
    if token == "drg." {
        return "drg.".to_string();
    }

    let lower = token.to_ascii_lowercase();
    match lower.as_str() {
        "prof" | "prof." => return "Prof.".to_string(),
        "dr" | "dr." => return "Dr.".to_string(),
        "drg" | "drg." => return "drg.".to_string(),
        "ir" | "ir." => return "Ir.".to_string(),
        "drs" | "drs." => return "Drs.".to_string(),
        "dra" | "dra." => return "Dra.".to_string(),
        "h" | "h." => return "H.".to_string(),
        "hj" | "hj." => return "Hj.".to_string(),
        "k.h." | "kh." | "kh" => return "K.H.".to_string(),
        "tgh" | "tgh." => return "TGH.".to_string(),
        "r." => return "R.".to_string(),
        "r.a." | "ra." => return "R.A.".to_string(),
        "mr" | "mr." => return "Mr.".to_string(),
        "mrs" | "mrs." => return "Mrs.".to_string(),
        "ms" | "ms." => return "Ms.".to_string(),
        _ => {}
    }

    if token.contains('.') {
        let parts: Vec<&str> = token.split('.').filter(|s| !s.is_empty()).collect();
        if !parts.is_empty() && parts.iter().all(|p| p.chars().count() <= 2) {
            let formatted: Vec<String> = parts
                .iter()
                .map(|p| {
                    if p.chars().count() == 1 {
                        p.to_uppercase()
                    } else {
                        capitalize_word(p)
                    }
                })
                .collect();
            return format!("{}.", formatted.join("."));
        }
    }

    if !is_first_word
        && matches!(
            lower.as_str(),
            "bin" | "binti" | "ibnu" | "van" | "von" | "de" | "del" | "der" | "da" | "di" | "la" | "le"
        )
    {
        return lower;
    }

    if token.contains('-') {
        return token
            .split('-')
            .map(normalize_apostrophe_segment)
            .collect::<Vec<_>>()
            .join("-");
    }

    normalize_apostrophe_segment(token)
}

fn normalize_degree_token(token: &str) -> String {
    let trimmed = token.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let compact: String = trimmed
        .chars()
        .filter(|c| *c != '.')
        .flat_map(char::to_lowercase)
        .collect();

    let canonical = match compact.as_str() {
        "amd" => Some("A.Md."),
        "amdpar" => Some("A.Md.Par."),
        "amdak" => Some("A.Md.Ak."),
        "amdkom" => Some("A.Md.Kom."),
        "ama" => Some("A.Ma."),
        "strpar" => Some("S.Tr.Par."),
        "strkom" => Some("S.Tr.Kom."),
        "strak" => Some("S.Tr.Ak."),
        "strsos" => Some("S.Tr.Sos."),
        "spar" => Some("S.Par."),
        "mpar" => Some("M.Par."),
        "se" => Some("S.E."),
        "sh" => Some("S.H."),
        "st" => Some("S.T."),
        "skom" => Some("S.Kom."),
        "spd" => Some("S.Pd."),
        "ssos" => Some("S.Sos."),
        "ssi" => Some("S.Si."),
        "sip" => Some("S.IP."),
        "spsi" => Some("S.Psi."),
        "sag" => Some("S.Ag."),
        "sak" => Some("S.Ak."),
        "smn" => Some("S.Mn."),
        "sab" => Some("S.AB."),
        "sikom" => Some("S.I.Kom."),
        "shum" => Some("S.Hum."),
        "sked" => Some("S.Ked."),
        "skep" => Some("S.Kep."),
        "sfarm" => Some("S.Farm."),
        "sgz" => Some("S.Gz."),
        "sp" => Some("S.P."),
        "spt" => Some("S.Pt."),
        "spi" => Some("S.Pi."),
        "ssn" => Some("S.Sn."),
        "sds" => Some("S.Ds."),
        "mm" => Some("M.M."),
        "msi" => Some("M.Si."),
        "mkom" => Some("M.Kom."),
        "mpd" => Some("M.Pd."),
        "mt" => Some("M.T."),
        "mh" => Some("M.H."),
        "msc" => Some("M.Sc."),
        "ma" => Some("M.A."),
        "mba" => Some("MBA"),
        "mak" => Some("M.Ak."),
        "mhum" => Some("M.Hum."),
        "mpsi" => Some("M.Psi."),
        "phd" => Some("Ph.D."),
        "bsc" => Some("B.Sc."),
        "ba" => Some("B.A."),
        "che" => Some("CHE"),
        "cha" => Some("CHA"),
        "diii" => Some("D.III"),
        "div" => Some("D.IV"),
        _ => None,
    };
    if let Some(hit) = canonical {
        return hit.to_string();
    }

    if trimmed.contains('.') {
        let ends_with_dot = trimmed.ends_with('.');
        let parts: Vec<&str> = trimmed.split('.').filter(|s| !s.is_empty()).collect();
        if !parts.is_empty() {
            let formatted: Vec<String> = parts
                .iter()
                .map(|p| {
                    if p.chars().count() == 1 {
                        p.to_uppercase()
                    } else if (2..=3).contains(&p.chars().count())
                        && p.chars().all(|c| c.is_ascii_uppercase())
                    {
                        (*p).to_string()
                    } else {
                        capitalize_word(p)
                    }
                })
                .collect();
            return if ends_with_dot {
                format!("{}.", formatted.join("."))
            } else {
                formatted.join(".")
            };
        }
    }

    if trimmed.chars().count() <= 4 && trimmed.chars().all(|c| c.is_ascii_alphabetic()) {
        return trimmed.to_ascii_uppercase();
    }
    capitalize_word(trimmed)
}

/// Normalizes recipient names, Indonesian/international prefix titles, and comma-separated academic degrees.
pub fn normalize_recipient_name(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let mut segments = trimmed.split(',');
    let name_part = segments.next().unwrap_or("");
    let words: Vec<&str> = name_part.split_whitespace().collect();
    let mut seen_name_word = false;
    let normalized_name: String = words
        .iter()
        .enumerate()
        .map(|(idx, w)| {
            let after_prof = idx > 0
                && words[idx - 1]
                    .to_ascii_lowercase()
                    .trim_end_matches('.')
                    == "prof";
            let is_first_name_word = !seen_name_word;
            if !is_prefix_title_token(w) {
                seen_name_word = true;
            }
            normalize_name_token(w, is_first_name_word, after_prof)
        })
        .collect::<Vec<_>>()
        .join(" ");

    let normalized_suffixes: Vec<String> = segments
        .map(|seg| {
            seg.split_whitespace()
                .map(normalize_degree_token)
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
                .join(" ")
        })
        .filter(|s| !s.is_empty())
        .collect();

    if normalized_suffixes.is_empty() {
        normalized_name
    } else if normalized_name.is_empty() {
        normalized_suffixes.join(", ")
    } else {
        format!("{}, {}", normalized_name, normalized_suffixes.join(", "))
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

fn format_department_item(segment: &str, include_the: bool) -> String {
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

    let prefix = if include_the { "the " } else { "" };
    match opt_dur {
        Some(dur) => format!("{}{} ({})", prefix, base, dur),
        None => format!("{}{}", prefix, base),
    }
}

fn format_departments_list(department_raw: Option<&str>, leading_the: bool) -> Option<String> {
    let raw = department_raw?.trim();
    if raw.is_empty() {
        return None;
    }
    let items: Vec<String> = raw
        .split(';')
        .map(str::trim)
        .filter(|s| !s.is_empty() && *s != "-")
        .enumerate()
        .map(|(idx, seg)| format_department_item(seg, leading_the || idx > 0))
        .collect();
    match items.len() {
        0 => None,
        1 => Some(items[0].clone()),
        2 => Some(format!("{} and {}", items[0], items[1])),
        _ => {
            let (last, head) = items.split_last()?;
            Some(format!("{}, and {}", head.join(", "), last))
        }
    }
}

/// Formats `; `-separated training divisions into formal hotel department phrasing.
pub fn format_divisions_phrase(department_raw: Option<&str>) -> Option<String> {
    format_departments_list(department_raw, true).map(|joined| format!("in {}", joined))
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

fn template_precedes_the(template: &str, placeholder: &str) -> bool {
    if let Some(pos) = template.find(placeholder) {
        let before = template[..pos].trim_end();
        return before
            .to_ascii_lowercase()
            .ends_with("the")
            && before
                .chars()
                .rev()
                .nth(3)
                .map_or(true, |c| !c.is_alphanumeric());
    }
    false
}

/// Renders the configurable certificate body sentence by substituting template variables.
pub fn render_body_template(data: &CertificateRenderData) -> String {
    let mut out = if data.body_template.trim().is_empty() {
        DEFAULT_BODY_TEMPLATE.to_string()
    } else {
        data.body_template.trim().to_string()
    };

    if data.is_same_day_span {
        for pat in [
            "from {mulai} to {selesai}",
            "from {tanggal_mulai} to {tanggal_selesai}",
            "from {start_date} to {end_date}",
        ] {
            if out.contains(pat) {
                out = out.replace(pat, "on {mulai}");
            }
        }
    }

    let leading_the = !template_precedes_the(&out, "{departemen}")
        && !template_precedes_the(&out, "{department}");

    if let Some(dept_text) = format_departments_list(data.department.as_deref(), leading_the) {
        out = out
            .replace("{departemen}", &dept_text)
            .replace("{department}", &dept_text);
    } else {
        for phrase in [
            "in the {departemen}",
            "in the {department}",
            "at the {departemen}",
            "at the {department}",
            "in {departemen}",
            "in {department}",
            "{departemen}",
            "{department}",
        ] {
            out = out.replace(phrase, "");
        }
    }

    let replacements = [
        ("{durasi}", data.duration_text.as_str()),
        ("{duration}", data.duration_text.as_str()),
        ("{jenis_program}", data.program_name.as_str()),
        ("{program}", data.program_name.as_str()),
        ("{properti}", data.property_name.as_str()),
        ("{property}", data.property_name.as_str()),
        ("{mulai}", data.start_date_display.as_str()),
        ("{tanggal_mulai}", data.start_date_display.as_str()),
        ("{start_date}", data.start_date_display.as_str()),
        ("{selesai}", data.end_date_display.as_str()),
        ("{tanggal_selesai}", data.end_date_display.as_str()),
        ("{end_date}", data.end_date_display.as_str()),
        ("{penerima}", data.recipient_name.as_str()),
        ("{recipient}", data.recipient_name.as_str()),
        ("{nomor}", data.certificate_no.as_str()),
        ("{certificate_no}", data.certificate_no.as_str()),
    ];
    for (token, val) in replacements {
        if out.contains(token) {
            out = out.replace(token, val);
        }
    }

    let normalized: String = out.split_whitespace().collect::<Vec<_>>().join(" ");
    let cleaned = normalized
        .replace(" ,", ",")
        .replace(" .", ".")
        .trim()
        .to_string();
    ensure_trailing_period(&cleaned)
}

/// Formats the `"Issued in [city], on [date]"` line rendered below the certificate body.
pub fn format_issued_line(data: &CertificateRenderData) -> Option<String> {
    let date_part = data.issued_date.trim();
    if date_part.is_empty() {
        return None;
    }
    match data
        .issued_city
        .as_deref()
        .map(str::trim)
        .filter(|c| !c.is_empty())
    {
        Some(city) => Some(format!("Issued in {}, on {}", city, date_part)),
        None => Some(format!("Issued on {}", date_part)),
    }
}

/// Builds balanced, non-redundant body lines for the certificate text block.
pub fn build_body_lines(data: &CertificateRenderData) -> Vec<String> {
    let is_a4 = data.paper_size.eq_ignore_ascii_case("A4");
    let max_chars = if is_a4 { 82 } else { 96 };
    let single_dept_max = if is_a4 { 108 } else { 120 };

    let mut lines = Vec::new();
    let uses_default_template = data.body_template.trim().is_empty()
        || data.body_template.trim() == DEFAULT_BODY_TEMPLATE;

    if uses_default_template {
        let prog_line = format!(
            "for successfully completing the {} {}",
            data.duration_text, data.program_name
        );
        let date_clause = if data.is_same_day_span {
            format!("on {}", data.start_date_display)
        } else {
            format!("from {}", data.date_range_display)
        };
        let place_period = format!("at {}, {}.", data.property_name, date_clause);

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
    } else {
        let rendered = render_body_template(data);
        lines.extend(wrap_balanced_text(&rendered, max_chars));
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
        issued_city: settings
            .get("certIssuedCity")
            .cloned()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_ISSUED_CITY.to_string()),
        body_template: settings
            .get("certBodyTemplate")
            .cloned()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_BODY_TEMPLATE.to_string()),
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

    let top_off = y_off * 0.6;

    let logo_w = 58.49;
    let logo_h = 30.95;
    let logo_x = cx - (logo_w / 2.0);
    let logo_y = 12.60 + top_off;

    let brand_fs = pt_to_mm(17.0);
    let brand_ls = brand_fs * 0.09;
    let brand_y = 47.20 + top_off + 5.55;

    let title_fs = pt_to_mm(44.2);
    let title_y = 64.50 + top_off + 14.30;

    let sub_fs = pt_to_mm(15.0);
    let sub_y = 92.20 + top_off + 4.85;

    let name_chars = data.recipient_name.chars().count();
    let recip_pt = if name_chars <= 26 {
        38.0
    } else {
        (38.0 * 26.0 / (name_chars as f64)).max(22.0)
    };
    let recip_fs = pt_to_mm(recip_pt);
    let recip_y = 109.20 + top_off + 8.0 + (recip_fs * 0.32);

    let body_lines = build_body_lines(data);
    let issued_line = format_issued_line(data);
    let line_count = body_lines.len();
    let visual_lines = line_count + usize::from(issued_line.is_some());
    let (body_pt, body_step, body_first_y) = match visual_lines {
        0..=2 => (11.0_f64, pt_to_mm(11.0) * 1.55, 139.50 + top_off + 3.55),
        3 => (11.0_f64, 5.60_f64, 136.50 + top_off + 3.55),
        4 => (10.5_f64, 5.00_f64, 134.20 + top_off + 3.40),
        5 => (10.0_f64, 4.50_f64, 132.40 + top_off + 3.20),
        6 => (9.2_f64, 4.00_f64, 130.60 + top_off + 2.95),
        _ => (8.6_f64, 3.55_f64, 129.20 + top_off + 2.75),
    };
    let body_fs = pt_to_mm(body_pt);
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

    let (issued_markup, body_bottom_y) = if let Some(issued_text) = &issued_line {
        let issued_fs = pt_to_mm((body_pt - 0.8).max(8.4));
        let issued_gap = if line_count == 0 {
            0.0
        } else {
            body_step + 1.0
        };
        let issued_y = if line_count == 0 {
            body_first_y
        } else {
            body_first_y + ((line_count - 1) as f64) * body_step + issued_gap
        };
        let markup = format!(
            "<text x=\"{cx:.2}\" y=\"{issued_y:.2}\" text-anchor=\"middle\" font-family=\"'Libre Baskerville', serif\" font-weight=\"400\" font-size=\"{issued_fs:.3}\" fill=\"#3F3F46\">{text}</text>",
            cx = cx,
            issued_y = issued_y,
            issued_fs = issued_fs,
            text = escape_xml(issued_text),
        );
        (markup, issued_y + (issued_fs * 0.35))
    } else if line_count == 0 {
        (String::new(), body_first_y)
    } else {
        (
            String::new(),
            body_first_y + ((line_count - 1) as f64) * body_step + (body_fs * 0.30),
        )
    };

    let signer_fs = pt_to_mm(13.4);
    let signer_ls = signer_fs * 0.025;
    let signer_y = 181.50 + y_off + 4.40;

    let divider_y = 189.56 + y_off;
    let divider_half = 50.26 / 2.0;

    let role_fs = pt_to_mm(7.8);
    let role_ls = role_fs * 0.08;
    let role_y = 191.14 + y_off + 2.55;

    let mut sig_markup = String::new();
    if data.show_signature {
        if let Some(sig_uri) = &data.signature_data_uri {
            let sig_w = 50.0;
            let signer_top_y = signer_y - 4.0;
            let max_sig_top = body_bottom_y + 1.5;
            let available = (signer_top_y - max_sig_top).max(0.0);
            if available >= 4.0 {
                let sig_h = available.min(20.0);
                let sig_y = signer_top_y - sig_h;
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
<text x=\"{cx:.2}\" y=\"{brand_y:.2}\" text-anchor=\"middle\" font-family=\"'Nunito Sans Condensed', sans-serif\" font-weight=\"400\" font-size=\"{brand_fs:.3}\" letter-spacing=\"{brand_ls:.3}\" fill=\"#000000\">{property_heading}</text>\
<text x=\"{cx:.2}\" y=\"{title_y:.2}\" text-anchor=\"middle\" font-family=\"'Source Serif Pro', serif\" font-weight=\"400\" font-size=\"{title_fs:.3}\" fill=\"#1F1F1F\">Certificate of Completion</text>\
<text x=\"{cx:.2}\" y=\"{sub_y:.2}\" text-anchor=\"middle\" font-family=\"'Libre Baskerville', serif\" font-weight=\"400\" font-size=\"{sub_fs:.3}\" fill=\"#000000\">This certificate is proudly presented to</text>\
<text x=\"{cx:.2}\" y=\"{recip_y:.2}\" text-anchor=\"middle\" font-family=\"'Source Serif Pro', serif\" font-weight=\"700\" font-size=\"{recip_fs:.3}\" fill=\"#000000\">{recipient}</text>\
{body_markup}\
{issued_markup}\
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
        property_heading = escape_xml(&data.property_heading),
        title_y = title_y,
        title_fs = title_fs,
        sub_y = sub_y,
        sub_fs = sub_fs,
        recip_y = recip_y,
        recip_fs = recip_fs,
        recipient = escape_xml(&data.recipient_name),
        body_markup = body_markup,
        issued_markup = issued_markup,
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
