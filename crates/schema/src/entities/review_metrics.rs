//! Review metrics entity - daily aggregated metrics from review_audit_log

use chrono::{DateTime, NaiveDate, Utc};
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "review_metrics")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub date: NaiveDate,
    pub incidents_reviewed: i32,
    pub acceptance_rate: f64,
    pub edit_rate: f64,
    pub escalation_rate: f64,
    pub mean_review_time_minutes: f64,
    pub defect_escape_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
