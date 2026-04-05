# TODO

Versioned roadmap from first commit to projected long-term capabilities. Each section header is a release tag. PATCH and MINOR increments between milestones follow the policy defined under each version — they are not planned in advance, they happen organically.

**SemVer policy:**
- `0.x.0` — pre-stable minor milestone. Schema and API may change.
- `0.x.x` — pre-stable patch. See per-version patch policy below.
- `1.0.0` — API contract frozen. Breaking changes only in `2.0.0`.
- `1.x.0` — backward-compatible additions post-stable. New endpoints, new optional fields, new enum values.
- `1.x.x` — post-stable patch. Bug fixes, record corrections, dependency updates. No schema changes.
- `2.0.0`+ — breaking changes. Field removals, renamed endpoints, new auth model.

**Status markers:** `[ ]` not started · `[~]` in progress · `[x]` done

---

## v0.0.1 — Project Scaffolding
**Target: Day 1–3**
**Goal: Solid foundation. Everything a contributor needs to clone, run, and contribute is in place before the first crawler line is written.**
**Release condition: Repo is public, CI is green, `make dev` produces a running stack, all tooling is pinned**

### v0.0.x — Patch Policy
Patches in the `0.0.x` range:
- Tooling version pin updates (Rust toolchain, Python, Node for frontend tooling)
- CI configuration fixes
- Dev environment setup corrections
- Documentation clarifications in scaffolding files

Patches do NOT add application logic, schema definitions, or crawler code. Those are `0.1.0`.

---

### Repository Structure

- [x] Initialize Git repository
- [x] Create GitHub org `id-siber-index` and push
- [x] Set repo to public
- [x] Add pre-release banner to `README.md`: schema unstable until `v0.1.0` tag
- [x] `README.md` — complete
- [x] `ARCHITECTURE.md` — complete
- [x] `TODO.md` — complete
- [x] `CHANGELOG.md` — initialized with `v0.0.1` entry
- [x] `LICENSE` — AGPL-3.0
- [x] `data/LICENSE` — CC BY 4.0
- [x] `CONTRIBUTING.md` — contribution guidelines
- [x] `CODE_OF_CONDUCT.md` — Contributor Covenant
- [x] `SECURITY.md` — responsible disclosure policy
- [x] `.gitignore` — Rust, Python, `.env`, OS artifacts
- [x] `.editorconfig` — consistent formatting across editors

### Rust Workspace

- [ ] `Cargo.toml` — workspace definition, all crates declared
- [ ] `rust-toolchain.toml` — pinned stable toolchain version
- [ ] Crate stubs created (empty `lib.rs` or `main.rs`, no logic yet):
  - [ ] `crates/schema`
  - [ ] `crates/crawler`
  - [ ] `crates/api`
  - [ ] `crates/migrate`
  - [ ] `crates/cli`
- [ ] `cargo clippy` passes on empty stubs
- [ ] `cargo fmt --check` passes
- [ ] `cargo test` passes (no tests yet — just confirms build)
- [ ] `cargo audit` clean (no known vulnerabilities in dependencies)

### Python Tooling

- [ ] `nlp/pyproject.toml` — `uv` managed project
- [ ] `nlp/.python-version` — pinned Python version
- [ ] Empty package structure: `nlp/enrichment/__init__.py`
- [ ] `uv sync` produces clean environment
- [ ] `ruff check` passes on empty stubs
- [ ] `ruff format --check` passes

### Infrastructure

- [ ] `docker-compose.yml` — PostgreSQL 16 + Meilisearch + placeholder API service
- [ ] `docker-compose.override.yml.example` — local dev overrides template
- [ ] `.env.example` — all required environment variables documented
- [ ] `Makefile`:
  - [ ] `make dev` — starts docker-compose stack
  - [ ] `make stop` — stops stack
  - [ ] `make test` — runs `cargo test` + `uv run pytest`
  - [ ] `make lint` — runs `cargo clippy` + `cargo fmt --check` + `ruff check`
  - [ ] `make audit` — runs `cargo audit`
  - [ ] `make clean` — removes build artifacts
