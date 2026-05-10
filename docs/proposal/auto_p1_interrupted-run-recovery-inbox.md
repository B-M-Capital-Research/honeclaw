# Proposal: Interrupted Run Recovery Inbox

status: proposed
priority: P1
created_at: 2026-05-11 05:03:48 +0800
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
- `docs/proposal/auto_p1_run_trace_workbench.md`
- `docs/proposal/auto_p1_runtime_readiness_matrix.md`
- `docs/proposal/auto_p1_delivery_decision_loop.md`
- `docs/proposal/auto_p1_automation_intent_control_plane.md`
- `docs/proposal/auto_p0_investment_output_safety_gate.md`
- `memory/src/session.rs`
- `memory/src/session_sqlite.rs`
- `memory/src/cron_job/history.rs`
- `bins/hone-feishu/src/handler.rs`
- `bins/hone-feishu/src/scheduler.rs`
- `crates/hone-channels/src/agent_session/core.rs`
- `crates/hone-channels/src/ingress.rs`
- `crates/hone-channels/src/execution.rs`
- `crates/hone-web-api/src/routes/history.rs`
- `crates/hone-web-api/src/routes/task_runs.rs`
- `packages/app/src/pages/chat.tsx`
- `packages/app/src/pages/sessions.tsx`
- `packages/app/src/pages/task-health.tsx`
- `packages/app/src/pages/logs.tsx`

## 背景与现状

Honeclaw 的主链路已经把用户消息尽早持久化，再进入 runner、tool、finalizer、session persistence 和 channel outbound。这个设计有利于保留用户输入，但也产生了一个明确的产品状态：如果进程重启、runner 卡死、桌面 sidecar 被清理、channel listener 崩溃或机器休眠，系统可能留下“最后一条是 user、没有 assistant 终态”的会话。

仓库里已经有若干底层恢复原语：

- `memory/src/session_sqlite.rs` 提供 `find_interrupted_sessions(channel, updated_after, updated_before)`，按 `actor_channel`、`session_kind='direct'`、`last_message_role='user'` 查找近期中断会话。
- `memory/src/session.rs` 在启用 `SessionIndex` 时把这个查询暴露给 runtime；纯 JSON 后端无法列出中断项，只能返回空列表。
- `bins/hone-feishu/src/handler.rs` 在 Feishu 启动时调用 `recover_interrupted_sessions`，对最近 30 分钟但超过 30 秒 grace 的直聊会话补发“服务重启，之前的消息处理已中断，请稍后重试。”，并写入一条 assistant 消息，避免下次重启重复通知。
- `memory/src/cron_job/history.rs` 提供 `recover_stale_started_executions`，能把 `running + pending` 且 stale 的 cron 执行记录终结为 `execution_failed / send_failed`。
- `bins/hone-feishu/src/scheduler.rs` 在 Feishu scheduler 启动时回收 stale pending 定时任务。
- `packages/app/src/pages/chat.tsx` 已有“正在恢复对话”的前端状态，`packages/app/src/pages/task-health.tsx` 能看周期任务健康，`packages/app/src/pages/logs.tsx` 能看日志，但没有一个以“中断运行”为核心对象的恢复入口。

这说明 Hone 已经能局部发现并终结部分中断状态，但产品架构仍是分散的：Feishu 直聊有启动补发，Feishu scheduler 有 stale row 回收，其它渠道、Web chat、桌面 bundled runtime、group chat 和 admin 工作台缺少统一模型。

## 问题或机会

中断运行不是普通错误。它发生在用户已经把问题交给 Hone 之后，系统却没有给出明确结果；这会直接损害核心信任，尤其是投资场景里的定时监控、持仓提醒、公司画像更新和跨渠道对话。

当前缺口主要有五类：

