# TEM market-move response integrity

- title: TEM market-move response integrity
- status: done
- created_at: 2026-08-22
- updated_at: 2026-08-22
- owner: Codex
- related_files:
  - `crates/hone-channels/src/investment_response_guard.rs`
  - `docs/current-plans/market-move-data-integrity.md`
  - `docs/invariants.md`
  - `docs/decisions.md`
- related_docs:
  - `docs/current-plan.md`
- verification:
  - `cargo check -p hone-channels --all-targets`
  - `cargo test -p hone-channels --lib investment_response_guard::tests -- --nocapture`
  - `rustfmt --edition 2024 crates/hone-channels/src/investment_response_guard.rs`
- risks:
  - The umbrella market-move data-integrity task remains active and contains overlapping uncommitted work from another development stream.
  - The technical-claim guard proves publication discipline; it does not add a new historical-price/RSI data provider.
  - Legal-entity causality is enforced through route instructions and dated evidence, but a future typed corporate-relationship graph would provide a stronger deterministic check.

## Outcome

HONE now separates a pure single-stock move explanation from a full investment decision. A user asking only why a stock rose or fell receives a five-section attribution report and cannot be given unsolicited entries, stops, targets or position actions. The server requires direct/indirect/background causal levels and confidence, and preserves the existing same-clause event date/domain rule.

The TEM regression also added fail-closed publication checks for negative valuation multiples, MRD terminology, GAAP profit quality in the conclusion, scenario-target calculation bridges, unsupported overbought/programmatic/short-covering claims and conflicting 200-day moving-average values.

## Verification result

- `cargo check -p hone-channels --all-targets`: passed. The only warning was the pre-existing unused `is_broad_scope_request` function.
- `investment_response_guard::tests`: 121 passed, 0 failed.
- The TEM bad-answer fixture is rejected for unsolicited action, GAAP-quality omission, negative P/E, MRD misclassification and unsupported technical claims.
- The deterministic move fallback passes the same five-section response gate and contains no trade action.

## Follow-up

- Continue the active umbrella plan for canonical close-to-close arithmetic, session basis and same-date sector comparisons.
- Add a typed corporate-relationship graph if HONE needs deterministic issuer/affiliate/pending-acquisition reasoning beyond the present source-contract enforcement.
- Add provider-backed historical-series calculations before allowing positive RSI or moving-average claims; until then, unsupported claims remain blocked.
