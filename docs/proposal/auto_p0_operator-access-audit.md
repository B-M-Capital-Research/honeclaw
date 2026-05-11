# Proposal: Operator Access Control and Admin Audit Trail

status: proposed
priority: P0
created_at: 2026-05-11 11:04:51 +0800
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
- `docs/proposal/auto_p1_user-data-trust-center.md`
- `docs/proposal/auto_p1_agent-mutation-ledger.md`
- `docs/proposal/auto_p1_usage_entitlement_ledger.md`
- `docs/proposal/auto_p1_invite_activation_funnel.md`
- `docs/proposal/auto_p1_runtime_readiness_matrix.md`
- `docs/proposal/auto_p1_run_trace_workbench.md`
- `docs/proposal/auto_p1_automation_intent_control_plane.md`
- `crates/hone-web-api/src/routes/mod.rs`
- `crates/hone-web-api/src/routes/auth.rs`
- `crates/hone-web-api/src/state.rs`
- `crates/hone-web-api/src/lib.rs`
- `crates/hone-web-api/src/routes/web_users.rs`
- `crates/hone-web-api/src/routes/channel_settings.rs`
- `crates/hone-web-api/src/routes/event_engine_admin.rs`
- `crates/hone-web-api/src/routes/portfolio.rs`
- `crates/hone-web-api/src/routes/company_profiles.rs`
- `crates/hone-web-api/src/routes/cron.rs`
- `crates/hone-core/src/config/server.rs`
- `crates/hone-core/src/config/agent.rs`
- `crates/hone-channels/src/core/bot_core.rs`
- `packages/app/src/lib/backend.ts`
- `packages/app/src/context/backend.tsx`
- `packages/app/src/pages/settings.tsx`
- `config.example.yaml`

## 背景与现状

Honeclaw 已经从本地投资聊天助手扩展成公开 Web、管理端、桌面端、多渠道 IM、Hone Cloud API key、主动事件引擎和长期投研记忆组合起来的产品。README 明确把 Web、Mac App、iMessage、Feishu、Telegram、Discord、公司画像、持仓监控和 scheduled tasks 作为核心卖点；`docs/repo-map.md` 也说明管理端与公开端已经按端口和构建产物分离，公开端服务 invite 登录用户，管理端承担配置、用户、任务、通知、日志、技能和数据操作。

当前 admin HTTP 认证模型仍然非常轻：

- `crates/hone-web-api/src/routes/auth.rs` 对 `/api/*` 管理端路由使用一个全局 Bearer token；如果没有配置 token，则 `deployment_mode == "local"` 时直接放行。
- `crates/hone-web-api/src/lib.rs` 从 `core.config.web.auth_token` 读取这个 token，并放入 `AuthState { bearer_token, sse_tickets }`。
- `crates/hone-core/src/config/server.rs` 的 `WebConfig` 只有 `auth_token`，没有 operator 用户、角色、会话、MFA、token rotation 或 per-route scope。
- `packages/app/src/lib/backend.ts` 只把单个 `bearerToken` 放进 `Authorization: Bearer ...`；桌面 remote backend 也沿用这个连接模型。
- `crates/hone-web-api/src/routes/mod.rs` 把 `/api/web-users/invites/*`、`/api/channel-settings`、`/api/event-engine/*`、`/api/portfolio/*`、`/api/company-profiles/*`、`/api/cron-jobs/*`、`/api/skills/*`、`/api/llm-audit`、`/api/logs` 等大量管理能力放在同一个 middleware 后面。
- `crates/hone-web-api/src/routes/web_users.rs` 能创建、停用、重置邀请码，获取或重置 public API key 明文；`channel_settings.rs` 能写 Telegram / Discord token 等渠道密钥；`company_profiles.rs` 能导出、导入、删除长期画像；`portfolio.rs` 和 `cron.rs` 能修改用户投资上下文和自动化任务。
- `crates/hone-core/src/config/agent.rs` 的 `AdminConfig` 与 `crates/hone-channels/src/core/bot_core.rs` 的 `is_admin_actor()` 解决的是 IM / CLI actor 在对话内是否是管理员，不等同于 Web 管理端 operator 身份。

