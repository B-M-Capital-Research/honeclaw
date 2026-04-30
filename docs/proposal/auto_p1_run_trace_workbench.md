# Proposal: Run Trace Workbench for Agent Reliability

- status: proposed
- priority: P1
- created_at: 2026-05-01 05:02 +0800
- owner: automation
- related_files:
  - `README.md`
  - `AGENTS.md`
  - `docs/repo-map.md`
  - `docs/invariants.md`
  - `docs/decisions.md`
  - `docs/current-plan.md`
  - `docs/proposal/auto_p1_delivery_decision_loop.md`
  - `docs/proposal/auto_p1_linked-user-workspace.md`
  - `docs/proposals/desktop-bundled-runtime-startup-ux.md`
  - `docs/proposals/skill-runtime-multi-agent-alignment.md`
  - `crates/hone-channels/src/agent_session/core.rs`
  - `crates/hone-channels/src/agent_session/emitter.rs`
  - `crates/hone-channels/src/core/logging.rs`
  - `crates/hone-channels/src/execution.rs`
  - `crates/hone-channels/src/prompt_audit.rs`
  - `crates/hone-channels/src/run_event.rs`
  - `memory/src/llm_audit.rs`
  - `memory/src/session_sqlite.rs`
  - `crates/hone-web-api/src/routes/logs.rs`
  - `crates/hone-web-api/src/routes/llm_audit.rs`
  - `crates/hone-web-api/src/routes/task_runs.rs`
  - `crates/hone-web-api/src/routes/history.rs`
  - `packages/app/src/pages/logs.tsx`
  - `packages/app/src/pages/llm-audit.tsx`
  - `packages/app/src/pages/task-health.tsx`
  - `packages/app/src/pages/sessions.tsx`
  - `packages/app/src/lib/log-refs.ts`

## 背景与现状

Hone 已经不是单一路径的聊天应用。一次用户请求可能从 Web、Feishu、Telegram、Discord、iMessage 或桌面 bundled runtime 进入，经过 `ingress`、session lock、prompt/skill turn 构建、runner 准备、ACP 或 function-calling 执行、tool 调用、响应 finalizer、session persistence，再回到对应 channel。定时任务和 heartbeat 还会走 `ExecutionMode::TransientTask`，复用同一套 execution preparation。

当前代码里已经有不少可靠性证据源：

- `crates/hone-channels/src/core/logging.rs` 已把消息流整理成 `[MsgFlow/<channel>] received -> step -> finished/failed`，并带有 `message_id`、`state`、user、session、elapsed、tools 等字段。
- `crates/hone-channels/src/agent_session/emitter.rs` 会把 runner progress、tool status、runner error 也写入结构化日志。
- `crates/hone-channels/src/agent_session/core.rs` 已有 `agent.run.progress` watchdog，长时间运行时能持续记录 runner 仍在执行，而不是静默卡住。
- `crates/hone-channels/src/execution.rs` 集中写 prompt-audit、创建 tool registry、runner、actor sandbox-backed request，是 session 与 transient task 的共同前置层。
- `crates/hone-channels/src/prompt_audit.rs` 会把 system prompt 与 runtime input 写到 `data/runtime/prompt-audit/<channel>/...json`。
- `memory/src/llm_audit.rs` 已用 SQLite 保存 LLM 请求、响应、错误、tokens、latency，并由 `/api/llm-audit` 和 `packages/app/src/pages/llm-audit.tsx` 展示。
- `/api/logs` 与 `packages/app/src/pages/logs.tsx` 能合并内存日志和 runtime log tail；`packages/app/src/lib/log-refs.ts` 已能从日志中提取 actor、session、task 反向跳转。
- `/api/admin/task-runs` 与 `packages/app/src/pages/task-health.tsx` 能读取 task run jsonl，展示周期任务 24h 健康状态。
- `/api/history` 与 session storage 能展示最近会话，还能识别 compact boundary、compact summary、tool transcript 与 local image artifact。

这些能力说明 Hone 已经具备“可观测原料”，但产品上仍是多个孤立页面：日志、LLM 审计、任务健康、会话历史、prompt-audit 文件、channel 侧 message id 需要维护者手工 grep 和来回跳转。

## 问题或机会

AI agent 产品的关键竞争力不只是“能跑”，而是能解释一次运行为什么这样跑、卡在哪里、用了哪些上下文、哪些 tool 影响了结论、失败后能否快速复盘。Hone 当前最容易被运维和用户感知的可靠性问题通常跨多个证据源：

