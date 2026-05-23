-- Rollback script for test fixture incidents
-- Removes the 10 test incidents created by test_incidents.sql
-- Usage: psql -h localhost -U postgres id_siber_index < tests/fixtures/test_incidents_rollback.sql

-- Delete test incidents by their UUIDs
DELETE FROM incidents WHERE id IN (
    '11111111-1111-1111-1111-111111111111'::uuid,
    '22222222-2222-2222-2222-222222222222'::uuid,
    '33333333-3333-3333-3333-333333333333'::uuid,
    '44444444-4444-4444-4444-444444444444'::uuid,
    '55555555-5555-5555-5555-555555555555'::uuid,
    '66666666-6666-6666-6666-666666666666'::uuid,
    '77777777-7777-7777-7777-777777777777'::uuid,
    '88888888-8888-8888-8888-888888888888'::uuid,
    '99999999-9999-9999-9999-999999999999'::uuid,
    'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa'::uuid
);
