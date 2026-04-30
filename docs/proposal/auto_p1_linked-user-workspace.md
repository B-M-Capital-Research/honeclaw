# Proposal: Linked User Workspace for Cross-Channel Continuity

- status: proposed
- priority: P1
- created_at: 2026-04-30 17:02 +0800
- owner: automation
- related_files:
  - `README.md`
  - `AGENTS.md`
  - `docs/repo-map.md`
  - `docs/invariants.md`
  - `docs/decisions.md`
  - `docs/current-plan.md`
  - `docs/proposal/auto_p1_delivery_decision_loop.md`
  - `docs/proposals/desktop-bundled-runtime-startup-ux.md`
  - `docs/proposals/skill-runtime-multi-agent-alignment.md`
  - `crates/hone-core/src/actor.rs`
  - `memory/src/web_auth.rs`
  - `memory/src/portfolio.rs`
  - `memory/src/cron_job/storage.rs`
  - `memory/src/company_profile/storage.rs`
  - `crates/hone-event-engine/src/prefs.rs`
  - `crates/hone-web-api/src/routes/public.rs`
  - `crates/hone-web-api/src/routes/company_profiles.rs`
  - `crates/hone-web-api/src/routes/cron.rs`
  - `crates/hone-web-api/src/routes/notification_prefs.rs`
  - `packages/app/src/lib/actors.ts`
  - `packages/app/src/context/company-profiles.tsx`
  - `packages/app/src/pages/users.tsx`
  - `packages/app/src/pages/settings.tsx`

## 背景与现状

Hone 当前已经具备跨 Web、桌面和 IM 的投资研究助手形态，但“同一个真实用户在多个渠道中使用 Hone”还主要依赖 `ActorIdentity(channel, user_id, channel_scope)` 作为隔离边界。

这个边界对安全和多租户隔离是正确的：

- `crates/hone-core/src/actor.rs` 明确区分 `ActorIdentity` 与 `SessionIdentity`：前者负责权限、quota、sandbox 和私有数据隔离，后者负责会话历史归属。
- `memory/src/portfolio.rs`、`memory/src/cron_job/storage.rs`、`memory/src/company_profile/storage.rs` 都按 actor 存储持仓、定时任务和公司画像。
- `crates/hone-event-engine/src/prefs.rs` 的通知偏好也以每 actor 一个 JSON 文件为运行时生效单元。
- Web 公共端在 `crates/hone-web-api/src/routes/public.rs` 中把邀请制登录用户固定映射为 `ActorIdentity::new("web", user.user_id, None)`。
- 管理端通过 `packages/app/src/lib/actors.ts` 和 `/users` 页面把 sessions、portfolio、company profiles 聚合为 actor 列表，而不是聚合为“真实用户工作区”。
- 公司画像已经有 actor-scoped export / import，`packages/app/src/context/company-profiles.tsx` 还支持从已有画像空间、会话用户或手工 actor 中选择目标，但这仍是手工搬运，不是持续绑定。

因此当前系统的真实状态是：Hone 有很强的 actor 级隔离和跨 actor 管理能力，但还没有一个稳定的“同一用户跨渠道工作区”抽象。一个用户先在 Web 建立持仓和公司画像，再去 Feishu 或 Telegram 对同一标的发问时，很可能进入另一个 actor sandbox、另一份会话历史、另一组通知偏好和另一组 cron job。

## 问题或机会

Hone 的产品价值不是“某个渠道的聊天记录”，而是长期投资研究资产：持仓、关注列表、公司画像、事件时间线、投资 thesis、通知偏好、定时监控任务，以及围绕这些资产形成的跨渠道提醒和复盘。

如果这些资产被渠道割裂，会直接影响核心体验和增长：

- 用户端：用户会觉得 Web 里已经维护过的公司画像，在 IM 里“不认识我”；IM 中创建的任务，在桌面或 Web 里也不自然出现。
- 管理端：运营和维护人员只能看到多个 actor，而不是“这个真实用户有哪些渠道、哪些资产、哪些通知端点、哪个渠道最近失效”。
- 桌面端：桌面 bundled runtime 能管 channel 进程，但还不能指导用户把本机 Web 账号和 IM channel 绑定为同一个可迁移工作区。
- 多渠道：同一个用户希望“在 Feishu 收盘中提醒，在 Web 看完整画像，在 Telegram 临时追问”，目前需要复制资产或依赖 agent 自己碰巧重建上下文。
- 商业化：邀请制 Web 用户已经有手机号、密码、TOS 和会话额度。未来如果转为试用、订阅或团队席位，单纯按 channel actor 计费和限额会把一个真实用户拆成多个账户。

