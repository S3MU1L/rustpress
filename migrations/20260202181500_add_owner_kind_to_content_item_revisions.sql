-- Add owner/kind metadata to revision rows (useful for auditing/scoping)

ALTER TABLE content_item_revisions
  ADD COLUMN IF NOT EXISTS owner_user_id uuid;

ALTER TABLE content_item_revisions
  ADD COLUMN IF NOT EXISTS kind text;

-- Backfill from current content_items state.
UPDATE content_item_revisions r
SET
  owner_user_id = ci.owner_user_id,
  kind = ci.kind
FROM content_items ci
WHERE r.content_item_id = ci.id
  AND (r.owner_user_id IS NULL OR r.kind IS NULL);
