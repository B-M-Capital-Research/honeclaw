# Reminder Cancellation And Feishu Table Production Activation

- title: Reminder Cancellation And Feishu Table Production Activation
- status: done
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
  - `docs/invariants.md`
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

- Targeted reminder/storage/tool/scheduler/channel tests passed; `hone-feishu` passed `69/69`, including standard Markdown, historical raw component syntax, direct, placeholder, scheduler, and shared-renderer cases.
- `bash scripts/ci/check_fmt_changed.sh` passed.
- `cargo check --workspace --all-targets --exclude hone-desktop --exclude hone-user-app` passed.
- `cargo test --workspace --all-targets --exclude hone-desktop --exclude hone-user-app` passed.
- Web passed `302/302`; Public Community Edge typecheck and `45/45` tests passed.
- `bash tests/regression/run_ci.sh` passed, including `44/44` finance contracts.
- Exact `caa45733819a404ebc7e383f8b830b0a26bcff80` immutable package passed all `500` payload checks; manifest SHA-256 is `4ee22285a327e22a8884770cf492893d39887103ea3c272f85f36b18e164edec`.
- Production reports `0.15.3`, cloud-authoritative PostgreSQL/OSS health, zero local durable dependencies, zero active chats, expected origin/public `401` JSON boundaries, exact new process paths, and established Feishu TLS connections.

## Documentation Sync

- Track this cross-module behavior task in `docs/current-plan.md` and this plan.
- Update the relevant bug ledger and any long-lived repository map/invariant only if the implementation changes a durable contract or module boundary.
- On completion, archive this plan, add a handoff and archive-index entry, and remove the active-plan entry.

## Risks / Open Questions

- Bulk cancellation remains actor-scoped and idempotent; storage write/read failures no longer report success.
- Claimed one-shot jobs retain their one valid execution, while jobs cancelled before work or before outbound delivery are suppressed.
- No live user-target Feishu message was sent during deployment; native table behavior is covered by `69/69` local Feishu tests and the production worker is connected.
- Discord remains in its pre-existing stopped credential state and is outside this repair.

## Completion

- Fix commit `caa45733819a404ebc7e383f8b830b0a26bcff80` is on `origin/main`.
- Production Web and Feishu now run `target/deploy-caa45733`; `target/deploy-8491a3c2` remains the immediate rollback.
- The established prompt answer format is unchanged; only two explicit tool-routing instructions were added.
- Handoff: `docs/handoffs/2026-07-27-reminder-cancellation-and-feishu-table-activation.md`.
