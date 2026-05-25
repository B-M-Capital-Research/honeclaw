# Proposal: Context Return Links for Actionable Investment Surfaces

status: proposed
priority: P1
created_at: 2026-05-25 08:05:13 +0800
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
- `docs/proposal/auto_p1_workspace-command-palette.md`
- `docs/proposal/auto_p1_instrument_identity_registry.md`
- `docs/proposal/auto_p1_public-pwa-notification-bridge.md`
- `docs/proposal/auto_p1_delivery_decision_loop.md`
- `docs/proposal/auto_p1_investment-thread-workbench.md`
- `packages/app/src/app.tsx`
- `packages/app/src/pages/layout.tsx`
- `packages/app/src/context/symbol-drawer.tsx`
- `packages/app/src/components/symbol-drawer.tsx`
- `packages/app/src/components/entity-ref-link.tsx`
- `packages/app/src/components/symbol-link.tsx`
- `packages/app/src/pages/public-portfolio.tsx`
- `packages/app/src/components/user-mainline-view.tsx`
- `crates/hone-web-api/src/routes/mod.rs`
- `crates/hone-web-api/src/routes/public.rs`
- `crates/hone-web-api/src/routes/public_digest.rs`
- `crates/hone-web-api/src/routes/event_engine_admin.rs`
- `crates/hone-web-api/src/routes/notifications.rs`
- `crates/hone-event-engine/src/sinks/feishu_card.rs`
- `crates/hone-event-engine/src/sinks/discord_embed.rs`
- `crates/hone-event-engine/src/event.rs`

## 背景与现状

Honeclaw 已经从聊天助手扩展成多个投资工作表面：公开用户端有 `/chat`、`/me`、`/portfolio`，管理端有 sessions、users、tasks、research、notifications、schedule、settings，桌面端复用同一 Web console 并承担本机 runtime 和渠道管理，多渠道侧通过 Feishu、Telegram、Discord、iMessage 和 public Web 收发消息。

代码里已经出现了“从一个对象回到相关上下文”的多个雏形：

- `packages/app/src/context/symbol-drawer.tsx` 使用 `?symbol=AAPL` 控制 admin 端标的抽屉，支持 URL 同步、后退关闭和分享当前 symbol 状态。
- `packages/app/src/components/symbol-drawer.tsx` 把一个 symbol 关联到当前 actor 的画像、研究任务、会话和 watchlist 操作。
- `packages/app/src/components/entity-ref-link.tsx` 已经把日志、审计和任务里的 actor/session/task 渲染成可点击 chip，并在注释里明确 `symbol / research / skill` 是后续 phase。
- `packages/app/src/pages/public-portfolio.tsx` 和 `components/user-mainline-view.tsx` 都能按 ticker 打开只读公司画像，但前者面向当前 public user，后者面向 admin 指定 actor。
- `crates/hone-web-api/src/routes/notifications.rs` 已经在 delivery detail 里保留 `event_id`、`event_kind`、`symbols`、`title`、`summary`、`url` 等信息，说明通知记录有足够上下文生成跳转入口。
- `crates/hone-event-engine/src/sinks/feishu_card.rs` 和 `discord_embed.rs` 会在 digest 中展示 symbol、headline 和外部 URL，但没有回到 Hone 内部上下文的稳定链接。
- `crates/hone-web-api/src/routes/public_digest.rs` 和 `event_engine_admin.rs` 为 public/admin 分别提供 mainline 和 company profile 读取 API，实际产品对象一致，但 URL、actor 解析和权限边界分散在不同 surface。

这些能力说明 Hone 已有基础对象和局部跳转，但缺少一个统一的 **Context Return Link** 契约：当用户从通知、聊天答案、日志、画像、持仓、研究任务或桌面状态进入某个对象时，系统应该稳定地把人带到“正确用户、正确标的、正确证据、正确下一步”。

## 问题或机会

这是 P1 级机会，因为 Hone 的核心价值不是单次回答，而是让用户在多渠道长期维护投资上下文。主动提醒和长期记忆越多，用户越需要从任何入口回到具体上下文继续处理。

当前缺口主要有五类：

1. **通知能触达，但不能稳定回到 Hone 上下文。**  
   Event-engine digest 可以告诉用户某个 symbol 出现事件，但 Feishu/Discord 卡片主要链接外部新闻 URL。用户点完外部来源后，仍要手工回到 `/portfolio`、找 ticker、打开画像、再问 chat 是否更新 thesis。

