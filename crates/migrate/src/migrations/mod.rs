//! Database migrations for the Indonesia Cybersecurity Incident Index

use sea_orm_migration::prelude::*;

mod m20240406_000001_create_incidents_table;
mod m20240406_000002_add_enum_types;
mod m20240407_000003_add_pg_trgm_extension;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240406_000001_create_incidents_table::Migration),
            Box::new(m20240406_000002_add_enum_types::Migration),
            Box::new(m20240407_000003_add_pg_trgm_extension::Migration),
        ]
    }
}
