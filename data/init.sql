-- Initialize database for Indonesia Cybersecurity Incident Index

-- Create extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_trgm";

-- Create custom types
CREATE TYPE incident_severity AS ENUM ('low', 'medium', 'high', 'critical');
CREATE TYPE incident_status AS ENUM ('open', 'investigating', 'resolved', 'closed');
CREATE TYPE incident_category AS ENUM ('phishing', 'malware', 'ransomware', 'ddos', 'sql_injection', 'xss', 'social_engineering', 'brute_force', 'data_breach', 'insider_threat', 'other');

-- Create indexes for better search performance
CREATE INDEX IF NOT EXISTS idx_incident_title_gin ON incidents USING gin(title gin_trgm_ops);
CREATE INDEX IF NOT EXISTS idx_incident_description_gin ON incidents USING gin(description gin_trgm_ops);
CREATE INDEX IF NOT EXISTS idx_incident_severity ON incidents(severity);
CREATE INDEX IF NOT EXISTS idx_incident_status ON incidents(status);
CREATE INDEX IF NOT EXISTS idx_incident_category ON incidents(category);
CREATE INDEX IF NOT EXISTS idx_incident_created_at ON incidents(created_at);
CREATE INDEX IF NOT EXISTS idx_incident_updated_at ON incidents(updated_at);

-- Create full-text search configuration
CREATE TEXT SEARCH CONFIGURATION IF NOT EXISTS indonesian (COPY = english);
CREATE TEXT SEARCH CONFIGURATION IF NOT EXISTS english_search (english);

-- Create search functions
CREATE OR REPLACE FUNCTION search_incidents(query_text TEXT)
RETURNS TABLE(id UUID, title TEXT, description TEXT, severity incident_severity, status incident_status, category incident_category, created_at TIMESTAMP WITH TIME ZONE, updated_at TIMESTAMP WITH TIME ZONE, rank REAL) AS $$
BEGIN
    RETURN QUERY
    SELECT
        i.id,
        i.title,
        i.description,
        i.severity,
        i.status,
        i.category,
        i.created_at,
        i.updated_at,
        ts_rank(
            setweight(to_tsvector('english', i.title), 'A') ||
            setweight(to_tsvector('english', i.description), 'B'),
            plainto_tsquery('english', query_text)
        ) as rank
    FROM incidents i
    WHERE
        to_tsvector('english', i.title || ' ' || i.description) @@ plainto_tsquery('english', query_text)
    ORDER BY rank DESC;
END;
$$ LANGUAGE plpgsql;

-- Create sample data (optional for development)
INSERT INTO incidents (id, title, description, severity, status, category, source_url, reported_at, created_at, updated_at) VALUES
    (uuid_generate_v4(), 'Phishing Attack on Indonesian Bank', 'Customers of Bank Central Asia reported receiving phishing emails attempting to steal credentials.', 'high', 'investigating', 'phishing', 'https://example.com/bca-phishing', NOW(), NOW(), NOW()),
    (uuid_generate_v4(), 'Ransomware Attack on Government Agency', 'Ministry of Finance systems infected with ransomware, demanding payment in Bitcoin.', 'critical', 'investigating', 'ransomware', 'https://example.gov/ransomware', NOW(), NOW(), NOW())
ON CONFLICT DO NOTHING;

-- Create views for common queries
CREATE OR REPLACE VIEW incident_summary AS
SELECT
    status,
    category,
    severity,
    COUNT(*) as count,
    DATE(created_at) as date
FROM incidents
GROUP BY status, category, severity, DATE(created_at)
ORDER BY date DESC;

-- Grant permissions (adjust as needed)
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO id_siber;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO id_siber;

-- Create notification triggers (optional)
CREATE OR REPLACE FUNCTION notify_incident_change() RETURNS TRIGGER AS $$
BEGIN
    PERFORM pg_notify('incident_change',
        json_build_object(
            'id', NEW.id,
            'action', TG_OP,
            'title', NEW.title,
            'severity', NEW.severity,
            'status', NEW.status
        )::text
    );
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER incident_notification
    AFTER INSERT OR UPDATE ON incidents
    FOR EACH ROW
    EXECUTE FUNCTION notify_incident_change();
