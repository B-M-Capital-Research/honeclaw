- title: Scheduler Commodity Guard False Positive
- status: done
- created_at: 2026-05-27 03:04 CST
- updated_at: 2026-05-27 03:04 CST
- owner: Codex
- related_files:
  - `crates/hone-channels/src/scheduler.rs`
  - `docs/bugs/README.md`
  - `docs/bugs/scheduler_commodity_guard_false_positive_market_review.md`
  - `docs/current-plan.md`
  - `docs/current-plans/active-bug-burn-down-2026-04-28.md`
- related_docs:
  - `docs/current-plans/active-bug-burn-down-2026-04-28.md`
- related_prs:

## Summary

Re-closed the reopened scheduler commodity-guard false positive so broad market / cross-asset scheduled reports are no longer fully replaced by the oil safety notice when they only mention oil as one observation item.

## What Changed

- Tightened `guard_commodity_causality_for_event(...)` in [`/Users/fengming2/Desktop/honeclaw/crates/hone-channels/src/scheduler.rs`](/Users/fengming2/Desktop/honeclaw/crates/hone-channels/src/scheduler.rs) to compare broad-market anchors against commodity anchors before treating a long or low-segmentation body as predominantly commodity-related.
- Kept the commodity guard active for true oil-dominant bodies, including ordinary non-heartbeat jobs whose正文主体仍然是原油播报。
- Updated the bug doc and bug navigation so this item is `Fixed` rather than active.

## Verification

- `cargo test -p hone-channels commodity_guard_ --lib -- --nocapture`
- `cargo test -p hone-channels commodity_ --lib -- --nocapture`
- `cargo check -p hone-channels --tests`
- `rustfmt --edition 2024 --config skip_children=true --check crates/hone-channels/src/scheduler.rs`

## Risks / Follow-ups

- This automation did not restart or redeploy any live process, so the item is `Fixed` rather than `Closed`.
- If a current binary still rewrites `OWALERT_*` / `XME` / broad-market scheduler content into the oil notice, reopen the same bug doc with the new `cron_job_runs` samples.

## Next Entry Point

Start from [`/Users/fengming2/Desktop/honeclaw/docs/bugs/README.md`](/Users/fengming2/Desktop/honeclaw/docs/bugs/README.md) to confirm the active queue is still empty, then inspect [`/Users/fengming2/Desktop/honeclaw/docs/bugs/scheduler_commodity_guard_false_positive_market_review.md`](/Users/fengming2/Desktop/honeclaw/docs/bugs/scheduler_commodity_guard_false_positive_market_review.md) if a fresh runtime sample appears.
