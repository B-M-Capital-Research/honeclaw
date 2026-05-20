# Proposal: Collaborative Research Rooms for Small Investment Teams

status: proposed
priority: P2
created_at: 2026-05-21 02:18:00 +0800
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
- `docs/proposal/auto_p1_linked-user-workspace.md`
- `docs/proposal/auto_p2_shareable-investment-briefs.md`
- `docs/proposal/auto_p2_investment-committee-mode.md`
- `docs/proposal/auto_p0_operator-access-audit.md`
- `docs/proposal/auto_p1_research_artifact_library.md`
- `docs/proposal/auto_p1_user-data-trust-center.md`
- `memory/src/web_auth.rs`
- `memory/src/portfolio.rs`
- `memory/src/company_profile/{storage,transfer}.rs`
- `memory/src/cron_job/storage.rs`
- `crates/hone-core/src/actor.rs`
- `crates/hone-channels/src/ingress.rs`
- `crates/hone-channels/src/agent_session/mod.rs`
- `crates/hone-channels/src/prompt.rs`
- `crates/hone-web-api/src/routes/public.rs`
- `crates/hone-web-api/src/routes/web_users.rs`
- `crates/hone-web-api/src/routes/company_profiles.rs`
- `crates/hone-web-api/src/routes/users.rs`
- `packages/app/src/pages/public-me.tsx`
- `packages/app/src/pages/public-portfolio.tsx`
- `packages/app/src/pages/users.tsx`
- `packages/app/src/components/chat-share-modal.tsx`

## 背景与现状

Hone 当前的产品形态已经覆盖个人投资助手的大部分基础面：public Web 登录、管理端、桌面端、多渠道 IM、公司画像、持仓/关注列表、定时任务、事件推送和技能系统。README 把它定位为“专业投资助手”，而不是普通聊天玩具。

代码和文档显示，当前系统的核心边界仍是个人或单 actor：

- `ActorIdentity(channel, user_id, channel_scope)` 是持仓、cron、公司画像、quota、sandbox 和权限隔离的主键。
- `SessionIdentity` 已经能表达群聊共享 session，但它主要解决“这条消息写进哪个会话”，不是一个可管理、可授权、可长期沉淀资产的团队空间。
- `memory/src/web_auth.rs` 的 public Web 用户是邀请制手机号白名单；一个 active phone 映射为一个 `web` actor。
- `memory/src/portfolio.rs` 按 actor 保存 JSON 持仓；公司画像也按 actor sandbox 保存，Web UI 只读展示，创建和更新仍通过 agent file operation。
- `crates/hone-web-api/src/routes/company_profiles.rs` 已有 actor-scoped export / import / preview / apply，说明跨空间搬运是显式动作，但还不是多人实时协作。
- `packages/app/src/pages/public-me.tsx` 和 `public-portfolio.tsx` 面向单个登录用户展示账户、投资上下文和只读画像；没有成员、评论、共同决策、共享 room 或邀请他人参与的概念。
- 管理端 `/users` 可以按 actor 查看用户资产，但这是 operator 视角，不是终端用户之间的协作视角。

同时，现有提案已经覆盖了许多相邻能力：`Linked User Workspace` 解决同一个真实用户跨 Web/IM/桌面的资产连续性；`Shareable Investment Briefs` 解决公开脱敏分享；`Investment Committee Mode` 解决多 agent 角色审查；`Operator Access Audit` 解决管理端操作者权限。这些都没有直接解决“两个或多个人在同一个私密投研空间内共同讨论、标注、请 Hone 审查和留存决策”的产品形态。

## 问题或机会

Hone 的长期价值是投资研究资产，而真实使用场景并不总是单人完成。很多潜在高价值用户会和配偶、合伙人、投资小组、研究同事、客户顾问或社群核心成员共同讨论同一组公司和风险。当前他们只能：

