# Agent And Data Security Hardening

- title: Agent And Data Security Hardening
- status: archived
- created_at: 2026-07-12
- updated_at: 2026-07-12
- owner: Codex
- related_files:
  - `crates/hone-channels/src/execution.rs`
  - `crates/hone-channels/src/core/bot_core.rs`
  - `crates/hone-channels/src/sandbox.rs`
  - `crates/hone-web-api/src/routes/mod.rs`
  - `crates/hone-core/src/security.rs`
- related_docs:
  - `docs/handoffs/2026-07-12-agent-data-security-hardening.md`
  - `docs/invariants.md`
  - `docs/decisions.md`

## Goal

Prevent untrusted users and prompt-injected agents from reading host/repository secrets, crossing actor boundaries, invoking privileged system capabilities, or using public APIs to access another user's database/object rows.

## Completed Scope

- Routed non-admin actors away from native host-capable runners and failed closed without a safe provider.
- Enforced owner-only runtime/config/sandbox permissions and clean skill child environments.
- Restricted public credentialed CORS and audited actor binding across public/storage paths.
- Patched production dependency advisories and removed Discord's legacy rustls chain.
- Added runner, permissions, CORS, path, skill environment, session migration, and cross-actor regressions.
- Deployed and verified the complete runtime.

## Verification

See `docs/handoffs/2026-07-12-agent-data-security-hardening.md`.

## Risks

Admin ACP remains trusted; credentials require rotation; PostgreSQL RLS and two Tauri-only dependency advisories remain follow-ups.
