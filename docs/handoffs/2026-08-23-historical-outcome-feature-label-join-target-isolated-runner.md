# Historical Outcome Feature-label Join / Target Isolated Runner Handoff

## Outcome

Stage 40 registers an immutable isolated runner specification for the current independently approved join/target implementation. The record is create-once, content-addressed and permanently `registered_not_run`; it does not expose an execution entrypoint.

## Files

- Backend registry and validation: `crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_isolated_runners.rs`
- Routes: `crates/hone-web-api/src/routes/mod.rs`
- Readiness v37: `crates/hone-web-api/src/routes/investment_decisions.rs`
- Frontend types/API: `packages/app/src/lib/types.ts`, `packages/app/src/lib/api.ts`
- Administrator UI: `packages/app/src/components/public-admin-historical-outcome-feature-label-join-target-isolated-runner-panel.tsx`
- Governance/decision-brain integration: `packages/app/src/components/public-admin-historical-outcome-governance-panel.tsx`, `packages/app/src/components/public-admin-decision-brain-panel.tsx`

## Contract

- Exact current Stage 39 approval and complete upstream hashes are rebound on every eligibility read.
- The registrar is excluded from all implementation, specification and official-artifact upstream actors.
- Runner artifact/revision, runtime, read-only inputs, create-once untrusted output and static resource limits enter the content fingerprint.
- There is no callable entrypoint, environment inheritance, environment variable, secret, network, tool, child process, label/training-store or production access.
- Label access, join, target assignment, joined/training rows, output validation, training, reward, shadow, order, broker and trading authority remain false.

## Next gate

Only an independent first-execution authorization review may follow. Registration is not execution, does not create output and does not prove the engineering target is predictive or profitable.

## Verification

- Stage 40 focused backend tests: 8 passed.
- Stage 39 implementation-review regression: 9 passed; readiness v37 regression: 1 passed.
- Full `hone-web-api`: 734 passed, 2 credential/live tests ignored by design, 0 failed.
- Frontend: 517 tests passed; 31 decision-brain contract tests passed; TypeScript, normal production build and public-mode production build passed.
- Workspace all-target check passed with desktop bundled-resource existence validation explicitly skipped for the dev/IDE check; Rust format and diff hygiene passed.

No browser runtime, isolated process, live data join or trading execution is claimed by this handoff.
