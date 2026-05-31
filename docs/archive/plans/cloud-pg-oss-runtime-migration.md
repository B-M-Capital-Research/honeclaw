# Cloud PG / OSS Runtime Migration

- title: Cloud PG / OSS Runtime Migration
- status: done
- created_at: 2026-05-27
- updated_at: 2026-05-31
- owner: Codex
- related_files:
  - `config.example.yaml`
  - `crates/hone-core/src/config/server.rs`
  - `crates/hone-core/src/config/mod.rs`
  - `crates/hone-core/src/cloud_runtime.rs`
  - `bins/hone-cli/src/cloud.rs`
  - `memory/src/quota.rs`
  - `memory/src/session.rs`
  - `memory/src/web_auth.rs`
  - `memory/src/llm_audit.rs`
  - `memory/src/portfolio.rs`
  - `memory/src/company_profile/storage.rs`
  - `memory/src/company_profile/types.rs`
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
  - `crates/hone-web-api/src/routes/public.rs`
  - `crates/hone-web-api/src/routes/files.rs`
  - `crates/hone-web-api/src/routes/event_engine_admin.rs`
  - `crates/hone-web-api/src/routes/public_digest.rs`
  - `crates/hone-event-engine/src/global_digest/mainline_distill.rs`
  - `crates/hone-web-api/src/cloud_oss.rs`
  - `.env` (local only, ignored)
- related_docs:
  - `docs/current-plan.md`
  - `docs/repo-map.md`
  - `docs/runbooks/backend-deployment.md`
  - `docs/handoffs/cloud-pg-oss-runtime-migration-2026-05-27.md`
  - `docs/handoffs/cloud-runtime-impact-report-2026-05-28.md`

## Goal

Move the local runtime toward explicit local/cloud storage switching: default to local mode, use PG for cloud schema/index/locks and OSS for durable file objects when `cloud.mode=cloud`, and make remaining local durable dependencies visible so follow-up PG-backed store work is not hidden.

## Scope

- Add explicit `cloud.mode=local|cloud|auto`, where `local` is the default and PG/OSS env presence alone no longer hijacks local storage mode.
- Add `HONE_RUNTIME_ROLE=web|worker|all` and gate Web API scheduler/event-engine/channel sidecar startup for web-only deployments.
- Add PG/OSS runtime helpers, proxy support, schema bootstrap, `/api/meta` health fields, and `hone-cli cloud doctor`.
- Add `hone-cli cloud migrate` dry-run/apply: recognized durable files upload to actor-scoped OSS document keys and are indexed in PG `cloud_documents`; SQLite blobs are counted but skipped until structured table import lands.
- Switch conversation quota hot path to PG in `cloud.mode=cloud`: reserve / commit / release now use PG rows instead of local JSON, and `hone-cli cloud migrate --quota-only --apply` imports legacy quota JSON idempotently.
- Switch session hot path to PG in `cloud.mode=cloud`: `SessionStorage` writes / loads / lists `cloud_sessions` instead of local JSON files, and `hone-cli cloud migrate --session-only --apply` imports legacy session JSON idempotently.
- Switch Web invite users, API keys, and public login sessions to PG in `cloud.mode=cloud`: `WebAuthStorage` uses `cloud_web_invite_users` / `cloud_web_auth_sessions`, and `hone-cli cloud migrate --web-auth-only --apply` imports the legacy SQLite auth tables idempotently.
- Switch cron definitions, execution history, and due-job claims to PG in `cloud.mode=cloud`: `CronJobStorage::new_cloud` uses `cloud_cron_jobs`, `cloud_cron_job_runs`, and `cloud_cron_job_claims`, while local mode keeps JSON definitions plus SQLite execution history. Scheduler, admin cron API, `cron_job` tool, channel-target directory, and schedule overview now all resolve cron through the same cloud-aware storage factory.
- Switch skill registry to PG in `cloud.mode=cloud`: `hone_tools::skill_registry` uses injected PG `cloud_skill_registry` for global enabled/disabled overrides, while local mode keeps `data/runtime/skill_registry.json`.
- Switch notification preferences to PG in `cloud.mode=cloud`: `hone_event_engine::prefs::FilePrefsStorage` uses injected PG `cloud_notification_prefs` for runtime/tool/Web reads and writes, while local mode keeps `data/notif_prefs/*.json`.
- Switch portfolio storage to PG in `cloud.mode=cloud`: `PortfolioStorage` uses injected PG `cloud_portfolios` for tool/Web/event-engine reads and writes, while local mode keeps `data/portfolio/portfolio_*.json`.
- Switch LLM audit storage to PG in `cloud.mode=cloud`: `LlmAuditStorage` uses injected PG `cloud_llm_audit_records` for runtime writes and Web audit list/detail reads, while local mode keeps `data/llm_audit.sqlite3`.
- Switch company profile storage to PG in `cloud.mode=cloud`: `CompanyProfileStorage` uses injected PG `cloud_company_profile_files` for actor-scoped `profile.md` / `events/*.md`, while local mode keeps actor sandbox files. Agent native-file edits are synced to PG during response finalization so the existing `company_portrait` file workflow remains compatible.
- Add S3-compatible object-store support for Cloudflare R2. The local ignored `.env` currently points runtime `HONE_OSS_*` at R2, preserves Aliyun OSS under `HONE_ALIYUN_OSS_*` for rollback / benchmark comparison, and enables dotenv override so stale shell env cannot silently force the old provider.
- Switch local file tools to use actor-scoped OSS namespace when cloud mode is authoritative; keep local sandbox walk/read/search in local mode.
- Upload channel attachments and generated images to OSS in cloud mode where the current call site has enough context; local mode keeps current filesystem behavior. Generated image finalization now uploads both sandbox-local images and existing `gen_images` files to OSS before returning the assistant-visible marker.

