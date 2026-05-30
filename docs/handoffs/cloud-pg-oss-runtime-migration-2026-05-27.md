# Cloud PG / OSS Runtime Migration Handoff

- title: Cloud PG / OSS Runtime Migration Handoff
- status: in_progress
- created_at: 2026-05-27
- updated_at: 2026-05-30
- owner: Codex
- related_files:
  - `config.example.yaml`
- `crates/hone-core/src/config/server.rs`
- `crates/hone-core/src/cloud_runtime.rs`
- `bins/hone-cli/src/cloud.rs`
- `memory/src/quota.rs`
- `memory/src/session.rs`
- `memory/src/web_auth.rs`
- `memory/src/cron_job/mod.rs`
- `memory/src/cron_job/storage.rs`
- `memory/src/cron_job/history.rs`
- `crates/hone-tools/src/cron_job_tool.rs`
- `crates/hone-tools/src/notification_prefs_tool.rs`
- `crates/hone-tools/src/schedule_view.rs`
- `crates/hone-channels/src/core/bot_core.rs`
- `crates/hone-web-api/src/lib.rs`
- `crates/hone-web-api/src/routes/schedule.rs`
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

The local ignored `.env` now contains the cloud runtime manifest for the managed Postgres and object storage resources. Runtime `HONE_OSS_*` currently points at Cloudflare R2 because live benchmarks show materially faster writes than Aliyun OSS on this machine; the previous Aliyun OSS settings are preserved under `HONE_ALIYUN_OSS_*` for rollback and comparison. Real credentials remain outside git. Direct local TCP to the PG host still times out, but authenticated PG access succeeds through the available SOCKS path. Object-store health succeeds through the same network path.

Code now has first-class `cloud.postgres` and `cloud.oss` config sections with env fallbacks. `cloud.oss.provider` supports `aliyun_oss`, `r2`, and S3-compatible endpoints; `.env` loading can be set to override stale parent env with `HONE_DOTENV_OVERRIDE=true`. When object storage is configured, public Web uploads are written under `public-uploads/<user>/<date>/...`, the API returns `oss://bucket/key`, and `/api/public/image` / `/api/public/file` can proxy managed objects. `/api/meta` reports `cloud_runtime`, `cloud_postgres`, `cloud_oss`, and `oss_file_proxy` capabilities when the runtime env is present.

## Local Dependencies Remaining

Core runtime state is not fully cloud-backed yet. These paths are still local by design:

- LLM audit SQLite
- portfolio JSON
- notification preferences JSON
- generated images and KB/data artifacts
- runtime logs
- iMessage `chat.db` when that channel is enabled

Conversation quota is no longer a local durable dependency in `cloud.mode=cloud` when PG is configured: reserve / commit / release now use PG `conversation_quota`, and the legacy JSON files are migration input / rollback evidence only.

Session JSON is no longer a local durable dependency in `cloud.mode=cloud` when PG is configured: create / load / list / append / replace now use PG `cloud_sessions`, and the legacy JSON files are migration input / rollback evidence only.

Web auth is no longer a local durable dependency in `cloud.mode=cloud` when PG is configured: invite users, API key hashes, and public login sessions now use PG `cloud_web_invite_users` / `cloud_web_auth_sessions`, and the shared SQLite rows are migration input / rollback evidence only.

Cron definitions and execution history are no longer local durable dependencies in `cloud.mode=cloud` when PG is configured: definitions use PG `cloud_cron_jobs`, execution history uses PG `cloud_cron_job_runs`, and due-slot dedupe uses PG `cloud_cron_job_claims` before execution. Legacy cron JSON files are migration input / rollback evidence only.

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

Cloudflare R2 comparison added:

- Current runtime `HONE_OSS_*` points to R2; Aliyun OSS remains available through `HONE_ALIYUN_OSS_*`.
- `hone-cli cloud object-bench --size-kib 256 --iterations 3 --json` through proxy: Aliyun average PUT / HEAD / GET was 12284ms / 584ms / 1958ms; R2 was 1062ms / 217ms / 1180ms.
- `hone-cli cloud object-bench --size-kib 1024 --iterations 3 --json` through proxy: Aliyun average PUT / HEAD / GET was 5594ms / 470ms / 4811ms; R2 was 3358ms / 235ms / 4921ms.
- Conclusion for this machine: R2 should remain the runtime object store for now; Aliyun is retained as fallback because network/proxy behavior can vary.

Conversation quota PG cutover added:

