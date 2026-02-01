-- Sites: allow choosing a default template/theme per site
ALTER TABLE sites
    ADD COLUMN IF NOT EXISTS default_template text NOT NULL DEFAULT 'default';

CREATE INDEX IF NOT EXISTS idx_sites_owner_default_template
    ON sites(owner_user_id, default_template);
