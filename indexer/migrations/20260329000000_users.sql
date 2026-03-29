-- User profiles (mirrors on-chain UserProfile)
CREATE TABLE IF NOT EXISTS user_profiles (
    address         VARCHAR(100) PRIMARY KEY,
    username_hash   VARCHAR(64)  NOT NULL,
    contact_hash    VARCHAR(64)  NOT NULL,
    avatar_hash     VARCHAR(64),
    verification    VARCHAR(20)  NOT NULL DEFAULT 'Unverified',
    two_fa_enabled  BOOLEAN      NOT NULL DEFAULT FALSE,
    registered_at   BIGINT       NOT NULL,
    updated_at      BIGINT       NOT NULL,
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

-- Key/value preferences per user
CREATE TABLE IF NOT EXISTS user_preferences (
    address VARCHAR(100) NOT NULL,
    key     VARCHAR(100) NOT NULL,
    value   TEXT         NOT NULL,
    PRIMARY KEY (address, key)
);

-- Per-user trade analytics (denormalised for fast reads)
CREATE TABLE IF NOT EXISTS user_analytics (
    address          VARCHAR(100) PRIMARY KEY,
    total_trades     INT          NOT NULL DEFAULT 0,
    trades_as_seller INT          NOT NULL DEFAULT 0,
    trades_as_buyer  INT          NOT NULL DEFAULT 0,
    total_volume     BIGINT       NOT NULL DEFAULT 0,
    completed_trades INT          NOT NULL DEFAULT 0,
    disputed_trades  INT          NOT NULL DEFAULT 0,
    cancelled_trades INT          NOT NULL DEFAULT 0,
    updated_at       TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_user_profiles_verification ON user_profiles (verification);
