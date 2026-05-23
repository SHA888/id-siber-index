# Review Skills & SLA

This document defines the review process discipline and quality targets for the incident review workflow.

## Defect-Escape Rate SLA

### Measurement Definition

**Defect-escape rate** measures the fraction of reviewed incidents where a quality
issue was discovered after acceptance. It indicates the effectiveness of the review
process and serves as a gate for reviewer confidence calibration.

**Formula**:
```
defect_escape_rate = (incidents_with_corrections_found / incidents_accepted) × 100%
```

Where:
- `incidents_with_corrections_found`: count of distinct incidents marked for
  correction (via `incident_correction` table) in a calendar month
- `incidents_accepted`: count of distinct incidents with `action="ACCEPTED"` in
  the same month (from `review_audit_log`)

### SLA Target and Rationale

**Target (v0.1–v0.3)**: < 5% per month

**Rationale**:
- Baseline established from Phase 1 regression testing and metrics aggregation
  (tasks 1.8–1.13).
- 5% threshold balances tolerance for legitimate edge cases (hard-to-catch issues,
  novel attacks) against signal that the review process is working.
- This SLA applies to the *aggregate* monthly rate, not per-incident or per-reviewer.

### Measurement Cadence

- **Daily**: `metrics aggregation job` (task 1.8) counts corrections and incidents
  per day, stores in `review_metrics.defect_escape_count`.
- **Monthly**: Monthly audit script aggregates daily counts into monthly rate,
  compares against SLA, flags breaches.
- **Meta-review**: Monthly meta-review audit (task 1.11) samples reviews,
  checks escalation legitimacy, and recommends corrective actions if SLA is
  breached.

### Response to Breach

If monthly defect-escape rate **≥ 5%**:

1. **Alert**: Monthly audit report flags breach and triggers review team notification.
2. **Investigate**: Check if corrections indicate systematic review gaps (e.g., one
   reviewer, one sector, one attack type) or one-off anomalies.
3. **Correct**: If systematic: adjust confidence thresholds, escalation rules, or
   reviewer training. If anomaly: document and continue monitoring.
4. **Re-measure**: Include corrective actions in next month's baseline.

### Tuning (v0.4+)

During v0.4/v0.5 betas, the SLA target is tunable based on:
- Observed baseline from v0.1–v0.3 production data
- Feedback from review team on false-positive / false-negative rates
- Changes to attack-vector distribution or reviewer skill distribution

Tuning decisions are made via RFC and documented here.

## Reviewer Quality Principles

(Future sections: confidence calibration, escalation criteria, audit review process)
