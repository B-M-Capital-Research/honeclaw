- title: Web direct sandbox isolation hotfix
- status: done
- created_at: 2026-05-14
- updated_at: 2026-05-14 04:24 CST
- owner: Codex
- related_files:
  - crates/hone-channels/src/sandbox.rs
  - crates/hone-channels/src/execution.rs
  - bins/hone-desktop/src/sidecar.rs
  - bins/hone-desktop/src/sidecar/processes.rs
  - bins/hone-desktop/src/sidecar/runtime_env.rs
  - docs/bugs/web_direct_cross_session_sandbox_data_exposure.md
  - docs/bugs/README.md
- related_docs:
  - docs/current-plan.md
  - docs/current-plans/active-bug-burn-down-2026-04-28.md
  - docs/invariants.md
  - docs/repo-map.md
- related_prs:

## Summary

Closed the active P1 Web direct cross-session data exposure at the code level. The repo was still shipping actor sandboxes under repo `data/agent-sandboxes`, so Codex ACP could read sibling sandboxes and portfolio files despite the bug doc already claiming a fix.

## What Changed

- `hone-channels` sandbox root now ignores repo-internal `HONE_AGENT_SANDBOX_DIR` values and no longer derives from `HONE_DATA_DIR`; it falls back to a repo-external temp sandbox root instead.
- Actor sandbox initialization now deletes legacy `portfolio_*.json`, `portfolio/`, and `portfolios/` entries before exposing the directory to native-file runners.
- Desktop sidecar runtime now carries an explicit `sandbox_dir` and exports that path to child processes instead of hardcoding repo `data/agent-sandboxes`.
- Added regressions covering repo-internal sandbox override rejection, legacy portfolio cleanup, and desktop repo-checkout sandbox relocation.

## Verification

- `cargo test -p hone-channels sandbox --lib -- --nocapture`
- `cargo test -p hone-channels prepare_ignores_repo_internal_sandbox_override --lib -- --nocapture`
- `HONE_SKIP_BUNDLED_RESOURCE_CHECK=1 cargo test -p hone-desktop runtime_env -- --nocapture`
- `cargo check -p hone-channels --tests`
- `HONE_SKIP_BUNDLED_RESOURCE_CHECK=1 cargo check -p hone-desktop`

## Risks / Follow-ups

- Live runtime was not restarted in this automation, so the original Web direct prompt still needs post-deploy verification before the bug can be upgraded from `Fixed` to `Closed`.
- Feishu `company_profile_absolute_path_leak` remains active and is likely reduced by the repo-external sandbox root, but it still needs a dedicated user-visible output recheck because that bug also concerns path rendering, not only file reachability.

## Next Entry Point

Start with `crates/hone-channels/src/sandbox.rs` and `bins/hone-desktop/src/sidecar/runtime_env.rs`, then re-run the original Web direct reproduction prompt after the next normal deploy/restart.
