-- Add migration script here
DROP TABLE IF EXISTS article_paths;

ALTER TABLE articles
ADD COLUMN path TEXT NOT NULL;

ALTER TABLE articles
DROP COLUMN IF EXISTS deleted_at;

CREATE UNIQUE INDEX idx_article_path ON articles (path);