这个设计适合单机、本地开发和早期维护者自用；但随着 public invite、Hone Cloud client、远程 desktop backend、用户数据导出/删除、API key、事件引擎配置和多渠道运维进入同一个控制台，单个共享 admin token 已经成为产品架构上的核心风险。

## 问题或机会

这是 P0 级问题，因为它直接影响数据安全、用户信任、关键管理链路可信度和未来商业化基础。Hone 保存的是投资上下文、会话、持仓、长期公司画像、定时任务、模型审计、渠道凭证和 public API key；管理端一旦被误用或凭证泄露，影响不是 UI 设置丢失，而是用户长期研究资产、推送策略、隐私和付费试用入口同时暴露。

当前缺口集中在六类：

1. **没有 operator 身份，只有共享 token。**
   多个人或多个自动化使用同一个 Bearer token 时，系统无法回答“谁创建了这个邀请码”“谁重置了 API key”“谁删除了画像”“谁把 Telegram token 改掉了”。日志里只能看到请求结果，不能看到责任主体。

2. **高危 admin 操作没有权限分层。**
   查看 logs、刷新 meta、修改语言、创建 public invite、获取 API key 明文、重置 API key、导入画像、删除画像、修改 event engine RSS、改渠道 token、停用用户，当前都在同一认证边界下。最小权限原则无法落地。

3. **审计与现有数据治理提案缺少操作者维度。**
   User Data Trust Center 可以描述用户数据有哪些；Agent Mutation Ledger 可以记录用户状态怎么被改；Run Trace Workbench 可以解释一次运行发生了什么。但如果 admin API 的操作者仍是“共享 token”，这些审计都缺少关键的一列：哪个 operator 执行或批准了动作。

4. **远程管理和桌面 remote 模式风险被低估。**
   `docs/invariants.md` 已要求非本地 Web console 必须启用 Bearer token，但 token 仍然是长期共享密钥。桌面 remote backend 也允许用户把 bearer token 保存在本地配置里。token 泄露后不能只撤销某个 operator 或某台设备，通常只能整体换 token，影响所有接入者。

5. **敏感 secret 操作缺少二次确认和一次性显示记录。**
   管理端能获取或重置 public API key 明文，设置渠道 bot token、LLM key、FMP/Tavily key 等。现有 config mutation 会脱敏日志和返回值中的 key 字段，但 admin 操作本身没有“谁看过明文、何时看过、为什么看、是否过期”的审计面。

6. **商业化和团队协作无法建立。**
   Invite 用户、API key、额度、使用成本、数据导出和删除请求都需要有人处理。没有 operator 账号、角色、审计和待办分派，就很难支持客服、运营、开发、投资研究顾问或企业客户管理员共同使用同一个后台。

机会是：不必一开始建设复杂企业 IAM。先把管理端从“一个 token 代表所有人”升级为“operator session + role/scopes + admin audit event”，就能显著降低远程部署和多人协作风险，并为用户数据、mutation、entitlement、support 等后续提案补齐安全底座。

## 方案概述

新增 **Operator Access Control and Admin Audit Trail**，把 Web 管理端访问拆成四个一等对象：

- `OperatorIdentity`：管理端操作者，独立于 `ActorIdentity`。Actor 是被服务的用户 / 渠道身份，Operator 是操作控制台的人或自动化。
- `OperatorSession`：短期登录态或 API token，支持设备标识、过期、撤销、最后使用时间和来源 IP/user-agent 摘要。
- `OperatorRole` / `AdminScope`：权限集合，例如 `viewer`、`support`、`operator`、`developer`、`owner`，以及细粒度 scope：`users.read`、`users.invite.write`、`api_keys.reveal`、`config.write`、`channels.secrets.write`、`profiles.delete`、`event_engine.write`、`logs.read`、`llm_audit.read`。
- `AdminAuditEvent`：所有敏感管理操作的不可静默跳过记录，包含 operator、route、action、target actor/resource、result、risk level、request id、reason、before/after 摘要和 redaction 状态。

一期目标：

