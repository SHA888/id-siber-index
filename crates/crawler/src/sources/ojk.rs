//! OJK (Financial Services Authority) crawler source
//!
//! This crawler parses OJK enforcement releases and complaint summary reports
//! to extract financial sector incidents and fraud complaints data.
//! It also links to relevant IDX disclosures where organizations match.

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
use tracing::{debug, warn};
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

/// OJK crawler for enforcement releases and complaint reports
pub struct OjkCrawler {
    config: SourceConfig,
    client: Client,
    rate_limiter: RateLimiter,
    keywords: OjkKeywordMatcher,
    // Pre-compiled regexes for performance
    org_name_regex: Regex,
    pt_regex: Regex, // Pre-compiled PT company pattern
    date_regexes: Vec<Regex>,
    // Pre-compiled selectors
    enforcement_selectors: Vec<Selector>,
    complaint_selectors: Vec<Selector>,
    link_selector: Selector,
    title_selector: Selector,
    base_url: Url,
}

/// OJK-specific keyword matcher for financial cyber incidents and fraud
pub struct OjkKeywordMatcher {
    bahasa_keywords: Vec<String>,
    english_keywords: Vec<String>,
    attack_type_mapping: HashMap<String, String>,
    fraud_type_mapping: HashMap<String, String>,
    /// Financial sub-sector mapping (banking, insurance, securities, fintech)
    sector_keywords: HashMap<String, String>,
}

impl Default for OjkKeywordMatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl OjkKeywordMatcher {
    pub fn new() -> Self {
        let bahasa_keywords = vec![
            // Cyber incident keywords
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
            // Financial fraud keywords
            "penipuan".to_string(),
            "fraud".to_string(),
            "pemalsuan".to_string(),
            "pencucian uang".to_string(),
            "money laundering".to_string(),
            "skimming".to_string(),
            "carding".to_string(),
            "pembobolan rekening".to_string(),
            "account takeover".to_string(),
            "unauthorized transaction".to_string(),
            "transaksi tidak sah".to_string(),
            "social engineering".to_string(),
            "modus penipuan".to_string(),
            "kejahatan finansial".to_string(),
            "financial crime".to_string(),
            "cyber crime".to_string(),
            "kejahatan siber".to_string(),
        ]
        .iter()
        .map(|s| s.to_lowercase())
        .collect();

        let english_keywords = vec![
            // Cyber incident keywords
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
            // Financial fraud keywords
            "fraud".to_string(),
            "financial fraud".to_string(),
            "banking fraud".to_string(),
            "payment fraud".to_string(),
            "identity theft".to_string(),
            "card skimming".to_string(),
            "account takeover".to_string(),
            "unauthorized transaction".to_string(),
            "social engineering".to_string(),
            "ponzi scheme".to_string(),
            "investment scam".to_string(),
            "financial crime".to_string(),
            "cyber crime".to_string(),
            "digital fraud".to_string(),
            "online scam".to_string(),
            "electronic fraud".to_string(),
        ]
        .iter()
        .map(|s| s.to_lowercase())
        .collect();

        let mut attack_type_mapping = HashMap::new();
        // Cyber attack mappings
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
        // Fraud mappings
        attack_type_mapping.insert("penipuan".to_lowercase(), "FRAUD".to_string());
        attack_type_mapping.insert("fraud".to_lowercase(), "FRAUD".to_string());
        attack_type_mapping.insert(
            "pencucian uang".to_lowercase(),
            "MONEY_LAUNDERING".to_string(),
        );
        attack_type_mapping.insert(
            "money laundering".to_lowercase(),
            "MONEY_LAUNDERING".to_string(),
        );
        attack_type_mapping.insert("skimming".to_lowercase(), "SKIMMING".to_string());
        attack_type_mapping.insert("carding".to_lowercase(), "CARDING".to_string());
        attack_type_mapping.insert(
            "pembobolan rekening".to_lowercase(),
            "ACCOUNT_TAKEOVER".to_string(),
        );
        attack_type_mapping.insert(
            "account takeover".to_lowercase(),
            "ACCOUNT_TAKEOVER".to_string(),
        );
        attack_type_mapping.insert(
            "social engineering".to_lowercase(),
            "SOCIAL_ENGINEERING".to_string(),
        );

        let mut fraud_type_mapping = HashMap::new();
        fraud_type_mapping.insert(
            "penipuan investasi".to_lowercase(),
            "INVESTMENT_FRAUD".to_string(),
        );
        fraud_type_mapping.insert(
            "investment scam".to_lowercase(),
            "INVESTMENT_FRAUD".to_string(),
        );
        fraud_type_mapping.insert("ponzi".to_lowercase(), "PONZI_SCHEME".to_string());
        fraud_type_mapping.insert("skimming".to_lowercase(), "CARD_SKIMMING".to_string());
        fraud_type_mapping.insert("carding".to_lowercase(), "CARDING".to_string());
        fraud_type_mapping.insert("phishing".to_lowercase(), "PHISHING".to_string());
        fraud_type_mapping.insert(
            "identity theft".to_lowercase(),
            "IDENTITY_THEFT".to_string(),
        );
        fraud_type_mapping.insert(
            "pencurian identitas".to_lowercase(),
            "IDENTITY_THEFT".to_string(),
        );

        // Financial sub-sector keywords
        let mut sector_keywords = HashMap::new();
        sector_keywords.insert("perbankan".to_lowercase(), "BANKING".to_string());
        sector_keywords.insert("bank".to_lowercase(), "BANKING".to_string());
        sector_keywords.insert("asuransi".to_lowercase(), "INSURANCE".to_string());
        sector_keywords.insert("insurance".to_lowercase(), "INSURANCE".to_string());
        sector_keywords.insert("sekuritas".to_lowercase(), "SECURITIES".to_string());
        sector_keywords.insert("efek".to_lowercase(), "SECURITIES".to_string());
        sector_keywords.insert("reksa dana".to_lowercase(), "MUTUAL_FUNDS".to_string());
        sector_keywords.insert("fintech".to_lowercase(), "FINTECH".to_string());
        sector_keywords.insert("pembiayaan".to_lowercase(), "FINANCING".to_string());
        sector_keywords.insert(
            "modal ventura".to_lowercase(),
            "VENTURE_CAPITAL".to_string(),
        );
        sector_keywords.insert("pialang".to_lowercase(), "BROKERAGE".to_string());

        Self {
            bahasa_keywords,
            english_keywords,
            attack_type_mapping,
            fraud_type_mapping,
            sector_keywords,
        }
    }

