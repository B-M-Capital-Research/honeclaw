# Proposal: Cloud Worker Lease Plane for Side-Effect Ownership

status: proposed
priority: P1
created_at: 2026-06-02 08:05:42 +0800
owner: automation
verification: see `## 验证方式`
risks: see `## 风险与取舍`

## related_files

- `README.md`
- `AGENTS.md`
- `docs/repo-map.md`
- `docs/invariants.md`
- `docs/decisions.md`
- `docs/current-plan.md`
- `docs/proposal/auto_p1_runtime_readiness_matrix.md`
- `docs/proposal/auto_p1_config-apply-evidence.md`
- `docs/proposal/auto_p1_runtime-dependency-circuit-breaker.md`
- `docs/proposal/auto_p1_storage-schema-migration-registry.md`
- `docs/proposal/auto_p1_product-rollout-kill-switch.md`
- `docs/proposal/auto_p1_user-journey-replay-lab.md`
- `docs/proposal/auto_p1_redacted-support-bundle.md`
- `docs/handoffs/cloud-runtime-impact-report-2026-05-28.md`
- `config.example.yaml`
- `crates/hone-core/src/cloud_runtime.rs`
- `crates/hone-web-api/src/lib.rs`
- `crates/hone-web-api/src/routes/meta.rs`
- `bins/hone-cli/src/start.rs`
- `bins/hone-cli/src/cloud.rs`
- `memory/src/cron_job/storage.rs`
- `memory/src/cron_job/history.rs`
- `crates/hone-event-engine/src/engine.rs`
- `crates/hone-channels/src/scheduler.rs`
- `bins/hone-desktop/src/sidecar/processes.rs`

## 背景与现状

Hone 正在从单机本地助手扩展到 public Web、Hone Cloud API、桌面 remote/bundled、多渠道 IM、event-engine 主动推送和 cloud mode。仓库已经补上了不少云化基础：

- `config.example.yaml` 明确 `cloud.mode=local|cloud|auto`，并提醒 `cloud.strict_no_local_storage=true` 只能在所有存储后端都有 PG / OSS 实现后打开。
- `crates/hone-core/src/cloud_runtime.rs` 已有 `RuntimeRole::from_env()`，通过 `HONE_RUNTIME_ROLE=web|worker|all` 区分 Web-only 与 worker-capable 进程。
- `crates/hone-web-api/src/lib.rs` 在 `runtime_role.runs_worker_tasks()` 为 true 时才启动 event-engine、mainline distill cron、scheduler 和 scheduler event handler；否则记录 `runtime_role=web: scheduler/event-engine/channel worker tasks disabled`。
- `bins/hone-cli/src/start.rs` 也根据同一个 runtime role 决定是否启动 iMessage / Discord / Feishu / Telegram sidecar。
- `/api/meta` 已返回 `runtime_role`、`cloud_storage_authoritative`、`local_durable_dependency_count`，但 `worker_leader` 当前固定为 `None`。
- `memory/src/cron_job/storage.rs` 在 cloud cron 路径里已经使用 PG `try_claim_cron_due_job` 对单个 due slot 做 claims，说明仓库已经接受“PG 负责分布式 claim/fencing”的方向。

这些变化很关键，但它们还没有形成一个完整的 **worker ownership** 产品架构。现在 runtime role 只能粗粒度阻止 Web-only 进程启动 worker 任务；它不能回答“当前谁是 worker leader”“某个 side-effect subsystem 是否已被租约保护”“worker 失联后谁接管”“两个 worker 同时运行时哪些副作用会重复”“iMessage 这类本地 privileged integration 是否被绑定到单机 leader”。

这在 Hone 的产品形态里不是纯运维细节。event-engine、scheduler、channel listener、digest flush、mainline distill、webhook/email/PWA 等后续能力都会产生外部副作用：调用 LLM、消耗 FMP/Tavily、写状态、发送通知、推送 IM、更新任务历史。云化后如果只靠“部署时不要多开 worker”这种人工约定，最终会表现为重复推送、重复计费、错乱的 delivery log、同一 cron slot 多次执行或某个机器故障后无人接管。

## 问题或机会

1. **Runtime role 不是 ownership。**
   `HONE_RUNTIME_ROLE=worker` 只能说明这个进程可以运行 worker 任务，不能说明它已经获得某个租约，也不能防止两个 worker 同时启动同一类副作用。