- [ ] `make dev` produces a running PostgreSQL + Meilisearch stack with no errors

### CI/CD

- [ ] `.github/workflows/ci.yml`:
  - [ ] Triggers: push to `main`, all PRs
  - [ ] Jobs: `cargo clippy`, `cargo fmt --check`, `cargo test`, `cargo audit`
  - [ ] Python jobs: `ruff check`, `ruff format --check`, `uv run pytest`
  - [ ] All jobs must pass before merge
- [ ] `.github/workflows/release.yml`:
  - [ ] Triggers: push of `v*` tag
  - [ ] Creates GitHub Release with changelog entry
- [ ] `.github/CODEOWNERS` — assign maintainer to all paths
- [ ] `.github/issue_templates/`:
  - [ ] `incident_report.md`
  - [ ] `crawler_bug.md`
  - [ ] `schema_proposal.md`
  - [ ] `bug_report.md`
- [ ] `.github/pull_request_template.md`
- [ ] GitHub branch protection on `main`: CI must pass, no direct push
- [ ] GitHub Discussions enabled

### OSS Hygiene

- [ ] Dependency license audit: all Rust + Python deps are OSS-compatible with AGPL-3.0
- [ ] `NOTICE` file if any dependencies require attribution
- [ ] No secrets, credentials, or `.env` files in git history (verify with `git log`)
- [ ] `cargo deny check` — license, advisories, and duplicate dependency check
- [ ] Repository topics set on GitHub: `cybersecurity`, `indonesia`, `threat-intelligence`, `open-data`, `rust`
- [ ] Repository description set: "Open index of cybersecurity incidents affecting Indonesian organizations"

---

## v0.1.0 — Public Corpus MVP
**Target: 4 weeks from first commit**
**Goal: Working public repository with ~200 verified incidents, free API, open source launch**
**Release condition: All tasks complete, 200+ verified records in production, API publicly accessible**

### v0.1.x — Patch Policy
Patches in the `0.1.x` range require no planning. Cut a patch when:
- A crawler parser breaks due to upstream source HTML change
- A normalization bug produces incorrect field values on existing records
- A security fix is needed in the API server
- New incident records are backfilled (data additions are patches, not features)
- Documentation corrections

Patches do NOT change the schema, add new endpoints, or add new data sources. Those are `0.2.0`.

---

### Schema

- [ ] Define `Incident` struct in `crates/schema/src/incident.rs`
- [ ] Define all enums: `Sector`, `AttackType`, `SourceType`, `DataCategory`
- [ ] Write SQL migration for `incidents` table
- [ ] Write SQL migration for `sources` table (audit trail per record)
- [ ] Add `pg_trgm` extension migration for full-text search
- [ ] Document schema in `schema/README.md` with field definitions and enum values
- [ ] Write `schema/incident.json` (JSON Schema for external consumers)

### IDX Crawler

- [ ] Parse IDX electronic disclosure feed (`eidnews.idx.co.id`)
- [ ] Keyword filter: Bahasa + English cyber incident terms
  - [ ] `serangan siber`, `kebocoran data`, `ransomware`, `gangguan sistem`
  - [ ] `cyber attack`, `data breach`, `system disruption`, `unauthorized access`
- [ ] Extract: org name, disclosure date, incident description, URL
- [ ] Normalize to `IncidentDraft` struct
- [ ] Historical backfill: 2020–present
- [ ] Deduplication: same org + same date window → single record
- [ ] Unit tests for parser

### BSSN Crawler

- [ ] Parse BSSN press releases (`bssn.go.id/siaran-pers`)
- [ ] Parse BSSN annual threat landscape reports (PDF extraction)
- [ ] Extract: org name if named, incident date, attack type, sector
- [ ] Handle BSSN's pattern of aggregate statistics vs named incidents
- [ ] Unit tests for parser

