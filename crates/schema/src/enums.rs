//! Schema enums for the Indonesia Cybersecurity Incident Index

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// Organization sectors for classification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Sector {
    // Government & Public Sector
    #[sea_orm(string_value = "GOVERNMENT")]
    Government,
    #[sea_orm(string_value = "MILITARY")]
    Military,
    #[sea_orm(string_value = "LAW_ENFORCEMENT")]
    LawEnforcement,
    #[sea_orm(string_value = "HEALTHCARE")]
    Healthcare,
    #[sea_orm(string_value = "EDUCATION")]
    Education,
    #[sea_orm(string_value = "PUBLIC_SERVICES")]
    PublicServices,

    // Financial Services
    #[sea_orm(string_value = "BANKING")]
    Banking,
    #[sea_orm(string_value = "INSURANCE")]
    Insurance,
    #[sea_orm(string_value = "FINANCIAL_SERVICES")]
    FinancialServices,
    #[sea_orm(string_value = "FINTECH")]
    Fintech,

    // Technology & Communications
    #[sea_orm(string_value = "TECHNOLOGY")]
    Technology,
    #[sea_orm(string_value = "TELECOMMUNICATIONS")]
    Telecommunications,
    #[sea_orm(string_value = "INTERNET_SERVICES")]
    InternetServices,
    #[sea_orm(string_value = "SOFTWARE")]
    Software,
    #[sea_orm(string_value = "HARDWARE")]
    Hardware,

    // Critical Infrastructure
    #[sea_orm(string_value = "ENERGY")]
    Energy,
    #[sea_orm(string_value = "TRANSPORTATION")]
    Transportation,
    #[sea_orm(string_value = "WATER_UTILITIES")]
    WaterUtilities,
    #[sea_orm(string_value = "MANUFACTURING")]
    Manufacturing,
    #[sea_orm(string_value = "AGRICULTURE")]
    Agriculture,

    // Commercial
    #[sea_orm(string_value = "RETAIL")]
    Retail,
    #[sea_orm(string_value = "ECOMMERCE")]
    Ecommerce,
    #[sea_orm(string_value = "MEDIA")]
    Media,
    #[sea_orm(string_value = "ENTERTAINMENT")]
    Entertainment,
    #[sea_orm(string_value = "HOSPITALITY")]
    Hospitality,

    // Professional Services
    #[sea_orm(string_value = "CONSULTING")]
    Consulting,
    #[sea_orm(string_value = "LEGAL")]
    Legal,
    #[sea_orm(string_value = "ACCOUNTING")]
    Accounting,
    #[sea_orm(string_value = "REAL_ESTATE")]
    RealEstate,

    // Other
    #[sea_orm(string_value = "NON_PROFIT")]
    NonProfit,
    #[sea_orm(string_value = "OTHER")]
    Other,
    #[sea_orm(string_value = "UNKNOWN")]
    Unknown,
}

/// Types of cyber attacks
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AttackType {
    // Malware
    #[sea_orm(string_value = "RANSOMWARE")]
    Ransomware,
    #[sea_orm(string_value = "MALWARE")]
    Malware,
    #[sea_orm(string_value = "SPYWARE")]
    Spyware,
    #[sea_orm(string_value = "ADWARE")]
    Adware,
    #[sea_orm(string_value = "TROJAN")]
    Trojan,
    #[sea_orm(string_value = "WORM")]
    Worm,
    #[sea_orm(string_value = "ROOTKIT")]
    Rootkit,

    // Social Engineering
    #[sea_orm(string_value = "PHISHING")]
    Phishing,
    #[sea_orm(string_value = "SPEAR_PHISHING")]
    SpearPhishing,
    #[sea_orm(string_value = "WHALING")]
    Whaling,
    #[sea_orm(string_value = "SMISHING")]
    Smishing,
    #[sea_orm(string_value = "VISHING")]
    Vishing,
    #[sea_orm(string_value = "SOCIAL_ENGINEERING")]
    SocialEngineering,

    // Network Attacks
    #[sea_orm(string_value = "DDOS")]
    Ddos,
    #[sea_orm(string_value = "MAN_IN_THE_MIDDLE")]
    ManInTheMiddle,
    #[sea_orm(string_value = "SNIFFING")]
    Sniffing,
    #[sea_orm(string_value = "SPOOFING")]
    Spoofing,
    #[sea_orm(string_value = "HIJACKING")]
    Hijacking,

    // Web Application Attacks
    #[sea_orm(string_value = "SQL_INJECTION")]
    SqlInjection,
    #[sea_orm(string_value = "XSS")]
    Xss,
    #[sea_orm(string_value = "CSRF")]
    Csrf,
    #[sea_orm(string_value = "PATH_TRAVERSAL")]
    PathTraversal,
    #[sea_orm(string_value = "FILE_INCLUSION")]
    FileInclusion,
    #[sea_orm(string_value = "COMMAND_INJECTION")]
    CommandInjection,

    // Data Breaches
    #[sea_orm(string_value = "DATA_BREACH")]
    DataBreach,
    #[sea_orm(string_value = "DATA_EXFILTRATION")]
    DataExfiltration,
    #[sea_orm(string_value = "DATA_LEAKAGE")]
    DataLeakage,

    // System Attacks
    #[sea_orm(string_value = "PRIVILEGE_ESCALATION")]
    PrivilegeEscalation,
    #[sea_orm(string_value = "REMOTE_CODE_EXECUTION")]
    RemoteCodeExecution,
    #[sea_orm(string_value = "BUFFER_OVERFLOW")]
    BufferOverflow,
    #[sea_orm(string_value = "ZERO_DAY")]
    ZeroDay,

    // Physical/Insider
    #[sea_orm(string_value = "INSIDER_THREAT")]
    InsiderThreat,
    #[sea_orm(string_value = "PHYSICAL_THEFT")]
    PhysicalTheft,
    #[sea_orm(string_value = "SABOTAGE")]
    Sabotage,

    // Other
    #[sea_orm(string_value = "UNKNOWN")]
    Unknown,
    #[sea_orm(string_value = "OTHER")]
    Other,
}