1. **恢复能力不对称。** Feishu direct 有局部补发，其它 channel 和 Web/public chat 没有统一恢复策略；用户在不同入口遇到同一类中断，会得到不同体验。
2. **中断项没有台账。** Admin 只能从 session history、logs、task-health 或 SQLite 状态推断，无法看到“哪些请求中断、是否已通知用户、是否能重试、是否已放弃”。
3. **恢复动作只有“补发失败提示”。** 有些中断请求可以安全重试，有些只能告知失败，有些需要用户确认，有些是 cron/task 应该在下一窗口自动恢复。当前没有结构化决策。
4. **group chat 和 channel target 信息不足。** 现有 Feishu 恢复只处理 unscoped direct session，因为 group 回复需要 chat_id/target；这暴露出恢复模型必须显式保存 origin、target、message id 和 reply semantics。
5. **用户无法区分“没处理”“处理失败”“已恢复”。** Public Web 和 Desktop 用户看不到后台是否已经捕获中断，也不知道是否该重发同一个问题。维护者也难以衡量重启、sidecar 清理、runner timeout 对用户体验的真实影响。

这值得列为 P1：它不改变 runner 语义，也不要求先落地完整 Run Trace Workbench，但能显著提升核心可用性、运维闭环和用户信任。对一个多渠道 agent 产品来说，“中断后有明确台账和恢复动作”是可靠体验的基础。

## 方案概述

新增 **Interrupted Run Recovery Inbox**：把“会话最后一条是 user、任务执行停在 running/pending、outbound 未终态”等中断状态统一转成可查询、可恢复、可审计的 recovery item。

核心对象：

- `RecoveryItem`：一次未闭环运行，来源可以是 chat session、public API request、channel message、cron execution、heartbeat、research task。
- `RecoveryCause`：`process_restart`、`runner_timeout`、`sidecar_conflict_cleanup`、`channel_listener_restart`、`outbound_unknown`、`stale_running_row`、`manual_marked`。
- `RecoveryDecision`：`notify_only`、`retry_once`、`ask_user_confirmation`、`suppress_stale`、`mark_failed`、`wait_next_schedule`、`manual_review`。
- `RecoveryStatus`：`open`、`notified`、`retrying`、`recovered`、`failed`、`dismissed`。
- `RecoveryTarget`：actor、session identity、channel target、message id、task id、execution id、可选 trace id。

一期目标是只读发现 + 明确终态，不做自动重跑所有中断请求。先把现有 Feishu direct 和 cron stale recovery 抽象成共享模型，然后把 Web/admin/desktop 看得见，最后再对低风险请求提供一次性 retry。

## 用户体验变化

### 用户端

- Public `/chat` 如果检测到上一条消息已中断，不再只让用户困惑地停在 loading 或历史尾部，而是展示一条明确状态：“上一条请求因服务重启未完成，可以重新发送或查看记录。”
- 如果系统已经补发失败提示，聊天历史中出现的是可理解的 assistant 状态消息，并带有“可重试”动作，而不是沉默。
- 对投资敏感、附件较大或已经跨时间窗口的问题，默认要求用户确认后再重试，避免把旧行情上下文当作新问题再次执行。

### 管理端

- 新增 `Recovery` 视图，或在 `Sessions` / `Task health` 增加 `Interrupted` tab，列出最近中断项。
- 每条 item 显示 origin、actor、channel、session/task、last user preview、发生时间、当前状态、建议动作、是否已通知用户。
- 管理员可以执行：标记已通知、补发失败提示、触发一次 retry、dismiss、打开 session、打开 task run、跳到 logs/trace。
- Dashboard 增加“最近 24h interrupted runs”指标，分 chat、cron、channel outbound、desktop sidecar。

### 桌面端

- Bundled runtime 启动后，如果上次退出留下未完成会话或 cron rows，desktop dashboard 显示“有 N 个未闭环请求”，用户可以打开本地 console 处理。
- sidecar cleanup 或进程锁接管不再只停留在日志层；若清理动作导致某些运行中断，Recovery Inbox 给出结果。
- Remote mode 只展示远端 backend 返回的 recovery 状态，不用本机文件扫描推断远端情况。

