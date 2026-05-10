# Source CLI Start And Launch Retirement Handoff

- title: Source CLI Start And Launch Retirement
- status: done
- created_at: 2026-05-10
- updated_at: 2026-05-10
- owner: Codex
- related_files:
  - `bins/hone-cli/src/start.rs`
  - `bins/hone-cli/src/main.rs`
  - `bins/hone-cli/src/onboard.rs`
  - `launch.sh`
  - `scripts/restart_hone.sh`
  - `crates/hone-tools/src/restart_hone.rs`
  - `tests/regression/ci/test_source_cli_start_contract.sh`
  - `README.md`
  - `README_EN.md`
  - `README_ZH.md`
  - `docs/wiki.md`
  - `docs/runbooks/hone-cli-install-and-start.md`
  - `docs/runbooks/source-web-startup.md`
  - `docs/runbooks/desktop-dev-runtime.md`
  - `docs/runbooks/desktop-release-app-runtime.md`
  - `packages/app/src/lib/public-content.ts`
- related_docs:
  - `docs/archive/plans/source-cli-start-retire-launch.md`
  - `docs/current-plan.md`
  - `docs/archive/index.md`
- related_prs:

## Summary

Source checkout startup now goes through the local CLI build path:

```bash
cargo run -p hone-cli -- start --build
```

Installed users still use the packaged CLI:

```bash
hone-cli start
```

README files only show the current paths. Historical launcher context is kept out of first-time reader docs.

## What Changed

- Added `hone-cli start --build` and `--source-root` for source checkout build-and-start.
- `hone-cli start` now writes `data/runtime/current.pid` after startup.
- `launch.sh` is a compatibility shim that points users to the CLI path.
- `restart_hone` now restarts through the CLI source path.
- Active docs/runbooks/public install content now show CLI-based startup and explicit Tauri frontend/desktop commands.
- Replaced the old launch regression with `test_source_cli_start_contract.sh`.

## Verification

- `cargo test -p hone-cli start`
- `bash tests/regression/ci/test_source_cli_start_contract.sh`
- `bash tests/regression/ci/test_install_hone_cli_path_resolution.sh`
- CLI channel configuration smoke against a temporary config
- `cargo run -q -p hone-cli -- start --help`
- Source startup smoke with `cargo run -q -p hone-cli -- --config <tmp>/config.yaml start --build` and `/api/meta` on port `19077`
- `cargo test -p hone-cli`
- `cargo check --workspace --all-targets --exclude hone-desktop`
- `bun run typecheck:web`
- `bun run test:web`
- `bash tests/regression/run_ci.sh`

## Risks / Follow-ups

- `hone-cli start --build` is the source runtime path. Plain `hone-cli start` still assumes runtime binaries are already available beside the CLI or in the installed bundle.
- Desktop development is intentionally split: CLI backend in one terminal, Vite/Tauri commands in separate terminals.
- Old release notes and archived handoffs may still mention historical launcher commands; they were left as history.

## Next Entry Point

Use `docs/runbooks/hone-cli-install-and-start.md` for install/source startup, and `docs/runbooks/desktop-dev-runtime.md` for desktop development lanes.
