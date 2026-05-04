# Review Discipline & Audit Trail

This document defines how `isi review` records accountability decisions in `review_audit_log`.

## Purpose

The audit trail ensures every review decision is:

- attributable to a reviewer (`reviewer_id`)
- explainable (`justification`)
- reproducible (`prior_status` and `post_status` snapshots)
- measurable (`confidence_score`, timestamps, and action history)

## Audit Trail Fields

Each review action writes one row to `review_audit_log`:

- `id`: UUID primary key for the audit record.
- `incident_id`: incident being reviewed.
- `reviewer_id`: CLI user / agent identity that made the decision.
- `action`: one of `ACCEPTED`, `REJECTED`, `EDITED`, `SKIPPED`.
- `reviewed_at`: UTC timestamp for the action.
- `justification`: free-text rationale. Required for rejects (unless forced), recommended for edits.
- `confidence_score`: reviewer confidence from `0.0` to `1.0`.
- `prior_status`: JSON snapshot before action.
- `post_status`: JSON snapshot after action (or null for destructive actions).

## CLI Rules

### Reviewer identity

- Use `--reviewer-id <id>` to identify the reviewer explicitly.
- If omitted, CLI prompts on first invocation and reuses cached identity (config/env).

### Justification requirements

- Accept: optional justification, recommended for ambiguous records.
- Reject: justification required, or `--force` must be explicitly used.
- Edit: changes should be documented in justification text.
- Batch auto-accept: requires explicit `--justification "Batch: <reason>"`.

### Confidence scoring

- Accept flow prompts confidence as `low`, `medium`, `high`, or custom float.
- Default confidence is `medium (0.5)` when not provided.
- Numeric confidence must be within `0.0..=1.0`.

## Querying Review History

To inspect full review history for an incident:

- `isi review --audit-trail <incident_id>`

This shows chronological reviewer decisions, justifications, and confidence values.

## Interpretation Guide

- High confidence + weak justification is a review quality smell.
- Repeated edits before accept may indicate schema ambiguity or parser drift.
- Frequent forced rejects (`--force`) should be monitored and minimized.
- Batch actions should always have explicit business context in justification.

## Good Justification Examples

- `Source verified against official disclosure PDF and incident date corrected.`
- `Duplicate of incident 123e4567-e89b-12d3-a456-426614174000.`
- `Batch: historical backfill from vetted IDX archive (Q1 2024).`