2. **同一对象在 public/admin/desktop 的 URL 语义不统一。**  
   Public `/portfolio` 只知道当前登录用户，admin `/users/:actor/mainline` 需要 actor key，admin `?symbol=` 依赖当前选中 actor。桌面 remote/bundled 还要处理本机 backend base URL。缺少统一 link resolver 会导致每个页面各写自己的跳转规则。

3. **EntityRefLink 已经有对象 chip，但还不是跨 surface 契约。**  
   现在 `EntityRefLink` 是前端组件级 helper，Phase 1 支持 actor/session/task。它没有服务端生成、权限降级、public token、桌面 base URL、通知 payload 或 IM 文案约束，因此还不能成为外部回流链接的底座。

4. **多渠道用户缺少“处理下一步”的入口。**  
   用户在 IM 里收到提醒，合理下一步可能是打开公司画像、补充 watchlist、进入 chat 让 Hone 复盘这条事件、查看为什么被推送、调整通知偏好或创建研究任务。现在这些动作散在多个页面，提醒文本也没有携带可执行上下文。

5. **后续提案会反复需要 deep link。**  
   Public PWA Notification Bridge、Delivery Decision Loop、Evidence Review Queue、Research Artifact Library、Temporal Operations Calendar、Workspace Command Palette 都提到或隐含 deep link。如果没有统一契约，后续会在各模块临时拼 URL、重复处理权限和路由漂移。

机会是先建立一个轻量、可测试、权限感知的链接层，把现有对象连起来。第一版不需要新的金融数据源，也不要求完成 workspace 或 instrument registry。它先解决“用户点哪里、落到哪里、为什么可以看、下一步是什么”。

## 方案概述

新增 **Context Return Link** 产品与架构层，定义从任意 Hone 对象生成可解释、可降级、可审计的内部链接。

核心对象：

- `ContextTarget`：描述目标对象，例如 `actor`、`session`、`task`、`research_task`、`symbol`、`company_profile`、`portfolio_holding`、`notification_event`、`digest_item`、`skill`、`settings_section`。
- `ContextLink`：可渲染链接，包含 `surface`、`href`、`label`、`target`、`actor`、`symbol`、`expires_at`、`requires_auth`、`fallback_href`、`actions`。
- `ContextAction`：低副作用下一步，例如 `open_profile`、`open_chat_with_prompt`、`explain_delivery`、`open_notification_prefs`、`start_research_draft`、`add_watchlist_draft`。
- `ContextLinkResolver`：后端或前端共享的解析规则，输入 target + requester surface，输出可用链接和降级原因。
- `context_ref` 标记：在通知记录、chat response metadata、public push payload、admin chips 和未来 artifacts 中保存稳定目标，不只保存拼好的 URL。

第一版目标：

1. 统一 admin 内部对象跳转，替换散落的 URL 拼接。
2. 为 public Web 当前登录用户生成安全链接，例如 `/portfolio?symbol=AAPL&focus=profile` 或 `/chat?context=...`。
3. 为 event-engine digest/notification 生成 Hone 内部回流入口，同时保留外部新闻 URL。
4. 让 IM 卡片、PWA push、public portfolio、admin notifications 和 logs 使用同一 link resolver。
5. 明确不可访问时的降级：未登录去登录，非当前 actor 去 `/chat` 或 public home，admin-only 对象在 public 端不暴露。

## 用户体验变化

### 用户端

- Public `/portfolio` 支持带上下文进入：`?symbol=AAPL&focus=mainline/profile/notification`。页面自动滚到对应持仓或画像卡片，打开只读画像 modal 或提示先登录。
- Public `/chat` 支持 context draft：用户从通知进入后，输入框预填“请基于这条 AAPL 事件复盘公司画像，说明 thesis 是否改变”，用户确认发送后才交给 agent。
- 每条 public digest/notification 显示两个入口：`查看来源` 打开外部新闻，`回到 Hone` 打开投资上下文或 chat draft。
- 如果用户没有公司画像，落地页显示明确下一步：“建立 AAPL 公司画像”或“加入 watchlist 后再开启提醒”，而不是空白或 404。

### 管理端

