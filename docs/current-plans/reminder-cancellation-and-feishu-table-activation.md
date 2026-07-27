# Reminder Cancellation And Feishu Table Production Activation

- title: Reminder Cancellation And Feishu Table Production Activation
- status: in_progress
- created_at: 2026-07-27
- updated_at: 2026-07-27
- owner: Codex
- related_files:
  - `crates/hone-scheduler/`
  - `crates/hone-event-engine/`
  - `crates/hone-channels/src/`
  - `bins/hone-feishu/src/markdown.rs`
  - `bins/hone-feishu/src/outbound.rs`
  - `docs/bugs/`
- related_docs:
  - `docs/handoffs/2026-07-27-feishu-native-table-rendering.md`
  - `docs/runbooks/backend-deployment.md`
  - `docs/current-plan.md`
  - `docs/archive/index.md`

## Goal

Guarantee that a user request to cancel all automatic reminders removes or disables every actor-owned scheduled delivery so no later automatic task fires, and activate the already released `v0.15.3` Feishu native-table renderer in production without changing the established answer format.

## Scope

- Enumerate user-facing reminder creation, listing, cancellation, scheduler due-job, event-engine, heartbeat, and direct-channel delivery paths.
- Identify whether “cancel all” is parsed as a single-task delete, scoped incompletely, or bypassed by another durable source.
- Make cancellation authoritative and idempotent across all automatic reminder sources while preserving other users' tasks.
- Add regression coverage proving cancelled actor-owned tasks cannot be claimed or delivered, including legacy/durable records where applicable.
- Revalidate `v0.15.3` standard Markdown and historical raw-table conversion for Feishu direct, placeholder finalization, and scheduler delivery.
- Push the exact tested commit, build a new immutable runtime package, drain active chats, and restart Web/Feishu through the established supervisors.

## Validation

- Targeted scheduler, event-engine, channel, and Feishu tests for both defects
- `bash scripts/ci/check_fmt_changed.sh`
- `cargo check --workspace --all-targets --exclude hone-desktop --exclude hone-user-app`
- `cargo test --workspace --all-targets --exclude hone-desktop --exclude hone-user-app`
- `bun run test:web`
- `cd workers/public-community-edge && bun run typecheck && bun run test`
- `bash tests/regression/run_ci.sh`
- Exact immutable deployment manifest verification
- Production version/process-path, active-chat, cloud-storage, channel, auth-boundary, and Feishu delivery probes

## Documentation Sync

- Track this cross-module behavior task in `docs/current-plan.md` and this plan.
- Update the relevant bug ledger and any long-lived repository map/invariant only if the implementation changes a durable contract or module boundary.
- On completion, archive this plan, add a handoff and archive-index entry, and remove the active-plan entry.

## Risks / Open Questions

- “Automatic reminder” may cover explicit cron jobs, event-engine subscriptions, heartbeat tasks, or legacy scheduler records; fixing only the visible list/delete API would leave another trigger source active.
- Bulk cancellation must remain actor-scoped and must not delete another user's reminders.
- Production still runs `0.15.2`; Feishu table behavior cannot be judged from source state until the new exact build is deployed.
- Discord remains in its pre-existing stopped credential state and is outside this repair.
