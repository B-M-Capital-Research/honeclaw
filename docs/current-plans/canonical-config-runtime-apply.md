- title: Canonical Config And Runtime Apply Unification
- status: in_progress
- created_at: 2026-04-12
- updated_at: 2026-04-12
- owner: Codex
- related_files:
  - crates/hone-core/src/config.rs
  - bins/hone-cli/src/common.rs
  - bins/hone-cli/src/main.rs
  - bins/hone-cli/src/start.rs
  - bins/hone-desktop/src/sidecar.rs
  - bins/hone-desktop/src/sidecar/runtime_env.rs
  - bins/hone-desktop/src/sidecar/processes.rs
- related_docs:
  - docs/current-plan.md
  - docs/invariants.md
  - docs/repo-map.md
  - docs/decisions.md

## Goal

把 Hone 从 legacy runtime seed/overlay 配置模型切换到单一 canonical config + runtime apply plane，统一 CLI / desktop 的配置真相源、effective-config 生成、legacy 迁移和运行时生效语义。

## Scope

- 在 `hone-core` 中实现 canonical config 读写、effective-config 生成、legacy runtime config 迁移与 apply 分类
- 让 `hone-cli` 以 canonical config 为默认配置入口，并在 `start` 时只给子进程注入 generated effective config
- 让 `opencode_acp` 默认继承用户本机 OpenCode 配置，而不是通过 Hone runtime overlay 隐式替换全局 OpenCode config root
- 让 desktop bundled/runtime 路径切到 canonical config，并按 live/component/full 三类应用配置
- 更新安装脚本、文档和相关手工/命令验证

## Current State

- 已完成：
  - canonical config path 解析、effective-config 生成与 legacy runtime 文件迁移入口
  - `hone-cli` 的 `config / models / channels / status / doctor / start / onboard`
  - `scripts/install_hone_cli.sh`、CLI install-layout smoke、`hone-cli start`
  - `opencode_acp` 默认继承本机 OpenCode config，不再由 Hone 隐式强推 OpenRouter 默认路由
- 仍待完成：
  - desktop bundled 模式下真正的 `live_apply / component_restart / full_restart` 行为收口
  - desktop settings 与 canonical config / apply result contract 的最后一轮对齐验证
  - release 资产级别的真实安装 E2E 与剩余文档收尾

## Validation

- `cargo test -p hone-core`
- `cargo test -p hone-cli`
- `cargo check --workspace --all-targets --exclude hone-desktop`
- 手工验证：
  - legacy `config_runtime.yaml + .overrides.yaml` 自动迁移到 canonical `config.yaml`
  - `hone-cli config file/get/set/unset/validate`
  - `hone-cli status` / `doctor` 同时展示 canonical config 与 effective config
  - `hone-cli start` 生成 `data/runtime/effective-config.yaml`
  - desktop bundled 模式下 agent/provider 配置保存后自动应用；channel 配置保存后只重启受影响 listener；full restart 类变更触发确认

## Documentation Sync

- 更新 `docs/current-plan.md`
- 更新 `docs/invariants.md`
- 更新 `docs/repo-map.md`
- 更新 `docs/decisions.md`
- 更新安装 / 启动 runbook
- 完成后补 `docs/handoffs/2026-04-12-canonical-config-runtime-apply.md` 并归档计划页

## Risks / Open Questions

- desktop bundled backend 目前是内嵌 web server + 外部 channel sidecar；`live_apply` 需要在不破坏当前并发模型的前提下给出可接受的自动应用策略
- direct cut 会触发路径与环境变量语义变化，需要同时覆盖 install wrapper、CLI、desktop sidecar 与诊断脚本
- 当前密钥仍留在 YAML，本轮只做路径与运行时生效收口，不引入系统 keychain
