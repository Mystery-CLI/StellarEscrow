-- Full-text search: add tsvector columns and GIN indexes for fast FTS

-- 1. Events: searchable on event_type, seller, buyer, trade_id from JSONB
ALTER TABLE events
    ADD COLUMN IF NOT EXISTS search_vec tsvector;

UPDATE events
SET search_vec = to_tsvector('english',
    coalesce(event_type, '') || ' ' ||
    coalesce(data->>'seller', '') || ' ' ||
    coalesce(data->>'buyer', '') || ' ' ||
    coalesce(data->>'trade_id', '') || ' ' ||
    coalesce(data->>'arbitrator', '')
);

CREATE INDEX IF NOT EXISTS idx_events_search_vec ON events USING GIN (search_vec);

-- Keep search_vec in sync on insert/update
CREATE OR REPLACE FUNCTION events_search_vec_update() RETURNS trigger AS $$
BEGIN
    NEW.search_vec := to_tsvector('english',
        coalesce(NEW.event_type, '') || ' ' ||
        coalesce(NEW.data->>'seller', '') || ' ' ||
        coalesce(NEW.data->>'buyer', '') || ' ' ||
        coalesce(NEW.data->>'trade_id', '') || ' ' ||
        coalesce(NEW.data->>'arbitrator', '')
    );
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_events_search_vec ON events;
CREATE TRIGGER trg_events_search_vec
    BEFORE INSERT OR UPDATE ON events
    FOR EACH ROW EXECUTE FUNCTION events_search_vec_update();

-- 2. User profiles: searchable on address, username_hash, verification
ALTER TABLE user_profiles
    ADD COLUMN IF NOT EXISTS search_vec tsvector;

UPDATE user_profiles
SET search_vec = to_tsvector('english',
    coalesce(address, '') || ' ' ||
    coalesce(username_hash, '') || ' ' ||
    coalesce(verification, '')
);

CREATE INDEX IF NOT EXISTS idx_user_profiles_search_vec ON user_profiles USING GIN (search_vec);

CREATE OR REPLACE FUNCTION user_profiles_search_vec_update() RETURNS trigger AS $$
BEGIN
    NEW.search_vec := to_tsvector('english',
        coalesce(NEW.address, '') || ' ' ||
        coalesce(NEW.username_hash, '') || ' ' ||
        coalesce(NEW.verification, '')
    );
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_user_profiles_search_vec ON user_profiles;
CREATE TRIGGER trg_user_profiles_search_vec
    BEFORE INSERT OR UPDATE ON user_profiles
    FOR EACH ROW EXECUTE FUNCTION user_profiles_search_vec_update();

-- 3. GIN index on events.data for fast JSONB key lookups
CREATE INDEX IF NOT EXISTS idx_events_data_gin ON events USING GIN (data);
