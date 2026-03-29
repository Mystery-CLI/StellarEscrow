-- Compliance checks: KYC + AML results per address/trade
CREATE TABLE IF NOT EXISTS compliance_checks (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    address         TEXT NOT NULL,
    trade_id        BIGINT,
    kyc_result      JSONB NOT NULL DEFAULT '{}',
    aml_result      JSONB NOT NULL DEFAULT '{}',
    status          TEXT NOT NULL DEFAULT 'pending',
    risk_score      INTEGER NOT NULL DEFAULT 0 CHECK (risk_score BETWEEN 0 AND 100),
    notes           TEXT,
    checked_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    reviewed_by     TEXT,
    reviewed_at     TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_compliance_address ON compliance_checks (address);
CREATE INDEX IF NOT EXISTS idx_compliance_trade_id ON compliance_checks (trade_id);
CREATE INDEX IF NOT EXISTS idx_compliance_status ON compliance_checks (status);
CREATE INDEX IF NOT EXISTS idx_compliance_checked_at ON compliance_checks (checked_at DESC);
CREATE INDEX IF NOT EXISTS idx_compliance_risk_score ON compliance_checks (risk_score DESC);
