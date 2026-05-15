-- Review queue: tracks incidents under review with escalation support
-- per REVIEW_SKILLS.md principles 3.2 (escalation) and 3.5 (dashboard)

CREATE TYPE review_queue_status AS ENUM (
    'PENDING', 'IN_REVIEW', 'ACCEPTED', 'REJECTED', 'ESCALATED'
);

CREATE TABLE IF NOT EXISTS review_queue (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    incident_id     UUID NOT NULL UNIQUE REFERENCES incidents(id) ON DELETE CASCADE,
    status          review_queue_status NOT NULL DEFAULT 'PENDING',
    reviewer_id     VARCHAR(255),
    escalated_by    VARCHAR(255),
    escalation_reason TEXT,
    escalated_at    TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_review_queue_status
    ON review_queue(status);

CREATE INDEX IF NOT EXISTS idx_review_queue_escalated_by
    ON review_queue(escalated_by);
