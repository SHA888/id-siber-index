//! CLI command implementations

use clap::Subcommand;

pub mod crawl;
pub mod db;
pub mod export;
pub mod import;
pub mod search;
pub mod serve;

#[derive(Subcommand)]
pub enum DbCommands {
    /// Run database migrations
    Migrate,
    /// Reset database (dangerous!)
    Reset,
    /// Show database status
    Status,
}
