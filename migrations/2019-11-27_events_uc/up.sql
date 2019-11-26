-- Add missing unique constraint as unique index
CREATE UNIQUE INDEX IF NOT EXISTS events_uc ON events (uid);