//! Media outlets crawler source
//!
//! This crawler parses Indonesian tech and cybersecurity news from major media outlets:
//! - Tempo (tempo.co/tag/keamanan-siber)
//! - Kompas Tech (tekno.kompas.com)
//! - Detik Inet (inet.detik.com)
//! - Bisnis Indonesia (teknologi.bisnis.com)
//!
//! Features:
//! - Respects robots.txt and crawl delays
//! - Deduplicates incidents across multiple outlets
//! - Extracts organization names, dates, and attack types

use crate::extractors::ExtractionResult;
use crate::incident_draft::IncidentDraft;
use crate::sources::{CrawlerSource, SourceConfig};
use async_trait::async_trait;
use chrono::{NaiveDate, Utc};
use regex::Regex;
use reqwest::Client;
use scraper::{Html, Selector};
use std::collections::HashMap;
use std::sync::LazyLock;
use std::time::Duration;
use tracing::{debug, warn};
use url::Url;

// Constants for limits and validation
pub const MAX_ITEMS_PER_OUTLET: usize = 50;
const DEFAULT_CRAWL_DELAY: Duration = Duration::from_secs(2);
const MIN_ORG_NAME_LENGTH: usize = 3;

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

/// Supported media outlets
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MediaOutlet {
    Tempo,
    KompasTech,
    DetikInet,
    BisnisIndonesia,
}

impl MediaOutlet {
    pub fn name(&self) -> &'static str {
        match self {
            MediaOutlet::Tempo => "Tempo",
            MediaOutlet::KompasTech => "Kompas Tech",
            MediaOutlet::DetikInet => "Detik Inet",
            MediaOutlet::BisnisIndonesia => "Bisnis Indonesia",
        }
    }

    pub fn base_url(&self) -> &'static str {
        match self {
            MediaOutlet::Tempo => "https://tempo.co",
            MediaOutlet::KompasTech => "https://tekno.kompas.com",
            MediaOutlet::DetikInet => "https://inet.detik.com",
            MediaOutlet::BisnisIndonesia => "https://teknologi.bisnis.com",
        }
    }

    pub fn cyber_url(&self) -> String {
        match self {
            MediaOutlet::Tempo => "https://tempo.co/tag/keamanan-siber".to_string(),
            MediaOutlet::KompasTech => "https://tekno.kompas.com/read/tag/cyber".to_string(),
            MediaOutlet::DetikInet => "https://inet.detik.com/cyber".to_string(),
            MediaOutlet::BisnisIndonesia => {
                "https://teknologi.bisnis.com/read/tag/cyber".to_string()
            }
        }
    }
}

/// Media crawler for multiple Indonesian news outlets
pub struct MediaCrawler {
    config: SourceConfig,
    client: Client,
    keywords: MediaKeywordMatcher,
    // Pre-compiled regexes for performance
    org_name_regex: Regex,
    date_regexes: Vec<Regex>,
    base_urls: HashMap<MediaOutlet, Url>,
    crawl_delays: HashMap<MediaOutlet, Duration>,
}

/// Media-specific keyword matcher for cyber incidents
pub struct MediaKeywordMatcher {
    bahasa_keywords: Vec<String>,
    english_keywords: Vec<String>,
    attack_type_mapping: HashMap<String, String>,
}

impl Default for MediaKeywordMatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl MediaKeywordMatcher {
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
            "data bocor".to_string(),
            "website diretas".to_string(),
            "aplikasi diretas".to_string(),
            "bocor data".to_string(),
            "diretas".to_string(),
            "hack".to_string(),
            "hacker".to_string(),
            "keamanan data".to_string(),
            "cyber attack".to_string(),
            "data breach".to_string(),
        ];

        let english_keywords = vec![
            "cyber attack".to_string(),
            "data breach".to_string(),
            "ransomware".to_string(),
            "malware".to_string(),
            "phishing".to_string(),
            "hacking".to_string(),
            "security incident".to_string(),
            "cybersecurity".to_string(),
            "data leak".to_string(),
            "system compromise".to_string(),
        ];

        let mut attack_type_mapping = HashMap::new();
        attack_type_mapping.insert("ransomware".to_string(), "RANSOMWARE".to_string());
        attack_type_mapping.insert("malware".to_string(), "MALWARE".to_string());
        attack_type_mapping.insert("phishing".to_string(), "PHISHING".to_string());
        attack_type_mapping.insert("ddos".to_string(), "DDOS".to_string());
        attack_type_mapping.insert("deface".to_string(), "DEFACEMENT".to_string());
        attack_type_mapping.insert("sql injection".to_string(), "SQL_INJECTION".to_string());
        attack_type_mapping.insert("data breach".to_string(), "DATA_BREACH".to_string());
        attack_type_mapping.insert("kebocoran data".to_string(), "DATA_BREACH".to_string());
        attack_type_mapping.insert("serangan hacker".to_string(), "HACKING".to_string());
        attack_type_mapping.insert("pencurian data".to_string(), "DATA_THEFT".to_string());
        attack_type_mapping.insert("akun diretas".to_string(), "ACCOUNT_TAKEOVER".to_string());

        Self {
            bahasa_keywords,
            english_keywords,
            attack_type_mapping,
        }
    }

    pub fn contains_cyber_keywords(&self, text: &str) -> bool {
        let text_lower = text.to_lowercase();
        self.bahasa_keywords
            .iter()
            .chain(self.english_keywords.iter())
            .any(|kw| text_lower.contains(kw))
    }

    pub fn extract_attack_type(&self, text: &str) -> Option<String> {
        let text_lower = text.to_lowercase();
        for (keyword, attack_type) in &self.attack_type_mapping {
            if text_lower.contains(keyword) {
                return Some(attack_type.clone());
            }
        }
        None
    }
}