- 在群聊里 @ Hone，但群聊 session 没有稳定的成员权限、资产边界和决策留存。
- 把聊天回答导出成图片或未来 brief，但那是单向传播，不支持私密协作。
- 通过公司画像 export/import 复制资料，但这更像搬运包，不是共同维护。
- 让管理员代看多个 actor，但这不是用户之间的平等协作，也不适合商业团队套餐。

这会影响四条链路：

- 用户体验：多人讨论的上下文分散在 IM 群、Web chat、个人画像和截图里，Hone 无法知道哪些问题已经达成共识、哪些结论仍有异议。
- 研发架构：直接把 group chat 当成团队 workspace 会混淆 `SessionIdentity` 和长期资产归属；直接共享个人 actor 数据又会破坏隐私边界。
- 商业化：家庭/团队/顾问型付费需要 seat、role、room、共享资产和审计，而不是单个 actor 的 daily quota。
- 增长：Hone 的严肃投研定位很适合“邀请一个人一起审查这个 thesis”，但当前只有公开分享或人工复制，没有私密协作入口。

因此建议新增 **Collaborative Research Rooms**：一个明确的、私密的、成员制投研空间。它不替代个人 workspace，也不把所有个人资产自动共享；它只承载被成员显式带入的研究对象、讨论、Hone 审查结果和共同决策记录。

优先级为 P2：它有清晰商业和产品价值，但依赖身份、权限、数据信任和 workspace 基础逐步成熟，不应打断当前 P0/P1 的安全、运行可靠性和个人核心体验建设。

## 方案概述

新增 `ResearchRoom` 产品层，用于小型多人协作研究。

核心对象：

- `ResearchRoom`：一个私密研究空间，包含 room id、名称、owner、状态、创建时间、默认语言、共享风险边界。
- `RoomMember`：成员与角色，角色建议从 `owner`、`editor`、`commenter`、`viewer` 起步。
- `RoomAsset`：被显式加入 room 的研究对象，例如 ticker、company profile snapshot、research artifact、share brief draft、digest item 或用户上传文档摘要。
- `RoomThread`：围绕某个 asset 或主题的讨论串，可来自 Web、桌面或绑定的 IM 群。
- `RoomDecision`：一次共同结论或待办，例如“继续观察毛利率趋势，不改变长期 thesis”“需要补三份证据后再更新画像”。
- `RoomAgentRun`：Hone 在 room 中执行的一次审查，保存用户可见摘要、引用 asset、参与成员、是否建议写入个人/room 画像。

关键原则：

- 不自动共享个人持仓、成本、完整公司画像或会话历史。
- 个人 actor 仍是认证、quota、sandbox 和私有资产的安全边界。
- room 是显式协作层：成员主动把某个摘要、snapshot 或 artifact 加入 room。
- room 内的 agent 读取 room asset，不直接切换到某个成员的 actor sandbox。
- room 决策不等于交易指令；继续沿用 `prompt.rs` 的投资安全边界。

## 用户体验变化

### 用户端

- Public `/me` 增加 `Research rooms` 区块：用户可以创建 room、邀请成员、查看自己加入的 room。
- Public `/portfolio` 的公司主线和画像旁增加 `Add to room` 动作；默认加入的是脱敏 snapshot，而不是完整个人持仓和成本。
- Room 页面包含四个基本区块：
  - `Assets`：共享 ticker、画像摘要、报告、digest item、brief draft。
  - `Threads`：围绕 asset 的讨论和 Hone 回复。
  - `Decisions`：已确认结论、open questions、后续验证条件。
  - `Members`：成员、角色、最近活动、邀请状态。
- 用户可以在 room 中 @Hone，例如“请总结大家对 MU 的分歧”“把这个结论和我个人画像对比，但不要暴露我的成本”。
- 当 Hone 需要读取某个成员的个人资产时，必须显示授权请求，例如“是否允许本次 run 使用你的 AAPL 画像摘要作为输入”。默认只读 room 已共享资产。

### 管理端

