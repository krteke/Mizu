-- Add migration script here
ALTER TABLE articles
ADD COLUMN deleted_at TIMESTAMPTZ;