2. **`/api/meta.worker_leader=None` 暴露了产品层缺口。**
   管理端和支持人员可以看到 runtime role，但看不到 leader identity、lease expiry、最后 heartbeat、接管历史或当前副作用是否受保护。

3. **Cron due claim 只保护单个执行窗口，不保护 worker 子系统。**
   PG due claim 能降低同一 cron job 同一 slot 重复执行，但 event-engine poller、global digest、mainline distill、channel listener、iMessage 和未来 webhook/email worker 仍需要 subsystem-level lease。

4. **Cloud storage authoritative 与 worker safety 是两条不同成熟度。**
   即便 session、quota、portfolio、cron、notification prefs 等逐步迁到 PG/OSS，只要副作用 worker 没有租约，云部署仍然不能安全多副本。

5. **本地桌面和云部署需要不同解释。**
   Desktop bundled 模式可以默认 `all`，因为同一 app 管理本机 sidecar；cloud Web replica 应默认 `web`；cloud worker 应显式持有 lease；iMessage 仍应是单机 privileged worker，不应被普通 cloud replica 自动接管。

这是 P1：它不直接改投资输出质量，但会影响云部署可用性、主动通知可信度、成本控制和运维恢复。没有 worker lease plane，后续 storage migration、billing、PWA/email/webhook、release readiness 和 public growth 都会踩同一类重复副作用风险。

## 方案概述

新增 **Cloud Worker Lease Plane**：一个以 PG lease 为核心、以 `/api/meta` / admin UI / CLI doctor 可见的副作用所有权控制面。它把“哪些进程可以运行 worker 任务”升级为“哪些进程在什么时间持有哪些副作用租约，并如何续约、释放、接管和降级”。

核心对象：

- `WorkerInstance`
  当前进程身份。包含 `instance_id`、hostname、pid、version、runtime_role、deployment_mode、started_at、last_heartbeat_at、capabilities。

- `WorkerLease`
  对某个副作用 subsystem 的独占权。建议首批 key：`scheduler`, `event_engine`, `mainline_distill`, `channel:feishu`, `channel:telegram`, `channel:discord`, `channel:imessage`, `webhook_delivery`。

- `LeaseState`
  `held`、`renewing`、`expired`、`stolen`、`released`、`disabled`、`local_only`。

- `LeaseFenceToken`
  递增 fencing token。长期目标是让每次副作用写入或投递都能记录当时 token，避免旧 leader 在网络抖动后继续写入。

- `WorkerTopologySnapshot`
  管理端可见投影：当前 Web replicas、worker candidates、held leases、过期租约、最近 takeover、未受保护的 local-only side effects。

第一版不需要完成全分布式调度系统。它先把云部署最危险的“多 worker 重复副作用”收束为可观测、可测试、可逐步接入的 lease plane。

## 用户体验变化

### 用户端

- Public `/chat` 用户不会直接看到 lease 细节，但会减少重复 digest、重复 IM 推送和“同一个定时任务执行两次”的体验。
- 当 worker lease 丢失时，主动推送可以进入 `temporarily_delayed`，而不是多个进程抢着补发。
- 如果公共服务处于 Web-only 模式且没有 worker leader，用户端定时任务和主动提醒入口可以显示“后台 worker 暂不可用”，避免假装任务会执行。

### 管理端

- Dashboard 或 Runtime 页面增加 `Worker Ownership` 区块：
  - 当前 runtime role：`web` / `worker` / `all`。
  - 当前 worker leader instance id、版本、最后 heartbeat、lease expiry。
  - 每个 subsystem 的 lease 状态：scheduler、event-engine、channel listeners、iMessage、mainline distill。
  - 最近 24h takeover / expired / renewal failure 记录。
- `/api/meta` 不再返回空 `worker_leader`，而是返回可读摘要；cloud mode 下若 `runtime_role=web` 且没有 worker lease，readiness 显示 degraded。
- `/task-health`、`/notifications` 和 `/logs` 可以引用 lease token，区分“任务没有跑是因为没有 leader”与“任务运行失败”。

### 桌面端