### OJK Crawler

- [ ] Parse OJK enforcement releases and complaint summary reports
- [ ] Extract: financial sector incidents, fraud complaints data
- [ ] Link to relevant IDX disclosures where org is the same
- [ ] Unit tests for parser

### Media Crawler

- [ ] Tempo (`tempo.co/tag/keamanan-siber`)
- [ ] Kompas Tech (`tekno.kompas.com`)
- [ ] Detik Inet (`inet.detik.com`)
- [ ] Bisnis Indonesia (`teknologi.bisnis.com`)
- [ ] Respect `robots.txt` and crawl delay for all sources
- [ ] Deduplicate: same incident in multiple outlets → single record, multiple source URLs
- [ ] Unit tests per outlet parser

### Normalization Pipeline

- [ ] `IncidentDraft` → `Incident` normalization logic
- [ ] Date parsing: handle Indonesian date formats (`8 Mei 2024`, `May 8, 2024`)
- [ ] Org name normalization: `PT Bank X Tbk` → `Bank X` canonical name
- [ ] Attack type classification: keyword-based rules (ML in v0.2.0)
- [ ] Sector classification: keyword + org name lookup rules
- [ ] Confidence scoring on each normalized field

### Verification CLI

- [ ] `isi review` — interactive CLI to review `IncidentDraft` records
- [ ] Accept / reject / edit individual fields
- [ ] Mark record as `verified: true` on accept
- [ ] Batch review mode for historical backfill

### API Server (Axum)

- [ ] `GET /v1/incidents` — list with filters (sector, attack_type, from, until)
- [ ] `GET /v1/incidents/{id}` — single record
- [ ] `GET /v1/incidents/recent` — last 30 days
- [ ] `GET /v1/stats` — aggregate counts by sector, attack type, year
- [ ] IP-based rate limiting (100 req/day sliding window)
- [ ] CORS headers for browser access
- [ ] JSON response with `data` + `meta` envelope including `meta.version`
- [ ] Health check endpoint `GET /health`
- [ ] Request logging (IP addresses anonymized — no PII in logs)

### Infrastructure

- [ ] `docker-compose.yml` — PostgreSQL + Meilisearch + API
- [ ] Database migrations runner (`cargo run --bin migrate`)
- [ ] `.env.example` with all required environment variables
- [ ] `Makefile` with `make dev`, `make crawl`, `make test`
- [ ] GitHub Actions: CI on PR (test + clippy + fmt)
- [ ] GitHub Actions: Nightly crawl run

### Open Source Launch

- [ ] `README.md` — complete
- [ ] `ARCHITECTURE.md` — complete
- [ ] `CONTRIBUTING.md` — data contribution and code contribution guidelines
- [ ] `CODE_OF_CONDUCT.md`
- [ ] `LICENSE` — AGPL-3.0
- [ ] `data/LICENSE` — CC BY 4.0 for incident data
- [ ] `CHANGELOG.md` — initialized, entry for v0.1.0
- [ ] GitHub issue templates: incident report, crawler bug, schema proposal
- [ ] GitHub discussions enabled for community coordination
- [ ] Initial public announcement to Indonesian security community

---

## v0.2.0 — Dark Web Layer + Search + NLP Enrichment
**Target: Month 2–3**
**Goal: Dark web coverage active, Meilisearch search live, NLP enrichment pipeline running**
**Release condition: Dark web crawler operational, search endpoint live, NLP enrichment running on >80% of records**
**Prerequisite: Formal legal opinion on UU PDP boundary for dark web metadata (see Known Hard Problems)**

### v0.2.x — Patch Policy
Patches in the `0.2.x` range:
- Dark web source URL changes (forums migrate, domains change)
- NLP model accuracy fixes
- Meilisearch index configuration tuning
- New ransomware leak site added to monitor list
- Bug fixes in OPSEC bundle import pipeline
- Record enrichment corrections from NLP pipeline improvements

