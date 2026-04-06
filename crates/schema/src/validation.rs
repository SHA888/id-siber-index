//! Data validation utilities

use chrono::NaiveDate;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Invalid organization name: {0}")]
    InvalidOrgName(String),
    #[error("Invalid sector: {0}")]
    InvalidSector(String),
    #[error("Invalid attack type: {0}")]
    InvalidAttackType(String),
    #[error("Invalid date: {0}")]
    InvalidDate(String),
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
    #[error("Invalid data categories: {0}")]
    InvalidDataCategories(String),
}

/// Validator for incident records
pub struct IncidentValidator;

impl IncidentValidator {
    /// Validate an incident record
    pub fn validate_incident(
        incident: &crate::models::incident::CreateIncident,
    ) -> Result<(), ValidationError> {
        // Validate organization name
        if incident.org_name.trim().is_empty() {
            return Err(ValidationError::InvalidOrgName(
                "Organization name cannot be empty".to_string(),
            ));
        }

        // Validate sector
        if incident.org_sector.trim().is_empty() {
            return Err(ValidationError::InvalidSector(
                "Sector cannot be empty".to_string(),
            ));
        }

        // Validate attack type
        if incident.attack_type.trim().is_empty() {
            return Err(ValidationError::InvalidAttackType(
                "Attack type cannot be empty".to_string(),
            ));
        }

        // Validate dates
        let incident_date: NaiveDate = incident.incident_date.clone().into();
        let disclosure_date: NaiveDate = incident.disclosure_date.clone().into();

        if incident_date > disclosure_date {
            return Err(ValidationError::InvalidDate(
                "Incident date cannot be after disclosure date".to_string(),
            ));
        }

        // Validate URL
        if incident.source_url.trim().is_empty() {
            return Err(ValidationError::InvalidUrl(
                "Source URL cannot be empty".to_string(),
            ));
        }

        // Validate source type
        if incident.source_type.trim().is_empty() {
            return Err(ValidationError::InvalidUrl(
                "Source type cannot be empty".to_string(),
            ));
        }

        // Validate data categories
        if incident.data_categories.is_empty() {
            return Err(ValidationError::InvalidDataCategories(
                "At least one data category must be specified".to_string(),
            ));
        }

        Ok(())
    }
}
