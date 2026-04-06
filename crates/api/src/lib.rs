//! REST API server for the Indonesia Cybersecurity Incident Index
//!
//! This crate provides the HTTP API layer for serving incident data, handling
//! search requests, and managing user interactions with the index.

pub mod error;
pub mod handlers;
pub mod middleware;
pub mod routes;
pub mod server;

pub use error::*;
pub use handlers::*;
pub use middleware::*;
pub use routes::*;
pub use server::*;

/// Re-export commonly used types
pub mod prelude {
    pub use crate::error::ApiError;
    pub use crate::handlers::{IncidentHandler, SearchHandler, StatsHandler};
    pub use crate::server::ApiServer;
}
