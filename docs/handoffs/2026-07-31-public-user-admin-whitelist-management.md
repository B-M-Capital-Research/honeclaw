# Public User Admin Whitelist Management Handoff

- title: Public user administrator and whitelist management
- status: in_progress
- created_at: 2026-07-31
- updated_at: 2026-07-31
- owner: Codex
- related_files:
  - `crates/hone-core/src/cloud_runtime.rs`
  - `memory/src/web_auth.rs`
  - `crates/hone-web-api/src/routes/public_admin.rs`
  - `crates/hone-web-api/src/routes/public.rs`
  - `crates/hone-web-api/src/routes/mod.rs`
  - `crates/hone-web-api/src/types.rs`
  - `bins/hone-cli/src/cloud.rs`
  - `packages/app/src/components/public-admin-whitelist-panel.tsx`
  - `packages/app/src/pages/public-me.tsx`
  - `packages/app/src/lib/api.ts`
  - `packages/app/src/lib/types.ts`
- related_docs:
  - `docs/current-plans/public-user-admin-whitelist-management.md`
  - `docs/runbooks/public-user-admin.md`
  - `docs/repo-map.md`
  - `docs/invariants.md`
  - `docs/decisions.md`
- related_prs: none

## Summary

The shared public mobile/PC client now has a PostgreSQL-authoritative administrator role and an administrator-only “我的 → 管理” surface for domestic membership whitelist management. The production PostgreSQL row for `13871396421` is uniquely matched, active, and verified as an administrator. Source changes are complete locally but have not been committed, pushed, built into Apple clients, or deployed.

## What Changed

- Added additive `is_admin` role storage and `cloud_web_admin_actions` audit records in PostgreSQL, with SQLite parity for local development and tests.
- Added cookie-session-authenticated public administrator list/create/disable APIs. Every request rechecks the backend role; mutation requests also require the application action marker.
- Enforced at most five successful creations per administrator per Beijing natural day inside the database transaction. Duplicate and failed attempts do not consume the limit.
- Prevented self-disable and administrator-disable, and cleared all target Web sessions when an ordinary member is disabled.
- Added a responsive management panel shared by browser, macOS WebView, and iOS WebView. Ordinary users do not receive the panel.
- Added a dry-run-first `hone-cli cloud web-admin` operational command with masked output and write-after-read verification.

## Verification

- `cargo check -p hone-cli -p hone-web-api -p hone-memory --tests`
- `cargo test -p hone-memory`: 132 passed.
- `cargo test -p hone-core cloud_runtime::tests`: 20 passed.
- `cargo test -p hone-cli`: 85 passed.
- Focused `cargo test -p hone-web-api public_admin`: 3 passed.
- Full `cargo test -p hone-web-api`: 158 passed, 2 ignored, with one pre-existing unrelated failure in `public_chat_user_input_uses_shared_attachment_context`.
- `bun run typecheck`: passed.
- `bun test --preload ./happydom.ts ./src`: 312 passed.
- `VITE_HONE_APP_SURFACE=public bun run build`: passed.
- Browser QA at 390×844 and 1280×900 confirmed the management panel has no horizontal overflow and the mobile table becomes readable cards.
- `hone-cli cloud web-admin --phone 13871396421 --action grant --json` confirmed one active target and `verified_is_admin=true` without modifying the already-correct row.

## Risks / Follow-ups

- The database role is already active, but production will not show the module until this exact source is committed, built, and deployed.
- Do not test six real additions merely to exercise the limit. Use automated tests for exhaustion and only a controlled account for live verification.
- Public administrator responses deliberately exclude invite codes, API keys, password state, and internal quota details.
- The plan remains active until production deployment and live authorization checks are complete; do not archive it yet.

## Next Entry Point

1. Review the scoped diff, then commit and push only these feature and documentation changes.
2. Build/deploy the public Web backend/frontend; build Apple clients only if a new client release is requested.
3. Sign in as `13871396421` and confirm “我的 → 管理” appears.
4. Confirm an ordinary user receives `403` from `/api/public/admin/invites` and does not see the panel.
5. Perform one controlled create/disable cycle, then verify backend health and target-session revocation.
