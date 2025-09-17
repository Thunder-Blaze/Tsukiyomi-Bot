-- Restore the previous schema (this is a destructive down migration)
DROP TABLE IF EXISTS user_presences CASCADE;

-- Recreate the previous schema
CREATE TABLE user_presences (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL,
    guild_id BIGINT NOT NULL,
    status TEXT NOT NULL,
    activity_name TEXT,
    activity_type TEXT,
    last_seen_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, guild_id)
);
