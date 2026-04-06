//! Incident entity definition

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "incidents")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub org_name: String,
    pub org_sector: String,
    pub incident_date: Date,
    pub disclosure_date: Date,
    pub attack_type: String,
    pub data_categories: Json, // Vec<String> - will be properly typed in future
    pub record_count_estimate: Option<i32>,
    pub financial_impact_idr: Option<i64>,
    pub actor_alias: Option<String>,
    pub actor_group: Option<String>,
    pub source_url: String,
    pub source_type: String,
    pub verified: bool,
    pub notes: Option<String>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