这不是要削弱 `ActorIdentity`。相反，Hone 应保留 actor 作为执行期安全边界，同时引入一个显式、可撤销、可审计的 `WorkspaceIdentity`，把多个已验证 actor 绑定到同一个用户投资工作区。

## 方案概述

新增“Linked User Workspace”产品与架构层：一个 workspace 代表一个真实用户或一个明确的投资研究空间；多个 actor 可以选择性加入同一个 workspace。

核心原则：

- `ActorIdentity` 不变：继续作为权限、quota、sandbox、通道入站和本地文件可见性的安全边界。
- `SessionIdentity` 不变：继续回答“这条消息写进哪份会话历史”。
- 新增 `WorkspaceIdentity`：回答“这些 actor 是否属于同一个长期投资资产空间”。
- 绑定必须显式：由 Web 账号、桌面本机确认或 IM 一次性验证码完成，不根据手机号、昵称、channel user_id 自动合并。
- 迁移必须分阶段：先做只读聚合和显式导入，再允许新资产写入 workspace 级存储，避免破坏现有 actor 数据。

一期目标不是把所有存储一次性迁走，而是先让用户和管理员看到“同一个真实用户的跨渠道资产地图”，并提供安全的绑定 / 解绑 / 资产合并路径。

## 用户体验变化

用户端：

- Web `/me` 或账户页增加“已连接渠道”，展示当前 Web 账号已绑定的 Feishu、Telegram、Discord、iMessage 或桌面本机端点。
- 用户可以生成一次性绑定码，在 IM 中发送 `/link <code>` 或点击渠道深链完成绑定。
- 绑定后，Web 用户端能看到来自已绑定 IM actor 的任务、公司画像、持仓摘要和最近会话入口。
- 首次绑定时不自动覆盖资产，而是展示合并向导：保留 Web 资产、导入 IM 资产、或只建立未来共享。

管理端：

- `/users` 左栏从纯 actor 列表升级为两层视图：`Workspace` 与 `Unlinked Actors`。
- Workspace 详情页展示绑定的 actors、每个 actor 的 channel/scope、最近消息、持仓数量、画像数量、启用任务数、通知端点健康状态。
- 管理员可以给一个 Web 邀请用户预创建 workspace，也可以把孤立 actor 通过确认流程加入某个 workspace。
- 对异常状态提供运维提示：同一手机号下存在多个未绑定 Web 用户、同一 channel user_id 已绑定到两个 workspace、某 actor 解绑后仍有启用任务。

桌面端：

- 桌面 settings 或 onboarding 中增加“连接本机工作区”步骤：如果用户已经有 Web 账号，可把本机 bundled runtime 识别为该 workspace 的管理端。
- channel status 不只显示进程是否在线，还标识该 channel 是否已绑定 workspace；未绑定时给出绑定入口，而不是只显示技术状态。

多渠道：

- IM 中支持 `/link`、`/unlink`、`/workspace` 这类轻量命令。
- 绑定后的 IM 回复可以引用 workspace 资产，例如“我已找到你 Web 工作区里的 MU 公司画像，以下是基于同一 thesis 的更新”。
- 群聊仍然尊重 `SessionIdentity::Group`，默认不自动加入个人 workspace；只有明确的 group workspace 或管理员绑定才共享群资产。

## 技术方案

### 1. 新增 workspace 链接表，不改 ActorIdentity 语义

在 `memory` 增加一个本地 SQLite 存储，建议复用 Web auth DB 或新增 `workspace_links.db`：

- `workspaces`
  - `workspace_id`
  - `display_name`
  - `primary_web_user_id`
  - `created_at`
  - `updated_at`
  - `status`
- `workspace_actor_links`
  - `workspace_id`
  - `actor_channel`
  - `actor_user_id`
  - `actor_channel_scope`
  - `link_status`
  - `linked_at`
  - `linked_by`
  - `last_verified_at`
  - unique `(actor_channel, actor_user_id, actor_channel_scope)` when active
- `workspace_link_codes`
  - high-entropy one-time code hash
  - target workspace
  - expires_at
  - max_uses = 1
  - created_by_actor

`ActorIdentity` 仍然是所有执行路径的输入。新增解析函数只做查询：

- `resolve_workspace(actor) -> Option<WorkspaceIdentity>`
- `list_workspace_actors(workspace_id) -> Vec<ActorIdentity>`
- `require_workspace_actor(workspace_id, actor) -> bool`

### 2. 先做聚合读模型，再做写入迁移

