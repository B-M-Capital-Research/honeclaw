# Source CLI Start And Launch Retirement

- title: Source CLI Start And Launch Retirement
- status: done
- created_at: 2026-05-10
- updated_at: 2026-05-10
- owner: Codex
- related_files:
  - `launch.sh`
  - `bins/hone-cli/src/start.rs`
  - `bins/hone-cli/src/main.rs`
  - `scripts/install_hone_cli.sh`
  - `scripts/restart_hone.sh`
  - `crates/hone-tools/src/restart_hone.rs`
  - `tests/regression/ci/*.sh`
  - `README.md`
  - `README_EN.md`
  - `README_ZH.md`
  - `docs/wiki.md`
  - `docs/runbooks/hone-cli-install-and-start.md`
  - `docs/runbooks/source-web-startup.md`
  - `docs/runbooks/desktop-dev-runtime.md`
  - `docs/runbooks/desktop-release-app-runtime.md`
  - `docs/repo-map.md`
- related_docs:
  - `docs/current-plan.md`
  - `docs/handoffs/source-cli-start-retire-launch-2026-05-10.md`
  - `docs/archive/index.md`

## Goal

Retire `launch.sh` as the source-checkout and local-install startup path. A developer in the project directory should build and start through the local `hone-cli`, while normal users install the same CLI through Homebrew or the release installer and run `hone-cli start`.

## Scope

- Keep the public install story centered on `hone-cli`: `curl | bash`, Homebrew, then `hone-cli onboard` / `hone-cli start`.
- Make source-checkout runtime startup work through a local CLI build, not a separate launcher contract.
- Remove or rewrite active docs, tests, and skills that still tell users to start with `launch.sh`.
- Do not edit old release notes or archived historical handoffs except where an active test/runbook depends on them.
- Validate the channel configuration changes from the previous plan through real CLI commands.

## Validation

- CLI parser and start-path tests for local source build behavior.
- Installer path regression.
- CLI channel configuration smoke using a temporary config.
- `cargo test -p hone-cli`.
- `cargo check --workspace --all-targets --exclude hone-desktop`.
- `bun run typecheck:web` / `bun run test:web` if public content changes.
- `git diff --check`.

## Progress

### 2026-05-10

- Added `hone-cli start --build --source-root <DIR>` so source checkouts can build `hone-cli`, `hone-console-page`, `hone-mcp`, and channel binaries before starting from the local target dir.
- `hone-cli start` now writes `data/runtime/current.pid` after successful startup, so restart tooling can find the active CLI supervisor.
- Replaced the previous launcher body with a compatibility shim that points to `cargo run -p hone-cli -- start --build` for source users and `hone-cli start` for installed users.
- Updated `restart_hone` script/tool and `hone_admin` skill to restart through the CLI source startup path.
- Updated README files to show only the current startup paths; no historical launch-path note is shown to first-time readers.
- Updated source/Web/desktop runbooks, wiki, repo map, config comments, public install content, and CI-safe regression contracts.
- Verified the previous channel configuration work through real CLI commands against a temporary config:
  - Telegram / Discord `allow_from` and `chat_scope`
  - Feishu email/mobile/open_id allowlists and `chat_scope`
  - iMessage `target_handle`
  - `channels list --json`
  - `channels targets --json`
  - effective config generation
- Verification run:
  - `cargo test -p hone-cli start`
  - `bash tests/regression/ci/test_source_cli_start_contract.sh`
  - `bash tests/regression/ci/test_install_hone_cli_path_resolution.sh`
  - CLI channel configuration smoke with `cargo run -q -p hone-cli -- --config <tmp>/config.yaml ...`
  - `cargo run -q -p hone-cli -- start --help`
  - Source startup smoke: `cargo run -q -p hone-cli -- --config <tmp>/config.yaml start --build`, then `curl http://127.0.0.1:19077/api/meta`
  - `rustfmt --edition 2024 bins/hone-cli/src/main.rs bins/hone-cli/src/start.rs bins/hone-cli/src/onboard.rs crates/hone-tools/src/restart_hone.rs`
  - `cargo test -p hone-cli`
  - `cargo check --workspace --all-targets --exclude hone-desktop`
  - `bun run typecheck:web`
  - `bun run test:web`
  - `bash tests/regression/run_ci.sh`

## Documentation Sync

- Update `docs/current-plan.md` while active.
- Update `docs/repo-map.md`, `docs/wiki.md`, and the relevant runbook with the new startup contract.
- Add a handoff when the startup workflow change is complete.
- Archive this plan and update `docs/archive/index.md` when closed.

## Risks / Open Questions

- Desktop dev and release desktop flows now have explicit Tauri/build commands in runbooks; they are no longer hidden behind a source launcher.
- `restart_hone` now targets the CLI source startup path and `hone-cli start` writes `current.pid`.
- Source startup should use `--build`; plain `hone-cli start` still expects peer runtime binaries to already exist beside the CLI or under the installed bundle.