---

### OPSEC Infrastructure

- [ ] Dark web crawler as separate Rust binary (`crates/darkweb-crawler`)
- [ ] VM isolation setup documentation (`opsec/vm/README.md`)
- [ ] Tor circuit management (`opsec/tor/circuit.rs`)
- [ ] Signed JSON bundle export from OPSEC VM
- [ ] Bundle import CLI with GPG signature verification
- [ ] Source-type tagging: all dark web records tagged `source_type: DarkWeb`

### Dark Web Sources

- [ ] BreachForums Indonesia-relevant listing monitor
- [ ] Darkforums.st Indonesia organization monitor
- [ ] Ransomware leak site aggregator (LockBit, RansomHub, ALPHV, Cl0p active sites)
- [ ] Bahasa-language Telegram public channel monitor (read-only)
- [ ] Deduplication against IDX/media records: same org + timeframe → enrich, not duplicate

### Python NLP Pipeline

- [ ] `uv` project setup in `nlp/`
- [ ] spaCy Indonesian model integration (`id_core_news_sm`)
- [ ] NER pipeline: org name extraction from Bahasa article text
- [ ] Date extraction from Bahasa text
- [ ] Attack type classifier: IndoBERT fine-tuned on v0.1.0 verified records
- [ ] Sector classifier
- [ ] Bahasa → EN enum normalization mapping
- [ ] Batch enrichment runner: processes `pending_enrichment` queue in PostgreSQL
- [ ] Confidence scores on NLP-extracted fields vs rule-extracted fields

### Meilisearch Integration

- [ ] Meilisearch index configuration (Indonesian tokenization settings)
- [ ] Sync job: PostgreSQL → Meilisearch on record insert/update
- [ ] `GET /v1/search?q=<query>` endpoint
- [ ] Faceted search: sector, attack type, year, verified status
- [ ] Bahasa and English query support simultaneously

### Threat Actor Model

- [ ] `actors` table migration
- [ ] Link incidents to actors via `actor_id`
- [ ] Actor profiles for top 10 groups actively targeting Indonesia:
  - [ ] LockBit 3.0 / Brain Cipher
  - [ ] RansomHub
  - [ ] ALPHV / BlackCat
  - [ ] Lazarus Group (DPRK)
  - [ ] Mustang Panda (PRC)
  - [ ] TA505 / FIN11
  - [ ] Bjorka (local)
  - [ ] Others as evidenced by v0.1.0 corpus
- [ ] `GET /v1/actors` — actor list with incident counts
- [ ] `GET /v1/actors/{id}` — actor profile with linked incidents

### API Authentication

- [ ] API key generation and management
- [ ] Standard tier enforcement: 10,000 req/day per key
- [ ] Premium tier enforcement: unlimited
- [ ] Partner tier: API key + mutual TLS endpoint
- [ ] Key provisioning via email verification (low-friction abuse barrier)

---

## v0.3.0 — Frontend + Partner Program
**Target: Month 4–6**
**Goal: Public search UI live, first Partner data-sharing agreements signed**
**Release condition: SvelteKit frontend deployed, at least 2 Partner agreements active with data flowing**

### v0.3.x — Patch Policy
Patches in the `0.3.x` range:
- UI bug fixes and accessibility improvements
- New media outlet added to crawler
- Data quality improvements from partner feedback
- Org name alias table additions
- Frontend translation corrections (Bahasa UI)

---

### SvelteKit Frontend

- [ ] Public search interface at `idsiberindex.id` (coming soon)
- [ ] Incident search and filter UI
- [ ] Incident detail page with source citation display
- [ ] Sector and trend statistics dashboard
- [ ] Actor profiles page
- [ ] API documentation (Scalar or Redoc)
- [ ] Bahasa Indonesia / English language toggle
- [ ] Responsive mobile layout
- [ ] No tracking, no analytics, no third-party cookies