- `HoneBotCore::new` selects `ConversationQuotaStorage::new_cloud(...)` in `cloud.mode=cloud` when PG is configured; local mode keeps the existing JSON store.
- PG reserve / commit / release are implemented in `hone-core::cloud_runtime` against `conversation_quota`.
- `hone-cli cloud migrate --from-data-dir ./data --quota-only --apply --json` imports legacy quota JSON into PG with a single JSONB batch upsert.
- Live result on this machine: first apply imported 698 quota JSON rows with 366 changed / 332 skipped; second apply changed 0 / skipped 698 with 0 conflicts.
- `hone-cli cloud doctor --ensure-schema --json` now reports `local_durable_dependency_count=9`; quota disappeared from the remaining durable local dependency list.
- Validation: `cargo test -p hone-core cloud_runtime --lib`, `cargo test -p hone-memory quota --lib`, and `cargo check --workspace --all-targets --exclude hone-desktop` passed.

Session PG cutover added:

- `HoneBotCore::new` selects `SessionStorage::new_cloud(...)` in `cloud.mode=cloud` when PG is configured; local mode keeps the existing JSON / SQLite behavior.
- `hone-web-api` Feishu contact recovery now also reads sessions from PG in cloud mode.
- PG session load / list / upsert are implemented in `hone-core::cloud_runtime` against `cloud_sessions`.
- `hone-cli cloud migrate --from-data-dir ./data --session-only --apply --json` imports legacy session JSON into PG with a single JSONB batch upsert.
- Live result on this machine: first apply imported 117 session JSON rows; second apply changed 0 / skipped 117 with 0 conflicts.
- `hone-cli cloud doctor --ensure-schema --json` now reports `local_durable_dependency_count=8`; `sessions_dir` disappeared from the remaining durable local dependency list.
- Validation: `cargo test -p hone-memory session --lib`, `cargo test -p hone-core cloud_runtime --lib`, and `cargo check --workspace --all-targets --exclude hone-desktop` passed.

Web auth PG cutover added:

- `hone-web-api` selects `WebAuthStorage::new_cloud(...)` in `cloud.mode=cloud` when PG is configured; local mode keeps the existing SQLite behavior.
- PG invite-user and auth-session read / write helpers are implemented in `hone-core::cloud_runtime`.
- `hone-cli cloud migrate --from-data-dir ./data --web-auth-only --apply --json` imports legacy SQLite invite users and auth sessions into PG.
- Live result on this machine: first apply imported 30 web invite users and 3 auth sessions; second apply changed 0 users / 0 sessions and skipped 30 users / 3 sessions with 0 conflicts.
- `hone-cli cloud doctor --ensure-schema --json` now reports `local_durable_dependency_count=7`; `sessions.sqlite3` disappeared from the remaining durable local dependency list.
- Validation: `cargo test -p hone-memory web_auth --lib`, `cargo test -p hone-core cloud_runtime --lib`, and `cargo check --workspace --all-targets --exclude hone-desktop` passed.

Cron PG cutover added:

- `HoneBotCore::cron_job_storage()` selects `CronJobStorage::new_cloud(...)` in `cloud.mode=cloud` when PG is configured; local mode keeps JSON definitions plus SQLite execution history. Scheduler, admin cron API, the `cron_job` tool, notification overview, and schedule overview now all go through the cloud-aware cron storage path.
- PG cron definition / execution-history / due-claim helpers are implemented in `hone-core::cloud_runtime` against `cloud_cron_jobs`, `cloud_cron_job_runs`, and `cloud_cron_job_claims`.
- `hone-cli cloud migrate --from-data-dir ./data --cron-only --apply --json` imports legacy cron JSON definitions into PG.
- Live result on this machine: first apply imported 54 cron jobs from 23 cron JSON files; second apply changed 0 / skipped 54 with 0 conflicts.
- `hone-cli cloud doctor --ensure-schema --json` now reports `local_durable_dependency_count=6`; `cron_jobs_dir` disappeared from the remaining durable local dependency list.
- Validation: `cargo test -p hone-memory cron_job --lib`, `cargo test -p hone-core cloud_runtime --lib`, and `cargo check --workspace --all-targets --exclude hone-desktop` passed.

## Risks / Open Questions

- PG-backed implementations still need to replace the remaining local JSON / SQLite hot-path stores before this can honestly be called “all local removed.”
- Some durable file surfaces now have OSS paths in cloud mode: public uploads, channel attachment ingest, generated image finalization, and local file tools. Company profile / audit / portfolio / notification prefs hot paths still need dedicated PG / OSS adapters; sessions, web auth, quota, and cron are now cut over to PG in cloud mode.
- SQLite structured import is pending; do not upload `llm_audit.sqlite3` as a blob as a substitute for PG audit rows. This is now the main migration gap for historical local state.
- The local desktop remote-backend health at `https://hone-claw.com/api/meta` was not revalidated in this slice.
- `.env`, `config.yaml`, `data/`, logs, and runtime backend JSON must remain untracked.
