# Proposal: Usage Entitlement Ledger for Conversion and Cost Control

status: proposed
priority: P1
created_at: 2026-05-01 17:03:06 CST
owner: automation

## related_files

- `README.md`
- `AGENTS.md`
- `docs/repo-map.md`
- `docs/invariants.md`
- `docs/decisions.md`
- `docs/current-plan.md`
- `docs/proposal/auto_p1_delivery_decision_loop.md`
- `docs/proposal/auto_p1_evidence_review_queue.md`
- `docs/proposal/auto_p1_linked-user-workspace.md`
- `docs/proposal/auto_p1_run_trace_workbench.md`
- `docs/proposals/desktop-bundled-runtime-startup-ux.md`
- `docs/proposals/skill-runtime-multi-agent-alignment.md`
- `config.example.yaml`
- `crates/hone-core/src/config/agent.rs`
- `crates/hone-core/src/audit.rs`
- `memory/src/quota.rs`
- `memory/src/web_auth.rs`
- `memory/src/llm_audit.rs`
- `memory/src/cron_job/types.rs`
- `memory/src/cron_job/storage.rs`
- `crates/hone-channels/src/agent_session/core.rs`
- `crates/hone-web-api/src/routes/public.rs`
- `crates/hone-web-api/src/routes/web_users.rs`
- `packages/app/src/pages/chat.tsx`
- `packages/app/src/pages/settings.tsx`
- `packages/app/src/lib/types.ts`

## 背景与现状

Honeclaw 已经具备从开源本地工具向可运营产品演进的基础：

- Public Web 使用邀请码和手机号登录，`memory/src/web_auth.rs` 把 invite user、密码 hash、TOS、HttpOnly session 保存在 SQLite 中。
- Web 用户被映射为 `ActorIdentity::new("web", user_id, None)`；`crates/hone-web-api/src/routes/public.rs` 会在 `to_public_auth_user` 中返回 `daily_limit`、`success_count`、`in_flight` 和 `remaining_today`。
- `memory/src/quota.rs` 现在按 actor + 北京日期写 JSON quota 文件，`AgentSession::reserve_conversation_quota` 在用户对话开始时预留，成功回答后 commit，失败时 release；admin actor 与 scheduled task 会绕过这条日对话限制。
- `config.example.yaml` 和 `crates/hone-core/src/config/agent.rs` 只有一个全局 `agent.daily_conversation_limit`，默认 12，`0` 表示不限。
- 管理端 Settings 的 Web 邀请码表只展示同一个全局日额度与剩余次数；public chat composer 只在额度为 0 时禁用发送并提示“今日额度已用完”。
- Cron 任务有独立的硬上限：`memory/src/cron_job/types.rs` 规定每个 actor 最多 12 个启用任务，但这不是可配置的产品 plan。
- LLM audit 已经记录 provider/model/latency/prompt_tokens/completion_tokens/total_tokens，适合作为成本观测原料；但它和用户额度、渠道、任务、附件、通知没有被组织成一张 usage ledger。

这说明 Hone 已有“限流”和“审计”能力，但还没有“权益”产品层。当前限制是单点的、全局的、按 actor 分散落盘的，无法表达试用、付费、团队、运营赠送、渠道差异、任务额度、token 成本或滥用风控。

## 问题或机会

Hone 的核心成本和体验不是只有一次 chat。一个真实用户可能同时消耗：

- 公共 Web 对话次数和附件上传处理。
- 多渠道 IM 对话和群聊触发。
- 定时任务、heartbeat、portfolio monitoring 和主动通知。
- LLM token、search/data API、图表生成、公司画像读写和长期 session compaction。

目前这些消耗没有一个统一视图，会带来几类问题：

- 用户端：用户只看到“今日剩余次数”，不知道权益为什么耗尽、明天何时恢复、哪些能力仍可用、如何升级或联系获取更多额度。
- 管理端：运营只能在邀请码表看当前 Web actor 剩余次数，无法按用户、渠道、功能、模型、token 或任务类型判断成本与留存。
- 桌面端：本地 desktop/bundled runtime 面向单用户很自然应不限或高限额，但 remote/public 部署需要严格控制成本；当前只能靠同一个全局配置分流。
- 多渠道：Telegram/Feishu/Discord actor 与 Web invite user 是不同 actor，既不共享商业权益，也不能被统一评估滥用风险；linked workspace 提案落地前更缺一个可独立演进的 entitlement 基础。
- 商业化：邀请制、手机号、TOS、public chat、日额度已经接近试用产品形态，但缺少 plan、trial、overage、grant、usage event、admin 调整和升级提示，无法支撑真实转化。