### Partner Program

- [ ] Partner agreement template (data sharing terms, attribution requirements)
- [ ] Partner data submission API endpoint (authenticated, write access)
- [ ] Partner data format specification (JSON schema)
- [ ] Contribution validation pipeline (submitted records enter verification queue)
- [ ] Partner provenance tag on records contributed externally
- [ ] Target first anchor partners: ITSEC Asia, Xynexis, Vaksincom

### Community Contribution

- [ ] Public incident report form (web + GitHub issue)
- [ ] Source URL required — no unverified anonymous reports accepted
- [ ] Verification queue status visible to contributor
- [ ] Contributor acknowledgment in record provenance history

### Data Quality

- [ ] Fuzzy org name deduplication (Levenshtein distance on canonical names)
- [ ] Org name alias table (Bank BRI → Bank Rakyat Indonesia → BRI)
- [ ] Cross-source confidence scoring: same incident in 3+ sources → higher confidence
- [ ] Stale record detection: flag records with no update in 180 days

---

## v1.0.0 — Stable API + STIX/TAXII + Production Infrastructure
**Target: Month 6–9**
**Goal: API contract frozen, STIX export live, production infrastructure, first paying Standard subscribers**
**Release condition: All tasks complete, load tested, SLA documented, at least 5 paying Standard subscribers**
**This tag freezes the `/v1/` API contract. No breaking changes until v2.0.0.**

### v1.x.0 — Minor Release Policy
Post-stable minor releases (`v1.1.0`, `v1.2.0`, etc.) are backward-compatible additions:
- New optional query parameters on existing endpoints
- New optional response fields (existing fields never removed or renamed)
- New endpoints under `/v1/` that do not affect existing routes
- New enum values (existing values never removed)
- New export formats alongside existing ones
- New sector or subsector coverage additions

### v1.x.x — Patch Policy
- Security fixes — ship immediately, do not wait for minor release window
- Bug fixes in STIX serialization or TAXII endpoints
- Performance improvements (query optimization, caching tuning)
- Dependency security updates (`cargo audit` clean)
- Record corrections from verified reports
- Documentation fixes

---

### STIX 2.1 Export

- [ ] STIX Bundle serialization (`crates/schema/src/stix.rs`)
- [ ] Map `Incident` → STIX `Incident` object
- [ ] Map `Actor` → STIX `Threat Actor` object
- [ ] Map `IOC` → STIX `Indicator` object
- [ ] `GET /v1/export/stix/{id}` — single incident as STIX bundle
- [ ] `GET /v1/export/stix/bundle?sector=BFSI` — filtered bundle export
- [ ] STIX spec validation on all outputs

### TAXII 2.1

- [ ] TAXII server implementation (`/taxii2/` endpoint prefix)
- [ ] Discovery endpoint
- [ ] Collection endpoints (by sector, by actor, by date range)
- [ ] Authentication: Premium and Partner tiers only
- [ ] Compatibility verified: OpenCTI connector, MISP feed

### IOC Database

- [ ] `iocs` table migration
- [ ] IOC extraction from dark web listings and Partner IR contributions
- [ ] IOC types: IPv4, IPv6, domain, URL, MD5, SHA1, SHA256, email
- [ ] IOC deduplication and confidence scoring
- [ ] IOC → Incident linkage
- [ ] `GET /v1/iocs?type=domain&value=<value>`
- [ ] IOC feed endpoint (Premium/Partner): `GET /v1/feed/iocs`
- [ ] IOC expiry: mark stale after 90 days without re-observation

### MITRE ATT&CK Integration

- [ ] ATT&CK technique tagging on enriched incident records
- [ ] ATT&CK navigator layer export for Indonesian incident subset
- [ ] TTP pattern report: top techniques observed across Indonesian incidents