## Validation

- `cargo check --workspace --all-targets --exclude hone-desktop`
- Targeted config / web-api tests where practical.
- Manual PG / OSS health probe through the available network path.
- Confirm `.env`, `config.yaml`, `data/`, logs, and runtime backend JSON remain untracked.
- 2026-05-29 verified: `cargo check --offline -p hone-core -p hone-tools -p hone-channels -p hone-web-api -p hone-cli --tests`.
- 2026-05-29 verified: `hone-cli cloud doctor --ensure-schema --json` reports PG connected through proxy, OSS connected through proxy, and schema ensured.
- 2026-05-29 verified: migration dry-run counts 117 sessions, 193 uploads/attachments, 204 company profiles, 25 portfolio JSON, 23 cron JSON, 22 notification prefs, 698 quota JSON, 50 SQLite files.
- 2026-05-29 verified: full live migrate apply now completes with `--concurrency 12` plus a follow-up `--reuse-existing --concurrency 4` retry. Result: 1282 non-SQLite durable files uploaded/reused in OSS and indexed in PG `cloud_documents`; 50 SQLite files intentionally skipped for structured row-wise PG import.
- 2026-05-29 verified: `hone-cli cloud object-bench --size-kib 1024 --iterations 3 --json` through proxy. Aliyun OSS average PUT / HEAD / GET was 5594ms / 470ms / 4811ms; Cloudflare R2 average PUT / HEAD / GET was 3358ms / 235ms / 4921ms. R2 wins writes and metadata checks on this machine; reads are effectively comparable at 1MiB.
- 2026-05-29 verified: `HONE_CLOUD_MODE=cloud cargo run --offline -p hone-cli -- cloud migrate --from-data-dir ./data --quota-only --apply --json` imported 698 legacy quota JSON rows into PG; first run changed 366 rows / skipped 332, second run changed 0 / skipped 698 with 0 conflicts.
- 2026-05-29 verified: `HONE_CLOUD_MODE=cloud hone-cli cloud doctor --ensure-schema --json` reports PG/R2 healthy and `local_durable_dependency_count=9`; quota is no longer counted as a durable local dependency when PG is configured.
- 2026-05-29 verified: `cargo test -p hone-core cloud_runtime --lib`, `cargo test -p hone-memory quota --lib`, and `cargo check --workspace --all-targets --exclude hone-desktop`.
- 2026-05-30 verified: `HONE_CLOUD_MODE=cloud cargo run --offline -p hone-cli -- cloud migrate --from-data-dir ./data --session-only --apply --json` imported 117 legacy session JSON rows into PG; second run changed 0 / skipped 117 with 0 conflicts.
- 2026-05-30 verified: `HONE_CLOUD_MODE=cloud hone-cli cloud doctor --ensure-schema --json` reports PG/R2 healthy and `local_durable_dependency_count=8`; `sessions_dir` is no longer counted as a durable local dependency when PG is configured.
- 2026-05-30 verified: `cargo test -p hone-memory session --lib`, `cargo test -p hone-core cloud_runtime --lib`, and `cargo check --workspace --all-targets --exclude hone-desktop`.
- 2026-05-30 verified: `HONE_CLOUD_MODE=cloud cargo run --offline -p hone-cli -- cloud migrate --from-data-dir ./data --web-auth-only --apply --json` imported 30 legacy web invite users and 3 auth sessions into PG; second run changed 0 users / 0 sessions and skipped 30 users / 3 sessions with 0 conflicts.
- 2026-05-30 verified: `HONE_CLOUD_MODE=cloud hone-cli cloud doctor --ensure-schema --json` reports PG/R2 healthy and `local_durable_dependency_count=7`; `sessions.sqlite3` is no longer counted as a durable local dependency when PG is configured.
- 2026-05-30 verified: `cargo test -p hone-memory web_auth --lib`, `cargo test -p hone-core cloud_runtime --lib`, and `cargo check --workspace --all-targets --exclude hone-desktop`.
- 2026-05-30 verified: `HONE_CLOUD_MODE=cloud cargo run --offline -p hone-cli -- cloud migrate --from-data-dir ./data --cron-only --apply --json` imported 54 cron jobs from 23 legacy cron JSON files into PG; second run changed 0 / skipped 54 with 0 conflicts.
- 2026-05-30 verified: `HONE_CLOUD_MODE=cloud cargo run --offline -p hone-cli -- cloud doctor --ensure-schema --json` reports PG/R2 healthy and `local_durable_dependency_count=6`; `cron_jobs_dir` is no longer counted as a durable local dependency when PG is configured.
- 2026-05-30 verified: `cargo test -p hone-memory cron_job --lib`, `cargo test -p hone-core cloud_runtime --lib`, and `cargo check --workspace --all-targets --exclude hone-desktop`.
- 2026-05-31 verified: `HONE_CLOUD_MODE=cloud cargo run --offline -p hone-cli -- cloud migrate --from-data-dir ./data --skill-registry-only --apply --json` found no local `runtime/skill_registry.json` on this machine and returned 0 changed / 0 skipped with 0 conflicts.
- 2026-05-31 verified: `HONE_CLOUD_MODE=cloud cargo run --offline -p hone-cli -- cloud doctor --ensure-schema --json` reports PG/R2 healthy and `local_durable_dependency_count=5`; `data/runtime/skill_registry.json` is no longer counted as a durable local dependency when PG is configured.
- 2026-05-31 verified: `cargo test --offline -p hone-tools skill_registry --lib`, `cargo test --offline -p hone-core cloud_runtime --lib`, `cargo check --offline -p hone-core -p hone-tools -p hone-channels -p hone-web-api -p hone-cli --tests`, and `HONE_CLOUD_MODE=local cargo run --offline -p hone-cli -- cloud doctor --json`.
- 2026-05-31 verified: `HONE_CLOUD_MODE=cloud cargo run --offline -p hone-cli -- cloud migrate --from-data-dir ./data --notification-prefs-only --apply --json` imported / verified 22 legacy notification prefs JSON rows in PG `cloud_notification_prefs`; final run changed 0 / skipped 22 with 0 conflicts.
- 2026-05-31 verified: `HONE_CLOUD_MODE=cloud cargo run --offline -p hone-cli -- cloud doctor --ensure-schema --json` reports PG/R2 healthy and `local_durable_dependency_count=4`; `data/notif_prefs` is no longer counted as a durable local dependency when PG is configured.
- 2026-05-31 verified: `cargo test --offline -p hone-event-engine prefs --lib`, `cargo test --offline -p hone-core cloud_runtime --lib`, `cargo check --offline -p hone-core -p hone-event-engine -p hone-tools -p hone-channels -p hone-web-api -p hone-cli --tests`, and `HONE_CLOUD_MODE=local cargo run --offline -p hone-cli -- cloud doctor --json`.
- 2026-05-31 verified: `HONE_CLOUD_MODE=cloud cargo run --offline -p hone-cli -- cloud migrate --from-data-dir ./data --portfolio-only --apply --json` imported / verified 25 legacy portfolio JSON rows in PG `cloud_portfolios`; final run changed 1 / skipped 24 with 0 conflicts.
- 2026-05-31 verified: `HONE_CLOUD_MODE=cloud cargo run --offline -p hone-cli -- cloud doctor --ensure-schema --json` reports PG/R2 healthy and `local_durable_dependency_count=3`; `data/portfolio` is no longer counted as a durable local dependency when PG is configured. Remaining local durable dependencies are `./data/agent-sandboxes`, `data/gen_images`, and `data/llm_audit.sqlite3`.
- 2026-05-31 verified: `cargo test --offline -p hone-memory portfolio --lib`, `cargo test --offline -p hone-core cloud_runtime --lib`, `cargo check --offline -p hone-core -p hone-memory -p hone-event-engine -p hone-tools -p hone-channels -p hone-web-api -p hone-cli --tests`, and `HONE_CLOUD_MODE=local cargo run --offline -p hone-cli -- cloud doctor --json`.
- 2026-05-31 verified: `HONE_CLOUD_MODE=cloud cargo run --offline -p hone-cli -- cloud migrate --from-data-dir ./data --llm-audit-only --apply --json` imported / verified 1028 legacy SQLite LLM audit rows in PG `cloud_llm_audit_records`; final run changed 0 / skipped 1028 with 0 conflicts.
- 2026-05-31 verified: `HONE_CLOUD_MODE=cloud cargo run --offline -p hone-cli -- cloud doctor --ensure-schema --json` reports PG/R2 healthy and `local_durable_dependency_count=2`; `data/llm_audit.sqlite3` is no longer counted as a durable local dependency when PG is configured. Remaining local durable dependencies are `./data/agent-sandboxes` and `data/gen_images`.
- 2026-05-31 verified: `cargo test --offline -p hone-memory llm_audit --lib`, `cargo test --offline -p hone-core cloud_runtime --lib`, `cargo check --offline -p hone-core -p hone-memory -p hone-channels -p hone-web-api -p hone-cli --tests`, and `HONE_CLOUD_MODE=local cargo run --offline -p hone-cli -- cloud doctor --json`.
- 2026-05-31 verified: cloud-mode generated image finalization now uploads `gen_images` files to OSS and returns `oss://...` markers when OSS is configured; local mode keeps `file://` markers and the existing local copy behavior.
- 2026-05-31 verified: `HONE_CLOUD_MODE=cloud cargo run --offline -p hone-cli -- cloud doctor --ensure-schema --json` reports PG/R2 healthy and `local_durable_dependency_count=1`; `data/gen_images` is no longer counted as a durable local dependency when OSS is configured. The remaining local durable dependency is `./data/agent-sandboxes`.
- 2026-05-31 verified: `cargo test --offline -p hone-channels normalize_local_image_references --lib`, `cargo test --offline -p hone-core cloud_runtime --lib`, `cargo check --offline -p hone-core -p hone-memory -p hone-channels -p hone-web-api -p hone-cli --tests`, and `HONE_CLOUD_MODE=local cargo run --offline -p hone-cli -- cloud doctor --json`.
- 2026-05-31 verified: `HONE_CLOUD_MODE=cloud cargo run --offline -p hone-cli -- cloud migrate --from-data-dir ./data --company-profiles-only --apply --json` imported 172 actor-scoped company profile Markdown files into PG `cloud_company_profile_files` with 0 conflicts.
- 2026-05-31 verified: `HONE_CLOUD_MODE=cloud cargo run --offline -p hone-cli -- cloud doctor --ensure-schema --json` reports PG/R2 healthy and `local_durable_dependency_count=0`; `./data/agent-sandboxes` is no longer counted as a durable local dependency when PG is configured.
- 2026-05-31 verified: `cargo test --offline -p hone-core cloud_runtime --lib`, `cargo test --offline -p hone-memory company_profile --lib`, `cargo test --offline -p hone-event-engine mainline_distill --lib`, `cargo test --offline -p hone-channels normalize_local_image_references --lib`, `cargo check --offline -p hone-core -p hone-memory -p hone-event-engine -p hone-channels -p hone-web-api -p hone-cli --tests`, and `HONE_CLOUD_MODE=local cargo run --offline -p hone-cli -- cloud doctor --json`.

