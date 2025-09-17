-- Drop the existing table and recreate with simplified schema
DROP TABLE IF EXISTS user_presences CASCADE;

-- Create simplified user presences table
CREATE TABLE user_presences (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL UNIQUE,
    status TEXT NOT NULL,
    last_seen_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create index for performance
CREATE INDEX idx_user_presences_updated_at ON user_presences(updated_at DESC);
CREATE INDEX idx_user_presences_user_id ON user_presences(user_id);
