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
  - `docs/archive/plans/public-web-multi-session-auth.md`
  - `docs/archive/index.md`
- related_prs: N/A

## Summary

Public web auth now allows multiple active sessions per user. This fixes the health-check automation and a real browser repeatedly invalidating each other's `hone_web_session` cookie.

## What Changed

- `create_session_for_invite` no longer deletes existing sessions for the same user during normal invite login.
- `create_session_for_user` no longer deletes existing sessions during password login or other normal session creation.
- Password setup still rotates the current session token to avoid session fixation, but it deletes only the current cookie's old session instead of logging out other devices.
- Admin revocation and invite reset still clear all active sessions for the affected user.

## Verification

- `rustfmt --edition 2024 --check memory/src/web_auth.rs crates/hone-web-api/src/routes/public.rs`
- `cargo test -p hone-memory web_auth -- --nocapture`
- `cargo check -p hone-web-api -p hone-memory`
- `cargo test -p hone-web-api public -- --nocapture`

`cargo fmt --all --check` was not used as completion evidence because unrelated pre-existing Rust formatting differences exist outside this change. `bash scripts/ci/check_fmt_changed.sh` could not run on the local default Bash because `mapfile` is unavailable.

## Risks / Follow-ups

Future remote logout/device management should build on the now-multiple `web_auth_sessions` rows instead of assuming one session per user.

## Next Entry Point

`memory/src/web_auth.rs`