/// Types of data sources
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SourceType {
    // Official Sources
    #[sea_orm(string_value = "GOVERNMENT_REPORT")]
    GovernmentReport,
    #[sea_orm(string_value = "REGULATORY_FILING")]
    RegulatoryFiling,
    #[sea_orm(string_value = "PRESS_RELEASE")]
    PressRelease,

    // Media Sources
    #[sea_orm(string_value = "NEWS_ARTICLE")]
    NewsArticle,
    #[sea_orm(string_value = "BLOG_POST")]
    BlogPost,
    #[sea_orm(string_value = "SOCIAL_MEDIA")]
    SocialMedia,

    // Security Sources
    #[sea_orm(string_value = "SECURITY_REPORT")]
    SecurityReport,
    #[sea_orm(string_value = "VENDOR_REPORT")]
    VendorReport,
    #[sea_orm(string_value = "RESEARCH_REPORT")]
    ResearchReport,

    // Company Sources
    #[sea_orm(string_value = "COMPANY_STATEMENT")]
    CompanyStatement,
    #[sea_orm(string_value = "INVESTOR_REPORT")]
    InvestorReport,
    #[sea_orm(string_value = "INTERNAL_REPORT")]
    InternalReport,

    // Community Sources
    #[sea_orm(string_value = "COMMUNITY_REPORT")]
    CommunityReport,
    #[sea_orm(string_value = "USER_SUBMISSION")]
    UserSubmission,

    // Other
    #[sea_orm(string_value = "OTHER")]
    Other,
    #[sea_orm(string_value = "UNKNOWN")]
    Unknown,
}

/// Categories of data affected
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DataCategory {
    // Personal Information
    #[sea_orm(string_value = "PERSONAL_DATA")]
    PersonalData,
    #[sea_orm(string_value = "CONTACT_INFORMATION")]
    ContactInformation,
    #[sea_orm(string_value = "IDENTIFICATION")]
    Identification,
    #[sea_orm(string_value = "FINANCIAL_INFORMATION")]
    FinancialInformation,
    #[sea_orm(string_value = "HEALTH_INFORMATION")]
    HealthInformation,

    // Business Information
    #[sea_orm(string_value = "BUSINESS_DATA")]
    BusinessData,
    #[sea_orm(string_value = "INTELLECTUAL_PROPERTY")]
    IntellectualProperty,
    #[sea_orm(string_value = "TRADE_SECRETS")]
    TradeSecrets,
    #[sea_orm(string_value = "FINANCIAL_DATA")]
    FinancialData,
    #[sea_orm(string_value = "CUSTOMER_DATA")]
    CustomerData,
    #[sea_orm(string_value = "EMPLOYEE_DATA")]
    EmployeeData,

    // System Information
    #[sea_orm(string_value = "SYSTEM_DATA")]
    SystemData,
    #[sea_orm(string_value = "CREDENTIALS")]
    Credentials,
    #[sea_orm(string_value = "ACCESS_TOKENS")]
    AccessTokens,
    #[sea_orm(string_value = "CONFIGURATION_DATA")]
    ConfigurationData,

    // Other
    #[sea_orm(string_value = "OTHER")]
    Other,
    #[sea_orm(string_value = "UNKNOWN")]
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
