-- File storage table for avatars, dispute evidence, and documents
CREATE TABLE IF NOT EXISTS files (
    id UUID PRIMARY KEY,
    owner_id VARCHAR(100) NOT NULL,          -- Stellar address of the uploader
    file_type VARCHAR(20) NOT NULL,          -- 'avatar' | 'evidence' | 'document'
    original_name VARCHAR(255) NOT NULL,
    stored_name VARCHAR(255) NOT NULL UNIQUE,
    mime_type VARCHAR(100) NOT NULL,
    size_bytes BIGINT NOT NULL,
    checksum VARCHAR(64) NOT NULL,           -- SHA-256 hex
    trade_id BIGINT,                         -- optional association to a trade
    is_compressed BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_files_owner ON files (owner_id);
CREATE INDEX IF NOT EXISTS idx_files_type ON files (file_type);
CREATE INDEX IF NOT EXISTS idx_files_trade ON files (trade_id) WHERE trade_id IS NOT NULL;
