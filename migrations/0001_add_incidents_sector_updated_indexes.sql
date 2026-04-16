-- Index for sector-filtered queries (e.g. WHERE org_sector = 'Healthcare')
CREATE INDEX IF NOT EXISTS idx_incidents_sector
    ON incidents(org_sector);

-- Index for incremental sync queries (e.g. WHERE updated_at >= $1)
CREATE INDEX IF NOT EXISTS idx_incidents_updated
    ON incidents(updated_at);