### 多渠道

- Feishu 现有启动补发逻辑迁入共享 recovery service 后，Telegram、Discord、iMessage 可以复用同一套 direct-message 通知策略。
- 群聊中断默认不主动乱发；只有恢复 item 保存了可用 `channel_target` / reply target 时才补发，否则进入 admin manual review。
- 定时任务中断不再只写 stale failed row；用户端任务页可看到“本次运行被恢复为失败，下一次计划仍会执行”。

## 技术方案

### 1. 新增 recovery 存储与服务

建议在 `memory` 新增 `recovery.rs` 或 `memory/src/recovery/mod.rs`，用 SQLite 保存 item：

```text
recovery_items (
  item_id TEXT PRIMARY KEY,
  origin TEXT NOT NULL,
  status TEXT NOT NULL,
  cause TEXT NOT NULL,
  actor_channel TEXT NOT NULL,
  actor_user_id TEXT NOT NULL,
  actor_scope TEXT,
  session_id TEXT,
  task_id TEXT,
  execution_id TEXT,
  channel_target TEXT,
  channel_message_id TEXT,
  trace_id TEXT,
  last_user_preview TEXT,
  decision TEXT NOT NULL,
  detail_json TEXT NOT NULL,
  detected_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  notified_at TEXT,
  recovered_at TEXT,
  dismissed_at TEXT
)
```

服务接口：

```rust
pub trait RecoveryStore {
    fn upsert_item(&self, item: RecoveryItem) -> HoneResult<()>;
    fn list_items(&self, filter: RecoveryFilter) -> HoneResult<Vec<RecoveryItem>>;
    fn mark_status(&self, item_id: &str, status: RecoveryStatus, detail: Value) -> HoneResult<()>;
}

pub struct RecoveryService { ... }
```

第一版可以只在启动扫描时创建 item；后续再从 live `AgentSession` timeout/outbound unknown 直接写入。

### 2. 抽象中断发现器

把现有局部逻辑改造成发现器：

- `SessionInterruptionDetector`：复用 `SessionStorage::find_interrupted_sessions`，并补充 last user preview、message id、session metadata、channel target。
- `CronStaleRunDetector`：在 `recover_stale_started_executions` 前后创建或更新 recovery item，保留 execution id 和 job id。
- `OutboundUnknownDetector`：后续可接入 channel outbound send failure / unknown 状态。
- `ResearchTaskDetector`：后续可覆盖 `research` running/pending 但无 `answer_markdown` 的任务。

发现器只负责产生 item，不直接通知用户；通知和 retry 由 decision 层决定。

### 3. Recovery decision 层

根据 origin 和数据完整度选择动作：

- Direct chat + channel target 明确 + 最后一条 user 较新：`notify_only`，可附 `retry_once` 建议。
- Public Web chat：`notify_in_history`，前端显示重试按钮。
- Group chat + target 缺失：`manual_review`，不主动发群消息。
- Cron stale running row：`mark_failed` + `wait_next_schedule`，不立即重跑，避免重复主动推送。
- Heartbeat 或行情敏感任务：默认 `wait_next_schedule`，除非用户/管理员显式 retry。
- 有附件且附件仍在 actor sandbox：可 retry；附件缺失则 `ask_user_confirmation` 或 `failed`。

这层应该是确定性规则，不调用 LLM；后续可让 Run Trace Workbench 或 Safety Gate 读取 recovery item，但 recovery 本身不判断投资内容质量。

### 4. API 与前端

Admin API：

- `GET /api/recovery/items?status=&origin=&actor=&from=&to=`
- `POST /api/recovery/items/:id/notify`
- `POST /api/recovery/items/:id/retry`
- `POST /api/recovery/items/:id/dismiss`

Public API：

- `GET /api/public/recovery/recent`
- `POST /api/public/recovery/:id/retry`