1. 保留现有 `web.auth_token` 作为 bootstrap / break-glass 兼容入口。
2. 新增本地 operator 存储和登录会话，管理端 UI 用 operator session 访问 API。
3. 对高危路由加 scope 校验和审计事件。
4. 对 shared bearer token 访问标记为 `operator=legacy_token`，并在非 local deployment 中给出迁移 warning。
5. 先覆盖 admin route，不改变 public invite 用户登录、IM admin actor 注册或 agent runner 权限模型。

## 用户体验变化

### 管理端

- 首次远程打开管理端时，不再只要求粘贴全局 token，而是引导 owner 创建第一个 operator。
- Settings 增加 `Operators` 页面：创建 operator、分配角色、禁用账号、撤销会话、生成有限 scope 的 automation token。
- 用户、邀请、API key、渠道设置、事件引擎、技能、画像、任务等页面的高危按钮会显示操作者身份和要求原因，例如“重置 API key 需要 `api_keys.rotate` 权限并填写 reason”。
- `/users`、`/notifications`、`/task-health`、`/llm-audit`、`/logs` 等排障页面可以显示相关 admin audit event，帮助解释状态变化。
- Owner 能看到“最近 24 小时高危操作”：API key reveal/reset、channel secret write、profile delete/import, cron delete, event engine config write, skill registry reset。

### 用户端

- Public 用户不会直接看到 operator 体系，但未来 Data Trust Center、删除请求、API key 重置、客服介入可以展示“由 Hone support 于某时间处理”，而不是只有系统状态。
- 当 operator 代用户修复 portfolio、task 或 company profile 时，可以在用户可见的变更记录里显示“由管理员处理”与简短原因，避免用户以为 agent 自己改了长期状态。

### 桌面端

- Local bundled 模式默认可以继续无登录使用，但 UI 应明确标注 “Local owner mode”。
- Desktop remote backend 不再长期保存全局 admin token，而是保存 operator session 或有限 scope token；会话可从服务端撤销。
- 诊断包里可以包含 operator session id/hash 和 audit event id，方便排查远程连接问题，但不得包含 token 明文。

### 多渠道与自动化

- IM 里的 `/register-admin` 仍然用于 actor-side admin 识别，不自动成为 Web operator。
- 自动化脚本或监控可以使用 scoped service token，例如只允许 `meta.read`、`task_runs.read`、`notifications.read`，不能重置用户 API key 或改渠道 secret。
- 未来如果 agent 需要代表 operator 执行管理动作，必须通过 service token 或 explicit approval，不能复用 public user session。

## 技术方案

### 1. Operator 存储

在 `memory` 或 `crates/hone-web-api` 内新增 SQLite 存储，建议文件放在 canonical runtime/config data 下，例如 `storage.web_auth_db_path` 同级或新增 `operator_auth.sqlite3`：

```text
operators (
  operator_id TEXT PRIMARY KEY,
  display_name TEXT NOT NULL,
  email TEXT,
  phone TEXT,
  status TEXT NOT NULL,
  password_hash TEXT,
  roles_json TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  disabled_at TEXT
)

operator_sessions (
  session_id TEXT PRIMARY KEY,
  operator_id TEXT NOT NULL,
  token_hash TEXT NOT NULL,
  created_at TEXT NOT NULL,
  expires_at TEXT NOT NULL,
  last_used_at TEXT,
  user_agent_hash TEXT,
  ip_prefix TEXT,
  revoked_at TEXT
)

operator_service_tokens (
  token_id TEXT PRIMARY KEY,
  operator_id TEXT,
  label TEXT NOT NULL,
  token_hash TEXT NOT NULL,
  scopes_json TEXT NOT NULL,
  expires_at TEXT,
  last_used_at TEXT,
  revoked_at TEXT,
  created_at TEXT NOT NULL
)

admin_audit_events (
  event_id TEXT PRIMARY KEY,
  created_at TEXT NOT NULL,
  operator_kind TEXT NOT NULL,
  operator_id TEXT,
  session_id TEXT,
  service_token_id TEXT,
  action TEXT NOT NULL,
  route TEXT NOT NULL,
  method TEXT NOT NULL,
  target_type TEXT,
  target_id TEXT,
  actor_channel TEXT,
  actor_scope TEXT,
  actor_user_id TEXT,
  risk TEXT NOT NULL,
  result TEXT NOT NULL,
  reason TEXT,
  request_id TEXT,
  before_summary_json TEXT,
  after_summary_json TEXT,
  error_class TEXT
)
```

