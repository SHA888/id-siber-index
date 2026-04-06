//! Data schema and models for the Indonesia Cybersecurity Incident Index
//!
//! This crate defines the core data structures, database entities, and validation
//! logic for incident records in the id-siber-index project.

pub mod entities;
pub mod models;
pub mod validation;

pub use entities::*;
pub use models::*;
pub use validation::*;

/// Re-export commonly used types
pub mod prelude {
    pub use crate::entities::incident::*;
    pub use crate::models::incident::*;
    pub use crate::validation::IncidentValidator;
}
