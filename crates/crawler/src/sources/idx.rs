//! IDX (Indonesia Stock Exchange) crawler source

use crate::extractors::ExtractionResult;
use crate::incident_draft::IncidentDraft;
use crate::rate_limiter::RateLimiter;
use crate::sources::{CrawlerSource, SourceConfig};
use async_trait::async_trait;
use chrono::{Datelike, NaiveDate, Utc};
use regex::Regex;
use reqwest::Client;
use scraper::{Html, Selector};
use std::collections::HashMap;
use std::sync::LazyLock;
use std::time::Duration;
use url::Url;

// Constants for limits and validation
const MAX_ITEMS_PER_PAGE: usize = 1000;
const MAX_PAGES: usize = 100;
const RATE_LIMIT_TIMEOUT: Duration = Duration::from_secs(30);
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

/// IDX electronic disclosure feed crawler
pub struct IdxCrawler {
    config: SourceConfig,
    client: Client,
    rate_limiter: RateLimiter,
    keywords: KeywordMatcher,
    // Pre-compiled regexes for performance
    org_name_regex: Regex,
    date_regexes: Vec<Regex>,
    // Pre-compiled selectors to avoid panic risks
    disclosure_selectors: Vec<Selector>,
    link_selector: Selector,
    base_url: Url,
}

/// Cyber incident keyword matcher for Bahasa and English
struct KeywordMatcher {
    bahasa_keywords: Vec<String>,
    english_keywords: Vec<String>,
    attack_type_mapping: HashMap<String, String>,
}

impl KeywordMatcher {
    fn new() -> Self {
        let mut bahasa_keywords = vec![
            "serangan siber".to_string(),
            "kebocoran data".to_string(),
            "ransomware".to_string(),
            "gangguan sistem".to_string(),
            "pelanggaran data".to_string(),
            "insiden keamanan".to_string(),
            "serangan hacker".to_string(),
            "malware".to_string(),
            "phishing".to_string(),
            "deface".to_string(),
            "ddos".to_string(),
            "sql injection".to_string(),
            "cross-site scripting".to_string(),
            "penipuan digital".to_string(),
            "pencurian data".to_string(),
            "akun diretas".to_string(),
            "sistem diretas".to_string(),
            "keamanan siber".to_string(),
            "insiden teknologi".to_string(),
            "gangguan teknologi informasi".to_string(),
        ];

        let mut english_keywords = vec![
            "cyber attack".to_string(),
            "data breach".to_string(),
            "system disruption".to_string(),
            "unauthorized access".to_string(),
            "ransomware".to_string(),
            "malware".to_string(),
            "phishing".to_string(),
            "ddos".to_string(),
            "sql injection".to_string(),
            "cross-site scripting".to_string(),
            "hacking".to_string(),
            "data leak".to_string(),
            "security incident".to_string(),
            "cybersecurity".to_string(),
            "data theft".to_string(),
            "account compromised".to_string(),
            "system compromised".to_string(),
            "digital fraud".to_string(),
            "website defacement".to_string(),
        ];

        // Add variations and common misspellings
        bahasa_keywords.extend(vec![
            "serangan cyber".to_string(),
            "kebocoran data pribadi".to_string(),
            "kerentanan keamanan".to_string(),
            "ancaman siber".to_string(),
        ]);

        english_keywords.extend(vec![
            "cyberattack".to_string(),
            "data breach".to_string(),
            "security breach".to_string(),
            "cyber security".to_string(),
            "it security".to_string(),
        ]);

        let mut attack_type_mapping = HashMap::new();

        // Bahasa mappings
        attack_type_mapping.insert("ransomware".to_string(), "RANSOMWARE".to_string());
        attack_type_mapping.insert("malware".to_string(), "MALWARE".to_string());
        attack_type_mapping.insert("phishing".to_string(), "PHISHING".to_string());
        attack_type_mapping.insert("ddos".to_string(), "DDOS".to_string());
        attack_type_mapping.insert("sql injection".to_string(), "SQL_INJECTION".to_string());
        attack_type_mapping.insert("cross-site scripting".to_string(), "XSS".to_string());
        attack_type_mapping.insert("deface".to_string(), "WEBSITE_DEFACEMENT".to_string());
        attack_type_mapping.insert("serangan siber".to_string(), "CYBER_ATTACK".to_string());
        attack_type_mapping.insert("kebocoran data".to_string(), "DATA_BREACH".to_string());
        attack_type_mapping.insert("pencurian data".to_string(), "DATA_THEFT".to_string());
        attack_type_mapping.insert("akun diretas".to_string(), "ACCOUNT_COMPROMISE".to_string());

        // English mappings
        attack_type_mapping.insert("cyber attack".to_string(), "CYBER_ATTACK".to_string());
        attack_type_mapping.insert("data breach".to_string(), "DATA_BREACH".to_string());
        attack_type_mapping.insert("data leak".to_string(), "DATA_BREACH".to_string());
        attack_type_mapping.insert("security breach".to_string(), "DATA_BREACH".to_string());
        attack_type_mapping.insert("hacking".to_string(), "HACKING".to_string());
        attack_type_mapping.insert("data theft".to_string(), "DATA_THEFT".to_string());
        attack_type_mapping.insert(
            "account compromised".to_string(),
            "ACCOUNT_COMPROMISE".to_string(),
        );
        attack_type_mapping.insert(
            "system compromised".to_string(),
            "SYSTEM_COMPROMISE".to_string(),
        );
        attack_type_mapping.insert("digital fraud".to_string(), "DIGITAL_FRAUD".to_string());
        attack_type_mapping.insert(
            "website defacement".to_string(),
            "WEBSITE_DEFACEMENT".to_string(),
        );

        Self {
            bahasa_keywords,
            english_keywords,
            attack_type_mapping,
        }
    }

