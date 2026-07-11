# Agent And Data Security Hardening

- title: Agent And Data Security Hardening
- status: in_progress
- created_at: 2026-07-12
- updated_at: 2026-07-12
- owner: Codex
- related_files:
  - `crates/hone-channels/src/execution.rs`
  - `crates/hone-channels/src/core/bot_core.rs`
  - `crates/hone-channels/src/sandbox.rs`
  - `crates/hone-web-api/src/routes/mod.rs`
  - `crates/hone-core/src/config/agent.rs`
- related_docs:
  - `docs/invariants.md`
  - `docs/decisions.md`
  - `docs/repo-map.md`

## Goal

Prevent untrusted users and prompt-injected agents from reading host/repository secrets, crossing actor boundaries, invoking privileged system capabilities, or using public APIs to access another user's database/object rows.

## Scope

- Route non-admin actors away from native CLI/ACP runners with broad host read/process capabilities.
- Keep Codex ACP available only to explicitly configured administrators/local trusted actors.
- Enforce owner-only permissions on configuration, runtime data, and actor sandbox roots.
- Restrict credentialed public CORS to an explicit HONE/localhost origin allowlist.
- Patch production dependency advisories and remove Discord's unpatched legacy rustls chain.
- Audit actor binding in public routes, sessions, uploads, push rows, portfolios, cron, profiles, and cloud records.
- Record residual risks and required secret rotation.

## Validation

- Add unit tests for runner trust classification, secure fallback, sandbox modes, permissions, CORS origins, path traversal, and actor isolation.
- Verify the lockfile no longer contains the high-severity `rustls-webpki 0.102` branch.
- Run focused Rust tests, workspace check/test where feasible, frontend tests, and CI-safe regressions.
- Restart the runtime and verify public/admin health plus effective runner routing logs.

## Documentation Sync

- Replace the accepted ACP out-of-bounds-read exception in `docs/invariants.md`.
- Record the trusted-runner boundary in `docs/decisions.md` and update `docs/repo-map.md`.
- Write a security handoff with findings, fixes, residual risks, and credential-rotation requirements.
- Archive this plan and update `docs/archive/index.md` after deployment.

## Risks / Open Questions

- Function-calling fallback requires a configured in-process LLM provider; fail closed if unavailable.
- Administrators retain privileged ACP capability and therefore remain part of the trusted computing base.
- Existing credentials found in local config must be rotated outside code; permission hardening does not invalidate already exposed values.
