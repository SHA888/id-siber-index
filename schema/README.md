# Indonesia Cybersecurity Incident Index Schema

This document describes the data schema for the Indonesia Cybersecurity Incident Index (ID-Siber).

## Overview

The ID-Siber schema defines the structure for storing and querying cybersecurity incident data affecting Indonesian organizations. The schema is designed to support comprehensive analysis, search, and reporting capabilities.

## Core Entities

### Incident

The primary entity representing a cybersecurity incident.

#### Fields

| Field | Type | Description | Required |
|-------|------|-------------|----------|
| `id` | UUID | Unique identifier for the incident | Yes |
| `org_name` | String | Name of the affected organization | Yes |
| `org_sector` | Sector | Economic sector of the organization | Yes |
| `incident_date` | Date | Date when the incident occurred | Yes |
| `disclosure_date` | Date | Date when the incident was disclosed | Yes |
| `attack_type` | AttackType | Type of cyber attack | Yes |
| `data_categories` | JSON (Array of DataCategory) | Types of data affected | Yes |
| `record_count_estimate` | Integer (Optional) | Estimated number of records affected | No |
| `financial_impact_idr` | BigInt (Optional) | Financial impact in Indonesian Rupiah | No |
| `actor_alias` | String (Optional) | Alias of the threat actor | No |
| `actor_group` | String (Optional) | Name of the threat actor group | No |
| `source_url` | String | URL of the source information | Yes |
| `source_type` | SourceType | Type of source | Yes |
| `verified` | Boolean | Whether the incident has been verified | Yes |
| `notes` | Text (Optional) | Additional notes about the incident | No |
| `created_at` | Timestamp with Time Zone | When the record was created | Yes |
| `updated_at` | Timestamp with Time Zone | When the record was last updated | Yes |

### Source

Audit trail for incident sources.

#### Fields

| Field | Type | Description | Required |
|-------|------|-------------|----------|
| `id` | UUID | Unique identifier for the source | Yes |
| `incident_id` | UUID | Foreign key to incident | Yes |
| `source_type` | SourceType | Type of source | Yes |
| `source_url` | String | URL of the source | Yes |
| `title` | String (Optional) | Title of the source content | No |
| `author` | String (Optional) | Author of the source content | No |
| `publish_date` | Timestamp with Time Zone (Optional) | When the source was published | No |
| `extracted_at` | Timestamp with Time Zone | When the source was extracted | Yes |
| `verified` | Boolean | Whether the source has been verified | Yes |
| `notes` | Text (Optional) | Additional notes about the source | No |
| `created_at` | Timestamp with Time Zone | When the record was created | Yes |
| `updated_at` | Timestamp with Time Zone | When the record was last updated | Yes |

## Enums

### Sector

Economic sectors for organization classification.

#### Values

- **Government & Public Sector**
  - `GOVERNMENT` - Government agencies
  - `MILITARY` - Military organizations
  - `LAW_ENFORCEMENT` - Police and law enforcement
  - `HEALTHCARE` - Healthcare providers
  - `EDUCATION` - Educational institutions
  - `PUBLIC_SERVICES` - Public service providers

- **Financial Services**
  - `BANKING` - Banking institutions
  - `INSURANCE` - Insurance companies
  - `FINANCIAL_SERVICES` - Financial services providers
  - `FINTECH` - Financial technology companies

- **Technology & Communications**
  - `TECHNOLOGY` - Technology companies
  - `TELECOMMUNICATIONS` - Telecom providers
  - `INTERNET_SERVICES` - Internet service providers
  - `SOFTWARE` - Software companies
  - `HARDWARE` - Hardware manufacturers

- **Critical Infrastructure**
  - `ENERGY` - Energy sector (power, oil, gas)
  - `TRANSPORTATION` - Transportation infrastructure
  - `WATER_UTILITIES` - Water and waste management
  - `MANUFACTURING` - Manufacturing sector
  - `AGRICULTURE` - Agricultural sector

