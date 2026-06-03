# Proposal: Agent Run Lifecycle Control for Cancel, Background, and Resume

status: proposed
priority: P1
created_at: 2026-06-03 08:06:42 +0800
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
- `docs/proposal/auto_p1_interrupted-run-recovery-inbox.md`
- `docs/proposal/auto_p1_agent-clarification-queue.md`
- `docs/proposal/auto_p1_agent-permission-broker.md`
- `crates/hone-web-api/src/routes/chat.rs`
- `crates/hone-web-api/src/routes/public.rs`
- `crates/hone-channels/src/agent_session/core.rs`
- `crates/hone-channels/src/agent_session/progress.rs`
- `crates/hone-channels/src/agent_session/types.rs`
- `crates/hone-channels/src/agent_session/guard.rs`
- `crates/hone-channels/src/run_event.rs`
- `crates/hone-channels/src/runners/opencode_acp.rs`
- `memory/src/quota.rs`
- `memory/src/session.rs`
- `memory/src/session_sqlite.rs`
- `packages/app/src/pages/chat.tsx`
- `packages/app/src/components/chat-view.tsx`
- `packages/app/src/lib/api.ts`
- `packages/app/src/lib/public-chat.ts`

## 背景与现状

Hone 的一次对话运行已经不再是短请求。用户可能上传附件、触发 `stock_research` / `company_portrait` / `chart_visualization` / `scheduled_task`，runner 可能通过 ACP、OpenCode、本地工具、LLM audit、prompt audit、session persistence 和 finalizer 走完整链路。仓库当前已经有一些运行期体验基础：

- `crates/hone-web-api/src/routes/chat.rs` 的 Web chat 通过 SSE 返回 `run_started`、`assistant_delta`、`tool_call`、`run_error`、`run_finished`，并在后台 `tokio::spawn` 中执行 `AgentSession::run()`。
- `crates/hone-channels/src/agent_session/progress.rs` 和 `core.rs` 已有 `agent.run.progress` watchdog，长时间运行时会持续发 progress event，避免完全静默。
- `packages/app/src/components/chat-view.tsx` 的管理端 chat pending bubble 已展示 queued / thinking / running / streaming / timeout / error，并有一个 UI 层 `stop` 按钮。
- `packages/app/src/pages/chat.tsx` 的 public chat 已有 `AbortController`、本地 pending assistant message、`backgroundPending`、`in_flight` 检测、历史轮询恢复和“本轮已完成”的反馈。
- `memory/src/quota.rs` 与 `agent_session/guard.rs` 已经用 in-flight / reservation guard 区分运行中和成功提交，说明运行期状态对额度、恢复和用户体验都有实际意义。

但这些能力还没有汇成一个服务端的一等 **run lifecycle**。当前 public chat 的停止主要是中止浏览器 fetch；管理端 pending stop 也更像前端状态控制。后端没有稳定 `run_id`、没有 cancel endpoint、没有可查询的 running run registry，也没有把“用户要求停止”“连接断开但后台继续”“后台完成后恢复历史”统一成可审计状态。

## 问题或机会

这是 P1，因为“我能不能停止一次长运行、刷新页面后是否还在处理、后台完成后去哪看结果”直接影响用户对 agent 的可控感和对成本/额度的信任。

1. **停止按钮不等于服务端取消。**  
   浏览器 `AbortController` 只能断开客户端读取；`routes/chat.rs` 中的后台 task 仍可能继续执行 runner、tool、LLM 调用和 session 写入。用户以为已经停止，系统却可能继续消耗 provider 成本、持有 session lock、提交 quota 或产生最终回复。

2. **后台继续是前端推断，不是稳定契约。**  
   Public chat 通过 `user.in_flight > 0 && lastIsUser` 推断后台有未完成运行，然后每 3 秒恢复 history。这对刷新恢复有用，但缺少 run id、阶段、预计动作和取消入口。多个运行来源或异常状态出现时，用户只能看到泛化的“思考中”。

3. **长运行缺少可控降级。**  
   投资研究、附件理解、图表生成和公司画像更新都可能需要较长时间。用户需要在“继续等、转后台、取消、稍后提醒、重新发送”之间做选择，而不是被迫等到整体 timeout 或刷新页面。

