-- Add basic account security fields
ALTER TABLE users
    ADD COLUMN IF NOT EXISTS email_verified_at timestamptz DEFAULT NULL;

-- Sites (a user can have multiple sites)
CREATE TABLE IF NOT EXISTS sites
(
    id            uuid PRIMARY KEY     DEFAULT gen_random_uuid(),
    -------------------------------------------------------------
    owner_user_id uuid        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name          text        NOT NULL,
    slug          text        NOT NULL,
    status        text        NOT NULL DEFAULT 'draft',
    created_at    timestamptz NOT NULL DEFAULT now(),
    edited_at     timestamptz NOT NULL DEFAULT now(),
    published_at  timestamptz          DEFAULT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_sites_owner_slug_unique
    ON sites(owner_user_id, slug);

CREATE INDEX IF NOT EXISTS idx_sites_owner
    ON sites(owner_user_id);
