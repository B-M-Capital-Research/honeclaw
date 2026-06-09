# Proposal: Browser Research Clipper for Web Evidence Capture

status: proposed
priority: P2
created_at: 2026-06-09 08:04:32 +0800
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
- `docs/proposal/auto_p1_investment_document_inbox.md`
- `docs/proposal/auto_p2_desktop-quick-capture-inbox.md`
- `docs/proposal/auto_p1_source-provenance-freshness.md`
- `docs/proposal/auto_p0_untrusted-content-instruction-boundary.md`
- `docs/proposal/auto_p1_evidence_review_queue.md`
- `docs/proposal/auto_p1_research_artifact_library.md`
- `crates/hone-web-api/src/routes/public.rs`
- `crates/hone-web-api/src/routes/files.rs`
- `crates/hone-web-api/src/routes/company_profiles.rs`
- `crates/hone-channels/src/attachments/{ingest,vision,vector_store}.rs`
- `crates/hone-channels/src/sandbox.rs`
- `memory/src/company_profile/{storage,transfer,types}.rs`
- `packages/app/src/app.tsx`
- `packages/app/src/pages/chat.tsx`
- `packages/app/src/pages/public-portfolio.tsx`
- `packages/app/src/pages/public-me.tsx`
- `packages/app/src/lib/public-chat.ts`

## 背景与现状

Hone 当前已经有多种把投资材料带进系统的入口：

- Public Web `/chat` 支持附件上传，`crates/hone-web-api/src/routes/public.rs` 对 public upload 做路径约束，并把文件送进当前 Web 用户会话。
- Feishu / Telegram / Discord 等 IM 入口复用 `hone-channels::attachments` 的摄取层，已有大小限制、PDF preview、压缩包清单、图片 gate 和 actor sandbox 落盘规则。
- 公司画像已经是 actor-scoped 的长期研究记忆，适合承接长期 thesis、风险、证据和事件增量。
- `auto_p1_investment_document_inbox.md` 已提出把上传文件登记为长期文档对象；`auto_p2_desktop-quick-capture-inbox.md` 已提出桌面 tray / clipboard / file capture；`auto_p1_source-provenance-freshness.md` 与 `auto_p0_untrusted-content-instruction-boundary.md` 已分别提出来源新鲜度和外部内容无指令权边界。

但用户日常投资研究中，很多高价值证据不是本地文件，而是正在浏览器里看的网页：公司 IR 页面、SEC filing 网页、新闻文章、博客、卖方摘要、行业数据页、竞品公告、X/社交媒体线程、券商网页摘录、以及用户手动高亮的一段文字。当前用户若想让 Hone 记住它，通常只能复制文本到 chat、上传另存后的 PDF/截图，或用未来桌面 capture 保存 URL note。

这会丢失几个关键上下文：网页 URL、标题、选区、DOM 摘要、抓取时间、用户为什么保存、是否允许抓取正文、是否只保存引用而不保存全文、以及这段网页材料后续应进入文档箱、证据复盘、公司画像还是研究报告。Hone 缺少一个专门面向浏览器研究流的低摩擦、可审计、可控剪藏入口。

## 问题或机会

这是 P2：它能显著改善研究材料进入 Hone 的体验和增长路径，但应建立在文档收件箱、来源边界、权限确认和存储治理之上，不应抢在 P0/P1 的安全与核心运行链路前。

主要问题：

1. **网页证据进入系统的摩擦高。**  
   用户看到一段新闻或 filing 片段时，需要手工复制、切换页面、解释来源。这个过程容易丢 URL、时间、上下文和用户意图。

2. **URL note 不等于网页证据。**  
   桌面 quick capture 提案明确第一版只支持 URL + note，不抓网页正文。它适合桌面入口，但不解决浏览器中选区、页面 metadata、正文快照、robots/版权边界、登录页敏感内容和来源证明。

