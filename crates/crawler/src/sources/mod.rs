//! Crawler source implementations

pub mod bssn;
pub mod bssn_tests;
pub mod idx;
pub mod media;
pub mod ojk;
pub mod ojk_tests;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceConfig {
    pub name: String,
    pub base_url: String,
    pub rate_limit: std::time::Duration,
    pub enabled: bool,
}

#[async_trait]
pub trait CrawlerSource: Send + Sync {
    fn name(&self) -> &str;
    fn config(&self) -> &SourceConfig;
    async fn crawl(&self) -> Result<Vec<crate::extractors::ExtractionResult>, anyhow::Error>;
}
