//! Incident draft data structure for crawler normalization

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// Normalized incident data structure for crawler outputs
/// This represents a draft incident that needs to be validated and processed
/// before being stored in the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncidentDraft {
    /// Organization name as extracted from source
    pub org_name: String,

    /// Organization sector (may be empty or inferred)
    pub org_sector: Option<String>,

    /// Date when the incident occurred (may be estimated)
    pub incident_date: Option<NaiveDate>,

    /// Date when the incident was disclosed
    pub disclosure_date: NaiveDate,

    /// Type of cyber attack (may be empty or inferred)
    pub attack_type: Option<String>,

    /// Categories of data affected (may be empty)
    pub data_categories: Vec<String>,

    /// Estimated number of records affected (may be unknown)
    pub record_count_estimate: Option<i32>,

    /// Financial impact in IDR (may be unknown)
    pub financial_impact_idr: Option<i64>,

    /// Actor alias or group responsible (may be unknown)
    pub actor_alias: Option<String>,
    pub actor_group: Option<String>,

    /// Source URL where the information was found
    pub source_url: String,

    /// Type of source (e.g., "IDX_DISCLOSURE", "PRESS_RELEASE")
    pub source_type: String,

    /// Additional notes or context
    pub notes: Option<String>,

    /// Confidence score (0.0-1.0) of the extraction quality
    pub confidence: f32,

    /// Raw text content for further processing
    pub raw_content: Option<String>,
}

impl IncidentDraft {
    /// Create a new incident draft with minimal required fields
    pub fn new(
        org_name: String,
        disclosure_date: NaiveDate,
        source_url: String,
        source_type: String,
    ) -> Self {
        Self {
            org_name,
            org_sector: None,
            incident_date: None,
            disclosure_date,
            attack_type: None,
            data_categories: Vec::new(),
            record_count_estimate: None,
            financial_impact_idr: None,
            actor_alias: None,
            actor_group: None,
            source_url,
            source_type,
            notes: None,
            confidence: 0.5, // Default confidence
            raw_content: None,
        }
    }

    /// Set the organization sector
    pub fn with_org_sector(mut self, sector: Option<String>) -> Self {
        self.org_sector = sector;
        self
    }

    /// Set the incident date
    pub fn with_incident_date(mut self, date: Option<NaiveDate>) -> Self {
        self.incident_date = date;
        self
    }

    /// Set the attack type
    pub fn with_attack_type(mut self, attack_type: Option<String>) -> Self {
        self.attack_type = attack_type;
        self
    }

    /// Set the data categories
    pub fn with_data_categories(mut self, categories: Vec<String>) -> Self {
        self.data_categories = categories;
        self
    }

    /// Set the confidence score
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }

    /// Set raw content
    pub fn with_raw_content(mut self, content: Option<String>) -> Self {
        self.raw_content = content;
        self
    }

    /// Set notes
    pub fn with_notes(mut self, notes: Option<String>) -> Self {
        self.notes = notes;
        self
    }

    /// Check if this draft might be a duplicate of another draft
    /// based on organization name and disclosure date proximity
    pub fn is_potential_duplicate(&self, other: &IncidentDraft, date_window_days: i64) -> bool {
        if self.org_name.to_lowercase() != other.org_name.to_lowercase() {
            return false;
        }

        let date_diff = (self.disclosure_date - other.disclosure_date)
            .num_days()
            .abs();
        date_diff <= date_window_days
    }

    /// Convert to ExtractionResult (for compatibility with existing crawler interface)
    pub fn to_extraction_result(&self) -> crate::extractors::ExtractionResult {
        crate::extractors::ExtractionResult {
            org_name: self.org_name.clone(),
            org_sector: self
                .org_sector
                .clone()
                .unwrap_or_else(|| "UNKNOWN".to_string()),
            incident_date: self.incident_date.unwrap_or(self.disclosure_date),
            disclosure_date: self.disclosure_date,
            attack_type: self
                .attack_type
                .clone()
                .unwrap_or_else(|| "UNKNOWN".to_string()),
            data_categories: self.data_categories.clone(),
            record_count_estimate: self.record_count_estimate,
            financial_impact_idr: self.financial_impact_idr,
            actor_alias: self.actor_alias.clone(),
            actor_group: self.actor_group.clone(),
            source_url: self.source_url.clone(),
            source_type: self.source_type.clone(),
            notes: self.notes.clone(),
            confidence: self.confidence,
        }
    }
}
