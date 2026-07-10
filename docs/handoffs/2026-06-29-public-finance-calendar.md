# Public Finance Calendar

- title: Public Finance Calendar
- status: done
- created_at: 2026-06-29
- updated_at: 2026-07-10
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
  - `docs/archive/plans/public-finance-calendar-polish.md`
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

## Stage 2: Experience And Visual Polish (2026-07-10)

- The quick action now always opens the current month and fetches it immediately. The dialog shows a true calendar-image preview, previous/next controls, a 12-month select, macro/earnings/holding counts, source state, and a send button that stays disabled until valid data is ready.
- Request failures have a distinct failed-preview state instead of looking permanently busy. FMP degradation is shown as a data-source status while the macro calendar remains sendable.
- The PNG is now a stable 1080 x 1350 editorial calendar: dark masthead, four high-impact events, a six-row grid with muted adjacent-month dates, category colors, holdings summary, and official source footer.
- Dense dates reserve space for both macro context and at least one holding earnings event. Remaining items use an explicit overflow count, while major macro events also remain visible in the highlight row.
- The frontend and API default-month rules now always return the current month; the former final-seven-days rollover was removed.
- July 2026 expanded from 4 to 17 macro events. Incorrect seeds were corrected: U.S. nonfarm payrolls are July 2 Beijing time, CPI is July 14, and the July 29 ET FOMC decision is July 30 at 02:00 Beijing time.

### Verified Event Sources

- BLS July 2026 release schedule: <https://www.bls.gov/schedule/2026/07_sched.htm>
- Federal Reserve July 2026 calendar: <https://www.federalreserve.gov/newsevents/2026-july.htm>
- BEA 2026 release schedule: <https://www.bea.gov/news/schedule/>
- Census 2026 economic indicator calendar: <https://www.census.gov/economic-indicators/calendar-listview.html>
- ISM 2026 PMI report calendar: <https://www.ismworld.org/supply-management-news-and-reports/reports/rob-report-calendar/>

July times are stored in Beijing time. U.S. Eastern releases were converted using the July daylight-saving offset (EDT, UTC-4; Beijing is UTC+8).

### Stage 2 Verification

- Changed TS/TSX syntax parse: passed with local esbuild.
- Finance-calendar helper smoke: passed for current-month default, 42 cells, adjacent dates, highlights, and dense-day earnings visibility.
- `git diff --check`: passed.
- `bash tests/regression/run_ci.sh`: passed the available finance, install, migration, ops, sidecar, and session checks, then stopped when a later check required missing `cargo`.
- Rust unit/check commands were not run because `cargo` is not installed.
- `bun run test:web` was not run because `bun` is not installed; dependency installation was also blocked by npm registry DNS resolution.
- In-app browser rendering was blocked for local `data:` URLs. A local 1080 x 1350 image proof was rendered separately and inspected for hierarchy, dense-day overflow, adjacent-month cells, and footer readability.
