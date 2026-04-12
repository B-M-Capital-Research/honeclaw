# Handoff Template

- title: Desktop Rust Check 与 IDE 语法检查解耦
- status: done
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
  - docs/archive/plans/desktop-rust-check-workflow.md
  - docs/archive/index.md
- related_prs:
  - N/A

## Summary

把 `hone-desktop` 的 Tauri sidecar 资源校验从默认开发语法检查路径里解耦。现在默认 workspace Rust 检查继续排除 `hone-desktop`，而 desktop 自身也可以通过 `HONE_SKIP_BUNDLED_RESOURCE_CHECK=1` 在不准备打包 sidecar 的情况下完成 Rust type-check，适合 IDE 和日常开发。

## What Changed

- `bins/hone-desktop/build.rs` 新增 `HONE_SKIP_BUNDLED_RESOURCE_CHECK` 开关；开启时会给 Tauri 注入补丁配置，关闭 `bundle.active` 并清空 `bundle.externalBin`
- `.vscode/settings.json` 为 rust-analyzer 默认注入 `HONE_SKIP_BUNDLED_RESOURCE_CHECK=1`
- `CONTRIBUTING.md`、`docs/invariants.md`、`docs/repo-map.md` 同步到“默认排除 desktop、桌面专项检查单独跑”的工作流
- 处理了 skip 模式下暴露出的 3 处真实 desktop 编译问题：
  - `runtime_env.rs` 的日志函数可见性
  - `processes.rs` 的 `start_enabled_channels` 可见性
  - `settings.rs` 适配 `apply_config_mutations` 的新返回值

## Verification

- `cargo check --workspace --all-targets --exclude hone-desktop`
- `HONE_SKIP_BUNDLED_RESOURCE_CHECK=1 cargo check -p hone-desktop`
- `HONE_SKIP_BUNDLED_RESOURCE_CHECK=1 cargo check --workspace --all-targets`

## Risks / Follow-ups

- 只有 VSCode 会自动吃到仓库内的 rust-analyzer env；JetBrains / Zed / 其他 IDE 仍需手动配置同名环境变量
- `HONE_SKIP_BUNDLED_RESOURCE_CHECK` 只解决开发时的资源假阳性，不代表桌面打包链路可用；release 仍需走 `bun run tauri:prep:*` 与 `bunx tauri build/dev`

## Next Entry Point

- `bins/hone-desktop/build.rs`
- `docs/repo-map.md`
