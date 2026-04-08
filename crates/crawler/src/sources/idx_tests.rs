//! Unit tests for IDX crawler

use crate::sources::idx::{IdxCrawler, KeywordMatcher};
use crate::incident_draft::IncidentDraft;
use chrono::NaiveDate;
use scraper::Html;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyword_matcher_bahasa() {
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

    #[test]
    fn test_keyword_matcher_english() {
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

    #[test]
    fn test_attack_type_extraction() {
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

    #[test]
    fn test_data_category_extraction() {
        let matcher = KeywordMatcher::new();
        
        // Test personal data
        let categories = matcher.extract_data_categories("Kebocoran data pribadi nasabah");
        assert!(categories.contains(&"PERSONAL_DATA".to_string()));
        
        // Test financial data
        let categories = matcher.extract_data_categories("Data kartu kredit bocor");
        assert!(categories.contains(&"FINANCIAL_DATA".to_string()));
        
        // Test health data
        let categories = matcher.extract_data_categories("Data medis pasien terancam");
        assert!(categories.contains(&"HEALTH_DATA".to_string()));
        
        // Test multiple categories
        let categories = matcher.extract_data_categories(
            "Data pribadi dan data keuangan nasabah bocor"
        );
        assert!(categories.contains(&"PERSONAL_DATA".to_string()));
        assert!(categories.contains(&"FINANCIAL_DATA".to_string()));
    }

    #[test]
    fn test_idx_crawler_creation() {
        let crawler = IdxCrawler::new();
        assert_eq!(crawler.name(), "IDX");
        assert!(crawler.config().enabled);
        assert_eq!(crawler.config().base_url, "https://www.idx.co.id");
    }

    #[test]
    fn test_org_name_extraction() {
        let crawler = IdxCrawler::new();
        
        // Test PT company names
        let org_name = crawler.extract_org_name("PT Bank Central Asia Tbk mengumumkan...");
        assert_eq!(org_name, "PT Bank Central Asia Tbk");
        
        // Test CV company names
        let org_name = crawler.extract_org_name("CV Teknologi Digital melaporkan...");
        assert_eq!(org_name, "CV Teknologi Digital");
        
        // Test fallback
        let org_name = crawler.extract_org_name("Bank Indonesia laporan tahunan");
        assert_eq!(org_name, "Bank Indonesia laporan tahunan");
    }

    #[test]
    fn test_date_extraction() {
        let crawler = IdxCrawler::new();
        
        // Test Indonesian date format
        let date = crawler.extract_date("15 April 2024").unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2024, 4, 15).unwrap());
        
        // Test ISO date format
        let date = crawler.extract_date("2024-04-15").unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2024, 4, 15).unwrap());
        
        // Test Indonesian month names
        let date = crawler.extract_date("15 Januari 2024").unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2024, 1, 15).unwrap());
        
        // Test fallback (should return current date)
        let date = crawler.extract_date("no date here").unwrap();
        // Should be today's date
        assert!(date <= chrono::Utc::now().date_naive());
    }

    #[test]
    fn test_description_extraction() {
        let crawler = IdxCrawler::new();
        
        let text = "PT Bank ABC mengalami serangan siber. Data nasabah terancam. Sistem akan diperbaiki.";
        let description = crawler.extract_description(text);
        
        assert!(description.contains("PT Bank ABC mengalami serangan siber"));
        assert!(description.contains("Data nasabah terancam"));
        assert!(description.contains("Sistem akan diperbaiki"));
    }

    #[test]
    fn test_incident_draft_creation() {
        let draft = IncidentDraft::new(
            "PT Test Company".to_string(),
            NaiveDate::from_ymd_opt(2024, 4, 15).unwrap(),
            "https://example.com/disclosure".to_string(),
            "IDX_DISCLOSURE".to_string(),
        )
        .with_attack_type(Some("RANSOMWARE".to_string()))
        .with_data_categories(vec!["PERSONAL_DATA".to_string()])
        .with_confidence(0.8)
        .with_notes(Some("Test incident".to_string()));

        assert_eq!(draft.org_name, "PT Test Company");
        assert_eq!(draft.attack_type, Some("RANSOMWARE".to_string()));
        assert_eq!(draft.data_categories, vec!["PERSONAL_DATA"]);
        assert_eq!(draft.confidence, 0.8);
        assert_eq!(draft.notes, Some("Test incident".to_string()));
    }

    #[test]
    fn test_duplicate_detection() {
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

        let draft3 = IncidentDraft::new(
            "PT Bank XYZ".to_string(),
            NaiveDate::from_ymd_opt(2024, 4, 16).unwrap(),
            "https://example.com/3".to_string(),
            "IDX_DISCLOSURE".to_string(),
        );

        // Test duplicate detection (7-day window)
        assert!(draft1.is_potential_duplicate(&draft2, 7)); // Same org, 3 days apart
        assert!(!draft1.is_potential_duplicate(&draft3, 7)); // Different org
        
        // Test with smaller window
        assert!(!draft1.is_potential_duplicate(&draft2, 2)); // Same org, but 3 days apart > 2-day window
    }

    #[test]
    fn test_html_parsing() {
        let html = r#"
        <div class="announcement-item">
            <a href="/announcement/123">PT Bank Test</a>
            <span>15 April 2024</span>
            <p>Terjadi serangan siber yang mengganggu sistem</p>
        </div>
        "#;

        let document = Html::parse_fragment(html);
        let crawler = IdxCrawler::new();
        
        // Test that we can parse HTML without crashing
        let items = crawler.parse_disclosure_items(html);
        assert!(items.is_ok());
    }

    #[tokio::test]
    async fn test_crawler_crawl_interface() {
        let crawler = IdxCrawler::new();
        
        // Test that the crawl method exists and returns the right type
        // Note: This test doesn't actually crawl since it would require network access
        let result = crawler.crawl().await;
        
        // Should return an error since we can't actually connect to IDX in tests
        assert!(result.is_err());
    }

    #[test]
    fn test_incident_draft_to_extraction_result() {
        let draft = IncidentDraft::new(
            "PT Test Company".to_string(),
            NaiveDate::from_ymd_opt(2024, 4, 15).unwrap(),
            "https://example.com/disclosure".to_string(),
            "IDX_DISCLOSURE".to_string(),
        )
        .with_attack_type(Some("RANSOMWARE".to_string()))
        .with_data_categories(vec!["PERSONAL_DATA".to_string()])
        .with_confidence(0.8);

        let extraction_result = draft.to_extraction_result();
        
        assert_eq!(extraction_result.org_name, "PT Test Company");
        assert_eq!(extraction_result.attack_type, "RANSOMWARE");
        assert_eq!(extraction_result.data_categories, vec!["PERSONAL_DATA"]);
        assert_eq!(extraction_result.confidence, 0.8);
        assert_eq!(extraction_result.source_type, "IDX_DISCLOSURE");
    }
}
