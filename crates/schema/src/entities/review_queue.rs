//! Review queue entity definition
//!
//! Tracks incidents under review with escalation support per REVIEW_SKILLS.md
//! principles 3.2 (escalation) and 3.5 (dashboard)

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "review_queue")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub incident_id: Uuid,
    pub status: String,
    pub reviewer_id: Option<String>,
    pub escalated_by: Option<String>,
    pub escalation_reason: Option<String>,
    pub escalated_at: Option<DateTimeWithTimeZone>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::incident::Entity",
        from = "Column::IncidentId",
        to = "super::incident::Column::Id"
    )]
    Incident,
}

impl Related<super::incident::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Incident.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

/// Review queue status types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReviewQueueStatus {
    Pending,
    InReview,
    Accepted,
    Rejected,
    Escalated,
}

impl ReviewQueueStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ReviewQueueStatus::Pending => "PENDING",
            ReviewQueueStatus::InReview => "IN_REVIEW",
            ReviewQueueStatus::Accepted => "ACCEPTED",
            ReviewQueueStatus::Rejected => "REJECTED",
            ReviewQueueStatus::Escalated => "ESCALATED",
        }
    }
}

impl std::fmt::Display for ReviewQueueStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for ReviewQueueStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "PENDING" => Ok(ReviewQueueStatus::Pending),
            "IN_REVIEW" => Ok(ReviewQueueStatus::InReview),
            "ACCEPTED" => Ok(ReviewQueueStatus::Accepted),
            "REJECTED" => Ok(ReviewQueueStatus::Rejected),
            "ESCALATED" => Ok(ReviewQueueStatus::Escalated),
            _ => Err(format!("Unknown review queue status: {}", s)),
        }
    }
}
