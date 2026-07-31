# Public User Admin Whitelist Management Handoff

- title: Public user administrator and whitelist management
- status: done
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
  - `docs/archive/plans/public-user-admin-whitelist-management.md`
  - `docs/runbooks/public-user-admin.md`
  - `docs/repo-map.md`
  - `docs/invariants.md`
  - `docs/decisions.md`
- related_prs: implementation/deployment commit `5eacfe98c0b2b3bdaac11fc23830c0ab91b14f3d` on `main`

## Summary

The shared public mobile/PC client now has a PostgreSQL-authoritative administrator role and an administrator-only “我的 → 管理” surface for domestic membership whitelist management. The production PostgreSQL row for `13871396421` is uniquely matched, active, and verified as an administrator. Commit `5eacfe98` is on `main`, Cloudflare Pages serves the matching client, and production Web/Feishu run the exact immutable build.

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
- Full `cargo test -p hone-web-api`: 159 passed, 2 credentialed tests ignored.
- `bun run typecheck:web`, `bun run test:web`, and `bun run build:web:public`: passed.
- Browser QA at 390×844 and 1280×900 confirmed the management panel has no horizontal overflow and the mobile table becomes readable cards.
- `hone-cli cloud web-admin --phone 13871396421 --action grant --json` confirmed one active target and `verified_is_admin=true` without modifying the already-correct row.
- `bash tests/regression/run_ci.sh`: passed; finance automation contracts are 44/44.
- Cloudflare Pages production entry is `assets/index-D7gVoQjr.js`; its API chunk contains `/api/public/admin/invites`, `X-Hone-Admin-Action`, and `whitelist`, while the `/me` chunk contains `is_admin`.
- Immutable runtime `target/deploy-5eacfe98` verifies 505 payload files; manifest SHA-256 is `28c2d27fe58d78a0aa6e83542f640ed15858ed199ca3aa1e7e68ac3235d97f13`.
- Production Web and Feishu processes load binaries from `target/deploy-5eacfe98`; the origin tunnel remains in its separate supervised lane. Final `/api/meta` reports version `0.15.3`, cloud mode, PostgreSQL/R2 healthy, `cloud_storage_authoritative=true`, zero local durable dependencies, and zero active chats.
- Local, origin, and public anonymous auth probes return expected `401 application/json`; the production administrator route is mounted and also fails closed with anonymous `401`.

## Risks / Follow-ups

- Do not test six real additions merely to exercise the limit. Use automated tests for exhaustion and only a controlled account for live verification.
- Public administrator responses deliberately exclude invite codes, API keys, password state, and internal quota details.
- The existing Chrome profile was logged out. No SMS was sent and no synthetic production session or real whitelist mutation was created solely for acceptance; the administrator can complete the final visual check on the next normal login.
- Retain `target/deploy-482c34d5` as the immediate Web rollback package. The current tunnel uses a separate supervisor and must not be folded into backend restart commands.

## Next Entry Point

On the next normal login as `13871396421`, confirm “我的 → 管理” appears. If a live mutation canary is desired, use one explicitly controlled phone, perform one create/disable cycle, and verify its sessions are revoked; do not consume the daily allowance with synthetic entries.
