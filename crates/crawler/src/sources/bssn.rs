//! BSSN (National Cyber and Crypto Agency) crawler source

use crate::extractors::ExtractionResult;
use crate::incident_draft::IncidentDraft;
use crate::rate_limiter::RateLimiter;
use crate::sources::{CrawlerSource, SourceConfig};
use async_trait::async_trait;
use chrono::{NaiveDate, Utc};
use regex::Regex;
use reqwest::Client;
use scraper::{Html, Selector};
use std::collections::HashMap;
use std::sync::LazyLock;
use std::time::Duration;
use url::Url;

// Constants for limits and validation
const MAX_ITEMS_PER_PAGE: usize = 100;
const RATE_LIMIT_TIMEOUT: Duration = Duration::from_secs(30);
const MIN_ORG_NAME_LENGTH: usize = 5;

// Static Indonesian months mapping for performance
static INDONESIAN_MONTHS: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    HashMap::from([
        ("Januari", "January"),
        ("Februari", "February"),
        ("Maret", "March"),
        ("April", "April"),
        ("Mei", "May"),
        ("Juni", "June"),
        ("Juli", "July"),
        ("Agustus", "August"),
        ("September", "September"),
        ("Oktober", "October"),
        ("November", "November"),
        ("Desember", "December"),
    ])
});

/// BSSN crawler for press releases and threat landscape reports
pub struct BssnCrawler {
    config: SourceConfig,
    client: Client,
    rate_limiter: RateLimiter,
    keywords: BssnKeywordMatcher,
    // Pre-compiled regexes for performance
    org_name_regex: Regex,
    date_regexes: Vec<Regex>,
    // Pre-compiled selectors
    press_release_selectors: Vec<Selector>,
    link_selector: Selector,
    title_selector: Selector,
    base_url: Url,
}

/// BSSN-specific keyword matcher for cyber incidents
pub struct BssnKeywordMatcher {
    bahasa_keywords: Vec<String>,
    english_keywords: Vec<String>,
    attack_type_mapping: HashMap<String, String>,
    sector_keywords: HashMap<String, String>,
}

