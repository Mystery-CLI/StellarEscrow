-- Dispute evidence: links uploaded files to disputes with participant/arbitrator access control
CREATE TABLE IF NOT EXISTS dispute_evidence (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    dispute_id  BIGINT NOT NULL,          -- trade_id acting as dispute identifier
    file_id     UUID NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    uploader    VARCHAR(100) NOT NULL,    -- Stellar address of uploader
    description TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_dispute_evidence_dispute ON dispute_evidence (dispute_id);
CREATE INDEX IF NOT EXISTS idx_dispute_evidence_file    ON dispute_evidence (file_id);

-- Signed download tokens (short-lived, single-use)
CREATE TABLE IF NOT EXISTS evidence_download_tokens (
    token       UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    file_id     UUID NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    requester   VARCHAR(100) NOT NULL,
    expires_at  TIMESTAMPTZ NOT NULL,
    used        BOOLEAN NOT NULL DEFAULT FALSE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_evidence_tokens_file    ON evidence_download_tokens (file_id);
CREATE INDEX IF NOT EXISTS idx_evidence_tokens_expires ON evidence_download_tokens (expires_at);
