-- Remove rows created by seed_test_data.sql (adjust table names to match your schema).

BEGIN;

-- DELETE FROM audit_log WHERE actor = 'system' AND action = 'test_seed';

COMMIT;
