# Cloud PG / OSS Runtime Migration Handoff

- title: Cloud PG / OSS Runtime Migration Handoff
- status: in_progress
- created_at: 2026-05-27
- updated_at: 2026-05-27
- owner: Codex
- related_files:
  - `config.example.yaml`
  - `crates/hone-core/src/config/server.rs`
  - `crates/hone-web-api/src/cloud_oss.rs`
  - `crates/hone-web-api/src/routes/public.rs`
  - `crates/hone-web-api/src/routes/files.rs`
  - `docs/current-plans/cloud-pg-oss-runtime-migration.md`
- related_docs:
  - `docs/current-plan.md`
  - `docs/repo-map.md`
  - `docs/runbooks/backend-deployment.md`
  - `docs/technical-spec.md`
  - `docs/wiki.md`

## Summary

The local ignored `.env` now contains the cloud runtime manifest for the managed Postgres and Aliyun OSS resources. Real credentials remain outside git. Direct local TCP to the PG host still times out, but authenticated PG access succeeds through the available SOCKS path. OSS bucket list succeeds through the same network path.

Code now has first-class `cloud.postgres` and `cloud.oss` config sections with env fallbacks. When OSS is configured, public Web uploads are written to OSS under `public-uploads/<user>/<date>/...`, the API returns `oss://bucket/key`, and `/api/public/image` / `/api/public/file` can proxy managed OSS objects. `/api/meta` reports `cloud_runtime`, `cloud_postgres`, `cloud_oss`, and `oss_file_proxy` capabilities when the runtime env is present.

## Local Dependencies Remaining

Core runtime state is not fully cloud-backed yet. These paths are still local by design:

- sessions JSON and `sessions.sqlite3`
- public Web auth sessions in the shared SQLite DB
- conversation quota JSON
- LLM audit SQLite
- portfolio JSON
- cron definitions JSON and cron history SQLite
- notification preferences JSON
- generated images and KB/data artifacts
- runtime logs
- iMessage `chat.db` when that channel is enabled

Startup now logs a redacted local-dependency summary whenever cloud runtime config is detected. If `cloud.strict_no_local_storage` or `HONE_CLOUD_STRICT_NO_LOCAL_STORAGE` is set true before PG-backed repositories are implemented, startup fails with the remaining dependency list.

## Verification

- `cargo check --workspace --all-targets --exclude hone-desktop` passed.
- `cargo test -p hone-core config::tests::config_example_avoids_stale_config_knobs` passed.
- `bun run test:web` passed: 185 tests.
- PG auth through SOCKS succeeded against database `db_bamang_research`, user `bamang_research`, PostgreSQL 17.4.
- OSS signed list-bucket through SOCKS succeeded with HTTP 200.
- `git diff --check` passed.

## Risks / Open Questions

- PG-backed implementations still need to replace the local JSON / SQLite stores before this can honestly be called “all local removed.”
- The backend currently stores only public uploads in OSS. Other generated artifacts still use local directories.
- The local desktop remote-backend health at `https://hone-claw.com/api/meta` was not revalidated in this slice.
- `.env`, `config.yaml`, `data/`, logs, and runtime backend JSON must remain untracked.
