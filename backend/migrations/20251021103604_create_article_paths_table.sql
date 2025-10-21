-- Add migration script here
ALTER TABLE articles DROP CONSTRAINT articles_pkey;

ALTER TABLE articles
ALTER COLUMN id TYPE UUID USING id::UUID;

ALTER TABLE articles
ADD PRIMARY KEY (id);

CREATE TABLE article_paths (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    path TEXT NOT NULL,
    article_id UUID NOT NULL REFERENCES articles(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE UNIQUE INDEX idx_active_path ON article_paths (path) WHERE deleted_at IS NULL;
