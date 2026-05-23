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

- [x] `Cargo.toml` — workspace definition, all crates declared
- [x] `rust-toolchain.toml` — pinned stable toolchain version
- [x] Crate stubs created (empty `lib.rs` or `main.rs`, no logic yet):
  - [x] `crates/schema`
  - [x] `crates/crawler`
  - [x] `crates/api`
  - [x] `crates/migrate`
  - [x] `crates/cli`
- [x] `cargo clippy` passes on empty stubs
- [x] `cargo fmt --check` passes
- [x] `cargo test` passes (no tests yet — just confirms build)
- [x] `cargo audit` clean (no known vulnerabilities in dependencies) - RSA vulnerability in sqlx-mysql ignored (we only use PostgreSQL), fxhash resolved by replacing scraper crate

### Python Tooling

- [x] `nlp/pyproject.toml` — `uv` managed project
- [x] `nlp/.python-version` — pinned Python version
- [x] Empty package structure: `nlp/enrichment/__init__.py`
- [x] `uv sync` produces clean environment
- [x] `ruff check` passes on empty stubs
- [x] `ruff format --check` passes

### Infrastructure

- [x] `docker-compose.yml` — PostgreSQL 16 + Meilisearch + placeholder API service
- [x] `docker-compose.override.yml.example` — local dev overrides template
- [x] `.env.example` — all required environment variables documented
- [x] `Makefile`:
  - [x] `make dev` — starts docker-compose stack
  - [x] `make stop` — stops stack
  - [x] `make test` — runs `cargo test` + `uv run pytest`
  - [x] `make lint` — runs `cargo clippy` + `cargo fmt --check` + `ruff check`
  - [x] `make audit` — runs `cargo audit`
  - [x] `make clean` — removes build artifacts
- [x] `make dev` produces a running PostgreSQL + Meilisearch stack with no errors

### CI/CD

- [x] `.github/workflows/ci.yml`:
  - [x] Triggers: push to `main`, all PRs
  - [x] Jobs: `cargo clippy`, `cargo fmt --check`, `cargo test`, `cargo audit`
  - [x] Python jobs: `ruff check`, `ruff format --check`, `uv run pytest`
  - [x] All jobs must pass before merge
- [x] `.github/workflows/release.yml`:
  - [x] Triggers: push of `v*` tag
  - [x] Creates GitHub Release with changelog entry
- [x] `.github/CODEOWNERS` — assign maintainer to all paths
- [x] `.github/issue_templates/`:
  - [x] `incident_report.md`
  - [x] `crawler_bug.md`
  - [x] `schema_proposal.md`
  - [x] `bug_report.md`
- [x] `.github/pull_request_template.md`
- [ ] GitHub branch protection on `main`: CI must pass, no direct push
- [ ] GitHub Discussions enabled

### Versioning Policy

#### 0.1.x — Patches

- Correct field values on existing records
- A security fix is needed in the API server
- New incident records are backfilled (data additions are patches, not features)
- Documentation corrections

Patches do NOT change the schema, add new endpoints, or add new data sources. Those are `0.2.0`.

---

### Schema

- [x] Define `Incident` struct in `crates/schema/src/incident.rs`
- [x] Define all enums: `Sector`, `AttackType`, `SourceType`, `DataCategory`
- [x] Write SQL migration for `incidents` table
- [x] Write SQL migration for `sources` table (audit trail per record)
- [x] Add `pg_trgm` extension migration for full-text search
- [x] Document schema in `schema/README.md` with field definitions and enum values
- [x] Write `schema/incident.json` (JSON Schema for external consumers)
- [x] Write SQL migration: performance indexes for sector + updated_at queries
  - [x] `CREATE INDEX IF NOT EXISTS idx_incidents_sector ON incidents(org_sector)`
  - [x] `CREATE INDEX IF NOT EXISTS idx_incidents_updated ON incidents(updated_at)`
  - [x] Required by downstream consumers filtering `WHERE org_sector = 'Healthcare' AND updated_at >= $1`

### IDX Crawler

- [x] Parse IDX electronic disclosure feed (`eidnews.idx.co.id`)
- [x] Keyword filter: Bahasa + English cyber incident terms
  - [x] `serangan siber`, `kebocoran data`, `ransomware`, `gangguan sistem`
  - [x] `cyber attack`, `data breach`, `system disruption`, `unauthorized access`
- [x] Extract: org name, disclosure date, incident description, URL
- [x] Normalize to `IncidentDraft` struct
- [x] Historical backfill: 2020–present
- [x] Deduplication: same org + same date window → single record
- [x] Unit tests for parser

### BSSN Crawler

- [x] Parse BSSN press releases (`bssn.go.id/siaran-pers`)
- [x] Parse BSSN annual threat landscape reports (PDF extraction)
- [x] Extract: org name if named, incident date, attack type, sector
- [x] Handle BSSN's pattern of aggregate statistics vs named incidents
- [x] Unit tests for parser

### OJK Crawler

- [x] Parse OJK enforcement releases and complaint summary reports
- [x] Extract: financial sector incidents, fraud complaints data
- [x] Link to relevant IDX disclosures where org is the same
- [x] Unit tests for parser

### Media Crawler

- [x] Tempo (`tempo.co/tag/keamanan-siber`)
- [x] Kompas Tech (`tekno.kompas.com`)
- [x] Detik Inet (`inet.detik.com`)
- [x] Bisnis Indonesia (`teknologi.bisnis.com`)
- [x] Respect `robots.txt` and crawl delay for all sources
- [x] Deduplicate: same incident in multiple outlets → single record, multiple source URLs
- [x] Unit tests per outlet parser