3. **网页内容不能被当成受信指令。**  
   网页正文、评论区、公告、论坛内容可能包含 prompt injection 或误导性文字。若直接复制进 chat，模型很难稳定区分“用户请求”与“网页内容”。浏览器剪藏应从入口处就打上 `untrusted_web_content` envelope。

4. **网页材料缺少长期对象身份。**  
   用户后续问“上次那篇关于 MU 库存周期的文章”时，系统没有稳定 `clip_id`、URL、hash、选区、来源时间和处理状态，难以复用或删除。

5. **公共 Web 激活缺少自然增长动作。**  
   投资用户愿意安装 PWA 或浏览器扩展，往往是因为它能在阅读新闻/公告时一键保存到自己的研究系统。Hone 目前 public Web 更像聊天和组合页，缺少“从外部网页回流到 Hone 工作台”的入口。

机会是新增 **Browser Research Clipper**：一个可选浏览器扩展 / PWA share target / bookmarklet 渐进方案，把网页 URL、选区、页面 metadata 和可选正文快照登记为 actor-scoped web evidence clip，再交给 Document Inbox、Evidence Review Queue、Company Portrait 或 Chat 草稿处理。

## 方案概述

新增 `WebEvidenceClip` 产品层。它不是新的研究真相源，也不直接修改公司画像；它只是把浏览器中的网页材料变成可治理、可追踪、可后续处理的证据入口。

核心能力：

1. **Clip capture**
   - 保存 URL、页面标题、canonical URL、选区文本、用户 note、页面语言、来源域名、capture time。
   - 可选保存正文快照，但必须由用户显式选择，并受大小、版权和敏感页面策略限制。

2. **Clip registry**
   - 每条 clip 有稳定 `clip_id`、actor、source browser、sha256/content hash、status、retention、processing state。
   - 不把浏览器本地绝对路径或 cookie/token 写进后端。

3. **Routing actions**
   - `Ask now`：打开 `/chat` 并把 clip 作为受控上下文草稿。
   - `Save to documents`：转成 Investment Document Inbox 对象。
   - `Queue evidence review`：进入 evidence review queue，等待判断是否影响 thesis。
   - `Update company portrait draft`：生成 agent-mediated handoff，不由 UI 直接改 `profile.md`。

4. **Provenance and trust envelope**
   - 每个 clip 默认标记为 `untrusted_web_content`。
   - URL、captured_at、selected_text hash、snapshot hash、fetch method、user note 分层保存。
   - 后续 agent 只能引用“用户保存的网页内容”，不能把网页中的指令当成系统或用户指令。

第一版建议从 PWA / bookmarklet / simple extension 中选最小入口：public `/me` 生成一个登录态 clip token 或 bookmarklet，用户点击后把当前 URL + selection + note 发回 Hone。完整 Chrome/Safari extension 可作为第二阶段。

## 用户体验变化

### 用户端

- Public `/me` 增加 `Research Clipper` 区块：
  - 显示是否启用、最近 clip、安装 bookmarklet / extension 的入口。
  - 解释默认只保存 URL、标题、选区和 note；保存正文快照需要确认。
- 用户在浏览器中选中一段文字，点击 `Save to Hone`：
  - 弹出轻量面板，展示标题、域名、选区、note 输入框。
  - 用户选择 `Ask now`、`Save evidence`、`Review later`、`Discard`。
  - 保存成功后得到短引用，例如 `clip:MU-20260609-01`。
- Public `/portfolio` 可以按 ticker 展示相关 clips：最近保存的文章、是否已复盘、是否已沉淀到画像。
- 删除 clip 时明确说明：clip 原文和快照会删除；如果后续已经写入公司画像或研究报告，派生成果不会被静默回滚，但来源会标记为 deleted。

### 管理端

