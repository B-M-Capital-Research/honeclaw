# Cloud PG / OSS Runtime Migration Handoff

- title: Cloud PG / OSS Runtime Migration Handoff
- status: done
- created_at: 2026-05-27
- updated_at: 2026-05-31
- owner: Codex
- related_files:
  - `config.example.yaml`
- `crates/hone-core/src/config/server.rs`
- `crates/hone-core/src/cloud_runtime.rs`
- `bins/hone-cli/src/cloud.rs`
- `memory/src/quota.rs`
- `memory/src/session.rs`
- `memory/src/web_auth.rs`
- `memory/src/llm_audit.rs`
- `memory/src/portfolio.rs`
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
  - `docs/archive/plans/cloud-pg-oss-runtime-migration.md`
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

- KB/data artifacts and actor sandbox research docs
- runtime logs
- iMessage `chat.db` when that channel is enabled

Conversation quota is no longer a local durable dependency in `cloud.mode=cloud` when PG is configured: reserve / commit / release now use PG `conversation_quota`, and the legacy JSON files are migration input / rollback evidence only.

Session JSON is no longer a local durable dependency in `cloud.mode=cloud` when PG is configured: create / load / list / append / replace now use PG `cloud_sessions`, and the legacy JSON files are migration input / rollback evidence only.

Web auth is no longer a local durable dependency in `cloud.mode=cloud` when PG is configured: invite users, API key hashes, and public login sessions now use PG `cloud_web_invite_users` / `cloud_web_auth_sessions`, and the shared SQLite rows are migration input / rollback evidence only.

Cron definitions and execution history are no longer local durable dependencies in `cloud.mode=cloud` when PG is configured: definitions use PG `cloud_cron_jobs`, execution history uses PG `cloud_cron_job_runs`, and due-slot dedupe uses PG `cloud_cron_job_claims` before execution. Legacy cron JSON files are migration input / rollback evidence only.

Skill registry, notification prefs, portfolio state, and LLM audit records are no longer local durable dependencies in `cloud.mode=cloud` when PG is configured: global skill toggles use PG `cloud_skill_registry`, notification preferences use PG `cloud_notification_prefs`, portfolio state uses PG `cloud_portfolios`, and LLM audit uses PG `cloud_llm_audit_records`. Legacy JSON / SQLite files remain migration input / rollback evidence only.

Generated images are no longer a declared local durable dependency in `cloud.mode=cloud` when OSS is configured: response finalization uploads both sandbox-local generated images and existing `gen_images` files to OSS and returns `oss://...` markers. Local mode keeps `file://` markers and local `gen_images` behavior.

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

Skill registry PG cutover added:

- `hone_tools::skill_registry` keeps the existing local JSON API, but `HoneBotCore::new` now injects PG `CloudPgRuntime` in `cloud.mode=cloud` when PG is configured.
- Runtime/tool/Web skill reads and Web skill enabled/disabled writes now use PG `cloud_skill_registry` in cloud mode; local mode keeps `data/runtime/skill_registry.json`.
- `hone-cli cloud migrate --from-data-dir ./data --skill-registry-only --apply --json` imports a legacy runtime skill registry JSON into PG when present. On this machine the file is absent, so the verification run returned 0 changed / 0 skipped with 0 conflicts and did not overwrite any existing PG row.
- `hone-cli cloud doctor --ensure-schema --json` now reports `local_durable_dependency_count=5`; `data/runtime/skill_registry.json` disappeared from the remaining durable local dependency list.
- Validation: `cargo test --offline -p hone-tools skill_registry --lib`, `cargo test --offline -p hone-core cloud_runtime --lib`, `cargo check --offline -p hone-core -p hone-tools -p hone-channels -p hone-web-api -p hone-cli --tests`, and `HONE_CLOUD_MODE=local hone-cli cloud doctor --json` passed.

Notification prefs PG cutover added:

- `hone_event_engine::prefs::FilePrefsStorage` keeps the existing local JSON API, but `HoneBotCore::new` now injects PG `CloudPgRuntime` in `cloud.mode=cloud` when PG is configured.
- Runtime notification routing, `notification_prefs` tool edits, Web notification prefs API, schedule overview, and mainline distill now go through `cloud_notification_prefs` in cloud mode; local mode keeps `data/notif_prefs/*.json`.
- `hone-cli cloud migrate --from-data-dir ./data --notification-prefs-only --apply --json` imports legacy notification prefs JSON into PG. On this machine a final verification run counted 22 JSON files, changed 0 rows, skipped 22 rows, and had 0 conflicts.
- `hone-cli cloud doctor --ensure-schema --json` now reports `local_durable_dependency_count=4`; `data/notif_prefs` disappeared from the remaining durable local dependency list.
- Validation: `cargo test --offline -p hone-event-engine prefs --lib`, `cargo test --offline -p hone-core cloud_runtime --lib`, `cargo check --offline -p hone-core -p hone-event-engine -p hone-tools -p hone-channels -p hone-web-api -p hone-cli --tests`, and `HONE_CLOUD_MODE=local hone-cli cloud doctor --json` passed.

Portfolio PG cutover added:

- `PortfolioStorage` keeps the existing local JSON API, but `HoneBotCore::new` now injects PG `CloudPgRuntime` in `cloud.mode=cloud` when PG is configured.
- Portfolio tool, Web portfolio API, public digest/admin event-engine reads, and event-engine subscription registry refresh now go through `cloud_portfolios` in cloud mode; local mode keeps `data/portfolio/portfolio_*.json`.
- `hone-cli cloud migrate --from-data-dir ./data --portfolio-only --apply --json` imports legacy portfolio JSON into PG. On this machine a verification run counted 25 JSON files, changed 1 row, skipped 24 rows, and had 0 conflicts.
- `hone-cli cloud doctor --ensure-schema --json` now reports `local_durable_dependency_count=3`; `data/portfolio` disappeared from the remaining durable local dependency list. The remaining listed paths are `./data/agent-sandboxes`, `data/gen_images`, and `data/llm_audit.sqlite3`.
- Validation: `cargo test --offline -p hone-memory portfolio --lib`, `cargo test --offline -p hone-core cloud_runtime --lib`, `cargo check --offline -p hone-core -p hone-memory -p hone-event-engine -p hone-tools -p hone-channels -p hone-web-api -p hone-cli --tests`, and `HONE_CLOUD_MODE=local hone-cli cloud doctor --json` passed.

LLM audit PG cutover added:

- `LlmAuditStorage` keeps the existing local SQLite API, but `HoneBotCore::new` now injects PG `CloudPgRuntime` in `cloud.mode=cloud` when PG is configured.
- Runtime LLM audit writes and Web audit list/detail reads now go through `cloud_llm_audit_records` in cloud mode; local mode keeps `data/llm_audit.sqlite3`.
- `hone-cli cloud migrate --from-data-dir ./data --llm-audit-only --apply --json` imports legacy SQLite audit rows into PG in 500-row batches. On this machine a verification run counted 1028 rows, changed 0 rows, skipped 1028 rows, and had 0 conflicts.
- `hone-cli cloud doctor --ensure-schema --json` now reports `local_durable_dependency_count=2`; `data/llm_audit.sqlite3` disappeared from the remaining durable local dependency list. The remaining listed paths are `./data/agent-sandboxes` and `data/gen_images`.
- Validation: `cargo test --offline -p hone-memory llm_audit --lib`, `cargo test --offline -p hone-core cloud_runtime --lib`, `cargo check --offline -p hone-core -p hone-memory -p hone-channels -p hone-web-api -p hone-cli --tests`, and `HONE_CLOUD_MODE=local hone-cli cloud doctor --json` passed.

Generated image OSS finalization tightened:

- `response_finalizer` now uploads images already under `data/gen_images` to OSS in cloud mode instead of returning a durable local `file://` path.
- `hone-cli cloud doctor --ensure-schema --json` now reports `local_durable_dependency_count=1`; `data/gen_images` disappeared from the remaining durable local dependency list. The only listed path is `./data/agent-sandboxes`.
- Validation: `cargo test --offline -p hone-channels normalize_local_image_references --lib`, `cargo test --offline -p hone-core cloud_runtime --lib`, `cargo check --offline -p hone-core -p hone-memory -p hone-channels -p hone-web-api -p hone-cli --tests`, and `HONE_CLOUD_MODE=local hone-cli cloud doctor --json` passed.

Company profile PG cutover completed:

- `CompanyProfileStorage` keeps the existing actor-scoped Markdown API, but `HoneBotCore::new` now injects PG `CloudPgRuntime` in `cloud.mode=cloud` when PG is configured.
- Cloud mode stores `profile.md` and `events/*.md` rows in PG `cloud_company_profile_files`; local mode keeps repo-external actor sandbox files.
- Event-engine mainline distill, admin digest context, public digest context, company profile API reads, and company profile transfer/import paths now go through `CompanyProfileStorage`, so they resolve the same backend as the current mode.
- Existing `company_portrait` / native-file runner behavior remains compatible: successful response finalization scans the current actor sandbox `company_profiles/` and upserts touched Markdown files to PG in cloud mode.
- `hone-cli cloud migrate --from-data-dir ./data --company-profiles-only --apply --json` imports legacy actor-scoped company profile Markdown into PG. On this machine it counted 204 company-profile files, imported 172 actor-scoped Markdown files, and had 0 conflicts.
- `hone-cli cloud doctor --ensure-schema --json` now reports `local_durable_dependency_count=0`; `./data/agent-sandboxes` disappeared from the remaining durable local dependency list. Local mode also reports 0 local durable dependencies.
- Validation: `cargo test --offline -p hone-core cloud_runtime --lib`, `cargo test --offline -p hone-memory company_profile --lib`, `cargo test --offline -p hone-event-engine mainline_distill --lib`, `cargo test --offline -p hone-channels normalize_local_image_references --lib`, `cargo check --offline -p hone-core -p hone-memory -p hone-event-engine -p hone-channels -p hone-web-api -p hone-cli --tests`, `HONE_CLOUD_MODE=cloud cargo run --offline -p hone-cli -- cloud doctor --ensure-schema --json`, and `HONE_CLOUD_MODE=local cargo run --offline -p hone-cli -- cloud doctor --json` passed.

## Risks / Open Questions

- The migration objective is complete for current runtime durable dependencies: cloud doctor is 0 when PG/R2 are configured, and local mode remains compatible.
- Agent native-file company-profile edits sync to PG at successful response finalization. If a runner crashes or is killed before finalization, run `HONE_CLOUD_MODE=cloud hone-cli cloud migrate --from-data-dir ./data --company-profiles-only --apply --json` to backfill the local sandbox copy.
- Historical SQLite files that are not sessions / web auth / LLM audit remain counted by the broad file migrator but are not current runtime hot-path dependencies.
- The local desktop remote-backend health at `https://hone-claw.com/api/meta` was not revalidated in this slice.
- `.env`, `config.yaml`, `data/`, logs, and runtime backend JSON must remain untracked.
