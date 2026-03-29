-- Event indexing optimization: add category, schema_version columns and supporting indexes.
-- These columns are derived from the two-topic layout (category, event_name) emitted by the
-- Soroban contract and enable efficient category-level filtering without JSON scanning.

ALTER TABLE events
    ADD COLUMN IF NOT EXISTS category       VARCHAR(20)  NOT NULL DEFAULT 'unknown',
    ADD COLUMN IF NOT EXISTS schema_version SMALLINT     NOT NULL DEFAULT 1;

-- Backfill category from event_type for existing rows
UPDATE events SET category = CASE
    WHEN event_type IN (
        'trade_created', 'trade_funded', 'trade_completed', 'trade_confirmed',
        'trade_cancelled', 'time_released', 'metadata_updated',
        'dispute_raised', 'dispute_resolved', 'partial_resolved'
    ) THEN 'trade'
    WHEN event_type IN (
        'arbitrator_registered', 'arbitrator_removed',
        'arbitrator_rated', 'arbitrator_rep_updated'
    ) THEN 'arb'
    WHEN event_type IN (
        'fee_updated', 'fees_withdrawn', 'fees_distributed', 'custom_fee_set',
        'tier_upgraded', 'tier_downgraded', 'tier_config_updated'
    ) THEN 'fee'
    WHEN event_type IN (
        'template_created', 'template_updated', 'template_deactivated', 'template_trade'
    ) THEN 'tmpl'
    WHEN event_type IN (
        'subscribed', 'subscription_renewed', 'subscription_cancelled'
    ) THEN 'sub'
    WHEN event_type IN (
        'proposal_created', 'vote_cast', 'proposal_executed', 'delegated'
    ) THEN 'gov'
    WHEN event_type IN (
        'insurance_provider_registered', 'insurance_provider_removed',
        'insurance_purchased', 'insurance_claimed'
    ) THEN 'ins'
    WHEN event_type IN (
        'oracle_registered', 'oracle_removed',
        'oracle_price_fetched', 'oracle_unavailable'
    ) THEN 'oracle'
    ELSE 'sys'
END
WHERE category = 'unknown';

-- Backfill schema_version from the JSON payload where available
UPDATE events
SET schema_version = (data->>'v')::SMALLINT
WHERE data->>'v' IS NOT NULL
  AND (data->>'v') ~ '^\d+$';

-- Indexes for the new columns
CREATE INDEX IF NOT EXISTS idx_events_category
    ON events (category);

CREATE INDEX IF NOT EXISTS idx_events_category_ledger
    ON events (category, ledger DESC);

CREATE INDEX IF NOT EXISTS idx_events_schema_version
    ON events (schema_version);

-- Composite index for the most common query pattern: category + event_type + ledger range
CREATE INDEX IF NOT EXISTS idx_events_category_type_ledger
    ON events (category, event_type, ledger DESC);

-- Timestamp range index (already exists in initial migration, kept for reference)
-- CREATE INDEX IF NOT EXISTS idx_events_timestamp ON events (timestamp DESC);

-- Partial index for trade events (highest query volume)
CREATE INDEX IF NOT EXISTS idx_events_trade_category
    ON events (ledger DESC, timestamp DESC)
    WHERE category = 'trade';
