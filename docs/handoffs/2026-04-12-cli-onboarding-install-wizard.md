# Handoff Template

- title: CLI 首装 Onboarding 与安装向导
- status: done
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
  - docs/archive/plans/cli-onboarding-install-wizard.md
  - docs/archive/index.md
- related_prs:
  - N/A

## Summary

`hone-cli` 现在有首装 TUI 向导 `hone-cli onboard`，并提供 `setup` 别名。全新安装用户可以在首次安装后直接选择 runner，并按渠道逐个决定是否启用、填写本地必填字段、查看关键权限/前置条件。`opencode_acp` 在首装时默认只切 runner，不强迫用户把 provider / model / key 再写一遍到 Hone。

## What Changed

- `bins/hone-cli/src/main.rs` 新增 `onboard` 子命令，并保留原有无参进入 REPL 的行为
- 向导会探测本机 `codex` / `codex-acp` / `opencode`；如果选择 `opencode_acp`，会提示用户先在本机 `opencode` 中完成 `/connect` 与默认模型配置，Hone 侧覆盖留给后续 `hone-cli models set`
- 向导支持 iMessage、Feishu、Telegram、Discord 的 enable 流程，并按渠道输出最关键的本地前置提示；启用时会要求填写必填字段
- 安装脚本 `scripts/install_hone_cli.sh` 改为 canonical config 语义，wrapper 默认导出 `HONE_USER_CONFIG_PATH`，并在交互终端下询问是否立即运行 `hone-cli onboard`
- 新增手工 smoke `tests/regression/manual/test_install_bundle_smoke.sh`，验证安装布局、`hone-cli doctor`、`hone-cli config file` 与 `hone-cli start`

## Verification

- `cargo check -p hone-cli`
- `cargo test -p hone-cli`
- `cargo check --workspace --all-targets --exclude hone-desktop`
- `HONE_SKIP_BUNDLED_RESOURCE_CHECK=1 cargo check -p hone-desktop`
- `bash tests/regression/manual/test_install_bundle_smoke.sh`
- 手工 PTY 回归：
  - 以临时 `config.yaml` + `soul.md` 运行 `target/debug/hone-cli --config <tmp>/config.yaml onboard`
  - 实测完成 `OpenCode ACP` 选择、Telegram token 配置、`doctor` 回显与退出

## Risks / Follow-ups

- 交互式向导仍然是手工验证路径，当前没有自动化 TTY 回归
- 渠道平台侧的远端权限开通仍需用户自行完成；向导目前只覆盖本地必填字段与关键提醒
- `doctor` 在纯临时目录下若未提供 `skills/` 会给出 `skills-dir` warn；正式安装 wrapper 会把它指向 bundle 内 skills 目录

## Next Entry Point

- `bins/hone-cli/src/main.rs`
- `scripts/install_hone_cli.sh`
- `docs/runbooks/hone-cli-install-and-start.md`