Token 只存 hash；服务端只在创建 service token 时显示一次明文。`OperatorIdentity` 不替代 `ActorIdentity`，它只描述管理动作的发起者。

### 2. Auth middleware 分层

把当前 `auth::require_api_auth` 拆成两层：

- `authenticate_admin_request`：解析 operator cookie/session、service token、legacy bearer token、SSE ticket，生成 `AdminPrincipal`。
- `authorize_scope(required_scope)`：按路由或 handler 要求校验 scope。

初始兼容策略：

- local deployment 且没有 `web.auth_token`、没有 operator store：允许 `LocalOwner` principal，通过所有 scope，但 UI 标注 local owner mode。
- 配置了 `web.auth_token`：允许 legacy bearer 访问，但 audit event 记录 `operator_kind=legacy_bearer`；非 local deployment 在 `/api/meta` 返回 `admin_auth_migration_required=true` warning。
- 创建第一个 owner 后，默认要求 operator session；legacy bearer 只保留 break-glass scope，例如创建 owner、撤销锁死状态。

### 3. Route scope matrix

第一版先覆盖高危和常用读路由：

```text
GET  /api/meta                         meta.read
GET  /api/users                        users.read
GET  /api/history                      sessions.read
POST /api/chat                         sessions.write
GET  /api/web-users/invites            web_users.read
POST /api/web-users/invites            web_users.invite.write
POST /api/web-users/invites/*/reset    web_users.invite.rotate
POST /api/web-users/invites/*/api-key  api_keys.reveal
POST /api/web-users/invites/*/api-key/reset api_keys.rotate
GET/PUT /api/channel-settings          config.read / channels.secrets.write
GET/PUT /api/event-engine/global-digest event_engine.read / event_engine.write
POST/PUT/DELETE /api/event-engine/rss-feeds* event_engine.write
POST /api/event-engine/mainline-distill event_engine.run
POST/PUT/DELETE /api/cron-jobs*        automation.write
POST/PUT/DELETE /api/portfolio*        portfolio.write
POST /api/company-profiles/import/*    profiles.import
DELETE /api/company-profiles/*         profiles.delete
PATCH /api/skills/*/state              skills.write
POST /api/skills/reset                 skills.reset
GET /api/llm-audit                     llm_audit.read
GET /api/logs                          logs.read
```

对于只读 admin 页面，可从 `viewer` 开始；对于 secret reveal/reset、profile delete、config secret write，要求 `owner` 或显式 scope。

### 4. Audit event 生成

采用 handler-level + middleware-level 混合：

- Middleware 记录认证失败、scope denied、legacy bearer 使用。
- Handler 在执行敏感操作前后写入 action-specific event，因为只有 handler 知道 target actor、resource id、before/after 摘要和结果。
- 对 API key、bot token、LLM key 等 secret，只记录 prefix、hash、字段名、是否 changed，不记录明文。
- 对导入画像、删除 profile、重置 skill registry、修改 event engine 配置等，记录 resource id、count、config paths、old/new digest，而非完整敏感内容。

可先实现 helper：

```rust
pub struct AdminAuditRecorder {
    pub principal: AdminPrincipal,
    pub request_id: String,
}

impl AdminAuditRecorder {
    pub fn record(&self, event: AdminAuditEventInput) -> HoneResult<()>;
}
```

### 5. Frontend 与 UX

新增或调整：

- `packages/app/src/context/backend.tsx`：连接远程 backend 时支持 operator login flow，保留 legacy bearer fallback。
- `packages/app/src/lib/backend.ts`：`buildAuthHeaders` 支持 session cookie 或 scoped token；SSE ticket 绑定 principal。
- `packages/app/src/pages/settings.tsx`：新增 Operators panel，管理 operator、role、service token、active sessions。
- 新增 `packages/app/src/pages/admin-audit.tsx` 或 Settings 子页：按时间、operator、target actor、action、risk、result 过滤 audit events。
- 高危按钮统一要求 reason modal；reason 进入 audit event，也可传给 Agent Mutation Ledger。

