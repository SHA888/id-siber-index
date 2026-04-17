//! Incident normalization pipeline
//!
//! This module provides the logic to normalize `IncidentDraft` into `Incident` records.
//! It handles:
//! - Organization name canonicalization (e.g., "PT Bank X Tbk" → "Bank X")
//! - Date parsing with Indonesian format support
//! - Attack type classification from keywords
//! - Sector classification from org names and keywords
//! - Per-field confidence scoring

use crate::incident_draft::IncidentDraft;
use chrono::NaiveDate;
use regex::Regex;
use std::collections::HashMap;
use std::sync::LazyLock;
use tracing::{debug, warn};

/// Normalized confidence scores for each field
#[derive(Debug, Clone, Default)]
pub struct FieldConfidence {
    pub org_name: f32,
    pub org_sector: f32,
    pub incident_date: f32,
    pub attack_type: f32,
    pub data_categories: f32,
}

impl FieldConfidence {
    /// Calculate overall confidence as weighted average
    pub fn overall(&self) -> f32 {
        let weights = [0.25, 0.20, 0.20, 0.20, 0.15]; // org_name most important
        let scores = [
            self.org_name,
            self.org_sector,
            self.incident_date,
            self.attack_type,
            self.data_categories,
        ];
        let weighted_sum: f32 = scores.iter().zip(weights.iter()).map(|(s, w)| s * w).sum();
        let weight_sum: f32 = weights.iter().sum();
        (weighted_sum / weight_sum).clamp(0.0, 1.0)
    }
}

/// Organization name canonicalization rules
static ORG_CANONICAL_NAMES: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    HashMap::from([
        // Major Indonesian Banks
        ("bca", "Bank Central Asia"),
        ("bank bca", "Bank Central Asia"),
        ("bank central asia", "Bank Central Asia"),
        ("pt bank central asia", "Bank Central Asia"),
        ("pt bank central asia tbk", "Bank Central Asia"),
        ("mandiri", "Bank Mandiri"),
        ("bank mandiri", "Bank Mandiri"),
        ("pt bank mandiri", "Bank Mandiri"),
        ("pt bank mandiri tbk", "Bank Mandiri"),
        ("bni", "Bank Negara Indonesia"),
        ("bank bni", "Bank Negara Indonesia"),
        ("bank negara indonesia", "Bank Negara Indonesia"),
        ("pt bank negara indonesia", "Bank Negara Indonesia"),
        ("bri", "Bank Rakyat Indonesia"),
        ("bank bri", "Bank Rakyat Indonesia"),
        ("bank rakyat indonesia", "Bank Rakyat Indonesia"),
        ("pt bank rakyat indonesia", "Bank Rakyat Indonesia"),
        ("btpn", "Bank BTPN"),
        ("bank btpn", "Bank BTPN"),
        ("cimb", "CIMB Niaga"),
        ("cimb niaga", "CIMB Niaga"),
        ("bank cimb niaga", "CIMB Niaga"),
        ("danamon", "Bank Danamon"),
        ("bank danamon", "Bank Danamon"),
        ("permata", "Bank Permata"),
        ("bank permata", "Bank Permata"),
        ("mega", "Bank Mega"),
        ("bank mega", "Bank Mega"),
        ("panin", "Panin Bank"),
        ("bank panin", "Panin Bank"),
        ("syariah mandiri", "Bank Syariah Indonesia"),
        ("bsi", "Bank Syariah Indonesia"),
        ("bank syariah indonesia", "Bank Syariah Indonesia"),
        // Telcos
        ("telkom", "Telkom Indonesia"),
        ("pt telkom", "Telkom Indonesia"),
        ("pt telkom indonesia", "Telkom Indonesia"),
        ("pt telekomunikasi indonesia", "Telkom Indonesia"),
        ("telkomsel", "Telkomsel"),
        ("xl", "XL Axiata"),
        ("xl axiata", "XL Axiata"),
        ("indosat", "Indosat Ooredoo Hutchison"),
        ("indosat ooredoo", "Indosat Ooredoo Hutchison"),
        ("ioh", "Indosat Ooredoo Hutchison"),
        ("tri", "3 (Tri)"),
        ("3", "3 (Tri)"),
        ("smartfren", "Smartfren"),
        // E-commerce / Tech
        ("tokopedia", "Tokopedia"),
        ("shopee", "Shopee"),
        ("lazada", "Lazada"),
        ("bukalapak", "Bukalapak"),
        ("blibli", "Blibli"),
        ("gojek", "Gojek"),
        ("goto", "GoTo"),
        ("grab", "Grab"),
        ("traveloka", "Traveloka"),
        ("tiket.com", "Tiket.com"),
        // Government
        ("kemenkeu", "Kementerian Keuangan"),
        ("kementerian keuangan", "Kementerian Keuangan"),
        ("kemendag", "Kementerian Perdagangan"),
        ("kemenkominfo", "Kementerian Komunikasi dan Informatika"),
        ("kominfo", "Kementerian Komunikasi dan Informatika"),
        ("kemenkes", "Kementerian Kesehatan"),
        ("bpjs", "BPJS"),
        ("djp", "Direktorat Jenderal Pajak"),
        ("dirjen pajak", "Direktorat Jenderal Pajak"),
    ])
});

