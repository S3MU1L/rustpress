-- Sites: homepage configuration
-- homepage_type: 'posts' (show latest posts) or 'page' (show specific page)
-- homepage_page_id: UUID of page to show when homepage_type = 'page'
ALTER TABLE sites
    ADD COLUMN IF NOT EXISTS homepage_type text NOT NULL DEFAULT 'posts',
    ADD COLUMN IF NOT EXISTS homepage_page_id uuid DEFAULT NULL REFERENCES content_items(id) ON DELETE SET NULL;

CREATE INDEX IF NOT EXISTS idx_sites_homepage_page
    ON sites(homepage_page_id) WHERE homepage_page_id IS NOT NULL;
