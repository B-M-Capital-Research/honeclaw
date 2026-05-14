# Proposal: Shareable Investment Briefs for Trust-Based Growth

status: proposed
priority: P2
created_at: 2026-05-15 02:02:38 +0800
owner: automation

## related_files

- `README.md`
- `AGENTS.md`
- `docs/repo-map.md`
- `docs/invariants.md`
- `docs/decisions.md`
- `docs/current-plan.md`
- `docs/proposal/auto_p1_invite_activation_funnel.md`
- `docs/proposal/auto_p1_research_artifact_library.md`
- `docs/proposal/auto_p1_response-feedback-learning-loop.md`
- `docs/proposal/auto_p1_source-provenance-freshness.md`
- `docs/proposal/auto_p1_user-data-trust-center.md`
- `crates/hone-web-api/src/routes/public.rs`
- `crates/hone-web-api/src/routes/public_digest.rs`
- `crates/hone-event-engine/src/global_digest/renderer.rs`
- `crates/hone-event-engine/src/global_digest/{collector,curator,mainline_distill}.rs`
- `packages/app/src/pages/chat.tsx`
- `packages/app/src/components/chat-share-modal.tsx`
- `packages/app/src/components/chat-share-card.tsx`
- `packages/app/src/pages/__share-preview.tsx`
- `packages/app/src/pages/public-portfolio.tsx`
- `packages/app/src/lib/public-chat.ts`

## 背景与现状

Hone 已经具备几块很接近“可传播投资交付物”的基础设施：

- Public `/chat` 已有 `ChatShareModal` 和 `ChatShareCard`，用户可以从最近几条对话中选内容，渲染为带品牌、二维码和 Markdown 样式的 PNG，再下载、复制或系统分享。
- `packages/app/src/pages/__share-preview.tsx` 是一个 dev-only 预览页，用来验证分享卡片截图质量，说明前端已经开始把回答从纯文本转成视觉资产。
- Public `/portfolio` 通过 `public_digest.rs` 读取当前 Web actor 的持仓、公司画像摘要、投资主线蒸馏结果和 skipped ticker，并允许用户手动刷新 digest context。
- 全局 digest 管道已经能把事件候选、全文抓取、mainline relation、Pass 2 personalize 结果渲染成短消息，`renderer.rs` 明确区分印证、证伪和宏观类信息。
- 公司画像保持 actor sandbox 内的 Markdown 真相源，public 端只读展示，修改仍通过 agent 和 `company_portrait` skill 完成。

但这些能力仍停在“用户自己看”和“IM 内转发截图”的阶段。Hone 的公开产品叙事是严肃投资助手，不是泛聊天工具；如果用户得到一段有用的研究结论、每日 digest 或公司画像摘要，目前没有一个安全、可回流、可衡量的分享对象。分享 PNG 只包含当前几条消息，不知道其来源、新鲜度、脱敏边界、是否可公开、是否能把外部读者带回一个试用入口。

这让增长链路偏弱：Hone 的高质量输出天然适合被用户发到群、社媒、投资社群或给朋友看，但现有分享无法承载“这是一份可验证、可回到 Hone 继续追问的 brief”。

## 问题或机会

### 问题

1. **分享对象过于临时。**
   现有聊天分享卡只是前端截取对话内容生成图片。它不保存服务端 metadata，不记录 share id，不知道来自哪个 session/run，也不能被后续分析、撤回、过期或复用。

2. **投资内容缺少脱敏与合规边界。**
   公司画像、持仓、投资主线、digest comment、聊天回答都可能包含用户私有仓位、成本、偏好、内部路径或非公开上下文。用户一键分享前没有“公开安全预览”层，容易把私有数据带出系统。

3. **外部读者没有回流路径。**
   PNG 里的二维码当前只是固定指向 public chat。读者无法打开一份对应 brief 的公开 landing，也无法看到被允许公开的上下文、来源、时间、免责声明或试用入口。

4. **增长与质量无法衡量。**
   分享成功、打开、转化、注册、继续提问、负反馈等行为没有 share-level attribution。后续做 invite activation、feedback learning 或 entitlement 时，无法知道哪些内容真正带来高意向用户。

5. **digest 和公司画像的传播价值没有产品化。**
   全局 digest 已经会生成“印证 / 证伪 / 宏观”的个性化判断，public portfolio 已经展示投资主线和画像摘要，但它们没有“生成可公开 brief”的路径，只能靠用户复制文字或截图。

### 机会

