-- tower-sessions store (in-tree Postgres backend; published sqlx-store is on sqlx 0.8).

CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    data BYTEA NOT NULL,
    expiry_date TIMESTAMPTZ NOT NULL
);

CREATE INDEX sessions_expiry_date_idx ON sessions (expiry_date);
