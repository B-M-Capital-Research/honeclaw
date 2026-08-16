# 调度与定时任务体系化整治

- status: `in_progress`
- created_at: `2026-08-15`
- updated_at: `2026-08-16`
- owner: `Claude`（接手自 Codex）
- related_files:
  - `crates/hone-channels/src/scheduler.rs`
  - `crates/hone-scheduler/src/lib.rs`
  - `crates/hone-web-api/src/routes/events.rs`
  - `crates/hone-web-api/src/routes/meta.rs`
  - `crates/hone-event-engine/src/weekly_report.rs`
  - `crates/hone-llm/src/openai_compatible.rs`
  - `crates/hone-core/src/cloud_sync.rs`
  - `crates/hone-core/src/cloud_runtime.rs`
  - `crates/hone-event-engine/src/global_digest/mainline_cron.rs`
  - `crates/hone-event-engine/src/global_digest/mainline_distill.rs`
  - `memory/src/company_profile/storage.rs`
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

### role=all CPU 修复（2026-08-16）

- **A1 cloud sync worker 配置**（`b82b2017`）：删除固定 `worker_threads(2)`；
  默认 worker 数严格等于 `std::thread::available_parallelism()`，探测失败回退到
  `NonZeroUsize::MIN`，并允许 `HONE_CLOUD_SYNC_WORKER_THREADS` 用合法非零整数覆盖。
  e2-standard-2 默认仍为 2；该项解决硬编码和运维覆盖，不把超配线程当作 CPU 修复。
- **A2 mainline cloud 画像读取放大**（`270a6cb5`）：确认 cron 按 actor 串行，
  actor 内 ticker LLM 最多 6 路并发、另有 1 次 style 调用；一轮调用量为
  `sum_actor(profile_ticker_matches + 1)`，最坏 `O(actor * ticker)`，且没有跨 actor
  cache。actor 私有画像会进入 prompt，所以同 ticker 不保证同答案，不能按 ticker
  无条件跨用户复用。
- CPU 热点不在 LLM 等待：LLM HTTP future 运行在 Web/event-engine Tokio runtime；
  采样中打满的 `hone-cloud-sync` 线程只执行 `run_cloud_sync` 提交的 PostgreSQL
  future。旧 `scan_profiles_for_actor` 先 `list_profiles_raw()`，随后对每个 profile
  调 `get_profile_raw()`；cloud 两个方法每次都重新读取该 actor 的全部文件，并由
  `list_company_profile_files()` 新建连接。若 actor 有 `P_a` 个 profile、`F_a` 个
  profile/event 文件，单轮读取量为 `sum((P_a + 1) * F_a)`，并伴随
  `sum(P_a + 1)` 次连接/查询；与 JSON/Markdown 小段解析相比，PG 握手、协议驱动、
  行解码和内容复制才会落在 `hone-cloud-sync` worker 上。
- 修复增加单次批量 raw-document 读取：每个 actor 只调用一次
  `cloud_files_for_actor()`，在线性单遍中组装全部 profile/event document；mainline
  直接消费批量结果。画像读取降为 `sum(F_a)` 和每 actor 1 次查询，LLM 个性化语义、
  6 路并发、失败 ticker 的 6 小时再试与 prefs 写入不变。`collector.rs` 只服务新闻
  候选收集，与 mainline cron 没有调用关系。
- 失败策略现状：ticker/provider 失败仍会写本轮蒸馏时间，缺失 ticker 最早 6 小时后
  重试；没有 mainline 级指数退避或跨 actor 熔断。OpenRouter 会遍历配置的 client/key
  并做兼容 fallback；OpenAI-compatible 对特定 transport error 最多额外重试一次，
  固定等待 2 秒。本轮没有改这些语义。
- 定向验证：`cargo test -p hone-core cloud_sync --lib -- --nocapture`（4 passed）；
  `cargo test -p hone-memory company_profile --lib -- --nocapture`（26 passed）；
  `cargo test -p hone-event-engine mainline --lib -- --nocapture`（41 passed）。
- 完整验收：workspace all-targets 为 2,588 passed / 0 failed / 113 ignored；
  `hone-memory --ignored` 为 93 passed / 0 failed；`tests/regression/run_ci.sh` 最终退出 0。
  regression 首次因 worktree 缺少 lockfile 已声明的 `@happy-dom/global-registrator`
  在加载阶段失败；执行 `bun install --frozen-lockfile` 后从头重跑通过，lockfile 与工作树
  均未变化。没有 SQL/schema 改动，因此不触发已有数据老库迁移验证。

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
- A1/A2 已按用户要求仅本地提交、未 push；后续先把提交接入可部署分支，再做本地
  source-runtime 验收 → GCE 部署 → 重新开 `role=all` → 生产波次对比

## Risks / Open Questions

- 当前 GCE 正在发生客户影响，生产止血优先于所有代码改动；回滚 Web role 时必须同时恢复 Feishu 独立 worker。
- detached worktree 的目标提交可能落后 `origin/main`；实现前必须 fetch 并审计与其它活跃任务的文件重叠，不能覆盖用户或并行任务改动。
- job 层并发闸、cloud runtime 复用和 history schema 均是跨模块行为变化，需保持现有同步调用点和幂等语义。
- 真实 LLM 冒烟依赖外部模型与凭据，失败必须区分实现回归和外部服务问题。
