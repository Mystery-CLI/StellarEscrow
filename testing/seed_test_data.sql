-- Sample data for local / CI integration tests (no production PII).
-- Pair with `cleanup_test_data.sql` between runs.
-- Version: see TEST_DATA_VERSION

BEGIN;

-- Example: insert a synthetic audit row if your schema includes audit_log
-- INSERT INTO audit_log (id, action, actor, payload, created_at)
-- VALUES (gen_random_uuid(), 'test_seed', 'system', '{}', NOW());

COMMIT;
