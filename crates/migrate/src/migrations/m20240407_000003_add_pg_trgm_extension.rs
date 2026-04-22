//! Add pg_trgm extension for full-text search

#![allow(clippy::enum_variant_names)]

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create pg_trgm extension for efficient full-text search
        manager
            .get_connection()
            .execute_unprepared("CREATE EXTENSION IF NOT EXISTS \"pg_trgm\";")
            .await?;

        // Create GIN indexes for text search on organization name and notes
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE INDEX IF NOT EXISTS idx_incidents_org_name_gin 
                ON incidents USING gin(org_name gin_trgm_ops);
                "#,
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE INDEX IF NOT EXISTS idx_incidents_notes_gin 
                ON incidents USING gin(notes gin_trgm_ops);
                "#,
            )
            .await?;

        // Also create indexes for sector and updated_at as specified in TODO.md
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE INDEX IF NOT EXISTS idx_incidents_sector 
                ON incidents(org_sector);
                "#,
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE INDEX IF NOT EXISTS idx_incidents_updated 
                ON incidents(updated_at);
                "#,
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop indexes (extension cannot be dropped if other objects depend on it)
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                DROP INDEX IF EXISTS idx_incidents_org_name_gin;
                DROP INDEX IF EXISTS idx_incidents_notes_gin;
                DROP INDEX IF EXISTS idx_incidents_sector;
                DROP INDEX IF EXISTS idx_incidents_updated;
                "#,
            )
            .await?;

        // Note: We don't drop the pg_trgm extension as it might be used by other tables
        // and dropping extensions requires special privileges
        
        Ok(())
    }
}