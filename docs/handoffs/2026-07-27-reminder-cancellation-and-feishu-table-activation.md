# Automatic Reminder Cancellation And Feishu Table Production Activation

- title: Automatic Reminder Cancellation And Feishu Table Production Activation
- status: done
- created_at: 2026-07-27
- updated_at: 2026-07-27
- owner: Codex
- related_files:
  - `memory/src/cron_job/storage.rs`
  - `crates/hone-tools/src/cron_job_tool.rs`
  - `crates/hone-tools/src/notification_prefs_tool.rs`
  - `crates/hone-scheduler/src/lib.rs`
  - `crates/hone-channels/src/scheduler.rs`
  - `bins/hone-feishu/src/scheduler.rs`
  - `bins/hone-feishu/src/markdown.rs`
  - `bins/hone-feishu/src/outbound.rs`
- related_docs:
  - `docs/archive/plans/reminder-cancellation-and-feishu-table-activation.md`
  - `docs/bugs/cancel_all_automatic_reminders_leaves_scheduled_jobs_active.md`
  - `docs/invariants.md`
  - `docs/runbooks/backend-deployment.md`
- related_prs:
  - fix commit `caa45733819a404ebc7e383f8b830b0a26bcff80` on `main`

## Summary

The repeated reminder cancellation failure had four independent exits: the cron tool could delete only one job, notification preference disable did not remove cron/heartbeat jobs, cron storage discarded persistence errors and could falsely report success, and already queued scheduler events were not revalidated before work or delivery. The Feishu table implementation was already released in `v0.15.3`, but production still ran the `0.15.2` package, so users continued to see historical raw `<table columns=... data=.../>` source.

Exact commit `caa45733819a404ebc7e383f8b830b0a26bcff80` closes the cancellation paths and is live for Web and Feishu. The established prompt answer format was preserved; only explicit routing for bulk cancellation actions was added.

## What Changed

- Added actor-scoped, idempotent `cron_job(remove_all)` and `notification_prefs(disable_all)`.
- `disable_all` removes the actor's cron and heartbeat jobs, disables event-engine preferences, and clears digest slots.
- Cron add/remove/bulk-remove now propagate durable load/save errors instead of reporting a false success; pending updates are cleared with deleted jobs.
- Scheduler events recheck durable job state before model/heartbeat work and channel workers recheck again immediately before outbound delivery.
- Claimed one-shot jobs retain exactly their already claimed run; later cancelled recurring work is suppressed.
- Web, Feishu, Telegram, and Discord scheduler handlers share the storage-aware execution boundary; Feishu, Telegram, and Discord also enforce the final pre-send check.
- Production now loads the existing `v0.15.3` Feishu renderer that converts standard Markdown and parseable historical raw table syntax into root-level Feishu JSON 2.0 table elements.

## Verification

- Reminder regressions:
  - memory bulk cancellation actor scope, idempotence, pending cleanup, and corrupt durable data
  - cron and notification tool actor isolation/idempotence
  - scheduler cancelled-job fail-closed and claimed one-shot semantics
  - channel cancellation before model work and before outbound send
  - empty/planning response finalization for both bulk actions
- Feishu: `69/69` tests passed, including standard Markdown, screenshot-shaped `dataIndex` records, raw-table compatibility, direct, scheduler, placeholder, and invalid-source non-leak cases.
- Repository gates:
  - changed-file rustfmt and `git diff --check`
  - full workspace check/test excluding Apple clients
  - Web `302/302`
  - Public Community Edge typecheck and `45/45`
  - complete CI-safe regression suite, including finance contracts `44/44`
- GitHub:
  - `origin/main` confirmed at `caa45733819a404ebc7e383f8b830b0a26bcff80`
  - pre-push rustfmt and gitleaks passed
- Immutable package:
  - path: `target/deploy-caa45733`
  - manifest: `501` lines / `500` payloads
  - contents: five binaries, `27` skill files, `soul.md`, and `467` public Web files
  - manifest SHA-256: `4ee22285a327e22a8884770cf492893d39887103ea3c272f85f36b18e164edec`
  - all `500` recorded SHA-256 values verified
- Production:
  - drained active chats twice before shutdown: `{"count":0}`
  - SIGINT shutdown completed cleanly; old ports released before replacement
  - Web PID `64767`, Console PID `64768`, and Feishu PID `64820` run only from `target/deploy-caa45733`
  - `/api/meta`: version `0.15.3`, cloud mode, PostgreSQL/OSS healthy, cloud storage authoritative, local durable dependency count `0`
  - origin and public unauthenticated auth endpoints both return `401 application/json`
  - Feishu worker has two established TLS connections
  - active chats remained `0` in repeated post-restart checks

## Risks / Follow-ups

- No user-facing Feishu canary was sent because no designated test actor/target was supplied. The renderer is covered by `69/69` tests and the production worker connection is healthy.
- Discord remains in its pre-existing stopped credential state and was not changed or restarted.
- Immediate rollback is the retained `target/deploy-8491a3c2` package (`0.15.2`). Drain active chats, SIGINT the current Web/Feishu supervisors, and restore both old package paths together.

## Next Entry Point

If cancellation is reported again, inspect the actor's durable cron list, notification preferences, and scheduler metadata `skipped=job_cancelled` before changing parsing. If Feishu exposes raw table source again, retain the exact assistant payload and whether it came from direct, scheduler, or placeholder finalization; do not first alter the answer-format prompt.
