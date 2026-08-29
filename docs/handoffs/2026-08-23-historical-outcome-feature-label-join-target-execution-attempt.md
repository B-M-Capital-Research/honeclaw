# Stage 42 Historical Outcome Feature-label Join / Target Execution Attempt

## Outcome

Implemented the one-shot execution boundary after Stage 41. The endpoint consumes one exact current authorization with a create-once claim before a fixed in-process projection. Success and failure both consume the authorization.

## Contract

- Inputs are limited to the exact independently validated official split manifest, official 65-feature bundle and current bound raw-outcome dataset.
- The join is exact one-to-one by `dataset_entry_id`; duplicate/missing keys, feature-catalog drift, future features, ambiguous missingness and purge/embargo leakage fail closed.
- Only train rows contain the frozen nine raw f64 target bit patterns. Validation and sealed-holdout targets are withheld and represented only by commitments.
- Output is a content-addressed untrusted candidate. It is not an official joined dataset or training input.
- Generic label/training stores, training, reward, shadow portfolio, order generation, broker access and trading remain closed.

## Surfaces

- Backend registry and `invoke-once` routes are mounted under `/api/public/admin/investment-decisions/historical-outcome-feature-label-join-target-execution-attempts`.
- The historical-outcome governance screen includes a Stage 42 panel with four irreversible confirmations and immutable attempt history.
- The decision-brain readiness card reports eligible authorizations, attempts, failures, untrusted candidates and independent-validation eligibility under readiness v39.

## Verification

- 9 focused Rust tests passed for exact join, withheld targets, duplicate/missing/future data, explicit missingness, purge handling, target-contract drift, claim authority, result authority and failure consumption.
- Full Web API library suite: 751 passed, 2 ignored credential/live tests, 0 failed (753 total).
- Frontend TypeScript passed; full frontend suite: 517 passed, 0 failed, 2256 assertions.
- The 31-test decision-brain source contract passed with 537 assertions.
- Standard and public production builds passed with the existing large-chunk advisory only.
- Workspace all-target Rust check passed with `HONE_SKIP_BUNDLED_RESOURCE_CHECK=1`; the flag only skips the absent local Tauri sidecar existence check. Rust format and `git diff --check` passed.

## Non-claims and next gate

No real authorization was claimed and no real dataset join was run. This stage adds no confirmed old-Wang investment logic and does not change Hari Invest 0.1.0. The only allowed next stage is independent output recomputation and leakage validation; it still must not copy candidates to a training store or train a model.
