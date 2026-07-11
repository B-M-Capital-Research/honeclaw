# Agent And Data Security Hardening

- title: Agent And Data Security Hardening
- status: done
- created_at: 2026-07-12
- updated_at: 2026-07-12
- owner: Codex
- related_files:
  - `crates/hone-channels/src/execution.rs`
  - `crates/hone-core/src/security.rs`
  - `crates/hone-tools/src/skill_tool.rs`
  - `crates/hone-web-api/src/routes/mod.rs`
  - `memory/src/cron_job/history.rs`
- related_docs:
  - `docs/invariants.md`
  - `docs/decisions.md#d-2026-07-12-01-reserve-native-agent-runners-for-trusted-administrators`
- related_prs:
  - commits `dbabbe77`, `a99bf096`

## Summary

The audit found a high-risk host boundary: production native CLI/ACP runners could read beyond an actor workspace, while ACP/MCP child configuration carried database/object-store runtime credentials. Local config, database, and state files were also group/world-readable. Public database/API routes were otherwise consistently actor-bound.

Non-admin actors now use the in-process function-calling runner with actor-bound tools and fail closed when it is unavailable. Native ACP/CLI remains only for explicit administrators. Runtime storage and sandbox permissions are owner-only, skill subprocesses no longer inherit server secrets, credentialed CORS is allowlisted, and the vulnerable production TLS/archive/crypto lockfile entries were patched.

## What Changed

- Added non-admin native-runner fallback plus admin-retention and fail-closed tests.
- Enforced Unix `0700` directories and `0600` config/SQLite files at startup and on the active host.
- Cleared inherited skill-script environments while preserving explicit artifact/runtime variables.
- Restricted public credentialed CORS to HONE, localhost, and exact operator-configured origins.
- Confirmed public sessions, uploads, pushes, portfolios, preferences, profiles, cron records, and cloud rows derive actor identity from authentication and/or query by `actor_storage_key`.
- Added explicit cross-actor push detail/read rejection assertions.
- Restored JSON-to-SQLite startup backfill when SQLite is the primary session backend.
- Patched `cmov`, `tar`, `openssl`, current `rustls-webpki`, and `rand`; moved Discord to Serenity native TLS to remove the unpatched `rustls-webpki 0.102` chain.

## Verification

- `hone-channels`: 495 passed.
- `hone-core`: 115 passed.
- `hone-memory`: 121 passed.
- `hone-tools`: 123 passed, 1 optional matplotlib smoke ignored.
- `hone-web-api`: 95 passed, 2 credentialed live tests ignored.
- `hone-discord`: 12 passed.
- `cargo check --workspace --all-targets --exclude hone-desktop --exclude hone-user-app`: passed with one existing dead-code warning.
- `bun run test:web`: 211 passed.
- `bash tests/regression/run_ci.sh`: passed.
- Runtime: admin/public ports healthy; production and origin auth probes returned expected JSON; PostgreSQL and S3 doctor checks passed; HONE CORS allowed and hostile Origin denied; Discord logged in and Feishu started.
- Dependabot open alerts reduced from 10 (`1 high / 5 medium / 4 low`) to 2 (`1 medium / 1 low`).

## Risks / Follow-ups

- Administrators remain trusted: native ACP/CLI can access host process/filesystem state and MCP runtime credentials. Do not grant admin status to ordinary Web/channel users.
- Rotate current database, OSS, model, channel, and Codex/OpenCode credentials because the pre-fix architecture made host reads possible; the audit found exposure potential, not evidence of exploitation.
- PostgreSQL isolation is application-query scoped rather than database RLS. Actor-key tests cover current paths, but a future missing predicate could still be dangerous; consider per-tenant RLS/session variables before multi-tenant scale increases.
- Remaining Dependabot alerts are Tauri-only upstream chains: Linux GTK `glib 0.18` (medium) and a Tauri HTML build dependency on `rand 0.7` (low). They are absent from the current backend runtime; resolve through a future Tauri/Wry dependency upgrade.
- Some event-engine enrichment profiles still report missing OpenRouter configuration and operate in documented degraded mode; the main `llm.profiles.main` provider used by safe user execution is configured.

## Next Entry Point

Start with `crates/hone-channels/src/execution.rs` for runner policy, `crates/hone-core/src/security.rs` for local permissions, and this handoff's follow-up list for credential rotation and eventual PostgreSQL RLS.
