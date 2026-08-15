# 调度与定时任务体系化整治

- status: `in_progress`
- created_at: `2026-08-15`
- updated_at: `2026-08-15`
- owner: `Claude`（接手自 Codex）
- related_files:
  - `crates/hone-channels/src/scheduler.rs`
  - `crates/hone-scheduler/src/lib.rs`
  - `crates/hone-web-api/src/routes/events.rs`
  - `crates/hone-web-api/src/routes/meta.rs`
  - `crates/hone-event-engine/src/weekly_report.rs`
  - `crates/hone-llm/src/openai_compatible.rs`
  - `crates/hone-core/src/cloud_runtime.rs`
  - `memory/src/cron_job/mod.rs`
  - `memory/src/cron_job/storage.rs`
  - `memory/src/cron_job/history.rs`
  - `bins/hone-console-page/src/main.rs`
  - `scripts/deploy_source_runtime.sh`
- related_docs:
  - `docs/conventions/periodic_tasks.md`
  - `docs/runbooks/backend-deployment.md`
  - `docs/bugs/README.md`

## Goal

先将当前 GCE 客户 API 从 `runtime_role=all` 回滚到基线拓扑并恢复 Feishu 独立 worker，随后在不重构调度框架的前提下修复心跳契约失败不重试、cron 健康不可见、齐射与传输重试、云 cron 短命 runtime、僵尸执行记录以及 `/api/meta` 阻塞等问题。完成全仓门禁、真实 LLM 契约冒烟、revision-bound 部署与生产基线对比。

## Scope

1. 生产止血：GCE Web 恢复 `HONE_RUNTIME_ROLE=web`，启用并启动 `hone-channel@feishu.service`，验收 public API、loopback readiness、CPU、进程与 journal。
2. P0：心跳契约违规单次 recovery；cron 写 `task_runs.jsonl`、记录 `started_at`/`duration_ms`，周报展示可获得的定时任务健康。
3. P1：确定性派发抖动、job 层并发闸、同凭据流式传输退避重试与 config 收口；复用长驻 cloud runtime；所有渠道统一回收 stale rows；预热 build info、完整云探活预算、meta 总预算、部署探针改为 readiness。
4. 明确不在本轮处理 claim TTL/lease、per-actor IANA 时区、全局 supervisor 和机械内容门禁。

## Validation

- 每个 bugfix 增加或复用自动化回归测试，先跑受影响 crate/模块的定向测试。
- `bash scripts/ci/check_fmt_changed.sh`
- `cargo check --workspace --all-targets --exclude hone-desktop --exclude hone-user-app`
- `cargo test --workspace --all-targets --exclude hone-desktop --exclude hone-user-app`
- `bun run test:web`
- `cd workers/public-community-edge && bun run typecheck && bun run test`
- `bash tests/regression/run_ci.sh`
- `cargo run --example heartbeat_prompt_llm_smoke -p hone-channels`
- 本地 source-runtime 验收后，以不可变 GHCR digest/revision 部署 GCE；观察至少一次半点心跳波次和一次 20:00 daily 波次，并对比任务失败率、CPU、bootstrap/meta 延迟与 stale running rows。

## Documentation Sync

- 新增心跳契约、齐射并发、meta 挂起三个 `docs/bugs/` 条目并更新 `docs/bugs/README.md`。
- 在 `docs/conventions/periodic_tasks.md` 明确用户 cron 也必须写 `task_runs.jsonl`。
- 若实际改动影响模块入口或长期不变量，同步 `docs/repo-map.md` / `docs/invariants.md` / `docs/runbooks/backend-deployment.md`。
- 完成后新增 handoff，更新 `docs/archive/index.md`，将本计划移入 `docs/archive/plans/` 并从 `docs/current-plan.md` 移除；若生产观察尚未覆盖完整窗口，则保持 `in_progress` 并写清剩余观察点。

## Progress

### 已完成（2026-08-15）

- **生产止血**：GCE 已回滚 `HONE_RUNTIME_ROLE=web` 并恢复
  `hone-channel@feishu.service`。复核：`/api/public/bootstrap` 从 49–83 秒
  回到 0.48 秒，load 从 3.x 降到 0.02。
- **P0-1 心跳契约重试** —— `docs/bugs/scheduler_heartbeat_contract_violation_never_retried.md`
- **P0-2 cron 纳入 `task_runs.jsonl`**，执行记录补 `started_at` / `duration_ms`
  （两列可空，NULL = 未知；不回填历史行）
- **P1-1 抖动 + 并发闸 + 流式退避重试** —— `docs/bugs/scheduler_due_burst_has_no_backpressure.md`
- **P1-3 僵尸记录回收覆盖四个渠道**（此前只有飞书）
- **P1-4 `/api/meta` 挂起与部署探针** —— `docs/bugs/meta_route_hangs_on_binary_hash_and_unbounded_pg_probe.md`

接手时对 Codex 已有改动的复核修正（均已合入）：

1. `started_at`/`duration_ms` 的迁移原本写在 `ensure_schema` /
   `init_execution_schema` 里，而这两者**每个调度事件都跑一次**：PG 侧
   `UPDATE ... WHERE started_at IS NULL` 全表扫 + `ALTER COLUMN SET NOT NULL`
   取 ACCESS EXCLUSIVE 锁，恰好加剧我们要修的齐射争用。改为两列可空、不回填。
2. PG 的 `GREATEST(0, NULL)` 返回 **0 而不是 NULL**（GREATEST 忽略 NULL），
   会把未知耗时写成"0 毫秒"；旧实现的 `MAX()` 反而是 NULL 传播。已改用
   `CASE WHEN started_at IS NULL THEN NULL`。
3. 无配对 started 行的直插终态记录原本写 `started_at = executed_at` +
   `duration_ms = 0`，谎报 0 毫秒；改为双 NULL。
4. 契约重试测试的末条断言用合规 JSON 去断 `is_none()`，因
   `profile != Primary` 提前返回而**恒真**，守不住回归；已改用违约内容断言
   「recovery 一次性」。

设计上与原计划的两处偏离（有意为之）：

- **抖动只加在 heartbeat 上**。用户显式设定时刻的定时任务（如 20:00 日报）
  改触发时刻属于产品语义变更，不顺手做；它们的削峰由并发闸完成，不动时刻。
  齐射主力本来也是 heartbeat（25 个 × 每天 48 轮 ≈ 1200 次，定时任务全天约 75 次）。
- **`task_runs` 的 `started_at` 保持取落账时刻**。读取侧只把它当事件时间戳用
  （窗口过滤 / 排序 / `last_seen_at`），不算时延；真实耗时在
  `cron_job_runs.duration_ms`。不值得为此往热路径加一次查询。

### 待办

- P0-2 余项：周报增加"定时任务健康"section
- 本地 source-runtime 部署验收 → GCE 部署 → 重新开 `role=all` → 生产波次对比

## Risks / Open Questions

- 当前 GCE 正在发生客户影响，生产止血优先于所有代码改动；回滚 Web role 时必须同时恢复 Feishu 独立 worker。
- detached worktree 的目标提交可能落后 `origin/main`；实现前必须 fetch 并审计与其它活跃任务的文件重叠，不能覆盖用户或并行任务改动。
- job 层并发闸、cloud runtime 复用和 history schema 均是跨模块行为变化，需保持现有同步调用点和幂等语义。
- 真实 LLM 冒烟依赖外部模型与凭据，失败必须区分实现回归和外部服务问题。
