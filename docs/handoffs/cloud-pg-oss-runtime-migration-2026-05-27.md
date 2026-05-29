# Cloud PG / OSS Runtime Migration Handoff

- title: Cloud PG / OSS Runtime Migration Handoff
- status: in_progress
- created_at: 2026-05-27
- updated_at: 2026-05-29
- owner: Codex
- related_files:
  - `config.example.yaml`
  - `crates/hone-core/src/config/server.rs`
  - `crates/hone-core/src/cloud_runtime.rs`
  - `bins/hone-cli/src/cloud.rs`
  - `crates/hone-tools/src/local_files.rs`
  - `crates/hone-channels/src/attachments/ingest.rs`
  - `crates/hone-channels/src/response_finalizer.rs`
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

## 2026-05-29 Update

The cloud runtime switch is now explicit:

- `cloud.mode=local` is the default and keeps existing local JSON / SQLite / filesystem runtime even when PG / OSS env vars are present.
- `cloud.mode=cloud` requires PG + OSS and is the only mode that claims cloud authority.
- `cloud.mode=auto` keeps the older env-presence behavior for development compatibility.
- `HONE_RUNTIME_ROLE=web|worker|all` is available; Web API and `hone-cli start` skip worker/channel sidecars when role is `web`.

New runtime helpers live in `hone-core::cloud_runtime`: PG schema / health / document index, OSS proxy / object helpers, actor-scoped OSS keys, `.env` loading, runtime role, and local durable dependency reporting. `/api/meta` now includes cloud mode, runtime role, PG / OSS health, cloud-authoritative status, and local durable dependency count.

`hone-cli cloud doctor --ensure-schema --json` was verified on this machine: PG through `HONE_POSTGRES_PROXY` connected, OSS through `HONE_OSS_PROXY` connected, and schema bootstrap was ensured.

`hone-cli cloud migrate --from-data-dir ./data --json` currently counts: 117 sessions, 193 uploads / attachments, 204 company profiles, 25 portfolio JSON, 23 cron JSON, 22 notification prefs, 698 quota JSON, and 50 SQLite files.

A one-file live apply succeeded and wrote OSS + PG index. The migrator now supports concurrent upload, per-object timeouts, and `--reuse-existing` retry. Full live apply completed on this machine after a retry:

- First pass: `hone-cli cloud migrate --from-data-dir ./data --upload-oss --apply --concurrency 12 --json`
- Retry pass: `hone-cli cloud migrate --from-data-dir ./data --upload-oss --apply --reuse-existing --concurrency 4 --json`
- Final result: 1282 non-SQLite durable files uploaded or reused in OSS and indexed in PG `cloud_documents`.
- Remaining: 50 SQLite files intentionally skipped for structured row-wise PG import.

## Risks / Open Questions

- PG-backed implementations still need to replace the local JSON / SQLite hot-path stores before this can honestly be called “all local removed.”
- Some durable file surfaces now have OSS paths in cloud mode: public uploads, channel attachment ingest, generated image finalization, and local file tools. Company profile / session / quota / auth / audit / cron hot paths still need dedicated PG / OSS adapters.
- SQLite structured import is pending; do not upload `llm_audit.sqlite3` as a blob as a substitute for PG audit rows. This is now the main migration gap for historical local state.
- The local desktop remote-backend health at `https://hone-claw.com/api/meta` was not revalidated in this slice.
- `.env`, `config.yaml`, `data/`, logs, and runtime backend JSON must remain untracked.