- 在用户详情或未来 Documents 页面增加 `Web Clips` filter：
  - 按 actor、domain、detected ticker、status、created_at、processed/unprocessed 过滤。
  - 能看到 clip 是否只有 URL/selection，还是包含正文快照。
  - 能跳转原 session、document、evidence review item、company profile handoff。
- 管理员可以发起重新提取或转交 review，但不能越权读取用户未授权保存的页面正文。

### 桌面端

- Desktop bundled/remote 复用同一 public/admin Web API。
- 若 desktop 已处于 remote backend，clipper 面板明确说明内容会发送到远端 Hone 服务。
- 与 Desktop Quick Capture 的边界清晰：桌面 capture 处理剪贴板、截图、本地文件；browser clipper 处理网页 URL、选区和可选正文快照。

### 多渠道

- 多渠道不直接安装浏览器 clipper，但用户可在私聊中引用 clip id：
  - “总结我刚保存的 clip”
  - “把今天保存的 MU 文章列出来”
  - “把这条证据加入 MU 画像复盘”
- 群聊默认不能读取个人 clips；如果未来 Linked Workspace 落地，也必须显式授权。

## 技术方案

### 1. WebEvidenceClip 存储

建议在 `memory` 新增 `web_clip` 模块，第一版用 SQLite metadata + actor-scoped blob 目录：

```text
web_evidence_clips (
  clip_id TEXT PRIMARY KEY,
  actor_channel TEXT NOT NULL,
  actor_user_id TEXT NOT NULL,
  actor_scope TEXT,
  source_surface TEXT NOT NULL,
  source_url TEXT NOT NULL,
  canonical_url TEXT,
  page_title TEXT,
  source_domain TEXT,
  selection_text TEXT,
  user_note TEXT,
  snapshot_path TEXT,
  selection_sha256 TEXT,
  snapshot_sha256 TEXT,
  capture_method TEXT NOT NULL,
  trust_level TEXT NOT NULL,
  status TEXT NOT NULL,
  detected_symbols_json TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  deleted_at TEXT
)

web_clip_actions (
  action_id TEXT PRIMARY KEY,
  clip_id TEXT NOT NULL,
  action_kind TEXT NOT NULL,
  status TEXT NOT NULL,
  target_document_id TEXT,
  target_review_id TEXT,
  target_session_id TEXT,
  target_profile_id TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
)
```

Blob path:

```text
<actor_sandbox>/web_clips/<clip_id>/
  metadata.json
  snapshot.html.txt
  readable_text.txt
```

Do not store browser cookies, local storage, request headers, or original absolute local paths.

### 2. Capture endpoint

新增 public API：

- `POST /api/public/web-clips`
- `GET /api/public/web-clips`
- `GET /api/public/web-clips/:clip_id`
- `DELETE /api/public/web-clips/:clip_id`
- `POST /api/public/web-clips/:clip_id/actions`

Admin API：

- `GET /api/web-clips?actor=&domain=&symbol=&status=`
- `GET /api/web-clips/:clip_id`
- `POST /api/web-clips/:clip_id/reprocess`
- `DELETE /api/web-clips/:clip_id`

Public endpoint 从 `hone_web_session` 推导 `ActorIdentity::new("web", user.user_id, None)`，不接受任意 actor 参数。浏览器扩展若走 API key，则必须使用 scoped clip token，不复用长期 Hone Cloud API key。

### 3. Clipper 客户端形态

推荐分阶段：

- Phase 1: Bookmarklet / PWA share target
  - 用户从 `/me` 拖拽 `Save to Hone` bookmarklet。
  - Bookmarklet 读取 `location.href`、`document.title`、`window.getSelection().toString()`，打开 Hone clip compose URL。
  - 不直接跨站 POST，避免 CORS 和 cookie 复杂度；由 Hone 页面承接并让用户确认。
- Phase 2: Browser extension
  - Chrome/Safari extension popup 支持保存 URL、selection、note。
  - 使用短期 OAuth-like clip token 或 public session handoff，不把主 cookie 暴露给 content script。