### Production Infrastructure

- [ ] PostgreSQL streaming replication (read replica for API queries)
- [ ] Redis caching for high-traffic endpoints (`/stats`, `/incidents/recent`)
- [ ] Stateless API server: horizontal scaling verified
- [ ] Daily database snapshots, 30-day retention, offsite storage
- [ ] Uptime monitoring and alerting (target: 99.5% uptime SLA)
- [ ] SLA documentation for Standard and Premium tiers

### API Stability

- [ ] `/v1` contract documented and frozen at this tag
- [ ] `/v2` prefix reserved, no routes assigned
- [ ] Deprecation policy in `README.md`: 12-month minimum notice for breaking changes
- [ ] API changelog maintained in `CHANGELOG.md`
- [ ] External schema review: minimum 2 IR practitioners stress-test schema before freeze

---

## v2.0.0 — Attacker Infrastructure Graph + Fraud Intelligence Layer
**Target: Month 10–18**
**Goal: Active C2 infrastructure mapping, first QRIS/BI-FAST fraud signal prototype**
**Breaking changes from v1.x.x:** Graph endpoints use new response format. IOC pivot model replaces flat IOC lookup. `/v2/` prefix for new endpoints; `/v1/` remains supported until deprecation window closes.

### v2.x.0 — Minor Release Policy
- New country added to infrastructure monitoring scope
- New fraud signal data partner integrated
- New phishing kit fingerprint category published
- New pivot capability added to infrastructure graph (backward-compatible)

### v2.x.x — Patch Policy
- Infrastructure graph edge weight recalculations
- Passive DNS source URL updates
- Phishing domain takedown status corrections
- Payment fraud signal model accuracy improvements

---

### Attacker Infrastructure Graph

- [ ] C2 infrastructure tracking: IP → domain → certificate relationships
- [ ] Passive DNS integration (CIRCL pDNS or equivalent)
- [ ] Certificate transparency log monitoring for Indonesian org impersonation domains
- [ ] Infrastructure reuse detection: same C2 across multiple Indonesian incidents
- [ ] Graph storage decision: PostgreSQL edges table vs Neo4j (decide at this milestone)
- [ ] `GET /v2/infrastructure/{ioc}` — pivot from IOC to related infrastructure and incidents

### Phishing Intelligence

- [ ] Indonesian bank phishing domain monitoring
- [ ] QRIS merchant impersonation detection (fake QR code campaign tracking)
- [ ] SMS phishing (smishing) infrastructure tracking
- [ ] Bahasa-language phishing kit fingerprinting
- [ ] Phishing takedown request workflow (BSSN / IDNic / registrar)

### Payment Fraud Signal Layer
*This workstream is the entry condition for Product 1 (QRIS/BI-FAST fraud intelligence platform). It establishes proof of concept and institutional relationships — it does not complete Product 1.*

- [ ] Design payment fraud signal data model (separate schema from incident model)
- [ ] Identify first institutional partner willing to share anonymized QRIS fraud patterns
- [ ] Prototype cross-merchant fraud pattern detection on partner-contributed data
- [ ] Regulatory consultation: publishability of aggregated anonymized payment fraud metadata under UU PDP
- [ ] Draft data sharing agreement template for Bank Indonesia / OJK conversations
- [ ] Document prototype findings and present to BI/OJK as entry to Product 1 negotiation

### Vulnerability Intelligence

- [ ] Indonesian organization internet exposure monitoring (Shodan + Censys)
- [ ] CVE-to-Indonesian-org exposure mapping
- [ ] Identify unpatched critical infrastructure (exposed Fortinet, Citrix, Exchange)
- [ ] Responsible disclosure workflow for identified exposures

---

## v3.0.0 — Southeast Asia Expansion
**Target: Year 2**
**Goal: Malaysia, Philippines, Vietnam, Singapore coverage; cross-country actor correlation active**
**Breaking changes from v2.x.x:** Country field now required on all records. Multi-country endpoints replace single-country assumptions in response format.

