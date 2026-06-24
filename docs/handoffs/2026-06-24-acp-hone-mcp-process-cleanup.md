# ACP `hone-mcp` Process Cleanup

- title: ACP `hone-mcp` Process Cleanup
- status: done
- created_at: 2026-06-24
- updated_at: 2026-06-24
- owner: Codex
- related_files:
  - `crates/hone-channels/src/runners/acp_common/process.rs`
  - `crates/hone-channels/src/runners/acp_common/mod.rs`
  - `crates/hone-channels/src/runners/codex_acp.rs`
  - `crates/hone-channels/src/runners/opencode_acp.rs`
  - `docs/current-plans/acp-runtime-refactor.md`
  - `docs/repo-map.md`
- related_docs:
  - `docs/current-plan.md`
  - `docs/current-plans/acp-runtime-refactor.md`
  - `docs/repo-map.md`
- related_prs:
  - none

## Summary

Fixed ACP runner lifecycle cleanup so `codex_acp` and `opencode_acp` no longer leave stdio `hone-mcp` grandchildren behind when initialization, `session/new`, model selection, prompt handling, timeout, or other error paths return early.

## What Changed

- Added `AcpChildGuard` and process-group setup in `acp_common/process.rs`.
- `codex_acp` and `opencode_acp` now spawn ACP CLI children in a dedicated process group, shut down stdin, terminate the group, and wait for cleanup before returning.
- Added a Unix regression test that spawns a child plus grandchild process and verifies guard cleanup kills the grandchild.

## Verification

- `rustfmt --edition 2024 --config skip_children=true --check crates/hone-channels/src/runners/acp_common/process.rs crates/hone-channels/src/runners/acp_common/mod.rs crates/hone-channels/src/runners/codex_acp.rs crates/hone-channels/src/runners/opencode_acp.rs`
- `cargo test -p hone-channels acp_child_guard_terminates_grandchild_process_group -- --nocapture`
- `cargo test -p hone-channels codex_acp -- --nocapture`
- `cargo check -p hone-channels --tests`
- `pgrep -fl 'hone-mcp' || true` returned no `hone-mcp` processes after cleanup and tests.

## Risks / Follow-ups

- `cargo fmt --check` for the whole repository still reports pre-existing formatting diffs in `crates/hone-channels/src/runtime.rs` and `crates/hone-core/src/cloud_runtime.rs`; those files were not changed here.
- One unrelated Clash Verge `<defunct>` process remains on the machine. Its parent `verge-mihomo` rejected `kill -9` with `operation not permitted`, so it is outside the current Hone cleanup scope.
- On Unix, ACP cleanup relies on spawning the ACP CLI in a separate process group. If a future runner deliberately needs to share the parent process group, it must provide another way to clean MCP grandchildren.

## Next Entry Point

Start with `crates/hone-channels/src/runners/acp_common/process.rs` for lifecycle behavior, then inspect the `run_codex_acp` / `run_opencode_acp` cleanup blocks if a future ACP runner shows process leaks.