- Bundled 桌面继续可以使用 `runtime_role=all`，但本机 sidecar ownership 应显示为 `local_only` 或 `desktop_owned`，不要和 cloud worker lease 混淆。
- Remote desktop 只展示远端 `/api/meta` 的 worker topology，不用本机进程扫描推断远端状态。
- 如果用户把 desktop 指向 cloud backend，桌面应能提示“远端 worker offline，聊天可用但主动推送暂停”。

### 多渠道

- Feishu / Telegram / Discord channel sidecar 在 cloud mode 下必须先拿到 `channel:<id>` lease 才能监听或主动投递。
- iMessage 标记为 `local_privileged_single_host`：即使 cloud mode 开启，也只能由明确本机 worker 持有；Web replicas 不应启动它。
- 群聊、direct chat 和 scheduler-created delivery 仍遵守现有 `ActorIdentity` / `SessionIdentity` / channel target 规则；lease 只决定“谁可以执行副作用”，不改变消息归属。

## 技术方案

### 1. PG lease schema

在 cloud PG 中增加最小表：

```sql
worker_instances (
  instance_id TEXT PRIMARY KEY,
  hostname TEXT NOT NULL,
  pid INTEGER,
  version TEXT NOT NULL,
  runtime_role TEXT NOT NULL,
  deployment_mode TEXT NOT NULL,
  capabilities JSONB NOT NULL,
  started_at TIMESTAMPTZ NOT NULL,
  last_heartbeat_at TIMESTAMPTZ NOT NULL
);

worker_leases (
  lease_key TEXT PRIMARY KEY,
  holder_instance_id TEXT NOT NULL,
  fence_token BIGINT NOT NULL,
  state TEXT NOT NULL,
  acquired_at TIMESTAMPTZ NOT NULL,
  renewed_at TIMESTAMPTZ NOT NULL,
  expires_at TIMESTAMPTZ NOT NULL,
  metadata JSONB NOT NULL
);

worker_lease_events (
  event_id TEXT PRIMARY KEY,
  lease_key TEXT NOT NULL,
  instance_id TEXT NOT NULL,
  fence_token BIGINT,
  event_kind TEXT NOT NULL,
  detail JSONB NOT NULL,
  created_at TIMESTAMPTZ NOT NULL
);
```

租约获取使用单条 PG transaction：

- 如果 lease 不存在或 `expires_at < now()`，当前 instance 可 acquire，并把 `fence_token = old + 1`。
- 当前 holder 可 renew。
- 非 holder 不能覆盖未过期 lease，除非管理员显式 force takeover。
- lease TTL 建议 30-90 秒，renew interval 建议 TTL 的 1/3。

### 2. Worker lease service

在 `hone-core::cloud_runtime` 或独立 `hone-worker` 支撑层定义：

```rust
pub struct WorkerLeaseService { ... }

impl WorkerLeaseService {
    pub async fn register_instance(&self, capabilities: WorkerCapabilities) -> HoneResult<WorkerInstance>;
    pub async fn acquire(&self, lease_key: &str, ttl: Duration) -> HoneResult<Option<HeldLease>>;
    pub async fn renew(&self, lease: &HeldLease) -> HoneResult<LeaseRenewal>;
    pub async fn release(&self, lease: &HeldLease, reason: &str) -> HoneResult<()>;
    pub async fn snapshot(&self) -> HoneResult<WorkerTopologySnapshot>;
}
```

Local mode 不要求 PG lease。建议返回：

- `LeaseMode::LocalOnly`：源码、本地 CLI、desktop bundled 默认。
- `LeaseMode::CloudRequired`：`cloud.mode=cloud` 且 subsystem 会产生外部副作用。
- `LeaseMode::Disabled`：runtime_role=web 或 feature disabled。

### 3. 启动顺序和接入边界

改造方向：

1. `start_server` 读取 runtime role 后，先注册 worker instance。
2. 对每个 worker subsystem 尝试 acquire lease。
3. 只有持有 lease 的 subsystem 才 spawn：
   - `scheduler`
   - `event_engine`
   - `mainline_distill`
   - `channel:<id>` listener / sidecar bridge
4. 每个 held lease 有 renew task；renew 失败或 token 失效时，subsystem 进入 graceful drain：
   - 不再 claim 新任务。
   - 不再发新 outbound。
   - 正在执行的任务按已有 timeout 收口，并在结果里记录 `lease_lost`。

