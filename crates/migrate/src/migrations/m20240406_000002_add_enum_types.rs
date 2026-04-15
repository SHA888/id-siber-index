//! Add enum types and update incidents table

#![allow(clippy::enum_variant_names)]

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create sources table for audit trail
        manager
            .create_table(
                Table::create()
                    .table(Source::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Source::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Source::IncidentId).uuid().not_null())
                    .col(ColumnDef::new(Source::SourceType).string().not_null())
                    .col(ColumnDef::new(Source::SourceUrl).string().not_null())
                    .col(ColumnDef::new(Source::Title).string())
                    .col(ColumnDef::new(Source::Author).string())
                    .col(ColumnDef::new(Source::PublishDate).timestamp_with_time_zone())
                    .col(
                        ColumnDef::new(Source::ExtractedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Source::Verified)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(ColumnDef::new(Source::Notes).text())
                    .col(
                        ColumnDef::new(Source::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Source::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_source_incident")
                            .from(Source::Table, Source::IncidentId)
                            .to(Incident::Table, Incident::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .name("idx_source_incident_id")
                            .col(Source::IncidentId),
                    )
                    .index(
                        Index::create()
                            .name("idx_source_type")
                            .col(Source::SourceType),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop sources table
        manager
            .drop_table(Table::drop().table(Source::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Incident {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Source {
    Table,
    Id,
    IncidentId,
    SourceType,
    SourceUrl,
    Title,
    Author,
    PublishDate,
    ExtractedAt,
    Verified,
    Notes,
    CreatedAt,
    UpdatedAt,
}
