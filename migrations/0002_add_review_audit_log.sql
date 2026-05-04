-- Review audit log: tracks every review action with accountability
-- per REVIEW_SKILLS.md principles 2.2 (citation) and 2.5 (signing)

CREATE TABLE IF NOT EXISTS review_audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    incident_id UUID NOT NULL REFERENCES incidents(id) ON DELETE CASCADE,
    reviewer_id VARCHAR(255) NOT NULL,
    action VARCHAR(255) NOT NULL CHECK (action IN ('ACCEPTED', 'REJECTED', 'EDITED', 'SKIPPED')),
    reviewed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    justification TEXT,
    confidence_score REAL CHECK (confidence_score >= 0.0 AND confidence_score <= 1.0),
    prior_status JSONB,
    post_status JSONB
);

-- Indexes for common query patterns
CREATE INDEX IF NOT EXISTS idx_review_audit_log_incident_id
    ON review_audit_log(incident_id);

CREATE INDEX IF NOT EXISTS idx_review_audit_log_reviewer_id
    ON review_audit_log(reviewer_id);

CREATE INDEX IF NOT EXISTS idx_review_audit_log_reviewed_at
    ON review_audit_log(reviewed_at DESC);

CREATE INDEX IF NOT EXISTS idx_review_audit_log_action
    ON review_audit_log(action);
