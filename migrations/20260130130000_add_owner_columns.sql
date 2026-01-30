-- Add ownership columns for WordPress-like multi-user admin.
-- Existing rows remain NULL-owned (global).

ALTER TABLE content_items
  ADD COLUMN IF NOT EXISTS owner_user_id uuid;

ALTER TABLE site_templates
  ADD COLUMN IF NOT EXISTS owner_user_id uuid;

-- Content uniqueness: enforce (kind, slug) unique for global items, and (owner_user_id, kind, slug)
-- unique for user-owned items.
DROP INDEX IF EXISTS idx_content_items_kind_slug;

CREATE UNIQUE INDEX IF NOT EXISTS idx_content_items_global_kind_slug
ON content_items(kind, slug)
WHERE owner_user_id IS NULL;

CREATE UNIQUE INDEX IF NOT EXISTS idx_content_items_owner_kind_slug
ON content_items(owner_user_id, kind, slug)
WHERE owner_user_id IS NOT NULL;

-- Templates uniqueness: allow per-user templates with same name, while keeping global/built-in unique.
DO $$
BEGIN
  IF EXISTS (
    SELECT 1
    FROM pg_constraint
    WHERE conname = 'site_templates_name_key'
      AND conrelid = 'site_templates'::regclass
  ) THEN
    EXECUTE 'ALTER TABLE site_templates DROP CONSTRAINT site_templates_name_key';
  END IF;
END$$;

CREATE UNIQUE INDEX IF NOT EXISTS idx_site_templates_global_name
ON site_templates(name)
WHERE owner_user_id IS NULL;

CREATE UNIQUE INDEX IF NOT EXISTS idx_site_templates_owner_name
ON site_templates(owner_user_id, name)
WHERE owner_user_id IS NOT NULL;