Phase 1 的所有长期数据仍按 actor 存储，只在 API 层提供 workspace 聚合：

- portfolio：列出 workspace 下所有 actor 的持仓，按 symbol 合并视图但保留来源 actor。
- company profiles：列出 workspace 下所有 actor 的画像空间，并标出重复 ticker。
- cron jobs：聚合所有 actor 的任务，保留任务真实 owner actor 和 channel target。
- notification prefs：展示每个 actor 的偏好，不自动合并。
- sessions：按 actor/session 原样展示，避免跨渠道上下文混写。

Phase 2 再为可共享资产定义 workspace-default：

- company profiles：允许某个 workspace 选择一个 canonical actor space，或新增 workspace-level profile root；旧 actor profile 通过显式 import 合并。
- portfolio：允许 workspace-level portfolio 成为新建/编辑默认源，但保留 actor-level portfolio 的兼容读取。
- notification prefs：保留 per-actor delivery prefs，因为不同渠道的打扰阈值天然不同；只抽出 workspace-level investment style / thesis 作为默认输入。
- cron jobs：默认仍由具体 actor 执行和推送；workspace 只提供可见性和复制模板，不把任务 owner 抽象掉。

### 3. Web auth 与绑定码

Web 邀请用户天然可以成为 workspace anchor：

- 创建 invite user 时同步创建 workspace，`primary_web_user_id = web user_id`。
- Web 登录后可生成短期绑定码，后端只存 hash，过期时间建议 10 分钟。
- IM channel 收到 `/link <code>` 后，以当前 channel actor 作为申请方，后端校验 code 后创建 `workspace_actor_links`。
- 已绑定 actor 再次绑定到其他 workspace 时必须先解绑，禁止静默迁移。

管理员路径需要更严格：

- 管理端可以发起绑定，但需要目标 actor 在 IM 中确认，除非是本地单用户部署的显式 admin bypass。
- 所有绑定、解绑、迁移都写 audit log，至少包含操作者 actor、目标 actor、workspace、时间和原因。

### 4. API 与前端形态

建议新增后端路由：

- `GET /api/workspaces`
- `GET /api/workspaces/:id`
- `POST /api/workspaces/:id/link-code`
- `POST /api/workspaces/link/confirm`
- `POST /api/workspaces/:id/unlink`
- `GET /api/workspaces/:id/assets`

Public Web 端只允许访问当前登录用户所属 workspace。Admin 端可以查询全部 workspace，但写操作要受 `is_admin_actor` 或 Web admin session 限制。

前端新增：

- `packages/app/src/context/workspaces.tsx`
- `packages/app/src/lib/workspaces.ts`
- `/users` 页面 workspace 分组和 unlinked actor 分组
- public `/me` 的 connected channels 区块
- settings 中 Web invite 列表增加 workspace 状态列

### 5. Agent runtime 使用方式

绑定后的 agent 不应把所有 actor 文件系统直接暴露给当前 runner。更安全的方式是：

- 当前消息仍只进入当前 actor sandbox。
- prompt session context 可以注入 workspace 摘要：已绑定渠道、可用 company profile 索引、canonical portfolio 摘要、最近任务摘要。
- 若需要读取其他 actor 的画像或任务详情，通过受控 tool/API 读取经过授权的 workspace asset，而不是切换 runner cwd。
- 所有跨 actor asset 读取都记录 audit，并在用户可见解释中说明来源，例如“来自 Web 工作区画像”。

这样既能提升连续性，又不破坏 `docs/invariants.md` 对 actor sandbox 和本地文件可见性的约束。

## 实施步骤

### Phase 1: Workspace 基础与只读聚合

- 在 `memory` 新增 workspace 存储和单元测试。
- 创建 Web invite user 时自动创建 workspace，并把 `web` actor 作为首个 link。
- 增加 admin-only workspace API。
- `/users` 页面把 actor 聚合成 workspace / unlinked 两类，所有现有 actor 级页面继续可进入。
- 不改 portfolio、company profiles、cron、prefs 的写入路径。

### Phase 2: 用户自助绑定

- Public Web `/me` 增加生成绑定码和 connected channels 列表。
- 在 channel pre-session intercept 中加入 `/link <code>`、`/unlink`、`/workspace`。
- 绑定成功后，在下一轮 agent prompt 中注入 workspace 摘要。
- 增加绑定/解绑 audit log 和过期 code 清理。

### Phase 3: 资产合并向导

