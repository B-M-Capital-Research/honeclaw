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
  - `docs/handoffs/2026-06-29-public-finance-calendar.md`
  - `docs/repo-map.md`

## Goal

Add a public user "我的财经日历" quick action after the existing portfolio push-mode tip. Users can choose one of 12 months, generate a month-view finance calendar image, and send it into their chat history as a Hone assistant message.

## Scope

- Backend public APIs:
  - `GET /api/public/finance-calendar?month=YYYY-MM`
  - `POST /api/public/finance-calendar/send`
- Finance-calendar data:
  - current date and 12 month choices
  - built-in macro seed events, initially 2026-07 only
  - actor-scoped portfolio/watchlist tickers and matching FMP earnings dates
- Frontend:
  - month picker quick action
  - renderable month-view calendar image template
  - html2canvas PNG upload and send flow
- Out of scope:
  - recurring automatic calendar push
  - admin-managed macro calendar editor
  - full-year single-image view

## Validation

- `cargo test -p hone-web-api finance_calendar`: not run; `cargo` missing in current environment.
- `bun run test:web`: not run; `bun` missing in current environment.
- `bash scripts/ci/check_fmt_changed.sh`: completed, skipped changed-file rustfmt because no base ref was discoverable.
- `cargo test --workspace --all-targets --exclude hone-desktop`: not run; `cargo` missing in current environment.

## Documentation Sync

- Updated `docs/repo-map.md` with the public chat quick action and API flow.
- Added `docs/handoffs/2026-06-29-public-finance-calendar.md`.
- Archived this plan from `docs/current-plans/public-finance-calendar.md` to this file.
- Updated `docs/archive/index.md`.

## Risks / Open Questions

- FMP stable earnings endpoint availability may vary by plan/key; API degrades to macro-only output instead of failing the user action.
- Public file proxy only permits current-user public uploads; generated PNGs must go through `/api/public/upload` before `/send`.
- Month-view cells can become dense if many holdings report in the same month; v1 caps displayed chips per day and summarizes overflow.
