/* User table definition */
CREATE TABLE IF NOT EXISTS users (
    id            uuid        PRIMARY KEY DEFAULT gen_random_uuid(),
    email         text        NOT NULL UNIQUE,
    password_hash text        NOT NULL,
    email_verified_at timestamptz,
    created_at    timestamptz NOT NULL DEFAULT now(),
    edited_at     timestamptz NOT NULL DEFAULT now(),
    deleted_at    timestamptz
);

/* Initial Seed Data */
INSERT INTO users
    (id, email, password_hash, email_verified_at, created_at, edited_at, deleted_at)
VALUES
    ('00000000-0000-0000-0000-000000000000', 'user0@test.com', 'password0', NULL, '2026-01-04 22:15:06+00', '2026-01-04 22:15:06+00', NULL),
    ('00000000-0000-0000-0000-000000000001', 'user1@test.com', 'password1', NULL, '2026-01-05 13:22:56+00', '2026-01-05 13:22:56+00', NULL),
    ('00000000-0000-0000-0000-000000000002', 'user2@test.com', 'password2', NULL, '2026-01-03 02:41:06+00', '2026-01-05 14:35:30+00', NULL),
    ('00000000-0000-0000-0000-000000000003', 'user3@test.com', 'password3', NULL, '2026-01-01 21:33:03+00', '2026-01-03 11:47:58+00', NULL),
    ('00000000-0000-0000-0000-000000000004', 'user4@test.com', 'password4', NULL, '2026-01-08 16:05:18+00', '2026-01-08 19:59:06+00', NULL),
    ('00000000-0000-0000-0000-000000000005', 'user5@test.com', 'password5', NULL, '2026-01-03 02:41:06+00', '2026-01-05 14:35:30+00', NULL),
    ('00000000-0000-0000-0000-000000000006', 'user6@test.com', 'password6', NULL, '2026-01-04 02:13:37+00', '2026-01-06 07:06:29+00', NULL),
    ('00000000-0000-0000-0000-000000000007', 'user7@test.com', 'password7', NULL, '2026-01-02 08:41:10+00', '2026-01-03 12:10:50+00', NULL),
    ('00000000-0000-0000-0000-000000000008', 'user8@test.com', 'password8', NULL, '2026-01-02 18:41:18+00', '2026-01-05 09:35:22+00', NULL),
    ('00000000-0000-0000-0000-000000000009', 'user9@test.com', 'password9', NULL, '2026-01-02 18:41:18+00', '2026-01-05 09:35:22+00', NULL),
    ('00000000-0000-0000-0000-00000000000a', 'userA@test.com', 'passwordA', NULL, '2026-01-02 18:41:18+00', '2026-01-05 09:35:22+00', '2026-01-05 09:35:22+00'),
    ('00000000-0000-0000-0000-00000000000b', 'userB@test.com', 'passwordB', NULL, '2026-01-02 18:41:18+00', '2026-01-05 09:35:22+00', '2026-01-05 09:35:22+00'),
    (
        '00000000-0000-0000-0000-00000000000c',
        'anon-00000000-0000-0000-0000-00000000000c@deleted.local',
        'ANONYMIZED',
        NULL,
        '2026-01-02 18:41:18+00',
        '2026-01-05 09:35:22+00',
        '2026-01-05 09:35:22+00'
    );
