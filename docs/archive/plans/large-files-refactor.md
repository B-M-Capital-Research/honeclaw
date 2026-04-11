# 大文件物理拆分重构

- title: 大文件物理拆分重构
- status: done
- created_at: 2026-03-22
- updated_at: 2026-04-11
- owner: shared
- related_files:
  - `crates/hone-core/src/config.rs`
  - `crates/hone-core/src/config/server.rs`
  - `crates/hone-channels/src/bootstrap.rs`
  - `crates/hone-web-api/src/{lib.rs,runtime.rs}`
  - `bins/hone-desktop/src/sidecar.rs`
  - `bins/hone-desktop/src/sidecar/{processes,runtime_env,settings}.rs`
  - `bins/hone-feishu/src/{handler.rs,scheduler.rs,outbound.rs}`
  - `bins/hone-telegram/src/{handler.rs,scheduler.rs}`
  - `bins/hone-discord/src/main.rs`
  - `bins/hone-imessage/src/main.rs`
  - `packages/app/src/pages/{settings.tsx,settings-model.ts,settings-model.test.ts}`
- related_docs:
  - `docs/repo-map.md`
  - `docs/decisions.md`
  - `docs/handoffs/2026-04-11-architecture-tightening-round1.md`

## Goal

Reduce architecture drift by tightening oversized runtime, desktop, channel, and frontend hotspots along stable module boundaries, while keeping behavior unchanged and tests green.

## Outcome

- Runtime path / storage override logic is now centralized in `hone-core::HoneConfig` and reused by both channel runtime and web API paths.
- Channel entry binaries now share one startup bootstrap in `hone-channels::bootstrap`, removing repeated logging / enabled-check / process-lock / heartbeat setup.
- Desktop sidecar helper concerns are physically split into `processes`, `runtime_env`, and `settings` modules instead of one catch-all helper surface.
- Telegram and Feishu scheduler logic is extracted out of the main handlers; Feishu outbound delivery helpers now also live in their own module.
- Frontend settings defaults / merge / API-key list mutations moved into `settings-model.ts` with dedicated tests.

## Validation

- `cargo check --workspace --all-targets --exclude hone-desktop`
- `cargo test --workspace --all-targets --exclude hone-desktop`
- `cargo test -p hone-telegram`
- `bun run test:web`
- `bash tests/regression/run_ci.sh`

## Risks / Open Questions

- `bins/hone-desktop/src/sidecar.rs`, `bins/hone-feishu/src/handler.rs`, and `packages/app/src/pages/settings.tsx` are materially smaller in responsibility but still large; future splits should follow ownership seams, not line-count targets.
- `bash tests/regression/run_ci.sh` exited successfully during this refactor round. The separate finance automation contract drift referenced at the time has since been closed and archived under `docs/archive/plans/finance-automation-contract-loop.md`.
