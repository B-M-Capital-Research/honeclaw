# Stage 43 Historical Outcome Feature-label Join / Target Output Validation

- title: Stage 43 independent join/target output validation
- status: completed
- created_at: 2026-08-23
- updated_at: 2026-08-23
- owner: Codex
- related_files: `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_output_validations.rs`, `crates/hone-web-api/src/routes/mod.rs`, `crates/hone-web-api/src/routes/investment_decisions.rs`, `packages/app/src/components/public-admin-historical-outcome-feature-label-join-target-output-validation-panel.tsx`, `packages/app/src/components/public-admin-historical-outcome-governance-panel.tsx`, `packages/app/src/components/public-admin-decision-brain-panel.tsx`, `packages/app/src/lib/api.ts`, `packages/app/src/lib/types.ts`
- related_docs: `docs/decisions.md`, `docs/invariants.md`, `docs/repo-map.md`, `docs/current-plans/hone-investment-decision-brain.md`
- related_prs: none

## Summary

Stage 43 independently recomputes the exact completed Stage 42 join/target candidate and persists one immutable validation record. A passing result remains an untrusted candidate and only becomes eligible for a future, separate admission review.

## What Changed

- Added an independent validator that reopens the exact claim/result/output, current authorization audit, current raw-outcome dataset and independently validated official artifact pair.
- Recomputed claim/result/output fingerprints, one-to-one keys, all 65 PIT features and explicit missingness, official split/purge/embargo, nine raw f64 target bit patterns and withheld-label commitments without calling the Stage 42 projection or record-validation helper.
- Required the validator to be outside the execution and complete upstream actor chain. The non-empty sorted exclusion set is bound into the self-hashed create-once record.
- Added administrator GET/POST routes, a four-confirmation validation panel, immutable history, readiness v40 and the ㊸ decision-brain status card.
- Kept official joined-dataset creation, training-store copy, training, reward, shadow portfolio, order generation, broker access and trading closed.

## Verification

- Stage 43 focused backend tests: 10 passed, 0 failed.
- Full Web API library suite: 761 passed, 2 credential/live tests ignored by design, 0 failed (763 total).
- Frontend suite: 517 passed with 2263 assertions; decision-brain contract: 31 passed with 544 assertions.
- TypeScript, standard and public production builds, and workspace all-target Rust check passed.
- Rust formatting and diff hygiene passed.

## Risks / Follow-ups

- No real Stage 42 candidate was validated in this work, so there is no runtime or real-dataset acceptance claim.
- Existing dead-code, Rust future-incompatibility and frontend large-chunk advisories remain unrelated warnings.
- Validation proves deterministic reconstruction and leakage boundaries only; it does not establish predictive value, strategy profitability or correspondence to confirmed old-Wang logic.

## Next Entry Point

Stage 44 may add a separate independent candidate-admission review that consumes only an exact passing Stage 43 record. It must not copy data to the training store, start training or grant reward, shadow, order, broker or trading authority.
