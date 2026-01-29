-- Add ownership for per-user templates and (optionally) content.
--
-- Goals:
-- - site_templates can be built-in (global) or user-owned
-- - content_items can optionally be owned (used to resolve templates per user)

-- site_templates: add owner_user_id
ALTER TABLE site_templates
ADD COLUMN IF NOT EXISTS owner_user_id uuid;

DO $$
BEGIN
	IF NOT EXISTS (
		SELECT 1
		FROM pg_constraint
		WHERE conname = 'fk_site_templates_owner_user'
	) THEN
		ALTER TABLE site_templates
		ADD CONSTRAINT fk_site_templates_owner_user
		FOREIGN KEY (owner_user_id)
		REFERENCES users(id)
		ON DELETE CASCADE;
	END IF;
END
$$;

-- The original schema had a global UNIQUE(name). Remove it so users can reuse names.
ALTER TABLE site_templates
DROP CONSTRAINT IF EXISTS site_templates_name_key;

-- Enforce uniqueness per user, while keeping built-in names globally unique.
CREATE UNIQUE INDEX IF NOT EXISTS idx_site_templates_builtin_name
ON site_templates(name)
WHERE owner_user_id IS NULL;

CREATE UNIQUE INDEX IF NOT EXISTS idx_site_templates_owner_name
ON site_templates(owner_user_id, name)
WHERE owner_user_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_site_templates_owner
ON site_templates(owner_user_id);

-- content_items: add owner_user_id (optional for now)
ALTER TABLE content_items
ADD COLUMN IF NOT EXISTS owner_user_id uuid;

DO $$
BEGIN
	IF NOT EXISTS (
		SELECT 1
		FROM pg_constraint
		WHERE conname = 'fk_content_items_owner_user'
	) THEN
		ALTER TABLE content_items
		ADD CONSTRAINT fk_content_items_owner_user
		FOREIGN KEY (owner_user_id)
		REFERENCES users(id)
		ON DELETE SET NULL;
	END IF;
END
$$;

CREATE INDEX IF NOT EXISTS idx_content_items_owner
ON content_items(owner_user_id);
