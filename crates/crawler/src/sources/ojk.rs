//! OJK (Financial Services Authority) crawler source

use crate::extractors::ExtractionResult;
use crate::sources::{CrawlerSource, SourceConfig};
use async_trait::async_trait;

pub struct OjkCrawler {
    config: SourceConfig,
}

impl OjkCrawler {
    pub fn new() -> Self {
        Self {
            config: SourceConfig {
                name: "OJK".to_string(),
                base_url: "https://ojk.go.id".to_string(),
                rate_limit: std::time::Duration::from_secs(1),
                enabled: true,
            },
        }
    }
}

#[async_trait]
impl CrawlerSource for OjkCrawler {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn config(&self) -> &SourceConfig {
        &self.config
    }

    async fn crawl(&self) -> Result<Vec<ExtractionResult>, anyhow::Error> {
        // TODO: Implement OJK crawler
        Ok(vec![])
    }
}
