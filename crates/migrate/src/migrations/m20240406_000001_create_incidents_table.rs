//! Create incidents table migration

#![allow(clippy::enum_variant_names)]

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Incident::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Incident::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Incident::OrgName).string().not_null())
                    .col(ColumnDef::new(Incident::OrgSector).string().not_null())
                    .col(ColumnDef::new(Incident::IncidentDate).date().not_null())
                    .col(ColumnDef::new(Incident::DisclosureDate).date().not_null())
                    .col(ColumnDef::new(Incident::AttackType).string().not_null())
                    .col(ColumnDef::new(Incident::DataCategories).json().not_null())
                    .col(ColumnDef::new(Incident::RecordCountEstimate).integer())
                    .col(ColumnDef::new(Incident::FinancialImpactIdr).big_integer())
                    .col(ColumnDef::new(Incident::ActorAlias).string())
                    .col(ColumnDef::new(Incident::ActorGroup).string())
                    .col(ColumnDef::new(Incident::SourceUrl).string().not_null())
                    .col(ColumnDef::new(Incident::SourceType).string().not_null())
                    .col(
                        ColumnDef::new(Incident::Verified)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(ColumnDef::new(Incident::Notes).text())
                    .col(
                        ColumnDef::new(Incident::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Incident::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .index(
                        Index::create()
                            .name("idx_incident_org_name")
                            .col(Incident::OrgName),
                    )
                    .index(
                        Index::create()
                            .name("idx_incident_sector")
                            .col(Incident::OrgSector),
                    )
                    .index(
                        Index::create()
                            .name("idx_incident_date")
                            .col(Incident::IncidentDate),
                    )
                    .index(
                        Index::create()
                            .name("idx_incident_attack_type")
                            .col(Incident::AttackType),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Incident::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Incident {
    Table,
    Id,
    OrgName,
    OrgSector,
    IncidentDate,
    DisclosureDate,
    AttackType,
    DataCategories,
    RecordCountEstimate,
    FinancialImpactIdr,
    ActorAlias,
    ActorGroup,
    SourceUrl,
    SourceType,
    Verified,
    Notes,
    CreatedAt,
    UpdatedAt,
}