新增 **Shareable Investment Briefs**：把聊天高价值回答、全球 digest 摘要、公司画像公开摘要和研究资产摘要转成一类 actor-scoped、可脱敏、可过期、可追踪的分享对象。

它的目标不是做社交平台，而是把 Hone 的核心价值转化为可信传播：

- 用户愿意分享的是“有判断、有证据、有边界的投资 brief”，而不是普通聊天截图。
- 外部读者看到的是可公开摘要、来源时间、风险声明和清晰 CTA，而不是用户私有工作区。
- 管理员能知道哪些 brief 带来试用、邀请申请、API key 兴趣或反馈。

## 方案概述

第一版新增一个轻量的 `ShareBrief` 层，不重写 chat、digest 或 company profile，只在它们输出后提供“生成公开 brief”的受控出口。

核心对象：

1. `ShareBrief`
   服务端记录 brief id、actor、source type、source refs、title、summary markdown、visibility、redaction status、expires_at、created_at、view_count、conversion refs。

2. `BriefSource`
   指向原始来源：chat session/run/message ids、global digest date/item ids、company profile id/ticker、research artifact id。第一版只要求 source ref 可追溯，不要求把完整原文复制进公开页。

3. `PublicBriefPayload`
   真正可公开展示的脱敏内容：标题、简短结论、允许公开的 bullet、来源时间、新鲜度标签、风险声明、CTA。它必须和原始 private workspace 分离。

4. `BriefRedactionPolicy`
   生成分享前的规则层：移除成本、股数、私有仓位权重、手机号、用户 id、本地路径、actor sandbox 绝对路径、内部 prompt、API key、未公开附件路径；可选保留 ticker、行业、公开新闻来源和模型生成结论。

5. `BriefAttribution`
   记录公开打开、二维码回流、invite request、public chat first message、feedback 等事件，用于增长与内容质量判断。

第一版建议只开放三类来源：

- Chat answer brief：从 public `/chat` 的一段回答生成。
- Digest brief：从全球 digest 的当日摘要或单条 personalized item 生成。
- Company thesis brief：从 public `/portfolio` 里某个 ticker 的只读画像摘要和 mainline distill 生成，不包含完整 profile markdown。

## 用户体验变化

### 用户端

- Public `/chat` 的分享按钮从“导出图片”升级为两步：
  1. 选择对话片段并预览分享卡片。
  2. 可选生成公开 brief 链接，系统提示会移除私有仓位、成本和本地路径。
- 分享卡片二维码不再只指向 `/chat`，而是可指向 `/brief/:id`；公开页底部再引导“用 Hone 继续问这个问题”。
- Public `/portfolio` 在每个 ticker 的投资主线旁提供“生成公开摘要”动作。默认只分享 ticker、公开主线摘要、最近更新时间和免责声明，不分享用户持仓数量、成本、完整画像或 event trail。
- Digest 页面或未来 digest 详情可以提供“分享今日三条要闻”或“分享这条证伪事件”的动作，保留来源链接与发生时间。
- 用户可以在 `/me` 查看自己创建过的 brief，并撤回或设置过期。

### 管理端

- Settings / Users 侧能看到 share brief 汇总：创建数、公开打开数、回流注册数、被撤回数、最近来源类型。
- 管理员可以配置默认 share policy：是否允许 public user 生成公开链接、默认过期天数、是否强制人工审核某些来源类型。
- 风险视图列出被 redaction policy 拦截的 brief，例如命中本地路径、疑似手机号、过长私有上下文或不符合投资输出安全规则。

### 桌面端

- Desktop bundled 仍可导出本地 PNG，但生成公开链接必须连接到 public/remote backend；本地单机模式只提供图片导出，避免意外暴露本机内容。
- Remote backend 模式复用 public brief API；桌面可以显示“share link ready / link disabled by server policy”的明确状态。

### 多渠道

- Feishu / Telegram / Discord 中不引入复杂编辑器；当用户回复“分享这条”时，agent 可以生成一张图片或返回 brief 链接。
- 群聊场景默认不允许把 shared group session 直接公开成 brief，除非后续 workspace/linking 有明确同意模型。第一版只允许 direct actor 或 public web user 主动创建。
- 多渠道发送 brief 时使用既有 outbound 渲染，但公开链接里的内容由 Web brief payload 承载，避免各渠道复制一份不同文本。

## 技术方案

### 1. 新增 share brief 存储

建议在 `memory` 增加 `share_brief` 模块，使用 SQLite 保存 metadata，公开 payload 可以存 JSON 或 Markdown 文件：

