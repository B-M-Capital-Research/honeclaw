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
  - `AGENTS.md`
  - `docs/repo-map.md`
  - `docs/invariants.md`
  - `docs/decisions.md`
  - `docs/runbooks/source-web-startup.md`

## Goal

在不共享可写 Cargo target、不削弱 revision-bound 源码部署 provenance 和回滚能力的前提下，压缩本机 Rust 构建与不可变 release 的长期磁盘占用，并安全移除已经审计为无未提交改动的其它 Codex worktree。

## Scope

- 为直接源码运行建立独立、低调试膨胀、关闭 incremental 的 Cargo profile 与产物目录。
- 为常规 Rust 开发、检查与测试关闭 incremental，并把调试信息限制到可保留行号回溯的级别；不改变 CI 命令和测试范围。
- 直接源码部署只保留当前与上一份有效不可变 release；未知目录、异常目录与失败部署不参与清理。
- 保持每个 worktree 独立的 Cargo target，避免并发构建跨 revision 覆盖产物。
- 删除已确认 clean、提交仍可达且无运行进程占用的 `788a`、`8654` worktree；不删除 canonical main checkout 和当前 `4321` worktree。

## Validation

- [x] `cargo` 生效配置证明 dev/source-runtime/test profile 均为 `debug=1`、`incremental=false`。
- [x] 冷构建产物位于 `target/source-runtime/`，部署 manifest 记录 profile，三项运行二进制与 revision/source provenance 验收不回归。
- [x] release retention 回归证明只清理可识别且安全的旧 release，保留 current、previous、未知目录和包含异常文件的目录。
- [x] 失败部署与 rollback 路径不提前清理旧 release。
- [x] 源码部署契约回归、CI-safe 回归、Rust check/test、Web 测试与 Worker typecheck/test 通过。
- [x] 记录优化前后实际磁盘基线；空间数值只作为本机验收证据，不编码为易漂移的 CI 阈值。
- [x] 两个其它 worktree 已完成 clean、提交可达性、进程占用审计并从 worktree registry 移除。

## Documentation Sync

- [x] 更新 `AGENTS.md` 与 `docs/invariants.md` 的长期 profile / target 隔离 / release retention 约束。
- [x] 更新 `docs/repo-map.md`、`docs/decisions.md` 与 `docs/runbooks/source-web-startup.md`。
- [x] 完成后将本计划移入 `docs/archive/plans/`，从 `docs/current-plan.md` 移除，并写入 handoff 与 `docs/archive/index.md`。

## Verification Evidence

- 三个真实 rustc invocation 均带 `-C debuginfo=1` 且不带 `-C incremental`；dev/test/source-runtime 的 incremental 目录最终均为 `0B`。
- 清理前保留 checkout 的 Cargo target 合计约 `39.8G`，另一个已移除 worktree 约 `9.9G`；Cargo 精确清理报告删除 `57.6GiB` 可重建文件，数据卷可用空间从 `33GiB` 上升到 `72GiB`。
- 完成所有冷构建和仓库门禁后，唯一活动开发 worktree 的完整 target 为 `8.3G`（`debug=5.5G`、`source-runtime=2.8G`），数据卷仍有 `64GiB` 可用；两个 incremental 目录均为空。
- `cargo check/test`、Web `347/347`、Edge Worker `45/45`、CI-safe regressions、源码部署成功/失败/rollback/retention 契约全部通过。
- 当前本地运行时保持原不可变 release `67b9a915`，`/api/meta` 与 `active chats=0` 健康；本任务没有部署或替换运行进程。

## Risks / Open Questions

- 调试信息过度裁剪会降低 panic/backtrace 可用性，因此保留 `debug=1`，暂不启用 strip。
- release 清理必须发生在新部署 commit 和 rollback disarm 之后；任何不满足严格目录契约的目标都 fail-safe 保留。
- 不采用跨 worktree 共享可写 target；若未来需要进一步压缩重复依赖，应单独评估 `sccache` 等内容寻址缓存。
