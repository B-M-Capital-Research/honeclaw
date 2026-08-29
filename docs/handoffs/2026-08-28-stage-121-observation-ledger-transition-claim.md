# Stage 121 账本转换执行尝试原子认领

- status: done
- created_at: 2026-08-28
- updated_at: 2026-08-28
- owner: Codex
- related_files:
  - crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_observation_ledger_transition_execution_attempt_claims.rs
  - crates/hone-web-api/src/routes/historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_observation_ledger_transition_first_execution_authorizations.rs
  - crates/hone-web-api/src/routes/investment_decisions.rs
  - packages/app/src/components/public-admin-controlled-shadow-observation-ledger-transition-execution-attempt-claim-panel.tsx
- related_docs:
  - docs/current-plans/hone-investment-decision-brain.md
  - docs/decisions.md
  - docs/invariants.md
- verification:
  - Stage 121 Rust 4/4
  - Web API 1244 passed / 2 ignored
  - frontend 663/663 with 3309 assertions; focused Stage 121/API coverage 146/146 with 1428 assertions
  - finance contracts 49/49
  - typecheck, standard/public builds, workspace all-target check, fmt/diff, stale-field scan and zero-record audit passed
- risks:
  - Stage 122 execution is intentionally absent.
  - Opening portfolio snapshot remains absent and financial-event allowlist remains empty.

## Outcome

Stage 121 creates one immutable, self-hashed claim before any runner entrypoint, runtime, Stage 114 admitted-output mount/read or observation-to-ledger execution. The claim atomically and permanently consumes one unexpired Stage 120 authorization. Claimant identity excludes the Stage 120 reviewer, artifact builder, Stage 119 registrar and the complete prior responsibility chain.

Stage 120 now derives consumed authorization IDs from persisted Stage 121 claims. A consumed authorization cannot be offered again, released, retried or restored after cancellation, expiry, failure or non-execution.

The claim is metadata-only. It does not execute an artifact, read an input, create a candidate output, admit an opening portfolio, write ledger events/positions/cash/NAV/performance, train a model, create reward, generate an order, access a broker or trade.

## Next gate

Only a separately reviewed Stage 122 one-shot execution gate may be designed next. It must revalidate the exact Stage 121 claim, Stage 120 artifact/manifest and Stage 114 admitted output before any read. With no separately admitted opening portfolio snapshot, it may at most create an untrusted non-financial notice candidate and must not create authoritative financial state.
