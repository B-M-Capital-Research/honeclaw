# Handoff

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
  - tests/regression/ci/test_install_hone_cli_path_resolution.sh
- related_docs:
  - docs/archive/plans/hone-cli-config-mvp.md
  - docs/repo-map.md
  - docs/runbooks/hone-cli-install-and-start.md
- related_prs:
  - N/A

## Summary

`hone-cli` 已从单一 REPL 扩展为带 `config / configure / models / channels / status / doctor / start` 的管理 CLI；runtime overlay 写入逻辑已收口到 `hone-core`，desktop settings 和 CLI 共用同一套最小 patch 写入服务。发布侧新增 GitHub release 资源布局与 `scripts/install_hone_cli.sh`，安装后可直接使用 `hone-cli start`，不再要求 `./launch.sh` 或 `hone-desktop`。

## What Changed

- `hone-core` 新增共享 runtime-config edit service，负责 dot-path get/set/unset、overlay diff、原子写入、敏感字段脱敏，以及首次 runtime config seed
- `hone-desktop` sidecar 改为调用共享配置服务，消除 desktop / CLI 双写逻辑
- `hone-cli` 改为 clap 子命令结构，保留无参默认进入 chat REPL，并新增 `start`
- release workflow 改为产出安装所需的多平台 tarball 与安装脚本；install script 会写 wrapper 并注入 `HONE_HOME` / `HONE_CONFIG_PATH` / `HONE_BASE_CONFIG_PATH`
- 安装脚本同日继续收口 PATH 行为：优先把 `hone-cli` wrapper 写入当前 `PATH` 中可写的用户态 bin 目录，只有找不到合适目录时才回退到 `~/.local/bin`
- 冷缓存走查过程中补了一个首装缺口：当 `--config` 或 `HONE_CONFIG_PATH` 指向尚未生成的 `data/runtime/config_runtime.yaml` 时，CLI 现在会在写操作自动 seed runtime，在读操作回退到 base `config.yaml`

## Verification

- 缓存清理：删除仓库 `target/` 与 `~/Library/Caches/hone-financial/target` 后重新冷启动验证
- `cargo check --workspace --all-targets --exclude hone-desktop` 通过
- `cargo test -p hone-core` 通过，37 tests passed
- `cargo test -p hone-cli` 通过，4 tests passed
- CLI 命令走查通过：
  - `config file / validate / get / set / unset`
  - `models status / set`
  - `channels list / set / enable / disable`
  - `status / doctor`
  - `hone-cli start --help`
  - 无参 `hone-cli` 进入 REPL，输入 `quit` 可退出
- CI-safe 安装回归新增并通过：
  - `bash tests/regression/ci/test_install_hone_cli_path_resolution.sh`
  - 覆盖“PATH 中已有可写用户 bin 目录时直接安装到该目录”和“否则回退到 `~/.local/bin` 并打印 PATH 提示”
- 安装态 env 模式验证通过：
  - 仅设置 `HONE_HOME`、`HONE_BASE_CONFIG_PATH`、`HONE_CONFIG_PATH`、`HONE_DATA_DIR`、`HONE_SKILLS_DIR` 即可完成首次 `config set`，并正确生成 `data/runtime/config_runtime.yaml`
- 非阻塞失败：
  - `cargo check -p hone-desktop` 仍因 sidecar 资源 `binaries/hone-imessage-aarch64-apple-darwin` 缺失而失败，这不影响 `hone-cli` 安装路径

## Risks / Follow-ups

- `hone-cli start` 在源码目录直接 `cargo run` 时，`status/doctor` 会对 runtime bundle 二进制给出 warn；这是因为未处于安装布局，不是逻辑故障
- 当前密钥仍写入 `.overrides.yaml`；如需更接近 OpenClaw，下一阶段应补 secret ref/provider、profile、plugins、cron 等能力
- 桌面构建资源准备与 CLI 安装链路已解耦，但桌面 release 流程仍需单独维护 sidecar 产物

## Next Entry Point

- 安装/启动手册：`docs/runbooks/hone-cli-install-and-start.md`
- 历史计划与验证矩阵：`docs/archive/plans/hone-cli-config-mvp.md`
