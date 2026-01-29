CREATE TABLE IF NOT EXISTS content_items (
    id            uuid        PRIMARY KEY DEFAULT gen_random_uuid(),
    kind          text        NOT NULL CHECK (kind IN ('page', 'post')),
    status        text        NOT NULL CHECK (status IN ('draft', 'published')),
    title         text        NOT NULL,
    slug          text        NOT NULL,
    content       text        NOT NULL DEFAULT '',
    template      text        NOT NULL DEFAULT 'default',
    created_at    timestamptz NOT NULL DEFAULT now(),
    edited_at     timestamptz NOT NULL DEFAULT now(),
    published_at  timestamptz
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_content_items_kind_slug
ON content_items(kind, slug);

CREATE INDEX IF NOT EXISTS idx_content_items_kind_status_created
ON content_items(kind, status, created_at DESC);