### v3.x.0 — Minor Release Policy
- New ASEAN country added to coverage scope
- New regional stock exchange or government CERT source integrated
- New language NLP model deployed

### v3.x.x — Patch Policy
- Regional source URL or format changes
- NLP model accuracy fixes per language
- Cross-country deduplication edge cases

---

### Regional Coverage

- [ ] Malaysia: Bursa Malaysia disclosures, CyberSecurity Malaysia reports, MY media
- [ ] Philippines: PSE disclosures, DICT reports, PH media
- [ ] Vietnam: HNX/HOSE disclosures, VNCERT reports, VN media
- [ ] Singapore: SGX disclosures, CSA reports, SG media
- [ ] Country-specific sector enums and source type extensions per jurisdiction

### Cross-Country Correlation

- [ ] Actor campaigns spanning multiple ASEAN countries
- [ ] Shared attacker infrastructure across regional incidents
- [ ] `GET /v3/actors/{id}/campaigns` — multi-country campaign timeline
- [ ] ASEAN threat landscape report: quarterly, public, PDF + API

### ASEAN ISAC Positioning

- [ ] Proposal to ASEAN-CERT for formal data exchange integration
- [ ] Alignment with ASEAN Cybersecurity Cooperation Strategy framework
- [ ] Multi-country TAXII feed for regional incident data

### Localization

- [ ] Malaysian Bahasa (Bahasa Melayu) NLP support
- [ ] Filipino (Tagalog) NLP support
- [ ] Vietnamese NLP support
- [ ] UI language toggles: EN, ID, MS, TL, VI

---

## v4.0.0 — National Infrastructure Integration
**Target: Year 2–3, contingent on institutional relationships established in v2.0.0–v3.0.0**
**Goal: Formal BSSN MOU signed, Product 1 Bank Indonesia negotiations active**
**Breaking changes from v3.x.x:** BSSN-sourced records use new provenance model. Partner tier splits into Government and Commercial sub-tiers with separate endpoints.**

### v4.x.0 — Minor Release Policy
- New government agency integrated as data partner
- New compliance report type added for OJK/BSSN requirements
- National Digital Firewall integration expanded to new IOC categories

### v4.x.x — Patch Policy
- Government feed format changes
- Compliance report corrections
- IOC blocklist submission status updates

---

### BSSN Integration

- [ ] Formal data sharing MOU with BSSN
- [ ] Nat-CSIRT incident feed integration (bidirectional)
- [ ] BSSN-verified provenance badge on Nat-CSIRT confirmed records
- [ ] Co-branded national incident reports (quarterly)

### OJK / Bank Indonesia Payment Fraud Intelligence
*This is Product 1. Only reachable after v2.0.0 payment fraud signal prototype and regulatory consultation are complete.*

- [ ] Data sharing framework with Bank Indonesia for anonymized QRIS fraud patterns
- [ ] Cross-institutional fraud signal aggregation (requires BI mandate or consortium model)
- [ ] Real-time fraud pattern feed for PSPs and acquiring banks
- [ ] OJK integration: feed into mandatory cyber incident reporting compliance system

### National Digital Firewall Integration

- [ ] IOC feed integration with Komdigi national digital firewall
- [ ] Automated IOC submission to national blocklist
- [ ] Feedback loop: blocked IOC confirmed → incident record updated

---

## v* — Projected Long-Term Capabilities
*Directional projections only. Not committed roadmap items. Each requires validation at the preceding milestone.*

**AI-assisted incident analysis**
- Local LLM-powered incident description summarization (no external API dependency)
- Automated TTP extraction from unstructured incident reports
- Predictive alerting: org attack surface + known actor TTPs → elevated risk signal
- Anomaly detection: sector-level incident rate deviation alerts

