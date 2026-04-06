//! BSSN (National Cyber and Crypto Agency) crawler source

use crate::extractors::ExtractionResult;
use crate::sources::{CrawlerSource, SourceConfig};
use async_trait::async_trait;

pub struct BssnCrawler {
    config: SourceConfig,
}

impl BssnCrawler {
    pub fn new() -> Self {
        Self {
            config: SourceConfig {
                name: "BSSN".to_string(),
                base_url: "https://bssn.go.id".to_string(),
                rate_limit: std::time::Duration::from_secs(1),
                enabled: true,
            },
        }
    }
}

#[async_trait]
impl CrawlerSource for BssnCrawler {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn config(&self) -> &SourceConfig {
        &self.config
    }

    async fn crawl(&self) -> Result<Vec<ExtractionResult>, anyhow::Error> {
        // TODO: Implement BSSN crawler
        Ok(vec![])
    }
}