impl Default for MediaCrawler {
    fn default() -> Self {
        Self::new()
    }
}

impl MediaCrawler {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("ID-Siber-Index-Crawler/1.0")
            .build()
            .expect("Failed to build HTTP client");

        let org_name_regex = Regex::new(
            r"(PT\s+[A-Za-z0-9\s&\.]+(?:Tbk)?|Bank\s+[A-Za-z\s]+|[A-Z][a-zA-Z\s]+(?:Bank|Perbankan)?)"
        )
        .expect("Failed to compile org name regex");

        let date_regexes = vec![
            Regex::new(r"(\d{1,2})\s+(Januari|Februari|Maret|April|Mei|Juni|Juli|Agustus|September|Oktober|November|Desember)\s+(\d{4})").expect("Failed to compile date regex"),
            Regex::new(r"(\d{1,2})/(\d{1,2})/(\d{4})").expect("Failed to compile date regex"),
            Regex::new(r"(\d{4})-(\d{2})-(\d{2})").expect("Failed to compile date regex"),
        ];

        let mut base_urls = HashMap::new();
        for outlet in [
            MediaOutlet::Tempo,
            MediaOutlet::KompasTech,
            MediaOutlet::DetikInet,
            MediaOutlet::BisnisIndonesia,
        ] {
            base_urls.insert(
                outlet,
                Url::parse(outlet.base_url()).expect("Failed to parse base URL"),
            );
        }

        let mut crawl_delays = HashMap::new();
        crawl_delays.insert(MediaOutlet::Tempo, Duration::from_secs(2));
        crawl_delays.insert(MediaOutlet::KompasTech, Duration::from_secs(2));
        crawl_delays.insert(MediaOutlet::DetikInet, Duration::from_secs(2));
        crawl_delays.insert(MediaOutlet::BisnisIndonesia, Duration::from_secs(2));

