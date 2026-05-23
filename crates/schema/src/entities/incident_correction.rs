//! Incident correction entity - tracks defects that escaped the review process

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "incident_correction")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub incident_id: Uuid,
    pub correction_type: String,
    pub reported_by: String,
    pub reported_at: DateTime<Utc>,
    pub original_review_id: Uuid,
    pub description: String,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolution_action: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::incident::Entity",
        from = "Column::IncidentId",
        to = "super::incident::Column::Id"
    )]
    Incident,
    #[sea_orm(
        belongs_to = "super::review_audit_log::Entity",
        from = "Column::OriginalReviewId",
        to = "super::review_audit_log::Column::Id"
    )]
    ReviewAuditLog,
}

impl Related<super::incident::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Incident.def()
    }
}

impl Related<super::review_audit_log::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ReviewAuditLog.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

/// Correction type enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CorrectionType {
    FactualError,
    WrongSector,
    WrongAttackType,
    DuplicateMerge,
    Other,
}

impl CorrectionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            CorrectionType::FactualError => "FACTUAL_ERROR",
            CorrectionType::WrongSector => "WRONG_SECTOR",
            CorrectionType::WrongAttackType => "WRONG_ATTACK_TYPE",
            CorrectionType::DuplicateMerge => "DUPLICATE_MERGE",
            CorrectionType::Other => "OTHER",
        }
    }
}

impl std::fmt::Display for CorrectionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for CorrectionType {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "FACTUAL_ERROR" => Ok(CorrectionType::FactualError),
            "WRONG_SECTOR" => Ok(CorrectionType::WrongSector),
            "WRONG_ATTACK_TYPE" => Ok(CorrectionType::WrongAttackType),
            "DUPLICATE_MERGE" => Ok(CorrectionType::DuplicateMerge),
            "OTHER" => Ok(CorrectionType::Other),
            _ => Err(format!("Unknown correction type: {}", s)),
        }
    }
}
