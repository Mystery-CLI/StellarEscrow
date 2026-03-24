-- Create fraud_alerts table
CREATE TABLE IF NOT EXISTS fraud_alerts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    trade_id BIGINT NOT NULL,
    risk_score INT NOT NULL CHECK (risk_score >= 0 AND risk_score <= 100),
    rules_triggered JSONB NOT NULL, -- List of rules and their individual scores
    ml_score FLOAT,
    analysis_metadata JSONB, -- Contextual data like velocity, amounts, etc.
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create fraud_reviews table
CREATE TABLE IF NOT EXISTS fraud_reviews (
    trade_id BIGINT PRIMARY KEY,
    status VARCHAR(20) NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'approved', 'rejected')),
    reviewer VARCHAR(100),
    review_notes TEXT,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create fraud_blacklist table
CREATE TABLE IF NOT EXISTS fraud_blacklist (
    address VARCHAR(100) PRIMARY KEY,
    reason TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for efficient lookup
CREATE INDEX IF NOT EXISTS idx_fraud_alerts_trade_id ON fraud_alerts (trade_id);
CREATE INDEX IF NOT EXISTS idx_fraud_alerts_score ON fraud_alerts (risk_score DESC);
CREATE INDEX IF NOT EXISTS idx_fraud_reviews_status ON fraud_reviews (status);