- 公司画像：复用现有 bundle preview/apply 能力，提供 workspace 内 actor 间复制/合并 UI。
- 持仓：提供 symbol-level diff，避免自动合并 shares/avg_cost 造成误导。
- 通知偏好：只提供“复制偏好到另一个渠道”或“使用 workspace thesis 默认值”，不做全局覆盖。
- Cron：支持把一个 actor 的任务复制到另一个 channel target，原 owner 不变。

### Phase 4: Workspace-aware agent 工具

- 增加只读工具：`workspace_assets`、`workspace_profile_lookup`、`workspace_portfolio_summary`。
- 让 company portrait skill 在分析前优先查同 workspace 下的相关画像摘要，但只读跨 actor asset。
- 为跨 actor asset 引用加入 prompt audit / tool audit 记录。

## 验证方式

- Rust 单元测试：
  - workspace 创建、actor link 唯一性、解绑、过期绑定码、重复绑定冲突。
  - `resolve_workspace(actor)` 对 direct actor、group actor、unknown actor 的返回正确。
  - workspace 聚合资产保持 actor 来源，不把同 symbol 持仓静默相加为唯一真相源。
- Web API 测试：
  - public 用户只能读取自己的 workspace。
  - admin 可以读取所有 workspace。
  - 未绑定 actor 不能通过 workspace API 读取其他 actor 的资产。
  - 绑定码只能使用一次，过期后不可用。
- 前端验证：
  - `bun run test:web` 覆盖 actor/workspace 分组、绑定状态展示和资产 diff 转换。
  - 手工检查 `/users`、public `/me`、settings invite 表格在移动和桌面视口不溢出。
- 手工回归：
  - Web 登录生成 code，Feishu/Telegram/Discord 发送 `/link` 后在 `/users` 看到同 workspace。
  - 绑定后 IM 发问能看到 Web workspace 的画像摘要提示，但不能直接访问 sandbox 外绝对路径。
  - 解绑后新消息不再注入 workspace 摘要，旧 actor 数据仍保留。
- 产品指标：
  - 新 Web 用户绑定至少一个 IM channel 的比例。
  - 绑定用户 7 日内二次会话率、任务创建率、公司画像复用率。
  - 管理端定位“用户在哪个渠道收不到提醒”的时间下降。

## 风险与取舍

- 风险：错误绑定会造成隐私泄露。取舍：绑定必须显式确认，绑定码短期有效且只存 hash，禁止基于昵称或手机号自动合并。
- 风险：引入 workspace 后数据归属变复杂。取舍：一期只读聚合，写路径仍保持 actor-scoped，直到迁移策略和回滚路径成熟。
- 风险：跨 actor 读取画像可能破坏 sandbox 隔离。取舍：runner cwd 不变，通过受控 tool/API 读取授权摘要，不暴露其他 actor 文件系统。
- 风险：用户可能期待所有偏好自动同步。取舍：通知偏好默认 per-channel，因为 Web、Feishu、Telegram 的打扰容忍度不同，只同步 thesis / style 这类研究语义。
- 风险：group chat workspace 容易误合并多人数据。取舍：群聊默认不进入个人 workspace，必须单独创建 group workspace 或管理员显式绑定。
- 不做：不把 `ActorIdentity` 替换为 `WorkspaceIdentity`，不把所有会话历史合并为一条，不自动合并持仓金额，不把公司画像变成直接 UI 编辑器。

## 与已有提案的差异

查重范围：

- `docs/proposal/auto_p1_delivery_decision_loop.md`
- `docs/proposals/desktop-bundled-runtime-startup-ux.md`
- `docs/proposals/skill-runtime-multi-agent-alignment.md`

差异结论：

- 与 `auto_p1_delivery_decision_loop.md` 不重复：该提案聚焦 event-engine `delivery_log`、`NotificationPrefs` 和 missed events 的投递解释 / 偏好调优闭环；本提案聚焦真实用户跨 Web/IM/桌面渠道的长期资产归属、绑定和聚合视图。
- 与 `desktop-bundled-runtime-startup-ux.md` 不重复：该提案解决 desktop bundled runtime 的进程锁、启动接管和组件恢复；本提案只把桌面作为 workspace 绑定入口之一，不改变 sidecar ownership 或启动策略。
- 与 `skill-runtime-multi-agent-alignment.md` 不重复：该提案解决 skill frontmatter、active skill state 与 multi-agent runner 阶段语义；本提案只在后期建议 workspace-aware 只读工具，不改 skill runtime 的核心状态机。
- 本提案补的是当前 repo 中 actor-scoped 数据模型与跨渠道用户体验之间的产品架构缺口：让隔离边界继续存在，同时为真实用户建立可见、可管理、可审计的共享投资工作区。
