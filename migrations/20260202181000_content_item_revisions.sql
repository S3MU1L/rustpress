-- Content item revision history (backend undo/redo)
--
-- Model:
-- - content_items.current_rev points at the revision number that is currently applied
-- - content_item_revisions stores immutable snapshots (rev=1..N)
-- - When editing after an undo, redo history (rev > current_rev) is discarded

ALTER TABLE content_items
  ADD COLUMN IF NOT EXISTS current_rev integer NOT NULL DEFAULT 1;

CREATE TABLE IF NOT EXISTS content_item_revisions
(
    id                uuid        PRIMARY KEY DEFAULT gen_random_uuid(),
    content_item_id   uuid        NOT NULL REFERENCES content_items(id) ON DELETE CASCADE,
    rev               integer     NOT NULL,
    title             text        NOT NULL,
    slug              text        NOT NULL,
    content           text        NOT NULL,
    template          text        NOT NULL,
    status            text        NOT NULL,
    created_by_user_id uuid       REFERENCES users(id) ON DELETE SET NULL,
    created_at        timestamptz NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_content_item_revisions_item_rev
ON content_item_revisions(content_item_id, rev);

CREATE INDEX IF NOT EXISTS idx_content_item_revisions_item_created
ON content_item_revisions(content_item_id, created_at DESC);

-- Seed an initial rev=1 snapshot for existing content.
INSERT INTO content_item_revisions (content_item_id, rev, title, slug, content, template, status, created_by_user_id, created_at)
SELECT
  ci.id,
  1,
  ci.title,
  ci.slug,
  ci.content,
  ci.template,
  ci.status,
  ci.owner_user_id,
  ci.created_at
FROM content_items ci
ON CONFLICT (content_item_id, rev) DO NOTHING;

-- Safety: if any rows somehow have current_rev < 1, normalize.
UPDATE content_items
SET current_rev = 1
WHERE current_rev < 1;