/// Sector classification keywords
static SECTOR_KEYWORDS: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    HashMap::from([
        ("bank", "BANKING"),
        ("banking", "BANKING"),
        ("perbankan", "BANKING"),
        ("asuransi", "INSURANCE"),
        ("insurance", "INSURANCE"),
        ("fintech", "FINTECH"),
        ("sekuritas", "FINANCIAL_SERVICES"),
        ("telkom", "TELECOMMUNICATIONS"),
        ("telekomunikasi", "TELECOMMUNICATIONS"),
        ("provider", "TELECOMMUNICATIONS"),
        ("operator", "TELECOMMUNICATIONS"),
        ("seluler", "TELECOMMUNICATIONS"),
        ("e-commerce", "ECOMMERCE"),
        ("ecommerce", "ECOMMERCE"),
        ("marketplace", "ECOMMERCE"),
        ("online shop", "ECOMMERCE"),
        ("startup", "TECHNOLOGY"),
        ("rumah sakit", "HEALTHCARE"),
        ("rs ", "HEALTHCARE"),
        ("hospital", "HEALTHCARE"),
        ("klinik", "HEALTHCARE"),
        ("universitas", "EDUCATION"),
        ("university", "EDUCATION"),
        ("sekolah", "EDUCATION"),
        ("institut", "EDUCATION"),
        ("kementerian", "GOVERNMENT"),
        ("pemerintah", "GOVERNMENT"),
        ("government", "GOVERNMENT"),
        ("bumn", "GOVERNMENT"),
        ("pln", "ENERGY"),
        ("pertamina", "ENERGY"),
        ("listrik", "ENERGY"),
        ("kereta api", "TRANSPORTATION"),
        ("garuda", "TRANSPORTATION"),
        ("lion air", "TRANSPORTATION"),
        ("pelabuhan", "TRANSPORTATION"),
    ])
});

/// Attack type classification keywords
static ATTACK_KEYWORDS: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    HashMap::from([
        ("ransomware", "RANSOMWARE"),
        ("malware", "MALWARE"),
        ("phishing", "PHISHING"),
        ("spear phishing", "SPEAR_PHISHING"),
        ("ddos", "DDOS"),
        ("denial of service", "DDOS"),
        ("deface", "DEFACEMENT"),
        ("defacement", "DEFACEMENT"),
        ("sql injection", "SQL_INJECTION"),
        ("sqlinj", "SQL_INJECTION"),
        ("xss", "XSS"),
        ("cross site scripting", "XSS"),
        ("data breach", "DATA_BREACH"),
        ("kebocoran data", "DATA_BREACH"),
        ("data leak", "DATA_LEAKAGE"),
        ("pencurian data", "DATA_EXFILTRATION"),
        ("credential stuffing", "CREDENTIAL_STUFFING"),
        ("brute force", "BRUTE_FORCE"),
        ("man in the middle", "MAN_IN_THE_MIDDLE"),
        ("mitm", "MAN_IN_THE_MIDDLE"),
        ("zero day", "ZERO_DAY"),
        ("zero-day", "ZERO_DAY"),
        ("supply chain", "SUPPLY_CHAIN"),
        ("insider", "INSIDER_THREAT"),
        ("hacking", "HACKING"),
        ("diretas", "HACKING"),
        ("serangan", "HACKING"),
    ])
});

