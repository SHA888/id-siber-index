//! Schema enums for the Indonesia Cybersecurity Incident Index

use serde::{Deserialize, Serialize};

/// Organization sectors for classification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Sector {
    // Government & Public Sector
    Government,
    Military,
    LawEnforcement,
    Healthcare,
    Education,
    PublicServices,

    // Financial Services
    Banking,
    Insurance,
    FinancialServices,
    Fintech,

    // Technology & Communications
    Technology,
    Telecommunications,
    InternetServices,
    Software,
    Hardware,

    // Critical Infrastructure
    Energy,
    Transportation,
    WaterUtilities,
    Manufacturing,
    Agriculture,

    // Commercial
    Retail,
    Ecommerce,
    Media,
    Entertainment,
    Hospitality,

    // Professional Services
    Consulting,
    Legal,
    Accounting,
    RealEstate,

    // Other
    NonProfit,
    Other,
    Unknown,
}

/// Types of cyber attacks
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AttackType {
    // Malware
    Ransomware,
    Malware,
    Spyware,
    Adware,
    Trojan,
    Worm,
    Rootkit,

    // Social Engineering
    Phishing,
    SpearPhishing,
    Whaling,
    Smishing,
    Vishing,
    SocialEngineering,

    // Network Attacks
    Ddos,
    ManInTheMiddle,
    Sniffing,
    Spoofing,
    Hijacking,

    // Web Application Attacks
    SqlInjection,
    Xss,
    Csrf,
    PathTraversal,
    FileInclusion,
    CommandInjection,

    // Data Breaches
    DataBreach,
    DataExfiltration,
    DataLeakage,

    // System Attacks
    PrivilegeEscalation,
    RemoteCodeExecution,
    BufferOverflow,
    ZeroDay,

    // Physical/Insider
    InsiderThreat,
    PhysicalTheft,
    Sabotage,

    // Other
    Unknown,
    Other,
}

/// Types of data sources
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SourceType {
    // Official Sources
    GovernmentReport,
    RegulatoryFiling,
    PressRelease,

    // Media Sources
    NewsArticle,
    BlogPost,
    SocialMedia,

    // Security Sources
    SecurityReport,
    VendorReport,
    ResearchReport,

    // Company Sources
    CompanyStatement,
    InvestorReport,
    InternalReport,

    // Community Sources
    CommunityReport,
    UserSubmission,

    // Other
    Other,
    Unknown,
}

/// Categories of data affected
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DataCategory {
    // Personal Information
    PersonalData,
    ContactInformation,
    Identification,
    FinancialInformation,
    HealthInformation,

    // Business Information
    BusinessData,
    IntellectualProperty,
    TradeSecrets,
    FinancialData,
    CustomerData,
    EmployeeData,

    // System Information
    SystemData,
    Credentials,
    AccessTokens,
    ConfigurationData,

    // Other
    Other,
    Unknown,
}

impl Sector {
    /// Get all valid sector values
    pub fn all_values() -> Vec<&'static str> {
        vec![
            "GOVERNMENT",
            "MILITARY",
            "LAW_ENFORCEMENT",
            "HEALTHCARE",
            "EDUCATION",
            "PUBLIC_SERVICES",
            "BANKING",
            "INSURANCE",
            "FINANCIAL_SERVICES",
            "FINTECH",
            "TECHNOLOGY",
            "TELECOMMUNICATIONS",
            "INTERNET_SERVICES",
            "SOFTWARE",
            "HARDWARE",
            "ENERGY",
            "TRANSPORTATION",
            "WATER_UTILITIES",
            "MANUFACTURING",
            "AGRICULTURE",
            "RETAIL",
            "ECOMMERCE",
            "MEDIA",
            "ENTERTAINMENT",
            "HOSPITALITY",
            "CONSULTING",
            "LEGAL",
            "ACCOUNTING",
            "REAL_ESTATE",
            "NON_PROFIT",
            "OTHER",
            "UNKNOWN",
        ]
    }
}

impl AttackType {
    /// Get all valid attack type values
    pub fn all_values() -> Vec<&'static str> {
        vec![
            "RANSOMWARE",
            "MALWARE",
            "SPYWARE",
            "ADWARE",
            "TROJAN",
            "WORM",
            "ROOTKIT",
            "PHISHING",
            "SPEAR_PHISHING",
            "WHALING",
            "SMISHING",
            "VISHING",
            "SOCIAL_ENGINEERING",
            "DDOS",
            "MAN_IN_THE_MIDDLE",
            "SNIFFING",
            "SPOOFING",
            "HIJACKING",
            "SQL_INJECTION",
            "XSS",
            "CSRF",
            "PATH_TRAVERSAL",
            "FILE_INCLUSION",
            "COMMAND_INJECTION",
            "DATA_BREACH",
            "DATA_EXFILTRATION",
            "DATA_LEAKAGE",
            "PRIVILEGE_ESCALATION",
            "REMOTE_CODE_EXECUTION",
            "BUFFER_OVERFLOW",
            "ZERO_DAY",
            "INSIDER_THREAT",
            "PHYSICAL_THEFT",
            "SABOTAGE",
            "UNKNOWN",
            "OTHER",
        ]
    }
}

impl SourceType {
    /// Get all valid source type values
    pub fn all_values() -> Vec<&'static str> {
        vec![
            "GOVERNMENT_REPORT",
            "REGULATORY_FILING",
            "PRESS_RELEASE",
            "NEWS_ARTICLE",
            "BLOG_POST",
            "SOCIAL_MEDIA",
            "SECURITY_REPORT",
            "VENDOR_REPORT",
            "RESEARCH_REPORT",
            "COMPANY_STATEMENT",
            "INVESTOR_REPORT",
            "INTERNAL_REPORT",
            "COMMUNITY_REPORT",
            "USER_SUBMISSION",
            "OTHER",
            "UNKNOWN",
        ]
    }
}

impl DataCategory {
    /// Get all valid data category values
    pub fn all_values() -> Vec<&'static str> {
        vec![
            "PERSONAL_DATA",
            "CONTACT_INFORMATION",
            "IDENTIFICATION",
            "FINANCIAL_INFORMATION",
            "HEALTH_INFORMATION",
            "BUSINESS_DATA",
            "INTELLECTUAL_PROPERTY",
            "TRADE_SECRETS",
            "FINANCIAL_DATA",
            "CUSTOMER_DATA",
            "EMPLOYEE_DATA",
            "SYSTEM_DATA",
            "CREDENTIALS",
            "ACCESS_TOKENS",
            "CONFIGURATION_DATA",
            "OTHER",
            "UNKNOWN",
        ]
    }
}