**Digital forensic artifact repository**
- Malware sample database: Indonesian-targeted strains (hash + YARA rules, no full binaries publicly)
- Memory forensics artifacts from IR engagements (Partner-contributed, TLP:AMBER)
- Forensic timeline templates for common Indonesian attack patterns
- Automated hash enrichment via MalwareBazaar and VirusTotal integration

**Regulatory intelligence layer**
- UU PDP breach notification tracking: who reported, outcome, enforcement action
- OJK enforcement action timeline and sector exposure mapping
- Compliance gap analysis: high-incident sectors vs low regulatory enforcement rate

**Citizen-facing layer**
- Organizational exposure check: "has this organization been indexed?" (org-level only, not personal)
- Indonesian MSME security posture check (QRIS merchant exposure assessment)
- Bahasa-language public cyber incident feed (RSS + API)

**Academic and research integration**
- Annual anonymized incident dataset release (CC BY 4.0)
- Free API access tier for verified Indonesian academic researchers (with attribution requirement)
- Research collaboration framework: Universitas Indonesia, ITB, ITS

**Open protocol contributions**
- Indonesian incident schema extensions proposed to STIX working group
- Indonesian NLP models contributed back to spaCy community
- Indonesian cybersecurity sector taxonomy published as open standard
- Indonesian threat actor naming convention proposal to MITRE ATT&CK

---

## Permanent Backlog
*Valid ideas, deliberately deferred until the core pipeline is proven.*

- [ ] Mobile app (iOS/Android) — after web API stable at v1.0.0
- [ ] Email/webhook alerting for new incidents by sector — after auth system in v0.2.0
- [ ] Splunk / Elastic / Microsoft Sentinel native integration — after STIX/TAXII in v1.0.0
- [ ] Indonesian cyber crime court case tracking — separate domain, separate project
- [ ] Bug bounty program for id-siber-index infrastructure
- [ ] Hardware security key support for admin CLI
- [ ] Multi-region deployment (Jakarta primary + Surabaya failover)
- [ ] Quarterly offline data export (snapshot downloads)
- [ ] Slack/Discord bot for security team incident alerts

---

## Known Hard Problems
*Track here. Do not defer silently.*

**UU PDP legal boundary for dark web metadata**
Indexing organizational incident metadata (not personal data) is believed to be defensible under UU PDP. This has not been tested in Indonesian courts. Formal legal opinion required before `v0.2.0` dark web layer goes live. Track in: `legal/udp-dark-web-analysis.md`.

**Org name normalization at scale**
`PT Bank Rakyat Indonesia (Persero) Tbk`, `Bank BRI`, `BRI`, `PT BRI` are the same entity. The alias table approach works to ~500 organizations. Beyond that a proper entity resolution system is required. Must be resolved before `v1.0.0` or it becomes a production data quality bottleneck.

**Contributor spam and data poisoning**
A publicly contributable index is a target for false incident reports designed to damage organization reputations. Manual verification is the only control through `v0.3.0`. A contributor trust scoring system must be designed before the contribution pipeline opens to anonymous sources at scale.

**Dark web forum access continuity**
Forum domains change, get seized, go dark. The dark web crawler has no long-term stable source guarantee. This is a human operational problem — it requires ongoing monitoring and source substitution that cannot be fully automated. Assign a person to this before `v0.2.0` ships.

**BSSN relationship**
Everything in `v4.0.0` depends on BSSN treating `id-siber-index` as a partner rather than a threat to its mandate. This is a trust problem, not a technical one. Credibility must be built through transparent operation and demonstrated public benefit across `v0.1.0` through `v3.0.0`. No formal approach to BSSN until the repository has meaningful coverage depth. Do not rush this.

**Schema stability pressure**
After `v1.0.0` freezes the API contract, any discovered schema design mistake becomes expensive to fix. Invest in schema review before cutting `v1.0.0`. Get at least two external IR practitioners to stress-test the schema against real incident records before the freeze.