/// Indonesian months mapping for date parsing
static INDONESIAN_MONTHS: LazyLock<HashMap<&'static str, u32>> = LazyLock::new(|| {
    HashMap::from([
        ("januari", 1),
        ("februari", 2),
        ("maret", 3),
        ("april", 4),
        ("mei", 5),
        ("juni", 6),
        ("juli", 7),
        ("agustus", 8),
        ("september", 9),
        ("oktober", 10),
        ("november", 11),
        ("desember", 12),
        // English month names also supported
        ("january", 1),
        ("february", 2),
        ("march", 3),
        ("may", 5),
        ("june", 6),
        ("july", 7),
        ("august", 8),
        ("october", 10),
        ("december", 12),
    ])
});

/// Main incident normalizer
pub struct IncidentNormalizer {
    org_regex: Regex,
    date_regexes: Vec<Regex>,
}

impl Default for IncidentNormalizer {
    fn default() -> Self {
        Self::new()
    }
}

impl IncidentNormalizer {
    pub fn new() -> Self {
        // Regex for organization name components
        let org_regex = Regex::new(r"(?i)(PT|CV|TBK|Tbk|Ltd|Inc|Corp|\.\s*|,\s*)")
            .expect("Failed to compile org regex");

        // Date parsing regexes
        let date_regexes = vec![
            // Indonesian: "8 Mei 2024" (Day Month Year)
            Regex::new(r"(\d{1,2})\s+([A-Za-z]+)\s+(\d{4})").expect("date regex"),
            // English: "May 8, 2024" (Month Day, Year)
            Regex::new(r"([A-Za-z]+)\s+(\d{1,2}),?\s+(\d{4})").expect("date regex"),
            // DD/MM/YYYY or DD-MM-YYYY
            Regex::new(r"(\d{1,2})[/-](\d{1,2})[/-](\d{4})").expect("date regex"),
            // YYYY-MM-DD (ISO)
            Regex::new(r"(\d{4})-(\d{2})-(\d{2})").expect("date regex"),
        ];

        Self {
            org_regex,
            date_regexes,
        }
    }

    /// Normalize an organization name to canonical form
    ///
    /// Examples:
    /// - "PT Bank X Tbk" → "Bank X"
    /// - "Bank Central Asia" → "Bank Central Asia"
    /// - "bca" → "Bank Central Asia"
    pub fn normalize_org_name(&self, name: &str) -> (String, f32) {
        let name_lower = name.to_lowercase().trim().to_string();

        // Check for exact match in canonical names first
        if let Some(canonical) = ORG_CANONICAL_NAMES.get(name_lower.as_str()) {
            debug!("Found exact canonical match for: {}", name);
            return (canonical.to_string(), 0.95);
        }

        // Check for substring matches in canonical names
        for (alias, canonical) in ORG_CANONICAL_NAMES.iter() {
            if name_lower.contains(alias) || alias.contains(&name_lower) {
                debug!("Found substring match: {} -> {}", name, canonical);
                return (canonical.to_string(), 0.90);
            }
        }

        // Clean up the name: remove legal entity suffixes
        let cleaned = self.org_regex.replace_all(name, "").trim().to_string();

        if cleaned != name && !cleaned.is_empty() {
            debug!("Cleaned org name: {} -> {}", name, cleaned);
            return (cleaned, 0.75);
        }

        // Return original with lower confidence
        (name.trim().to_string(), 0.50)
    }

