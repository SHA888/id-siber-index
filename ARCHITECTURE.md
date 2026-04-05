# Architecture

`id-siber-index` is a data pipeline with a public API. It has three functional layers: collection, storage, and serving. The stack is locked to Rust and Python.

---

## Stack Rationale

**Rust** handles all I/O-bound and systems work: crawlers, normalizers, the API server, CLI tooling, and the dark web OPSEC layer. Rust is chosen for the crawler specifically because robust error handling and memory safety matter when writing long-running scrapers against hostile infrastructure (dark web sources, inconsistent HTML, network timeouts). `tokio` async runtime handles concurrent crawling across multiple sources without thread overhead.

**Python (uv)** handles the NLP enrichment pipeline exclusively. The Indonesian spaCy model (`id_core_news_sm`) and the broader Python NLP ecosystem (Hugging Face transformers, pandas for analysis notebooks) have no equivalent in Rust. Python is not used for any I/O infrastructure — only for offline enrichment passes over already-collected records. `uv` is the package manager; no conda, no virtualenv.

**PostgreSQL** is the primary store. The incident schema is relational — organizations, incidents, actors, sources, IOCs all have foreign-key relationships that a document store handles poorly. Full-text search over Bahasa content uses `pg_trgm` in v1, Meilisearch in v2+. No ORM — raw SQL via `sqlx` in Rust.

**Meilisearch** provides the user-facing search layer from v0.2 onward. Indonesian tokenization is supported. Meilisearch is the search index, not the source of truth — PostgreSQL is canonical.

---

## Repository Structure

```
id-siber-index/
├── crates/
│   ├── crawler/          # Source-specific crawlers (Rust)
│   │   ├── src/
│   │   │   ├── sources/
│   │   │   │   ├── idx.rs        # IDX electronic disclosure crawler
│   │   │   │   ├── bssn.rs       # BSSN press release crawler
│   │   │   │   ├── ojk.rs        # OJK enforcement release crawler
│   │   │   │   ├── media.rs      # Indonesian media crawler
│   │   │   │   └── darkweb.rs    # Dark web sources (v0.2+, OPSEC-isolated)
│   │   │   ├── normalizer.rs     # Raw HTML → IncidentDraft struct
│   │   │   ├── dedup.rs          # Cross-source deduplication
│   │   │   └── scheduler.rs      # Crawl scheduling and rate limiting
│   ├── schema/           # Shared data types (Rust)
│   │   └── src/
│   │       ├── incident.rs       # Core Incident struct + enums
│   │       ├── actor.rs          # ThreatActor struct
│   │       ├── ioc.rs            # IOC struct (v0.2+)
│   │       └── stix.rs           # STIX 2.1 serialization (v1.0+)
│   ├── api/              # Axum REST API server (Rust)
│   │   └── src/
│   │       ├── routes/
│   │       │   ├── incidents.rs
│   │       │   ├── stats.rs
│   │       │   ├── actors.rs     # v0.2+
│   │       │   └── export.rs     # STIX/TAXII export (v1.0+)
│   │       ├── auth.rs           # API key auth (v0.2+)
│   │       └── ratelimit.rs
│   ├── migrate/          # Database migrations (Rust + sqlx)
│   └── cli/              # Admin CLI tooling (Rust)
├── nlp/                  # Python NLP enrichment pipeline
│   ├── pyproject.toml    # uv managed
│   ├── enrichment/
│   │   ├── ner.py        # Named entity extraction (spaCy id model)
│   │   ├── classify.py   # Attack type classification
│   │   └── translate.py  # Bahasa → EN normalization for enums
│   └── notebooks/        # Analysis and research notebooks
├── frontend/             # SvelteKit public search UI (v0.3+)
├── opsec/                # Dark web crawling infrastructure (v0.2+)
│   ├── tor/              # Tor circuit management
│   └── vm/               # VM isolation scripts
├── schema/               # JSON Schema + STIX bundle definitions
├── migrations/           # SQL migration files
├── docker-compose.yml
├── README.md
├── ARCHITECTURE.md
└── TODO.md
```