4. **运行期状态与配额、trace、recovery 分散。**  
   quota 有 in-flight，run trace 提案关注事后证据，中断恢复提案关注未闭环请求。但运行还在进行时，系统没有一个统一对象回答：谁启动了它、是否可取消、取消后 quota 如何处理、部分输出是否保留、最终历史是否会落库。

5. **多渠道体验不一致。**  
   Web 可以流式看进度，IM 依赖 placeholder/progress/outbound 更新，桌面只是承载 Web 或 sidecar。没有 lifecycle contract 时，每个 surface 都只能各自发明“停止”“仍在运行”“已完成”的表现。

机会是：Hone 已经有 session lock、quota guard、progress watchdog、SSE events、public in-flight 状态和 runner task abort 的局部代码。第一版不需要重写 runner，只要把一次 `AgentSession::run()` 包装成可登记、可查询、可请求取消的 `AgentRunLifecycle`，就能显著提升用户控制感和运维确定性。

## 方案概述

新增 **Agent Run Lifecycle Control**：为每次用户可见 agent run 分配稳定 `run_id`，并维护一个运行期状态机，让 Web/public/desktop/IM 都能围绕同一对象展示、取消、后台继续和恢复。

核心对象：

- `AgentRunRecord`：一次仍在运行或刚结束的 run，包含 run_id、actor、session_id、origin、runner、started_at、phase、cancelability、quota reservation、last_progress、client_connection_state。
- `RunLifecycleState`：`queued`、`preparing`、`running`、`streaming`、`cancelling`、`cancelled`、`completed`、`failed`、`timed_out`、`detached`。
- `RunControlAction`：`detach_client`、`request_cancel`、`force_mark_failed`、`resume_stream`、`retry_after_cancel`。
- `RunCancelPolicy`：决定当前 run 是否可以取消、取消如何传播到 runner、是否保留已写入 user message、是否释放/提交 quota。

第一版目标是保守可落地：

- 客户端停止先变成 `request_cancel`，而不是只 abort fetch。
- 后端尽力取消当前 runner task；无法硬取消的 runner 至少停止继续向客户端输出，并把最终状态标记为 `cancel_requested` / `cancelled_or_detached`。
- 用户消息保留，assistant 终态写入明确的取消状态，避免历史尾部永远停在 user message。
- quota 成功 commit 的条件保持清晰：用户主动取消且无有效 assistant 输出时释放 reservation；已经完成或产生可用回复时按现有成功路径提交。
- public chat 刷新后通过 run_id 查询后台状态，而不是只靠 `in_flight` 和 last user message 推断。

## 用户体验变化

### 用户端

- Public `/chat` 的停止按钮表达为“停止本轮”，点击后立即显示“正在停止”，后端确认后显示“已停止，本轮未完成”。
- 如果页面刷新或网络断开，重新进入时看到具体状态：`后台仍在处理`、`已完成`、`已取消`、`失败`，并能跳到对应消息。
- 长运行超过阈值时，composer 区显示轻量操作：继续等待、转后台、停止。转后台不丢失运行，后台完成后通过 history / public events 回填。
- 如果取消发生在工具写入前，用户可直接编辑原问题再重发；如果已经产生部分结果，UI 保留 partial preview 并标记未完成。

### 管理端

- Admin chat pending bubble 的 stop 按钮调用同一 `POST /api/runs/:id/cancel`，而不是只清理本地 pending。
- `/sessions` 或未来 `/traces` 能看到最近 run 的 lifecycle 状态：started、detached、cancel_requested、cancelled、completed。
- 管理员可以判断“用户说点了停止但模型还在跑”到底是 cancel 不支持、runner 未响应，还是只是客户端断流。

### 桌面端

- Desktop bundled/remote 都复用 Web API 的 run lifecycle，不需要 Tauri shell 单独理解 agent runner。
- 桌面关闭窗口或切换页面时，可以选择 detach 而不是隐式中止；重新打开后恢复当前 run 状态。
- 如果 sidecar 重启导致 running run 丢失，后续由 Interrupted Run Recovery Inbox 接管，但 lifecycle record 提供更准确的最后状态。

### 多渠道

