# Historical Outcome Feature-label Join / Target First-execution Authorization Handoff

## Outcome

Stage 41 adds an append-only, self-hashed independent authorization-review chain for the exact current Stage 40 runner. An approval is valid for 24 hours and grants at most one future isolated invocation. It does not claim or invoke that authorization.

## Files

- Backend authorization registry: `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_first_execution_authorizations.rs`
- Routes and readiness v38: `crates/hone-web-api/src/routes/mod.rs`, `crates/hone-web-api/src/routes/investment_decisions.rs`
- Frontend types/API: `packages/app/src/lib/types.ts`, `packages/app/src/lib/api.ts`
- Administrator review UI: `packages/app/src/components/public-admin-historical-outcome-feature-label-join-target-first-execution-authorization-panel.tsx`
- Governance and decision-brain integration: `packages/app/src/components/public-admin-historical-outcome-governance-panel.tsx`, `packages/app/src/components/public-admin-decision-brain-panel.tsx`

## Contract

- Every review binds the runner artifact/code/runtime/resources and the complete implementation, independent-review, specification, join, target, official-artifact and dataset hash chain.
- The reviewer must be independent from every upstream actor and all prior authorization reviewers.
- Approval requires independent confirmation of strict one-to-one join behavior, exactly nine raw f64 targets, point-in-time availability, explicit missingness, purge/embargo/split isolation and sealed-holdout protection.
- Approval expires after 24 hours and can support at most one future isolated invocation.
- This stage has no claim or invocation endpoint, starts no process, reads no generic label/training store, creates no output or joined rows and grants no training/reward/shadow/order/broker/trading authority.

## Next gate

Only a separately implemented and reviewed Stage 42 one-shot execution attempt may consume an unexpired authorization. This handoff does not implement or claim that gate.

## Verification

- Stage 41 focused backend tests: 8 passed, 0 failed.
- Stage 40 runner regression: 8 passed, 0 failed.
- Readiness v38 regression: passed.
- Full Web API library suite: 742 passed, 2 ignored, 0 failed (744 total).
- Frontend suite: 517 passed with 2248 assertions; decision-brain contract: 31 passed with 529 assertions.
- TypeScript, standard and public production builds, workspace all-target Rust check, Rust formatting and diff hygiene: passed.
- Remaining messages are existing dead-code, Rust future-incompatibility and frontend large-chunk warnings.

No browser runtime, isolated process execution, label access, data join, joined output, training or trading is claimed here.