- `EntityRefLink` 升级为 `ContextRefLink`，支持 actor、session、task、research、skill、symbol、company_profile、notification_event。
- Notifications 页面里每条 event-engine 记录可以直接跳到：对应 actor 的 mainline、portfolio symbol、delivery decision detail、外部 URL、通知偏好。
- Logs / LLM audit / task-health 中的 session、actor、task、symbol 使用统一 chip，减少维护者手动复制 ID。
- 当 actor 不存在、session 已删除或 profile 未建档时，chip 显示 degraded 状态，并提供可执行 fallback，例如去用户页或打开通知原始 detail。

### 桌面端

- Desktop bundled 通过 backend meta 暴露当前 admin/public base URL，link resolver 生成本机可打开的相对路径，避免硬编码生产域名。
- Desktop remote 模式生成 remote backend 的 public/admin 链接，但只在当前用户已连接 remote 时展示。
- 渠道状态、日志和通知弹窗可以复用同一 context link，不需要 desktop shell 单独理解每个业务对象。

### 多渠道

- Feishu / Discord digest 卡片在每个 item 下可增加一条短链接或按钮：`在 Hone 查看`。第一版可只对 public Web actor 或配置了 `public_base_url` 的部署启用。
- Telegram 文本场景使用短 URL 或简洁 markdown link，不在消息里暴露 actor key、sandbox path 或本机 file path。
- iMessage 若不适合富链接，保留短文本 URL；链接打开后仍走同一 public/admin resolver。
- 群聊默认不生成个人 actor 的 public 链接；只有 group actor 或已明确绑定 workspace/group surface 时才生成可访问入口。

## 技术方案

### 1. 定义 ContextTarget 和 ContextLink schema

建议先放在 Web API 层或一个小的 shared module，避免立刻把它扩展成全局复杂系统：

- `ContextTargetKind`：`actor | session | task | research | symbol | company_profile | portfolio_holding | notification_event | skill | settings_section`。
- `ContextSurface`：`admin | public | desktop_admin | desktop_public | channel_text | channel_card`。
- `ContextTarget` 字段：`kind`、`id`、`actor`、`symbol`、`event_id`、`session_id`、`profile_id`、`task_id`、`source_url`。
- `ContextLink` 字段：`href`、`label`、`requires_auth`、`allowed_surfaces`、`fallback_href`、`degraded_reason`、`actions`。

第一版可以 JSON-only，不需要数据库表。长期可把生成出的 context links 写入 delivery log 或 product event plane，用于点击率和排障。

### 2. 服务端提供受控解析 API

新增只读 API，例如：

- `POST /api/context-links/resolve`：admin surface，允许解析指定 actor 对象。
- `POST /api/public/context-links/resolve`：public surface，只能解析当前登录 Web actor 可访问的目标。
- `GET /api/context-links/notification/{event_id}`：把 notification record 投影为 link/action 列表。

public 端不接受任意 actor 参数；actor 必须由 session cookie 推导。admin 端仍要遵守未来 operator access/audit 的权限边界。

### 3. URL contract

为现有页面补齐稳定 query 约定：

- `/portfolio?symbol=AAPL&focus=profile|mainline|holding|chat`
- `/chat?context=<opaque_or_signed_id>` 或 `/chat?symbol=AAPL&intent=review_event&event_id=...`
- `/users/:actorKey/mainline?symbol=AAPL`
- `/users/:actorKey/profiles?profile=<profile_id>`
- `/users/:actorKey/portfolio?symbol=AAPL`
- `/notifications?event_id=<event_id>`
- `/tasks/:taskId`
- `/research/:taskId`
- `/sessions/:sessionId`
- `/skills/:skillId`

`context` 若携带较多敏感摘要，应使用短期 opaque id 或 signed token，而不是把完整事件摘要、actor key、内部路径塞进 URL。

### 4. 前端复用组件

- 将 `EntityRefLink` 扩展或替换为 `ContextRefLink`，从 target 生成 link，不再由组件内部手写所有 URL。
- `SymbolDrawer` 保持 `?symbol=` 能力，但增加 `focus`、`actor` degraded handling，并与 `ContextLinkResolver` 的 symbol target 对齐。
- `public-portfolio.tsx` 与 `user-mainline-view.tsx` 共享一套 focus/scroll/profile-open 行为，减少 public/admin 分叉。
- `notifications.tsx`、`logs.tsx`、`llm-audit.tsx`、`task-health.tsx` 使用统一 chips 和 action menu。

### 5. 多渠道生成策略