Public API 只能返回当前 cookie actor 的 item，不接受任意 actor query；retry 必须重新校验 quota/entitlement、附件存在性、当前时间和安全策略。

前端落点：

- `packages/app/src/lib/recovery.ts`
- `packages/app/src/context/recovery.tsx`
- `packages/app/src/pages/recovery.tsx` 或并入 `sessions.tsx` / `task-health.tsx`
- Public `chat.tsx` 在历史恢复完成后检查 recent recovery item，显示一条内联状态。

### 5. Retry 兼容策略

自动 retry 要非常保守：

- 不复用旧 runner 进程；重新走 `AgentSession::run()`。
- 新的 retry message 必须带 metadata：`recovery_item_id`、`recovered_from_session_id`、`original_message_id`。
- 对同一 item 只允许 `retry_once`，失败后进入 `manual_review`。
- 重试前重算 live Beijing time，不使用旧 prompt frozen time 当作当前时间。
- 对主动推送和 heartbeat，不默认立即 retry，避免在错误时间点补发过期市场信息。
- 保留原 session 中的失败提示，retry 结果作为新的 assistant turn，而不是覆盖历史。

## 实施步骤

### Phase 1: Recovery item 和只读台账

- 在 `memory` 新增 recovery store、类型和单元测试。
- 把 Feishu direct 启动发现的 interrupted sessions 写入 recovery item，并保持原补发行为不变。
- 把 Feishu scheduler stale pending recovery 写入 recovery item。
- 增加 admin `GET /api/recovery/items`，先展示只读列表。

### Phase 2: 共享通知服务

- 在 `hone-channels` 增加 channel-agnostic recovery notification trait。
- Feishu direct 使用共享服务；Telegram/Discord/iMessage direct 逐步接入。
- Public Web chat 显示当前 actor 的 recent recovery item。
- Recovery item 状态从 `open` 正确推进到 `notified` / `failed` / `dismissed`。

### Phase 3: 管理端处理动作

- Admin 支持 notify、dismiss、mark failed。
- `Sessions`、`Task health`、`Logs` 增加 recovery link。
- Desktop bundled dashboard 增加最近中断项摘要。
- 增加指标：中断数、已通知数、未闭环数、平均发现延迟。

### Phase 4: 有边界的 retry

- 只对 direct chat/public chat 且 target/attachment/context 完整的 item 开启 `retry_once`。
- 重试统一走 `AgentSession::run()`，并写入 retry metadata。
- 对 cron/heartbeat 只允许 admin 手动 rerun 或等待下一次 schedule。
- 接入未来 Run Trace Workbench 时，把 `trace_id` 和 `recovery_item_id` 双向链接。

## 验证方式

- Rust 单元测试：
  - recovery store 能 upsert/list/mark status，重复 detector 扫描不会产生重复 item。
  - `SessionInterruptionDetector` 对 last_message_role=user 的 direct session 生成 item；对 group target 缺失生成 `manual_review`。
  - cron stale row recover 后写入 `mark_failed / wait_next_schedule` item。
  - retry metadata 不覆盖原 session history。
- Web/API 测试：
  - admin `GET /api/recovery/items` 支持 status/origin/actor/time filter。
  - public recovery API 只能返回当前 cookie actor 的 item。
  - public chat 在有 recent item、无 item、item dismissed 三种状态下渲染正确。
- 手工验收：
  - 模拟 Feishu 直聊在 user message 持久化后进程重启，启动后 recovery item 出现，用户收到失败提示，session 尾部写入 assistant 状态。
  - 模拟 stale cron running/pending row，启动后 task-health 显示失败，recovery inbox 说明来自 stale pending recovery。
  - Web public chat 人为制造 interrupted session 后，用户重新登录能看到可理解提示。
  - Desktop bundled 模式清理旧 sidecar 后，dashboard 能显示最近未闭环请求。
