# Boundary

`id-siber-index` is a public, AGPL-3.0-licensed civic intelligence platform. A separate proprietary commercial layer named `vulnantara` is operated by the same maintainers and consumes this project's public API. This document defines the architectural and legal boundary between the two.

This boundary exists deliberately. It is not an afterthought.

---

## Why this boundary exists

Without a commercial layer, the civic project does not survive. Indonesian cybersecurity infrastructure has limited public funding, and incident curation is human-labor-intensive. A pure-OSS-only stance has a documented failure pattern in this market: the project either burns out unfunded, gets cannibalized by a better-resourced fork that strips civic governance, or pivots to relicensing under pressure (see MongoDB, Elastic, Redis, Sentry).

A clean public/private split is the durable alternative:
- The public layer remains the civic good — generously open, community-driven, AGPL-3.0
- The private layer absorbs commercial pressure — proprietary, customer-funded, sustains the operator
- The boundary discipline prevents the public layer from drifting toward "convenient for the operator" instead of "useful for everyone"

---

## What is in `id-siber-index` (this repo)

* Indonesian cybersecurity incident schema and ingestion pipeline (IDX, BSSN, OJK, media, dark web)
* Public API tier (free, IP-rate-limited, no auth)
* `id-kev` derived feed
* Threat actor model and attribution
* IOC database (alpha → stable)
* Standards-compliant exports: STIX 2.1, TAXII 2.1, MITRE ATT&CK
* Vulnerability intelligence substrate (CVE Program, CISA Vulnrichment, NVD, KEV, EPSS, LEV)
* Bahasa Indonesia NLP enrichment pipeline
* Public search frontend
* Documentation, contributor onboarding, public runbooks

License: AGPL-3.0 for code. CC BY 4.0 for incident data.

---

## What is in `vulnantara` (private repo, separate org)

* Indonesian-context application layer over the public vulnerability intelligence substrate
* OJK reporting workflow primitives (POJK 11/2022 + SEOJK 29/2022)
* QRIS / BI-FAST payment fraud intelligence (entirely)
* Sector-weighted vulnerability prioritization tied to Indonesian incident attribution
* MSSP integration adapters and partner-specific connectors
* Premium analytics and customer dashboards
* Customer authentication, multi-tenancy, billing, contract management
* Internal admin tooling, customer support workflows

License: proprietary, All Rights Reserved. Not source-available. Not a fork of this repo.

---

## How the two interact

**API only.** The private layer consumes this repo's public HTTP API as a privileged customer. The private layer holds an API key in the same tiering structure as any other customer (Standard / Premium / Partner). It receives no special routes, no privileged data access, no reduced rate limits beyond the standard tier rules.

**No imports.** The private layer never imports any crate, module, or compilation unit from this repo as a library. AGPL-3.0's reach over imports is exactly what would contaminate the proprietary layer; the boundary is enforced architecturally, not legally.

**Data flow direction is one-way.** Public schema and data flow into the private layer (consumed via API). The private layer's schema, signals, and customer data never flow back into this repo. Schema changes in this repo are decided based on public-utility merits, never to accommodate private-layer needs.

**Schema discipline.** When the private layer needs a schema extension, it requests it through the same channels as any external customer (GitHub issue, schema proposal). The proposal is evaluated on whether it benefits the public corpus. Private-layer needs do not constitute special pleading.

**Audit logs.** The private layer's API consumption appears in this repo's audit logs the same way any commercial customer's consumption appears. There is no privileged path that bypasses logging or rate limiting.

---

## What contributors should know

You are contributing to the public, AGPL-3.0 civic project. Your contributions:
- Stay in this repo, under AGPL-3.0
- Are subject to the Contributor License Agreement (see `CLA.md`)
- Grant the project (and via the project's retained rights, the operator) the ability to use your contribution under AGPL-3.0 and to relicense in the future if the project's governance decides — but never to silently relicense without community notice
- Do not flow into the private repo's codebase. The private layer does not import your code; it consumes the API your code helps build, like any other customer

The CLA exists specifically to keep relicensing flexibility open at the public-repo governance level. It does not transfer your contributions to the private repo. The private layer would have to write its own equivalent functionality, or pay for a commercial license to your AGPL contribution if it ever wanted to import it (and then the AGPL terms would apply to whatever derivative work resulted).

---

## What auditors and journalists should know

The split is documented, intentional, and one-way. The public repo is not subsidizing a hidden private fork; the private layer is built independently and consumes the public API as a customer. Public repo governance is not influenced by private commercial pressure beyond what any large API customer's feedback would warrant.

If you find evidence that:
- The private layer imports public-repo code without honoring AGPL-3.0
- Public-repo schema decisions favor private-layer needs over public utility
- Public-tier API access is degraded for non-private customers
- The CLA has been used to silently relicense without community notice

…file a security issue or contact the maintainers directly. These are boundary violations and treating them as such is part of the project's accountability.

---

## What regulators should know

The public repo (`id-siber-index`) operates under Indonesian law and follows the Personal Data Protection Law (UU No. 27 of 2022). Incident metadata is indexed; personal data is not stored or redistributed. The dark web layer follows the published OPSEC and legal-boundary policies in `ARCHITECTURE.md`.

The private layer (`vulnantara`) is a separate entity operating commercially under standard PT entity governance, with its own UU PDP DPO, ISO 27001 trajectory, and customer-data handling protocols. Customer-bound data lives only in `vulnantara`; aggregated incident metadata in this repo is the same publicly-accessible record any researcher can query.

Regulatory inquiries about specific incidents should be directed to this repo's maintainers via `SECURITY.md` channels. Inquiries about commercial customer relationships should be directed to the `vulnantara` operating entity.

---

## What this boundary does not promise

- It does not promise the private layer will ever ship publicly. It is intentionally proprietary.
- It does not promise relicensing of the public layer. AGPL-3.0 is the current license and the CLA preserves flexibility, but relicensing would happen only with public community notice and rationale.
- It does not promise the private layer will not compete with foreign vendors. It is designed to.
- It does not promise the operator's neutrality on commercial vendor selection in Indonesia. The private layer exists to win commercial accounts.

What it does promise: the civic project keeps existing, generously open, with its governance and data accessible to anyone who wants to use, audit, fork, or contribute.

---

## Versioning

This document is part of the public repo and follows the repo's SemVer policy. Material changes to the boundary contract require a CHANGELOG entry, version bump, and 30-day public notice before taking effect.

Last reviewed: at v0.3.0 boundary establishment.
