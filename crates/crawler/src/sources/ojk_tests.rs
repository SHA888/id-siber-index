//! Unit tests for OJK crawler

use crate::incident_draft::IncidentDraft;
use crate::sources::CrawlerSource;
use crate::sources::ojk::{OjkCrawler, OjkKeywordMatcher};
use chrono::NaiveDate;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ojk_keyword_matcher_bahasa() {
        let matcher = OjkKeywordMatcher::new();

        // Test Bahasa cyber keywords
        assert!(matcher.contains_cyber_keywords("Serangan siber terhadap sistem perbankan"));
        assert!(matcher.contains_cyber_keywords("Terjadi kebocoran data nasabah"));
        assert!(matcher.contains_cyber_keywords("Sistem terkena ransomware"));
        assert!(matcher.contains_cyber_keywords("Insiden keamanan siber di Bank ABC"));

        // Test Bahasa fraud keywords
        assert!(matcher.contains_cyber_keywords("Kasus penipuan investasi online"));
        assert!(matcher.contains_cyber_keywords("Ditemukan skimming ATM di beberapa lokasi"));
        assert!(matcher.contains_cyber_keywords("Pencucian uang melalui rekening fiktif"));
        assert!(matcher.contains_cyber_keywords("Account takeover pada nasabah Bank XYZ"));

        // Test non-cyber/non-fraud content
        assert!(!matcher.contains_cyber_keywords("Laporan kinerja perbankan triwulanan"));
        assert!(!matcher.contains_cyber_keywords("Rapat koordinasi industri keuangan"));
    }

    #[test]
    fn test_ojk_keyword_matcher_english() {
        let matcher = OjkKeywordMatcher::new();

        // Test English cyber keywords
        assert!(matcher.contains_cyber_keywords("Cyber attack against banking system"));
        assert!(matcher.contains_cyber_keywords("Data breach affecting customer accounts"));
        assert!(matcher.contains_cyber_keywords("Ransomware detected in financial network"));

        // Test English fraud keywords
        assert!(matcher.contains_cyber_keywords("Banking fraud investigation results"));
        assert!(matcher.contains_cyber_keywords("Identity theft reported by customers"));
        assert!(matcher.contains_cyber_keywords("Card skimming at multiple ATMs"));
        assert!(matcher.contains_cyber_keywords("Unauthorized transactions detected"));
        assert!(matcher.contains_cyber_keywords("Social engineering attacks on bank staff"));

        // Test non-cyber/non-fraud content
        assert!(!matcher.contains_cyber_keywords("Quarterly financial performance report"));
        assert!(!matcher.contains_cyber_keywords("Annual banking industry statistics"));
    }

    #[test]
    fn test_attack_type_extraction() {
        let matcher = OjkKeywordMatcher::new();

        // Test cyber attack types
        assert_eq!(
            matcher.extract_attack_type("Sistem terkena ransomware"),
            Some("RANSOMWARE".to_string())
        );
        assert_eq!(
            matcher.extract_attack_type("Kejadian kebocoran data nasabah"),
            Some("DATA_BREACH".to_string())
        );
        assert_eq!(
            matcher.extract_attack_type("Serangan phishing terhadap karyawan"),
            Some("PHISHING".to_string())
        );

        // Test fraud types
        assert_eq!(
            matcher.extract_attack_type("Kasus penipuan investasi bodong"),
            Some("FRAUD".to_string())
        );
        assert_eq!(
            matcher.extract_attack_type("Ditemukan pencucian uang"),
            Some("MONEY_LAUNDERING".to_string())
        );
        assert_eq!(
            matcher.extract_attack_type("Skimming kartu ATM"),
            Some("SKIMMING".to_string())
        );
        assert_eq!(
            matcher.extract_attack_type("Account takeover pada nasabah"),
            Some("ACCOUNT_TAKEOVER".to_string())
        );
    }

    #[test]
    fn test_fraud_type_extraction() {
        let matcher = OjkKeywordMatcher::new();

        assert_eq!(
            matcher.extract_fraud_type("Penipuan investasi berkedok crypto"),
            Some("INVESTMENT_FRAUD".to_string())
        );
        assert_eq!(
            matcher.extract_fraud_type("Skimming kartu kredit"),
            Some("CARD_SKIMMING".to_string())
        );
        // "Phishing untuk pencurian identitas" - both PHISHING and IDENTITY_THEFT match
        // HashMap iteration order is not deterministic, so check it's one of the valid types
        let fraud_type = matcher.extract_fraud_type("Phishing untuk pencurian identitas");
        assert!(fraud_type.is_some());
        let ft = fraud_type.unwrap();
        assert!(ft == "PHISHING" || ft == "IDENTITY_THEFT");
        assert_eq!(
            matcher.extract_fraud_type("Identity theft melalui social engineering"),
            Some("IDENTITY_THEFT".to_string())
        );
    }

    #[test]
    fn test_is_financial_fraud() {
        let matcher = OjkKeywordMatcher::new();

        assert!(matcher.is_financial_fraud("Kasus penipuan online"));
        assert!(matcher.is_financial_fraud("Money laundering investigation"));
        assert!(matcher.is_financial_fraud("Skimming ATM detection"));
        assert!(matcher.is_financial_fraud("Carding activities detected"));
        assert!(matcher.is_financial_fraud("Unauthorized transaction complaints"));

        assert!(!matcher.is_financial_fraud("Ransomware attack on servers"));
        assert!(!matcher.is_financial_fraud("Data breach notification"));
    }

    #[test]
    fn test_ojk_crawler_creation() {
        let crawler = OjkCrawler::new();
        assert!(crawler.is_ok());
        let crawler = crawler.unwrap();
        assert_eq!(crawler.name(), "OJK");
        assert!(crawler.config().enabled);
        assert_eq!(crawler.config().base_url, "https://ojk.go.id");
    }

    #[test]
    fn test_org_name_extraction() {
        let crawler = OjkCrawler::new().unwrap();

        // Test bank names with PT pattern
        let org_name =
            crawler.extract_org_name("PT Bank Central Asia Tbk mengalami serangan siber");
        assert!(org_name.is_some());
        assert!(org_name.as_ref().unwrap().contains("Bank"));
        assert!(org_name.as_ref().unwrap().contains("Tbk"));

        // Test PT company names with financial keywords
        let org_name = crawler.extract_org_name("PT Bank Mandiri mengalami kebocoran data");
        assert!(org_name.is_some());
        assert!(org_name.unwrap().contains("Bank"));

        // Test insurance company
        let org_name = crawler.extract_org_name("PT Asuransi Jiwa Sejahtera laporan insiden");
        assert!(org_name.is_some());
        assert!(org_name.unwrap().contains("Asuransi"));

        // Test fintech company
        let org_name = crawler.extract_org_name("PT Fintech Digital Indonesia fraud detection");
        assert!(org_name.is_some());

        // Test no valid org name (no financial keywords)
        let org_name = crawler.extract_org_name("123 456 789");
        assert_eq!(org_name, None);
    }

    #[test]
    fn test_date_extraction() {
        let crawler = OjkCrawler::new().unwrap();

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
        let crawler = OjkCrawler::new().unwrap();

        // Valid names
        assert!(crawler.is_valid_org_name("PT Test Company"));
        assert!(crawler.is_valid_org_name("Bank Indonesia"));
        assert!(crawler.is_valid_org_name("PT Asuransi Jiwa"));

        // Invalid names
        assert!(!crawler.is_valid_org_name("AB")); // Too short
        assert!(!crawler.is_valid_org_name("12345")); // All numeric
        assert!(!crawler.is_valid_org_name("")); // Empty
        assert!(!crawler.is_valid_org_name("Test")); // Single word
    }

    #[test]
    fn test_incident_draft_creation() {
        let draft = IncidentDraft::new(
            "PT Bank BCA".to_string(),
            NaiveDate::from_ymd_opt(2024, 4, 15).unwrap(),
            "https://ojk.go.id/enforcement/123".to_string(),
            "OJK_ENFORCEMENT".to_string(),
        )
        .with_org_sector(Some("FINANCIAL".to_string()))
        .with_attack_type(Some("FRAUD".to_string()))
        .with_confidence(0.8)
        .with_notes(Some("Investment fraud investigation".to_string()));

        assert_eq!(draft.org_name, "PT Bank BCA");
        assert_eq!(draft.org_sector, Some("FINANCIAL".to_string()));
        assert_eq!(draft.attack_type, Some("FRAUD".to_string()));
        assert_eq!(draft.confidence, 0.8);
        assert_eq!(
            draft.notes,
            Some("Investment fraud investigation".to_string())
        );
    }

    #[test]
    fn test_duplicate_detection() {
        let draft1 = IncidentDraft::new(
            "PT Bank ABC".to_string(),
            NaiveDate::from_ymd_opt(2024, 4, 15).unwrap(),
            "https://ojk.go.id/1".to_string(),
            "OJK_ENFORCEMENT".to_string(),
        );

        let draft2 = IncidentDraft::new(
            "PT Bank ABC".to_string(),
            NaiveDate::from_ymd_opt(2024, 4, 18).unwrap(), // 3 days later
            "https://ojk.go.id/2".to_string(),
            "OJK_ENFORCEMENT".to_string(),
        );

        let draft3 = IncidentDraft::new(
            "PT Bank XYZ".to_string(),
            NaiveDate::from_ymd_opt(2024, 4, 16).unwrap(),
            "https://ojk.go.id/3".to_string(),
            "OJK_ENFORCEMENT".to_string(),
        );

        // Test duplicate detection (7-day window)
        assert!(draft1.is_potential_duplicate(&draft2, 7)); // Same org, 3 days apart
        assert!(!draft1.is_potential_duplicate(&draft3, 7)); // Different org

        // Test with smaller window
        assert!(!draft1.is_potential_duplicate(&draft2, 2)); // Same org, but 3 days apart > 2-day window
    }

    #[test]
    fn test_html_parsing_enforcement() {
        let html = r#"
        <div class="news-item">
            <a href="/enforcement/123">
                <h3>Penindakan atas Kasus Kejahatan Siber di Sektor Perbankan</h3>
            </a>
            <span class="date">15 April 2024</span>
            <p>PT Bank Nasional mengalami serangan ransomware yang mengganggu sistem perbankan. OJK melakukan penindakan tegas.</p>
        </div>
        "#;

        let crawler = OjkCrawler::new().unwrap();

        // Test that we can parse HTML without crashing
        let items = crawler.parse_enforcement_items(html);
        assert!(items.is_ok());
    }

    #[test]
    fn test_html_parsing_complaint() {
        let html = r#"
        <div class="complaint-item">
            <a href="/complaint/456">
                <h3>Laporan Pengaduan Nasabah tentang Penipuan Online</h3>
            </a>
            <span class="date">20 Maret 2024</span>
            <p>Nasabah melaporkan kasus phishing yang mengakibatkan kerugian finansial. Bank XYZ sedang menangani kasus ini.</p>
        </div>
        "#;

        let crawler = OjkCrawler::new().unwrap();

        // Test that we can parse HTML without crashing
        let items = crawler.parse_complaint_items(html);
        assert!(items.is_ok());
    }

    #[test]
    fn test_incident_draft_to_extraction_result() {
        let draft = IncidentDraft::new(
            "PT Bank Test".to_string(),
            NaiveDate::from_ymd_opt(2024, 4, 15).unwrap(),
            "https://ojk.go.id/enforcement/123".to_string(),
            "OJK_ENFORCEMENT".to_string(),
        )
        .with_org_sector(Some("FINANCIAL".to_string()))
        .with_attack_type(Some("FRAUD".to_string()))
        .with_confidence(0.8);

        let extraction_result = draft.to_extraction_result();

        assert_eq!(extraction_result.org_name, "PT Bank Test");
        assert_eq!(extraction_result.org_sector, "FINANCIAL");
        assert_eq!(extraction_result.attack_type, "FRAUD");
        assert_eq!(extraction_result.confidence, 0.8);
        assert_eq!(extraction_result.source_type, "OJK_ENFORCEMENT");
    }

    #[test]
    fn test_link_to_idx_disclosures() {
        let crawler = OjkCrawler::new().unwrap();

        // Create mock OJK results
        let ojk_results = vec![crate::extractors::ExtractionResult {
            org_name: "PT Bank Central Asia Tbk".to_string(),
            org_sector: "FINANCIAL".to_string(),
            incident_date: NaiveDate::from_ymd_opt(2024, 4, 15).unwrap(),
            disclosure_date: NaiveDate::from_ymd_opt(2024, 4, 15).unwrap(),
            attack_type: "FRAUD".to_string(),
            data_categories: vec![],
            record_count_estimate: None,
            financial_impact_idr: None,
            actor_alias: None,
            actor_group: None,
            source_url: "https://ojk.go.id/enforcement/123".to_string(),
            source_type: "OJK_ENFORCEMENT".to_string(),
            notes: None,
            confidence: 0.8,
        }];

        // Create mock IDX results with matching organization
        let idx_results = vec![crate::extractors::ExtractionResult {
            org_name: "PT Bank Central Asia Tbk".to_string(),
            org_sector: "FINANCIAL".to_string(),
            incident_date: NaiveDate::from_ymd_opt(2024, 4, 15).unwrap(),
            disclosure_date: NaiveDate::from_ymd_opt(2024, 4, 15).unwrap(),
            attack_type: "DATA_BREACH".to_string(),
            data_categories: vec!["PERSONAL_DATA".to_string()],
            record_count_estimate: Some(10000),
            financial_impact_idr: None,
            actor_alias: None,
            actor_group: None,
            source_url: "https://idx.co.id/announcement/456".to_string(),
            source_type: "IDX_DISCLOSURE".to_string(),
            notes: None,
            confidence: 0.9,
        }];

        // Test linking
        let links = crawler.link_to_idx_disclosures(&ojk_results, &idx_results);
        assert!(!links.is_empty());
        assert!(links.contains_key("https://ojk.go.id/enforcement/123"));

        let linked_urls = links.get("https://ojk.go.id/enforcement/123").unwrap();
        assert_eq!(linked_urls.len(), 1);
        assert_eq!(linked_urls[0], "https://idx.co.id/announcement/456");
    }

    #[test]
    fn test_partial_org_name_matching() {
        let crawler = OjkCrawler::new().unwrap();

        // Test "Bank BCA" matches with "PT Bank Central Asia Tbk"
        let ojk_results = vec![crate::extractors::ExtractionResult {
            org_name: "Bank BCA".to_string(),
            org_sector: "FINANCIAL".to_string(),
            incident_date: NaiveDate::from_ymd_opt(2024, 4, 15).unwrap(),
            disclosure_date: NaiveDate::from_ymd_opt(2024, 4, 15).unwrap(),
            attack_type: "FRAUD".to_string(),
            data_categories: vec![],
            record_count_estimate: None,
            financial_impact_idr: None,
            actor_alias: None,
            actor_group: None,
            source_url: "https://ojk.go.id/enforcement/123".to_string(),
            source_type: "OJK_ENFORCEMENT".to_string(),
            notes: None,
            confidence: 0.8,
        }];

        let idx_results = vec![crate::extractors::ExtractionResult {
            org_name: "PT Bank Central Asia Tbk".to_string(),
            org_sector: "FINANCIAL".to_string(),
            incident_date: NaiveDate::from_ymd_opt(2024, 4, 15).unwrap(),
            disclosure_date: NaiveDate::from_ymd_opt(2024, 4, 15).unwrap(),
            attack_type: "DATA_BREACH".to_string(),
            data_categories: vec!["PERSONAL_DATA".to_string()],
            record_count_estimate: Some(10000),
            financial_impact_idr: None,
            actor_alias: None,
            actor_group: None,
            source_url: "https://idx.co.id/announcement/456".to_string(),
            source_type: "IDX_DISCLOSURE".to_string(),
            notes: None,
            confidence: 0.9,
        }];

        let links = crawler.link_to_idx_disclosures(&ojk_results, &idx_results);
        // Should match based on partial overlap ("Bank" and "BCA")
        assert!(!links.is_empty());
    }

    #[tokio::test]
    async fn test_crawler_crawl_interface() {
        let crawler = OjkCrawler::new().unwrap();

        // Test that the crawl method exists and returns the right type
        // Note: This test doesn't actually crawl since it would require network access
        // We just verify the method signature is correct
        std::mem::drop(crawler.crawl());
    }

    #[test]
    fn test_sector_extraction() {
        let matcher = OjkKeywordMatcher::new();

        // Test banking sector
        assert_eq!(
            matcher.extract_sector("Insiden di sektor perbankan"),
            Some("BANKING".to_string())
        );
        assert_eq!(
            matcher.extract_sector("Bank ABC mengalami serangan"),
            Some("BANKING".to_string())
        );

        // Test insurance sector
        assert_eq!(
            matcher.extract_sector("Asuransi Jiwa data breach"),
            Some("INSURANCE".to_string())
        );

        // Test securities sector
        assert_eq!(
            matcher.extract_sector("PT Sekuritas XYZ fraud case"),
            Some("SECURITIES".to_string())
        );

        // Test fintech sector
        assert_eq!(
            matcher.extract_sector("Fintech company security incident"),
            Some("FINTECH".to_string())
        );

        // Test no sector match
        assert_eq!(
            matcher.extract_sector("Generic organization incident"),
            None
        );
    }

    #[test]
    fn test_data_categories_extraction() {
        let matcher = OjkKeywordMatcher::new();

        // Test personal data detection
        let categories =
            matcher.extract_data_categories("Kebocoran data nasabah dan informasi pribadi");
        assert!(categories.contains(&"PERSONAL_DATA".to_string()));

        // Test financial data detection
        let categories =
            matcher.extract_data_categories("Data kartu kredit dan informasi rekening bocor");
        assert!(categories.contains(&"FINANCIAL_DATA".to_string()));

        // Test transaction data detection
        let categories = matcher.extract_data_categories("Riwayat transaksi nasabah terungkap");
        assert!(categories.contains(&"TRANSACTION_DATA".to_string()));

        // Test credentials detection
        let categories = matcher.extract_data_categories("Password dan OTP nasabah dicuri");
        assert!(categories.contains(&"CREDENTIALS".to_string()));

        // Test multiple categories
        let categories = matcher.extract_data_categories(
            "Data pribadi nasabah, nomor kartu kredit, dan password bocor",
        );
        assert!(categories.contains(&"PERSONAL_DATA".to_string()));
        assert!(categories.contains(&"FINANCIAL_DATA".to_string()));
        assert!(categories.contains(&"CREDENTIALS".to_string()));
    }

    #[test]
    fn test_fraud_keyword_coverage() {
        let matcher = OjkKeywordMatcher::new();

        // Test all major fraud types are covered
        assert!(matcher.contains_cyber_keywords("penipuan"));
        assert!(matcher.contains_cyber_keywords("fraud"));
        assert!(matcher.contains_cyber_keywords("pencucian uang"));
        assert!(matcher.contains_cyber_keywords("skimming"));
        assert!(matcher.contains_cyber_keywords("carding"));
        assert!(matcher.contains_cyber_keywords("account takeover"));
        assert!(matcher.contains_cyber_keywords("social engineering"));
    }

    #[test]
    fn test_get_linked_disclosures() {
        let crawler = OjkCrawler::new().unwrap();

        let ojk_result = crate::extractors::ExtractionResult {
            org_name: "PT Bank Mandiri".to_string(),
            org_sector: "FINANCIAL".to_string(),
            incident_date: NaiveDate::from_ymd_opt(2024, 4, 15).unwrap(),
            disclosure_date: NaiveDate::from_ymd_opt(2024, 4, 15).unwrap(),
            attack_type: "FRAUD".to_string(),
            data_categories: vec![],
            record_count_estimate: None,
            financial_impact_idr: None,
            actor_alias: None,
            actor_group: None,
            source_url: "https://ojk.go.id/enforcement/789".to_string(),
            source_type: "OJK_ENFORCEMENT".to_string(),
            notes: None,
            confidence: 0.8,
        };

        let idx_results = vec![crate::extractors::ExtractionResult {
            org_name: "PT Bank Mandiri".to_string(),
            org_sector: "FINANCIAL".to_string(),
            incident_date: NaiveDate::from_ymd_opt(2024, 4, 10).unwrap(),
            disclosure_date: NaiveDate::from_ymd_opt(2024, 4, 10).unwrap(),
            attack_type: "DATA_BREACH".to_string(),
            data_categories: vec![],
            record_count_estimate: None,
            financial_impact_idr: None,
            actor_alias: None,
            actor_group: None,
            source_url: "https://idx.co.id/announcement/999".to_string(),
            source_type: "IDX_DISCLOSURE".to_string(),
            notes: None,
            confidence: 0.9,
        }];

        let linked = crawler.get_linked_disclosures(&ojk_result, &idx_results);
        assert_eq!(linked.len(), 1);
        assert_eq!(linked[0], "https://idx.co.id/announcement/999");
    }
}
