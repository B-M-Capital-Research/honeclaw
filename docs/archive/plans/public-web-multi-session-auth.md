# Public Web Multi-Session Auth

- title: Public Web Multi-Session Auth
- status: done
- created_at: 2026-05-05
- updated_at: 2026-05-05
- owner: Codex
- related_files:
  - `memory/src/web_auth.rs`
  - `crates/hone-web-api/src/routes/public.rs`
- related_docs:
  - `docs/archive/index.md`
  - `docs/handoffs/2026-05-05-public-web-multi-session-auth.md`

## Goal

Allow one public web user to keep multiple active sessions so health-check automation, the user's browser, and other devices do not invalidate each other on normal login.

## Scope

- Change normal invite/password login to create a new session without deleting other active sessions for the same user.
- Preserve admin revocation/reset behavior that intentionally clears all sessions.
- Preserve current-session rotation for password setup without logging out other devices.

## Validation

- `rustfmt --edition 2024 --check memory/src/web_auth.rs crates/hone-web-api/src/routes/public.rs`
- `cargo test -p hone-memory web_auth -- --nocapture`
- `cargo check -p hone-web-api -p hone-memory`
- `cargo test -p hone-web-api public -- --nocapture`

## Documentation Sync

- Added this active plan while the behavior change was being implemented.
- Removed it from `docs/current-plan.md`, archived this plan, and added an archive index entry after verification.

## Risks / Open Questions

- Existing behavior intentionally limited users to one active session. Multi-session auth is a better fit for web + automation, but future device/session management may need a UI if operators want explicit remote logout.
