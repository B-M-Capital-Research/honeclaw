# Weekly Brief Readable Calendar

- title: Weekly Brief Readable Calendar
- status: `completed_locally`
- created_at: `2026-08-11`
- updated_at: `2026-08-11`
- owner: `Codex`
- related_files:
  - `crates/hone-web-api/src/routes/weekly_brief.rs`
  - `crates/hone-web-api/src/routes/public_finance_calendar.rs`
  - `crates/hone-web-api/src/routes/key_event_chain.rs`
  - `packages/app/src/components/weekly-brief-dashboard.tsx`
  - `packages/app/src/components/key-event-chain-dashboard.tsx`
  - `packages/app/src/pages/chat.tsx`

## Goal

Replace the event-chain ten-day verification view with a standalone weekly brief that presents the previous Beijing Monday–Sunday review and the next Monday–Sunday calendar in a readable, structured interface. Reuse finance-calendar JSON and attributed event-chain evidence instead of embedding or reusing a PNG.

## Scope

- Add an actor-scoped weekly-brief API that combines macro schedules, covered/held-company earnings when the configured provider can verify dates, and attributable industry milestones.
- Mark past calendar rows as schedules whose outcomes still need source verification; never infer releases or results from a date alone.
- Mark future rows as reminders, not predictions, and attach deterministic analysis of why each event matters and what to check.
- Add a chat-home launcher and a responsive agenda with three compact tabs: previous week, next week and a 30-day AI earnings/conference horizon.
- Unify launcher geometry and render Valuation Lab / Research Library as subordinate utility cards.
- Remove the ten-day view and wording from the key-event-chain dashboard while retaining its first-principles timeline.

## Validation

- Focused Rust tests for Beijing week boundaries, event classification, analysis text, deduplication and fail-closed earnings status.
- Frontend component/API/type contract tests, TypeScript, full Web tests and public production build.
- Authenticated desktop and mobile browser acceptance against the local Vite/API runtime.

## Documentation Sync

- Record the product boundary in `docs/decisions.md`.
- Add a handoff under `docs/handoffs/` and archive this plan after local acceptance.

## Risks / Open Questions

- The finance calendar's macro dates are curated schedules. The weekly brief must not present them as actual results.
- Earnings dates depend on the configured FMP coverage. Missing keys or unsupported symbols must remain visible as incomplete coverage, never be replaced with guessed dates.
- Historical key-event-chain evidence may be sparse or source-only. Empty weeks must be represented honestly.

## Completion

- Completed locally on 2026-08-11 with the standalone API, unified launcher hierarchy, tabbed responsive agenda, official Hot Chips/NVIDIA dates, event-chain extraction, tests, local API smoke and authenticated browser acceptance.
- No commit, push or production deployment was requested or performed.