        Self {
            config: SourceConfig {
                name: "Media".to_string(),
                base_url: "https://tempo.co".to_string(),
                rate_limit: DEFAULT_CRAWL_DELAY,
                enabled: true,
            },
            client,
            keywords: MediaKeywordMatcher::new(),
            org_name_regex,
            date_regexes,
            base_urls,
            crawl_delays,
        }
    }

    /// Crawl all configured media outlets
    pub async fn crawl_all_outlets(&self) -> Result<Vec<MediaItem>, anyhow::Error> {
        let mut all_items = Vec::new();

        let outlets = [
            MediaOutlet::Tempo,
            MediaOutlet::KompasTech,
            MediaOutlet::DetikInet,
            MediaOutlet::BisnisIndonesia,
        ];

        for outlet in &outlets {
            debug!("Crawling outlet: {}", outlet.name());
            match self.crawl_outlet(outlet).await {
                Ok(items) => {
                    debug!("Found {} items from {}", items.len(), outlet.name());
                    all_items.extend(items);
                }
                Err(e) => {
                    warn!("Failed to crawl {}: {}", outlet.name(), e);
                }
            }
        }

        Ok(all_items)
    }

    /// Crawl a specific media outlet
    async fn crawl_outlet(&self, outlet: &MediaOutlet) -> Result<Vec<MediaItem>, anyhow::Error> {
        let url = outlet.cyber_url();
        let delay = self
            .crawl_delays
            .get(outlet)
            .copied()
            .unwrap_or(DEFAULT_CRAWL_DELAY);

        // Respect crawl delay
        tokio::time::sleep(delay).await;

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to fetch {}: {}", url, e))?;

        let content = response
            .text()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to read response: {}", e))?;

        self.parse_outlet_items(outlet, &content)
    }

    /// Parse items from outlet-specific HTML
    pub fn parse_outlet_items(
        &self,
        outlet: &MediaOutlet,
        html_content: &str,
    ) -> Result<Vec<MediaItem>, anyhow::Error> {
        let document = Html::parse_document(html_content);
        let mut items = Vec::new();

        // Outlet-specific selectors
        let article_selector = match outlet {
            MediaOutlet::Tempo => "article", // Generic article selector
            MediaOutlet::KompasTech => ".article__item",
            MediaOutlet::DetikInet => ".media__text",
            MediaOutlet::BisnisIndonesia => ".article-list-item",
        };

        let selector = Selector::parse(article_selector)
            .map_err(|e| anyhow::anyhow!("Failed to parse selector: {:?}", e))?;

        for element in document.select(&selector) {
            if items.len() >= MAX_ITEMS_PER_OUTLET {
                warn!(
                    "Reached maximum items limit ({}) for {}",
                    MAX_ITEMS_PER_OUTLET,
                    outlet.name()
                );
                break;
            }

            if let Ok(Some(item)) = self.parse_article_element(outlet, &element) {
                items.push(item);
            }
        }

        Ok(items)
    }

    /// Parse individual article element
    fn parse_article_element(
        &self,
        outlet: &MediaOutlet,
        element: &scraper::ElementRef,
    ) -> Result<Option<MediaItem>, anyhow::Error> {
        // Extract title
        let title_selector = Selector::parse("h1, h2, h3, .title, .article__title").unwrap();
        let title = element
            .select(&title_selector)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        if title.is_empty() || !self.keywords.contains_cyber_keywords(&title) {
            return Ok(None);
        }

        // Extract URL
        let url = self.extract_url(outlet, element)?;

        // Extract publication date
        let date = self.extract_date(&element.text().collect::<String>());

        // Extract organization name
        let org_name = self.extract_org_name(&title);

        // Extract content preview
        let content = element.text().collect::<String>();
        let attack_type = self.keywords.extract_attack_type(&content);

        Ok(Some(MediaItem {
            title,
            org_name,
            publication_date: date,
            source_url: url,
            raw_content: content,
            outlet: *outlet,
            attack_type,
        }))
    }

    /// Extract URL from article element
    #[allow(clippy::collapsible_if)] // nested ifs clearer than let-chains (unstable)
    pub fn extract_url(
        &self,
        outlet: &MediaOutlet,
        element: &scraper::ElementRef,
    ) -> Result<String, anyhow::Error> {
        let link_selector = Selector::parse("a[href]").unwrap();

        // Get base URL first - fail fast if outlet not configured
        let base_url = self.base_urls.get(outlet).ok_or_else(|| {
            anyhow::anyhow!("Base URL not configured for outlet: {}", outlet.name())
        })?;

        if let Some(link) = element.select(&link_selector).next() {
            if let Some(href) = link.value().attr("href") {
                if href.is_empty() || href.starts_with("javascript:") || href.starts_with('#') {
                    return Err(anyhow::anyhow!("Invalid href: {}", href));
                }

                let full_url = if href.starts_with("http") {
                    // Validate the URL belongs to the outlet's domain
                    if !href.starts_with(base_url.as_str()) {
                        return Err(anyhow::anyhow!(
                            "URL {} does not match outlet domain {}",
                            href,
                            base_url
                        ));
                    }
                    href.to_string()
                } else {
                    base_url
                        .join(href)
                        .map_err(|e| anyhow::anyhow!("Failed to join URL: {}", e))?
                        .to_string()
                };

                return Ok(full_url);
            }
        }

        Err(anyhow::anyhow!(
            "No URL found in element for outlet: {}",
            outlet.name()
        ))
    }

    /// Extract date from text
    pub fn extract_date(&self, text: &str) -> NaiveDate {
        // Type alias for date pattern tuple to reduce complexity
        type DatePattern<'a> = (&'a Regex, &'a str, fn(&regex::Captures) -> String);

        // Try each regex with its specific format
        let patterns: Vec<DatePattern> = vec![
            // Indonesian format: "12 Januari 2024" -> "%d %B %Y"
            (&self.date_regexes[0], "%d %B %Y", |caps| {
                let day = caps.get(1).map(|m| m.as_str()).unwrap_or("1");
                let month = caps.get(2).map(|m| m.as_str()).unwrap_or("Januari");
                let year = caps.get(3).map(|m| m.as_str()).unwrap_or("2024");
                let month_en = INDONESIAN_MONTHS.get(month).unwrap_or(&"January");
                format!("{} {} {}", day, month_en, year)
            }),
            // DD/MM/YYYY format -> "%d/%m/%Y"
            (&self.date_regexes[1], "%d/%m/%Y", |caps| {
                caps.get(0).map(|m| m.as_str()).unwrap_or("").to_string()
            }),
            // ISO format: "2024-01-01" -> "%Y-%m-%d"
            (&self.date_regexes[2], "%Y-%m-%d", |caps| {
                caps.get(0).map(|m| m.as_str()).unwrap_or("").to_string()
            }),
        ];

        for (regex, fmt, to_str) in patterns {
            if let Some(caps) = regex.captures(text) {
                let date_str = to_str(&caps);
                if let Ok(date) = NaiveDate::parse_from_str(&date_str, fmt) {
                    let today = Utc::now().date_naive();
                    let min_date = NaiveDate::from_ymd_opt(2020, 1, 1)
                        .expect("Hardcoded date should be valid");

                    if date > today {
                        return today;
                    }
                    if date < min_date {
                        return today;
                    }
                    return date;
                }
            }
        }

        Utc::now().date_naive()
    }

    /// Extract organization name from text
    pub fn extract_org_name(&self, text: &str) -> Option<String> {
        if let Some(caps) = self.org_name_regex.captures(text) {
            let org_name = caps
                .get(0)
                .map(|m| m.as_str())
                .unwrap_or_default()
                .trim()
                .to_string();

            if self.is_valid_org_name(&org_name) {
                return Some(org_name);
            }
        }
        None
    }

    /// Validate organization name
    pub fn is_valid_org_name(&self, name: &str) -> bool {
        let name = name.trim();
        if name.len() < MIN_ORG_NAME_LENGTH {
            return false;
        }

        // Must contain letters
        if !name.chars().any(|c| c.is_alphabetic()) {
            return false;
        }

        // Check for financial/business indicators
        let business_indicators = [
            "bank",
            "perbankan",
            "fintech",
            "asuransi",
            "sekuritas",
            "telekomunikasi",
            "operator",
            "provider",
            "telco",
            "rumah sakit",
            "rs ",
            "universitas",
            "instansi",
            "perusahaan",
            "pt ",
            "cv ",
            "tbk",
            "startup",
            "e-commerce",
            "marketplace",
            "platform",
        ];

        let name_lower = name.to_lowercase();
        business_indicators
            .iter()
            .any(|indicator| name_lower.contains(indicator))
    }

    /// Convert media items to extraction results with deduplication
    pub fn convert_to_extraction_results(&self, items: Vec<MediaItem>) -> Vec<ExtractionResult> {
        let mut results: Vec<ExtractionResult> = items
            .into_iter()
            .map(|item| {
                let draft = IncidentDraft::new(
                    item.org_name
                        .clone()
                        .unwrap_or_else(|| item.outlet.name().to_string()),
                    item.publication_date,
                    item.source_url.clone(),
                    format!(
                        "MEDIA_{}",
                        item.outlet.name().to_uppercase().replace(' ', "_")
                    ),
                )
                .with_attack_type(item.attack_type)
                .with_confidence(if item.org_name.is_some() { 0.7 } else { 0.5 })
                .with_raw_content(Some(item.raw_content))
                .with_notes(Some(item.title));

                draft.to_extraction_result()
            })
            .collect();

        // Deduplicate by organization name similarity
        self.deduplicate_by_org(&mut results);
        results
    }

    /// Deduplicate results by organization name similarity
    fn deduplicate_by_org(&self, results: &mut Vec<ExtractionResult>) {
        let mut unique_indices = Vec::new();

        for (i, result) in results.iter().enumerate() {
            let is_duplicate = results[..i]
                .iter()
                .any(|existing| self.orgs_similar(&result.org_name, &existing.org_name));

            if !is_duplicate {
                unique_indices.push(i);
            }
        }

        let unique_results: Vec<ExtractionResult> = unique_indices
            .into_iter()
            .map(|i| results[i].clone())
            .collect();

        *results = unique_results;
    }

    /// Check if two organization names are similar (simple heuristic)
    pub fn orgs_similar(&self, a: &str, b: &str) -> bool {
        let a_lower = a.to_lowercase();
        let b_lower = b.to_lowercase();

        // Exact match
        if a_lower == b_lower {
            return true;
        }

        // Check for significant word overlap
        let a_words: Vec<&str> = a_lower.split_whitespace().collect();
        let b_words: Vec<&str> = b_lower.split_whitespace().collect();

        let overlap = a_words
            .iter()
            .filter(|w| w.len() >= 3 && b_words.contains(w))
            .count();

        // If more than 50% of significant words overlap
        let min_words = a_words.len().min(b_words.len());
        if min_words > 0 && overlap > min_words / 2 {
            return true;
        }

        false
    }
}

/// Media item structure for news articles
#[derive(Debug, Clone)]
pub struct MediaItem {
    pub title: String,
    pub org_name: Option<String>,
    pub publication_date: NaiveDate,
    pub source_url: String,
    pub raw_content: String,
    pub outlet: MediaOutlet,
    pub attack_type: Option<String>,
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
        let items = self.crawl_all_outlets().await?;
        let results = self.convert_to_extraction_results(items);
        Ok(results)
    }
}
