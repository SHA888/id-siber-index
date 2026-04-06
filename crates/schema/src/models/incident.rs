//! Incident data models

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use uuid::Uuid;

/// Serializable wrapper for NaiveDate
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SerializableDate(pub NaiveDate);

impl Serialize for SerializableDate {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.format("%Y-%m-%d").to_string().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for SerializableDate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        NaiveDate::parse_from_str(&s, "%Y-%m-%d")
            .map(SerializableDate)
            .map_err(serde::de::Error::custom)
    }
}

impl From<NaiveDate> for SerializableDate {
    fn from(date: NaiveDate) -> Self {
        Self(date)
    }
}

impl From<SerializableDate> for NaiveDate {
    fn from(date: SerializableDate) -> Self {
        date.0
    }
}

/// Incident record as exposed through the API
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Incident {
    pub id: Uuid,
    pub org_name: String,
    pub org_sector: String,
    #[serde(flatten)]
    pub incident_date: SerializableDate,
    #[serde(flatten)]
    pub disclosure_date: SerializableDate,
    pub attack_type: String,
    pub data_categories: Vec<String>,
    pub record_count_estimate: Option<i32>,
    pub financial_impact_idr: Option<i64>,
    pub actor_alias: Option<String>,
    pub actor_group: Option<String>,
    pub source_url: String,
    pub source_type: String,
    pub verified: bool,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Incident creation request
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateIncident {
    pub org_name: String,
    pub org_sector: String,
    #[serde(flatten)]
    pub incident_date: SerializableDate,
    #[serde(flatten)]
    pub disclosure_date: SerializableDate,
    pub attack_type: String,
    pub data_categories: Vec<String>,
    pub record_count_estimate: Option<i32>,
    pub financial_impact_idr: Option<i64>,
    pub actor_alias: Option<String>,
    pub actor_group: Option<String>,
    pub source_url: String,
    pub source_type: String,
    pub notes: Option<String>,
}

/// Incident update request
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateIncident {
    pub org_name: Option<String>,
    pub org_sector: Option<String>,
    #[serde(flatten)]
    pub incident_date: Option<SerializableDate>,
    #[serde(flatten)]
    pub disclosure_date: Option<SerializableDate>,
    pub attack_type: Option<String>,
    pub data_categories: Option<Vec<String>>,
    pub record_count_estimate: Option<i32>,
    pub financial_impact_idr: Option<i64>,
    pub actor_alias: Option<String>,
    pub actor_group: Option<String>,
    pub source_url: Option<String>,
    pub source_type: Option<String>,
    pub verified: Option<bool>,
    pub notes: Option<String>,
}

/// Incident search parameters
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IncidentSearchParams {
    pub query: Option<String>,
    pub org_name: Option<String>,
    pub sector: Option<String>,
    pub attack_type: Option<String>,
    pub verified: Option<bool>,
    #[serde(flatten)]
    pub from_date: Option<SerializableDate>,
    #[serde(flatten)]
    pub to_date: Option<SerializableDate>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

impl Default for IncidentSearchParams {
    fn default() -> Self {
        Self {
            query: None,
            org_name: None,
            sector: None,
            attack_type: None,
            verified: None,
            from_date: None,
            to_date: None,
            limit: Some(50),
            offset: Some(0),
        }
    }
}