    /// Check if text contains cyber incident keywords
    fn contains_cyber_keywords(&self, text: &str) -> bool {
        let text_lower = text.to_lowercase();

        // Check Bahasa keywords
        for keyword in &self.bahasa_keywords {
            if text_lower.contains(keyword) {
                return true;
            }
        }

        // Check English keywords
        for keyword in &self.english_keywords {
            if text_lower.contains(keyword) {
                return true;
            }
        }

        false
    }

    /// Extract attack type from text
    fn extract_attack_type(&self, text: &str) -> Option<String> {
        let text_lower = text.to_lowercase();

        for (keyword, attack_type) in &self.attack_type_mapping {
            if text_lower.contains(keyword) {
                return Some(attack_type.clone());
            }
        }

        // Check for generic security incident terms
        if text_lower.contains("insiden keamanan") || text_lower.contains("security incident") {
            return Some("UNKNOWN".to_string());
        }

        None
    }

    /// Extract data categories from text
    fn extract_data_categories(&self, text: &str) -> Vec<String> {
        let text_lower = text.to_lowercase();
        let mut categories = Vec::new();

        // Personal data keywords
        if text_lower.contains("data pribadi")
            || text_lower.contains("personal data")
            || text_lower.contains("data nasabah")
            || text_lower.contains("customer data")
            || text_lower.contains("data karyawan")
            || text_lower.contains("employee data")
        {
            categories.push("PERSONAL_DATA".to_string());
        }

        // Financial data keywords
        if text_lower.contains("data keuangan")
            || text_lower.contains("financial data")
            || text_lower.contains("data kartu kredit")
            || text_lower.contains("credit card data")
            || text_lower.contains("data rekening")
            || text_lower.contains("account data")
        {
            categories.push("FINANCIAL_DATA".to_string());
        }

        // Health data keywords
        if text_lower.contains("data kesehatan")
            || text_lower.contains("health data")
            || text_lower.contains("data medis")
            || text_lower.contains("medical data")
        {
            categories.push("HEALTH_DATA".to_string());
        }

        categories
    }
}

