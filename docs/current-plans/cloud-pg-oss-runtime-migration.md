# Cloud PG / OSS Runtime Migration

- title: Cloud PG / OSS Runtime Migration
- status: in_progress
- created_at: 2026-05-27
- updated_at: 2026-05-29
- owner: Codex
- related_files:
  - `config.example.yaml`
  - `crates/hone-core/src/config/server.rs`
  - `crates/hone-core/src/config/mod.rs`
  - `crates/hone-core/src/cloud_runtime.rs`
  - `bins/hone-cli/src/cloud.rs`
  - `crates/hone-tools/src/local_files.rs`
  - `crates/hone-channels/src/attachments/ingest.rs`
  - `crates/hone-channels/src/response_finalizer.rs`
  - `crates/hone-web-api/src/routes/public.rs`
  - `crates/hone-web-api/src/routes/files.rs`
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
- Add S3-compatible object-store support for Cloudflare R2. The local ignored `.env` currently points runtime `HONE_OSS_*` at R2, preserves Aliyun OSS under `HONE_ALIYUN_OSS_*` for rollback / benchmark comparison, and enables dotenv override so stale shell env cannot silently force the old provider.
- Switch local file tools to use actor-scoped OSS namespace when cloud mode is authoritative; keep local sandbox walk/read/search in local mode.
- Upload channel attachments and generated images to OSS in cloud mode where the current call site has enough context; local mode keeps current filesystem behavior.

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

## Documentation Sync

- Update `docs/current-plan.md` while active.
- Update `docs/repo-map.md` for the new cloud config / OSS upload path.
- Update `docs/runbooks/backend-deployment.md` with the runtime env contract.
- Add or update the handoff when the turn closes because the migration state is operationally useful.

Current handoff: `docs/handoffs/cloud-pg-oss-runtime-migration-2026-05-27.md`

Current impact report: `docs/handoffs/cloud-runtime-impact-report-2026-05-28.md`

## Risks / Open Questions

- Direct local TCP to the Aliyun PG endpoint still depends on proxy availability; verified path uses `HONE_POSTGRES_PROXY`.
- Object runtime honors `HONE_OSS_PROVIDER=aliyun_oss|r2|s3` and `HONE_OSS_PROXY`; live R2 bucket health passed through proxy.
- `/api/meta` now exposes `cloud_mode`, `runtime_role`, `cloud_storage_authoritative`, local durable dependency count, and PG/OSS health.
- Existing session, quota, audit, portfolio, cron, notification prefs, KB, and company profile runtime stores are not fully PG-backed yet. Durable files from the local `data/` snapshot have been uploaded/indexed, and selected runtime file surfaces now write OSS in cloud mode, but the hot-path repositories remain local until follow-up adapters land.
- SQLite structured import is pending; the migrator intentionally skips SQLite blob upload because LLM audit is large (about 1.5GB locally) and should be imported row-wise into PG tables.
- `https://hone-claw.com/api/meta` previously timed out, so desktop remote-backend health remains separate from PG / OSS credential health.