---

## Data Flow

### Public Source Pipeline (v0.1)

```
Public Sources
(IDX / BSSN / OJK / Media)
         │
         ▼
   Rust Crawler
   (source-specific parsers)
         │
         ▼
   Raw HTML / PDF
         │
         ▼
   Rust Normalizer
   (IncidentDraft struct)
         │
         ▼
   Python NLP Enrichment
   (entity extraction, classification)
         │
         ▼
   Deduplication Engine
   (cross-source merge)
         │
         ▼
   Manual Verification Queue
   (CLI tool for human review)
         │
         ▼
   PostgreSQL (canonical store)
         │
         ├──► Meilisearch (search index)
         │
         └──► Axum REST API
```

### Dark Web Pipeline (v0.2+)

```
Dark Web Sources
(BreachForums / Darkforums / Telegram)
         │
         ▼
   OPSEC Layer
   (Tor circuit, VM-isolated process)
         │
         ▼
   Rust Dark Web Crawler
   (separate binary, no network access to main infra)
         │
         ▼
   Airgapped Transfer
   (signed JSON bundles, manual import)
         │
         ▼
   Main Pipeline
   (same normalizer / enrichment / dedup flow)
```

The dark web crawler runs in a VM-isolated environment with no direct network path to the main database. Data exits the OPSEC environment as signed JSON bundles only. This prevents a compromised dark web session from reaching production infrastructure.

---

## Core Data Model

### `incidents` table

```sql
CREATE TABLE incidents (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_name        TEXT NOT NULL,
    org_sector      sector_enum NOT NULL,
    incident_date   DATE,
    disclosure_date DATE,
    attack_type     attack_type_enum NOT NULL,
    data_categories data_category_enum[] NOT NULL DEFAULT '{}',
    record_count_estimate BIGINT,
    financial_impact_idr  BIGINT,
    actor_id        UUID REFERENCES actors(id),
    source_url      TEXT NOT NULL,
    source_type     source_type_enum NOT NULL,
    verified        BOOLEAN NOT NULL DEFAULT false,
    notes           TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

### `actors` table (v0.2+)

```sql
CREATE TABLE actors (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    alias       TEXT NOT NULL,
    group_name  TEXT,
    nation_state TEXT,
    motivation  motivation_enum,
    first_seen  DATE,
    last_seen   DATE,
    notes       TEXT
);
```

### `iocs` table (v0.2+)

```sql
CREATE TABLE iocs (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    incident_id UUID NOT NULL REFERENCES incidents(id),
    ioc_type    ioc_type_enum NOT NULL,  -- ip, domain, hash_md5, hash_sha256, url
    value       TEXT NOT NULL,
    first_seen  DATE,
    last_seen   DATE,
    source      TEXT
);
```

### Enums

```sql
CREATE TYPE sector_enum AS ENUM (
    'BFSI', 'Government', 'Telco', 'Retail',
    'Healthcare', 'Education', 'Energy',
    'Logistics', 'Technology', 'Other'
);

CREATE TYPE attack_type_enum AS ENUM (
    'Ransomware', 'DataBreach', 'DDoS',
    'CredentialStuffing', 'Defacement',
    'SupplyChain', 'Phishing', 'AccountTakeover',
    'InsiderThreat', 'Unknown'
);

CREATE TYPE source_type_enum AS ENUM (
    'IDX', 'BSSN', 'OJK', 'Media',
    'DarkWeb', 'Academic', 'Partner'
);