```text
share_briefs (
  brief_id TEXT PRIMARY KEY,
  actor_channel TEXT NOT NULL,
  actor_user_id TEXT NOT NULL,
  actor_scope TEXT,
  source_type TEXT NOT NULL,
  source_ref_json TEXT NOT NULL,
  title TEXT NOT NULL,
  payload_path TEXT,
  visibility TEXT NOT NULL,
  status TEXT NOT NULL,
  redaction_status TEXT NOT NULL,
  expires_at TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  revoked_at TEXT
)

share_brief_events (
  event_id TEXT PRIMARY KEY,
  brief_id TEXT NOT NULL,
  event_type TEXT NOT NULL,
  created_at TEXT NOT NULL,
  referrer TEXT,
  user_agent_hash TEXT,
  actor_user_id TEXT,
  metadata_json TEXT
)
```

公开 payload 示例：

```json
{
  "title": "为什么这条新闻可能证伪某持仓主线",
  "source_type": "global_digest_item",
  "generated_at": "2026-05-15T02:02:38+08:00",
  "freshness": "event_time_known",
  "body_markdown": "...",
  "public_sources": [
    { "label": "reuters.com", "url": "https://..." }
  ],
  "risk_disclaimer": "仅供研究讨论，不构成买卖建议。"
}
```

### 2. Redaction policy

新增 `BriefRedactor`，第一版采用规则优先，不依赖 LLM：

- 删除或拒绝 `file://`、actor sandbox 绝对路径、本机路径、上传路径。
- 删除 shares、avg_cost、portfolio_weight、phone、email、API key prefix 以外的完整 key、session token。
- 限制 source markdown 长度，防止把完整公司画像或长聊天历史公开。
- 对包含“买入/卖出/仓位建议”的内容加硬性免责声明；若后续 `Investment Output Safety Gate` 落地，可复用其 verdict。
- 输出 `redaction_report`，供 UI 显示“已移除 2 处私有仓位信息”。

对 digest item 可直接从 `PersonalizedItem` 生成公开 payload，因为它已经是摘要级内容；对 company profile 只能读取 public portfolio 当前已有的摘要/主线，不读取完整事件目录；对 chat answer 只允许用户选中的 assistant message 加上必要上下文，不自动带入整段 session。

### 3. API

新增 public API：

- `POST /api/public/share-briefs`
  - 从当前 `hone_web_session` 推导 actor。
  - body: source type、source ref、selected message ids 或 ticker/date。
  - 返回 draft preview、redaction report、是否可发布。
- `POST /api/public/share-briefs/:id/publish`
- `GET /api/public/share-briefs`
- `POST /api/public/share-briefs/:id/revoke`
- `GET /api/public/brief/:id`
  - 无需登录，只返回公开 payload；过期、撤回或 policy blocked 返回 404/410。
- `POST /api/public/brief/:id/events`
  - 记录 view/click/cta；只保存最小匿名信息。

Admin API：

- `GET /api/share-briefs?actor=&status=&source_type=`
- `GET /api/share-briefs/:id`
- `POST /api/share-briefs/:id/moderate`

### 4. 前端

- 扩展 `ChatShareModal`：保留现有 PNG 导出；新增“生成公开链接”步骤，显示 redaction report、过期时间和公开预览。
- 把 `ChatShareCard` 改成可接收 `briefUrl`，二维码优先指向 brief，fallback 指向 public chat。
- 新增 public `/brief/:id` 页面，使用比 landing page 更克制的阅读布局：标题、正文、来源、时间、免责声明、CTA。
- `public-portfolio.tsx` 增加 ticker brief action；只在 profile/mainline 存在时可用。
- 管理端新增 lightweight brief list，第一版可先挂到 Users 或 Settings，避免新建复杂运营模块。

### 5. 兼容与迁移

- 现有图片分享保持完全兼容；没有服务端能力时仍然只导出 PNG。
- brief API 需要通过 `/api/meta` capability 暴露，前端按能力隐藏公开链接功能。
- 不修改公司画像真相源；brief payload 是派生公开摘要，撤回 brief 不影响 profile/session/digest 原始数据。
- 不修改 `docs/current-plan.md`，除非后续实际开始实现该 proposal。

## 实施步骤

### Phase 1: Chat brief 最小闭环

- 新增 `ShareBrief` metadata 存储和 public create/preview/publish/revoke/read API。
- 只支持 public chat assistant message 生成 brief。
- 实现规则型 redaction，覆盖本地路径、仓位字段、联系方式、长文本截断。
- `ChatShareModal` 增加公开链接能力，二维码指向 `/brief/:id`。

### Phase 2: Portfolio thesis brief

