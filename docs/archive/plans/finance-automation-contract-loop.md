# 金融自动化合同回归闭环

- title: 金融自动化合同回归闭环
- status: done
- created_at: 2026-04-09
- updated_at: 2026-04-11
- owner: codex
- related_files:
  - `crates/hone-tools/src/data_fetch.rs`
  - `tests/regression/ci/test_finance_automation_contracts.sh`
  - `tests/regression/manual/test_finance_snapshot_live.sh`
  - `tests/regression/manual/test_earnings_calendar_live.sh`
  - `skills/gold-analysis/SKILL.md`
  - `skills/position_advice/SKILL.md`
  - `skills/stock_selection/SKILL.md`
  - `skills/valuation/SKILL.md`
- related_docs:
  - `docs/current-plan.md`
  - `docs/handoffs/2026-04-09-finance-automation-contract-loop-round1.md`

## Goal

Fix finance automation contract drift through a stable sample set and round-based regression, starting with `data_fetch(snapshot)` support and ending with skill wording aligned to the runtime finance policy.

## Scope

- Lock one fixed 9-sample regression slice for finance automation.
- Round 1 fixed the `skills -> data_fetch(snapshot)` contract mismatch.
- Round 2 fixed the `earnings_calendar` default window drift and verified it with a real `hone-mcp` call.
- Round 3 fixed the remaining skill wording drift in `gold-analysis`, `position_advice`, `stock_selection`, and `valuation`.

## Validation

- `cargo test -p hone-tools`
- `bash tests/regression/ci/test_finance_automation_contracts.sh`
  - Result: `success=9 review=0 fail=0 total=9`
- `bash tests/regression/manual/test_finance_snapshot_live.sh`
  - Historical proof retained from Round 1 / 2
- `bash tests/regression/manual/test_earnings_calendar_live.sh`
  - Historical proof retained from Round 2
- `bash tests/regression/run_ci.sh`
  - Passed after the wording fixes

## Documentation Sync

- The task is archived because the fixed 9-sample slice is now fully green.
- Historical details remain in the existing handoff and archive index entry.

## Risks / Open Questions

- Future finance skills should be checked against the global finance prompt before landing to avoid reintroducing direct recommendation language.
- If the policy evolves further, the contract script should be extended instead of relying on manual review.