- IM 用户说“刚才那条消息没回”，维护者要从 channel message id 找日志，再找 session，再看 LLM audit 或 prompt-audit。
- agent 回答质量异常时，用户只看到最终答案；维护者需要手工拼出 system prompt、runtime input、skill expansion、tool calls 和 compact restore 状态。
- 定时任务失败时，`task-health` 知道 task failed，但不自然联到对应 transient execution、runner events、LLM audit 和 outbound 结果。
- 桌面端 channel 状态能显示进程和日志，但还没有一个“最近失败运行”的可操作入口。
- 多 runner 切换、ACP transcript restore、session compaction、skill activation 都在活跃重构中；如果没有统一 trace，对后续 regression 判断会越来越依赖个人经验。

这值得列为 P1，因为它显著提升核心链路稳定性、排障效率和用户信任，而且不要求重写 agent runtime。它可以先把已有事实源串成只读工作台，再逐步补齐稳定 correlation id 与 replay 包。

## 方案概述

新增“Run Trace Workbench”：以一次 agent run 为核心对象，把分散在日志、LLM audit、prompt-audit、session history、task runs、channel message id 中的证据聚合成一条可浏览、可分享、可导出的运行时间线。

核心对象建议叫 `RunTrace`：

- `trace_id`：一次用户消息或一次 transient task 的稳定追踪 id。
- `actor` / `session_identity` / `session_id` / `channel_target`。
- `origin`：web_chat、public_chat、feishu、telegram、discord、imessage、cron、heartbeat、desktop_sidecar。
- `channel_message_id` / `placeholder_message_id` / `task_run_id`，按来源可选。
- `runner` / `model` / `execution_mode` / `sandbox_root`。
- `timeline`：received、dedup、pretrigger flush、session lock、quota、prompt build、prompt audit written、runner start、tool status、LLM call、stream delta summary、finalizer、persisted、outbound sent、failed。
- `evidence_links`：session history、LLM audit record ids、prompt-audit file path、log file/line-ish anchor、task run jsonl record。
- `outcome`：ok、user_visible_error、runner_error、timeout、empty_success_fallback、outbound_failed、filtered_or_deduped、cancelled。

一期目标不是复杂 APM，而是让维护者从一个页面回答四个问题：

1. 这条请求有没有真正进入 agent 主链路？
2. 它使用了什么 prompt、runner、model、skill 和 tool？
3. 它在哪里失败或变慢？
4. 用户最终看到了什么，session 又落了什么？

## 用户体验变化

用户端：

- Public Web `/chat` 可在出错时展示一个短 trace reference，例如“本次运行编号已记录”，用户不需要看到内部 prompt 或 file path。
- `/me` 或用户侧问题反馈入口可以附带最近几次失败 trace id，降低用户描述成本。
- 普通用户默认不看完整 trace，避免泄露 sandbox path、prompt 或内部模型配置。

管理端：

- 新增 `/traces` 或在现有 `/logs` 增加 `Runs` tab，默认按最近时间列出每次 agent run。
- 列表展示 channel、actor、session、runner、耗时、状态、tool 数、LLM 调用数、token 用量、是否 compact、是否 outbound failed。
- 详情页按时间线展示阶段，点击阶段可展开对应日志、LLM audit request/response 摘要、prompt-audit 摘要和 session 写入结果。
- 从 `/logs`、`/llm-audit`、`/task-health`、`/sessions` 反向跳到同一个 trace，减少人工串联。
- 提供 `Export support bundle`，导出脱敏 JSON：trace metadata、timeline、错误、tokens、工具名、prompt hash、相关文件路径，不默认导出完整 prompt 和用户输入。

桌面端：

- Desktop bundled 模式的 dashboard 或 channel status 增加“最近失败运行”区块，点击打开 Web console trace。
- 当 sidecar 或 channel 失败时，用户看到的是“Feishu 最近 1 次运行失败，可打开诊断”，而不是只看滚动日志。

多渠道：

- Feishu / Telegram / Discord 的内部错误回复可附带短 trace reference，便于用户转给维护者。
- 群聊场景中，trace 仍按 `SessionIdentity` 串起共享上下文，同时保留当前触发 actor，避免把 group chat 当成个人 direct session。

## 技术方案

### 1. 引入轻量 `trace_id`，不替换现有 id

在 `AgentSession::run()` 和 transient scheduler execution 入口生成 `trace_id`，建议格式为短 UUID 或 `run_<timestamp>_<rand>`。它不是 session id、message id 或 task run id 的替代，而是本次运行的 correlation key。

传递范围：

