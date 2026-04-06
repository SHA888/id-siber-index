//! Web crawler and data extraction for the Indonesia Cybersecurity Incident Index
//!
//! This crate provides the infrastructure for crawling public sources (IDX disclosures,
//! BSSN publications, OJK reports, media coverage) and extracting structured incident
//! data from unstructured web content.

pub mod extractors;
pub mod rate_limiter;
pub mod scheduler;
pub mod sources;

pub use extractors::*;
pub use rate_limiter::*;
pub use scheduler::*;
pub use sources::*;

/// Re-export commonly used types
pub mod prelude {
    pub use crate::extractors::{DataExtractor, ExtractionResult};
    pub use crate::rate_limiter::RateLimiter;
    pub use crate::scheduler::CrawlerScheduler;
    pub use crate::sources::{CrawlerSource, SourceConfig};
}
