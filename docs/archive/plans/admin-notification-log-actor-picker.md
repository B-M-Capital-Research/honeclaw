# Admin Notification Log and Actor Picker

- title: Admin Notification Log and Actor Picker
- status: done
- created_at: 2026-04-29
- updated_at: 2026-04-29
- owner: Codex
- related_files:
  - packages/app/src/pages/notifications.tsx
  - packages/app/src/pages/schedule.tsx
  - packages/app/src/lib/api.ts
  - packages/app/src/components/actor-select.tsx
  - crates/hone-web-api/src/routes/notifications.rs
  - crates/hone-event-engine/src/store.rs
- related_docs:
  - docs/current-plan.md
  - docs/archive/index.md
  - docs/handoffs/2026-04-29-admin-notification-log-actor-picker.md

## Goal

Fix the admin push surfaces so operators can see actual Discord / event-engine pushes in the notification log and can choose users from an actor dropdown instead of memorizing raw user IDs.

## Scope

- Expanded the notification log data source beyond cron job execution rows to include event-engine `delivery_log` rows and business event kind labels from `events.kind_json.type`.
- Preserved the existing cron-job notification history view and filters.
- Added a reusable actor picker backed by existing session / portfolio / company-profile actor summaries.
- Replaced raw `user_id` entry on notification log and push schedule pages with actor selection.

## Validation

- `cargo test -p hone-web-api routes::notifications`
- `cargo test -p hone-event-engine list_recent_delivery_logs`
- `cargo test -p hone-event-engine store::tests::delivery_log_is_append_only_across_retries`
- `bun --filter @hone-financial/app typecheck`
- `git diff --check`
- Read-only SQLite check confirmed local `data/events.sqlite3` has operator-level event-engine rows after excluding `router`, `digest_item`, and `global_digest_item`.
- Browser shape check against `http://127.0.0.1:3002/notifications` and `/schedule` confirmed both pages use actor dropdowns and no longer render raw `user_id` / `scope` inputs.
- Read-only query for `discord::::1416435661946228746` confirmed the underlying delivery rows include multiple business kinds such as `macro_event`, `news_critical`, `price_alert`, `analyst_grade`, `sec_filing`, and `earnings_released`.

## Documentation Sync

- Removed this task from `docs/current-plan.md`.
- Archived this plan under `docs/archive/plans/`.
- Added archive index and handoff entries for future maintenance.

## Risks / Open Questions

- The live `127.0.0.1:8077` backend was served by the cached runtime under `~/Library/Caches/honeclaw/target`, so the Rust API behavior needs a rebuild/restart before this source change is visible in the running app.
- Actor dropdown options come from known actors in existing admin sources. A future dedicated actor index may still be useful for actors that only have notification prefs and no session / portfolio / company profile.