- `/users` 可以看到某个 public user 参与了哪些 room，但默认不显示 room 私密讨论正文，除非 operator 具备明确 support/debug scope。
- 管理端提供 room 健康视图：成员数、资产数、agent run 成功率、被阻止的隐私读取、最近高风险操作。
- 对商业化运营，room 可成为 team plan 的最小组织单元：seat 数、room 数、共享 artifact 数、agent run 用量。

### 桌面端

- Desktop remote mode 复用 Web room API；bundled local mode 可以创建本机私有 room，但邀请外部成员需要连接远程 backend。
- 桌面侧适合显示 room activity tray：新的评论、Hone 审查完成、待确认决策。
- 本地文件或报告拖入 room 时先生成摘要/preview，再由用户确认是否共享原文。

### 多渠道

- IM 群可以显式绑定到某个 room：`/room link <code>`。绑定后，群内 @Hone 的协作讨论写入 `RoomThread`，但群成员身份仍需通过 room invite 映射或标记为 external participant。
- 私聊用户可以发送 `/room add MU`、`/room ask <room> ...`，把当前问题投到指定 room。
- 未绑定 room 的普通群聊维持现状，不自动成为 research room，避免把临时聊天误当作持久团队资产。

## 技术方案

### 1. 数据模型

建议在 `memory` 新增 `research_room` SQLite 存储：

```text
research_rooms (
  room_id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  owner_actor_channel TEXT NOT NULL,
  owner_actor_user_id TEXT NOT NULL,
  owner_actor_scope TEXT,
  status TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
)

research_room_members (
  room_id TEXT NOT NULL,
  member_actor_channel TEXT NOT NULL,
  member_actor_user_id TEXT NOT NULL,
  member_actor_scope TEXT,
  role TEXT NOT NULL,
  status TEXT NOT NULL,
  invited_by_actor_json TEXT,
  joined_at TEXT,
  created_at TEXT NOT NULL,
  PRIMARY KEY (room_id, member_actor_channel, member_actor_user_id, member_actor_scope)
)

research_room_assets (
  asset_id TEXT PRIMARY KEY,
  room_id TEXT NOT NULL,
  asset_type TEXT NOT NULL,
  source_ref_json TEXT NOT NULL,
  title TEXT NOT NULL,
  summary_markdown TEXT NOT NULL,
  sensitivity TEXT NOT NULL,
  added_by_actor_json TEXT NOT NULL,
  created_at TEXT NOT NULL,
  revoked_at TEXT
)

research_room_threads (
  thread_id TEXT PRIMARY KEY,
  room_id TEXT NOT NULL,
  asset_id TEXT,
  title TEXT NOT NULL,
  status TEXT NOT NULL,
  created_by_actor_json TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
)

research_room_messages (
  message_id TEXT PRIMARY KEY,
  thread_id TEXT NOT NULL,
  actor_json TEXT,
  role TEXT NOT NULL,
  content_markdown TEXT NOT NULL,
  source_channel TEXT,
  created_at TEXT NOT NULL
)

research_room_decisions (
  decision_id TEXT PRIMARY KEY,
  room_id TEXT NOT NULL,
  asset_id TEXT,
  status TEXT NOT NULL,
  decision_markdown TEXT NOT NULL,
  open_questions_json TEXT NOT NULL,
  created_by_actor_json TEXT NOT NULL,
  confirmed_by_json TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
)
```

`ActorIdentity` 不被替换。Room 只在成员 ACL 和共享资产层引用 actor。这样能保留现有 per-actor storage，同时避免把 room 伪装成个人 actor。

### 2. 权限与隐私

第一版定义最小权限：

- `viewer`：读取 room assets、threads、decisions。
- `commenter`：新增 comment 和发起 Hone 问答。
- `editor`：新增/撤回 assets、创建 decision draft。
- `owner`：管理成员、角色、room 归档和导出。

资产共享必须经过 summary/snapshot 层：

- Portfolio：默认只共享 ticker、方向性 thesis 和可选区间化权重；不共享 shares、avg_cost、具体成本。
- Company profile：默认共享摘要 snapshot；完整 `profile.md` 需要用户单独确认，且只复制到 room asset，不授予其他成员读取原 actor sandbox。
- Research artifact：共享 artifact 摘要和公开 sections；原文/PDF 需要 owner 权限或 artifact policy。
- Chat answer / digest item：先过 redaction policy，再作为 room asset。

