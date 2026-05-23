-- Test fixture: 10 unverified incidents across 5+ sectors
-- Purpose: Integration test data for review CLI workflows
-- Setup: psql -h localhost -U postgres id_siber_index < tests/fixtures/test_incidents.sql
-- Cleanup: psql -h localhost -U postgres id_siber_index < tests/fixtures/test_incidents_rollback.sql

-- Incident 1: BFSI sector - data breach
INSERT INTO incidents (
    id, org_name, org_sector, incident_date, disclosure_date,
    attack_type, data_categories, source_url, source_type, verified, created_at, updated_at
) VALUES (
    '11111111-1111-1111-1111-111111111111'::uuid,
    'Bank Utama Indonesia',
    'BFSI',
    '2024-01-15',
    '2024-02-01',
    'data_breach',
    '["customer_pii", "financial_records"]'::jsonb,
    'https://example.com/incident-1',
    'media',
    false,
    now(),
    now()
);

-- Incident 2: Healthcare sector - ransomware
INSERT INTO incidents (
    id, org_name, org_sector, incident_date, disclosure_date,
    attack_type, data_categories, source_url, source_type, verified, created_at, updated_at
) VALUES (
    '22222222-2222-2222-2222-222222222222'::uuid,
    'Rumah Sakit Pusat Jakarta',
    'Healthcare',
    '2024-02-10',
    '2024-02-15',
    'ransomware',
    '["medical_records", "patient_pii"]'::jsonb,
    'https://example.com/incident-2',
    'media',
    false,
    now(),
    now()
);

-- Incident 3: Energy sector - system disruption
INSERT INTO incidents (
    id, org_name, org_sector, incident_date, disclosure_date,
    attack_type, data_categories, source_url, source_type, verified, created_at, updated_at
) VALUES (
    '33333333-3333-3333-3333-333333333333'::uuid,
    'PT Pertamina',
    'Energy',
    '2024-01-20',
    '2024-02-05',
    'system_disruption',
    '["operational_data"]'::jsonb,
    'https://example.com/incident-3',
    'idx',
    false,
    now(),
    now()
);

-- Incident 4: Telecommunications sector - unauthorized access
INSERT INTO incidents (
    id, org_name, org_sector, incident_date, disclosure_date,
    attack_type, data_categories, source_url, source_type, verified, created_at, updated_at
) VALUES (
    '44444444-4444-4444-4444-444444444444'::uuid,
    'PT Telkom Indonesia',
    'Telecommunications',
    '2024-03-01',
    '2024-03-05',
    'unauthorized_access',
    '["customer_pii", "billing_data"]'::jsonb,
    'https://example.com/incident-4',
    'media',
    false,
    now(),
    now()
);

-- Incident 5: Technology sector - data breach
INSERT INTO incidents (
    id, org_name, org_sector, incident_date, disclosure_date,
    attack_type, data_categories, source_url, source_type, verified, created_at, updated_at
) VALUES (
    '55555555-5555-5555-5555-555555555555'::uuid,
    'PT Mitra Teknologi Digital',
    'Technology',
    '2024-02-20',
    '2024-03-02',
    'data_breach',
    '["employee_pii", "source_code"]'::jsonb,
    'https://example.com/incident-5',
    'media',
    false,
    now(),
    now()
);

-- Incident 6: Manufacturing sector - intellectual property theft
INSERT INTO incidents (
    id, org_name, org_sector, incident_date, disclosure_date,
    attack_type, data_categories, source_url, source_type, verified, created_at, updated_at
) VALUES (
    '66666666-6666-6666-6666-666666666666'::uuid,
    'PT Industri Manufaktur Maju',
    'Manufacturing',
    '2024-01-25',
    '2024-02-10',
    'intellectual_property_theft',
    '["technical_designs", "trade_secrets"]'::jsonb,
    'https://example.com/incident-6',
    'idx',
    false,
    now(),
    now()
);

-- Incident 7: Retail sector - point of sale compromise
INSERT INTO incidents (
    id, org_name, org_sector, incident_date, disclosure_date,
    attack_type, data_categories, source_url, source_type, verified, created_at, updated_at
) VALUES (
    '77777777-7777-7777-7777-777777777777'::uuid,
    'PT Retail Indonesia Besar',
    'Retail',
    '2024-03-10',
    '2024-03-15',
    'point_of_sale_compromise',
    '["customer_pii", "payment_card_data"]'::jsonb,
    'https://example.com/incident-7',
    'media',
    false,
    now(),
    now()
);

-- Incident 8: BFSI sector - credential compromise
INSERT INTO incidents (
    id, org_name, org_sector, incident_date, disclosure_date,
    attack_type, data_categories, source_url, source_type, verified, created_at, updated_at
) VALUES (
    '88888888-8888-8888-8888-888888888888'::uuid,
    'PT Bank Regional Asia',
    'BFSI',
    '2024-02-28',
    '2024-03-08',
    'credential_compromise',
    '["employee_credentials", "customer_pii"]'::jsonb,
    'https://example.com/incident-8',
    'media',
    false,
    now(),
    now()
);

-- Incident 9: Government sector - system compromise
INSERT INTO incidents (
    id, org_name, org_sector, incident_date, disclosure_date,
    attack_type, data_categories, source_url, source_type, verified, created_at, updated_at
) VALUES (
    '99999999-9999-9999-9999-999999999999'::uuid,
    'Badan Siber dan Sandi Negara',
    'Government',
    '2024-02-05',
    '2024-02-20',
    'system_compromise',
    '["classified_information"]'::jsonb,
    'https://example.com/incident-9',
    'bssn',
    false,
    now(),
    now()
);

-- Incident 10: Healthcare sector - unauthorized disclosure
INSERT INTO incidents (
    id, org_name, org_sector, incident_date, disclosure_date,
    attack_type, data_categories, source_url, source_type, verified, created_at, updated_at
) VALUES (
    'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa'::uuid,
    'Klinik Kesehatan Prima Jaya',
    'Healthcare',
    '2024-03-05',
    '2024-03-10',
    'unauthorized_disclosure',
    '["patient_pii", "medical_records"]'::jsonb,
    'https://example.com/incident-10',
    'media',
    false,
    now(),
    now()
);