- Phase 3: Optional readable snapshot
  - 对用户确认的页面，extension 提取 readability 文本或当前 DOM text snapshot。
  - 受大小上限、敏感域 denylist、robots/copyright policy 和 user confirmation 约束。

第一版不要抓取登录后券商页面正文；只允许用户选区文本 + note，并在 UI 中提示可能包含敏感信息。

### 4. 与现有架构衔接

- Document Inbox：`save_document` action 将 clip 变成文档对象，原网页快照作为 text/html 或 markdown-like source。
- Evidence Review Queue：`queue_review` action 创建候选证据项，包含 URL、选区、用户 note、detected symbols。
- Company Portrait：`profile_handoff` action 只生成 agent prompt，要求通过 `company_portrait` skill 读取 clip 摘要后判断是否更新画像。
- Source Provenance：clip 注册时写入 source provenance envelope；若服务端后续 fetch URL，需要记录 fetch time、HTTP status、content hash 和 freshness。
- Untrusted Content Boundary：所有 clip 正文在 prompt 中必须渲染为 `untrusted_web_content`，用户 note 单独作为 `user_note`，两者不能拼成同一个 instruction block。
- Artifact Access Boundary：如果 clip snapshot 后续以文件形式下载或被 chat 引用，应走 artifact/document 授权，不暴露 actor sandbox 路径。

### 5. 解析与分类

第一版只做低成本确定性分类：

- 从 title / URL / selected text 中提取 uppercase ticker-like tokens，并用 instrument registry 或 portfolio/watchlist 做二次过滤。
- 域名分类：`sec.gov`、公司 IR、新闻、blog、social、brokerage、unknown。
- 日期识别：保存 capture 日期和文本中显式日期，避免把网页发布时间和抓取时间混淆。
- 不用 LLM 自动写画像；最多生成 review suggestion。

后续可以在 evidence review 或 research artifact 阶段使用 LLM，但必须保留 clip 原始来源和不可信正文边界。

## 实施步骤

### Phase 1: Confirmed URL and selection capture

- 新增 `WebEvidenceClip` metadata 存储和 public create/list/delete API。
- Public `/me` 增加 bookmarklet / clip compose 入口。
- 支持保存 URL、title、selection、note，不保存正文快照。
- Public `/portfolio` 或 `/me` 展示最近 clips。

### Phase 2: Routing actions

- 增加 `Ask now`：打开 `/chat` 并预填 clip reference，不自动发送。
- 增加 `Save to documents`：如果 Document Inbox 已落地则创建 document；否则显示 pending integration。
- 增加 `Queue evidence review`：生成 review item 或占位状态。
- 增加 admin clips list/filter。

### Phase 3: Browser extension and snapshot

- 实现 Chrome/Safari extension 最小版 popup。
- 引入短期 clip token 和 token revoke。
- 支持用户确认后的 readable text snapshot，加入大小上限、denylist 和 source provenance。

### Phase 4: Agent handoff and metrics

- 让 agent 可通过受控 tool 查询当前 actor 的 clips 摘要。
- Company portrait / evidence review handoff 使用 clip id，而不是把网页全文塞进 prompt。
- 增加 privacy-preserving product events：clip created、action selected、deleted、reviewed，用于判断是否降低研究证据录入摩擦。

## 验证方式

- Rust unit tests：
  - clip create/list/delete 按 actor 隔离。
  - URL canonicalization、domain extraction、selection hash、deleted tombstone 行为稳定。
  - public endpoint 不能用 query 参数读取其它 actor 的 clips。
- Web API tests：
  - 未登录 public user 创建 clip 返回 401。
  - 空 URL、超长 selection、敏感 snapshot 请求返回稳定错误 code。
  - delete 后 snapshot blob 不可读取，metadata 仅保留 tombstone。