### 3. API

Public API 从当前登录用户推导 actor：

- `GET /api/public/research-rooms`
- `POST /api/public/research-rooms`
- `GET /api/public/research-rooms/:id`
- `POST /api/public/research-rooms/:id/invites`
- `POST /api/public/research-rooms/:id/assets`
- `POST /api/public/research-rooms/:id/threads`
- `POST /api/public/research-rooms/:id/threads/:thread_id/messages`
- `POST /api/public/research-rooms/:id/decisions`
- `POST /api/public/research-rooms/:id/agent-runs`

Admin API 只做 support/diagnostic，不作为主要协作入口：

- `GET /api/research-rooms?actor=&status=`
- `GET /api/research-rooms/:id/health`
- `POST /api/research-rooms/:id/archive`

IM pre-session intercept 增加：

- `/room link <code>`
- `/room list`
- `/room ask <room> <question>`
- `/room add <ticker|artifact>`

### 4. Agent runtime

Room 内 agent run 使用当前请求者 actor 执行，但输入上下文来自 room：

1. 校验请求者是 room member 且具备 `commenter` 或以上权限。
2. 读取 room assets、thread summary、open decisions。
3. 如果用户请求对比个人资产，生成一次性授权 request，只把授权后的摘要加入 run。
4. 通过 `AgentSession::run()` 或后续 room-specific transient run 执行。
5. 将用户可见回答写入 `research_room_messages`，必要时生成 `decision draft`。

不要让 runner cwd 指向 room 存储目录，也不要把其他成员 actor sandbox 作为当前 runner 文件系统。跨成员资产读取只走受控 API/tool，并记录 room audit event。

### 5. 前端

新增模块建议：

- `packages/app/src/lib/research-rooms.ts`
- `packages/app/src/context/research-rooms.tsx`
- `packages/app/src/pages/public-rooms.tsx`
- `packages/app/src/pages/public-room-detail.tsx`
- `packages/app/src/components/research-room-*.tsx`

导航上，public `/me` 增加 room 入口；`/portfolio` 和 chat share modal 可以把 selected answer / ticker / digest item 加入 room。管理端可先只在 Users detail 增加 “rooms” tab。

## 实施步骤

### Phase 1: Room shell and membership

- 新增 `research_room` storage 与单元测试。
- Public Web 支持创建 room、列出 room、邀请/加入成员、角色校验。
- `/me` 展示 joined rooms。
- 不接 agent，不共享个人资产，只验证协作空间和权限。

### Phase 2: Asset snapshots

- 支持从 public portfolio 添加 ticker/mainline snapshot。
- 支持从 chat answer 添加 redacted message snapshot。
- 支持从 research artifact 或 company profile 添加摘要 snapshot。
- 增加撤回 asset、查看 redaction/sensitivity 的 UI。

### Phase 3: Threads and Hone room runs

- 增加 room thread、comment、@Hone room run。
- Room run 只读取 room assets 和当前 thread，不读取成员个人数据。
- Web/desktop 显示 Hone 输出、open questions 和 decision draft。

### Phase 4: IM group binding

- 增加 `/room link <code>`，把一个 IM 群或私聊入口绑定到 room thread。
- 群内消息只在明确 @Hone 或 `/room ask` 时进入 room。
- 外部群成员如果没有 room membership，显示为 `external_participant`，不能读取 Web room。

### Phase 5: Commercial and governance extension

- 与 usage entitlement ledger 对接 seat、room、agent run 和 shared asset limits。
- 与 operator audit / data trust center 对接 room export、delete、support access。
- 增加 room-level retention、archive 和 export。

## 验证方式

- Rust 单元测试：
  - room 创建、成员角色、邀请加入、角色降级、归档。
  - 非成员不能读取 room；viewer 不能写 comment；commenter 不能添加 asset；editor 不能管理成员。
  - asset snapshot 不包含 shares、avg_cost、actor sandbox path、session token。
