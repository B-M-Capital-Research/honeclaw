# Plan Template

- title: CLI 首装 Onboarding 与安装向导
- status: archived
- created_at: 2026-04-12
- updated_at: 2026-04-12
- owner: codex
- related_files:
  - bins/hone-cli/src/main.rs
  - bins/hone-cli/src/common.rs
  - scripts/install_hone_cli.sh
  - tests/regression/manual/test_install_bundle_smoke.sh
  - docs/runbooks/hone-cli-install-and-start.md
  - docs/repo-map.md
- related_docs:
  - docs/current-plan.md
  - docs/handoffs/2026-04-12-cli-onboarding-install-wizard.md

## Goal

让全新用户在安装 `hone-cli` 后可以通过交互式 TUI 向导完成 runner 与渠道的首装配置，尤其是能直接切到 `opencode_acp` 并复用本机 `opencode` 默认配置，同时把安装脚本默认链路切到 canonical config 语义。

## Scope

- 新增 `hone-cli onboard` / `setup` 交互式向导
- 在向导中支持 runner 选择、`opencode_acp` 复用本机配置提示、渠道开启确认、必填字段收集和权限/前置提示
- 更新安装脚本，在交互终端下默认询问是否立即运行 onboarding
- 补一条 fresh-install / install-layout smoke 的手工回归脚本

## Validation

- `cargo check -p hone-cli`
- `cargo test -p hone-cli`
- `cargo check --workspace --all-targets --exclude hone-desktop`
- `HONE_SKIP_BUNDLED_RESOURCE_CHECK=1 cargo check -p hone-desktop`
- `bash tests/regression/manual/test_install_bundle_smoke.sh`
- 手工 PTY 回归 `target/debug/hone-cli --config <tmp>/config.yaml onboard`

## Documentation Sync

- 更新 `docs/runbooks/hone-cli-install-and-start.md`
- 更新 `docs/repo-map.md`
- 完成后补 handoff，并归档本计划

## Risks / Open Questions

- 首装脚本是 `curl | bash` 路径，必须保证非交互环境下不会卡死
- 各渠道平台侧权限/接入流程较长；本轮只提供本地配置必填项与关键前置提示，不在 installer 内自动完成平台授权