- **Commercial**
  - `RETAIL` - Retail businesses
  - `ECOMMERCE` - E-commerce platforms
  - `MEDIA` - Media organizations
  - `ENTERTAINMENT` - Entertainment industry
  - `HOSPITALITY` - Hospitality sector

- **Professional Services**
  - `CONSULTING` - Consulting firms
  - `LEGAL` - Legal services
  - `ACCOUNTING` - Accounting firms
  - `REAL_ESTATE` - Real estate companies

- **Other**
  - `NON_PROFIT` - Non-profit organizations
  - `OTHER` - Other sectors not listed
  - `UNKNOWN` - Unknown sector

### AttackType

Types of cyber attacks.

#### Values

- **Malware**
  - `RANSOMWARE` - Ransomware attacks
  - `MALWARE` - General malware
  - `SPYWARE` - Spyware
  - `ADWARE` - Adware
  - `TROJAN` - Trojan horses
  - `WORM` - Computer worms
  - `ROOTKIT` - Rootkits

- **Social Engineering**
  - `PHISHING` - General phishing
  - `SPEAR_PHISHING` - Targeted phishing
  - `WHALING` - Executive phishing
  - `SMISHING` - SMS phishing
  - `VISHING` - Voice phishing
  - `SOCIAL_ENGINEERING` - Other social engineering

- **Network Attacks**
  - `DDOS` - Distributed denial of service
  - `MAN_IN_THE_MIDDLE` - Man-in-the-middle attacks
  - `SNIFFING` - Packet sniffing
  - `SPOOFING` - IP/email spoofing
  - `HIJACKING` - Session hijacking

- **Web Application Attacks**
  - `SQL_INJECTION` - SQL injection
  - `XSS` - Cross-site scripting
  - `CSRF` - Cross-site request forgery
  - `PATH_TRAVERSAL` - Path traversal
  - `FILE_INCLUSION` - File inclusion attacks
  - `COMMAND_INJECTION` - Command injection

- **Data Breaches**
  - `DATA_BREACH` - General data breach
  - `DATA_EXFILTRATION` - Data exfiltration
  - `DATA_LEAKAGE` - Data leakage

- **System Attacks**
  - `PRIVILEGE_ESCALATION` - Privilege escalation
  - `REMOTE_CODE_EXECUTION` - Remote code execution
  - `BUFFER_OVERFLOW` - Buffer overflow
  - `ZERO_DAY` - Zero-day exploits

- **Physical/Insider**
  - `INSIDER_THREAT` - Insider threats
  - `PHYSICAL_THEFT` - Physical theft
  - `SABOTAGE` - Sabotage

- **Other**
  - `UNKNOWN` - Unknown attack type
  - `OTHER` - Other attacks not listed

### SourceType

Types of information sources.

#### Values

- **Official Sources**
  - `GOVERNMENT_REPORT` - Government reports
  - `REGULATORY_FILING` - Regulatory filings
  - `PRESS_RELEASE` - Official press releases

- **Media Sources**
  - `NEWS_ARTICLE` - News articles
  - `BLOG_POST` - Blog posts
  - `SOCIAL_MEDIA` - Social media posts

- **Security Sources**
  - `SECURITY_REPORT` - Security research reports
  - `VENDOR_REPORT` - Vendor security reports
  - `RESEARCH_REPORT` - Academic/research reports

- **Company Sources**
  - `COMPANY_STATEMENT` - Company statements
  - `INVESTOR_REPORT` - Investor reports
  - `INTERNAL_REPORT` - Internal company reports

- **Community Sources**
  - `COMMUNITY_REPORT` - Community-submitted reports
  - `USER_SUBMISSION` - Direct user submissions

- **Other**
  - `OTHER` - Other source types
  - `UNKNOWN` - Unknown source type

### DataCategory

Types of affected data.

#### Values

- **Personal Information**
  - `PERSONAL_DATA` - General personal data
  - `CONTACT_INFORMATION` - Contact details
  - `IDENTIFICATION` - Identification documents
  - `FINANCIAL_INFORMATION` - Financial information
  - `HEALTH_INFORMATION` - Health information