`CronJobStorage::get_due_jobs` 的 PG due claim 可以保留，它是 per-job slot 的二级防线。Worker lease 是子系统入口防线，两者互补。

### 4. Fencing token 逐步落地

第一版至少把 fence token 写入：

- `cron_job_runs.detail_json`
- event-engine `delivery_log.detail`
- task observer `task_runs` rows
- channel heartbeat / process registration
- `/api/meta.worker_leader`

后续再把写入路径升级为强校验：例如 delivery log append 前确认当前 token 仍有效；旧 leader 失去 lease 后的投递标为 `rejected_stale_worker_token`。

### 5. API、CLI 和 UI

新增或扩展：

- `/api/meta`
  - `worker_leader`: `{ instance_id, lease_key, fence_token, expires_at }`
  - `worker_topology`: 简要聚合，可按部署模式裁剪。
- `/api/runtime/worker-leases`
  - admin-only 列表和事件历史。
- `hone-cli cloud doctor`
  - 输出 PG lease table 是否存在、当前 role、当前 instance、held leases、expired leases。
- `hone-cli status --json`
  - 增加 worker ownership 摘要。

前端可以先把 Worker Ownership 放在 dashboard/settings 的只读诊断区，不立即提供 force takeover。force takeover 应等审计和权限提案落地后再开放。

## 实施步骤

### Phase 1: Lease schema 和只读 topology

- 在 cloud PG schema/migrate 路径增加 `worker_instances`、`worker_leases`、`worker_lease_events`。
- 实现 `WorkerLeaseService` 和 local-only fallback。
- `/api/meta` 返回非空 worker topology 摘要；local mode 显示 `local_only`。
- `hone-cli cloud doctor` 显示 lease readiness。

### Phase 2: Scheduler / event-engine lease gate

- 在 `start_server` 中对 `scheduler`、`event_engine`、`mainline_distill` 获取 lease 后再 spawn。
- renew 失败时停止 claim 新任务，记录 `lease_lost`。
- 给 cron run、event delivery、task_runs 写入 fence token。
- 增加 admin 只读 Worker Ownership 面板。

### Phase 3: Channel listener ownership

- CLI start 和 desktop sidecar 启动 channel 前，按 cloud/local mode 判断是否需要 `channel:<id>` lease。
- Feishu / Telegram / Discord 接入 lease heartbeat。
- iMessage 标为 local privileged lease，不允许普通 cloud Web replica 启动。
- channel status 合并 process heartbeat 与 lease holder，显示“进程在线但未持有 lease”这种危险状态。

### Phase 4: Takeover、告警和强 fencing

- 支持 worker leader 故障后的自动 takeover，记录 lease event。
- 给管理端增加最近 takeover / expired lease 告警。
- 对关键写入和外发路径校验 fence token，旧 leader 不能继续写 delivery log 或主动发送。
- 后续接入 rollout kill switch、dependency circuit breaker 和 redacted support bundle。

## 验证方式

- Rust 单元测试：
  - acquire/renew/release 在未过期、已过期、同 holder、不同 holder 场景下行为正确。
  - fence token 每次 takeover 单调递增。
  - local mode 返回 `local_only`，不要求 PG。
  - `runtime_role=web` 不尝试获取 worker leases。
- 集成测试：
  - 两个 mock worker 同时争抢 `scheduler`，只有一个获得 lease。
  - holder 停止 renew 后，另一个 worker 在 TTL 后 takeover。
  - `CronJobStorage::get_due_jobs` 仍能用 due claim 防止同一 slot 重复执行。
  - event-engine / scheduler 未持有 lease 时不 spawn。
- 前端测试：
  - Worker Ownership 面板能展示 `local_only`、`held`、`expired`、`degraded`。
  - `/api/meta.worker_leader` 缺失时显示 degraded 而不是崩溃。
- 手工验收：
  - 本地 desktop bundled 模式不配置 PG 时仍正常运行，显示 local-only ownership。
  - cloud mode 启动一个 `web` 和一个 `worker`，确认 Web 不跑 scheduler/event-engine，worker 持有 lease。
  - 启动两个 worker，确认只有一个发 digest / 执行 cron / 跑 event-engine。
  - 杀掉 worker 后，另一个 worker 在 TTL 后接管；重复推送率为 0 或有明确 fence 拒绝记录。
