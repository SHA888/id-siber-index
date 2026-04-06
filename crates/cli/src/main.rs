//! Command-line interface for the Indonesia Cybersecurity Incident Index
//!
//! This CLI tool provides commands for managing the index, running crawlers,
//! starting the API server, and performing administrative tasks.

use anyhow::Result;
use clap::{Parser, Subcommand};

mod commands;

#[derive(Parser)]
#[command(name = "idsiber")]
#[command(about = "Command-line interface for id-siber-index")]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the API server
    Serve {
        #[arg(short, long, default_value = "8080")]
        port: u16,
        #[arg(short, long, default_value = "0.0.0.0")]
        host: String,
    },
    /// Run the crawler
    Crawl {
        #[arg(short, long, help = "Specific source to crawl (idx, bssn, ojk, media)")]
        source: Option<String>,
        #[arg(long, help = "Run in continuous mode")]
        continuous: bool,
    },
    /// Database operations
    Db {
        #[command(subcommand)]
        command: commands::DbCommands,
    },
    /// Search incidents
    Search {
        query: String,
        #[arg(short, long, help = "Limit number of results")]
        limit: Option<usize>,
        #[arg(short, long, help = "Filter by sector")]
        sector: Option<String>,
        #[arg(short, long, help = "Filter by attack type")]
        attack_type: Option<String>,
    },
    /// Import data from file
    Import {
        file: String,
        #[arg(short, long, help = "File format (json, csv)")]
        format: Option<String>,
    },
    /// Export data to file
    Export {
        file: String,
        #[arg(short, long, help = "File format (json, csv)")]
        format: Option<String>,
        #[arg(short, long, help = "Filter by sector")]
        sector: Option<String>,
    },
}

#[derive(Subcommand)]
enum DbCommands {
    /// Run database migrations
    Migrate,
    /// Reset database (dangerous!)
    Reset,
    /// Show database status
    Status,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Serve { port, host } => {
            commands::serve::run(port, host).await?;
        }
        Commands::Crawl { source, continuous } => {
            commands::crawl::run(source, continuous).await?;
        }
        Commands::Db { command } => {
            commands::db::run(command).await?;
        }
        Commands::Search {
            query,
            limit,
            sector,
            attack_type,
        } => {
            commands::search::run(query, limit, sector, attack_type).await?;
        }
        Commands::Import { file, format } => {
            commands::import::run(file, format).await?;
        }
        Commands::Export {
            file,
            format,
            sector,
        } => {
            commands::export::run(file, format, sector).await?;
        }
    }

    Ok(())
}