impl IdxCrawler {
    pub fn new() -> Result<Self, anyhow::Error> {
        let config = SourceConfig {
            name: "IDX".to_string(),
            base_url: "https://www.idx.co.id".to_string(),
            rate_limit: Duration::from_secs(2), // Respectful crawling
            enabled: true,
        };

        let base_url =
            Url::parse(&config.base_url).map_err(|e| anyhow::anyhow!("Invalid base URL: {}", e))?;

        // Pre-compile all regexes
        let org_name_regex = Regex::new(r"^([A-Za-z\s&\.\-\(\)]+(?:PT|Tbk|CV|FA)\s+[A-Za-z\s&\.\-\(\)]+|[A-Za-z\s&\.\-\(\)]+(?:PT|Tbk|CV|FA))")
            .map_err(|e| anyhow::anyhow!("Failed to compile org name regex: {}", e))?;

        let date_regexes = vec![
            Regex::new(r"(\d{1,2})\s+([A-Za-z]+)\s+(\d{4})") // 15 April 2024
                .map_err(|e| anyhow::anyhow!("Failed to compile date regex 1: {}", e))?,
            Regex::new(r"(\d{4}-\d{2}-\d{2})") // 2024-04-15
                .map_err(|e| anyhow::anyhow!("Failed to compile date regex 2: {}", e))?,
            Regex::new(r"(\d{2}/\d{2}/\d{4})") // 15/04/2024
                .map_err(|e| anyhow::anyhow!("Failed to compile date regex 3: {}", e))?,
        ];

        // Pre-compile selectors
        let disclosure_selectors = vec![
            Selector::parse(".announcement-item")
                .map_err(|e| anyhow::anyhow!("Invalid selector '.announcement-item': {}", e))?,
            Selector::parse(".pengumuman-item")
                .map_err(|e| anyhow::anyhow!("Invalid selector '.pengumuman-item': {}", e))?,
            Selector::parse("tr[data-id]")
                .map_err(|e| anyhow::anyhow!("Invalid selector 'tr[data-id]': {}", e))?,
            Selector::parse("table tr")
                .map_err(|e| anyhow::anyhow!("Invalid selector 'table tr': {}", e))?,
        ];

        let link_selector =
            Selector::parse("a").map_err(|e| anyhow::anyhow!("Invalid selector 'a': {}", e))?;

        Ok(Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .user_agent("ID-Siber-Index-Crawler/1.0")
                .build()
                .map_err(|e| anyhow::anyhow!("Failed to create HTTP client: {}", e))?,
            rate_limiter: RateLimiter::new(1), // 1 request per 2 seconds
            keywords: KeywordMatcher::new(),
            config,
            org_name_regex,
            date_regexes,
            disclosure_selectors,
            link_selector,
            base_url,
        })
    }

    /// Fetch IDX electronic disclosure feed
    async fn fetch_disclosure_feed(&self) -> Result<String, anyhow::Error> {
        // Rate limiting with timeout
        tokio::time::timeout(RATE_LIMIT_TIMEOUT, self.rate_limiter.acquire())
            .await
            .map_err(|_| {
                anyhow::anyhow!(
                    "Rate limiter timeout after {} seconds",
                    RATE_LIMIT_TIMEOUT.as_secs()
                )
            })??;

        // Try multiple IDX disclosure endpoints
        let endpoints = vec![
            "https://www.idx.co.id/umbraco/Surface/Announcement/GetAnnouncement",
            "https://www.idx.co.id/en/announcements",
            "https://www.idx.co.id/id/pengumuman",
        ];

        for endpoint in endpoints {
            match self.client.get(endpoint).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        let content = response
                            .text()
                            .await
                            .map_err(|e| anyhow::anyhow!("Failed to read response body: {}", e))?;
                        if !content.trim().is_empty() {
                            return Ok(content);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to fetch from {}: {}", endpoint, e);
                }
            }
        }

        Err(anyhow::anyhow!(
            "Failed to fetch IDX disclosure feed from all endpoints"
        ))
    }

    /// Parse disclosure items from HTML content
    fn parse_disclosure_items(
        &self,
        html_content: &str,
    ) -> Result<Vec<DisclosureItem>, anyhow::Error> {
        let document = Html::parse_document(html_content);
        let mut items = Vec::new();
        let mut total_items = 0;

        // Try different selectors for disclosure items
        for selector in &self.disclosure_selectors {
            for element in document.select(selector) {
                total_items += 1;
                if total_items > MAX_ITEMS_PER_PAGE {
                    eprintln!(
                        "Warning: Reached maximum items limit ({})",
                        MAX_ITEMS_PER_PAGE
                    );
                    break;
                }

                if let Ok(Some(item)) = self.parse_disclosure_item(&element) {
                    items.push(item);
                }
            }

            if !items.is_empty() {
                break; // Use first successful selector
            }
        }

        Ok(items)
    }

    /// Parse individual disclosure item
    fn parse_disclosure_item(
        &self,
        element: &scraper::ElementRef,
    ) -> Result<Option<DisclosureItem>, anyhow::Error> {
        let text_content = element.text().collect::<String>();

        // Extract organization name (usually in the first few words)
        let org_name = self.extract_org_name(&text_content)?;

        // Extract date
        let disclosure_date = self.extract_date(&text_content)?;

        // Extract URL
        let source_url = self.extract_url(element)?;

        // Extract title/description
        let description = self.extract_description(&text_content);

        if org_name.is_empty() || source_url.is_empty() {
            return Ok(None);
        }

        Ok(Some(DisclosureItem {
            org_name,
            disclosure_date,
            source_url,
            description,
            raw_content: text_content,
        }))
    }

    /// Extract organization name from text
    fn extract_org_name(&self, text: &str) -> Result<String, anyhow::Error> {
        // Try to extract organization name from the beginning of the text
        let lines: Vec<&str> = text.lines().collect();
        if !lines.is_empty() {
            let first_line = lines[0].trim();

            // Common patterns for company names in IDX disclosures
            if let Some(caps) = self.org_name_regex.captures(first_line) {
                let mut org_name = caps[1].trim().to_string();

                // Truncate at common sentence endings or action words
                let truncate_words = [
                    "mengumumkan",
                    "melaporkan",
                    "menginformasikan",
                    "menyatakan",
                    "terjadi",
                    "mengalami",
                ];
                for word in &truncate_words {
                    if let Some(pos) = org_name.to_lowercase().find(word) {
                        org_name = org_name[..pos].trim().to_string();
                        break;
                    }
                }

                if self.is_valid_org_name(&org_name) {
                    return Ok(org_name);
                }
            }

            // Fallback: first few words
            let words: Vec<&str> = first_line.split_whitespace().collect();
            if words.len() >= 2 {
                let mut org_name = words.iter().take(3).cloned().collect::<Vec<_>>().join(" ");

                // Truncate at action words for fallback too
                let truncate_words = [
                    "mengumumkan",
                    "melaporkan",
                    "menginformasikan",
                    "menyatakan",
                    "terjadi",
                    "mengalami",
                ];
                for word in &truncate_words {
                    if let Some(pos) = org_name.to_lowercase().find(word) {
                        org_name = org_name[..pos].trim().to_string();
                        break;
                    }
                }

                if self.is_valid_org_name(&org_name) {
                    return Ok(org_name);
                }
            }
        }

        // If no valid org name found, return empty string
        Ok(String::new())
    }

    /// Validate organization name
    fn is_valid_org_name(&self, name: &str) -> bool {
        let trimmed_name = name.trim();
        let words: Vec<&str> = trimmed_name.split_whitespace().collect();

        // Basic checks
        if trimmed_name.len() < MIN_ORG_NAME_LENGTH {
            return false;
        }

        // Must contain at least one alphabetic character
        if !trimmed_name.chars().any(|c| c.is_alphabetic()) {
            return false;
        }

        // Cannot be all numeric
        if trimmed_name.chars().all(|c| c.is_numeric()) {
            return false;
        }

        // Cannot contain control characters
        if trimmed_name.chars().any(|c| c.is_control()) {
            return false;
        }

        // For short names (under 10 chars), require at least 3 words
        if trimmed_name.len() < 10 {
            if words.len() < 3 {
                return false;
            }
        } else {
            // For longer names, require at least 2 words
            if words.len() < 2 {
                return false;
            }
        }

        // Reject single-character words (except common abbreviations)
        let single_char_words: Vec<&str> = words.iter().filter(|w| w.len() == 1).copied().collect();
        if single_char_words.len() > 2 {
            return false;
        }

        // Reject patterns where first word is very short (1-2 chars) and contains common filler words
        if words.len() >= 3 {
            let first_word = words[0];
            if first_word.len() <= 2 {
                let filler_words = ["laporan", "tahunan", "informasi", "pengumuman", "berita"];
                if words
                    .iter()
                    .any(|w| filler_words.contains(&w.to_lowercase().as_str()))
                {
                    return false;
                }
            }
        }

        true
    }

    /// Extract date from text
    fn extract_date(&self, text: &str) -> Result<NaiveDate, anyhow::Error> {
        for regex in &self.date_regexes {
            if let Some(caps) = regex.captures(text) {
                // Safe access to capture groups
                let date_str = caps
                    .get(0)
                    .ok_or_else(|| anyhow::anyhow!("No date match found"))?
                    .as_str();

                // Try Indonesian month names with static mapping
                let mut normalized_date = date_str.to_string();
                for (id, en) in INDONESIAN_MONTHS.iter() {
                    normalized_date = normalized_date.replace(id, en);
                }

                // Try different date formats
                let possible_formats = vec!["%d %B %Y", "%Y-%m-%d", "%d/%m/%Y"];
                for format in possible_formats {
                    if let Ok(date) = NaiveDate::parse_from_str(&normalized_date, format) {
                        // Validate date is reasonable (not too far in the past or future)
                        let today = Utc::now().date_naive();
                        let min_date = NaiveDate::from_ymd_opt(2020, 1, 1)
                            .ok_or_else(|| anyhow::anyhow!("Invalid min date"))?;

                        if date < min_date {
                            eprintln!("Warning: Date {} is before minimum date 2020-01-01", date);
                            return Ok(today); // Fallback to today
                        }

                        if date > today {
                            eprintln!(
                                "Warning: Date {} is in the future, using today instead",
                                date
                            );
                            return Ok(today); // Fallback to today
                        }

                        return Ok(date);
                    }
                }
            }
        }

        // Fallback to current date with warning
        eprintln!("Warning: Could not extract date from text, using current date");
        Ok(Utc::now().date_naive())
    }

    /// Extract URL from element
    fn extract_url(&self, element: &scraper::ElementRef) -> Result<String, anyhow::Error> {
        if let Some(link) = element.select(&self.link_selector).next()
            && let Some(href) = link.value().attr("href")
        {
            let url = if href.starts_with("http") {
                href.to_string()
            } else {
                // Safe URL construction
                self.base_url
                    .join(href)
                    .map_err(|e| {
                        anyhow::anyhow!(
                            "Failed to construct URL from base {} and href {}: {}",
                            self.base_url,
                            href,
                            e
                        )
                    })?
                    .to_string()
            };
            return Ok(url);
        }

        Err(anyhow::anyhow!("No URL found"))
    }

    /// Extract description from text
    fn extract_description(&self, text: &str) -> String {
        // Take first few sentences as description
        let sentences: Vec<&str> = text.split(&['.', '!', '?'][..]).collect();
        sentences
            .iter()
            .take(3)
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(". ")
    }

    /// Filter disclosures for cyber incidents
    fn filter_cyber_incidents(&self, items: Vec<DisclosureItem>) -> Vec<DisclosureItem> {
        items
            .into_iter()
            .filter(|item| {
                self.keywords.contains_cyber_keywords(&item.description)
                    || self.keywords.contains_cyber_keywords(&item.raw_content)
            })
            .collect()
    }

    /// Convert disclosure items to incident drafts
    fn convert_to_incident_drafts(&self, items: Vec<DisclosureItem>) -> Vec<IncidentDraft> {
        items
            .into_iter()
            .map(|item| {
                let attack_type = self.keywords.extract_attack_type(&item.description);
                let data_categories = self.keywords.extract_data_categories(&item.description);

                let confidence = if attack_type.is_some() { 0.8 } else { 0.6 };

                IncidentDraft::new(
                    item.org_name,
                    item.disclosure_date,
                    item.source_url,
                    "IDX_DISCLOSURE".to_string(),
                )
                .with_attack_type(attack_type)
                .with_data_categories(data_categories)
                .with_confidence(confidence)
                .with_raw_content(Some(item.raw_content))
                .with_notes(Some(item.description))
            })
            .collect()
    }

    /// Remove duplicates based on organization and date proximity
    fn remove_duplicates(&self, drafts: Vec<IncidentDraft>) -> Vec<IncidentDraft> {
        let mut unique_drafts = Vec::new();

        for draft in drafts {
            let is_duplicate = unique_drafts
                .iter()
                .any(|existing| draft.is_potential_duplicate(existing, 7)); // 7-day window

            if !is_duplicate {
                unique_drafts.push(draft);
            }
        }

        unique_drafts
    }

    /// Historical backfill for 2020-present data
    pub async fn backfill_historical(
        &self,
        start_year: i32,
        end_year: i32,
    ) -> Result<Vec<ExtractionResult>, anyhow::Error> {
        let mut all_results = Vec::new();

        for year in start_year..=end_year {
            eprintln!("Backfilling year: {}", year);

            for month in 1..=12 {
                let start_date = NaiveDate::from_ymd_opt(year, month, 1)
                    .ok_or_else(|| anyhow::anyhow!("Invalid date: {}-{}-1", year, month))?;

                let end_date = if month == 12 {
                    NaiveDate::from_ymd_opt(year + 1, 1, 1)
                } else {
                    NaiveDate::from_ymd_opt(year, month + 1, 1)
                }
                .ok_or_else(|| anyhow::anyhow!("Invalid date calculation"))?;

                // Fetch disclosures for this month
                if let Ok(monthly_results) = self
                    .fetch_historical_disclosures(start_date, end_date)
                    .await
                {
                    eprintln!(
                        "Found {} disclosures for {}-{:02}",
                        monthly_results.len(),
                        year,
                        month
                    );
                    all_results.extend(monthly_results);
                }

                // Rate limiting between months
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }

        // Remove duplicates across all historical data
        let unique_drafts: Vec<IncidentDraft> = all_results
            .into_iter()
            .map(|result| {
                // Convert ExtractionResult back to IncidentDraft for deduplication
                IncidentDraft::new(
                    result.org_name,
                    result.disclosure_date,
                    result.source_url,
                    result.source_type,
                )
                .with_attack_type(Some(result.attack_type))
                .with_data_categories(result.data_categories)
                .with_confidence(result.confidence)
                .with_notes(result.notes)
            })
            .collect();

        let final_drafts = self.remove_duplicates(unique_drafts);

        eprintln!("Total unique incidents found: {}", final_drafts.len());

        // Convert back to ExtractionResult
        Ok(final_drafts
            .into_iter()
            .map(|draft| draft.to_extraction_result())
            .collect())
    }

    /// Fetch historical disclosures for a specific date range
    async fn fetch_historical_disclosures(
        &self,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<ExtractionResult>, anyhow::Error> {
        // Rate limiting with timeout
        tokio::time::timeout(RATE_LIMIT_TIMEOUT, self.rate_limiter.acquire())
            .await
            .map_err(|_| {
                anyhow::anyhow!(
                    "Rate limiter timeout after {} seconds",
                    RATE_LIMIT_TIMEOUT.as_secs()
                )
            })??;

        // Try historical IDX disclosure endpoints with date parameters
        let endpoints = vec![
            format!(
                "https://www.idx.co.id/umbraco/Surface/Announcement/GetAnnouncement?from={}&to={}",
                start_date.format("%Y-%m-%d"),
                end_date.format("%Y-%m-%d")
            ),
            format!(
                "https://www.idx.co.id/en/announcements?from={}&to={}",
                start_date.format("%Y-%m-%d"),
                end_date.format("%Y-%m-%d")
            ),
        ];

        for endpoint in endpoints {
            match self.fetch_with_pagination(&endpoint).await {
                Ok(results) => {
                    if !results.is_empty() {
                        return Ok(results);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to fetch historical data from {}: {}", endpoint, e);
                }
            }
        }

        Ok(Vec::new())
    }

    /// Fetch data with pagination support
    async fn fetch_with_pagination(
        &self,
        base_url: &str,
    ) -> Result<Vec<ExtractionResult>, anyhow::Error> {
        let mut all_results = Vec::new();
        let mut page = 1;

        loop {
            let page_url = format!("{}&page={}&limit={}", base_url, page, MAX_ITEMS_PER_PAGE);

            match self.client.get(&page_url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        let content = response
                            .text()
                            .await
                            .map_err(|e| anyhow::anyhow!("Failed to read response body: {}", e))?;

                        if content.trim().is_empty() {
                            break;
                        }

                        // Parse and filter the page data
                        let items = self.parse_disclosure_items(&content)?;
                        if items.is_empty() {
                            break; // No more items
                        }

                        let cyber_items = self.filter_cyber_incidents(items);
                        let page_results: Vec<ExtractionResult> = cyber_items
                            .into_iter()
                            .map(|item| {
                                let draft = IncidentDraft::new(
                                    item.org_name,
                                    item.disclosure_date,
                                    item.source_url,
                                    "IDX_DISCLOSURE".to_string(),
                                )
                                .with_attack_type(
                                    self.keywords.extract_attack_type(&item.description),
                                )
                                .with_data_categories(
                                    self.keywords.extract_data_categories(&item.description),
                                )
                                .with_confidence(0.8)
                                .with_raw_content(Some(item.raw_content))
                                .with_notes(Some(item.description));

                                draft.to_extraction_result()
                            })
                            .collect();

                        all_results.extend(page_results);

                        // Safety check to prevent infinite pagination
                        if page >= MAX_PAGES {
                            eprintln!("Warning: Reached maximum page limit ({})", MAX_PAGES);
                            break;
                        }

                        page += 1;

                        // Rate limiting between pages
                        tokio::time::sleep(Duration::from_millis(500)).await;
                    } else {
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("Failed to fetch page {}: {}", page_url, e);
                    break;
                }
            }
        }

        Ok(all_results)
    }

    /// Get available date ranges for historical data
    pub async fn get_available_date_ranges(
        &self,
    ) -> Result<Vec<(NaiveDate, NaiveDate)>, anyhow::Error> {
        // This would typically query the IDX API for available date ranges
        // For now, return hardcoded ranges from 2020 to present
        let current_year = chrono::Utc::now().year();
        let mut ranges = Vec::new();

        for year in 2020..=current_year {
            for month in 1..=12 {
                if year == current_year && month > chrono::Utc::now().month() {
                    break;
                }

                let start_date = NaiveDate::from_ymd_opt(year, month, 1)
                    .ok_or_else(|| anyhow::anyhow!("Invalid date"))?;

                let end_date = if month == 12 {
                    NaiveDate::from_ymd_opt(year + 1, 1, 1)
                } else {
                    NaiveDate::from_ymd_opt(year, month + 1, 1)
                }
                .ok_or_else(|| anyhow::anyhow!("Invalid date"))?;

                ranges.push((start_date, end_date));
            }
        }

        Ok(ranges)
    }

    /// Validate historical data quality
    pub fn validate_historical_data(
        &self,
        results: &[ExtractionResult],
    ) -> Result<(), anyhow::Error> {
        let mut issues = Vec::new();

        for (i, result) in results.iter().enumerate() {
            // Check for required fields
            if result.org_name.is_empty() {
                issues.push(format!("Item {}: Missing organization name", i + 1));
            }

            if result.source_url.is_empty() {
                issues.push(format!("Item {}: Missing source URL", i + 1));
            }

            // Check date validity (should be between 2020 and present)
            if result.disclosure_date.year() < 2020 {
                issues.push(format!(
                    "Item {}: Disclosure date {} is before 2020",
                    i + 1,
                    result.disclosure_date
                ));
            }

            // Check confidence score
            if result.confidence < 0.3 {
                issues.push(format!(
                    "Item {}: Low confidence score {}",
                    i + 1,
                    result.confidence
                ));
            }
        }

        if !issues.is_empty() {
            eprintln!("Data validation issues found:");
            for issue in issues {
                eprintln!("  - {}", issue);
            }
        }

        Ok(())
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
        // Fetch disclosure feed
        let html_content = self.fetch_disclosure_feed().await?;

        // Parse disclosure items
        let items = self.parse_disclosure_items(&html_content)?;

        // Filter for cyber incidents
        let cyber_items = self.filter_cyber_incidents(items);

        // Convert to incident drafts
        let drafts = self.convert_to_incident_drafts(cyber_items);

        // Remove duplicates
        let unique_drafts = self.remove_duplicates(drafts);

        // Convert to extraction results
        let results: Vec<ExtractionResult> = unique_drafts
            .into_iter()
            .map(|draft| draft.to_extraction_result())
            .collect();

        Ok(results)
    }
}

/// Helper function to create IDX crawler with error handling
pub fn create_idx_crawler() -> Result<IdxCrawler, anyhow::Error> {
    IdxCrawler::new()
}

/// Disclosure item structure
#[derive(Debug, Clone)]
struct DisclosureItem {
    org_name: String,
    disclosure_date: NaiveDate,
    source_url: String,
    description: String,
    raw_content: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::incident_draft::IncidentDraft;

    #[tokio::test]
    async fn test_keyword_matcher_bahasa() {
        let matcher = KeywordMatcher::new();

        // Test Bahasa keywords
        assert!(matcher.contains_cyber_keywords("PT Bank ABC mengalami serangan siber"));
        assert!(matcher.contains_cyber_keywords("Terjadi kebocoran data nasabah"));
        assert!(matcher.contains_cyber_keywords("Sistem terkena ransomware"));
        assert!(matcher.contains_cyber_keywords("Gangguan sistem informasi"));

        // Test non-cyber content
        assert!(!matcher.contains_cyber_keywords("Laporan keuangan triwulanan"));
        assert!(!matcher.contains_cyber_keywords("Rapat umum pemegang saham"));
    }

    #[tokio::test]
    async fn test_keyword_matcher_english() {
        let matcher = KeywordMatcher::new();

        // Test English keywords
        assert!(matcher.contains_cyber_keywords("Company experienced cyber attack"));
        assert!(matcher.contains_cyber_keywords("Data breach affecting customers"));
        assert!(matcher.contains_cyber_keywords("System disruption due to malware"));
        assert!(matcher.contains_cyber_keywords("Unauthorized access detected"));

        // Test non-cyber content
        assert!(!matcher.contains_cyber_keywords("Financial report Q1 2024"));
        assert!(!matcher.contains_cyber_keywords("Annual shareholder meeting"));
    }

    #[tokio::test]
    async fn test_attack_type_extraction() {
        let matcher = KeywordMatcher::new();

        // Test Bahasa attack types
        assert_eq!(
            matcher.extract_attack_type("Sistem terkena ransomware"),
            Some("RANSOMWARE".to_string())
        );
        assert_eq!(
            matcher.extract_attack_type("Kejadian kebocoran data"),
            Some("DATA_BREACH".to_string())
        );

        // Test English attack types
        assert_eq!(
            matcher.extract_attack_type("System infected with malware"),
            Some("MALWARE".to_string())
        );
        assert_eq!(
            matcher.extract_attack_type("Phishing email detected"),
            Some("PHISHING".to_string())
        );

        // Test unknown attack type
        assert_eq!(
            matcher.extract_attack_type("Security incident occurred"),
            Some("UNKNOWN".to_string())
        );

        // Test no attack type
        assert_eq!(
            matcher.extract_attack_type("Regular business operation"),
            None
        );
    }

    #[tokio::test]
    async fn test_incident_draft_creation() {
        let draft = IncidentDraft::new(
            "PT Test Company".to_string(),
            NaiveDate::from_ymd_opt(2024, 4, 15).unwrap(),
            "https://example.com/disclosure".to_string(),
            "IDX_DISCLOSURE".to_string(),
        )
        .with_attack_type(Some("RANSOMWARE".to_string()))
        .with_data_categories(vec!["PERSONAL_DATA".to_string()])
        .with_confidence(0.8);

        assert_eq!(draft.org_name, "PT Test Company");
        assert_eq!(draft.attack_type, Some("RANSOMWARE".to_string()));
        assert_eq!(draft.data_categories, vec!["PERSONAL_DATA"]);
        assert_eq!(draft.confidence, 0.8);
    }

    #[tokio::test]
    async fn test_duplicate_detection() {
        let draft1 = IncidentDraft::new(
            "PT Bank ABC".to_string(),
            NaiveDate::from_ymd_opt(2024, 4, 15).unwrap(),
            "https://example.com/1".to_string(),
            "IDX_DISCLOSURE".to_string(),
        );

        let draft2 = IncidentDraft::new(
            "PT Bank ABC".to_string(),
            NaiveDate::from_ymd_opt(2024, 4, 18).unwrap(), // 3 days later
            "https://example.com/2".to_string(),
            "IDX_DISCLOSURE".to_string(),
        );

        // Test duplicate detection (7-day window)
        assert!(draft1.is_potential_duplicate(&draft2, 7)); // Same org, 3 days apart
    }

    #[tokio::test]
    async fn test_historical_date_ranges() {
        let crawler = IdxCrawler::new().unwrap();

        // Test that we can get date ranges
        let ranges = crawler.get_available_date_ranges().await;
        assert!(ranges.is_ok());

        let ranges = ranges.unwrap();
        assert!(!ranges.is_empty());

        // Check that ranges start from 2020
        assert!(ranges.iter().any(|(start, _)| start.year() >= 2020));
    }

    #[tokio::test]
    async fn test_idx_crawler_creation() {
        let crawler = IdxCrawler::new();
        assert!(crawler.is_ok());

        let crawler = crawler.unwrap();
        assert_eq!(crawler.name(), "IDX");
        assert!(crawler.config().enabled);
        assert_eq!(crawler.config().base_url, "https://www.idx.co.id");
    }

    #[tokio::test]
    async fn test_org_name_extraction() {
        let crawler = IdxCrawler::new().unwrap();

        // Test PT company names
        let org_name = crawler
            .extract_org_name("PT Bank Central Asia Tbk mengumumkan...")
            .unwrap();
        assert_eq!(org_name, "PT Bank Central Asia Tbk");

        // Test CV company names
        let org_name = crawler
            .extract_org_name("CV Teknologi Digital melaporkan...")
            .unwrap();
        assert_eq!(org_name, "CV Teknologi Digital");

        // Test valid fallback case
        let org_name = crawler
            .extract_org_name("Bank Indonesia laporan tahunan")
            .unwrap();
        assert_eq!(org_name, "Bank Indonesia laporan");

        // Test invalid org names (too short) - should return empty
        let org_name = crawler.extract_org_name("AB laporan tahunan").unwrap();
        assert_eq!(org_name, "");

        // Test numeric only - should return empty
        let org_name = crawler.extract_org_name("123 456 789 laporan").unwrap();
        assert_eq!(org_name, "");
    }

    #[tokio::test]
    async fn test_data_validation() {
        let crawler = IdxCrawler::new().unwrap();

        // Create test data with some issues
        let results = vec![
            crate::extractors::ExtractionResult {
                org_name: "PT Test Company".to_string(),
                org_sector: "BANKING".to_string(),
                incident_date: NaiveDate::from_ymd_opt(2024, 4, 15).unwrap(),
                disclosure_date: NaiveDate::from_ymd_opt(2024, 4, 16).unwrap(),
                attack_type: "RANSOMWARE".to_string(),
                data_categories: vec!["PERSONAL_DATA".to_string()],
                record_count_estimate: Some(1000),
                financial_impact_idr: Some(5000000000),
                actor_alias: Some("TestActor".to_string()),
                actor_group: None,
                source_url: "https://example.com".to_string(),
                source_type: "IDX_DISCLOSURE".to_string(),
                notes: Some("Test note".to_string()),
                confidence: 0.9,
            },
            crate::extractors::ExtractionResult {
                org_name: "".to_string(), // Missing org name
                org_sector: "BANKING".to_string(),
                incident_date: NaiveDate::from_ymd_opt(2024, 4, 15).unwrap(),
                disclosure_date: NaiveDate::from_ymd_opt(2024, 4, 16).unwrap(),
                attack_type: "RANSOMWARE".to_string(),
                data_categories: vec!["PERSONAL_DATA".to_string()],
                record_count_estimate: Some(1000),
                financial_impact_idr: Some(5000000000),
                actor_alias: Some("TestActor".to_string()),
                actor_group: None,
                source_url: "https://example.com".to_string(),
                source_type: "IDX_DISCLOSURE".to_string(),
                notes: Some("Test note".to_string()),
                confidence: 0.2, // Low confidence
            },
        ];

        // Validation should pass but log issues
        let validation_result = crawler.validate_historical_data(&results);
        assert!(validation_result.is_ok());
    }
}
