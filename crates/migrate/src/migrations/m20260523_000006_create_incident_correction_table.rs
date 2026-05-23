//! Create incident_correction table migration

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
                    .table(IncidentCorrection::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(IncidentCorrection::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(IncidentCorrection::IncidentId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(IncidentCorrection::CorrectionType)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(IncidentCorrection::ReportedBy)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(IncidentCorrection::ReportedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(IncidentCorrection::OriginalReviewId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(IncidentCorrection::Description)
                            .text()
                            .not_null(),
                    )
                    .col(ColumnDef::new(IncidentCorrection::ResolvedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(IncidentCorrection::ResolutionAction).text())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_incident_correction_incident_id")
                            .from(IncidentCorrection::Table, IncidentCorrection::IncidentId)
                            .to(Incident::Table, Incident::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_incident_correction_original_review_id")
                            .from(
                                IncidentCorrection::Table,
                                IncidentCorrection::OriginalReviewId,
                            )
                            .to(ReviewAuditLog::Table, ReviewAuditLog::Id),
                    )
                    .index(
                        Index::create()
                            .name("idx_incident_correction_incident_id")
                            .col(IncidentCorrection::IncidentId),
                    )
                    .index(
                        Index::create()
                            .name("idx_incident_correction_original_review_id")
                            .col(IncidentCorrection::OriginalReviewId),
                    )
                    .index(
                        Index::create()
                            .name("idx_incident_correction_correction_type")
                            .col(IncidentCorrection::CorrectionType),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(IncidentCorrection::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Incident {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum ReviewAuditLog {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum IncidentCorrection {
    Table,
    Id,
    IncidentId,
    CorrectionType,
    ReportedBy,
    ReportedAt,
    OriginalReviewId,
    Description,
    ResolvedAt,
    ResolutionAction,
}
