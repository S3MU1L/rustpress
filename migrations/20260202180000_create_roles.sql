-- Roles and RBAC (MVP)
--
-- Goals:
-- - Built-in roles: admin, editor
-- - Users can have multiple roles (user_roles join table)
-- - Admin can create additional roles via UI

CREATE TABLE IF NOT EXISTS roles
(
    id          uuid        PRIMARY KEY DEFAULT gen_random_uuid(),
    name        text        NOT NULL UNIQUE,
    description text        NOT NULL DEFAULT '',
    created_at  timestamptz NOT NULL DEFAULT now(),
    edited_at   timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS user_roles
(
    user_id     uuid        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role_id     uuid        NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    assigned_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, role_id)
);

CREATE INDEX IF NOT EXISTS idx_user_roles_user_id ON user_roles(user_id);
CREATE INDEX IF NOT EXISTS idx_user_roles_role_id ON user_roles(role_id);

-- Built-in roles
INSERT INTO roles (name, description)
VALUES
  ('admin',  'Full access. Can manage roles.'),
  ('editor', 'Can create and edit content.')
ON CONFLICT (name) DO NOTHING;

-- Default: if a user has no roles, grant 'editor'.
INSERT INTO user_roles (user_id, role_id)
SELECT u.id, r.id
FROM users u
JOIN roles r ON r.name = 'editor'
WHERE NOT EXISTS (SELECT 1 FROM user_roles ur WHERE ur.user_id = u.id)
ON CONFLICT DO NOTHING;