这值得列为 P1：它直接影响成本控制、转化路径、留存体验和运维判断，而且可以先以本地 ledger + plan 配置落地，不依赖接入 Stripe/支付网关，也不需要改变 runner 或 channel 主链路。

## 方案概述

新增一个 Usage Entitlement Ledger，把“用户拥有哪些权益”和“每次运行消耗了什么”从单一 daily quota 升级为可审计、可解释、可扩展的产品层。

核心对象：

- `EntitlementPlan`：定义某类用户的能力包，例如 trial、community、pro-local、ops-admin。
- `EntitlementGrant`：把 plan 或一次性赠送授予某个 actor，后续可迁到 workspace。
- `UsageEvent`：记录一次能力消耗或预留，例如 chat_success、chat_in_flight、scheduled_run、llm_tokens、attachment_upload、notification_delivery。
- `UsageSnapshot`：按 actor/date/period 汇总当前可展示额度。
- `EntitlementDecision`：在请求进入主链路前给出 allow / deny / warn / bypass，以及用户可读原因。

一期目标不做支付，只把现有全局 quota 迁到可配置的 plan/grant/ledger 模型，并把 public/admin UI 从“剩余次数”升级为“今日/本周期权益状态”。后续再接外部支付、团队 seat、workspace-level entitlement。

## 用户体验变化

### 用户端

- Public `/chat` 顶部从“剩余 N/12”升级为一个简洁权益条：今日对话、任务额度、附件额度和恢复时间。
- 额度不足时，不只禁用发送，还解释是哪类权益不足：对话次数、并发运行、附件大小、定时任务数量或后台任务额度。
- 用户看到明确的下一步：明日恢复、联系管理员、输入兑换码、升级计划或切换到本地桌面 unlimited 模式。
- 对 trial 用户，最后 2 次对话时给出轻量提醒，但不打断当前投资研究流程。

### 管理端

- Settings 的 Web 邀请码表增加 plan、period usage、last 7d cost、token estimate、manual grant 和 revoke/extend 操作。
- 新增 Usage/Entitlements 页面：按 actor/channel/plan/feature/model 查看用量，支持筛选“高成本用户”“额度快耗尽”“失败但消耗 token 的运行”“scheduled task bypass 占比”。
- 管理员可以给某个 invite user 增加一次性 grant，例如“本周额外 20 次 chat”或“临时开放 30 天 pro plan”，并保留 audit log。
- 当用户反馈“额度不对”时，管理员能看到每次扣减来自哪个 session、runner、task 或 API。

### 桌面端

- Bundled local mode 默认显示 “Local plan”，可不限制 chat，但仍展示本地 LLM/token 用量和 scheduled task 数量，帮助用户理解资源消耗。
- Remote mode 显示当前 backend 返回的 entitlement，而不是假设本地无限。
- Desktop onboarding 可以把“本地免费运行”和“公共服务额度”解释清楚，降低用户把两种部署模式混淆的概率。

### 多渠道

- Feishu / Telegram / Discord / iMessage 在被 quota 拒绝时返回同一类可读原因，而不是每个渠道各自拼接错误文案。
- Scheduled task 继续可以走 bypass，但 ledger 必须记录 `usage_event.kind=scheduled_run` 和 `decision=bypassed`，让运维知道后台成本来自哪里。
- 群聊触发可先按 actor 计费；后续 workspace proposal 落地后，再把 plan 移到 workspace 或 group workspace。

## 技术方案

### 1. 新增 entitlement 存储

建议在 `memory` 下新增 SQLite 存储，例如 `memory/src/entitlement.rs`，复用本地 WAL 模式：

```text
entitlement_plans (
  plan_id TEXT PRIMARY KEY,
  display_name TEXT NOT NULL,
  status TEXT NOT NULL,
  limits_json TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
)

entitlement_grants (
  grant_id TEXT PRIMARY KEY,
  actor_channel TEXT NOT NULL,
  actor_user_id TEXT NOT NULL,
  actor_scope TEXT,
  plan_id TEXT NOT NULL,
  starts_at TEXT NOT NULL,
  expires_at TEXT,
  grant_reason TEXT,
  created_by TEXT,
  created_at TEXT NOT NULL,
  revoked_at TEXT
)

usage_events (
  event_id TEXT PRIMARY KEY,
  actor_channel TEXT NOT NULL,
  actor_user_id TEXT NOT NULL,
  actor_scope TEXT,
  feature TEXT NOT NULL,
  amount INTEGER NOT NULL,
  unit TEXT NOT NULL,
  decision TEXT NOT NULL,
  session_id TEXT,
  message_id TEXT,
  task_id TEXT,
  runner TEXT,
  model TEXT,
  metadata_json TEXT,
  occurred_at TEXT NOT NULL
)
```

`limits_json` 第一版可以支持：

