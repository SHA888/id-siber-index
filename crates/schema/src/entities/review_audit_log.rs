//! Review audit log entity definition
//!
//! Tracks every review action with accountability per REVIEW_SKILLS.md
//! principles 2.2 (citation) and 2.5 (signing).

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "review_audit_log")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub incident_id: Uuid,
    pub reviewer_id: String,
    pub action: String,
    pub reviewed_at: DateTimeWithTimeZone,
    pub justification: Option<String>,
    pub confidence_score: Option<f32>,
    pub prior_status: Option<Json>,
    pub post_status: Option<Json>,
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

/// Review action types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReviewAction {
    Accepted,
    Rejected,
    Edited,
    Skipped,
}

impl ReviewAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            ReviewAction::Accepted => "ACCEPTED",
            ReviewAction::Rejected => "REJECTED",
            ReviewAction::Edited => "EDITED",
            ReviewAction::Skipped => "SKIPPED",
        }
    }
}

impl std::fmt::Display for ReviewAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for ReviewAction {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "ACCEPTED" => Ok(ReviewAction::Accepted),
            "REJECTED" => Ok(ReviewAction::Rejected),
            "EDITED" => Ok(ReviewAction::Edited),
            "SKIPPED" => Ok(ReviewAction::Skipped),
            _ => Err(format!("Unknown review action: {}", s)),
        }
    }
}
