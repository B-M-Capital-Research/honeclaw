# Finance Automation Contract Loop Round 1

- title: Finance Automation Contract Loop Round 1
- status: done
- created_at: 2026-04-09
- updated_at: 2026-04-11
- owner: codex
- related_files:
  - `crates/hone-tools/src/data_fetch.rs`
  - `tests/regression/ci/test_finance_automation_contracts.sh`
  - `docs/archive/plans/finance-automation-contract-loop.md`
  - `skills/gold-analysis/SKILL.md`
  - `skills/position_advice/SKILL.md`
  - `skills/stock_selection/SKILL.md`
  - `skills/valuation/SKILL.md`
- related_docs:
  - `docs/current-plan.md`
- related_prs:

## Summary

Round 1 fixed the `snapshot` contract gap. Round 2 then fixed the `earnings_calendar` default window drift and proved it with a real `hone-mcp` live run. Round 3 cleared the remaining skill wording drift so the fixed 9-sample finance contract slice is now fully green.

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
- Replaced the template stub in `skills/gold-analysis/SKILL.md` with a real gold / ETF / miner analysis workflow.
- Reworded `skills/position_advice/SKILL.md` to focus on risk, triggers, and exposure structure instead of direct trim/add/hold instructions.
- Reworded `skills/stock_selection/SKILL.md` to output a comparison shortlist rather than a recommendation list.
- Reworded `skills/valuation/SKILL.md` so the conclusion is conditional on assumptions instead of categorical.

## Verification

- `cargo test -p hone-tools -- --nocapture`
  - `42 passed`
- `bash tests/regression/ci/test_finance_automation_contracts.sh`
  - after Round 2: `success=5 review=1 fail=3 total=9`
  - after Round 3: `success=9 review=0 fail=0 total=9`
- `bash tests/regression/manual/test_finance_snapshot_live.sh`
  - passed with a real `AAPL snapshot` response through `hone-mcp`
- `bash tests/regression/manual/test_earnings_calendar_live.sh`
  - passed with a real default-window earnings-calendar response through `hone-mcp`
- `bash tests/regression/run_ci.sh`
  - passed after the Round 3 skill wording cleanup
- Baseline versus current:
  - Baseline: `success=1 review=1 fail=7`
  - After Round 1: `success=3 review=1 fail=5`
  - After Round 2: `success=5 review=1 fail=3`
  - After Round 3: `success=9 review=0 fail=0`

## Risks / Follow-ups

- The remaining risk is regression drift if future finance skills reintroduce direct action wording without updating the contract script at the same time.

## Next Entry Point

- Reopen from `docs/archive/plans/finance-automation-contract-loop.md` only if a future finance capability drifts away from the fixed 9-sample contract slice.
