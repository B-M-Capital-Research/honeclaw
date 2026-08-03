# Build Storage Optimization

- title: 构建存储与源码部署产物治理
- status: done
- created_at: 2026-08-03
- updated_at: 2026-08-03
- owner: Codex
- related_files:
  - `Cargo.toml`
  - `scripts/deploy_source_runtime.sh`
  - `tests/regression/ci/test_source_runtime_deploy_contract.sh`
- related_docs:
  - `docs/archive/plans/build-storage-optimization.md`
  - `docs/decisions.md#d-2026-08-03-03-bound-local-build-storage-without-sharing-writable-targets`
  - `docs/runbooks/source-web-startup.md`
- related_prs:
  - 直接推送 `main`，无 PR、release 或 tag

## Summary

本机接近 50GB 的 Rust 构建占用已从“多个 worktree 各自保留完整调试与 incremental 图”收敛为独立但有界的 target。直接源码部署、日常 dev/check 和 test 都保留行号级调试信息并关闭 incremental；revision-bound release 在下一次成功部署后只保留 current + previous，且异常目录 fail-safe 保留。

## What Changed

- `[profile.dev]`、`[profile.test]`、`[profile.source-runtime]` 统一采用 `debug=1`、`incremental=false`；源码部署产物隔离到 `target/source-runtime/`。
- release manifest 增加 `cargo_profile=source-runtime`，复用旧 release 时同时校验 profile 与现有 revision/source/hash。
- 成功部署提交 `current` 时维护 `previous`，解除 rollback 后才清理更旧 release；只删除严格 40 位小写 SHA 目录中的四种已知普通文件。
- 外部部署模拟器覆盖 target 路径、manifest、current/previous、正常清理、异常保留以及失败部署零清理。
- 移除 clean 且提交仍可达、无进程占用的 `788a` 与 `8654` worktree；canonical main 与当前 worktree 保留。

## Verification

- `bash scripts/ci/check_fmt_changed.sh`
- `cargo check --workspace --all-targets --exclude hone-desktop --exclude hone-user-app`
- `cargo test --workspace --all-targets --exclude hone-desktop --exclude hone-user-app`
- `bun run test:web`：`347/347`
- `workers/public-community-edge` typecheck + `45/45`
- `bash tests/regression/run_ci.sh`，含 source runtime success/rollback/retention 契约
- dev/test/source-runtime 的真实 rustc 命令均为 `debuginfo=1` 且无 incremental flag；最终 incremental 目录 `0B`。
- 清理报告删除 `57.6GiB` 可重建文件；完成全门禁后 target 为 `8.3G`，磁盘可用空间相较任务开始净增加约 `31GiB`。

## Risks / Follow-ups

- 当前本地 runtime 仍运行已验证的不可变 `67b9a915`，本任务未部署；live release store 仍为旧保留数量，下一次成功调用新部署脚本才会收敛为 current + previous。
- release 清理遇到未知目录、symlink、额外文件或删除失败会保留并记日志，需要人工审计，而不是扩大删除权限。
- canonical main worktree 可能承载其它任务，本次只清理其 ignored Cargo target，没有修改、提交或重置其源码状态。
- 若后续还需减少不同 worktree 的依赖重复编译，单独评估内容寻址的 `sccache`；不要共享可写 `CARGO_TARGET_DIR`。

## Next Entry Point

下一次本地 revision-bound 部署从 `scripts/deploy_source_runtime.sh` 与 `docs/runbooks/source-web-startup.md` 开始；部署完成后检查 `data/releases/source/current`、`previous`、manifest 的 `cargo_profile` 和清理日志。
