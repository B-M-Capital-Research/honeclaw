# Market-move attribution and quote data integrity

- title: Market-move attribution and quote data integrity
- status: in_progress
- created_at: 2026-08-22
- updated_at: 2026-08-22
- owner: Codex
- related_files:
  - `crates/hone-tools/src/data_fetch.rs`
  - `crates/hone-channels/src/investment_response_guard.rs`
  - `agents/function_calling/src/lib.rs`
  - `tests/regression/ci/test_finance_automation_contracts.sh`
- related_docs:
  - `docs/current-plan.md`
  - `docs/decisions.md`
  - `docs/invariants.md`

## Goal

Prevent HONE from mixing pre-market, regular-session and post-market return bases; from publishing internally inconsistent provider quote fields; from using stale or different-date evidence as a same-day market-move cause; and from calling a single-stock move sector-wide when same-date sector evidence contradicts it.

## Scope

- Add canonical regular-session close-to-close and extended-session comparison semantics to normalized market data.
- Quarantine quote change fields unless price, previous close, absolute change and percentage change are complete and arithmetically consistent.
- Make the market-move final checker consume only quote evidence authorized for change claims.
- Require same-date sector comparison before a single-stock answer may claim a systemic or sector-wide selloff.
- Remove paragraph-wide uncertainty bypasses from causal-claim checking.
- Keep unsolicited trading actions out of a pure attribution answer.
- Add a regression based on the 2026-08-21 MRVL case and audit checked-in market-price artifacts for arithmetic/date consistency.

## Validation

- Targeted `hone-tools` unit tests for quote arithmetic and session-return bases.
- Targeted `function_calling` tests for MRVL percentage, stale-cause, systemic-selloff contradiction and unsolicited-action rejection.
- `cargo test -p hone-tools` and `cargo test -p function_calling` (or the exact package name from the workspace).
- `bash tests/regression/ci/test_finance_automation_contracts.sh`.
- Run the repository quote-integrity diagnostic against checked-in daily market artifacts.
- Run formatting checks on changed Rust files.

## Documentation Sync

- Record the durable quote/attribution contract in `docs/invariants.md` and `docs/decisions.md`.
- On completion, add a handoff, move this plan to `docs/archive/plans/`, update `docs/archive/index.md`, and remove this entry from `docs/current-plan.md`.

## Risks / Open Questions

- The worktree already contains unrelated and overlapping investment/data changes; preserve them and patch only the relevant functions.
- Provider fields can be missing or rounded. Missing data must fail closed for exact change claims; reasonable provider rounding remains allowed.
- A deterministic guard can reject contradicted/systemic claims but must not invent the true economic cause of a move.

## 2026-08-22 TEM response-integrity increment

Status: `done` within this still-active umbrella plan.

- Added a dedicated `EquityMove` route for a single-stock “why did it rise/fall” question. It produces a five-section attribution report and does not inherit the nine-section investment-decision template.
- Made attribution answers distinguish direct company events, indirect industry/related-entity mapping and background factors, with an explicit confidence level and the existing per-clause date/domain evidence contract.
- Closed the prompt/validator contradiction that previously told the model not to give advice while rejecting any deep single-stock answer without an action section. A pure attribution answer now rejects unsolicited buy/hold/add/reduce/sell/stop/entry-price copy.
- Added publication guards for the TEM failure classes: negative P/E or EV/EBITDA must be `N/M`; MRD cannot be described as population cancer screening; precise Bull/Base/Bear price targets require a disclosed formula and share/dilution bridge; unsupported RSI/overbought, programmatic-buying, short-covering and inconsistent 200-day-average claims fail closed; the first GAAP-profit claim must disclose operating-profit and non-recurring/unrealized-income quality in the conclusion.
- Expanded the runtime research contract so GAAP/Adjusted metrics, relationship legal entities and transaction status, medical terminology, technical indicators and scenario-valuation arithmetic are checked before publication.

Verification completed:

- `cargo check -p hone-channels --all-targets` — passed; one pre-existing dead-code warning for `is_broad_scope_request`.
- `cargo test -p hone-channels --lib investment_response_guard::tests -- --nocapture` — 121 passed, 0 failed.
- TEM-specific bad-answer, safe-fallback, route-selection and valuation/technical helper regressions — passed.
- `rustfmt --edition 2024 crates/hone-channels/src/investment_response_guard.rs` — passed.
