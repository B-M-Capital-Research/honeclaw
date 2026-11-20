-- Hone private trading notebook v1. Apply with the existing migration role.
-- Generated from routes/trade_manager/store.rs SCHEMA; application role needs DML only.
BEGIN;
SELECT pg_advisory_xact_lock(742103821);

CREATE TABLE IF NOT EXISTS cloud_trade_workspaces (
 actor_storage_key TEXT PRIMARY KEY,
 state JSONB NOT NULL,
 revision BIGINT NOT NULL CHECK (revision > 0),
 updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE TABLE IF NOT EXISTS cloud_trade_revisions (
 actor_storage_key TEXT NOT NULL,
 id TEXT NOT NULL,
 revision BIGINT NOT NULL CHECK (revision > 0),
 state JSONB NOT NULL,
 label TEXT NOT NULL,
 request_id TEXT NOT NULL,
 request_fingerprint TEXT NOT NULL,
 created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
 PRIMARY KEY (actor_storage_key, id),
 UNIQUE (actor_storage_key, revision),
 UNIQUE (actor_storage_key, request_id)
);

COMMIT;