    /// Parse a date string with Indonesian format support
    ///
    /// Handles:
    /// - Indonesian: "8 Mei 2024", "1 Januari 2023"
    /// - English: "May 8, 2024", "January 1, 2023"
    /// - Numeric: "08/05/2024", "2024-05-08"
    pub fn parse_date(&self, text: &str) -> Option<(NaiveDate, f32)> {
        // Clean up the text
        let text = text.trim();

        for (idx, regex) in self.date_regexes.iter().enumerate() {
            if let Some(caps) = regex.captures(text) {
                let confidence = match idx {
                    0 => 0.90, // Indonesian: Day Month Year
                    1 => 0.90, // English: Month Day Year
                    2 => 0.80, // DD/MM/YYYY (ambiguous month/day)
                    3 => 0.95, // ISO format (unambiguous)
                    _ => 0.70,
                };

                let date = match idx {
                    0 => {
                        // Indonesian: day month year
                        let day: u32 = caps.get(1)?.as_str().parse().ok()?;
                        let month_str = caps.get(2)?.as_str().to_lowercase();
                        let year: i32 = caps.get(3)?.as_str().parse().ok()?;

                        let month = *INDONESIAN_MONTHS.get(month_str.as_str())?;
                        NaiveDate::from_ymd_opt(year, month, day)?
                    }
                    1 => {
                        // English: month day year
                        let month_str = caps.get(1)?.as_str().to_lowercase();
                        let day: u32 = caps.get(2)?.as_str().parse().ok()?;
                        let year: i32 = caps.get(3)?.as_str().parse().ok()?;

                        let month = *INDONESIAN_MONTHS.get(month_str.as_str())?;
                        NaiveDate::from_ymd_opt(year, month, day)?
                    }
                    2 => {
                        // DD/MM/YYYY format
                        let day: u32 = caps.get(1)?.as_str().parse().ok()?;
                        let month: u32 = caps.get(2)?.as_str().parse().ok()?;
                        let year: i32 = caps.get(3)?.as_str().parse().ok()?;
                        NaiveDate::from_ymd_opt(year, month, day)?
                    }
                    3 => {
                        // ISO format
                        let year: i32 = caps.get(1)?.as_str().parse().ok()?;
                        let month: u32 = caps.get(2)?.as_str().parse().ok()?;
                        let day: u32 = caps.get(3)?.as_str().parse().ok()?;
                        NaiveDate::from_ymd_opt(year, month, day)?
                    }
                    _ => return None,
                };

                // Validate date is reasonable (not in future, not too old)
                let today = chrono::Utc::now().date_naive();
                let min_date = NaiveDate::from_ymd_opt(2020, 1, 1)?;

                if date > today {
                    warn!("Future date detected: {}", date);
                    return Some((today, confidence * 0.5));
                }
                if date < min_date {
                    warn!("Date too old: {}", date);
                    return Some((min_date, confidence * 0.5));
                }

                return Some((date, confidence));
            }
        }

        None
    }

    /// Classify attack type from text
    pub fn classify_attack_type(&self, text: &str) -> (String, f32) {
        let text_lower = text.to_lowercase();

        // Check for keyword matches
        for (keyword, attack_type) in ATTACK_KEYWORDS.iter() {
            if text_lower.contains(keyword) {
                return (attack_type.to_string(), 0.85);
            }
        }

        // Default to UNKNOWN with low confidence
        ("UNKNOWN".to_string(), 0.30)
    }

    /// Classify sector from organization name and text
    pub fn classify_sector(&self, org_name: &str, text: &str) -> (String, f32) {
        let combined = format!("{} {}", org_name, text).to_lowercase();

        // Check sector keywords
        for (keyword, sector) in SECTOR_KEYWORDS.iter() {
            if combined.contains(keyword) {
                return (sector.to_string(), 0.80);
            }
        }

        // Check org name patterns
        let org_lower = org_name.to_lowercase();
        if org_lower.contains("bank")
            || org_lower.contains("bca")
            || org_lower.contains("mandiri")
            || org_lower.contains("bni")
            || org_lower.contains("bri")
        {
            return ("BANKING".to_string(), 0.75);
        }
        if org_lower.contains("telkom")
            || org_lower.contains("xl")
            || org_lower.contains("indosat")
            || org_lower.contains("smartfren")
        {
            return ("TELECOMMUNICATIONS".to_string(), 0.75);
        }
        if org_lower.contains("tokopedia")
            || org_lower.contains("shopee")
            || org_lower.contains("bukalapak")
            || org_lower.contains("gojek")
        {
            return ("ECOMMERCE".to_string(), 0.75);
        }

        ("UNKNOWN".to_string(), 0.30)
    }

