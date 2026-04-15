//! Media outlets crawler source

use crate::extractors::ExtractionResult;
use crate::sources::{CrawlerSource, SourceConfig};
use async_trait::async_trait;

pub struct MediaCrawler {
    config: SourceConfig,
}

impl Default for MediaCrawler {
    fn default() -> Self {
        Self::new()
    }
}

impl MediaCrawler {
    pub fn new() -> Self {
        Self {
            config: SourceConfig {
                name: "Media".to_string(),
                base_url: "https://news.google.com".to_string(),
                rate_limit: std::time::Duration::from_secs(1),
                enabled: true,
            },
        }
    }
}

#[async_trait]
impl CrawlerSource for MediaCrawler {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn config(&self) -> &SourceConfig {
        &self.config
    }

    async fn crawl(&self) -> Result<Vec<ExtractionResult>, anyhow::Error> {
        // TODO: Implement media crawler
        Ok(vec![])
    }
}
