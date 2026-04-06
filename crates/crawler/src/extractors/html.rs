//! HTML content extraction using html5ever and readability

use super::{DataExtractor, ExtractionResult};
use anyhow::Result;

pub struct HtmlExtractor;

impl HtmlExtractor {
    pub fn new() -> Self {
        Self
    }
}

impl Default for HtmlExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl DataExtractor for HtmlExtractor {
    fn extract(&self, content: &str, url: &str) -> Result<Vec<ExtractionResult>> {
        // TODO: Implement HTML extraction using html5ever and readability
        // For now, return empty result
        Ok(vec![])
    }
}
