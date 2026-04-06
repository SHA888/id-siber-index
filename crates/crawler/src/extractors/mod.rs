//! Data extraction utilities

use serde::{Deserialize, Serialize};

pub mod html;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionResult {
    pub org_name: String,
    pub org_sector: String,
    pub incident_date: chrono::NaiveDate,
    pub disclosure_date: chrono::NaiveDate,
    pub attack_type: String,
    pub data_categories: Vec<String>,
    pub record_count_estimate: Option<i32>,
    pub financial_impact_idr: Option<i64>,
    pub actor_alias: Option<String>,
    pub actor_group: Option<String>,
    pub source_url: String,
    pub source_type: String,
    pub notes: Option<String>,
    pub confidence: f32,
}

pub trait DataExtractor: Send + Sync {
    fn extract(&self, content: &str, url: &str) -> Result<Vec<ExtractionResult>, anyhow::Error>;
}
