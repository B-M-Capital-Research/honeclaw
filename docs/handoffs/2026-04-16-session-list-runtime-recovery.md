# Session List Runtime Recovery

- title: Session list runtime recovery
- status: done
- created_at: 2026-04-16
- updated_at: 2026-04-16
- owner: Codex
- related_files:
  - crates/hone-core/src/actor.rs
  - crates/hone-web-api/src/routes/users.rs
  - memory/src/session_sqlite.rs
  - docs/runbooks/desktop-release-app-runtime.md
  - .codex/automations/bug-2/automation.toml
- related_docs:
  - docs/archive/index.md
  - docs/runbooks/desktop-release-app-runtime.md
- related_prs:
  - N/A

## Summary

Desktop release runtime and local backend are both back to a healthy state against the repo-local `honeclaw/data` tree. The main user-visible bug was not missing data; it was a backend session-listing failure that caused `/api/users` to return an empty array even though `data/sessions.sqlite3` and `data/sessions/` still contained historical sessions.

## What Changed

- Added `SessionIdentity::from_session_id(...)` and `ActorIdentity::from_session_id(...)` so actor or shared-group session ids can be parsed back into displayable identities without depending on already-normalized metadata
- Updated the users route to fall back to session-id-derived identity when the stored actor metadata is absent or incomplete, so `/api/users` can still list historical sessions
- Updated SQLite session listing to skip unreadable `normalized_json` rows with a warning instead of failing the entire listing
- Expanded the desktop release runtime runbook with startup pitfalls, including stale lock handling, detached backend silent exits, the difference between desktop-vs-backend failures, and the need to treat `/api/users == []` as a likely backend bug when on-disk session data clearly exists
- Expanded the `bug-2` automation prompt so startup/runtime bugs explicitly require consulting the desktop runtime runbook and validating `/api/meta`, `/api/channels`, `/api/users`, and `/api/history`

## Verification

- `cargo test -p hone-core actor::tests::session_identity_can_be_restored_from_actor_session_id -- --exact`
- `cargo test -p hone-core actor::tests::session_identity_can_be_restored_from_shared_group_session_id -- --exact`
- `cargo test -p hone-core actor::tests::actor_identity_can_be_restored_from_actor_session_id -- --exact`
- `cargo test -p hone-memory session_sqlite::tests::list_sessions_skips_unreadable_rows -- --exact`
- `cargo test -p hone-web-api routes::users::tests::actor_session_id_is_enough_for_listing_identity -- --exact`
- `cargo test -p hone-web-api routes::users::tests::shared_group_session_id_is_enough_for_listing_identity -- --exact`
- `cargo fmt --all`
- `rustfmt --edition 2024 --check crates/hone-core/src/actor.rs crates/hone-web-api/src/routes/users.rs memory/src/session_sqlite.rs`
- `curl http://127.0.0.1:8077/api/meta`
- `curl http://127.0.0.1:8077/api/users`
- `curl 'http://127.0.0.1:8077/api/history?session_id=Actor_feishu__direct__ou_5f995a704ab20334787947a366d62192f7'`
- `curl http://127.0.0.1:8077/api/channels`

## Risks / Follow-ups

- The detached backend launch path can still fail silently on this machine; the runbook now treats foreground diagnostic startup as the first recovery path
- `scripts/ci/check_fmt_changed.sh` is not portable to the default macOS `bash` because it uses `mapfile`; when running locally on macOS, prefer the equivalent explicit `rustfmt --check` fallback unless the script is made portable
- The live runtime currently depends on the cache target directory `/Users/ecohnoch/Library/Caches/honeclaw/target`; rebuilding the wrong target tree will not refresh the running desktop/backend binaries

## Next Entry Point

- `docs/runbooks/desktop-release-app-runtime.md`