    /// Check if text contains cyber incident or fraud keywords
    pub fn contains_cyber_keywords(&self, text: &str) -> bool {
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

    /// Extract attack/fraud type from text
    pub fn extract_attack_type(&self, text: &str) -> Option<String> {
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

    /// Extract fraud type from text
    pub fn extract_fraud_type(&self, text: &str) -> Option<String> {
        let text_lower = text.to_lowercase();

        for (keyword, fraud_type) in &self.fraud_type_mapping {
            if text_lower.contains(keyword) {
                return Some(fraud_type.clone());
            }
        }

        None
    }

    /// Check if text is specifically about financial fraud
    pub fn is_financial_fraud(&self, text: &str) -> bool {
        let text_lower = text.to_lowercase();
        let fraud_keywords = [
            "penipuan",
            "fraud",
            "pencucian uang",
            "money laundering",
            "skimming",
            "carding",
            "pembobolan rekening",
            "account takeover",
            "unauthorized transaction",
            "transaksi tidak sah",
        ];

        fraud_keywords.iter().any(|kw| text_lower.contains(kw))
    }

    /// Extract financial sub-sector from text
    pub fn extract_sector(&self, text: &str) -> Option<String> {
        let text_lower = text.to_lowercase();

        for (keyword, sector) in &self.sector_keywords {
            if text_lower.contains(keyword) {
                return Some(sector.clone());
            }
        }

        None
    }

    /// Extract data categories from text for financial incidents
    pub fn extract_data_categories(&self, text: &str) -> Vec<String> {
        let text_lower = text.to_lowercase();
        let mut categories = Vec::new();

        // Personal/Customer data
        if text_lower.contains("data pribadi")
            || text_lower.contains("personal data")
            || text_lower.contains("data nasabah")
            || text_lower.contains("customer data")
            || text_lower.contains("data pemegang polis")
            || text_lower.contains("informasi nasabah")
        {
            categories.push("PERSONAL_DATA".to_string());
        }

        // Financial data
        if text_lower.contains("data keuangan")
            || text_lower.contains("financial data")
            || text_lower.contains("data kartu kredit")
            || text_lower.contains("credit card data")
            || text_lower.contains("data rekening")
            || text_lower.contains("account data")
            || text_lower.contains("informasi rekening")
            || text_lower.contains("nomor kartu")
            || text_lower.contains("cvv")
            || text_lower.contains("pin")
        {
            categories.push("FINANCIAL_DATA".to_string());
        }

        // Transaction data
        if text_lower.contains("data transaksi")
            || text_lower.contains("transaction data")
            || text_lower.contains("riwayat transaksi")
            || text_lower.contains("transaction history")
        {
            categories.push("TRANSACTION_DATA".to_string());
        }

        // Credentials
        if text_lower.contains("password")
            || text_lower.contains("kata sandi")
            || text_lower.contains("otp")
            || text_lower.contains("token")
            || text_lower.contains("kredential")
            || text_lower.contains("credentials")
        {
            categories.push("CREDENTIALS".to_string());
        }

        categories
    }
}

impl OjkCrawler {
    pub fn new() -> Result<Self, anyhow::Error> {
        let config = SourceConfig {
            name: "OJK".to_string(),
            base_url: "https://ojk.go.id".to_string(),
            rate_limit: Duration::from_secs(2),
            enabled: true,
        };

        let base_url =
            Url::parse(&config.base_url).map_err(|e| anyhow::anyhow!("Invalid base URL: {}", e))?;

        let org_name_regex = Regex::new(
            r"([A-Z][a-zA-Z\s&\.\-\(\)]+(?:PT|Tbk|CV|FA|Persero|Bank|Syariah)?|[A-Z][a-zA-Z\s]+(?:Bank|Perbankan)?)",
        )
        .map_err(|e| anyhow::anyhow!("Failed to compile org name regex: {}", e))?;

        let pt_regex = Regex::new(r"PT\s+[A-Za-z\s&\.]+(?:Tbk)?")
            .map_err(|e| anyhow::anyhow!("Failed to compile PT regex: {}", e))?;

        let date_regexes = vec![
            Regex::new(r"(\d{1,2})\s+([A-Za-z]+)\s+(\d{4})")
                .map_err(|e| anyhow::anyhow!("Failed to compile date regex 1: {}", e))?,
            Regex::new(r"(\d{4}-\d{2}-\d{2})")
                .map_err(|e| anyhow::anyhow!("Failed to compile date regex 2: {}", e))?,
            Regex::new(r"(\d{2}/\d{2}/\d{4})")
                .map_err(|e| anyhow::anyhow!("Failed to compile date regex 3: {}", e))?,
        ];

        let enforcement_selectors = vec![
            Selector::parse(".news-item")
                .map_err(|e| anyhow::anyhow!("Invalid selector '.news-item': {}", e))?,
            Selector::parse(".press-release-item")
                .map_err(|e| anyhow::anyhow!("Invalid selector '.press-release-item': {}", e))?,
            Selector::parse(".article-item")
                .map_err(|e| anyhow::anyhow!("Invalid selector '.article-item': {}", e))?,
            Selector::parse(".siaran-pers-item")
                .map_err(|e| anyhow::anyhow!("Invalid selector '.siaran-pers-item': {}", e))?,
            Selector::parse("div[class*='enforcement']").map_err(|e| {
                anyhow::anyhow!("Invalid selector 'div[class*='enforcement']': {}", e)
            })?,
        ];

        let complaint_selectors = vec![
            Selector::parse(".complaint-item")
                .map_err(|e| anyhow::anyhow!("Invalid selector '.complaint-item': {}", e))?,
            Selector::parse(".pengaduan-item")
                .map_err(|e| anyhow::anyhow!("Invalid selector '.pengaduan-item': {}", e))?,
            Selector::parse(".report-item")
                .map_err(|e| anyhow::anyhow!("Invalid selector '.report-item': {}", e))?,
            Selector::parse("div[class*='complaint']").map_err(|e| {
                anyhow::anyhow!("Invalid selector 'div[class*='complaint']': {}", e)
            })?,
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
            keywords: OjkKeywordMatcher::new(),
            config,
            org_name_regex,
            pt_regex,
            date_regexes,
            enforcement_selectors,
            complaint_selectors,
            link_selector,
            title_selector,
            base_url,
        })
    }

    /// Main crawl method - fetches both enforcement releases and complaint summaries
    async fn crawl_internal(&self) -> Result<Vec<ExtractionResult>, anyhow::Error> {
        let mut all_results = Vec::new();

        // Crawl enforcement releases
        if let Ok(enforcement_results) = self.crawl_enforcement_releases().await {
            all_results.extend(enforcement_results);
        } else {
            warn!("Failed to crawl OJK enforcement releases");
        }

        // Crawl complaint summaries
        if let Ok(complaint_results) = self.crawl_complaint_summaries().await {
            all_results.extend(complaint_results);
        } else {
            warn!("Failed to crawl OJK complaint summaries");
        }

        // Remove duplicates
        let unique_results = self.remove_duplicates(all_results);

        Ok(unique_results)
    }

    /// Crawl OJK enforcement releases
    async fn crawl_enforcement_releases(&self) -> Result<Vec<ExtractionResult>, anyhow::Error> {
        let mut results = Vec::new();

        let endpoints = vec![
            "https://ojk.go.id/id/kanal/iknb/data-dan-statistik/siaran-pers/Pages/default.aspx",
            "https://ojk.go.id/id/berita/siaran-pers",
            "https://ojk.go.id/id/kanal/perbankan/data-dan-statistik/siaran-pers",
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

                        let items = self.parse_enforcement_items(&content)?;
                        let filtered = self.filter_incident_items(items);
                        let converted =
                            self.convert_to_extraction_results(filtered, "OJK_ENFORCEMENT");
                        results.extend(converted);

                        if !results.is_empty() {
                            break;
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to fetch {}: {}", endpoint, e);
                }
            }
        }

        Ok(results)
    }