- `chat.daily_success_limit`
- `chat.concurrent_limit`
- `cron.enabled_job_limit`
- `attachments.daily_upload_limit`
- `attachments.max_file_mb`
- `scheduled.daily_run_limit`
- `llm.monthly_token_soft_limit`

默认迁移策略：如果没有任何 plan/grant，系统创建一个 `default` plan，其 `chat.daily_success_limit` 来自当前 `agent.daily_conversation_limit`，cron 上限保持 12，从而兼容现有部署。

### 2. 从 ConversationQuotaStorage 演进到决策层

不要一次性删除 `memory/src/quota.rs`。建议先加一个 `EntitlementService`：

- `decide(actor, feature, amount, context) -> EntitlementDecision`
- `reserve(actor, feature, amount, context) -> UsageReservation`
- `commit(reservation)`
- `release(reservation)`
- `snapshot(actor, period) -> UsageSnapshot`

`AgentSession::reserve_conversation_quota` 可先改为调用 service；service 内部第一版仍可写旧 quota JSON + 新 usage_events 双写。稳定后再让新 ledger 成为 source of truth。

关键兼容约束：

- `daily_conversation_limit=0` 仍表示 chat unlimited，但 usage_events 继续记录 amount 和 decision。
- admin actor 仍 bypass，但要记录 `decision=bypassed_admin`。
- scheduled task 仍不消耗 chat quota，但要记录 `feature=scheduled_run`。
- 失败运行不扣 chat success quota，但如果产生 LLM audit token，可记录 token usage，帮助成本排查。

### 3. 关联 LLM audit 与 usage event

`memory/src/llm_audit.rs` 已保存 token columns。建议在写入 LLM audit 后同步生成或异步归集 usage event：

- `feature=llm_tokens`
- `amount=total_tokens`
- `unit=tokens`
- metadata 包含 provider、model、source、operation、success、latency_ms、audit_record_id。

如果 runner 未返回 token，仍可记录 `amount=0` + `metadata.missing_usage=true`，避免管理端误以为没有成本。

后续若 run trace proposal 落地，可把 `trace_id` 一并写入 usage event；本提案不依赖它。

### 4. API 与权限

新增 admin API：

- `GET /api/entitlements/plans`
- `GET /api/entitlements/actors/:actor/snapshot`
- `POST /api/entitlements/actors/:actor/grants`
- `POST /api/entitlements/grants/:id/revoke`
- `GET /api/usage/events?actor=&feature=&from=&to=&page=`
- `GET /api/usage/summary?group_by=actor|feature|model|plan`

新增 public API：

- `GET /api/public/entitlement`

public route 必须从 `hone_web_session` 推导当前 `web` actor，不能接受 query actor。返回值只包含当前用户可见的 plan name、limits、remaining、period_end、warnings，不返回其他 actor 或 admin metadata。

### 5. 前端落点

前端新增数据层：

- `packages/app/src/lib/entitlements.ts`
- `packages/app/src/context/entitlements.tsx`
- `packages/app/src/pages/usage.tsx`

改造现有页面：

- `packages/app/src/pages/chat.tsx` 使用 `/api/public/entitlement` 替代单纯 auth user 里的 `daily_limit/remaining_today`；兼容字段可保留到迁移完成。
- `packages/app/src/pages/settings.tsx` 的 invite table 增加 plan 和 usage snapshot，减少在表格里硬编码“复用现有 12 次对话额度限制”的文案。
- 管理端新增 Usage 页面或 Settings 子 tab，避免把邀请码表变成过载运营后台。

## 实施步骤

### Phase 1: Ledger 骨架与双写

- 在 `memory` 新增 entitlement/usage SQLite 存储、类型和测试。
- 启动时创建默认 plan，读取 `agent.daily_conversation_limit` 作为兼容默认值。
- `AgentSession` 用户 chat 成功/失败路径双写 usage event，同时保留旧 quota JSON。
- LLM audit 写入后记录 token usage event。
- 增加 admin/public snapshot API，但 public UI 仍使用旧字段。

### Phase 2: 产品化 entitlement 决策

- 把 `reserve_conversation_quota` 切到 `EntitlementService`，支持 allow/deny/warn/bypass 原因。
- 将 cron enabled limit 从硬编码常量逐步接入 plan limit；先保留 12 作为 default plan 值。
- Public `/chat` 使用 entitlement snapshot 显示额度、恢复时间和 deny reason。
- Settings 邀请码表显示 plan 与本周期 usage summary。

### Phase 3: 管理端运营与增长闭环

- 新增 Usage/Entitlements 管理页，支持按 actor/feature/model/period 聚合。
- 增加 manual grant、extend trial、revoke grant、导出 usage CSV。
- 支持兑换码或 invite metadata 预设 plan，但不直接接支付。
- 将 high-cost / near-limit / abnormal-bypass 聚合接入 dashboard。

