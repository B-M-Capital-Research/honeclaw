# Finance Automation Contract Loop Round 1

- title: Finance Automation Contract Loop Round 1
- status: in_progress
- created_at: 2026-04-09
- updated_at: 2026-04-09
- owner: codex
- related_files:
  - `crates/hone-tools/src/data_fetch.rs`
  - `tests/regression/ci/test_finance_automation_contracts.sh`
  - `docs/current-plans/finance-automation-contract-loop.md`
- related_docs:
  - `docs/current-plan.md`
- related_prs:

## Summary

Round 1 fixed the `snapshot` contract gap. Round 2 then fixed the `earnings_calendar` default window drift and proved it with a real `hone-mcp` live run.

## What Changed

- Added `snapshot` to the `data_fetch` tool contract.
- Implemented aggregated `snapshot` payloads as `quote + profile + news`, with partial-error visibility when one component fails.
- Added Rust unit tests covering the new tool enum exposure and snapshot aggregation behavior.
- Added `tests/regression/ci/test_finance_automation_contracts.sh` to keep the same 9-sample slice stable across future rounds.
- Fixed a real execute-path bug discovered by a live `hone-mcp` call: `snapshot` was still rejected by an early legacy `match` in `execute()`.
- Added `tests/regression/manual/test_finance_snapshot_live.sh` so future rounds can repeat a real external `snapshot` run instead of relying only on static checks.
- Changed `earnings_calendar` to default to the current Beijing date through the next 14 days, with optional `from` / `to` overrides.
- Added `request_window` to `earnings_calendar` responses so live regression can verify the effective query window.
- Switched FMP response parsing to `text() + serde_json::from_str()` after a real live run showed `resp.json()` was failing on valid endpoint responses.
- Added `tests/regression/manual/test_earnings_calendar_live.sh` for the real default-window earnings-calendar path.

## Verification

- `rtk cargo test -p hone-tools -- --nocapture`
  - `42 passed`
- `rtk bash tests/regression/ci/test_finance_automation_contracts.sh`
  - `success=5 review=1 fail=3 total=9`
- `rtk bash tests/regression/manual/test_finance_snapshot_live.sh`
  - passed with a real `AAPL snapshot` response through `hone-mcp`
- `rtk bash tests/regression/manual/test_earnings_calendar_live.sh`
  - passed with a real default-window earnings-calendar response through `hone-mcp`
- Baseline versus current:
  - Baseline: `success=1 review=1 fail=7`
  - After Round 1: `success=3 review=1 fail=5`
  - After Round 2: `success=5 review=1 fail=3`

## Risks / Follow-ups

- `gold-analysis` remains a template stub.
- `position_advice` and `stock_selection` still conflict with the global finance policy by pushing direct action language.
- `valuation` remains under review because it still ends in categorical labels instead of explicitly conditional analysis.

## Next Entry Point

- Continue from `docs/current-plans/finance-automation-contract-loop.md`.
- Next likely step: Round 3 on the skill wording / policy conflict cluster, without mixing in unrelated tool-layer changes.
