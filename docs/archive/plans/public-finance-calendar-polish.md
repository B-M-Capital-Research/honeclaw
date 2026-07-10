# Public Finance Calendar Polish

- title: Public Finance Calendar Polish
- status: archived
- created_at: 2026-07-10
- updated_at: 2026-07-10
- owner: Codex
- related_files:
  - `crates/hone-web-api/src/routes/public_finance_calendar.rs`
  - `packages/app/src/pages/chat.tsx`
  - `packages/app/src/components/finance-calendar-card.tsx`
  - `packages/app/src/lib/finance-calendar.ts`
  - `packages/app/src/lib/finance-calendar.test.ts`
- related_docs:
  - `docs/archive/plans/public-finance-calendar.md`
  - `docs/handoffs/2026-06-29-public-finance-calendar.md`

## Goal

Make the public finance calendar useful at first click: default to the current month, load a visible preview immediately, make month switching and send states clear, redesign the generated PNG for sharing, and expand July 2026 macro coverage with verified official release dates.

## Scope

- Replaced the month-end rollover rule with a current-month default in web and API paths.
- Expanded the July 2026 macro seed from 4 to 17 high-signal events in Beijing time.
- Replaced the month-grid-only dialog with an immediate preview, previous/next controls, a 12-month select, data counts, source state, and a guarded send action.
- Redesigned the generated image at a stable 1080 x 1350 size with key-event highlights, category colors, a 42-cell adjacent-month grid, and earnings-aware dense-day overflow.
- Added focused frontend and Rust tests for the new defaults, event expansion, fixed grid, event categories, highlights, and dense-day behavior.

## Validation

- Changed TS/TSX files parsed successfully with local esbuild 0.27.7.
- Finance-calendar helper smoke passed for current-month default, 42 cells, adjacent dates, high-impact highlights, and dense-day earnings visibility.
- `git diff --check`: passed.
- `bash tests/regression/run_ci.sh`: finance automation, install path, migrations, ops arguments, sidecar wrapper, and session regressions passed; stopped at skill runtime stage because `cargo` is not installed.
- `cargo test -p hone-web-api finance_calendar`: not run; `cargo` is not installed.
- `cargo check -p hone-web-api`: not run; `cargo` is not installed.
- `bun run test:web`: not run; `bun` is not installed and dependency installation could not reach the npm registry.
- A local 1080 x 1350 visual proof was rendered and inspected at `tmp/finance-calendar-polish-2026-07.png` in the non-Git source workspace; in-app browser preview was blocked for local `data:` URLs.

## Documentation Sync

- Updated `docs/handoffs/2026-06-29-public-finance-calendar.md` with stage-two behavior, verified sources, timezone handling, and validation limits.
- `docs/repo-map.md` did not need an update because API ownership, module boundaries, and the upload/send data flow are unchanged.
- Removed this task from `docs/current-plan.md` and added the completed stage to `docs/archive/index.md`.

## Risks / Open Questions

- Macro events remain a maintained seed rather than a live economic-calendar feed; official schedules can change and should be refreshed before future months are added.
- The actual Solid/html2canvas path still needs a browser smoke in an environment with installed frontend dependencies.
- FMP plan/key availability remains independently degradable; the calendar still sends official macro events when earnings data is missing.
