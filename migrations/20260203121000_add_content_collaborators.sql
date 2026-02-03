-- Per-content-item collaborators (viewer/editor).

CREATE TABLE IF NOT EXISTS content_item_collaborators (
    content_item_id     uuid        NOT NULL REFERENCES content_items(id) ON DELETE CASCADE,
    user_id             uuid        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role                text        NOT NULL CHECK (role IN ('viewer', 'editor')),
    invited_by_user_id  uuid        REFERENCES users(id) ON DELETE SET NULL,
    created_at          timestamptz NOT NULL DEFAULT now(),

    PRIMARY KEY (content_item_id, user_id)
);

CREATE INDEX IF NOT EXISTS idx_content_item_collaborators_user
ON content_item_collaborators(user_id);

CREATE INDEX IF NOT EXISTS idx_content_item_collaborators_item
ON content_item_collaborators(content_item_id);
