# Public Finance Calendar

- title: Public Finance Calendar
- status: done
- created_at: 2026-06-29
- updated_at: 2026-06-29
- owner: Codex
- related_files:
  - `crates/hone-web-api/src/routes/public_finance_calendar.rs`
  - `crates/hone-web-api/src/routes/mod.rs`
  - `crates/hone-web-api/src/routes/public.rs`
  - `packages/app/src/pages/chat.tsx`
  - `packages/app/src/components/finance-calendar-card.tsx`
  - `packages/app/src/lib/finance-calendar.ts`
  - `packages/app/src/lib/api.ts`
  - `packages/app/src/lib/messages.ts`
  - `packages/app/src/lib/types.ts`
- related_docs:
  - `docs/archive/plans/public-finance-calendar.md`
  - `docs/repo-map.md`
- related_prs: N/A

## Summary

Public chat now has a second quick action, “我的财经日历”, after the existing portfolio push-mode tip. Users pick one of 12 months, the browser renders a month-view finance calendar PNG, uploads it through the existing public upload API, and asks the backend to append an assistant image message to the user's web chat session.

## What Changed

- Added `GET /api/public/finance-calendar?month=YYYY-MM` for actor-scoped calendar payloads. It returns today's date, 12 months for the selected year, portfolio/watchlist tickers, macro seed events, FMP earnings events, `earnings_status`, and non-fatal data-source errors.
- Added `POST /api/public/finance-calendar/send` for current-user uploaded PNGs. It reuses public upload-root validation, creates the session if missing, writes an assistant message containing `file://...png` or `oss://...png`, and broadcasts `push_message`.
- Added the July 2026 macro seed: 7/3 非农就业报告, 7/10 CPO发布, 7/24 第二季度GDP初值, 7/30 美联储利率决议+主席讲话.
- Added frontend helpers and tests for `YYYY-MM` parsing, default month selection, 12-month picker options, Monday-first grid cells, event grouping, and status labels.
- Added `FinanceCalendarCard`, rendered offscreen and captured with `html2canvas` + existing `canvasToPngBlob`.
- Extended chat message image parsing to render both `file://` and `oss://` image markers through the public image proxy.

## Verification

- `cargo test -p hone-web-api finance_calendar`: not run; current environment has no `cargo`.
- `bun run test:web`: not run; current environment has no `bun`.
- `cargo test --workspace --all-targets --exclude hone-desktop`: not run; current environment has no `cargo`.
- `bash scripts/ci/check_fmt_changed.sh`: completed, but skipped rustfmt because the workspace has no discoverable git base ref.
- Manual code-path review covered public auth scoping, public upload path validation, OSS image proxy support, SSE `push_message` filtering, and FMP stable endpoint base normalization for both `/api` and `/api/v3` config bases.

## Risks / Follow-ups

- The automated Rust and frontend test suites still need to be run in an environment with Rust and Bun installed.
- FMP `stable/earnings` availability may vary by plan/key. The endpoint intentionally degrades to macro-only output with `earnings_status` instead of failing the user action.
- The macro calendar is currently a code seed, not an admin-editable data source.
- Dense earnings days are capped visually in the PNG card with a `+N 项` overflow indicator; future versions may need detail drill-down or per-day expansion.

## Next Entry Point

- Backend API/data flow: `crates/hone-web-api/src/routes/public_finance_calendar.rs`
- Frontend quick action/send flow: `packages/app/src/pages/chat.tsx`
- PNG template: `packages/app/src/components/finance-calendar-card.tsx`