impl Default for BssnKeywordMatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl BssnKeywordMatcher {
    pub fn new() -> Self {
        let bahasa_keywords = vec![
            "serangan siber".to_string(),
            "kebocoran data".to_string(),
            "ransomware".to_string(),
            "gangguan sistem".to_string(),
            "insiden keamanan".to_string(),
            "insiden siber".to_string(),
            "serangan hacker".to_string(),
            "malware".to_string(),
            "phishing".to_string(),
            "deface".to_string(),
            "ddos".to_string(),
            "sql injection".to_string(),
            "pencurian data".to_string(),
            "akun diretas".to_string(),
            "sistem diretas".to_string(),
            "ancaman siber".to_string(),
            "kerentanan keamanan".to_string(),
            "serangan cyber".to_string(),
        ]
        .iter()
        .map(|s| s.to_lowercase())
        .collect();

        let english_keywords = vec![
            "cyber attack".to_string(),
            "data breach".to_string(),
            "system disruption".to_string(),
            "unauthorized access".to_string(),
            "ransomware".to_string(),
            "malware".to_string(),
            "phishing".to_string(),
            "ddos".to_string(),
            "hacking".to_string(),
            "data leak".to_string(),
            "security incident".to_string(),
            "cybersecurity".to_string(),
            "data theft".to_string(),
            "account compromised".to_string(),
            "system compromised".to_string(),
        ]
        .iter()
        .map(|s| s.to_lowercase())
        .collect();

        let mut attack_type_mapping = HashMap::new();
        attack_type_mapping.insert("ransomware".to_lowercase(), "RANSOMWARE".to_string());
        attack_type_mapping.insert("malware".to_lowercase(), "MALWARE".to_string());
        attack_type_mapping.insert("phishing".to_lowercase(), "PHISHING".to_string());
        attack_type_mapping.insert("ddos".to_lowercase(), "DDOS".to_string());
        attack_type_mapping.insert("sql injection".to_lowercase(), "SQL_INJECTION".to_string());
        attack_type_mapping.insert("deface".to_lowercase(), "WEBSITE_DEFACEMENT".to_string());
        attack_type_mapping.insert("serangan siber".to_lowercase(), "CYBER_ATTACK".to_string());
        attack_type_mapping.insert("kebocoran data".to_lowercase(), "DATA_BREACH".to_string());
        attack_type_mapping.insert("pencurian data".to_lowercase(), "DATA_THEFT".to_string());
        attack_type_mapping.insert(
            "akun diretas".to_lowercase(),
            "ACCOUNT_COMPROMISE".to_string(),
        );

        // English mappings
        attack_type_mapping.insert("cyber attack".to_lowercase(), "CYBER_ATTACK".to_string());
        attack_type_mapping.insert("data breach".to_lowercase(), "DATA_BREACH".to_string());
        attack_type_mapping.insert("data leak".to_lowercase(), "DATA_BREACH".to_string());
        attack_type_mapping.insert("security breach".to_lowercase(), "DATA_BREACH".to_string());
        attack_type_mapping.insert("hacking".to_lowercase(), "HACKING".to_string());
        attack_type_mapping.insert("data theft".to_lowercase(), "DATA_THEFT".to_string());
        attack_type_mapping.insert(
            "account compromised".to_lowercase(),
            "ACCOUNT_COMPROMISE".to_string(),
        );
        attack_type_mapping.insert(
            "system compromised".to_lowercase(),
            "SYSTEM_COMPROMISE".to_string(),
        );

        let mut sector_keywords = HashMap::new();
        sector_keywords.insert("perbankan".to_lowercase(), "FINANCIAL".to_string());
        sector_keywords.insert("bank".to_lowercase(), "FINANCIAL".to_string());
        sector_keywords.insert("keuangan".to_lowercase(), "FINANCIAL".to_string());
        sector_keywords.insert("kesehatan".to_lowercase(), "HEALTHCARE".to_string());
        sector_keywords.insert("rumah sakit".to_lowercase(), "HEALTHCARE".to_string());
        sector_keywords.insert("klinik".to_lowercase(), "HEALTHCARE".to_string());
        sector_keywords.insert("pemerintahan".to_lowercase(), "GOVERNMENT".to_string());
        sector_keywords.insert("pemerintah".to_lowercase(), "GOVERNMENT".to_string());
        sector_keywords.insert("instansi".to_lowercase(), "GOVERNMENT".to_string());
        sector_keywords.insert("pendidikan".to_lowercase(), "EDUCATION".to_string());
        sector_keywords.insert("universitas".to_lowercase(), "EDUCATION".to_string());
        sector_keywords.insert("sekolah".to_lowercase(), "EDUCATION".to_string());
        sector_keywords.insert(
            "telekomunikasi".to_lowercase(),
            "TELECOMMUNICATIONS".to_string(),
        );
        sector_keywords.insert("telkom".to_lowercase(), "TELECOMMUNICATIONS".to_string());
        sector_keywords.insert("indosat".to_lowercase(), "TELECOMMUNICATIONS".to_string());
        sector_keywords.insert("xl".to_lowercase(), "TELECOMMUNICATIONS".to_string());
        sector_keywords.insert("e-commerce".to_lowercase(), "E-COMMERCE".to_string());
        sector_keywords.insert("tokopedia".to_lowercase(), "E-COMMERCE".to_string());
        sector_keywords.insert("shopee".to_lowercase(), "E-COMMERCE".to_string());
        sector_keywords.insert("bukalapak".to_lowercase(), "E-COMMERCE".to_string());
        sector_keywords.insert("energi".to_lowercase(), "ENERGY".to_string());
        sector_keywords.insert("pln".to_lowercase(), "ENERGY".to_string());
        sector_keywords.insert("pertamina".to_lowercase(), "ENERGY".to_string());

        Self {
            bahasa_keywords,
            english_keywords,
            attack_type_mapping,
            sector_keywords,
        }
    }

    /// Check if text contains cyber incident keywords
    pub fn contains_cyber_keywords(&self, text: &str) -> bool {
        let text_lower = text.to_lowercase();
        for keyword in &self.bahasa_keywords {
            if text_lower.contains(keyword) {
                return true;
            }
        }
        for keyword in &self.english_keywords {
            if text_lower.contains(keyword) {
                return true;
            }
        }
        false
    }