- IM 通道里的“停止”第一版不做复杂交互，只支持受控命令，例如用户回复 `/cancel` 到最近仍在运行的 direct session。
- 群聊默认不允许任意成员取消共享 session run；需要匹配触发 actor、管理员或明确 group policy。
- Placeholder/progress 更新可以显示“用户已请求停止”，避免看起来像模型自己失败。

## 技术方案

### 1. 运行期 registry

在 `hone-web-api` 或 `hone-channels` 增加一个进程内 `RunLifecycleRegistry`，第一版只管理当前进程运行中的 run：

```rust
pub struct AgentRunHandle {
    pub run_id: String,
    pub actor: ActorIdentity,
    pub session_id: String,
    pub origin: RunOrigin,
    pub state: RunLifecycleState,
    pub started_at: DateTime<Utc>,
    pub last_progress_at: DateTime<Utc>,
    pub cancel_requested_at: Option<DateTime<Utc>>,
    pub cancel_tx: tokio::sync::watch::Sender<bool>,
}
```

持久化策略分两层：

- 运行中：进程内 map，支持 cancel、status、resume 查询。
- 结束后短期：写入 session message metadata 或轻量 JSONL/SQLite，保留最近 24-72 小时，供刷新恢复、trace 和 support bundle 查询。

云/多 worker 后续可迁到 PG lease / run table；第一版不承诺跨进程取消，只要 status 明确返回 `not_found_or_process_restarted`，由 recovery 提案接管。

### 2. AgentSession 支持取消信号

给 `AgentRunOptions` 增加可选 cancellation token：

```rust
pub struct AgentRunOptions {
    pub timeout: Option<Duration>,
    pub segmenter: Option<Arc<dyn Fn(&str) -> Vec<String> + Send + Sync>>,
    pub quota_mode: AgentRunQuotaMode,
    pub model_override: Option<String>,
    pub cancellation: Option<RunCancellationToken>,
}
```

`AgentSession::run()` 在关键边界检查 token：

- 准备执行前：取消则不启动 runner，释放 quota。
- runner 执行中：`tokio::select!` 监听 cancel signal，与 timeout 同级处理。
- finalizer 前：如果 cancel 已确认且没有可用 assistant 输出，写入取消状态消息而不是空成功 fallback。
- artifact sync / company profile sync 前：若取消发生在 runner 之后，需要根据是否已有完整成功结果决定是否继续同步；第一版保守地只在 `response.success=true` 且未取消时执行副作用同步。

对具体 runner：

- ACP / opencode：当前已有一些 task abort 逻辑，可先通过 `tokio::select!` abort child task；后续再补 ACP 协议级 cancel（如果 runner 支持）。
- function-calling / LLM provider：若 HTTP request 无法中断，至少停止等待结果并标记 cancel；底层 request 自然结束后不得再写入用户可见 session。
- skill script：若有 child process handle，后续可接入 kill；第一版可标记 `cancel_requested_runner_may_continue`。

### 3. Web API

新增 API：

- `GET /api/runs/current?actor=...`
- `GET /api/runs/:run_id`
- `POST /api/runs/:run_id/detach`
- `POST /api/runs/:run_id/cancel`

Public API：

- `GET /api/public/runs/current`
- `GET /api/public/runs/:run_id`
- `POST /api/public/runs/:run_id/detach`
- `POST /api/public/runs/:run_id/cancel`

Public 路由只能访问当前 cookie actor 的 run；不能传任意 actor。Admin 路由可按 actor 查询，但 cancel 需要 admin 权限。

`POST /api/public/chat` 的 SSE `run_started` payload 增加 `run_id`：

```json
{ "runner": "opencode_acp", "run_id": "run_20260603_x7k9", "text": "" }
```

客户端收到 `run_id` 后，stop 按钮调用 cancel endpoint；若 SSE 断开但用户选择后台继续，调用 detach 或直接让服务端记录 `client_connection_state=detached`。

### 4. 前端模型

Public chat：