    /// Normalize an IncidentDraft and compute per-field confidence
    pub fn normalize(&self, draft: &IncidentDraft) -> (NormalizedIncident, FieldConfidence) {
        // Normalize organization name
        let (org_name, org_confidence) = self.normalize_org_name(&draft.org_name);

        // Classify sector
        let text = draft.raw_content.as_deref().unwrap_or("");
        let (org_sector, sector_confidence) = self.classify_sector(&org_name, text);

        // Parse/normalize incident date
        let (incident_date, date_confidence) = if let Some(date) = draft.incident_date {
            (
                date,
                if Some(draft.disclosure_date) == draft.incident_date {
                    0.60 // Estimated date (same as disclosure)
                } else {
                    0.85 // Explicitly provided
                },
            )
        } else {
            // Try to parse from raw content
            if let Some(content) = &draft.raw_content {
                if let Some((date, conf)) = self.parse_date(content) {
                    (date, conf * 0.80)
                } else {
                    (draft.disclosure_date, 0.40)
                }
            } else {
                (draft.disclosure_date, 0.40)
            }
        };

        // Classify attack type
        let combined_text = format!(
            "{} {}",
            draft.notes.as_deref().unwrap_or(""),
            draft.raw_content.as_deref().unwrap_or("")
        );
        let (attack_type, attack_confidence) = if let Some(ref at) = draft.attack_type {
            (at.clone(), 0.90) // Trust provided attack type
        } else {
            self.classify_attack_type(&combined_text)
        };

        // Data categories confidence
        let data_confidence = if draft.data_categories.is_empty() {
            0.30
        } else {
            0.75
        };

        let confidence = FieldConfidence {
            org_name: org_confidence,
            org_sector: sector_confidence,
            incident_date: date_confidence,
            attack_type: attack_confidence,
            data_categories: data_confidence,
        };

        let normalized = NormalizedIncident {
            org_name,
            org_sector,
            incident_date,
            disclosure_date: draft.disclosure_date,
            attack_type,
            data_categories: draft.data_categories.clone(),
            record_count_estimate: draft.record_count_estimate,
            financial_impact_idr: draft.financial_impact_idr,
            actor_alias: draft.actor_alias.clone(),
            actor_group: draft.actor_group.clone(),
            source_url: draft.source_url.clone(),
            source_type: draft.source_type.clone(),
            notes: draft.notes.clone(),
            overall_confidence: confidence.overall(),
        };

        (normalized, confidence)
    }
}

/// Normalized incident ready for database storage
#[derive(Debug, Clone)]
pub struct NormalizedIncident {
    pub org_name: String,
    pub org_sector: String,
    pub incident_date: NaiveDate,
    pub disclosure_date: NaiveDate,
    pub attack_type: String,
    pub data_categories: Vec<String>,
    pub record_count_estimate: Option<i32>,
    pub financial_impact_idr: Option<i64>,
    pub actor_alias: Option<String>,
    pub actor_group: Option<String>,
    pub source_url: String,
    pub source_type: String,
    pub notes: Option<String>,
    pub overall_confidence: f32,
}