- `AgentSession`、`ExecutionRequest`、`AgentRunnerRequest` 增加 optional `trace_id`。
- `log_message_received/step/finished/failed` 和 runner emitter 日志的 structured fields 增加 `trace_id`。
- `LlmAuditRecord.metadata` 写入 `trace_id`、runner、execution_mode、message_id、task_run_id。
- prompt-audit payload 写入 `trace_id`，文件名可以继续按 timestamp + session slug，避免迁移旧路径。
- session message metadata 写入 `trace_id`，便于 `/api/history` 从 transcript 反查。
- task run jsonl 记录写入 `trace_id`，便于 `task-health` 跳转。

兼容策略：旧记录没有 `trace_id` 时，API 可以用 session_id + time window + message_id best-effort 关联，但 UI 必须标注为 inferred。

### 2. 建立只读 trace 聚合 API

新增 `crates/hone-web-api` 路由：

- `GET /api/traces?actor=&session_id=&status=&origin=&from=&to=`
- `GET /api/traces/:trace_id`
- `GET /api/traces/:trace_id/export?redacted=true`

一期聚合来源：

- runtime logs：按 structured `trace_id` 优先，fallback 到 session/message id。
- LLM audit SQLite：按 `metadata.trace_id` 或 session/time window。
- prompt-audit JSON：按 payload `trace_id` 或 latest session audit。
- session storage：按 message metadata `trace_id` 或 session tail。
- task runs：按 `trace_id` 或 task/time window。

聚合 API 不应把所有原始内容默认返回。建议分层：

- list 只返回 metadata 和 outcome。
- detail 返回 timeline、错误、tokens、tool names、prompt hash、audit ids。
- 需要完整 prompt / request_json 时，沿用 `/api/llm-audit/:id` 和受控 prompt-audit 读取权限。

### 3. 将日志页面升级为实体化时间线

当前 `packages/app/src/lib/log-refs.ts` 已能从日志中提取 actor/session/task。可以扩展为：

- 识别 `trace_id` 并生成 `EntityRefLink kind="trace"`。
- `/logs` 仍保留原始流式视图，但新增“按运行分组”视图。
- `/llm-audit` 详情里展示所属 trace link。
- `/task-health` 每条 failed run 旁边展示 trace link。
- `/sessions` 或 chat shell 的消息 metadata 若包含 trace id，可在管理员视图中显示诊断入口。

前端数据模型放在：

- `packages/app/src/lib/traces.ts`
- `packages/app/src/context/traces.tsx`
- `packages/app/src/pages/traces.tsx`

### 4. 隐私、脱敏与权限

Run trace 会碰到高敏内容：用户输入、system prompt、runtime input、tool result、本地文件路径、sandbox path、API 响应。默认策略应保守：

- Public 用户只能看到自己的简短 trace reference 和用户可理解状态，不开放原始 prompt。
- Admin 才能看完整 trace detail。
- Export 默认 redacted：用户输入截断、prompt 只给 hash/长度/skill ids，路径改写为 sandbox-relative，LLM response 只给摘要和错误。
- 完整 prompt 和 request_json 仍需要显式展开，并沿用现有本地 admin 能力边界。
- 不把 trace 数据上传到外部 SaaS；本提案只做本地工作台。

### 5. 与运行时重构保持边界

本提案不改变 runner 选择、ACP 协议、skill runtime 或 session persistence 语义。它只要求这些路径在关键阶段带上同一个 `trace_id`，并把已有证据归档到同一索引。

对活跃任务的影响：

- ACP runtime refactor：trace 能帮助验证 transcript restore、compact boundary、empty success retry、timeout fallback 是否按预期发生。
- Skill runtime 对齐：trace 能看到某 turn 发现了哪些 skill、真正注入了哪个 skill、tool allowlist 是否匹配。
- Feishu direct placeholder 修复：trace 能串起 message_id、placeholder update、agent run 和 outbound final send。
- Canonical config/runtime apply：trace 能暴露某次 run 实际 runner/model/config_path，而不是只看设置页期望值。

## 实施步骤

### Phase 1: Correlation id 与只读聚合

- 在 session run 和 transient task run 入口生成 `trace_id`。
- 将 `trace_id` 写入 MsgFlow structured logs、runner emitter logs、LLM audit metadata、prompt-audit payload、session message metadata、task run records。
- 增加 `GET /api/traces` 和 `GET /api/traces/:id`，先聚合 logs + LLM audit + session history。
- 在 `/logs` 和 `/llm-audit` 增加 trace link。

### Phase 2: 管理端 Run Trace Workbench