- `pendingAssistantMessage` 增加 `runId`、`cancelState`、`detached`。
- stop 按钮从 `activeController.abort()` 改为先调用 cancel endpoint；只有 cancel endpoint 不可达时才本地 abort 并展示“连接已断开，后台状态未知”。
- `backgroundPending` 从 `{ since }` 升级为 `{ runId, since, phase }`。
- restore 过程先查 `/api/public/runs/current`，再读 history；避免只靠 `in_flight` 推断。

Admin chat：

- `PendingState` 增加 run id 与 cancel state。
- `onStopPending` 走 run cancel API。
- terminal 状态显示 `cancelled` 与 `detached`，不混入 `error`。

共享前端 helper：

- `packages/app/src/lib/run-lifecycle.ts`
- `packages/app/src/pages/chat.tsx`
- `packages/app/src/components/chat-view.tsx`

### 5. Quota 与 session 规则

建议明确以下行为：

- 用户 message 一旦接收并通过校验，继续按现有逻辑尽早持久化。
- run 在 `preparing` 前取消：释放 quota reservation，不写 assistant 正文，只写可选 system/cancel marker。
- run 在 runner 已产生完整可用回答后取消：如果 finalizer 已完成，视为 completed；用户取消只作为 late cancel ignored。
- run 已产生 partial stream 但未完成：写入 assistant cancel marker，partial 内容可选择不落库或以 metadata 保留给 trace，默认不作为正式 assistant answer。
- scheduled task / heartbeat 不接受普通用户 cancel；只能由 admin 或任务 owner 停用 job。用户取消只针对当前交互 run。

这能避免“取消后还扣额度”“取消后又突然出现完整回答”“取消导致历史尾部失真”三类体验问题。

## 实施步骤

### Phase 1: Run id and status visibility

- 增加 run id 生成与进程内 registry。
- `POST /api/chat` 和 `POST /api/public/chat` 的 `run_started` 返回 run id。
- 增加 current/status API。
- Public chat 的 background pending 使用 run status，而不是只靠 `in_flight` 推断。
- 不改变 runner 取消行为，只建立可见状态。

### Phase 2: Cooperative cancel

- `AgentRunOptions` 增加 cancellation token。
- `AgentSession::run()` 在 prepare、runner select、finalizer 前检查 cancel。
- Web/admin/public stop 按钮调用 cancel endpoint。
- session 和 quota 对 cancel 做明确终态处理。

### Phase 3: Runner-specific hardening

- ACP runners 尽量接入协议级 cancel 或 child task abort。
- skill script / tool execution 增加可选 cancel hook。
- 对无法中断的 provider 调用标记 `cancel_requested_runner_may_continue`，并阻止迟到结果写入用户可见历史。

### Phase 4: Multi-channel command support

- Direct IM 支持 `/cancel` 最近一个本 actor running run。
- Group chat cancel 需要触发者匹配或管理员权限。
- Placeholder/progress 更新展示 cancel state。
- 与 Run Trace Workbench 链接 run_id / trace_id，与 Interrupted Recovery 在进程重启后衔接。

## 验证方式

- Rust 单元测试：
  - registry register/status/cancel/finish 状态转换正确。
  - public actor 不能查询或取消其它 actor 的 run。
  - `AgentSession::run()` 在 prepare 前取消会释放 quota reservation。
  - runner 执行中 cancel 进入 `cancelled`，不会把迟到 response 写成正常 assistant answer。
  - 已完成 run 的 late cancel 返回 `already_finished`。

- Web/API 测试：
  - SSE `run_started` 包含 run id。
  - `GET /api/public/runs/current` 在 active、detached、completed、not_found 四种状态返回稳定 payload。
  - cancel endpoint 对非 owner / 非 admin 拒绝。

- 前端测试：
  - public chat stop 从 pending -> cancelling -> cancelled。
  - 刷新页面后 background pending 能从 run status 恢复。
  - cancel 后 composer 解锁，用户能编辑并重发。
  - 管理端 pending bubble 能区分 error、timeout、cancelled。

- 手工验收：
  - 用长时间 research / tool run 触发 public chat，点击停止后后端 run 状态变为 cancelled，历史中不出现完整迟到答案。
  - 刷新页面时后台 run 仍在执行，页面恢复为同一 run id 的 pending 状态，完成后自动拉到结果。
  - 断开 SSE 但不 cancel，后台完成后 history restore 能看到最终答案。
  - 模拟后端重启，current run status 返回 not found，并由 recovery 机制处理未闭环 session。

