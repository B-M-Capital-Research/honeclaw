# Plan Template

- title: Desktop Rust Check 与 IDE 语法检查解耦
- status: archived
- created_at: 2026-04-12
- updated_at: 2026-04-12
- owner: codex
- related_files:
  - bins/hone-desktop/build.rs
  - bins/hone-desktop/Cargo.toml
  - bins/hone-desktop/src/sidecar/runtime_env.rs
  - bins/hone-desktop/src/sidecar/processes.rs
  - bins/hone-desktop/src/sidecar/settings.rs
  - .vscode/settings.json
  - CONTRIBUTING.md
  - docs/repo-map.md
  - docs/invariants.md
- related_docs:
  - AGENTS.md
  - docs/current-plan.md
  - docs/handoffs/2026-04-12-desktop-rust-check-workflow.md

## Goal

让默认开发与 IDE 的 Rust 语法检查不再被 `hone-desktop` 的 Tauri 打包资源缺失阻塞，同时保留桌面打包 / release 路径对 sidecar 资源的严格校验。

## Scope

- 为 `hone-desktop` build script 增加显式的开发检查豁免开关
- 给仓库内 VSCode rust-analyzer 提供默认开发配置
- 同步贡献与代码库地图文档中的检查建议
- 修复因此暴露出来的 desktop 真实编译错误

## Validation

- `cargo check --workspace --all-targets --exclude hone-desktop`
- `HONE_SKIP_BUNDLED_RESOURCE_CHECK=1 cargo check -p hone-desktop`
- `HONE_SKIP_BUNDLED_RESOURCE_CHECK=1 cargo check --workspace --all-targets`

## Documentation Sync

- 更新 `docs/invariants.md` 中默认检查与桌面专项检查说明
- 更新 `docs/repo-map.md` 中 desktop check 约定
- 更新 `CONTRIBUTING.md` 中推荐本地检查命令

## Risks / Open Questions

- `rust-analyzer` 的仓库内配置主要覆盖 VSCode；其他 IDE 仍需用户自行设置等效 env
- `HONE_SKIP_BUNDLED_RESOURCE_CHECK` 只应影响开发检查，不应被 release 打包流程误用
