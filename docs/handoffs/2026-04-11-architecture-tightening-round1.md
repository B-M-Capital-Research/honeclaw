# Architecture Tightening Round 1

- title: Architecture Tightening Round 1
- status: done
- created_at: 2026-04-11
- updated_at: 2026-04-11
- owner: shared
- related_files:
  - `crates/hone-core/src/{config.rs,config/server.rs}`
  - `crates/hone-channels/src/bootstrap.rs`
  - `crates/hone-web-api/src/{lib.rs,runtime.rs}`
  - `bins/hone-desktop/src/sidecar.rs`
  - `bins/hone-desktop/src/sidecar/{processes,runtime_env,settings}.rs`
  - `bins/hone-feishu/src/{handler.rs,scheduler.rs,outbound.rs}`
  - `bins/hone-telegram/src/{handler.rs,scheduler.rs}`
  - `packages/app/src/pages/{settings.tsx,settings-model.ts,settings-model.test.ts}`
- related_docs:
  - `docs/repo-map.md`
  - `docs/decisions.md`
  - `docs/archive/plans/large-files-refactor.md`
- related_prs:
  - `N/A`

## Summary

This round closed the active large-file architecture-tightening task by consolidating shared runtime bootstrap / override logic, moving desktop sidecar helpers into concern-oriented modules, and extracting channel scheduler / outbound helpers out of oversized handler files. The frontend settings page also moved pure state mutations into a test-covered model helper.

## What Changed

- Added `hone-core` config helpers so runtime data-root / skills-dir overrides and runtime directory creation have one source of truth.
- Added `crates/hone-channels/src/bootstrap.rs` and switched Discord / Feishu / Telegram / iMessage binaries onto the shared startup path.
- Split desktop sidecar helpers into `sidecar/{processes,runtime_env,settings}.rs`.
- Split Telegram scheduler into `bins/hone-telegram/src/scheduler.rs`.
- Split Feishu scheduler and outbound delivery helpers into `bins/hone-feishu/src/{scheduler.rs,outbound.rs}`.
- Extracted frontend settings defaults and API-key mutation helpers into `packages/app/src/pages/settings-model.ts` and added `settings-model.test.ts`.
- Updated `docs/repo-map.md` and recorded the shared-runtime / bootstrap rule in `docs/decisions.md`.

## Verification

- `cargo check --workspace --all-targets --exclude hone-desktop` passed.
- `cargo test --workspace --all-targets --exclude hone-desktop` passed.
- `cargo test -p hone-telegram` passed after removing stale imports left by the scheduler extraction.
- `bun install` restored missing workspace test dependencies locally, after which `bun run test:web` passed.
- `bash tests/regression/run_ci.sh` passed; its finance automation contract sub-report still shows the existing `success=5 / review=1 / fail=3` status tracked by the separate finance regression task.

## Risks / Follow-ups

- The remaining large files are now boundary-cleaner, but not all of them are small. Future extractions should only proceed when they create a stable ownership seam such as a distinct command surface or a separate protocol adapter.
- The finance automation contract warnings are unrelated to this architecture round and should continue under `docs/current-plans/finance-automation-contract-loop.md`.

## Next Entry Point

- Start with `docs/repo-map.md`, then inspect `crates/hone-channels/src/bootstrap.rs`, `crates/hone-core/src/config.rs`, and the per-channel sibling modules if another architecture-tightening round is needed.
