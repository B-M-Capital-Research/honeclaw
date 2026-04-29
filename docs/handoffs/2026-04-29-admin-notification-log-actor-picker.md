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
  - docs/archive/plans/admin-notification-log-actor-picker.md
  - docs/archive/index.md
- related_prs: N/A

## Summary

The admin notification log previously only queried cron execution records, so event-engine pushes delivered through Discord and other sinks were invisible. The notification log now merges cron rows with event-engine `delivery_log` rows, and both notification log and schedule pages use an actor dropdown instead of manual `user_id` entry.

## What Changed

- Added `EventStore::list_recent_delivery_logs` with filters for time window, actor, event id, status, delivery channel, operator-level rows, and `events.kind_json.type`.
- Updated `/api/admin/notifications` to merge cron records and event-engine delivery records, preserve source labels, expose event-engine statuses such as `sent`, `dryrun`, `queued`, `quiet_held`, `filtered`, `capped`, `cooled_down`, `omitted`, and `failed`, and return `event_kind` for the frontend event-type column.
- Excluded `router`, `digest_item`, and `global_digest_item` from the default operator-level event log so no-actor routing misses and item-level digest internals do not hide actual delivery rows.
- Added `ActorSelect` and used it on `/notifications` and `/schedule`.

## Verification

- `cargo test -p hone-web-api routes::notifications`
- `cargo test -p hone-event-engine list_recent_delivery_logs`
- `cargo test -p hone-event-engine store::tests::delivery_log_is_append_only_across_retries`
- `bun --filter @hone-financial/app typecheck`
- `git diff --check`
- Read-only SQLite check on `data/events.sqlite3` confirmed operator-level rows such as `sent|sink`, `dryrun|sink`, `queued|digest`, and `quiet_held|sink` exist after internal rows are excluded.
- Browser shape check against `http://127.0.0.1:3002/notifications` and `/schedule` confirmed both pages use actor dropdowns and no longer render raw `user_id` / `scope` inputs.
- Read-only query for `discord::::1416435661946228746` confirmed the underlying delivery rows include multiple business kinds such as `macro_event`, `news_critical`, `price_alert`, `analyst_grade`, `sec_filing`, and `earnings_released`.

## Risks / Follow-ups

- The live `127.0.0.1:8077` backend was served by the cached runtime under `~/Library/Caches/honeclaw/target`, so rebuild/restart that runtime before expecting the merged event-engine log behavior in the running app.
- The dropdown uses existing session / portfolio / company-profile actor summaries. Actors with only notification preferences but no record in those sources will still not appear until a dedicated actor index exists.

## Next Entry Point

- `/notifications` for merged push log behavior.
- `/schedule` for per-actor push schedule lookup.
