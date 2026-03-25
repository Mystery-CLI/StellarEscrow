-- Notification preferences per user address
CREATE TABLE IF NOT EXISTS notification_preferences (
    address         VARCHAR(64) PRIMARY KEY,
    email_enabled   BOOLEAN     NOT NULL DEFAULT FALSE,
    email_address   TEXT,
    sms_enabled     BOOLEAN     NOT NULL DEFAULT FALSE,
    phone_number    TEXT,
    push_enabled    BOOLEAN     NOT NULL DEFAULT FALSE,
    push_token      TEXT,
    -- Granular event toggles (all on by default)
    on_trade_created    BOOLEAN NOT NULL DEFAULT TRUE,
    on_trade_funded     BOOLEAN NOT NULL DEFAULT TRUE,
    on_trade_completed  BOOLEAN NOT NULL DEFAULT TRUE,
    on_trade_confirmed  BOOLEAN NOT NULL DEFAULT TRUE,
    on_dispute_raised   BOOLEAN NOT NULL DEFAULT TRUE,
    on_dispute_resolved BOOLEAN NOT NULL DEFAULT TRUE,
    on_trade_cancelled  BOOLEAN NOT NULL DEFAULT TRUE,
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Outbound notification log
CREATE TABLE IF NOT EXISTS notification_log (
    id          BIGSERIAL   PRIMARY KEY,
    address     VARCHAR(64) NOT NULL,
    channel     VARCHAR(16) NOT NULL,   -- email | sms | push
    template_id VARCHAR(64) NOT NULL,
    subject     TEXT,
    body        TEXT        NOT NULL,
    status      VARCHAR(16) NOT NULL DEFAULT 'sent',  -- sent | failed
    error       TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_notif_log_address ON notification_log (address, created_at DESC);