- Event-engine `DigestItem` 保留外部 `url`，新增可选 `context_target` 或在 sink 层从 item + actor 推导。
- Feishu/Discord card renderer 不直接拼业务 URL，而调用 link builder 或消费预先生成的 `ContextLink`。
- 文本通道只展示 1 个最有价值入口，避免每条 digest 变成链接噪音。
- 缺少 `public_base_url` 或 `admin_base_url` 时，不生成外部可点击 Hone 链接，只在管理端通知日志中显示 link actions。

### 6. 兼容和迁移

不需要迁移旧数据。旧 delivery log、cron run、session history 没有 `context_ref` 时，resolver 根据已有字段 best-effort 生成 degraded links：

- 有 `event_id + symbols`：生成 notification/event link 和 symbol link。
- 有 `actor + symbol`：生成 portfolio/mainline link。
- 有 `session_id`：生成 session link。
- 只有外部 `url`：保留来源链接，不生成 Hone 内部入口。

## 实施步骤

### Phase 1: Admin-only resolver and chips

1. 定义 `ContextTarget` / `ContextLink` TypeScript 类型和 Rust JSON DTO。
2. 改造 `EntityRefLink` 为 resolver-driven 组件，先覆盖 actor/session/task/symbol/research/skill。
3. 在 logs、LLM audit、notifications 中用统一 chip 渲染现有对象。
4. 为 `/users/:actorKey/*`、`/notifications?event_id=`、`?symbol=` 增加前端 focus 行为。

### Phase 2: Public portfolio and chat return flow

1. 为 `/portfolio?symbol=&focus=` 增加自动定位、打开 profile modal、缺 profile 的下一步提示。
2. 为 `/chat` 增加 safe context draft，不自动发送。
3. 新增 `/api/public/context-links/resolve`，只解析当前登录 actor。
4. 为 public notification/PWA 后续能力保留 link payload contract。

### Phase 3: Event-engine and channel links

1. 从 delivery log 和 digest item 生成 `ContextTarget`。
2. Feishu/Discord card 增加可配置 `在 Hone 查看` link。
3. Telegram/iMessage 采用短文本链接。
4. 在 Notifications 管理页显示每条记录的 link/actions 和 degraded reason。

### Phase 4: Metrics and hardening

1. 记录 link generated/opened/action selected 的隐私克制事件，可等待 product events plane 落地后接入。
2. 增加链接过期、签名、base URL 配置检查和安全测试。
3. 将 link resolver 纳入 smoke：public 当前用户、admin 指定 actor、缺 profile、缺 session、无 base URL、群聊 actor。

## 验证方式

- Unit tests：
  - `ContextTarget` 到 admin/public href 的解析覆盖 actor、session、task、symbol、notification、company_profile。
  - public resolver 拒绝非当前 actor、admin-only target 和包含内部路径的 target。
  - 缺 profile、缺 session、缺 base URL 返回 degraded link 而不是 panic/500。
- Frontend tests：
  - `/portfolio?symbol=AAPL&focus=profile` 会定位 AAPL 卡片并打开 profile modal 或显示建档提示。
  - `ContextRefLink` 对 symbol 调用 SymbolDrawer，对 session/task/research/skill 导航到稳定 URL。
  - Notifications 页面能从 event record 渲染 symbol、actor、event detail、source URL 和 Hone return link。
- Integration / regression：
  - 构造一条 event-engine delivery log，确认 `/api/admin/notifications` detail 可以生成对应 context target。
  - public 登录用户只能解析自己的 portfolio/company-profile target。
  - Feishu/Discord card renderer 在 base URL 缺失时不输出空链接，在存在 base URL 时输出合法 HTTPS 链接。
- Manual acceptance：
  - 从一条 digest 提醒进入 public portfolio，能在 2 次点击内看到对应 symbol 的主线或画像。
  - 从 admin notification record 进入 actor mainline，再回到 delivery decision detail，不需要复制 ID。
  - Desktop bundled/remote 打开的链接都指向当前可用 backend surface。
- Metrics：
  - notification click-through to Hone context。
  - notification to chat follow-up rate。
  - admin time-to-open relevant actor/session from notification/log。
  - degraded link rate by reason。

## 风险与取舍