- Frontend tests：
  - bookmarklet compose 页面正确接收 URL/title/selection/note。
  - `Ask now` 只预填 chat 草稿，不自动发送。
  - remote/backend disconnected 时提示保存失败或仅保留本地草稿。
- 手工验收：
  - 在 Chrome/Safari 中选中一段公开新闻文字，保存到 Hone 后能在 `/me` 看到 clip。
  - 从 clip 进入 chat，模型回答中区分用户 note 和网页选区内容。
  - 删除 clip 后，列表隐藏正文，后续引用显示 source deleted。
- 指标：
  - 每周保存 clip 的活跃用户数。
  - clip 到 chat/document/review/profile handoff 的转化率。
  - 用户重复上传同一网页截图/文本的比例下降。

## 风险与取舍

- **风险：浏览器扩展扩大隐私面。**  
  取舍：先做 bookmarklet/compose 确认流；extension 延后，且不读取 cookie/localStorage。
- **风险：网页版权与全文存储边界复杂。**  
  取舍：第一版只存 URL、title、selection 和用户 note；正文快照必须显式确认，并受大小/域名策略限制。
- **风险：prompt injection 从网页进入 agent。**  
  取舍：所有网页正文默认 `untrusted_web_content`；用户 note 与网页内容分层；工具调用和画像更新必须走 handoff。
- **风险：与 Document Inbox / Desktop Capture 概念重叠。**  
  取舍：clipper 是浏览器入口；Document Inbox 是长期文档对象；Desktop Capture 是系统级剪贴板/截图/文件入口。
- **风险：未处理 clips 堆积。**  
  取舍：提供 review later、delete、retention 和按 ticker/domain 的轻量整理，不把每条 clip 自动升级为长期记忆。
- **不做：** 不静默抓取网页全文，不读取登录态页面敏感数据，不自动写公司画像，不把 URL 内容当作实时已核验事实，不替代 source provenance 或 document retention。

## 与已有提案的差异

查重范围：本轮已检查 `docs/proposal/` 与历史 `docs/proposals/`，并重点对比 document、desktop capture、provenance、untrusted content、research artifact、evidence review、PWA/browser 相关主题。

- 不重复 `auto_p1_investment_document_inbox.md`：该提案处理用户上传文件进入系统后的文档资产、解析和治理；本提案处理浏览器网页 URL/选区/note 的 capture 入口和 clip 对象。
- 不重复 `auto_p2_desktop-quick-capture-inbox.md`：该提案处理桌面 tray、clipboard、截图、本地文件和 remote upload 边界；本提案处理浏览器页面上下文、选区、bookmarklet/extension 和网页来源证明。
- 不重复 `auto_p1_source-provenance-freshness.md`：该提案是所有外部事实源的 provenance/freshness registry；本提案是一个新的用户主动网页证据入口，会向 provenance 层写入摘要。
- 不重复 `auto_p0_untrusted-content-instruction-boundary.md`：该提案定义外部内容没有指令权；本提案要求 browser clips 从创建时就遵守该边界。
- 不重复 `auto_p1_research_artifact_library.md` 或 `auto_p1_evidence_review_queue.md`：它们处理研究交付物和证据复盘；本提案只负责把网页材料低摩擦、可治理地带入这些后续流程。

差异结论：当前仓库已有上传文档、桌面捕获和来源治理方向，但没有覆盖“浏览器中正在阅读的网页证据如何以 URL/选区/note/snapshot 的形式进入 Hone，并被安全地路由到 chat、文档、证据复盘或公司画像 handoff”。本提案填补的是 public Web 增长入口与研究证据第一公里。

## 文档同步说明

本轮只新增 proposal，不开始执行实现，因此不更新 `docs/current-plan.md`、`docs/repo-map.md` 或 `docs/invariants.md`。若后续开始实现本提案，应按影响范围新增动态计划，并在引入新 API、存储或浏览器扩展目录时同步更新 repo map 与相关 runbook。
