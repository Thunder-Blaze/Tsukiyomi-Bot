-- Clean up any existing schema (for fresh start)
DROP TABLE IF EXISTS presence_history CASCADE;
DROP TABLE IF EXISTS user_presences CASCADE;
DROP VIEW IF EXISTS active_users CASCADE;
DROP TYPE IF EXISTS presence_status CASCADE;
DROP TYPE IF EXISTS activity_type CASCADE;

-- Create the main user_presences table (single table, status as TEXT)
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
    
    -- Unique constraint to prevent duplicates per user per guild
    UNIQUE(user_id, guild_id)
);

-- Create indexes for better performance and to avoid slow queries
CREATE INDEX idx_user_presences_user_id ON user_presences(user_id);
CREATE INDEX idx_user_presences_guild_id ON user_presences(guild_id);
CREATE INDEX idx_user_presences_status ON user_presences(status);
CREATE INDEX idx_user_presences_last_seen ON user_presences(last_seen_at);
CREATE INDEX idx_user_presences_updated ON user_presences(updated_at);

-- Create function to automatically update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Create trigger to automatically update updated_at
CREATE TRIGGER update_user_presences_updated_at 
    BEFORE UPDATE ON user_presences 
    FOR EACH ROW 
    EXECUTE FUNCTION update_updated_at_column();