    /// Crawl OJK complaint summaries
    async fn crawl_complaint_summaries(&self) -> Result<Vec<ExtractionResult>, anyhow::Error> {
        let mut results = Vec::new();

        let endpoints = vec![
            "https://ojk.go.id/id/kanal/perbankan/pengaduan-nasabah/Pages/default.aspx",
            "https://ojk.go.id/id/kanal/iknb/pengaduan-konsumen",
            "https://ojk.go.id/id/pengaduan",
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

                        let items = self.parse_complaint_items(&content)?;
                        let filtered = self.filter_incident_items(items);
                        let converted =
                            self.convert_to_extraction_results(filtered, "OJK_COMPLAINT");
                        results.extend(converted);
                    }
                }
                Err(e) => {
                    warn!("Failed to fetch {}: {}", endpoint, e);
                }
            }
        }

        Ok(results)
    }

    /// Parse enforcement items from HTML
    pub fn parse_enforcement_items(
        &self,
        html_content: &str,
    ) -> Result<Vec<OjkItem>, anyhow::Error> {
        let document = Html::parse_document(html_content);
        let mut items = Vec::new();
        let mut total_items = 0;

        for selector in &self.enforcement_selectors {
            for element in document.select(selector) {
                total_items += 1;
                if total_items > MAX_ITEMS_PER_PAGE {
                    warn!(
                        "Reached maximum items limit ({}) for enforcement items",
                        MAX_ITEMS_PER_PAGE
                    );
                    break;
                }

                if let Ok(Some(item)) = self.parse_item(&element) {
                    items.push(item);
                }
            }

            if !items.is_empty() {
                break;
            }
        }

        Ok(items)
    }

    /// Parse complaint items from HTML
    pub fn parse_complaint_items(&self, html_content: &str) -> Result<Vec<OjkItem>, anyhow::Error> {
        let document = Html::parse_document(html_content);
        let mut items = Vec::new();
        let mut total_items = 0;

        for selector in &self.complaint_selectors {
            for element in document.select(selector) {
                total_items += 1;
                if total_items > MAX_ITEMS_PER_PAGE {
                    warn!(
                        "Reached maximum items limit ({}) for complaint items",
                        MAX_ITEMS_PER_PAGE
                    );
                    break;
                }

                if let Ok(Some(item)) = self.parse_item(&element) {
                    items.push(item);
                }
            }

            if !items.is_empty() {
                break;
            }
        }

        Ok(items)
    }

    /// Parse individual item from element
    fn parse_item(&self, element: &scraper::ElementRef) -> Result<Option<OjkItem>, anyhow::Error> {
        let text_content = element.text().collect::<String>();

        // Extract organization name
        let org_name = self.extract_org_name(&text_content);

        // Extract date
        let publication_date = self.extract_date(&text_content);

        // Extract URL
        let source_url = self.extract_url(element).unwrap_or_default();

        // Extract title
        let title = self.extract_title(element);

        if title.is_empty() {
            return Ok(None);
        }

        Ok(Some(OjkItem {
            title,
            org_name,
            publication_date,
            source_url,
            raw_content: text_content,
        }))
    }

    /// Extract organization name from text
    pub fn extract_org_name(&self, text: &str) -> Option<String> {
        let lines: Vec<&str> = text.lines().collect();
        if !lines.is_empty() {
            let first_line = lines[0].trim();

            // Try to match company names with common Indonesian patterns
            // Note: caps.get(0) returns the full match, which is what we want here
            if let Some(caps) = self.org_name_regex.captures(first_line) {
                let org_name = caps
                    .get(0)
                    .map(|m| m.as_str())
                    .unwrap_or_default()
                    .trim()
                    .to_string();

                // Filter for financial institution keywords
                let financial_keywords = [
                    "bank",
                    "bpr",
                    "asuransi",
                    "reksa dana",
                    "sekuritas",
                    "efek",
                    "kredit",
                    "pinjaman",
                    "fintech",
                    "pembiayaan",
                    "modal",
                    "investasi",
                    "dana",
                    "pialang",
                    "perusahaan",
                ];

                let org_lower = org_name.to_lowercase();
                if financial_keywords.iter().any(|kw| org_lower.contains(kw))
                    && self.is_valid_org_name(&org_name)
                {
                    return Some(org_name);
                }
            }

            // Fallback: look for "PT" patterns specifically (using pre-compiled regex)
            if let Some(caps) = self.pt_regex.captures(first_line) {
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
        }

        None
    }

    /// Validate organization name
    pub fn is_valid_org_name(&self, name: &str) -> bool {
        let trimmed_name = name.trim();
        let words: Vec<&str> = trimmed_name.split_whitespace().collect();

        if trimmed_name.len() < MIN_ORG_NAME_LENGTH {
            return false;
        }

        if !trimmed_name.chars().any(|c| c.is_alphabetic()) {
            return false;
        }

        if trimmed_name.chars().all(|c| c.is_numeric()) {
            return false;
        }

        if trimmed_name.chars().any(|c| c.is_control()) {
            return false;
        }

        // Require at least 2 words for organization names
        if words.len() < 2 {
            return false;
        }

        true
    }

    /// Extract date from text
    pub fn extract_date(&self, text: &str) -> NaiveDate {
        for regex in &self.date_regexes {
            if let Some(caps) = regex.captures(text) {
                let date_str = caps.get(0).map(|m| m.as_str()).unwrap_or_default();

                let mut normalized_date = date_str.to_string();
                for (id, en) in INDONESIAN_MONTHS.iter() {
                    normalized_date = normalized_date.replace(id, en);
                }

                let possible_formats = vec!["%d %B %Y", "%Y-%m-%d", "%d/%m/%Y"];
                for format in possible_formats {
                    if let Ok(date) = NaiveDate::parse_from_str(&normalized_date, format) {
                        let today = Utc::now().date_naive();
                        let min_date = NaiveDate::from_ymd_opt(2020, 1, 1)
                            .expect("Hardcoded date should be valid");

                        if date < min_date {
                            debug!("Date {} is before minimum date, using today instead", date);
                            return today;
                        }

                        if date > today {
                            debug!("Date {} is in the future, using today instead", date);
                            return today;
                        }

                        return date;
                    }
                }
                debug!("Failed to parse date '{}' with any format", normalized_date);
            }
        }

        debug!("No date found in text, using current date as fallback");
        Utc::now().date_naive()
    }

    /// Extract URL from element
    fn extract_url(&self, element: &scraper::ElementRef) -> Result<String, anyhow::Error> {
        if let Some(link) = element.select(&self.link_selector).next()
            && let Some(href) = link.value().attr("href")
        {
            // Skip empty hrefs and javascript: links
            if href.is_empty() || href.starts_with("javascript:") || href == "#" {
                return Err(anyhow::anyhow!("Invalid or empty href"));
            }

            let url = if href.starts_with("http") {
                href.to_string()
            } else if href.starts_with("//") {
                format!("{}:{}", self.base_url.scheme(), href)
            } else {
                self.base_url
                    .join(href)
                    .map_err(|e| anyhow::anyhow!("Failed to join URL: {}", e))?
                    .to_string()
            };

            // Validate URL is not empty
            if url.is_empty() {
                return Err(anyhow::anyhow!("Constructed URL is empty"));
            }

            return Ok(url);
        }

        Err(anyhow::anyhow!("No URL found"))
    }

    /// Extract title from element
    fn extract_title(&self, element: &scraper::ElementRef) -> String {
        if let Some(title_elem) = element.select(&self.title_selector).next() {
            title_elem.text().collect::<String>().trim().to_string()
        } else {
            element
                .text()
                .collect::<String>()
                .lines()
                .next()
                .unwrap_or("")
                .trim()
                .to_string()
        }
    }

    /// Filter items for cyber incidents or financial fraud
    fn filter_incident_items(&self, items: Vec<OjkItem>) -> Vec<OjkItem> {
        items
            .into_iter()
            .filter(|item| {
                let combined_text = format!("{} {}", item.title, item.raw_content);
                self.keywords.contains_cyber_keywords(&combined_text)
            })
            .collect()
    }

    /// Convert OJK items to extraction results
    fn convert_to_extraction_results(
        &self,
        items: Vec<OjkItem>,
        source_type: &str,
    ) -> Vec<ExtractionResult> {
        items
            .into_iter()
            .map(|item| {
                let combined_text = format!("{} {}", item.title, item.raw_content);
                let attack_type = self.keywords.extract_attack_type(&combined_text);
                let fraud_type = self.keywords.extract_fraud_type(&combined_text);

                // Use fraud type if no attack type found
                let final_attack_type = attack_type.or(fraud_type);

                // Extract specific financial sub-sector (e.g., BANKING, INSURANCE, FINTECH)
                let sector = self.keywords.extract_sector(&combined_text);

                // Extract data categories from content
                let data_categories = self.keywords.extract_data_categories(&combined_text);

                let draft = IncidentDraft::new(
                    item.org_name
                        .clone()
                        .unwrap_or_else(|| "OJK Report".to_string()),
                    item.publication_date,
                    item.source_url,
                    source_type.to_string(),
                )
                .with_org_sector(sector.or_else(|| Some("FINANCIAL".to_string())))
                .with_attack_type(final_attack_type)
                .with_data_categories(data_categories)
                .with_confidence(if item.org_name.is_some() { 0.8 } else { 0.6 })
                .with_raw_content(Some(item.raw_content))
                .with_notes(Some(item.title));

                draft.to_extraction_result()
            })
            .collect()
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

    /// Link to IDX disclosures for matching organizations
    /// Returns a map of OJK results to their matching IDX disclosure URLs
    pub fn link_to_idx_disclosures(
        &self,
        ojk_results: &[ExtractionResult],
        idx_results: &[ExtractionResult],
    ) -> HashMap<String, Vec<String>> {
        let mut links: HashMap<String, Vec<String>> = HashMap::new();

        // Pre-normalize IDX results once for efficiency
        let normalized_idx: Vec<(String, &ExtractionResult)> = idx_results
            .iter()
            .map(|idx| (idx.org_name.to_lowercase(), idx))
            .collect();

        for ojk_result in ojk_results {
            let ojk_org_lower = ojk_result.org_name.to_lowercase();
            let ojk_words: Vec<&str> = ojk_org_lower.split_whitespace().collect();

            let matching_idx: Vec<String> = normalized_idx
                .iter()
                .filter(|(idx_org_lower, _idx)| {
                    // Exact match
                    if ojk_org_lower == *idx_org_lower {
                        return true;
                    }

                    // Partial match (one contains the other)
                    if ojk_org_lower.contains(idx_org_lower)
                        || idx_org_lower.contains(&ojk_org_lower)
                    {
                        return true;
                    }

                    // Check for significant word overlap (>3 chars)
                    let idx_words: Vec<&str> = idx_org_lower.split_whitespace().collect();
                    ojk_words
                        .iter()
                        .any(|w| w.len() > 3 && idx_words.contains(w))
                })
                .map(|(_, idx)| idx.source_url.clone())
                .collect();

            if !matching_idx.is_empty() {
                links.insert(ojk_result.source_url.clone(), matching_idx);
            }
        }

        links
    }

    /// Get linked IDX disclosures for a specific OJK result
    pub fn get_linked_disclosures(
        &self,
        ojk_result: &ExtractionResult,
        idx_results: &[ExtractionResult],
    ) -> Vec<String> {
        let links = self.link_to_idx_disclosures(std::slice::from_ref(ojk_result), idx_results);
        links
            .get(&ojk_result.source_url)
            .cloned()
            .unwrap_or_default()
    }
}

/// OJK item structure for enforcement and complaint reports
#[derive(Debug, Clone)]
#[must_use]
pub struct OjkItem {
    pub title: String,
    pub org_name: Option<String>,
    pub publication_date: NaiveDate,
    pub source_url: String,
    pub raw_content: String,
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
        self.crawl_internal().await
    }
}
