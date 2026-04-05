# id-siber-index

**Indonesia Cybersecurity Incident Index** — a structured, open, continuously updated public record of cybersecurity incidents affecting Indonesian organizations.

> **⚠️ PRE-RELEASE WARNING**  
> This repository is in active development (`v0.0.1`). The schema, API structure, and data formats are unstable and will change before the `v0.1.0` release. Do not use this in production yet.

[![License: AGPL-3.0](https://img.shields.io/badge/License-AGPL%203.0-blue.svg)](LICENSE)
[![Status: Active Development](https://img.shields.io/badge/Status-Active%20Development-yellow)]()
[![Data Sources](https://img.shields.io/badge/Sources-IDX%20%7C%20BSSN%20%7C%20OJK%20%7C%20Media-green)]()

---

## What This Is

Indonesia is Southeast Asia's most attacked digital economy — recording over 3.6 billion anomalous cyberattacks in the first seven months of 2025 alone. Yet no single, queryable, public record of incidents affecting Indonesian organizations exists. BSSN publishes annual PDFs. IDX disclosures are buried in filing systems. Media coverage fades. Institutional memory does not accumulate.

`id-siber-index` is the missing layer: a structured incident database built entirely from verifiable public sources, normalized into a common schema, and served through a free public API. It indexes the fact that incidents occurred — organization, sector, date, attack type, data categories, estimated scale, source — without reproducing stolen data.

This is public digital infrastructure, not a commercial product.

---

## What This Is Not

- **Not a breach data repository.** We do not store or redistribute stolen personal data. We index incident metadata only.
- **Not a threat intelligence feed.** Enriched attacker IOCs, TTPs, and attribution data are in the paid API tier. The public layer is incident metadata.
- **Not affiliated with BSSN, OJK, or any government agency.** We are an independent open-source project.
- **Not complete.** We index what is publicly verifiable. Many incidents go unreported. This index reflects what surfaces publicly, not the full scope of Indonesian cyber risk.

---

## Data Sources

All v1 sources are publicly available with no authentication required.

| Source | Type | Coverage |
|---|---|---|
| IDX Electronic Disclosures (`eidnews.idx.co.id`) | Structured filings | Listed company material cyber events |
| BSSN Press Releases & Annual Reports | PDF / HTML | National-level incident summaries |
| OJK Enforcement Releases | HTML | Financial sector incidents and fraud |
| Tempo, Kompas, Detik, Bisnis Indonesia | News media | Incident reporting with organizational detail |

**Month 2+ sources (dark web layer):**
| Source | Type | Coverage |
|---|---|---|
| BreachForums, Darkforums.st | Dark web forums | Indonesian organization data leak listings |
| Bahasa-language Telegram channels | Messaging | Threat actor communications targeting Indonesia |
| Ransomware leak sites | Dark web | Indonesian victim organization postings |

Dark web sources require OPSEC infrastructure (Tor routing, VM isolation) and are handled separately from the public source pipeline. See `ARCHITECTURE.md`.

---

## Schema

Every incident record contains:

```json
{
  "id": "uuid",
  "org_name": "PT Bank Syariah Indonesia Tbk",
  "org_sector": "BFSI",
  "incident_date": "2023-05-08",
  "disclosure_date": "2023-05-11",
  "attack_type": "Ransomware",
  "data_categories": ["financial_records", "customer_pii"],
  "record_count_estimate": null,
  "financial_impact_idr": null,
  "actor_alias": "LockBit",
  "actor_group": "LockBit 3.0",
  "source_url": "https://...",
  "source_type": "Media",
  "verified": true,
  "notes": "1.5TB data exfiltrated. Services disrupted for 3 days.",
  "created_at": "2026-01-15T00:00:00Z",
  "updated_at": "2026-01-15T00:00:00Z"
}
```

Full schema specification: [`schema/incident.rs`](schema/incident.rs)

---

## API

The public API requires no authentication. Rate limit: 100 requests/day per IP.

**Base URL:** `https://api.idsiberindex.id/v1` #(planned)

```bash
# Search incidents by organization
GET /incidents?org=bank+syariah

# Filter by sector and attack type
GET /incidents?sector=BFSI&attack_type=Ransomware

# Filter by date range
GET /incidents?from=2024-01-01&until=2024-12-31

# Get single incident
GET /incidents/{id}

# Summary statistics
GET /stats

# Recent incidents (last 30 days)
GET /incidents/recent
```

All responses are JSON. STIX 2.1 export is available in the Standard API tier.

---

## API Tiers

| Tier | Access | Rate Limit | Price |
|---|---|---|---|
| **Public** | Incident metadata | 100 req/day | Free, no auth |
| **Standard** | Full records + STIX export + IOC data | 10,000 req/day | IDR 3–8M/month |
| **Premium** | Full API + attacker profiling + TAXII feed | Unlimited | IDR 15–30M/month |
| **Partner** | Bidirectional data sharing | Unlimited | Free — requires contributing enrichment data |

Partner tier is intended for Indonesian IR firms, MSSPs, BSSN, and national CIRTs. If you respond to Indonesian cybersecurity incidents and are willing to contribute anonymized case metadata back to the index, contact us.

---

## Quick Start (Self-Hosted)

```bash
# Clone the repository
git clone https://github.com/id-siber-index/id-siber-index
cd id-siber-index

# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Python dependencies (NLP pipeline)
cd nlp && uv sync

# Set up PostgreSQL and Meilisearch
docker-compose up -d

# Run database migrations
cargo run --bin migrate

# Start the crawler (public sources only)
cargo run --bin crawler -- --source idx --source bssn --source media

# Start the API server
cargo run --bin api

# API available at http://localhost:8080
```

Full deployment documentation: [`docs/deployment.md`](docs/deployment.md)

---

## Contributing

We welcome contributions in four categories:

**Data contributions:** If you have verifiable evidence of a cybersecurity incident affecting an Indonesian organization that is not yet in the index, open an issue with source URLs. We verify before indexing.

**Crawler improvements:** New public source integrations, improved normalization logic, better Bahasa NLP extraction. See [`CONTRIBUTING.md`](CONTRIBUTING.md).

**Schema proposals:** Field additions, enum extensions, STIX alignment improvements.

**Research:** Analysis notebooks, sector trend reports, attacker infrastructure correlation.

We do not accept contributions that include actual stolen data, non-public information, or data obtained through unauthorized access.

---

## Legal and Ethics

**License:** AGPL-3.0. The codebase is fully open source. Commercial use of the codebase requires compliance with AGPL terms. The incident data is licensed separately under CC BY 4.0 — you may use it freely with attribution.

**Indonesian law:** This project operates within the bounds of Indonesian law, including UU PDP (Law No. 27 of 2022). We index organizational incident metadata, not personal data. We do not store or redistribute stolen personal records. If you believe a record is inaccurate or creates legal exposure, contact us
<!-- at legal@idsiberindex.id -->
.

**Dark web data:** For sources derived from dark web monitoring, we index only the fact of an incident and its organizational metadata — not the leaked data itself. Our handling follows the approach established by Have I Been Pwned.

**Responsible disclosure:** If you discover a security vulnerability in this project, email 
<!-- security@idsiberindex.id --> before public disclosure.

---

## Roadmap

See [`TODO.md`](TODO.md) for the full versioned roadmap from v0.1 through projected future capabilities.

---

## Maintainers

`id-siber-index` is maintained by independent contributors committed to open Indonesian cybersecurity infrastructure. We are not affiliated with any government agency, commercial TI vendor, or political organization.

---

## Acknowledgments

Data indexed in this project is derived from public sources including Indonesia Stock Exchange disclosures, BSSN publications, OJK reports, and Indonesian media. We thank the Indonesian security research community whose public work forms the foundation of this index.
