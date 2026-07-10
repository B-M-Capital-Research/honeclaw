# Web Scheduled Push Inbox

- title: Web Scheduled Push Inbox
- status: done
- created_at: 2026-07-10
- updated_at: 2026-07-10
- owner: Codex
- related_files: `crates/hone-web-api/src/routes/public_pushes.rs`, `memory/src/cron_job/history.rs`, `packages/app/src/components/public-push-center.tsx`, `packages/app/src/pages/chat.tsx`
- related_docs: `docs/archive/plans/web-scheduled-push-inbox.md`, `docs/decisions.md`, `docs/repo-map.md`
- related_prs: N/A

## Summary

Web scheduled-task results now arrive as concise bilingual cards, open full Markdown on demand, and collect in a unified push center. Read state is durable and actor-scoped but not shown per item; reading push N marks N and all older pushes read, so reading the newest push clears the red dot. Non-Web channels are unchanged.

## What Changed

- Scheduler Web turns carry push metadata while the canonical transcript remains available to the agent and admin diagnostics.
- SQLite and PostgreSQL stores keep summary/full content separately in `web_push_messages` / `cloud_web_push_messages`.
- Public API list payloads omit full content; the authenticated open endpoint returns it and commits mark-through read state.
- Public history suppresses scheduler trigger prompts and projects current/legacy scheduled responses into cards.
- Desktop rail and mobile header expose the push center and aggregate unread dot.

## Verification

- `cargo test -p hone-memory web_push_read_through_keeps_newer_pushes_unread -- --nocapture`
- `cargo test -p hone-web-api --lib -- --nocapture` (93 passed, 2 ignored)
- `cargo check -p hone-channels -p hone-web-api`
- `bun run typecheck:web`, `bun run test:web` (203 passed), `bun run build:web:public`
- `cargo check --workspace --all-targets --exclude hone-desktop`, `bash tests/regression/run_ci.sh`, and `target/debug/hone-cli doctor`
- PostgreSQL + authenticated HTTP smoke: list returned 3 unread, opening the middle returned 1 unread, opening the latest returned 0; all temporary rows were cleaned.
- Final in-app browser reload reached the public SMS login page without restore retries.

## Risks / Follow-ups

- Legacy cards use transcript fallback content and deliberately have no fabricated read state.
- The isolated in-app browser had no SMS session, so it verified public login/proxy health but not the authenticated drawer visually.
- Local user UI must proxy to public backend `8088`, not admin backend `8077`.

## Next Entry Point

Start with `crates/hone-web-api/src/routes/public_pushes.rs` for API/read behavior and `packages/app/src/components/public-push-center.tsx` for UI changes.

## Mobile Production Hotfix

- The production backend was restarted before the matching Cloudflare Pages bundle was deployed. The old bundle treated new `scheduled_push` history rows as empty assistant messages, producing copy/share-only shells on mobile and no push-center UI.
- `PublicNav` previously placed every extra action inside `.pub-nav-links`, which is hidden below 768px. The push bell now has a dedicated mobile action next to the hamburger while the desktop rail/header entry remains unchanged.
- The first actor-scoped push-list request now detects whether legacy rows were imported, extracts pre-upgrade scheduler user/assistant pairs, assigns deterministic `legacy:*` ids, and bulk-upserts them with one PostgreSQL statement or one SQLite transaction. Conflict updates intentionally leave `read_at` unchanged.
- A temporary secondary session for the affected actor triggered the migration without invalidating existing sessions. It imported 79 historical pushes in 93ms; the temporary session was deleted and no push was opened during verification, so user read state was not changed.
- Verification passed: Web API 94/94 non-live tests, Web 203 tests, typecheck, public build, workspace check, and CI-safe regression. The unrelated existing memory test `sqlite_runtime_backend_backfills_existing_json_even_when_shadow_write_disabled` still fails with `QueryReturnedNoRows`; the new legacy batch/id tests pass.
- Commit `383058fe` was pushed to `main`. Cloudflare Pages switched from the stale `index-CTmcEZn7.js` bundle to `index-BeqwKSm5.js`; production chat assets expose the push center and dedicated mobile action, and authenticated Worker verification returned five recent legacy cards with 79 unread total.

## Mobile Overlay And Calendar Follow-up

- The push center now uses a compact full-viewport mobile layout above the fixed nav; push details use a bounded bottom sheet, scheduled cards are denser, and the unread dot no longer covers the bell glyph.
- Opening the push center immediately hides the dot and acknowledges the latest push present at open time through the existing mark-through endpoint. A push arriving afterward remains unread.
- Finance-calendar preview is now explicitly tappable and opens a full-screen viewer with fit, zoom in/out, and scroll behavior. The viewer uses a document-body Portal so nav stacking contexts cannot intercept controls.
- Added `packages/app/e2e/public-mobile-overlays.spec.ts`, covering a `390x844` viewport, unread clearing, full-width center, compact cards, bottom-sheet detail, calendar large preview, and a real zoom interaction.
- Verification passed: Web typecheck, complete Web unit suite, public production build, and the focused mobile Playwright E2E. The isolated QA account and two test pushes were deleted; the affected actor remained at 79 total / 79 unread pushes.