- **Business Information**
  - `BUSINESS_DATA` - General business data
  - `INTELLECTUAL_PROPERTY` - Intellectual property
  - `TRADE_SECRETS` - Trade secrets
  - `FINANCIAL_DATA` - Business financial data
  - `CUSTOMER_DATA` - Customer data
  - `EMPLOYEE_DATA` - Employee data

- **System Information**
  - `SYSTEM_DATA` - System configuration data
  - `CREDENTIALS` - Login credentials
  - `ACCESS_TOKENS` - Access tokens
  - `CONFIGURATION_DATA` - Configuration files

- **Other**
  - `OTHER` - Other data types
  - `UNKNOWN` - Unknown data types

## Database Features

### Full-Text Search

The schema includes PostgreSQL's `pg_trgm` extension for efficient full-text search on text fields:
- Organization names (`org_name`)
- Notes and descriptions (`notes`)

### Indexes

The following indexes are maintained for performance:

#### Incident Table
- `idx_incident_org_name` - Organization name
- `idx_incident_sector` - Organization sector
- `idx_incident_date` - Incident date
- `idx_incident_attack_type` - Attack type
- `idx_incident_org_name_trgm` - Full-text search on organization name
- `idx_incident_notes_trgm` - Full-text search on notes

#### Source Table
- `idx_source_incident_id` - Foreign key to incident
- `idx_source_type` - Source type

### Audit Trail

The `sources` table provides an audit trail for all incident sources, allowing:
- Multiple sources per incident
- Source verification tracking
- Historical source information
- Source attribution and credibility assessment

## Data Validation

All fields are validated according to the following rules:

### Required Fields
- All required fields must be present and non-null
- String fields must not be empty
- Dates must be valid calendar dates
- URLs must be valid URLs
- UUID fields must be valid UUIDs

### Enum Values
- Enum fields must use one of the predefined values
- Case sensitivity follows the enum definitions (SCREAMING_SNAKE_CASE)

### Data Types
- Financial values must be non-negative integers
- Record counts must be non-negative integers
- Boolean fields must be true/false
- JSON fields must contain valid JSON arrays

## API Representation

The schema is exposed through the API with the following considerations:

### Serialization
- All dates are serialized as ISO 8601 strings (YYYY-MM-DD)
- Timestamps include timezone information
- Enums are serialized as strings in SCREAMING_SNAKE_CASE
- UUIDs are standard UUID strings
- Financial values are integers (in IDR)

### Example Incident

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "org_name": "PT Teknologi Indonesia",
  "org_sector": "TECHNOLOGY",
  "incident_date": "2024-01-15",
  "disclosure_date": "2024-01-20",
  "attack_type": "RANSOMWARE",
  "data_categories": ["PERSONAL_DATA", "FINANCIAL_DATA"],
  "record_count_estimate": 50000,
  "financial_impact_idr": 1000000000,
  "actor_alias": "Unknown",
  "actor_group": "LockBit",
  "source_url": "https://example.com/news/cyber-attack",
  "source_type": "NEWS_ARTICLE",
  "verified": true,
  "notes": "Attack targeted customer database",
  "created_at": "2024-01-21T10:00:00Z",
  "updated_at": "2024-01-21T10:00:00Z"
}
```

## Migration History

The schema evolves through database migrations:

1. **m20240406_000001_create_incidents_table** - Initial incidents table
2. **m20240406_000002_add_enum_types** - Add enum types, sources table, and full-text search

Each migration is reversible and includes proper rollback procedures.

## Extending the Schema

When extending the schema:

1. **Add new enum values** - Create a new migration to alter enum types
2. **Add new fields** - Create a migration to alter tables
3. **Add new entities** - Create a migration with new tables
4. **Update indexes** - Add performance indexes as needed

All changes should maintain backward compatibility where possible and include proper migration and rollback procedures.
