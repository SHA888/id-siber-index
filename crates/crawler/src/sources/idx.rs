//! IDX (Indonesia Stock Exchange) crawler source

use crate::extractors::ExtractionResult;
use crate::sources::{CrawlerSource, SourceConfig};
use async_trait::async_trait;

pub struct IdxCrawler {
    config: SourceConfig,
}

impl IdxCrawler {
    pub fn new() -> Self {
        Self {
            config: SourceConfig {
                name: "IDX".to_string(),
                base_url: "https://www.idx.co.id".to_string(),
                rate_limit: std::time::Duration::from_secs(1),
                enabled: true,
            },
        }
    }
}

#[async_trait]
impl CrawlerSource for IdxCrawler {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn config(&self) -> &SourceConfig {
        &self.config
    }

    async fn crawl(&self) -> Result<Vec<ExtractionResult>, anyhow::Error> {
        // TODO: Implement IDX crawler
        Ok(vec![])
    }
}