    /// Extract attack type from text
    pub fn extract_attack_type(&self, text: &str) -> Option<String> {
        let text_lower = text.to_lowercase();
        for (keyword, attack_type) in &self.attack_type_mapping {
            if text_lower.contains(keyword) {
                return Some(attack_type.clone());
            }
        }
        if text_lower.contains("insiden keamanan") || text_lower.contains("security incident") {
            return Some("UNKNOWN".to_string());
        }
        None
    }

    /// Extract sector from text
    pub fn extract_sector(&self, text: &str) -> Option<String> {
        let text_lower = text.to_lowercase();
        for (keyword, sector) in &self.sector_keywords {
            if text_lower.contains(keyword) {
                return Some(sector.clone());
            }
        }
        None
    }
}

impl BssnCrawler {
    pub fn new() -> Result<Self, anyhow::Error> {
        let config = SourceConfig {
            name: "BSSN".to_string(),
            base_url: "https://bssn.go.id".to_string(),
            rate_limit: Duration::from_secs(2),
            enabled: true,
        };

        let base_url =
            Url::parse(&config.base_url).map_err(|e| anyhow::anyhow!("Invalid base URL: {}", e))?;

        let org_name_regex = Regex::new(
            r"([A-Z][a-zA-Z\s&\.\-\(\)]+(?:PT|Tbk|CV|FA|Persero|Inc\.|Corp\.)?|[A-Z][a-zA-Z\s]+)",
        )
        .map_err(|e| anyhow::anyhow!("Failed to compile org name regex: {}", e))?;

        let date_regexes = vec![
            Regex::new(r"(\d{1,2})\s+([A-Za-z]+)\s+(\d{4})")
                .map_err(|e| anyhow::anyhow!("Failed to compile date regex 1: {}", e))?,
            Regex::new(r"(\d{4}-\d{2}-\d{2})")
                .map_err(|e| anyhow::anyhow!("Failed to compile date regex 2: {}", e))?,
            Regex::new(r"(\d{2}/\d{2}/\d{4})")
                .map_err(|e| anyhow::anyhow!("Failed to compile date regex 3: {}", e))?,
        ];

        let press_release_selectors = vec![
            Selector::parse(".news-item")
                .map_err(|e| anyhow::anyhow!("Invalid selector '.news-item': {}", e))?,
            Selector::parse(".press-release-item")
                .map_err(|e| anyhow::anyhow!("Invalid selector '.press-release-item': {}", e))?,
            Selector::parse(".article-item")
                .map_err(|e| anyhow::anyhow!("Invalid selector '.article-item': {}", e))?,
            Selector::parse(".siaran-pers-item")
                .map_err(|e| anyhow::anyhow!("Invalid selector '.siaran-pers-item': {}", e))?,
            Selector::parse("div[class*='item']")
                .map_err(|e| anyhow::anyhow!("Invalid selector 'div[class*='item']': {}", e))?,
        ];

        let link_selector =
            Selector::parse("a").map_err(|e| anyhow::anyhow!("Invalid selector 'a': {}", e))?;

        let title_selector = Selector::parse("h2, h3, h4, .title")
            .map_err(|e| anyhow::anyhow!("Invalid selector 'h2, h3, h4, .title': {}", e))?;

        Ok(Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .user_agent("ID-Siber-Index-Crawler/1.0")
                .build()
                .map_err(|e| anyhow::anyhow!("Failed to create HTTP client: {}", e))?,
            rate_limiter: RateLimiter::new(1),
            keywords: BssnKeywordMatcher::new(),
            config,
            org_name_regex,
            date_regexes,
            press_release_selectors,
            link_selector,
            title_selector,
            base_url,
        })
    }

    /// Main crawl method - fetches both press releases and threat reports
    async fn crawl_internal(&self) -> Result<Vec<ExtractionResult>, anyhow::Error> {
        let mut all_results = Vec::new();

        // Crawl press releases
        if let Ok(press_results) = self.crawl_press_releases().await {
            all_results.extend(press_results);
        } else {
            eprintln!("Warning: Failed to crawl BSSN press releases");
        }

        // Crawl threat landscape reports (PDFs)
        if let Ok(pdf_results) = self.crawl_threat_reports().await {
            all_results.extend(pdf_results);
        } else {
            eprintln!("Warning: Failed to crawl BSSN threat reports");
        }

        // Remove duplicates
        let unique_results = self.remove_duplicates(all_results);

        Ok(unique_results)
    }

    /// Crawl BSSN press releases
    async fn crawl_press_releases(&self) -> Result<Vec<ExtractionResult>, anyhow::Error> {
        let mut results = Vec::new();

        let endpoints = vec![
            "https://bssn.go.id/siaran-pers",
            "https://bssn.go.id/berita",
            "https://bssn.go.id/news",
        ];

        for endpoint in endpoints {
            tokio::time::timeout(RATE_LIMIT_TIMEOUT, self.rate_limiter.acquire())
                .await
                .map_err(|_| anyhow::anyhow!("Rate limiter timeout"))??;

            match self.client.get(endpoint).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        let content = response
                            .text()
                            .await
                            .map_err(|e| anyhow::anyhow!("Failed to read response: {}", e))?;

                        let items = self.parse_press_releases(&content)?;
                        let filtered = self.filter_cyber_incidents(items);
                        let converted = self.convert_to_extraction_results(filtered);
                        results.extend(converted);

                        if !results.is_empty() {
                            break;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to fetch {}: {}", endpoint, e);
                }
            }
        }

        Ok(results)
    }

    /// Parse press release items from HTML
    pub fn parse_press_releases(
        &self,
        html_content: &str,
    ) -> Result<Vec<PressReleaseItem>, anyhow::Error> {
        let document = Html::parse_document(html_content);
        let mut items = Vec::new();
        let mut total_items = 0;

        for selector in &self.press_release_selectors {
            for element in document.select(selector) {
                total_items += 1;
                if total_items > MAX_ITEMS_PER_PAGE {
                    eprintln!("Warning: Reached maximum items limit");
                    break;
                }

                if let Ok(Some(item)) = self.parse_press_release_item(&element) {
                    items.push(item);
                }
            }

            if !items.is_empty() {
                break;
            }
        }

        Ok(items)
    }

    /// Parse individual press release item
    fn parse_press_release_item(
        &self,
        element: &scraper::ElementRef,
    ) -> Result<Option<PressReleaseItem>, anyhow::Error> {
        let text_content = element.text().collect::<String>();

        // Extract title
        let title = self.extract_title(element);

        // Extract date
        let publication_date = self.extract_date(&text_content);

        // Extract URL
        let source_url = self.extract_url(element);

        // Extract organization name if present
        let org_name = self.extract_org_name(&text_content);

        // Extract sector
        let sector = self.keywords.extract_sector(&text_content);

        if source_url.is_empty() {
            return Ok(None);
        }

        Ok(Some(PressReleaseItem {
            title,
            publication_date,
            source_url,
            org_name,
            sector,
            raw_content: text_content,
        }))
    }

    /// Extract title from element
    fn extract_title(&self, element: &scraper::ElementRef) -> String {
        if let Some(title_elem) = element.select(&self.title_selector).next() {
            let title = title_elem.text().collect::<String>();
            if !title.trim().is_empty() {
                return title.trim().to_string();
            }
        }
        String::new()
    }

    /// Extract date from text
    pub fn extract_date(&self, text: &str) -> NaiveDate {
        for regex in &self.date_regexes {
            if let Some(caps) = regex.captures(text) {
                let date_str = caps.get(0).map(|m| m.as_str()).unwrap_or("");

                let mut normalized_date = date_str.to_string();
                for (id, en) in INDONESIAN_MONTHS.iter() {
                    normalized_date = normalized_date.replace(id, en);
                }

                let possible_formats = vec!["%d %B %Y", "%Y-%m-%d", "%d/%m/%Y"];
                for format in possible_formats {
                    if let Ok(date) = NaiveDate::parse_from_str(&normalized_date, format) {
                        let today = Utc::now().date_naive();
                        let min_date = NaiveDate::from_ymd_opt(2020, 1, 1);

                        if let Some(min) = min_date
                            && date >= min
                            && date <= today
                        {
                            return date;
                        }
                    }
                }
            }
        }

        Utc::now().date_naive()
    }

    /// Extract URL from element
    fn extract_url(&self, element: &scraper::ElementRef) -> String {
        if let Some(link) = element.select(&self.link_selector).next()
            && let Some(href) = link.value().attr("href")
        {
            let url = if href.starts_with("http") {
                href.to_string()
            } else if href.starts_with("//") {
                // Protocol-relative URL
                format!("{}:{}", self.base_url.scheme(), href)
            } else {
                self.base_url
                    .join(href)
                    .map(|u| u.to_string())
                    .unwrap_or_default()
            };
            return url;
        }
        String::new()
    }

    /// Extract organization name from text
    pub fn extract_org_name(&self, text: &str) -> Option<String> {
        if let Some(caps) = self.org_name_regex.captures(text) {
            let mut org_name = caps.get(1)?.as_str().trim().to_string();

            // Truncate at common sentence endings or action words
            let truncate_words = [
                "mengalami",
                "mengumumkan",
                "melaporkan",
                "menginformasikan",
                "menyatakan",
                "terjadi",
                "diretas",
                "serangan",
                "keamanan",
            ];
            for word in &truncate_words {
                let word_lower = word.to_lowercase();
                let org_lower = org_name.to_lowercase();

                // Check for word with space before it (word boundary)
                if let Some(pos) = org_lower.find(&format!(" {}", word_lower)) {
                    org_name = org_name[..pos].trim().to_string();
                    break;
                }
                // Also check for word at end of string
                if org_lower.ends_with(&word_lower)
                    && let Some(pos) = org_lower.rfind(word_lower.as_str())
                {
                    org_name = org_name[..pos].trim().to_string();
                    break;
                }
            }

            if self.is_valid_org_name(&org_name) {
                return Some(org_name);
            }
        }
        None
    }

    /// Validate organization name
    pub fn is_valid_org_name(&self, name: &str) -> bool {
        let trimmed = name.trim();
        if trimmed.len() < MIN_ORG_NAME_LENGTH {
            return false;
        }
        if !trimmed.chars().any(|c| c.is_alphabetic()) {
            return false;
        }
        if trimmed.chars().all(|c| c.is_numeric()) {
            return false;
        }
        true
    }

    /// Filter press releases for cyber incidents
    fn filter_cyber_incidents(&self, items: Vec<PressReleaseItem>) -> Vec<PressReleaseItem> {
        items
            .into_iter()
            .filter(|item| {
                self.keywords.contains_cyber_keywords(&item.title)
                    || self.keywords.contains_cyber_keywords(&item.raw_content)
            })
            .collect()
    }

    /// Convert press release items to extraction results
    fn convert_to_extraction_results(&self, items: Vec<PressReleaseItem>) -> Vec<ExtractionResult> {
        items
            .into_iter()
            .map(|item| {
                let attack_type = self.keywords.extract_attack_type(&item.title);
                let sector = item
                    .sector
                    .clone()
                    .or_else(|| self.keywords.extract_sector(&item.raw_content));

                let draft = IncidentDraft::new(
                    item.org_name
                        .clone()
                        .unwrap_or_else(|| "BSSN Report".to_string()),
                    item.publication_date,
                    item.source_url,
                    "BSSN_PRESS_RELEASE".to_string(),
                )
                .with_org_sector(sector)
                .with_attack_type(attack_type)
                .with_confidence(if item.org_name.is_some() { 0.8 } else { 0.6 })
                .with_raw_content(Some(item.raw_content))
                .with_notes(Some(item.title));

                draft.to_extraction_result()
            })
            .collect()
    }

    /// Crawl BSSN threat landscape reports (PDFs)
    async fn crawl_threat_reports(&self) -> Result<Vec<ExtractionResult>, anyhow::Error> {
        let mut results = Vec::new();

        // BSSN threat landscape report URLs (known annual reports)
        let report_urls = vec![
            "https://bssn.go.id",
            // Add specific PDF URLs when available
        ];

        for url in report_urls {
            tokio::time::timeout(RATE_LIMIT_TIMEOUT, self.rate_limiter.acquire())
                .await
                .map_err(|_| anyhow::anyhow!("Rate limiter timeout"))??;

            match self.client.get(url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        let content = response
                            .text()
                            .await
                            .map_err(|e| anyhow::anyhow!("Failed to read response: {}", e))?;

                        // Look for PDF links in the page
                        let pdf_links = self.extract_pdf_links(&content);

                        for pdf_url in pdf_links {
                            if let Ok(pdf_result) = self.parse_pdf_report(&pdf_url).await {
                                results.push(pdf_result);
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to fetch {}: {}", url, e);
                }
            }
        }

        Ok(results)
    }

    /// Extract PDF links from HTML content
    pub fn extract_pdf_links(&self, html_content: &str) -> Vec<String> {
        let document = Html::parse_document(html_content);
        let mut pdf_urls = Vec::new();

        // Regex to match .pdf extensions with or without query parameters
        let pdf_regex = Regex::new(r"\.pdf(\?|$)").unwrap();

        for element in document.select(&self.link_selector) {
            if let Some(href) = element.value().attr("href")
                && pdf_regex.is_match(href)
            {
                let url = if href.starts_with("http") {
                    href.to_string()
                } else if href.starts_with("//") {
                    format!("{}:{}", self.base_url.scheme(), href)
                } else {
                    self.base_url
                        .join(href)
                        .map(|u| u.to_string())
                        .unwrap_or_default()
                };
                if !url.is_empty() {
                    pdf_urls.push(url);
                }
            }
        }

        pdf_urls
    }

    /// Parse PDF report and extract incident information
    async fn parse_pdf_report(&self, pdf_url: &str) -> Result<ExtractionResult, anyhow::Error> {
        tokio::time::timeout(RATE_LIMIT_TIMEOUT, self.rate_limiter.acquire())
            .await
            .map_err(|_| anyhow::anyhow!("Rate limiter timeout"))??;

        let response = self
            .client
            .get(pdf_url)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to fetch PDF: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "PDF request failed with status: {}",
                response.status()
            ));
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to download PDF: {}", e))?;

        // Extract text from PDF
        let text = pdf_extract::extract_text_from_mem(&bytes)
            .map_err(|e| anyhow::anyhow!("Failed to extract PDF text: {}", e))?;

        // Check if PDF contains cyber incident information
        if !self.keywords.contains_cyber_keywords(&text) {
            return Err(anyhow::anyhow!(
                "PDF does not contain cyber incident keywords"
            ));
        }

        // Extract information from PDF text
        let org_name = self
            .extract_org_name(&text)
            .unwrap_or_else(|| "BSSN Threat Report".to_string());
        let attack_type = self.keywords.extract_attack_type(&text);
        let sector = self.keywords.extract_sector(&text);

        // Try to extract date from PDF
        let publication_date = self.extract_date(&text);

        let draft = IncidentDraft::new(
            org_name,
            publication_date,
            pdf_url.to_string(),
            "BSSN_THREAT_REPORT".to_string(),
        )
        .with_org_sector(sector)
        .with_attack_type(attack_type)
        .with_confidence(0.7) // PDF extraction has lower confidence
        .with_raw_content(Some(text));

        Ok(draft.to_extraction_result())
    }

    /// Remove duplicates based on organization and date proximity
    fn remove_duplicates(&self, results: Vec<ExtractionResult>) -> Vec<ExtractionResult> {
        let mut unique_results = Vec::new();

        for result in results {
            let is_duplicate = unique_results.iter().any(|existing: &ExtractionResult| {
                result.org_name.to_lowercase() == existing.org_name.to_lowercase()
                    && result
                        .disclosure_date
                        .signed_duration_since(existing.disclosure_date)
                        .num_days()
                        .abs()
                        <= 7
            });

            if !is_duplicate {
                unique_results.push(result);
            }
        }

        unique_results
    }
}

/// Press release item structure
#[derive(Debug, Clone)]
pub struct PressReleaseItem {
    pub title: String,
    pub publication_date: NaiveDate,
    pub source_url: String,
    pub org_name: Option<String>,
    pub sector: Option<String>,
    pub raw_content: String,
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
        self.crawl_internal().await
    }
}

#[cfg(test)]
mod tests {}