- 新增 `/traces` 页面，列表按最近运行排序。
- 详情页展示阶段时间线、慢阶段、错误、tool/LLM 摘要、相关 session 和 audit links。
- `/task-health` failed rows 和 `/sessions` admin message 入口接入 trace。
- 增加 redacted support bundle 导出。

### Phase 3: 质量指标与回归门禁

- 统计 p50/p95 run latency、runner timeout、empty-success retry、outbound failed、tool error、LLM failed、compact triggered。
- 增加本地 regression：模拟一次 successful run、一次 runner error、一次 transient task failed，验证 trace 能串到对应证据。
- 在 dashboard 加入最近 24h run health，不替代 task-health，只展示 agent execution 层指标。

## 验证方式

- Rust 单元测试：
  - `trace_id` 生成稳定非空，且不会改变现有 session id / message id 语义。
  - LLM audit metadata、prompt-audit payload、session message metadata、task run record 都能序列化并保留 `trace_id`。
  - trace 聚合 API 在完整 trace、缺少 LLM audit、缺少 prompt-audit、旧记录 fallback 四种情况下返回 degraded timeline 而不是 500。
- Web API / 前端测试：
  - `/api/traces` 支持 actor、session、status、time range 过滤。
  - `/logs`、`/llm-audit`、`/task-health` 的 trace link 数据转换有单元测试。
  - `bun run test:web` 覆盖 `traces.ts` 与 `log-refs.ts` 的 trace 提取。
- 手工验收：
  - Web 发送一次普通消息，能在 `/traces` 找到 received -> runner start -> LLM call -> finished -> persisted。
  - 触发一次失败 runner 或超时，trace 详情能定位失败阶段和错误来源。
  - 触发一次 cron/heartbeat transient run，能从 task-health 跳到 trace。
  - 桌面 bundled 模式下，最近失败运行入口能打开同一 trace。
- 指标：
  - 维护者定位“消息没回 / 回答异常 / 任务失败”的平均跳转次数减少到 1 次。
  - 90% 以上新运行具备显式 `trace_id`；旧运行只在 fallback 场景标记 inferred。

## 风险与取舍

- 风险：trace 可能收集过多敏感内容。取舍：默认 redacted，完整 prompt/LLM request 继续走 admin-only 明确展开。
- 风险：为所有路径传 `trace_id` 会扩大接口改动面。取舍：先 optional 字段、向后兼容；旧代码不带 trace 也能运行。
- 风险：聚合多个文件和 SQLite 查询可能拖慢页面。取舍：list 只读轻量 metadata，detail 再按 trace id 懒加载；日志 fallback 限定时间窗口。
- 风险：support bundle 被误当成云端遥测。取舍：明确只做本地导出，不引入外部上报。
- 风险：timeline 过细会让非技术用户困惑。取舍：完整 Workbench 只面向 admin；public 只显示短 reference 和简短状态。
- 不做：不改事件投递决策、不做跨用户 workspace 合并、不改变 runner/ACP/skill runtime 语义、不引入外部 observability SaaS、不把 trace 作为长期投资研究记忆。

## 与已有提案的差异

查重范围：

- `docs/proposal/auto_p1_delivery_decision_loop.md`
- `docs/proposal/auto_p1_linked-user-workspace.md`
- `docs/proposals/desktop-bundled-runtime-startup-ux.md`
- `docs/proposals/skill-runtime-multi-agent-alignment.md`

差异结论：

- 与 `auto_p1_delivery_decision_loop.md` 不重复：该提案聚焦 event-engine `delivery_log`、NotificationPrefs 和主动通知偏好闭环；本提案聚焦一次 agent run 的端到端执行追踪，覆盖 chat、cron、runner、prompt、LLM audit、session、outbound。
- 与 `auto_p1_linked-user-workspace.md` 不重复：该提案新增真实用户跨渠道 workspace 抽象；本提案不合并用户资产，只为单次运行建立诊断 correlation。
- 与 `desktop-bundled-runtime-startup-ux.md` 不重复：该提案处理桌面启动锁、旧进程接管和组件恢复；本提案处理运行后的可观测性与复盘入口。
- 与 `skill-runtime-multi-agent-alignment.md` 不重复：该提案讨论 skill schema、调用语义、multi-agent 对齐；本提案只把 skill 是否发现、注入、执行记录进 trace，作为后续排障证据。

本轮选择该主题，是因为当前仓库已经有日志、审计、prompt-audit、task-health 和 session 证据源，但缺少统一运行对象。Run Trace Workbench 能直接降低运维排障成本，并为正在进行的 ACP runtime、skill runtime、Feishu direct 和 canonical config 工作提供共同验证面。
