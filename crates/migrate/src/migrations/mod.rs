//! Database migrations for the Indonesia Cybersecurity Incident Index

use sea_orm_migration::prelude::*;

mod m20240406_000001_create_incidents_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20240406_000001_create_incidents_table::Migration)]
    }
}
