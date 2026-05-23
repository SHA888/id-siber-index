//! Create review_metrics table migration

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
                    .table(ReviewMetrics::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ReviewMetrics::Date)
                            .date()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ReviewMetrics::IncidentsReviewed)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(ReviewMetrics::AcceptanceRate)
                            .decimal_len(3, 2)
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(ReviewMetrics::EditRate)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(ReviewMetrics::EscalationRate)
                            .decimal_len(3, 2)
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(ReviewMetrics::MeanReviewTimeMinutes)
                            .decimal_len(8, 2)
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(ReviewMetrics::DefectEscapeCount)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(ReviewMetrics::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(ReviewMetrics::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .index(
                        Index::create()
                            .name("idx_review_metrics_date")
                            .col(ReviewMetrics::Date),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ReviewMetrics::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum ReviewMetrics {
    Table,
    Date,
    IncidentsReviewed,
    AcceptanceRate,
    EditRate,
    EscalationRate,
    MeanReviewTimeMinutes,
    DefectEscapeCount,
    CreatedAt,
    UpdatedAt,
}