- Web API 测试：
  - public API 忽略任意 actor query，只使用当前 session actor。
  - 同一 actor 被移出 room 后无法继续读取 detail 或写入 thread。
  - revoked asset 不再进入 room agent run context。
- 前端测试：
  - room list、member role badge、asset sensitivity badge、thread empty state。
  - `/me`、room detail、portfolio add-to-room 在移动端和桌面端布局不溢出。
- 多渠道手工回归：
  - Web 创建 room 和 link code，Telegram/Discord/Feishu 群 `/room link` 后 @Hone 写入 room thread。
  - 未绑定 room 的群聊不产生 room asset。
  - 非成员从 Web 打开 room URL 被拒绝。
- 产品指标：
  - room 创建率、成员邀请接受率、asset 添加率、room 内 Hone run 成功率、decision draft 确认率。
  - 对比普通分享链路，观察 room 成员的 7 日留存和付费转化。

## 风险与取舍

- 风险：多人协作会放大隐私泄露风险。取舍：默认只共享 snapshot，不共享完整个人资产；个人资料读取需要一次性授权。
- 风险：与 `Linked User Workspace` 混淆。取舍：workspace 表示同一真实用户跨渠道资产；room 表示多个成员的私密协作空间，二者 owner、权限和数据流不同。
- 风险：与群聊 session 重叠。取舍：群聊 session 只解决聊天上下文；room 是显式成员制资产空间，必须通过 `/room link` 绑定。
- 风险：早期实现可能变成复杂项目管理工具。取舍：第一版只做 assets、threads、decisions 和 Hone runs，不做任务看板、文件夹、实时协同编辑或社交 feed。
- 风险：room 决策被理解成交易建议。取舍：决策记录应表达 thesis、证据、open questions 和风险条件，不表达买卖指令；输出继续经过金融安全约束。
- 风险：support/operator 访问 room 内容会引入合规压力。取舍：admin 默认只看 health metadata；读取正文需要明确 scope、reason 和 audit。

## 与已有提案的差异

查重范围覆盖 `docs/proposal/` 与 `docs/proposals/` 下全部现有提案，重点比对如下：

- 与 `auto_p1_linked-user-workspace.md` 不重复：该提案把同一个真实用户的多个 channel actor 绑定为一个 workspace；本提案处理多个真实用户共同参与的私密 research room。它不会自动合并个人资产，也不替代 workspace 身份归并。
- 与 `auto_p2_shareable-investment-briefs.md` 不重复：shareable briefs 是公开或半公开的单向传播对象；research rooms 是成员制、私密、多轮讨论和决策留存空间。
- 与 `auto_p2_investment-committee-mode.md` 不重复：investment committee 是多 agent/多角色研究生成模式；research rooms 是多人协作容器，可以调用 committee mode，但核心是成员、资产、讨论和共同决策。
- 与 `auto_p0_operator-access-audit.md` 不重复：operator audit 管管理端人员和高危后台操作；research rooms 面向 end-user 协作，support 访问只作为受控诊断扩展。
- 与 `auto_p1_research_artifact_library.md` 不重复：artifact library 保存研究报告和画像交接；research rooms 可以引用 artifact 摘要，但它关注多人围绕 artifact 的讨论和决策。
- 与 `auto_p1_user-data-trust-center.md` 不重复：data trust center 解决个人数据清单、导出和删除；research rooms 需要接入它，但新增的是共享空间里的 membership、room asset 和协作记录。

## 文档同步说明

本轮只新增 proposal，不开始实现，因此不更新 `docs/current-plan.md`，也不归档计划页。若后续实际落地本提案，应新增或复用 `docs/current-plans/collaborative-research-rooms.md`，并在引入 room storage、public room API、IM `/room` 命令或 room-level entitlement 时同步更新 `docs/repo-map.md`、`docs/invariants.md`、`docs/decisions.md`，必要时补充 room 隐私与协作权限 ADR。