- 指标：
  - 新中断运行 90% 以上能在 2 分钟内进入 recovery inbox。
  - 重启后重复补发率接近 0。
  - 用户报告“消息没回但不知道发生什么”的支持问题下降。

## 风险与取舍

- 风险：自动 retry 可能使用过期市场上下文。取舍：一期不自动重跑，Phase 4 只对 direct/public 且用户确认或低风险场景开放一次 retry。
- 风险：Recovery Inbox 与 Run Trace Workbench 看起来重叠。取舍：Recovery 处理“未闭环请求的状态与动作”，Trace 解释“一次运行内部发生了什么”；二者可通过 `trace_id` 链接但职责不同。
- 风险：group chat 恢复可能误发到错误群或泄露上下文。取舍：target 不完整时只进 admin review，不主动补发。
- 风险：额外 SQLite store 增加维护面。取舍：先只保存 item metadata 和 preview，不搬迁 session/cron 真相源。
- 风险：用户看到“服务重启”提示可能降低信任。取舍：沉默更糟；文案应简短、明确、给出重试或等待下一次计划的动作。
- 不做：不引入外部告警 SaaS，不替代 task-health，不做通用工作流引擎，不改变 session ownership / ActorIdentity 语义，不在第一版自动重跑投资敏感定时任务。

## 与已有提案的差异

查重范围包括 `docs/proposal/` 和 `docs/proposals/` 下全部现有提案，重点比对：

- `auto_p1_run_trace_workbench.md`
- `auto_p1_runtime_readiness_matrix.md`
- `auto_p1_delivery_decision_loop.md`
- `auto_p1_automation_intent_control_plane.md`
- `auto_p0_investment_output_safety_gate.md`
- `auto_p1_usage_entitlement_ledger.md`
- `docs/proposals/desktop-bundled-runtime-startup-ux.md`

差异结论：

- 与 `auto_p1_run_trace_workbench.md` 不重复：Run Trace 聚合一次运行的日志、prompt、LLM audit、session evidence；本提案处理“运行没有闭环之后”如何发现、通知、台账化和有限重试。
- 与 `auto_p1_runtime_readiness_matrix.md` 不重复：Readiness 关注运行前配置和能力是否可用；本提案关注运行中断后的用户体验和恢复动作。
- 与 `auto_p1_delivery_decision_loop.md` 不重复：Delivery decision 关注通知是否应该送达、偏好和事件投递闭环；本提案关注已经开始处理但未产生终态的 chat/task recovery。
- 与 `auto_p1_automation_intent_control_plane.md` 不重复：Automation intent 治理任务创建和变更意图；本提案治理执行中断后的恢复台账。
- 与 `auto_p0_investment_output_safety_gate.md` 不重复：Safety gate 判断输出内容能否安全送达；本提案不判断投资内容，只决定中断后如何告知、标记或重试。
- 与 `auto_p1_usage_entitlement_ledger.md` 不重复：Entitlement ledger 记录成本和权益；本提案记录未闭环运行和恢复状态。
- 与 `desktop-bundled-runtime-startup-ux.md` 不重复：该历史提案解决桌面启动和进程接管；本提案把启动/重启留下的未闭环请求转成用户和管理员可处理的 recovery item。

本轮选择该主题，是因为代码已经出现了 Feishu direct 和 cron 的局部恢复实现，但缺少统一产品层。将它提炼为 Recovery Inbox 能以较小架构增量提升多渠道可靠性，并为后续 trace、readiness、safety gate 提供共同的失败闭环对象。

## 文档同步说明

本轮只新增 proposal，未开始执行提案、未改变模块边界、入口、数据流或运行规则，因此不更新 `docs/current-plan.md`、`docs/repo-map.md`、`docs/invariants.md` 或 `docs/decisions.md`。若后续进入实现阶段，应按动态计划准入标准新建/复用 `docs/current-plans/interrupted-run-recovery-inbox.md`，并在落地存储/API 后同步更新 repo map 和必要决策记录。