### Phase 4: Workspace 与支付预留

- 如果 `WorkspaceIdentity` 落地，把 grant owner 从 actor 扩展到 workspace，同时保留 actor override。
- 为外部支付预留 `external_customer_id` / `external_subscription_id` 字段，但不在第一版耦合任何支付供应商。
- 定义团队 seat、渠道席位、共享任务额度等更高阶 plan。

## 验证方式

- Rust 单元测试：
  - 默认 plan 能从 `agent.daily_conversation_limit` 兼容生成。
  - actor grant 生效、过期、撤销和优先级选择正确。
  - chat reserve/commit/release 在成功、失败、并发、admin bypass、daily limit=0 场景下行为与现有 quota 一致。
  - scheduled task 不扣 chat quota，但会写 `scheduled_run` usage event。
  - LLM audit token 能生成 `llm_tokens` usage event，缺失 token 时不会误报。
- Web API 测试：
  - public entitlement 只能读取当前 cookie actor。
  - admin summary 能按 actor/feature/model/date 过滤。
  - grant/revoke 需要 admin 权限并写 audit metadata。
- 前端验证：
  - `bun run test:web` 覆盖 entitlement snapshot 转换、near-limit 状态和 deny reason。
  - public chat 在 unlimited、remaining>0、remaining=0、warning 四类状态下可用性正确。
  - Settings invite table 在旧后端字段和新 entitlement 字段共存期间不崩。
- 手工验收：
  - 创建一个 Web invite，完成 1 次成功对话后，old quota 和 usage ledger 都能看到扣减。
  - 触发一次失败 runner，不扣 chat success，但 token usage 如有返回会进入 ledger。
  - admin 给用户增加临时 grant 后，public chat 立即显示新额度。
- 指标：
  - 能按日回答“哪些用户/功能/模型消耗最多成本”。
  - 能按用户回答“为什么被限额拒绝、何时恢复、谁授予过额外额度”。
  - 转化实验能区分 trial near-limit 用户和自然低频用户。

## 风险与取舍

- 风险：过早引入 billing 概念会增加复杂度。取舍：第一版只做本地 plan/grant/ledger，不接支付，不改变开源本地默认体验。
- 风险：双写期间旧 quota 与新 ledger 可能不一致。取舍：保留旧 quota 为行为 source of truth，ledger 先做观测；切换前增加一致性检查。
- 风险：token usage 不完整，尤其 ACP runner 未必总返回标准 token。取舍：记录 missing usage，不用它做硬性限额，只用于成本趋势。
- 风险：按 actor 授权会与未来 workspace 合并重复。取舍：schema 预留 owner type，第一版 actor-scoped，workspace 落地后迁移 grant owner。
- 风险：用户看到太多额度细节会降低体验。取舍：public 只显示当前阻塞相关的简洁权益，复杂 ledger 只给 admin。
- 不做：不接 Stripe/微信支付/支付宝，不做自动扣费，不把公司画像变成付费墙，不改变 `ActorIdentity` / `SessionIdentity` 语义，不把 scheduled task 突然纳入 chat 日额度。

## 与已有提案的差异

- 与 `auto_p1_delivery_decision_loop.md` 不重复：该提案解释事件为什么推送或过滤；本提案解释用户和功能为什么有权或无权消耗系统资源。
- 与 `auto_p1_evidence_review_queue.md` 不重复：该提案处理市场证据到公司画像的复盘闭环；本提案处理产品权益、用量、成本和转化。
- 与 `auto_p1_linked-user-workspace.md` 不重复：该提案解决跨渠道真实用户资产归属；本提案第一版仍 actor-scoped，只为未来 workspace-level grant 预留迁移路径。
- 与 `auto_p1_run_trace_workbench.md` 不重复：该提案面向单次 agent run 的排障 trace；本提案面向跨运行、跨功能的 usage ledger 与 entitlement decision。
- 与 `desktop-bundled-runtime-startup-ux.md` 不重复：该提案解决桌面启动和 sidecar ownership；本提案只让 desktop 展示 local/remote entitlement 差异。
- 与 `skill-runtime-multi-agent-alignment.md` 不重复：该提案解决 skill runtime 与 multi-agent 执行语义；本提案不改变 skill 调用，只记录和限制资源消耗。

查重结论：现有 proposal 覆盖通知可信度、证据复盘、跨渠道 workspace、运行排障、桌面启动和 skill runtime，但没有覆盖“商业权益/试用额度/成本 ledger/按功能用量”的产品与架构层。因此本主题是新的、可落地的 P1 提案。