CREATE TYPE data_category_enum AS ENUM (
    'customer_pii', 'financial_records', 'credentials',
    'government_id', 'health_records', 'employee_data',
    'source_code', 'operational_data', 'unknown'
);
```

---

## API Design

### Versioning

All endpoints are prefixed with `/v1`. Breaking changes increment the major version. The public tier free API will maintain backward compatibility across major versions for a minimum of 12 months.

### Response Format

```json
{
  "data": [...],
  "meta": {
    "total": 248,
    "page": 1,
    "per_page": 20,
    "version": "1.0.0"
  }
}
```

### Authentication

v0.1: No authentication. IP-based rate limiting only.
v0.2+: API key via `Authorization: Bearer <key>` header for Standard and Premium tiers.
Partner tier: mutual TLS for bidirectional data sharing endpoints.

### Rate Limiting

```
Public:   100 req/day per IP (sliding window)
Standard: 10,000 req/day per key
Premium:  Unlimited
Partner:  Unlimited
```

Rate limit headers on every response:
```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 87
X-RateLimit-Reset: 1735689600
```

---

## NLP Pipeline Detail

The Python enrichment pipeline runs as an offline batch process, not in the request path. It operates on `IncidentDraft` records in the `pending_enrichment` table.

### Indonesian NER

Uses `id_core_news_sm` (spaCy Indonesian model) for:
- Organization name extraction and normalization
- Date extraction from Bahasa-language text
- Location tagging (relevant for sector classification)

### Attack Classification

A fine-tuned text classifier maps Bahasa-language incident descriptions to `attack_type_enum` values. Training data: manually labeled subset of historical incidents. Model: `indobert-base-p2` (IndoBERT, pretrained on Indonesian corpus).

### Translation Normalization

Sector and attack type values appear in both Bahasa and English in source material. A mapping layer normalizes:
- `serangan siber` → generic, requires further classification
- `kebocoran data` → `DataBreach`
- `ransomware` → `Ransomware` (loanword, appears as-is)
- `sektor perbankan` → `BFSI`

---

## OPSEC Infrastructure (v0.2+)

Dark web crawling is operationally isolated from all other infrastructure.

**Network isolation:** The dark web crawler runs in a dedicated VM with no network route to production PostgreSQL. It outputs signed JSON bundles only.

**Identity management:** Each dark web source uses a separate Tor circuit. No persistent identities. No posting, no account registration — read-only access only.

**Legal boundary:** We access only publicly readable forum posts and leak site listings. We do not access password-protected areas, purchase data, or interact with threat actors. We record metadata about listings (organization name, data categories claimed, date posted, price if listed) without downloading the advertised data.

**Bundle signing:** Exported JSON bundles from the dark web VM are signed with a dedicated GPG key before transfer to the main pipeline. Unsigned bundles are rejected.

---

## Deployment

### Minimal (v0.1 — single server)

```
Ubuntu 24.04 LTS
PostgreSQL 16
Meilisearch 1.x
Rust API binary (systemd service)
Crawler binary (cron / systemd timer)
Nginx reverse proxy
```

### Production (v1.0+)

```
Kubernetes or Nomad cluster
PostgreSQL with read replicas
Meilisearch cluster
Separate crawler workers per source
Redis for rate limiting
Separate OPSEC VM for dark web crawler
CDN for static API documentation
```

---

## Security Posture

- The API server has no write endpoints on the public tier. All writes go through the crawler pipeline or the admin CLI.
- The admin CLI requires local access — no remote admin endpoints exposed.
- Database credentials are never in the repository. Environment variables only, injected at runtime.
- All dependencies are pinned. `Cargo.lock` and `uv.lock` are committed.
- AGPL-3.0 license means any derivative deployment that modifies the code must publish those modifications. This prevents silent forks that strip attribution or data governance controls.

---

## Standards Compliance

| Standard | Status | Notes |
|---|---|---|
| STIX 2.1 | v1.0 | Incident and actor export |
| TAXII 2.1 | v1.0 | Feed endpoint for Premium/Partner |
| MITRE ATT&CK | v0.2 | TTP tagging on enriched records |
| TLP (Traffic Light Protocol) | v0.2 | All records tagged TLP:WHITE (public) or TLP:GREEN (Standard+) |
| ISO 27001 | Future | Referenced for sector classification alignment |