### Normalization Pipeline

- [x] `IncidentDraft` → `Incident` normalization logic (`crates/crawler/src/normalizer.rs`)
- [x] Date parsing: handle Indonesian date formats (`8 Mei 2024`, `May 8, 2024`)
- [x] Org name normalization: `PT Bank X Tbk` → `Bank X` canonical name
- [x] Attack type classification: keyword-based rules (ML in v0.2.0)
- [x] Sector classification: keyword + org name lookup rules
- [x] Confidence scoring on each normalized field

### Verification CLI

**Core implementation (done):**

- [x] `isi review` — interactive CLI to review unverified incident records (`crates/cli/src/commands/review.rs`)
- [x] Accept / reject / edit individual fields
- [x] Mark record as `verified: true` on accept
- [x] Batch review mode for historical backfill

**Review accountability & audit trail (per REVIEW_SKILLS.md 2.5, 2.2):**

- [x] Schema addition: `review_audit_log` table with columns:
  - `id` (UUID primary key)
  - `incident_id` (FK to incidents)
  - `reviewer_id` (user/agent identifier — string, e.g., CLI username or agent name)
  - `action` (enum: ACCEPTED, REJECTED, EDITED, SKIPPED)
  - `reviewed_at` (timestamp)
  - `justification` (text field — optional but required for REJECTED, recommended for EDITED)
  - `confidence_score` (float 0.0–1.0 — reviewer's confidence in the decision)
  - `prior_status` (snapshot of record state before change — JSON)
  - `post_status` (snapshot of record state after change — JSON)
- [x] CLI enhancement: prompt reviewer for `--reviewer-id` on first invocation (cached in `.env.local` or config)
- [x] CLI enhancement: on accept/reject/edit, prompt for optional `--justification` text (e.g., "Source verified against original report", "Duplicate of incident #123")
- [x] CLI enhancement: on accept, prompt for `--confidence` (low/medium/high) or exact float 0.0–1.0. Default to medium (0.5)
- [x] Rejection must provide a justification or require `--force` flag; editing must document what changed
- [x] Batch mode (`--auto-accept`) requires `--justification "Batch: <reason>"` to be explicit; log all auto-actions to audit trail
- [x] `isi review` command enhancement: `--audit-trail <incident_id>` flag to show full review history for an incident
- [x] Documentation: extend `docs/review-discipline.md` explaining the audit trail fields and interpretation

**Re-review & escalation (per REVIEW_SKILLS.md section 3, step 8):**

- [x] Schema addition: `review_queue` table with status enum (PENDING, IN_REVIEW, ACCEPTED, REJECTED, ESCALATED)
- [x] CLI enhancement: `isi review --queue` — show incidents in review queue by status
- [x] CLI enhancement: `isi review --escalate <incident_id> --reason <text>` — move incident to ESCALATED status for human reviewer
- [x] Escalation triggers (auto-flag for human review):
  - `confidence_score < 0.3` after edit
  - Second reviewer disagrees with first (different `reviewer_id`, contradictory decisions)
  - Batch-auto-accepted incident later flagged as incorrect (defect escape)
- [x] Dashboard: `isi review --stats` — acceptance rate, rejection rate, edit rate, escalation rate, mean time-in-review per sector

**Regression testing & metrics (per REVIEW_SKILLS.md section 6 test discipline, section 8 metrics):**

- [x] Integration test suite for review CLI:
  - [x] Setup: create 10 unverified test incidents across sectors
  - [x] Test: interactive accept, reject, edit workflows (using stdin mock)
  - [x] Test: batch mode conflict detection (both `--auto-accept` and `--auto-reject` together)
  - [x] Test: missing `--reviewer-id` prompts user
  - [x] Test: audit trail logged correctly for each action
  - [x] Test: confidence_score bounds (0.0–1.0)
  - [x] Test: escalation triggered on low confidence
  - [x] Cleanup: rollback test incidents (transactional isolation with auto-rollback)
- [x] Metrics table: `review_metrics` (daily aggregates):
  - [x] `date` (DATE)
  - [x] `incidents_reviewed` (count)
  - [x] `acceptance_rate` (float 0.0–1.0)
  - [x] `edit_rate` (count edited before accept)
  - [x] `escalation_rate` (float 0.0–1.0)
  - [x] `mean_review_time_minutes` (per incident)
  - [x] `defect_escape_count` (incidents later marked incorrect)
- [x] CI job: daily metrics report to METRICS.md or dashboard URL
- [x] Monthly meta-review: audit 10% sample of review_audit_log for REVIEW_SKILLS.md compliance
  - [x] Check: are justifications substantive? Do they cite sources or schema?
  - [x] Check: confidence scores reasonable (not all 1.0 or all 0.5)?
  - [x] Check: escalations legitimate?
  - [x] Report: findings and corrective actions tracked as GH issues

**Defect-escape tracking (per REVIEW_SKILLS.md section 8 "Defect-escape rate"):**

- [x] Schema addition: `incident_correction` table:
  - [x] `id` (UUID)
  - [x] `incident_id` (FK to incidents)
  - [x] `correction_type` (enum: FACTUAL_ERROR, WRONG_SECTOR, WRONG_ATTACK_TYPE, DUPLICATE_MERGE, OTHER)
  - [x] `reported_by` (who found the error — reviewer_id or user)
  - [x] `reported_at` (timestamp)
  - [x] `original_review_id` (FK to review_audit_log entry that accepted the incorrect record)
  - [x] `description` (text)
  - [x] `resolved_at` (timestamp — when corrected)
  - [x] `resolution_action` (text — what was changed)
- [x] CLI: `isi review --mark-defect <incident_id> --type <type> --description <text>` — log that an incident was found to be incorrect after acceptance
- [ ] Metrics: defect-escape rate calculated as `incident_correction.count / total_accepted_reviews` (monthly)
- [ ] Target: defect-escape rate < 5% per month (tuned during v0.4, v0.5 betas)
- [ ] Query: `SELECT original_review_id, reviewer_id FROM review_audit_log WHERE id IN (SELECT original_review_id FROM incident_correction)` — which reviewers have the highest defect-escape rate?

**User experience & batch backfill safeguards:**

- [ ] CLI flag `--dry-run` for batch mode: print what would be accepted/rejected without committing
- [ ] CLI flag `--max-batch-size` (default 50) to prevent accidental bulk operations
- [ ] Batch mode with `--auto-accept --limit 500 --dry-run` should be safe to run; `--no-dry-run` to confirm
- [ ] Documentation: `docs/review-workflow.md` with examples of each CLI invocation and safeguards

**Review accountability & audit trail (per REVIEW_SKILLS.md 2.5, 2.2):**

- [ ] Schema addition: `review_audit_log` table with columns:
  - `id` (UUID primary key)
  - `incident_id` (FK to incidents)
  - `reviewer_id` (user/agent identifier — string, e.g., CLI username or agent name)
  - `action` (enum: ACCEPTED, REJECTED, EDITED, SKIPPED)
  - `reviewed_at` (timestamp)
  - `justification` (text field — optional but required for REJECTED, RECOMMENDED for EDITED)
  - `confidence_score` (float 0.0–1.0 — reviewer's confidence in the decision)
  - `prior_status` (snapshot of record state before change — JSON)
  - `post_status` (snapshot of record state after change — JSON)

* CLI enhancement: prompt reviewer for `--reviewer-id` on first invocation (cached in `.env.local` or config)
* CLI enhancement: on accept/reject/edit, prompt for optional `--justification` text (e.g., "Source verified against original report", "Duplicate of incident #123", "Sector classification unclear — deferred")
* CLI enhancement: on accept, prompt for `--confidence` (low/medium/high) or exact float 0.0–1.0. Default to medium (0.5).
* Rejection must provide a justification or require `--force` flag. Editing must document what changed.
* Batch mode (`--auto-accept`) requires `--justification "Batch: <reason>"` to be explicit; log all auto-actions to audit trail.
* `isi review` command enhancement: `--audit-trail <incident_id>` flag to show full review history for an incident
* Documentation: extend `docs/review-discipline.md` explaining the audit trail fields and interpretation

**Re-review & escalation (per REVIEW_SKILLS.md 3 step 8):**

- [ ] Schema addition: `review_queue` table with status enum (PENDING, IN_REVIEW, ACCEPTED, REJECTED, ESCALATED)

* CLI enhancement: `isi review --queue` — show incidents in review queue by status
* CLI enhancement: `isi review --escalate <incident_id> --reason <text>` — move incident to ESCALATED status for human reviewer
* Escalation triggers (auto-flag for human review):
  - `confidence_score < 0.3` after edit
  - Second reviewer disagrees with first (different `reviewer_id`, contradictory decisions)
  - Batch-auto-accepted incident later flagged as incorrect (defect escape)
* Dashboard: `isi review --stats` — acceptance rate, rejection rate, edit rate, escalation rate, mean time-in-review per sector

**Regression testing & metrics (per REVIEW_SKILLS.md 6 test discipline, Section 8 metrics):**

- [ ] Integration test suite for review CLI:
  - Setup: create 10 unverified test incidents across sectors
  - Test: interactive accept, reject, edit workflows (using stdin mock)
  - Test: batch mode with conflicts (both `--auto-accept` and `--auto-reject` flags together)
  - Test: missing `--reviewer-id` prompts user
  - Test: audit trail logged correctly for each action
  - Test: confidence_score bounds (0.0–1.0)
  - Test: escalation triggered on low confidence
  - Cleanup: rollback test incidents

* Metrics table: `review_metrics` (daily aggregates)
  - `date` (DATE)
  - `incidents_reviewed` (count)
  - `acceptance_rate` (float 0.0–1.0)
  - `edit_rate` (count edited before accept)
  - `escalation_rate` (float 0.0–1.0)
  - `mean_review_time_minutes` (per incident)
  - `defect_escape_count` (incidents later marked incorrect; see Known Hard Problems)
* CI job: daily metrics report to METRICS.md or dashboard URL
* Monthly meta-review: audit 10% sample of review_audit_log for compliance with REVIEW_SKILLS.md principles
  - Check: are justifications substantive? Do they cite sources or schema?
  - Check: confidence scores reasonable (i.e., not all 1.0 or all 0.5)?
  - Check: escalations legitimate?
  - Report: findings and corrective actions tracked as GH issues

**Defect-escape tracking (per REVIEW_SKILLS.md 8 "Defect-escape rate"):**

- [ ] Schema addition: `incident_correction` table
  - `id` (UUID)
  - `incident_id` (FK to incidents)
  - `correction_type` (enum: FACTUAL_ERROR, WRONG_SECTOR, WRONG_ATTACK_TYPE, DUPLICATE_MERGE, OTHER)
  - `reported_by` (who found the error — reviewer_id or user)
  - `reported_at` (timestamp)
  - `original_review_id` (FK to review_audit_log entry that accepted the incorrect record)
  - `description` (text)
  - `resolved_at` (timestamp — when corrected)
  - `resolution_action` (text — what was changed)

* CLI: `isi review --mark-defect <incident_id> --type <type> --description <text>` — log that an incident was found to be incorrect after acceptance
* Metrics: defect-escape rate calculated as `incident_correction.count / total_accepted_reviews` (monthly)
* Target: defect-escape rate < 5% per month (tuned during v0.4, v0.5 betas)
* Query: `SELECT original_review_id, reviewer_id FROM review_audit_log WHERE id IN (SELECT original_review_id FROM incident_correction)` — which reviewers have the highest defect-escape rate?

**User experience & batch backfill safeguards:**

- [ ] CLI flag `--dry-run` for batch mode: print what would be accepted/rejected without committing
- [ ] CLI flag `--max-batch-size` (default 50) to prevent accidental bulk operations

* Batch mode with `--auto-accept --limit 500 --dry-run` should be safe to run; `--no-dry-run` to confirm
* Documentation: `docs/review-workflow.md` with examples of each CLI invocation and safeguards

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

### Schema Additions

- SQL migration: add `cve_refs TEXT[] NOT NULL DEFAULT '{}'` column to `incidents` table
- Add `CHECK` constraint enforcing each entry matches `^CVE-\d{4}-\d{4,}$` regex
- Update `Incident` struct in `crates/schema/src/incident.rs` — `cve_refs: Vec<CveId>` field with `serde(default)` for backward read compatibility
- Add `CveId` newtype in `crates/schema/src/cve.rs` — wraps `String`, validates format on construction, implements `Display` and `FromStr`
- Update `schema/incident.json` JSON Schema: add `cve_refs` as optional `array<string>` with `pattern: "^CVE-\\d{4}-\\d{4,}$"`
- Field semantics: populated ONLY when source material explicitly attributes a CVE. Never inferred from `attack_type`, `actor_alias`, or temporal correlation
- Backfill task: review verified v0.1.0 records — flag any source-attributed CVE references missed during initial pipeline; populate via verification CLI
- Documentation: extend `schema/README.md` with `cve_refs` field semantics and the no-inference rule

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

### id-kev Derived Feed

First public artifact of integration with the planned vulnerability intelligence layer. Produces a regional KEV-style JSON feed listing CVEs publicly tied to verified Indonesian incidents. Lightweight, no upstream vuln substrate required — depends only on `cve_refs` populated in v0.2.0.

- Define `id_kev.json` schema:
  - `cve_id` (string, format `CVE-YYYY-NNNN+`)
  - `first_seen_in_id` (date — earliest verified Indonesian incident date with this CVE attributed)
  - `incident_count` (integer — count of verified incidents in id-siber-index referencing this CVE)
  - `sectors_affected` (array of `sector_enum` values)
  - `attribution_sources` (array of source URLs supporting the CVE attribution)
  - `last_updated` (timestamp)
- Build extractor in `crates/api/src/feeds/id_kev.rs` — scan verified incidents with non-empty `cve_refs`, group by CVE ID, emit feed entries
- Publish endpoint: `GET /v1/feed/id-kev.json` — public, no auth, daily refresh
- Mirror feed to GitHub repo as static artifact at `feeds/id-kev.json` for offline / air-gapped consumers
- Document feed semantics in `docs/id-kev-spec.md`:
  - This is _publicly attributable_ exploitation evidence, not a CISA-KEV-class authoritative source
  - Inclusion threshold: at least one verified incident with explicit source-attributed CVE
  - Removal policy: never (incident history is permanent)
- Add badge / link in `README.md` once first 5 entries populate the feed
- Acceptance criterion: 3+ Indonesian IR firm reviewers confirm feed semantics are unambiguous before promoting publicly

### PT Entity & Indonesian Procurement Readiness

Non-engineering tasks required before any paid-tier customer onboarding. BUMN, OJK-regulated entities, and government procurement explicitly require PT-entity invoicing and Indonesian-soil hosting. Without these in place the Standard / Premium / Partner tiers are technically live but commercially unsellable to the target segments.

- Establish PT (Perseroan Terbatas) entity for invoicing and contracting
- NPWP and PKP registration as applicable
- Indonesian bank account for IDR receipts; foreign-currency receipt path for cross-border Premium customers
- Local data hosting confirmed: PDN-approved CSP region or on-prem option for Standard+ tiers
- Standard contract templates: Bahasa Indonesia and English versions
- Compliance documentation pack pre-prepared:
  - ISO 27001 control mapping
  - UU PDP DPO appointment evidence
  - POJK 11/2022 third-party vendor due-diligence questionnaire answers
  - SBOM for the deployed stack
- Bukti potong (withholding tax) handling documented for Premium-tier foreign-currency contracts
- Legal review of standard contract templates by Indonesian counsel before first signature

---

## v0.4.0 — Operational Hardening + IOC Alpha + Beta Program

**Target: Month 6–8**
**Prerequisites: v0.3.0 shipped — frontend deployed, 2+ Partner agreements signed, id-kev feed live with 5+ entries, PT entity established (per v0.3.0 surgical addition).**
**Goal: Production-grade operations established. IOC pipeline running as alpha. First 3 design-partner customers signed under beta agreements.**
**Release condition: Pentest report received and findings remediated or accepted-with-rationale. 3 design partners onboarded. IOC alpha endpoint serving real data. 500+ verified incidents in database with diversified sector and source coverage.**

### v0.4.x — Patch Policy

Patches in the `0.4.x` range:

- Bug fixes from beta-customer feedback
- IOC source URL changes
- Pentest finding remediations
- Operational runbook corrections
- Audit log query optimization

Patches do NOT add new alpha endpoints, change beta-tier pricing, or onboard new beta cohort members beyond the v0.4.0 set. Those are v0.5.0.

---

### Operational Hardening: Load Testing

- Define SLO targets per endpoint: p50, p95, p99 latency budgets for `/incidents`, `/incidents/{id}`, `/search`, `/stats`
- Build load-test harness using k6 or vegeta with realistic query mix
- Generate production-equivalent dataset (cloned schema, anonymized payloads if needed)
- Run baseline test, identify bottlenecks
- Performance fixes — query optimization, index additions, caching layer if warranted
- Re-run test, document baseline numbers in `docs/performance-baseline.md`
- Acceptance: every endpoint meets defined SLO at 10x current production traffic

### Operational Hardening: Third-Party Security Audit

- Select pentest provider — local options (e.g., Spentera, NOOSC) or international (e.g., Bishop Fox, NCC Group, Trail of Bits)
- Define scope:
  - API server (auth, rate limiting, input validation, IDOR, injection)
  - Public frontend (XSS, CSRF, dependency audit)
  - OPSEC dark web pipeline (highest risk — VM isolation, GPG signing chain, Tor circuit handling)
  - Admin CLI and database access surface
- Sign engagement contract; typical engagement: 2–3 weeks
- Receive findings report; triage by severity
- Critical and High findings: fix before v0.5.0 begins
- Medium and Low findings: track in security backlog with target version
- Publish redacted summary in `SECURITY.md` to build public trust

### Operational Hardening: Incident Response Runbooks

- Runbook: API server outage (`docs/runbooks/api-outage.md`)
- Runbook: Database compromise (`docs/runbooks/db-compromise.md`)
- Runbook: Crawler abuse / scraping attack against id-siber-index (`docs/runbooks/scraping-attack.md`)
- Runbook: Suspected data poisoning or contributor abuse (`docs/runbooks/data-poisoning.md`)
- Runbook: GPG key compromise — dark web bundle signing (`docs/runbooks/gpg-compromise.md`)
- Runbook: Tor circuit failure / OPSEC compromise (`docs/runbooks/opsec-compromise.md`)
- Runbook: Defacement attempt on public frontend (`docs/runbooks/defacement.md`)
- Tabletop exercise: 2 scenarios per quarter, document outcomes

### Operational Hardening: Disaster Recovery

- Document RTO and RPO targets in `docs/dr-plan.md`
- Backup verification: full restore-from-snapshot drill on isolated infrastructure
- Off-site backup test: replication and restore from secondary region
- Database point-in-time recovery (PITR) test: target arbitrary timestamp, verify integrity
- Quarterly DR drill scheduled

### Operational Hardening: Audit Logging

- `audit_log` table migration (Standard+ tier API access)
- Logged fields: API key prefix (never full key), endpoint, query parameters, timestamp, client IP, response status
- Retention: 90 days minimum, configurable
- Customer-facing audit log endpoint: `GET /v1/audit/me` — Standard+ auth
- Privacy controls: never log full response bodies; never log query parameters that contain PII verbatim (e.g., personal name search → log hash)
- Internal admin audit endpoint: separate, requires CLI access

### Bahasa Indonesia API Documentation

- Full Bahasa Indonesia translation of API reference (not just frontend toggle)
- Examples in Bahasa for all endpoints
- Bahasa Indonesia error message catalog (`crates/api/src/errors_id.rs`)
- Translator: native speaker review — not auto-translated
- Bilingual code examples (curl, Python, Go) with Bahasa explanatory text

### IOC Alpha

- IOC schema design — review against STIX 2.1 indicator pattern syntax for forward compatibility
- Migrations and table created (per ARCHITECTURE.md surgical patches)
- IOC types in alpha: IPv4, IPv6, domain, URL, MD5, SHA1, SHA256, email
- Extraction pipeline:
  - Dark web listings → IOC table
  - Partner IR contributions → IOC table (high trust)
- Deduplication and confidence scoring rules documented
- IOC ↔ Incident linkage (FK)
- Endpoints (alpha — `/v0/` prefix, breaking changes allowed before v1.0):
  - `GET /v0/iocs?type=domain&value=<value>`
  - `GET /v0/iocs/{id}`
- IOC expiry: 90-day stale flag without re-observation
- Export: STIX-compatible IOC JSON (alpha; full STIX in v0.5)

### Beta Program

- Beta agreement template (`docs/agreements/beta-agreement.md`) — separate from Partner agreement
- Beta tier definition between Standard and Premium:
  - Early access to new features
  - Feedback expected (monthly survey + optional check-ins)
  - Reduced SLA (best-effort, not contracted)
- Beta pricing: 50% of Standard tier OR free in exchange for written feedback
- Onboarding workflow: contract signature → API key issuance → kickoff call → weekly check-in for first month → monthly thereafter
- Recruit 3 design partners across sectors:
  - 1 banking sector (commercial bank, BPD, or shariah bank)
  - 1 MSSP from v0.3 anchor partner list (Xynexis, ITSEC Asia, Vaksincom)
  - 1 fintech, e-commerce, or digital-native sector
- Feedback collection: structured monthly survey + ad-hoc Slack or email channel
- Feedback triage process: weekly internal review → backlog ticket → version assignment

### First MSSP Integration Pilot

- Concrete integration pilot with one v0.3 anchor partner
- Define integration scope:
  - Data flow: id-siber-index API → MSSP's existing SOC tooling
  - Output formats supported: JSON, CSV, STIX (alpha)
  - Authentication: scoped Partner API key
- MSSP-branded wrapper (their UI consuming our API; their customer relationship)
- Joint case study documented for v1.0 marketing
- Pilot success criteria:
  - MSSP successfully serves 3+ of their own customers using id-siber-index data
  - Feedback captured and ingested into v0.5.0 backlog
  - Reference quote secured for public marketing

### Data Correction & Takedown Workflow

- Formal incident correction request endpoint or form
- SLA: acknowledgment within 5 business days; resolution within 30 days for non-disputed
- Workflow: review → verify with original sources → update record → log change in record provenance
- Takedown policy documented in `docs/takedown-policy.md`
- Permitted takedown grounds:
  - Factual error in record (after re-verification)
  - Defamation risk on legal advice
  - Organization name change (rename, not delete)
  - Regulatory request (BSSN, PDP Agency once operational)
- Boundary: never silently delete; always retain audit trail of corrections
- Public dashboard: count of corrections processed by category (transparency)

### Volume Threshold Gate

Acceptance gates before considering v0.5.0 work:

- 500+ verified incidents in database
- Sector coverage: at least 5 distinct sectors with 20+ incidents each
- Source diversity: at least 4 source types active (IDX, BSSN, OJK, Media; Dark Web optional but tracked)
- Time coverage: incidents spanning 2020–present, not biased to last 6 months only
- If thresholds unmet: extend v0.4 patch cycle (`v0.4.x` minor), do not begin v0.5

---

## v0.5.0 — Pre-Freeze Review + STIX/TAXII Alpha + Hard Problems Closure

**Target: Month 8–10**
**Prerequisites: v0.4.0 shipped. 500+ verified incidents. 3 design partners onboarded with feedback flowing. Pentest critical and high findings remediated.**
**Goal: Schema externally reviewed and stable. STIX/TAXII shipping as alpha. Three Known Hard Problems closed. Ready to freeze API contract for v1.0.0.**
**Release condition: External schema review complete with sign-off from 2+ IR practitioners. STIX/TAXII alpha operational and validated against OpenCTI/MISP. Org-name entity resolution at scale shipping. Contributor trust scoring system live. UU PDP comprehensive legal audit complete with documented compliance posture. 5 design-partner customers in active beta. Pricing finalized.**

### v0.5.x — Patch Policy

Patches in the `0.5.x` range:

- Schema review feedback corrections (alpha endpoints still permit breaks; stable contract not yet frozen)
- STIX/TAXII compliance fixes from connector validation
- Entity resolution accuracy improvements
- Trust scoring threshold tuning
- Documentation polish on closure docs

Patches do NOT freeze the API contract — that happens at v1.0.0 only.

---

### External Schema Review

- Recruit 2+ external IR practitioners — target candidates:
  - Senior IR analysts at Indonesian commercial banks
  - MSSP CISOs from v0.3/v0.4 partner cohort
  - Regional ASEAN-CERT contacts (where relationship exists)
  - Indonesian academic researchers in cybersecurity
- NDAs signed where appropriate
- Stress-test schema against real incident records they have handled
- Schema review sessions: 2–3 working sessions, 90 minutes each
- Issue list with severity triage (Blocker / Major / Minor)
- Fix or accept-with-rationale every Blocker before v1.0.0
- Major issues: fix before v1.0 if feasible; otherwise document as v1.x.0 backlog
- Schema change moratorium during review window: no schema-affecting commits without review approval
- Document review process and outcomes in `docs/schema-review-v1.md`

### STIX 2.1 Alpha

- STIX bundle serialization in `crates/schema/src/stix.rs`
- Mappings:
  - Incident → STIX `Incident` object
  - Actor → STIX `Threat Actor` object
  - IOC → STIX `Indicator` object
  - `cve_refs` → STIX `Vulnerability` object (per Patch 1 surgical addition from prior pass)
  - Relationships: incident-attributed-to-actor, incident-uses-indicator, incident-targets-vulnerability
- Validation against STIX 2.1 spec using upstream validators
- Endpoints (alpha — `/v0/` prefix):
  - `GET /v0/export/stix/{id}` — single incident as STIX bundle
  - `GET /v0/export/stix/bundle?sector=BFSI` — filtered bundle
- Document any spec deviations explicitly

### TAXII 2.1 Alpha

- TAXII server scaffold under `/taxii2/` endpoint prefix
- Discovery endpoint per spec
- Collection endpoints: by sector, by actor, by date range
- Authentication: Premium and Partner tiers only
- Compatibility validation:
  - OpenCTI connector — full integration test with reference OpenCTI deployment
  - MISP feed — feed-format compatibility test
- Document any spec deviations
- Performance baseline: collection iteration latency

### MITRE ATT&CK Tagging Alpha

- Map common Indonesian incident patterns to ATT&CK techniques
- `attack_techniques TEXT[]` field on enriched incident records (alpha; nullable)
- Manual tagging of historical incidents in test cohort (50–100 incidents)
- ATT&CK navigator layer export for Indonesian incident subset (alpha JSON)
- TTP frequency report — top techniques observed across Indonesian incidents (markdown report in `reports/`)

### Hard Problem Closure: Org Name Entity Resolution

Currently tracked in Known Hard Problems as a v1.0.0 blocker. Resolution shipped here.

- Build entity resolution pipeline beyond simple alias table:
  - Stage 1: deterministic name normalization (legal-form stripping — `PT`, `Tbk`, `Persero`, etc.)
  - Stage 2: fuzzy matching with Levenshtein distance threshold
  - Stage 3: semantic embedding similarity (sentence-transformers, multilingual model)
  - Stage 4: manual review queue for low-confidence matches (CLI tool extension)
- Target accuracy: 95%+ on validated test set
- Test set: 200+ manually labeled organization name pairs covering BUMN, banks, fintech, common name variants
- Deploy as offline batch process initially; real-time matching deferred to v1.x.0+
- Document approach in `docs/entity-resolution.md`
- Update Known Hard Problems entry to "Resolved in v0.5.0"

### Hard Problem Closure: Contributor Trust Scoring

Currently tracked in Known Hard Problems. Resolution shipped here.

- Trust score per contributor in range 0.0 – 1.0
- Inputs to score:
  - Account age
  - Count of prior accepted contributions
  - Source URL diversity
  - Count of prior false positives
  - Manual reviewer confidence (CLI annotation)
- Threshold tiers:
  - High trust (>=0.8): contributions auto-promoted to standard verification queue
  - Medium trust (0.4–0.8): standard verification with elevated scrutiny
  - Low trust (<0.4): deeper verification queue, secondary reviewer required
- Anonymous contributions accepted at lowest trust tier
- Document scoring algorithm and tier behavior in `docs/contributor-trust.md`
- Update Known Hard Problems entry to "Resolved in v0.5.0"

### Hard Problem Closure: UU PDP Comprehensive Audit

Beyond the dark-web-specific opinion handled in v0.2.0. Comprehensive UU PDP compliance audit by Indonesian counsel.

- Engage Indonesian privacy counsel (Indonesian-licensed law firm with PDP specialization)
- Audit scope:
  - Data subject rights handling (access, correction, deletion, withdrawal of consent)
  - Cross-border data transfer compliance
  - Consent model: org-level vs individual-level data
  - Breach notification obligations
  - DPO appointment requirements
  - International data transfer mechanisms (post-RPP PDP issuance)
- Audit deliverable: written legal opinion + compliance gap list
- Implementation work: gap remediation tracked as GH issues for v1.0.0 closure
- Document compliance posture in `docs/uu-pdp-compliance.md`
- Update Known Hard Problems entry to "Initial audit complete in v0.5.0; remediation in v1.0.0"

### Pricing Finalization

- Decision: Standard tier IDR price point (anchor: market research, design-partner feedback, OJK-workflow tier-fit)
- Decision: Premium tier IDR price point
- Decision: Beta-to-Standard conversion path and pricing
- Decision: Partner tier financial model (currently free for data contributions; revisit with bidirectional value)
- Document in `docs/pricing.md` (private/internal — not committed to public repo until v1.0 launch)
- Communicate finalized pricing to beta customers 60+ days before v1.0 transition
- Standard contract templates updated to reflect finalized pricing

### Beta Cohort Expansion

- 5 design partners total (up from 3 in v0.4)
- Sector diversity check: BFSI + government/BUMN + fintech + MSSP + healthcare or telco
- Quarterly feedback synthesis sessions
- Conversion target: 3 of 5 beta partners convert to paying Standard subscribers at v1.0 launch
- Conversion incentive: locked-in beta pricing for first 12 months post-conversion
- Reference customer permissions: secure written permission from 2+ partners to be cited publicly at v1.0 launch

---

## v1.0.0 — Stable API + STIX/TAXII + Production Infrastructure

**Target: Month 10–12** _(was Month 6–9 before v0.4 / v0.5 split)_
**Prerequisites: v0.5.0 shipped. Schema externally reviewed and stable. STIX/TAXII alpha validated against OpenCTI and MISP. All three Known Hard Problems closed. 5 design-partner customers in active beta. Pricing finalized.**
**Goal: API contract frozen. Standards-compliance features (STIX, TAXII, ATT&CK, IOC) graduate from alpha to stable. Production infrastructure hardened. First paying Standard subscribers from converted beta cohort.**
**Release condition: All tasks complete. Load tested at 10x current production traffic. SLA documented. At least 5 paying Standard subscribers, with at least 3 converted from the v0.5.0 beta cohort (not cold-acquired). Final pentest delta scan against v0.4 baseline shows no new critical findings.**
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

### OJK Reporting Workflow Primitives

The compliance feature that justifies bank-tier Standard+ pricing. Banks regulated under POJK 11/2022 + SEOJK 29/2022 must submit initial cyber incident notification to OJK within 24 hours and a detailed report within 5 business days. This workstream produces data primitives — Claude/team never auto-submits.

- Map SEOJK 29/2022 incident report fields to internal `Incident` schema; document field-by-field mapping in `docs/compliance/ojk-mapping.md`
- Draft generator: `POST /v1/compliance/ojk/incident-report/draft`
  - Auth: Standard+ API key
  - Input: `incident_id` or inline incident payload
  - Output: pre-filled SEOJK 29 report draft (structured JSON + Bahasa Indonesia narrative)
  - Never auto-submits to OJK; output is input to the bank's compliance team
- Maturity self-assessment helper: `GET /v1/compliance/ojk/maturity-input`
  - Auth: Standard+ API key
  - Output: aggregated metrics by maturity dimension (governance, operations, technology, third-party risk) for the bank's annual self-assessment
- Output formats: structured JSON + Bahasa Indonesia narrative draft + PDF export
- Explicit boundary documentation: `docs/compliance-disclaimer.md`
  - Drafts are inputs, not legally binding submissions
  - No fitness-for-purpose warranty for regulatory acceptance
  - Bank's compliance team retains full responsibility for submitted reports
- Legal review of disclaimer text by Indonesian counsel before tier ships
- Reference customer validation: at least 1 Indonesian commercial bank confirms the draft format aligns with their internal OJK submission workflow before promoting feature publicly

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

**Target: Month 13–21** _(shifted from Month 10–18 by v0.4 / v0.5 insertion)_
**Goal: Active C2 infrastructure mapping, first QRIS/BI-FAST fraud signal prototype**
**Breaking changes from v1.x.x:** Graph endpoints use new response format. IOC pivot model replaces flat IOC lookup. `/v2/` prefix for new endpoints; `/v1/` remains supported until deprecation window closes.

**Scope clarification:** Vulnerability intelligence (this version) is integrated substrate within id-siber-index, not a separate sibling project. Product 1 (QRIS/BI-FAST fraud intelligence platform) remains the explicit commercial play, scoped via this milestone's payment fraud signal prototype and concluded in v4.0.0. This version expands the existing `### Vulnerability Intelligence` task block into substrate + application layers — see Patch 6.

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

_This workstream is the entry condition for Product 1 (QRIS/BI-FAST fraud intelligence platform). It establishes proof of concept and institutional relationships — it does not complete Product 1._

- [ ] Design payment fraud signal data model (separate schema from incident model)
- [ ] Identify first institutional partner willing to share anonymized QRIS fraud patterns
- [ ] Prototype cross-merchant fraud pattern detection on partner-contributed data
- [ ] Regulatory consultation: publishability of aggregated anonymized payment fraud metadata under UU PDP
- [ ] Draft data sharing agreement template for Bank Indonesia / OJK conversations
- [ ] Document prototype findings and present to BI/OJK as entry to Product 1 negotiation

### Vulnerability Intelligence

**Substrate ingestion** (must land before the application bullets below — see Known Hard Problems entry on NVD enrichment policy 2026):

- Multi-source CVE ingestion (single-source NVD insufficient post-April-2026):
  - CVE Program 5.x JSON from MITRE `cvelistV5` GitHub repo (canonical CVE identity + CNA-supplied CVSS)
  - CISA Vulnrichment ADP records (parallel CVSS/CWE/SSVC for non-KEV CVEs)
  - NVD 2.0 API for the slice still enriched (KEV + federal + EO 14028 critical software)
  - Schema validation against current CVE 5.1.x; fetcher flags drift and refuses to ingest schema-invalid records
- CISA KEV daily JSON ingestion → `kev_entries` table
- FIRST EPSS daily CSV ingestion → `epss_history` table with full time-series retention (LEV requires history; never overwrite previous days)
- Per-field provenance tracking — when sources disagree on CVSS, store all values with source + timestamp; never silent merge
- LEV (Likely Exploited Vulnerabilities) computation per NIST CSWP 41:
  - LEV variant (30-day windows over EPSS history)
  - LEV2 variant (finer-grained windows)
  - Daily refresh after EPSS ingestion
  - Stored on `cves` table as `latest_lev` and `latest_lev2`
- CVSS calculator with FIRST reference-vector fuzzing in CI (CVSS v4.0 was empirically calibrated against 270 expert-ranked equivalence sets; reference vectors are the only safe oracle)
- Schema additions (see ARCHITECTURE.md):
  - `cves` table
  - `cvss_records` table (per-source storage)
  - `epss_history` table (full daily series)
  - `kev_entries` table

**Application** (existing tasks, unchanged):

- Indonesian organization internet exposure monitoring (Shodan + Censys)
- CVE-to-Indonesian-org exposure mapping
- Identify unpatched critical infrastructure (exposed Fortinet, Citrix, Exchange)
- Responsible disclosure workflow for identified exposures

---

## v3.0.0 — Southeast Asia Expansion

**Target: Year 2 (months ~22+)** _(timeline reflects v0.4 / v0.5 insertion upstream)_
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

**Target: Year 2–3, contingent on institutional relationships established in v2.0.0–v3.0.0** _(unchanged — v4.0 was already gated on relationship maturity, not engineering throughput)_
**Goal: Formal BSSN MOU signed, Product 1 Bank Indonesia negotiations active**
**Breaking changes from v3.x.x:** BSSN-sourced records use new provenance model. Partner tier splits into Government and Commercial sub-tiers with separate endpoints.\*\*

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

_This is Product 1. Only reachable after v2.0.0 payment fraud signal prototype and regulatory consultation are complete._

- [ ] Data sharing framework with Bank Indonesia for anonymized QRIS fraud patterns
- [ ] Cross-institutional fraud signal aggregation (requires BI mandate or consortium model)
- [ ] Real-time fraud pattern feed for PSPs and acquiring banks
- [ ] OJK integration: feed into mandatory cyber incident reporting compliance system

### National Digital Firewall Integration

- [ ] IOC feed integration with Komdigi national digital firewall
- [ ] Automated IOC submission to national blocklist
- [ ] Feedback loop: blocked IOC confirmed → incident record updated

---

## v\* — Projected Long-Term Capabilities

_Directional projections only. Not committed roadmap items. Each requires validation at the preceding milestone._

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

_Valid ideas, deliberately deferred until the core pipeline is proven._

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

_Track here. Do not defer silently._

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

**NVD enrichment policy 2026**
As of April 15, 2026, NIST will only fully enrich CVEs that (a) appear in CISA KEV, (b) affect U.S. federal government software, or (c) qualify as critical software under Executive Order 14028. All other CVEs are categorized as "Not Scheduled," and the pre-March-2026 backlog has been moved to that category. NIST also stops providing its own CVSS score when a CVE Numbering Authority has already supplied one. Practical impact: as of mid-2026, ~70% of CVE-2025 entries lack full NVD enrichment, and the gap widens monthly. The vulnerability intelligence layer in v2.0.0 cannot depend on NVD as canonical CVE+CVSS source. Multi-source ingestion (CVE Program 5.x JSON from MITRE, CISA Vulnrichment ADP, NVD 2.0 API for the enriched slice) is required from day one of substrate work. Track in: `docs/vuln-source-strategy.md`.
