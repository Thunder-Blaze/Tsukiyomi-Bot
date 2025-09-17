-- Drop the user_presences table and related objects
DROP TABLE IF EXISTS user_presences CASCADE;
DROP TYPE IF EXISTS presence_status CASCADE;
DROP FUNCTION IF EXISTS update_updated_at_column() CASCADE;
