//! Create review_audit_log table migration

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
                    .table(ReviewAuditLog::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ReviewAuditLog::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ReviewAuditLog::IncidentId).uuid().not_null())
                    .col(
                        ColumnDef::new(ReviewAuditLog::ReviewerId)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(ReviewAuditLog::Action).string().not_null())
                    .col(
                        ColumnDef::new(ReviewAuditLog::ReviewedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(ColumnDef::new(ReviewAuditLog::Justification).text())
                    .col(ColumnDef::new(ReviewAuditLog::ConfidenceScore).decimal_len(3, 2))
                    .col(ColumnDef::new(ReviewAuditLog::PriorStatus).json())
                    .col(ColumnDef::new(ReviewAuditLog::PostStatus).json())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_review_audit_log_incident_id")
                            .from(ReviewAuditLog::Table, ReviewAuditLog::IncidentId)
                            .to(Incident::Table, Incident::Id),
                    )
                    .index(
                        Index::create()
                            .name("idx_review_audit_log_incident_id")
                            .col(ReviewAuditLog::IncidentId),
                    )
                    .index(
                        Index::create()
                            .name("idx_review_audit_log_reviewer_id")
                            .col(ReviewAuditLog::ReviewerId),
                    )
                    .index(
                        Index::create()
                            .name("idx_review_audit_log_reviewed_at")
                            .col(ReviewAuditLog::ReviewedAt),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ReviewAuditLog::Table).to_owned())
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
    IncidentId,
    ReviewerId,
    Action,
    ReviewedAt,
    Justification,
    ConfidenceScore,
    PriorStatus,
    PostStatus,
}
