- title: Hone CLI Config MVP And Installable Start Flow
- status: done
- created_at: 2026-04-12
- updated_at: 2026-04-12
- owner: Codex
- related_files:
  - bins/hone-cli/src/main.rs
  - bins/hone-cli/src/common.rs
  - bins/hone-cli/src/repl.rs
  - bins/hone-cli/src/start.rs
  - crates/hone-core/src/config.rs
  - bins/hone-desktop/src/sidecar.rs
  - bins/hone-desktop/src/sidecar/settings.rs
  - .github/workflows/release.yml
  - scripts/install_hone_cli.sh
- related_docs:
  - docs/repo-map.md
  - docs/runbooks/hone-cli-install-and-start.md
  - docs/handoffs/2026-04-12-hone-cli-config-mvp.md

## Goal

把 `hone-cli` 从单一 REPL 扩展为可配置、可诊断、可直接启动的本地管理 CLI，对齐本机 `openclaw` 的高频 CLI 能力，并补齐 macOS 通过 GitHub release 资源 `curl|bash` 一键安装后的 `hone-cli start` 工作流。

## Scope

- 抽出共享 runtime config edit service，统一 desktop / CLI 对 `config_runtime.overrides.yaml` 的写入
- 为 `hone-cli` 增加 `chat`、`config`、`configure`、`models`、`channels`、`status`、`doctor`、`start` 子命令
- 默认配置解析优先 `data/runtime/config_runtime.yaml`，不存在时回退 `config.yaml`
- 新增 macOS 安装脚本与 release 资源打包，支持安装后直接使用 `hone-cli`
- 同步更新 repo map、安装/启动 runbook，并验证 `HONE_CONFIG_PATH` / `HONE_BASE_CONFIG_PATH` 首装链路

## Validation

- `cargo test -p hone-core`
- `cargo test -p hone-cli`
- `cargo check --workspace --all-targets --exclude hone-desktop`
- 手工验证：
  - `cargo run -q -p hone-cli -- --config /tmp/.../data/runtime/config_runtime.yaml config file`
  - `cargo run -q -p hone-cli -- --config /tmp/.../data/runtime/config_runtime.yaml config validate`
  - `cargo run -q -p hone-cli -- --config /tmp/.../data/runtime/config_runtime.yaml config get agent.runner`
  - `cargo run -q -p hone-cli -- --config /tmp/.../data/runtime/config_runtime.yaml config set agent.runner opencode_acp`
  - `cargo run -q -p hone-cli -- --config /tmp/.../data/runtime/config_runtime.yaml models status --json`
  - `cargo run -q -p hone-cli -- --config /tmp/.../data/runtime/config_runtime.yaml models set --runner opencode_acp --model openrouter/openai/gpt-5.4 --variant medium`
  - `cargo run -q -p hone-cli -- --config /tmp/.../data/runtime/config_runtime.yaml channels list --json`
  - `cargo run -q -p hone-cli -- --config /tmp/.../data/runtime/config_runtime.yaml channels set telegram --enabled true --bot-token xxx --chat-scope all`
  - `cargo run -q -p hone-cli -- --config /tmp/.../data/runtime/config_runtime.yaml channels enable discord`
  - `cargo run -q -p hone-cli -- --config /tmp/.../data/runtime/config_runtime.yaml channels disable discord`
  - `cargo run -q -p hone-cli -- --config /tmp/.../data/runtime/config_runtime.yaml status --json`
  - `cargo run -q -p hone-cli -- --config /tmp/.../data/runtime/config_runtime.yaml doctor --json`
  - `printf 'quit\n' | cargo run -q -p hone-cli -- --config /tmp/.../data/runtime/config_runtime.yaml`
  - `HONE_CONFIG_PATH=/tmp/.../data/runtime/config_runtime.yaml HONE_BASE_CONFIG_PATH=/tmp/.../config.yaml cargo run -q -p hone-cli -- config set agent.runner opencode_acp`
  - `HONE_CONFIG_PATH=/tmp/.../data/runtime/config_runtime.yaml HONE_BASE_CONFIG_PATH=/tmp/.../config.yaml cargo run -q -p hone-cli -- doctor --json`
- 非阻塞补充检查：
  - `cargo check -p hone-desktop` 失败，原因是桌面打包 sidecar 资源 `binaries/hone-imessage-aarch64-apple-darwin` 缺失；CLI 安装链路不依赖该资源

## Documentation Sync

- 已更新 `docs/repo-map.md`
- 已新增 `docs/runbooks/hone-cli-install-and-start.md`
- 已新增 `docs/handoffs/2026-04-12-hone-cli-config-mvp.md`
- 已从活跃索引移出，并归档本计划页

## Risks / Open Questions

- `hone-cli start` 在源码仓库直接 `cargo run` 时会对 runtime 二进制给出 warn；完整 green path 依赖 release bundle 或安装脚本布局
- `hone-desktop` 构建仍要求桌面 sidecar 资源齐全，这与 CLI 安装路径已解耦，但桌面 release 流程仍需单独维护
- Phase 1 仍采用明文 `.overrides.yaml` 保存密钥；secret provider / profile / plugin / cron 等 OpenClaw 对齐项留待 phase 2