第一版不需要实现复杂 SSO，但数据模型应保留 `email`、`external_subject` 的扩展位，未来可接 GitHub/Google/企业 IdP。

### 6. 与现有 admin actor 的边界

`AdminConfig` 和 `HoneBotCore::is_admin_actor()` 继续处理 IM/CLI channel 里的对话权限，例如谁能用 `/register-admin` 或 admin-only skill。新的 operator auth 只处理 Web 管理端和 service API。

如果未来要把 IM admin actor 绑定到 Web operator，必须显式 linking：

- operator 在管理端生成一次性 code；
- IM admin 发送 `/link-operator <code>`；
- 后端记录 `operator_channel_links`，只用于审计显示，不自动扩大 IM actor 权限。

## 实施步骤

1. **Audit-only skeleton**
   - 新增 operator/auth/audit 数据模型和 SQLite migration。
   - 不改变现有认证，只把 legacy bearer/local owner 访问写入 `admin_audit_events`。
   - 给 web user invite create/reset/API key、channel settings write、company profile delete/import、cron delete、event engine write 加 handler-level audit。

2. **Scope matrix dry-run**
   - 为 route 注册 required scope，但先只在 response/meta 或 log 中报告缺失，不阻断。
   - 管理端展示 “this action will require scope X”。
   - 补齐 route/action 命名规范和测试 fixtures。

3. **Operator login v1**
   - Bootstrap 创建第一个 owner。
   - 支持 password-based operator login、session cookie、logout、session revoke。
   - 管理端 remote mode 默认使用 operator session；legacy bearer 显示迁移提示。

4. **Enforce high-risk scopes**
   - 先强制 `api_keys.reveal`、`api_keys.rotate`、`channels.secrets.write`、`profiles.delete`、`skills.reset`、`event_engine.write`。
   - 其它读路由和低风险写路由继续兼容一段时间。

5. **Service tokens and desktop remote migration**
   - 支持 scoped service token，替代共享 admin token 给监控/自动化使用。
   - Desktop remote backend 设置页改为 operator session 或 scoped token，并提供 token revoke 指引。

6. **Link to data/mutation/trace surfaces**
   - User Data Trust Center 删除/导出请求记录 operator。
   - Agent Mutation Ledger 的 `source=admin_web` 记录 `operator_id`。
   - Run Trace / logs / task-health 详情页可跳到相关 audit event。

## 验证方式

- 单元测试：
  - Bearer token legacy path 仍兼容。
  - local owner mode 只在 `deployment_mode == "local"` 且无 operator/token 时放行。
  - operator session token 只存 hash，过期/撤销后拒绝。
  - scope matrix 对高危路由正确 allow/deny。
  - SSE ticket 绑定已认证 principal，过期后无效。

- API 集成测试：
  - 无凭证访问远程 admin API 返回 401。
  - viewer 能读 `/api/users`，不能 reset API key。
  - support 能创建 invite，但不能 reveal channel token。
  - owner 能执行 high-risk action，并产生 audit event。
  - legacy bearer 执行 high-risk action 时产生 `legacy_bearer` audit event 和 migration warning。

- 前端测试：
  - remote backend operator login flow。
  - Settings Operators 页面创建/禁用 operator。
  - 高危 action reason modal 必填。
  - Audit 页面能按 action/operator/result 过滤。

- 安全回归：
  - audit event 不包含 API key、bot token、LLM key 明文。
  - service token 明文只在创建时返回一次。
  - `config.example.yaml` 不新增默认 secret。
  - 日志和错误信息不输出 operator session token。

- 手工验收：
  - 本地 bundled 模式不被登录流程打断。
  - 远程 admin 从 legacy bearer 迁移到 operator owner 后可正常管理 invites、channels、event engine。
  - 撤销某 operator session 后，已打开的管理端下一次 API 请求失败并要求重新登录。

## 风险与取舍