impl NormalizedIncident {
    /// Convert to schema::CreateIncident for database insertion
    pub fn to_create_incident(&self) -> schema::models::incident::CreateIncident {
        use schema::models::incident::CreateIncident;

        CreateIncident {
            org_name: self.org_name.clone(),
            org_sector: self.org_sector.clone(),
            incident_date: self.incident_date.into(),
            disclosure_date: self.disclosure_date.into(),
            attack_type: self.attack_type.clone(),
            data_categories: self.data_categories.clone(),
            record_count_estimate: self.record_count_estimate,
            financial_impact_idr: self.financial_impact_idr,
            actor_alias: self.actor_alias.clone(),
            actor_group: self.actor_group.clone(),
            source_url: self.source_url.clone(),
            source_type: self.source_type.clone(),
            notes: self.notes.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

    #[test]
    fn test_org_name_normalization() {
        let normalizer = IncidentNormalizer::new();

        // Test canonical name mapping
        let (name, conf) = normalizer.normalize_org_name("PT Bank Central Asia Tbk");
        assert_eq!(name, "Bank Central Asia");
        assert!(conf >= 0.75);

        let (name, conf) = normalizer.normalize_org_name("bca");
        assert_eq!(name, "Bank Central Asia");
        assert!(conf >= 0.90);

        let (name, conf) = normalizer.normalize_org_name("Bank Mandiri");
        assert_eq!(name, "Bank Mandiri");
        assert!(conf >= 0.90);

        // Test cleaning
        let (name, conf) = normalizer.normalize_org_name("PT Some Company Ltd.");
        assert!(!name.contains("PT"));
        assert!(!name.contains("Ltd"));
        assert!(conf >= 0.70);
    }

    #[test]
    fn test_date_parsing_indonesian() {
        let normalizer = IncidentNormalizer::new();

        // Indonesian format
        let (date, conf) = normalizer.parse_date("8 Mei 2024").unwrap();
        assert_eq!(date.year(), 2024);
        assert_eq!(date.month(), 5);
        assert_eq!(date.day(), 8);
        assert!(conf >= 0.85);

        let (date, conf) = normalizer.parse_date("1 Januari 2023").unwrap();
        assert_eq!(date.year(), 2023);
        assert_eq!(date.month(), 1);
        assert_eq!(date.day(), 1);

        // English format
        let (date, _) = normalizer.parse_date("May 8, 2024").unwrap();
        assert_eq!(date.month(), 5);
        assert_eq!(date.day(), 8);
    }

    #[test]
    fn test_date_parsing_iso() {
        let normalizer = IncidentNormalizer::new();

        let (date, conf) = normalizer.parse_date("2024-05-08").unwrap();
        assert_eq!(date.year(), 2024);
        assert_eq!(date.month(), 5);
        assert_eq!(date.day(), 8);
        assert!(conf >= 0.90);
    }

    #[test]
    fn test_attack_classification() {
        let normalizer = IncidentNormalizer::new();

        let (attack, conf) = normalizer.classify_attack_type("ransomware attack terjadi");
        assert_eq!(attack, "RANSOMWARE");
        assert!(conf >= 0.80);

        let (attack, conf) = normalizer.classify_attack_type("phishing email detected");
        assert_eq!(attack, "PHISHING");
        assert!(conf >= 0.80);

        let (attack, _) = normalizer.classify_attack_type("some random text");
        assert_eq!(attack, "UNKNOWN");
    }

    #[test]
    fn test_sector_classification() {
        let normalizer = IncidentNormalizer::new();

        let (sector, conf) = normalizer.classify_sector("Bank BCA", "some text");
        assert_eq!(sector, "BANKING");
        assert!(conf >= 0.75);

        let (sector, conf) =
            normalizer.classify_sector("Some Company", "perusahaan telekomunikasi");
        assert_eq!(sector, "TELECOMMUNICATIONS");
        assert!(conf >= 0.75);

        let (sector, _) = normalizer.classify_sector("Unknown", "text");
        assert_eq!(sector, "UNKNOWN");
    }

    #[test]
    fn test_field_confidence_overall() {
        let fc = FieldConfidence {
            org_name: 0.9,
            org_sector: 0.8,
            incident_date: 0.7,
            attack_type: 0.6,
            data_categories: 0.5,
        };

        let overall = fc.overall();
        // Should be weighted average
        assert!(overall > 0.5 && overall < 0.9);
    }

    #[test]
    fn test_full_normalization() {
        let normalizer = IncidentNormalizer::new();

        let draft = IncidentDraft::new(
            "PT Bank BCA Tbk".to_string(),
            NaiveDate::from_ymd_opt(2024, 5, 8).unwrap(),
            "https://example.com".to_string(),
            "TEST".to_string(),
        )
        .with_attack_type(Some("RANSOMWARE".to_string()))
        .with_org_sector(Some("BANKING".to_string()))
        .with_raw_content(Some("Bank BCA mengalami serangan ransomware".to_string()))
        .with_confidence(0.7);

        let (normalized, confidence) = normalizer.normalize(&draft);

        assert_eq!(normalized.org_name, "Bank Central Asia");
        assert_eq!(normalized.org_sector, "BANKING");
        assert_eq!(normalized.attack_type, "RANSOMWARE");
        assert!(confidence.org_name >= 0.90); // High confidence for canonical name
        assert!(confidence.overall() > 0.0);
    }
}
