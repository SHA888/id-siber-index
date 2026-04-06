//! API request handlers

pub mod incident;
pub mod search;
pub mod stats;

pub use incident::*;
pub use search::*;
pub use stats::*;

// Type aliases for handler functions
pub type IncidentHandler = fn() -> axum::response::Json<Vec<schema::models::incident::Incident>>;
pub type SearchHandler = fn() -> axum::response::Json<Vec<schema::models::incident::Incident>>;
pub type StatsHandler = fn() -> axum::response::Json<serde_json::Value>;