## Documentation Sync

- Remove this task from `docs/current-plan.md` after completion.
- Update `docs/repo-map.md` for the new cloud config / OSS upload path.
- Update `docs/runbooks/backend-deployment.md` with the runtime env contract.
- Add or update the handoff when the turn closes because the migration state is operationally useful.

Current handoff: `docs/handoffs/cloud-pg-oss-runtime-migration-2026-05-27.md`

Current impact report: `docs/handoffs/cloud-runtime-impact-report-2026-05-28.md`

## Risks / Open Questions

- Direct local TCP to the Aliyun PG endpoint still depends on proxy availability; verified path uses `HONE_POSTGRES_PROXY`.
- Object runtime honors `HONE_OSS_PROVIDER=aliyun_oss|r2|s3` and `HONE_OSS_PROXY`; live R2 bucket health passed through proxy.
- `/api/meta` now exposes `cloud_mode`, `runtime_role`, `cloud_storage_authoritative`, local durable dependency count, and PG/OSS health.
- Company profile runtime storage is PG-backed in cloud mode, and agent native-file writes are synced to PG at successful response finalization. If a runner process crashes before finalization, the local sandbox copy may need a manual `--company-profiles-only` import pass.
- Historical SQLite files that are not sessions / web auth / LLM audit remain counted by the broad migrator but are not current runtime hot-path dependencies.
- `https://hone-claw.com/api/meta` previously timed out, so desktop remote-backend health remains separate from PG / OSS credential health.