- 从 `public_digest.rs` 已暴露的 digest context 和 profile summaries 生成 ticker thesis brief。
- Public `/portfolio` 增加“分享摘要”动作。
- 禁止分享完整 `profile.md` 和 events；只使用 mainline、更新时间、ticker、公开摘要。

### Phase 3: Digest brief

- 从 global digest renderer / personalized item 构造 brief payload。
- 支持分享“今日要闻摘要”与单条印证/证伪 item。
- 在 brief payload 中保留 public source URL、事件时间和 freshness hint。

### Phase 4: Attribution and admin controls

- 记录 brief view、CTA click、invite/register attribution。
- Settings / Users 展示 brief stats 与风险拦截项。
- 增加 server policy：禁用公开链接、默认过期、敏感来源人工审核。

## 验证方式

- 单元测试：
  - `BriefRedactor` 删除 `file://`、sandbox 绝对路径、shares/avg_cost、手机号和疑似 key。
  - public payload 不包含 actor user id、session token、本地路径、完整 profile markdown。
  - revoked/expired brief 返回 404 或 410。
- API 测试：
  - public create brief 只能读取当前 cookie actor，不能接受任意 actor query。
  - 未登录用户只能读取已发布且未过期的 brief，不能读取 draft metadata。
  - publish 前必须有 redaction report。
- 前端测试：
  - `ChatShareModal` 在无 capability 时仍能 PNG 导出。
  - 有 capability 时显示公开预览、过期时间、redaction summary 和 brief URL。
  - `/brief/:id` 在移动端和桌面端不遮挡正文、来源和 CTA。
- 手工验收：
  - 从 public chat 生成一条 brief，分享到外部浏览器，打开后能看到公开页和 CTA。
  - 从 public portfolio 生成 ticker thesis brief，确认不包含持仓数量、成本或完整画像。
  - 撤回 brief 后公开 URL 不再显示内容。
- 指标：
  - share brief 创建率、publish 成功率、redaction block rate、外部 view、CTA click、invite/register attribution。

## 风险与取舍

- 风险：投资内容公开分享可能被误读为荐股。取舍：公开页必须带风险声明，默认不输出买卖动作；命中直接交易建议时需要 safety gate 或阻断。
- 风险：脱敏规则漏掉私有信息。取舍：第一版只允许摘要级来源和用户显式发布；完整 profile、完整 session、附件和 group chat 不进入 brief。
- 风险：公开链接成为隐私攻击面。取舍：brief id 使用高熵随机值，支持过期和撤回；公开 API 只读 payload，不暴露 actor metadata。
- 风险：增长统计变成过度追踪。取舍：brief event 只保存最小匿名 attribution，不存原始 IP；已登录转化再关联 actor。
- 风险：产品范围膨胀成内容管理系统。取舍：第一版只做创建、公开页、撤回和最小统计，不做评论、点赞、订阅或社交 feed。

## 与已有提案的差异

- 与 `auto_p1_research_artifact_library.md` 不重复：该提案把深度研究报告变成 actor-scoped 长期资产和画像交接对象；本提案把已生成的聊天/digest/画像摘要变成可脱敏公开传播的 brief。
- 与 `auto_p1_invite_activation_funnel.md` 不重复：该提案衡量 invite 用户激活阶段；本提案提供外部分享和回流 attribution，可作为其 evidence 来源之一。
- 与 `auto_p1_response-feedback-learning-loop.md` 不重复：该提案处理回答质量反馈和复盘队列；本提案处理用户主动公开分享、撤回和转化。
- 与 `auto_p1_source-provenance-freshness.md` 不重复：该提案建立来源证明和新鲜度注册；本提案只在公开 brief 中消费轻量来源/时间标签，不定义全局 provenance 真相源。
- 与 `auto_p1_user-data-trust-center.md` 不重复：该提案处理导出、删除、隐私和账户数据权利；本提案处理单个公开 brief 的发布、脱敏、过期与撤回。
- 与现有 `ChatShareModal` 不重复：现有实现是纯前端 PNG 导出；本提案新增服务端 share id、公开 payload、redaction、public landing 和增长归因。

## 文档同步说明

本轮只新增 proposal，不开始实现，因此不更新 `docs/current-plan.md`，也不归档计划页。若后续实际落地本提案，应新增或复用 `docs/current-plans/shareable-investment-briefs.md`，并在引入 share brief 存储、公开 API、redaction policy 或 public route 时同步更新 `docs/repo-map.md`、`docs/invariants.md`，必要时补充隐私与公开分享相关 decision。
