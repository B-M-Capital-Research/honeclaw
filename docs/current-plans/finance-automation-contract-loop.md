# 金融自动化合同回归闭环

- title: 金融自动化合同回归闭环
- status: in_progress
- created_at: 2026-04-09
- updated_at: 2026-04-09
- owner: codex
- related_files:
  - `docs/current-plan.md`
  - `crates/hone-tools/src/data_fetch.rs`
  - `tests/regression/ci/test_finance_automation_contracts.sh`
  - `tests/regression/manual/test_finance_snapshot_live.sh`
  - `tests/regression/manual/test_earnings_calendar_live.sh`
- related_docs:
  - `AGENTS.md`

## Goal

Fix finance automation contract drift through a stable sample set and round-based regression, starting with `data_fetch(snapshot)` support as the first high-frequency issue cluster.

## Scope

- Lock one fixed 9-sample regression slice for finance automation.
- Round 1 only fixes the `skills -> data_fetch(snapshot)` contract mismatch.
- Add automated proof so later rounds reuse the same sample slice instead of changing cases.
- Round 1 implementation is complete.
- Round 2 fixes the `earnings_calendar` default window drift and verifies it with a real `hone-mcp` call.
- Later rounds remain open for the skill policy wording drift.

## Validation

- `rtk cargo test -p hone-tools -- --nocapture`
  - Result: `42 passed`
- `rtk bash tests/regression/ci/test_finance_automation_contracts.sh`
  - Result: `success=5 review=1 fail=3 total=9`
  - Round 2 accepted because the fixed sample slice improved from baseline `1/1/7` to `5/1/3`, and the resolved cluster was the two earnings-window failures
- `rtk bash tests/regression/manual/test_finance_snapshot_live.sh`
  - Result: real `hone-mcp -> data_fetch(snapshot)` run for `AAPL` returned quote, profile, and news successfully after fixing the execute-path bug
- `rtk bash tests/regression/manual/test_earnings_calendar_live.sh`
  - Result: real `hone-mcp -> data_fetch(earnings_calendar)` run returned `request_window.from=2026-04-09`, `request_window.to=2026-04-23`, and a non-error list payload

## Documentation Sync

- Keep this file and `docs/current-plan.md` aligned while the loop remains active.
- Round 1 is scoped to contract completion and regression coverage; no long-lived rule changes are expected in `docs/invariants.md`.

## Risks / Open Questions

- The fixed sample script intentionally leaves non-Round-1 gaps failing or under review; this is expected and should not be masked as success.
- Later rounds will need to decide whether skill wording issues are best fixed in prompt policy, skill docs, or both.