- **迁移复杂度**：如果一次性强制所有远程部署使用 operator，会阻断现有维护者。应分 audit-only、dry-run、high-risk enforce 三阶段推进。
- **本地体验**：Hone 仍是 local-first 工具，不能让单机用户每次开桌面都登录。Local owner mode 应保留，但在远程暴露时必须可检测和提示。
- **权限矩阵过细会拖慢开发**：第一版只定义少量稳定 role + scope，高危写操作先管住；普通读路由可暂时粗粒度。
- **审计日志本身也敏感**：audit event 会暴露用户 id、操作目标和配置字段名，需要纳入 User Data Trust Center 和导出/删除策略，但不应记录 secret 明文。
- **不能替代 agent mutation ledger**：operator audit 记录“哪个管理员调用了什么管理动作”；mutation ledger 记录“用户长期状态如何改变、能否撤销”。两者应互相关联，不互相吞并。
- **不做外部 SSO 起步**：第一版不接 Google/GitHub/企业 IdP，避免把安全改善卡在外部登录集成上；保留字段和接口扩展位即可。

## 与已有提案的差异

本轮查重范围包括 `docs/proposal/` 和历史 `docs/proposals/` 下全部现有提案，重点核对：

- `auto_p1_user-data-trust-center.md`
- `auto_p1_agent-mutation-ledger.md`
- `auto_p1_usage_entitlement_ledger.md`
- `auto_p1_invite_activation_funnel.md`
- `auto_p1_runtime_readiness_matrix.md`
- `auto_p1_run_trace_workbench.md`
- `auto_p1_automation_intent_control_plane.md`
- `auto_p1_response-feedback-learning-loop.md`
- `docs/proposals/desktop-bundled-runtime-startup-ux.md`

差异结论：

- 不重复 `auto_p1_user-data-trust-center.md`：该提案关注用户数据 inventory、导出、删除和隐私执行；本提案关注谁能在管理端访问/修改这些数据，以及每次 admin 操作如何授权和审计。
- 不重复 `auto_p1_agent-mutation-ledger.md`：mutation ledger 关注 agent/UI 对用户状态的 before/after、确认和撤销；本提案关注 Web admin operator 身份、角色、scope、session、service token 和管理操作审计。两者未来应通过 `operator_id` 和 mutation record 关联。
- 不重复 `auto_p1_usage_entitlement_ledger.md`：entitlement 处理用户或 workspace 的权益、额度、成本；本提案处理管理员和自动化自身的访问权限，不定义付费 plan。
- 不重复 `auto_p1_invite_activation_funnel.md`：activation funnel 判断 invite 用户是否完成价值里程碑；本提案只治理 operator 创建/维护 invite 用户时的权限和审计。
- 不重复 `auto_p1_runtime_readiness_matrix.md`：readiness matrix 判断模型、渠道、provider、sidecar 是否可用；本提案判断管理动作是否由有权限的 operator 发起，以及是否留下审计证据。
- 不重复 `auto_p1_run_trace_workbench.md`：run trace 面向一次 agent run 的执行过程；admin audit 面向跨运行的控制台操作记录。查看 trace 可以是一个 audited action，但 trace 本身不解决 operator identity。
- 不重复 `auto_p1_automation_intent_control_plane.md`：automation intent 解决 cron/heartbeat 定义变更前的预览和确认；本提案解决谁有权在管理端创建、修改、删除或批准这些自动化。
- 不重复 `desktop-bundled-runtime-startup-ux.md`：desktop startup UX 解决本机 sidecar 启动冲突和恢复；本提案只在 desktop remote/bundled 管理访问上提供身份与审计模型，不改变进程锁策略。

查重结论：现有提案已经覆盖用户数据、agent 状态变更、权益、邀请激活、运行追踪、自动化意图和桌面启动，但没有覆盖“Web 管理端 operator 身份、角色权限、service token、会话撤销和 admin 操作审计”这一安全底座。本主题是新的、可落地的 P0 产品/架构提案。

## 文档同步说明

本轮只新增 proposal，不开始执行该提案，不修改业务代码、测试代码、运行配置或 `docs/current-plan.md`。如果后续进入实现阶段，应新增动态计划并同步更新 `docs/invariants.md` 中的非本地 Web console 认证约束、`docs/repo-map.md` 中的管理端认证结构，以及必要的 runbook / handoff。
