# Web Scheduled Push Inbox

- title: Web Scheduled Push Inbox
- status: archived
- created_at: 2026-07-10
- updated_at: 2026-07-10
- owner: Codex
- related_files:
  - `crates/hone-core/src/cloud_runtime.rs`
  - `memory/src/cron_job/`
  - `crates/hone-channels/src/scheduler.rs`
  - `crates/hone-web-api/src/routes/`
  - `packages/app/src/components/public-push-center.tsx`
  - `packages/app/src/pages/chat.tsx`
- related_docs:
  - `docs/decisions.md`
  - `docs/repo-map.md`
  - `docs/handoffs/2026-07-10-web-scheduled-push-inbox.md`

## Goal

Upgrade Web-only scheduled-task delivery from full-length chat text into summary cards with full-content drill-down, a unified push inbox, durable read state, and a sidebar unread indicator without changing Feishu, Discord, Telegram, or iMessage delivery.

## Result

- Added actor-scoped SQLite/PostgreSQL push persistence with stable scheduler delivery keys, separate summaries/full content, delivery time, and `read_at`.
- Added authenticated summary-list and full-detail public APIs with mark-through read semantics.
- Added Web scheduler metadata, SSE/history card projection, and compatibility cards for legacy scheduled transcript pairs.
- Added bilingual desktop/mobile push center, summary cards, detail dialog, pagination, and aggregate unread dot.

## Verification

- Rust Web API suite: 93 passed, 2 ignored.
- Targeted memory read-through regression and `cargo check -p hone-channels -p hone-web-api` passed.
- Web tests: 203 passed; typecheck and public production build passed.
- `cargo check --workspace --all-targets --exclude hone-desktop`, `bash tests/regression/run_ci.sh`, and `hone-cli doctor` passed.
- Local PostgreSQL + HTTP smoke proved unread transition `3 -> 1 -> 0` when opening the middle and latest pushes; temporary records were removed.
- The in-app browser reached the SMS login surface after the final restart and after correcting the dev proxy to the public backend on `8088`.

## Documentation Sync

- Recorded decision `D-2026-07-10-01` in `docs/decisions.md`.
- Updated `docs/repo-map.md`, archived this plan, added a handoff, and removed the active-plan index entry.

## Risks / Follow-ups

- Legacy scheduled transcript cards have no durable push id and therefore intentionally do not change unread state.
- Full authenticated browser QA requires an existing SMS-authenticated browser session; API behavior, component logic, typecheck, tests, and production build provide the current verification evidence.
- When starting `hone-cli web user-ui --dev` manually, pass `--backend-url http://127.0.0.1:8088`; automatic local runtime port discovery can otherwise select the admin API on `8077`.