- 指标：
  - 每个 subsystem 的 lease availability。
  - takeover count、renew failure count、stale-token reject count。
  - worker offline 导致的 delayed scheduled runs 数量。

## 风险与取舍

- 风险：lease plane 增加云部署复杂度。取舍：local/desktop 默认走 `local_only`，只在 cloud authoritative 或明确 worker role 下要求 PG lease。
- 风险：renew 失败时中止正在执行的任务可能导致用户看到失败。取舍：第一版 graceful drain，不强杀已有 run；强 fencing 逐步落地。
- 风险：只做 subsystem lease 仍可能有单 job 重复。取舍：保留现有 cron due claim 作为二级防线，二者职责不同。
- 风险：force takeover 可能被误用。取舍：第一版不做 UI force takeover，只做自动过期接管和只读诊断。
- 风险：iMessage 与 cloud worker 心智冲突。取舍：明确 iMessage 是 local privileged single-host integration，不把它纳入可任意迁移的 cloud worker pool。
- 不做：不实现 Kubernetes operator，不替换现有 process lock，不把所有 runtime logs 迁入 PG，不在第一版改写 event-engine 内部 poller 架构，不默认开启多 worker 自动扩缩容。

## 与已有提案的差异

查重范围覆盖 `docs/proposal/` 与 `docs/proposals/` 下全部现有提案，并重点比对以下相邻主题：

- `auto_p1_runtime_readiness_matrix.md`
- `auto_p1_config-apply-evidence.md`
- `auto_p1_runtime-dependency-circuit-breaker.md`
- `auto_p1_storage-schema-migration-registry.md`
- `auto_p1_product-rollout-kill-switch.md`
- `auto_p1_user-journey-replay-lab.md`
- `auto_p1_redacted-support-bundle.md`
- `docs/handoffs/cloud-runtime-impact-report-2026-05-28.md`

差异结论：

- 不重复 `auto_p1_runtime_readiness_matrix.md`：readiness 回答“当前能力能不能用、缺什么配置”；本提案回答“哪个 worker instance 拥有副作用执行权，以及失联后如何接管”。
- 不重复 `auto_p1_config-apply-evidence.md`：config apply 解释 canonical/effective/running state 是否一致；本提案不处理配置 drift，而处理运行期 lease 和 side-effect ownership。
- 不重复 `auto_p1_runtime-dependency-circuit-breaker.md`：circuit breaker 在依赖故障时快速降级；本提案在多个 worker 存在时保证只有 lease holder 可以执行副作用。
- 不重复 `auto_p1_storage-schema-migration-registry.md`：storage registry 管 schema/migration/version；本提案只新增 worker lease schema 和 ownership contract，不迁移业务 store。
- 不重复 `auto_p1_product-rollout-kill-switch.md`：rollout/kill switch 控制功能是否开放；worker lease 控制某个开放功能由谁执行。
- 不重复 `auto_p1_user-journey-replay-lab.md`：replay lab 是发布前产品旅程验证；本提案提供可被 replay 验证的 worker failover/duplicate prevention 机制。
- 不重复 `auto_p1_redacted-support-bundle.md`：support bundle 导出排障证据；本提案产生 worker topology / lease events 这类证据来源。
- 与 `cloud-runtime-impact-report-2026-05-28.md` 的关系：该 handoff 是一次影响评估和后续建议，不是 proposal。它指出 runtime role 和 distributed leases 是云化前置风险；本提案把其中的 worker/lease 子问题拆成可执行产品架构方案。

本轮选择该主题，是因为当前代码已经具备 runtime role gate 和局部 PG cron claim，但还缺少可观测、可接管、可逐步强 fencing 的 worker lease plane。它是 Hone 从单机/桌面走向可靠 cloud 多副本时必须补上的副作用所有权层。

## 文档同步说明

本轮只新增 proposal，不开始执行实现，不修改业务代码、测试代码、运行配置或 `docs/current-plan.md`。若后续实际落地，应按动态计划准入标准新增或复用 `docs/current-plans/cloud-worker-lease-plane.md`，并在新增 lease schema、runtime ownership contract、启动顺序、`/api/meta` shape、CLI doctor 输出或 channel sidecar ownership 行为时同步更新 `docs/repo-map.md`、`docs/invariants.md`、`docs/decisions.md`、相关 runbook 和 handoff/archive 索引。
