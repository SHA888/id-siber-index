//! Unit tests for BSSN crawler

use crate::incident_draft::IncidentDraft;
use crate::sources::CrawlerSource;
use crate::sources::bssn::{BssnCrawler, BssnKeywordMatcher};
use chrono::NaiveDate;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bssn_keyword_matcher_bahasa() {
        let matcher = BssnKeywordMatcher::new();

        // Test Bahasa keywords
        assert!(matcher.contains_cyber_keywords("Serangan siber terhadap sistem perbankan"));
        assert!(matcher.contains_cyber_keywords("Terjadi kebocoran data nasabah"));
        assert!(matcher.contains_cyber_keywords("Sistem terkena ransomware"));
        assert!(matcher.contains_cyber_keywords("Gangguan sistem informasi"));
        assert!(matcher.contains_cyber_keywords("Insiden keamanan siber"));

        // Test non-cyber content
        assert!(!matcher.contains_cyber_keywords("Laporan kegiatan tahunan"));
        assert!(!matcher.contains_cyber_keywords("Rapat koordinasi nasional"));
    }

    #[test]
    fn test_bssn_keyword_matcher_english() {
        let matcher = BssnKeywordMatcher::new();

        // Test English keywords
        assert!(matcher.contains_cyber_keywords("Cyber attack against banking system"));
        assert!(matcher.contains_cyber_keywords("Data breach affecting customers"));
        assert!(matcher.contains_cyber_keywords("System disruption due to malware"));
        assert!(matcher.contains_cyber_keywords("Unauthorized access detected"));

        // Test non-cyber content
        assert!(!matcher.contains_cyber_keywords("Annual activity report"));
        assert!(!matcher.contains_cyber_keywords("National coordination meeting"));
    }

    #[test]
    fn test_attack_type_extraction() {
        let matcher = BssnKeywordMatcher::new();

        // Test Bahasa attack types
        assert_eq!(
            matcher.extract_attack_type("Sistem terkena ransomware"),
            Some("RANSOMWARE".to_string())
        );
        assert_eq!(
            matcher.extract_attack_type("Kejadian kebocoran data"),
            Some("DATA_BREACH".to_string())
        );
        assert_eq!(
            matcher.extract_attack_type("Serangan siber terdeteksi"),
            Some("CYBER_ATTACK".to_string())
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
        assert_eq!(
            matcher.extract_attack_type("DDOS attack blocked"),
            Some("DDOS".to_string())
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
    fn test_sector_extraction() {
        let matcher = BssnKeywordMatcher::new();

        // Test financial sector
        assert_eq!(
            matcher.extract_sector("Insiden di sektor perbankan"),
            Some("FINANCIAL".to_string())
        );
        assert_eq!(
            matcher.extract_sector("Bank terkena serangan"),
            Some("FINANCIAL".to_string())
        );

        // Test healthcare sector
        assert_eq!(
            matcher.extract_sector("Rumah sakit mengalami gangguan"),
            Some("HEALTHCARE".to_string())
        );
        assert_eq!(
            matcher.extract_sector("Klinik data bocor"),
            Some("HEALTHCARE".to_string())
        );

        // Test government sector
        assert_eq!(
            matcher.extract_sector("Instansi pemerintahan diretas"),
            Some("GOVERNMENT".to_string())
        );

        // Test education sector
        assert_eq!(
            matcher.extract_sector("Universitas terkena malware"),
            Some("EDUCATION".to_string())
        );

        // Test telecommunications sector
        assert_eq!(
            matcher.extract_sector("Telkom gangguan sistem"),
            Some("TELECOMMUNICATIONS".to_string())
        );

        // Test e-commerce sector
        assert_eq!(
            matcher.extract_sector("Tokopedia data breach"),
            Some("E-COMMERCE".to_string())
        );

        // Test energy sector
        assert_eq!(
            matcher.extract_sector("PLN sistem terganggu"),
            Some("ENERGY".to_string())
        );

        // Test no sector
        assert_eq!(
            matcher.extract_sector("Generic organization incident"),
            None
        );
    }

    #[test]
    fn test_bssn_crawler_creation() {
        let crawler = BssnCrawler::new();
        assert!(crawler.is_ok());
        let crawler = crawler.unwrap();
        assert_eq!(crawler.name(), "BSSN");
        assert!(crawler.config().enabled);
        assert_eq!(crawler.config().base_url, "https://bssn.go.id");
    }

    #[test]
    fn test_org_name_extraction() {
        let crawler = BssnCrawler::new().unwrap();

        // Test PT company names
        let org_name =
            crawler.extract_org_name("PT Bank Central Asia Tbk mengalami serangan siber");
        assert_eq!(org_name, Some("PT Bank Central Asia Tbk".to_string()));

        // Test CV company names
        let org_name = crawler.extract_org_name("CV Teknologi Digital melaporkan insiden");
        assert_eq!(org_name, Some("CV Teknologi Digital".to_string()));

        // Test generic names
        let org_name = crawler.extract_org_name("Bank Indonesia laporan keamanan");
        assert_eq!(org_name, Some("Bank Indonesia laporan".to_string()));

        // Test no valid org name
        let org_name = crawler.extract_org_name("123 456 789");
        assert_eq!(org_name, None);
    }

    #[test]
    fn test_date_extraction() {
        let crawler = BssnCrawler::new().unwrap();

        // Test Indonesian date format
        let date = crawler.extract_date("15 April 2024");
        assert_eq!(date, NaiveDate::from_ymd_opt(2024, 4, 15).unwrap());

        // Test ISO date format
        let date = crawler.extract_date("2024-04-15");
        assert_eq!(date, NaiveDate::from_ymd_opt(2024, 4, 15).unwrap());

        // Test Indonesian month names
        let date = crawler.extract_date("15 Januari 2024");
        assert_eq!(date, NaiveDate::from_ymd_opt(2024, 1, 15).unwrap());

        // Test date in text
        let date = crawler.extract_date("Insiden terjadi pada 25 Desember 2023");
        assert_eq!(date, NaiveDate::from_ymd_opt(2023, 12, 25).unwrap());

        // Test fallback (should return current date)
        let date = crawler.extract_date("no date here");
        assert!(date <= chrono::Utc::now().date_naive());
    }

    #[test]
    fn test_org_name_validation() {
        let crawler = BssnCrawler::new().unwrap();

        // Valid names
        assert!(crawler.is_valid_org_name("PT Test Company"));
        assert!(crawler.is_valid_org_name("Bank Indonesia"));
        assert!(crawler.is_valid_org_name("CV Technology"));

        // Invalid names
        assert!(!crawler.is_valid_org_name("ABCD")); // Too short (less than 5 chars)
        assert!(!crawler.is_valid_org_name("12345")); // All numeric
        assert!(!crawler.is_valid_org_name("")); // Empty
    }

    #[test]
    fn test_incident_draft_creation() {
        let draft = IncidentDraft::new(
            "PT Bank BCA".to_string(),
            NaiveDate::from_ymd_opt(2024, 4, 15).unwrap(),
            "https://bssn.go.id/press-release/123".to_string(),
            "BSSN_PRESS_RELEASE".to_string(),
        )
        .with_org_sector(Some("FINANCIAL".to_string()))
        .with_attack_type(Some("RANSOMWARE".to_string()))
        .with_confidence(0.8)
        .with_notes(Some("Ransomware attack detected".to_string()));

        assert_eq!(draft.org_name, "PT Bank BCA");
        assert_eq!(draft.org_sector, Some("FINANCIAL".to_string()));
        assert_eq!(draft.attack_type, Some("RANSOMWARE".to_string()));
        assert_eq!(draft.confidence, 0.8);
        assert_eq!(draft.notes, Some("Ransomware attack detected".to_string()));
    }

    #[test]
    fn test_duplicate_detection() {
        let draft1 = IncidentDraft::new(
            "PT Bank ABC".to_string(),
            NaiveDate::from_ymd_opt(2024, 4, 15).unwrap(),
            "https://bssn.go.id/1".to_string(),
            "BSSN_PRESS_RELEASE".to_string(),
        );

        let draft2 = IncidentDraft::new(
            "PT Bank ABC".to_string(),
            NaiveDate::from_ymd_opt(2024, 4, 18).unwrap(), // 3 days later
            "https://bssn.go.id/2".to_string(),
            "BSSN_PRESS_RELEASE".to_string(),
        );

        let draft3 = IncidentDraft::new(
            "PT Bank XYZ".to_string(),
            NaiveDate::from_ymd_opt(2024, 4, 16).unwrap(),
            "https://bssn.go.id/3".to_string(),
            "BSSN_PRESS_RELEASE".to_string(),
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
        <div class="news-item">
            <a href="/press-release/123">
                <h3>Serangan Siber Terhadap Bank Nasional</h3>
            </a>
            <span class="date">15 April 2024</span>
            <p>PT Bank Nasional mengalami serangan ransomware yang mengganggu sistem perbankan.</p>
        </div>
        "#;

        let crawler = BssnCrawler::new().unwrap();

        // Test that we can parse HTML without crashing
        let items = crawler.parse_press_releases(html);
        assert!(items.is_ok());
    }

    #[test]
    fn test_pdf_link_extraction() {
        let html = r#"
        <html>
            <body>
                <a href="/reports/threat-landscape-2023.pdf">Threat Landscape 2023</a>
                <a href="https://bssn.go.id/reports/annual-report.pdf">Annual Report</a>
                <a href="/news/regular-news">Regular News</a>
            </body>
        </html>
        "#;

        let crawler = BssnCrawler::new().unwrap();
        let pdf_links = crawler.extract_pdf_links(html);

        assert_eq!(pdf_links.len(), 2);
        assert!(
            pdf_links
                .iter()
                .any(|link: &String| link.contains("threat-landscape-2023.pdf"))
        );
        assert!(
            pdf_links
                .iter()
                .any(|link: &String| link.contains("annual-report.pdf"))
        );
    }

    #[tokio::test]
    async fn test_crawler_crawl_interface() {
        let crawler = BssnCrawler::new().unwrap();

        // Test that the crawl method exists and returns the right type
        // Note: This test doesn't actually crawl since it would require network access
        // We just verify the method signature is correct
        std::mem::drop(crawler.crawl());
    }

    #[test]
    fn test_incident_draft_to_extraction_result() {
        let draft = IncidentDraft::new(
            "PT Test Company".to_string(),
            NaiveDate::from_ymd_opt(2024, 4, 15).unwrap(),
            "https://bssn.go.id/press-release/123".to_string(),
            "BSSN_PRESS_RELEASE".to_string(),
        )
        .with_org_sector(Some("FINANCIAL".to_string()))
        .with_attack_type(Some("RANSOMWARE".to_string()))
        .with_confidence(0.8);

        let extraction_result = draft.to_extraction_result();

        assert_eq!(extraction_result.org_name, "PT Test Company");
        assert_eq!(extraction_result.org_sector, "FINANCIAL");
        assert_eq!(extraction_result.attack_type, "RANSOMWARE");
        assert_eq!(extraction_result.confidence, 0.8);
        assert_eq!(extraction_result.source_type, "BSSN_PRESS_RELEASE");
    }

    #[test]
    fn test_aggregate_vs_named_incidents() {
        let matcher = BssnKeywordMatcher::new();

        // Named incident (contains org name)
        let named_text = "PT Bank ABC mengalami serangan ransomware pada 15 April 2024";
        assert!(matcher.contains_cyber_keywords(named_text));

        // Aggregate statistics (no specific org name)
        let aggregate_text = "Pada tahun 2023, terdapat 500 insiden siber di sektor keuangan";
        assert!(matcher.contains_cyber_keywords(aggregate_text));

        // The crawler should handle both types through org_name extraction
        // Named incidents will have org_name set, aggregate will default to "BSSN Report"
    }

    #[test]
    fn test_sector_keyword_coverage() {
        let matcher = BssnKeywordMatcher::new();

        // Test all major sectors are covered
        assert!(matcher.extract_sector("perbankan").is_some());
        assert!(matcher.extract_sector("kesehatan").is_some());
        assert!(matcher.extract_sector("pemerintah").is_some());
        assert!(matcher.extract_sector("pendidikan").is_some());
        assert!(matcher.extract_sector("telekomunikasi").is_some());
        assert!(matcher.extract_sector("e-commerce").is_some());
        assert!(matcher.extract_sector("energi").is_some());
    }
}
