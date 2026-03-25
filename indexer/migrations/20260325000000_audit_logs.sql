-- Audit log table for security, compliance, and debugging
CREATE TABLE IF NOT EXISTS audit_logs (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    -- Who performed the action
    actor       VARCHAR(100) NOT NULL,
    -- Category: security | trade | admin | governance | system
    category    VARCHAR(30)  NOT NULL,
    -- Specific action name e.g. "trade.created", "admin.pause", "auth.unauthorized"
    action      VARCHAR(100) NOT NULL,
    -- Optional resource being acted upon
    resource_type VARCHAR(50),
    resource_id   VARCHAR(100),
    -- Outcome: success | failure | denied
    outcome     VARCHAR(20)  NOT NULL DEFAULT 'success',
    -- Ledger sequence at time of action (NULL for off-chain events)
    ledger      BIGINT,
    -- Transaction hash if on-chain
    tx_hash     VARCHAR(100),
    -- Arbitrary structured context (IP, user-agent, error message, etc.)
    metadata    JSONB        NOT NULL DEFAULT '{}',
    -- Severity: info | warn | error | critical
    severity    VARCHAR(10)  NOT NULL DEFAULT 'info',
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

-- Indexes for common query patterns
CREATE INDEX IF NOT EXISTS idx_audit_actor       ON audit_logs (actor, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_audit_category    ON audit_logs (category, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_audit_action      ON audit_logs (action, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_audit_resource    ON audit_logs (resource_type, resource_id) WHERE resource_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_audit_outcome     ON audit_logs (outcome) WHERE outcome != 'success';
CREATE INDEX IF NOT EXISTS idx_audit_severity    ON audit_logs (severity) WHERE severity IN ('warn','error','critical');
CREATE INDEX IF NOT EXISTS idx_audit_created_at  ON audit_logs (created_at DESC);
CREATE INDEX IF NOT EXISTS idx_audit_ledger      ON audit_logs (ledger) WHERE ledger IS NOT NULL;

-- Retention policy: automatically delete logs older than 90 days
-- Run this periodically via a cron job or pg_cron:
-- DELETE FROM audit_logs WHERE created_at < NOW() - INTERVAL '90 days';

-- View: recent security events (denied + error/critical)
CREATE OR REPLACE VIEW audit_security_alerts AS
SELECT id, actor, category, action, resource_type, resource_id,
       outcome, severity, metadata, created_at
FROM   audit_logs
WHERE  outcome = 'denied'
   OR  severity IN ('error', 'critical')
ORDER  BY created_at DESC;

-- View: daily action summary for dashboards
CREATE OR REPLACE VIEW audit_daily_summary AS
SELECT DATE_TRUNC('day', created_at) AS day,
       category,
       action,
       outcome,
       COUNT(*)                       AS count
FROM   audit_logs
GROUP  BY 1, 2, 3, 4
ORDER  BY 1 DESC, 5 DESC;