- 指标：
  - cancel 请求到 UI 终态 p95 < 2s。
  - cancel 后迟到 assistant answer 写入率为 0。
  - background restore 成功率、detached run 完成率、cancelled run quota release 正确率可从 trace / product event 后续接入。

## 风险与取舍

- **风险：runner 无法真正中断底层 provider 调用。** 取舍：第一版先保证用户可见状态和 session 写入边界正确；硬取消按 runner 能力逐步增强。
- **风险：取消语义被误用为规避额度。** 取舍：只有无完整有效回答的用户主动取消才释放 quota；已经完成或产生有效终态的 run 仍按现有规则提交。
- **风险：多进程 / 云 worker 下进程内 registry 不够。** 取舍：v1 明确只支持当前 web process 内取消；跨进程取消等待 cloud worker lease plane 或 PG run table，状态缺失时进入 recovery。
- **风险：partial 输出处理引起争议。** 取舍：partial stream 默认不作为正式 answer 落库；若需要保留，仅进入 admin trace/support evidence，不在用户历史中伪装成完成回答。
- **风险：IM `/cancel` 取消错对象。** 取舍：只允许取消当前 actor 最近一个 active direct run；群聊默认不开放，直到 group policy 明确。
- **不做范围：** 不实现通用 workflow pause/resume，不做多 run 并发队列，不替代 Run Trace Workbench，不替代 Interrupted Run Recovery Inbox，不把 cancel 扩展到所有后台 cron/event-engine 任务。

## 与已有提案的差异

查重范围覆盖 `docs/proposal/` 与 `docs/proposals/` 下全部现有提案，并重点阅读 / 检索了 `run trace`、`interrupted`、`recovery`、`clarification`、`permission`、`mutation`、`automation`、`runtime readiness`、`circuit breaker`、`journey replay`、`desktop alert`、`cancel`、`background`、`progress`、`resume` 等关键词。

- 不重复 `auto_p1_run_trace_workbench.md`：Run Trace 关注一次 run 结束后的证据聚合和排障；本提案关注运行进行中如何查询、停止、转后台和恢复。
- 不重复 `auto_p1_interrupted-run-recovery-inbox.md`：Recovery Inbox 处理进程重启、runner 卡死或 stale row 造成的未闭环请求；本提案处理正常运行期间的用户控制，并在进程丢失后把缺口交给 recovery。
- 不重复 `auto_p1_agent-clarification-queue.md`：Clarification Queue 处理 agent 需要用户补充信息或确认的问题；本提案处理用户主动控制当前 run 生命周期。
- 不重复 `auto_p1_agent-permission-broker.md`：Permission Broker 处理工具/文件/外部动作权限；本提案只管理 run 是否继续，不授权具体工具。
- 不重复 `auto_p1_runtime-dependency-circuit-breaker.md`：Circuit Breaker 在依赖故障趋势中阻止或降级新调用；本提案允许用户控制已经开始的 run。
- 不重复 `auto_p1_automation_intent_control_plane.md`：Automation Intent 管理任务变更草稿和确认；本提案不改变 cron job 定义，只处理当前 agent run。

查重结论：现有提案覆盖了运行后追踪、中断后恢复、权限确认和依赖熔断，但没有覆盖“用户在一次长 agent run 进行中如何可靠停止、转后台、刷新恢复，并保证 quota/session/迟到输出语义一致”的产品和架构契约。因此本主题是新的、可落地的 P1 提案。

本轮只新增 proposal，不开始实施，不修改业务代码、测试代码、运行配置或 `docs/current-plan.md`。该任务属于定期产品/架构提案产出，未进入执行态，因此无需把计划落盘到 `docs/current-plans/`，也无需归档计划页。若后续开始实施，应按动态计划准入标准新增或复用 `docs/current-plans/agent-run-lifecycle-control.md`，并在引入 run registry、cancel API、AgentRunOptions cancellation、quota/session 取消语义或多渠道 `/cancel` 后同步更新 `docs/repo-map.md`、`docs/invariants.md`、相关 runbook 和必要的 decision/ADR。
