/* User table */
CREATE TABLE IF NOT EXISTS "users"
(
    id              uuid PRIMARY KEY     DEFAULT gen_random_uuid(),
    ---------------------------------------------------------------
    email           text UNIQUE NOT NULL,
    password_hash   text        NOT NULL,
    created_at      timestamptz NOT NULL DEFAULT now(),
    edited_at       timestamptz NOT NULL DEFAULT now(),
    deleted_at      timestamptz          DEFAULT NULL
);

/*  */
CREATE TABLE IF NOT EXISTS error_logs
(
    id            uuid PRIMARY KEY     DEFAULT gen_random_uuid(),
    -------------------------------------------------------------
    occurred_at   timestamptz NOT NULL DEFAULT now(),
    location      text        NOT NULL,
    parameters    jsonb
);

CREATE INDEX idx_error_logs_occurred_at ON error_logs(occurred_at DESC);
