-- Migration: create review_audit_log table
CREATE TABLE review_audit_log (
    id UUID PRIMARY KEY,
    incident_id UUID NOT NULL REFERENCES incidents(id) ON DELETE CASCADE,
    reviewer_id TEXT NOT NULL,
    action TEXT NOT NULL CHECK (action IN ('ACCEPTED','REJECTED','EDITED','SKIPPED')),
    reviewed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    justification TEXT,
    confidence_score REAL CHECK (confidence_score BETWEEN 0.0 AND 1.0),
    prior_status JSONB,
    post_status JSONB
);
