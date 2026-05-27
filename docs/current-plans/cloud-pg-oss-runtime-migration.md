# Cloud PG / OSS Runtime Migration

- title: Cloud PG / OSS Runtime Migration
- status: in_progress
- created_at: 2026-05-27
- updated_at: 2026-05-27
- owner: Codex
- related_files:
  - `config.example.yaml`
  - `crates/hone-core/src/config/server.rs`
  - `crates/hone-core/src/config/mod.rs`
  - `crates/hone-web-api/src/routes/public.rs`
  - `crates/hone-web-api/src/routes/files.rs`
  - `crates/hone-web-api/src/cloud_oss.rs`
  - `.env` (local only, ignored)
- related_docs:
  - `docs/current-plan.md`
  - `docs/repo-map.md`
  - `docs/runbooks/backend-deployment.md`
  - `docs/handoffs/cloud-pg-oss-runtime-migration-2026-05-27.md`

## Goal

Move the local runtime toward remote ownership for database and object storage: load PG / OSS settings from local runtime env, route public uploads and file proxy reads through OSS when configured, and make remaining local dependencies explicit so follow-up PG-backed storage work is not hidden.

## Scope

- Add first-class cloud runtime config for Postgres and Aliyun OSS without committing secrets.
- Materialize the local `.env` with PG / OSS runtime variables.
- Switch public web upload storage from local filesystem to OSS when OSS config is present.
- Keep existing local JSON / SQLite / directory storage as fallback until PG-backed stores land.
- Document the remaining local dependencies and the exact verification gap.

## Validation

- `cargo check --workspace --all-targets --exclude hone-desktop`
- Targeted config / web-api tests where practical.
- Manual PG / OSS health probe through the available network path.
- Confirm `.env`, `config.yaml`, `data/`, logs, and runtime backend JSON remain untracked.

## Documentation Sync

- Update `docs/current-plan.md` while active.
- Update `docs/repo-map.md` for the new cloud config / OSS upload path.
- Update `docs/runbooks/backend-deployment.md` with the runtime env contract.
- Add or update the handoff when the turn closes because the migration state is operationally useful.

Current handoff: `docs/handoffs/cloud-pg-oss-runtime-migration-2026-05-27.md`

## Risks / Open Questions

- Direct local TCP to the Aliyun PG endpoint timed out; current verified PG health uses the local SOCKS proxy path.
- Existing session, quota, audit, portfolio, cron, notification prefs, KB, and log stores are still local JSON / SQLite / directories until PG-backed repository implementations are added.
- `https://hone-claw.com/api/meta` previously timed out, so desktop remote-backend health remains separate from PG / OSS credential health.
