-- In-tree Postgres job queue (apalis-postgres is still on sqlx 0.8).

CREATE TABLE jobs (
    id UUID PRIMARY KEY,
    kind TEXT NOT NULL,
    payload JSONB NOT NULL,
    run_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    attempts INT NOT NULL DEFAULT 0,
    max_attempts INT NOT NULL DEFAULT 5,
    last_error TEXT,
    locked_at TIMESTAMPTZ,
    locked_by TEXT,
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX jobs_due_idx ON jobs (run_at, id) WHERE completed_at IS NULL;
CREATE INDEX jobs_kind_idx ON jobs (kind);

-- At most one pending hourly cleanup, across workers and restarts.
CREATE UNIQUE INDEX jobs_cleanup_pending_idx
    ON jobs (kind)
    WHERE completed_at IS NULL AND kind = 'cleanup_expired_tokens';