- 风险：链接泄露 actor key、内部路径或私有上下文。取舍：public/channel URL 不包含 actor key、sandbox path、file path 或完整摘要；敏感 context 使用短期 opaque id 或 signed token。
- 风险：错误链接把用户带到错误 actor。取舍：public resolver 永远从 session 推导 actor；admin 链接展示 actor label 和 degraded 状态；group actor 不默认映射个人 actor。
- 风险：消息卡片变得过度拥挤。取舍：文本通道只展示一个 Hone return link；管理端保留完整 action menu。
- 风险：和未来 workspace/instrument registry 发生重叠。取舍：本提案只定义 target/link/action 契约，identity 解析仍可由 Instrument Registry 后续替换，跨 actor 范围仍由 Linked Workspace 后续提供。
- 风险：URL contract 变化造成旧链接失效。取舍：把 URL contract 写入 resolver 测试；旧链接保留 fallback route 或 graceful redirect。
- 风险：点击率指标诱导过度推送。取舍：link metrics 只用于判断回流是否顺畅，通知频率仍由 prefs、delivery decision 和 safety gate 控制。
- 不做：不自动执行买卖建议，不自动修改公司画像，不在未确认时发送 chat prompt，不把外部新闻内容全文写进 URL，不替代 command palette 或 full-text search。

## 与已有提案的差异

查重范围：已检查 `docs/proposal/` 与 `docs/proposals/` 下所有现有提案文件名和标题，并重点阅读 `auto_p1_workspace-command-palette.md`、`auto_p1_instrument_identity_registry.md`、`auto_p1_public-pwa-notification-bridge.md`、`auto_p1_delivery_decision_loop.md`、`auto_p1_investment-thread-workbench.md`、`auto_p1_research_artifact_library.md`、`auto_p1_evidence_review_queue.md`、`auto_p2_shareable-investment-briefs.md`、`auto_p1_product-rollout-kill-switch.md`。

- 与 `auto_p1_workspace-command-palette.md` 不重复：Command Palette 解决用户主动搜索和命令执行；本提案解决系统生成的通知、日志、聊天和资产对象如何带用户回到正确上下文。一个是 pull，一个是 return link。
- 与 `auto_p1_instrument_identity_registry.md` 不重复：Instrument Registry 解决 ticker/company/asset 身份解析；本提案可以消费它的结果，但本身只定义 context target 到 surface URL/action 的回流契约。
- 与 `auto_p1_public-pwa-notification-bridge.md` 不重复：PWA Bridge 让浏览器成为投递渠道；本提案定义通知点击后应该落到哪里、如何鉴权、如何降级。PWA 可复用本提案的 `NotificationDeepLink` 实现。
- 与 `auto_p1_delivery_decision_loop.md` 不重复：Delivery Decision Loop 解释为什么推送、过滤或降级；本提案提供从该解释或通知记录跳回 portfolio/profile/chat/prefs 的链接层。
- 与 `auto_p1_investment-thread-workbench.md` 不重复：Thread Workbench 组织长期议题；本提案不创建议题对象，只在已有对象之间建立可点击回路。
- 与 `auto_p1_research_artifact_library.md` 不重复：Research Artifact Library 管理报告资产；本提案只把 artifact/report 作为 future `ContextTarget`。
- 与 `auto_p1_evidence_review_queue.md` 不重复：Evidence Queue 把事件变成待处理复盘项；本提案只提供从事件或待办进入复盘上下文的 link/action。
- 与 `auto_p2_shareable-investment-briefs.md` 不重复：Shareable Briefs 面向外部公开分享和增长回流；本提案面向已授权用户或管理员在 Hone 内部回到私有上下文。
- 与 `auto_p1_product-rollout-kill-switch.md` 不重复：Rollout 提供功能灰度和关闭；本提案可被灰度控制，但不定义 rollout 系统。

差异结论：已有提案多次提到 deep link、target_url 或回流，但都把它作为各自功能的局部字段。当前仓库已经出现 `EntityRefLink`、`SymbolDrawer`、public/admin mainline 视图和 notification detail，说明统一回流契约已经到达可单独设计的时点。本提案补的是“对象到上下文”的横向产品架构层，不替代任何具体资产、搜索、通知或身份提案。

## 文档同步说明

本轮只新增 proposal，不开始实施，不修改业务代码、测试代码、运行配置或 `docs/current-plan.md`。若后续实际落地，应按动态计划准入标准新增或复用 `docs/current-plans/context-return-links.md`，并在引入 context link DTO、API、URL contract、多渠道卡片链接、public signed context 或前端路由 focus 行为时同步更新 `docs/repo-map.md`、`docs/invariants.md`、相关 runbook 和必要的 decision/ADR。
