# Public Login ToS Runtime Mismatch

- title: Public Login ToS Runtime Mismatch
- status: done
- created_at: 2026-05-20
- updated_at: 2026-05-20
- owner: Codex
- related_files:
  - `packages/app/src/lib/tos.ts`
  - `crates/hone-web-api/src/routes/public.rs`
  - `packages/app/e2e/public-sms-login.spec.ts`
  - `packages/app/dist-public/`
  - `target/release/hone-console-page`
- related_docs:
  - `docs/runbooks/desktop-release-app-runtime.md`
- related_prs: N/A

## Summary

Public SMS login was blocked by a ToS version mismatch between the running frontend bundle and the running backend binary. Source already had `TOS_VERSION = "2.1"` on both sides, but the live `hone-console-page` binary was still expecting `2.0` while the rebuilt public bundle served `2.1`.

## What Changed

- Rebuilt `packages/app/dist-public` with `bun run build:web:public`.
- Rebuilt `target/release/hone-console-page` with `cargo build --release -p hone-console-page`.
- Restarted only `hone-console-page-prod`, leaving Feishu, Discord, workflow, public Vite, and desktop screens running.
- Updated `packages/app/e2e/public-sms-login.spec.ts` from ToS `2.0` to `2.1`.

## Verification

- `curl http://127.0.0.1:8088/chat` serves `assets/index-F7vNO0UA.js`.
- `curl http://127.0.0.1:8088/assets/tos-BE1K_gk6.js` returns `const E="2.1",T="2026-05-18"`.
- `POST /api/public/auth/sms/login` with `tos_version: "2.0"` now returns `协议版本已更新，请刷新页面后重新确认`.
- The same request with `tos_version: "2.1"` no longer fails the ToS gate and proceeds to the whitelist/SMS path.
- `PATH="$HOME/.bun/bin:$PATH" bun --filter @hone-financial/app test -- public-sms-login` passed the app unit suite.
- Targeted Playwright E2E could not run because local Chromium was not installed under `~/Library/Caches/ms-playwright`.

## Risks / Follow-ups

- Rebuilding only frontend or only backend can reintroduce this mismatch. When bumping `TOS_VERSION`, rebuild both `packages/app/dist-public` and `target/release/hone-console-page`, then restart the backend serving port 8088.
- Existing browser tabs may still have the old JS bundle in memory; refreshing loads the new hashed bundle.

## Next Entry Point

Start with `packages/app/src/lib/tos.ts`, `crates/hone-web-api/src/routes/public.rs`, and the live `/assets/tos-*.js` served by port 8088.
