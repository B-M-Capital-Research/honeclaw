# Proposal: Investment Document Inbox for User-Supplied Evidence

status: proposed
priority: P1
created_at: 2026-05-06 05:03:24 CST
owner: automation

## related_files

- `README.md`
- `AGENTS.md`
- `docs/repo-map.md`
- `docs/invariants.md`
- `docs/decisions.md`
- `docs/current-plan.md`
- `docs/proposal/auto_p0_investment_output_safety_gate.md`
- `docs/proposal/auto_p1_delivery_decision_loop.md`
- `docs/proposal/auto_p1_evidence_review_queue.md`
- `docs/proposal/auto_p1_investment_context_intake.md`
- `docs/proposal/auto_p1_linked-user-workspace.md`
- `docs/proposal/auto_p1_research_artifact_library.md`
- `docs/proposal/auto_p1_run_trace_workbench.md`
- `docs/proposal/auto_p1_usage_entitlement_ledger.md`
- `docs/proposals/desktop-bundled-runtime-startup-ux.md`
- `docs/proposals/skill-runtime-multi-agent-alignment.md`
- `crates/hone-channels/src/attachments.rs`
- `crates/hone-channels/src/attachments/ingest.rs`
- `crates/hone-channels/src/attachments/vector_store.rs`
- `crates/hone-channels/src/attachments/vision.rs`
- `crates/hone-channels/src/sandbox.rs`
- `crates/hone-web-api/src/routes/public.rs`
- `crates/hone-web-api/src/routes/chat.rs`
- `crates/hone-web-api/src/types.rs`
- `packages/app/src/pages/chat.tsx`
- `packages/app/src/lib/api.ts`
- `bins/hone-feishu/src/handler.rs`
- `bins/hone-telegram/src/handler.rs`
- `bins/hone-discord/src/{attachments.rs,handlers.rs}`
- `memory/src/company_profile/{storage,transfer,types}.rs`
- `skills/company_portrait/SKILL.md`

## 背景与现状

Hone 已经支持用户从多个入口把文件带进对话：

- Public Web chat 有上传入口，`packages/app/src/pages/chat.tsx` 允许最多 4 个待发送附件，`packages/app/src/lib/api.ts` 调用 `/api/public/upload`，再把上传结果随 `/api/public/chat` 发送。
- `crates/hone-web-api/src/routes/public.rs` 会把 public 上传文件写到当前 Web 用户的 `public-uploads/<date>/` 目录，并用 `validate_public_upload_path` 限制 chat 只能引用该用户上传根目录下的路径，避免任意本地文件引用。
- Feishu / Telegram / Discord 已经走共享附件摄取层：各 channel 收集 `RawAttachment`，调用 `hone-channels::attachments::ingest_raw_attachments`，再用 `build_user_input` 把附件清单、PDF 文本片段、压缩包解压清单和默认处理策略写入本轮 prompt。
- `crates/hone-channels/src/attachments/ingest.rs` 已有附件分类、文件名清洗、5MB 通用附件上限、3MB 图片上限、压缩包展开限制、PDF 文本预览、附件 manifest 持久化和本轮 placeholder ack。
- `crates/hone-channels/src/attachments/vision.rs` 已拦截超大图片、极端长宽比和异常像素量，说明附件安全边界已被当作运行时契约处理。
- `crates/hone-channels/src/attachments/vector_store.rs` 已提供 PDF 全量文本提取 helper，虽然当前 prompt 路径主要使用 preview。
- `crates/hone-channels/src/sandbox.rs` 把 channel 附件放在 actor sandbox 下的 `uploads/<session_id>/`，符合 actor 隔离约束。
- 公司画像已经是 actor sandbox 内的长期投资记忆，`skills/company_portrait/SKILL.md` 要求把主线、证据、风险、证伪条件和事件增量写入 `company_profiles/<profile_id>/profile.md` 与 `events/*.md`。

这些实现说明 Hone 已有“接收文件并让模型本轮使用”的基础，但还没有“用户供给的投资文档”产品层。现在附件更多像 chat message 的临时上下文：能被当前 turn 看见、能落一个 manifest、能在历史里以路径标记回显，但它不是可搜索、可复盘、可引用、可授权删除、可送入公司画像或研究报告流程的长期对象。

对投资研究助手来说，用户上传的文件往往不是普通附件，而是高价值证据：券商持仓截图、交易记录、财报 PDF、会议纪要、研报、公告、Excel 估值表、朋友转发的观点截图、公司电话会 transcript、行业数据压缩包。这些材料如果只被一次对话消耗掉，Hone 很难兑现“长期投资研究资产”的定位。

## 问题或机会

当前缺口集中在六类链路。

1. 附件没有跨会话资产身份。
   Channel 附件按 `uploads/<session_id>/` 存放，public Web 上传按日期存放；系统没有稳定的 `DocumentId`、来源、状态、解析结果、关联 ticker、保留策略和删除入口。用户后续问“上次我发的那份 MU 研报”时，agent 很难可靠定位。

2. Web 与 IM 附件处理语义不一致。
   IM channel 通过 `ingest_raw_attachments` 得到 PDF preview、压缩包清单和处理策略；public Web 目前只是把 `[附件: /path]` 拼进消息，由运行时和模型自行处理。这会导致同一份 PDF 或截图在 Web 与 IM 中获得不同的解析质量和安全反馈。

3. 用户供给证据没有进入长期研究循环。
   现有 evidence review queue 关注 event-engine / digest 产生的外部事件；research artifact library 关注深度研究报告交付物；investment context intake 关注持仓、偏好和画像缺口。用户主动上传的文件处在这些流程之外，不能自然形成“这份文件是否应更新画像 / 触发复盘 / 创建研究任务”的待办。

4. 管理端缺少附件可治理视图。
   运维能看日志、会话、LLM audit、任务健康和公司画像，但无法按 actor 查看最近上传文件、解析失败、扫描件 OCR 缺口、超限拦截、潜在敏感材料、存储占用或保留期限。

5. 隐私与成本边界还停留在单次上传。
   上传限制能防止单次滥用，但没有长期 retention、删除请求、导出包、hash 去重、解析成本、OCR/embedding 是否启用、哪些文档会进入模型上下文等可解释边界。Public 服务已写明隐私条款会收集上传附件，因此产品上应给用户一个可管理入口。

6. 多渠道体验无法把“发材料给 Hone”变成增长动作。
   用户在 Feishu / Telegram / Discord 发送研报或截图时，Hone 可以当场回答，但没有一个 Web/desktop 页面承接“材料已归档、可稍后继续研究、可沉淀到公司画像”。这削弱了从 IM 轻量触达到 Web/desktop 深度工作台的转化。

这是 P1，因为它能显著提升核心体验、长期记忆质量、用户信任、运维效率和未来商业化承接，但不要求推翻现有 runner、company portrait、research artifact 或 event-engine 架构。第一版可以把已有附件摄取能力实体化为只读/半自动文档收件箱，再逐步接入 OCR、向量索引和画像交接。

## 方案概述

新增 actor-scoped 的 `InvestmentDocumentInbox`：把用户上传到 Web / IM / desktop 的投资相关文件统一登记为可治理的文档对象，并围绕它建立解析、分类、复盘和长期记忆交接流程。

核心对象建议：

- `InvestmentDocument`：稳定文档元数据，包含 `document_id`、actor、source channel、session_id、message_id、filename、kind、mime、size、sha256、storage_path、created_at、status、retention_policy。
- `DocumentExtraction`：解析结果，包含 PDF 文本片段 / 全文路径、图片 OCR 状态、表格 schema preview、压缩包文件清单、extract error、language、page_count、detected_symbols、detected_dates。
- `DocumentRoutingHint`：将文档归类为 `broker_statement`、`holding_screenshot`、`earnings_report`、`research_report`、`filing`、`valuation_sheet`、`meeting_notes`、`misc_evidence`，并给出候选 ticker / company / action。
- `DocumentReviewItem`：需要用户或 agent 处理的队列项，例如“提取持仓草稿”“更新公司画像”“创建研究报告”“标记为噪音”“删除敏感文件”。
- `DocumentAccessPolicy`：控制 public 用户、admin、当前 actor、future workspace 是否能读取原文、摘要、导出包或删除。

第一版目标不是搭完整 DMS，也不是把所有上传内容自动写入长期记忆。建议先做三个清晰能力：

1. 统一登记：Web 和 IM 入口都把已接受附件写成 `InvestmentDocument`。
2. 统一解析摘要：复用现有 PDF/Archive/Image gate，并把解析状态保存在服务端，而不是只拼进本轮 prompt。
3. 统一交接动作：从文档详情生成受控 agent prompt，让现有 `portfolio_tool`、`company_portrait`、research artifact 或 evidence queue 处理后续，不由文档 API 直接改投资真相源。

## 用户体验变化

### 用户端

- Public `/me` 或 `/portfolio` 增加“我的材料”入口，展示最近上传的投资文档、解析状态、关联标的和待处理建议。
- Public `/chat` 上传完成后，不只在 composer 里显示文件名，还提示“已归入材料收件箱”，并在回复里可引用短文档编号，例如 `doc:MU-20260506-01`。
- 文档详情展示：
  - 文件名、类型、大小、上传来源、所属会话。
  - PDF 文本是否提取成功；如果是扫描件，提示需要 OCR 或重新上传可复制文本。
  - 识别到的 ticker、日期、报表类型或持仓字段。
  - 可执行动作：继续追问、提取持仓草稿、让 Hone 更新公司画像、加入证据复盘队列、删除文件。
- 用户可以删除自己上传的文档。删除应明确说明：原文件和解析缓存会删除；已经被 agent 写入公司画像或研究报告的派生成果不会被静默回滚，但会保留来源已删除的标记。

### 管理端

- 新增 `/documents` 或在 `/users/:actor` 下增加 `Documents` tab。
- 管理员可按 actor、channel、kind、status、detected symbol、created_at、size、error 类型过滤。
- 列表中突出高价值和高风险状态：
  - PDF 提取失败 / 扫描件。
  - 压缩包部分解压失败。
  - 表格疑似持仓但未确认。
  - 文件过大被拒绝。
  - 文件长期未处理但被多次在 chat 中引用。
- 详情页可跳转到原 session、附件 manifest、company profile、research artifact 或 future trace id。
- 管理员可以为用户创建 document review task，但不能直接越权把用户文件写入其它 actor 的画像。

### 桌面端

- Desktop bundled/remote 模式复用同一 Web console 文档页。
- Dashboard 增加轻量提醒：最近上传材料、解析失败数、待处理证据数、存储占用。
- 本地 desktop 用户可选择更长 retention；remote/public 用户默认更保守，避免服务端长期堆积敏感文件。

### 多渠道

- Feishu / Telegram / Discord 收到附件后，placeholder ack 保留当前“已收到并解析”反馈，同时增加短文档编号。
- 用户在 IM 中可以说“把刚才那份 PDF 写进 MU 画像”或“列出我本周上传的材料”，agent 通过 document id / recent document tool 查找受控摘要。
- 群聊附件默认归属于 group `SessionIdentity` 对应的 actor/scope，不自动写入个人 workspace；若要进入个人投资文档箱，应要求私聊确认或显式 `/save-to-me`。

## 技术方案

### 1. 新增文档元数据存储

建议在 `memory` 增加 `investment_document` 模块，优先使用 SQLite 存 metadata，文件仍保存在 actor sandbox 或 sessions dir 下。第一版不迁移旧文件，只对新上传生成记录。

建议表：

```text
investment_documents (
  document_id TEXT PRIMARY KEY,
  actor_channel TEXT NOT NULL,
  actor_user_id TEXT NOT NULL,
  actor_scope TEXT,
  source_channel TEXT NOT NULL,
  source_session_id TEXT,
  source_message_id TEXT,
  filename TEXT NOT NULL,
  kind TEXT NOT NULL,
  mime TEXT,
  size_bytes INTEGER NOT NULL,
  sha256 TEXT NOT NULL,
  storage_path TEXT NOT NULL,
  status TEXT NOT NULL,
  retention_policy TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  deleted_at TEXT
)

document_extractions (
  document_id TEXT PRIMARY KEY,
  extraction_status TEXT NOT NULL,
  text_preview TEXT,
  full_text_path TEXT,
  archive_manifest_json TEXT,
  image_ocr_status TEXT,
  detected_symbols_json TEXT,
  detected_dates_json TEXT,
  routing_kind TEXT,
  error_message TEXT,
  updated_at TEXT NOT NULL
)

document_review_items (
  review_id TEXT PRIMARY KEY,
  document_id TEXT NOT NULL,
  actor_channel TEXT NOT NULL,
  actor_user_id TEXT NOT NULL,
  actor_scope TEXT,
  target_symbol TEXT,
  action_kind TEXT NOT NULL,
  status TEXT NOT NULL,
  agent_session_id TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
)
```

文件路径建议继续遵守 actor sandbox 约束：

```text
<actor_sandbox>/documents/<document_id>/
  original/<safe_filename>
  extraction/text.txt
  extraction/archive_manifest.json
  metadata.json
```

Public Web 现有 `public-uploads/<date>/` 可以在登记时复制或移动到 actor document path。为兼容，第一版也可以保留原文件位置，并在 metadata 记录 canonical path；后续再做迁移工具。

### 2. 统一附件摄取入口

现有 IM 入口已经使用 `ingest_raw_attachments`。建议把它拆出一个更上层的“接收并登记”流程：

- `ingest_raw_attachments` 继续负责准入、落盘、PDF preview、archive preview 和 manifest。
- 新增 `register_ingested_documents(actor, session_id, source, attachments)`，将每个 accepted attachment 写入 `investment_documents` 和 `document_extractions`。
- Public Web `/api/public/upload` 或 `/api/public/chat` 不再只拼 `[附件: path]`；应复用同一登记流程，并把 document id 回传给前端。
- `build_user_input` 可以从 `document_id` 读取 extraction summary，避免 Web/IM 两套 prompt 语义漂移。

兼容策略：

- 旧 chat request 中的 `attachments: [{ path }]` 仍可工作；如果 path 没有 document record，则按旧方式引用，同时 opportunistically 登记。
- 旧 session 历史里的 `[附件: path]` 不强制迁移；文档页只显示新登记的对象。

### 3. 文档路由与 review item

第一版不要依赖昂贵 LLM 分类。可以先用确定性和轻量规则：

- 文件名 / MIME / ext：PDF、CSV、XLSX、image、archive。
- 文本 preview 中识别 ticker、财报关键词、10-K/10-Q、earnings、transcript、portfolio、holding、position、cost basis。
- 表格文件先只做 header preview，不解析复杂公式。
- 图片先只保留元数据和可选 OCR 状态；没有 OCR 时明确显示 `ocr_unavailable`。

当 routing hint 命中高价值类别时创建 `DocumentReviewItem`：

- `broker_statement` / `holding_screenshot` -> 提取持仓草稿，进入 investment context intake 的 preview/apply。
- `earnings_report` / `filing` / `meeting_notes` -> 加入 evidence review queue，候选 action 为“是否更新画像”。
- `research_report` -> 可创建 research artifact handoff 或画像更新草稿。
- `valuation_sheet` -> 可生成继续追问或估值复核 prompt，但默认不写画像。

### 4. API 与前端

建议新增 API：

- `GET /api/documents?actor=&kind=&status=&symbol=&from=&to=`
- `GET /api/documents/:document_id`
- `GET /api/documents/:document_id/file`
- `GET /api/documents/:document_id/extraction`
- `POST /api/documents/:document_id/reprocess`
- `POST /api/documents/:document_id/review-items`
- `DELETE /api/documents/:document_id`
- `GET /api/public/documents`
- `GET /api/public/documents/:document_id`
- `DELETE /api/public/documents/:document_id`

前端建议新增：

- `packages/app/src/lib/documents.ts`
- `packages/app/src/context/documents.tsx`
- `packages/app/src/pages/documents.tsx`
- Public `/me` 或 `/portfolio` 的 documents section
- Admin `/users/:actor/documents` tab

权限原则：

- Public 用户只能访问当前 `hone_web_session` actor 的文档。
- Admin 可读取 metadata 和 extraction summary；读取原文需要本地 admin 权限，并应在 UI 上明确。
- 所有本地绝对路径通过 `/api/file` / `/api/public/file` 受控读取，不把路径直接暴露成用户可复制的长期引用。

### 5. 与现有长期资产的关系

本提案不替代已有资产层：

- `InvestmentDocument` 是用户供给证据的收件箱，不是公司画像。
- 公司画像仍以 `profile.md` 和 `events/*.md` 为真相源，文档只能通过 agent-mediated prompt 触发更新。
- Evidence review queue 处理“是否需要复盘证据”；document inbox 负责“证据从用户上传进入系统、可被后续引用和治理”。
- Research artifact library 处理“深度研究报告交付物”；document inbox 可以成为报告输入或外部研报归档，但不负责报告生成状态机。
- Usage entitlement ledger 可后续把 OCR、PDF 全文提取、长期存储和文档数量纳入权益，但第一版只记录 usage event，不阻塞基础功能。

## 实施步骤

### Phase 1: 文档登记与只读收件箱

- 在 `memory` 增加 `investment_document` 存储、类型和单元测试。
- 在 IM attachment persist pipeline 后登记 accepted attachments。
- 在 public upload/chat 路径登记文档，并返回 `document_id`。
- 新增 admin/public documents list/detail API。
- 前端先做只读列表、详情、解析状态和原 session 跳转。
- 不引入 OCR，不迁移历史附件。

### Phase 2: 统一解析摘要与 prompt 引用

- 让 Web 和 IM 都通过同一 extraction summary 组装 prompt。
- PDF preview/full text 提取结果写入 `document_extractions`，并记录失败原因。
- Archive manifest 结构化保存，避免只写入本轮 prompt。
- 增加 document id 引用语法，例如 `doc:<short_id>`，由 tool/API 解析为当前 actor 可访问摘要。
- 增加删除接口，删除原文和 extraction cache，并在 review item 中标记 source deleted。

### Phase 3: 路由到投资工作流

- 实现轻量 routing hints 和 `DocumentReviewItem`。
- 与 investment context intake 对接：持仓截图 / 表格生成 intake draft，而不是直接写 portfolio。
- 与 evidence review queue 对接：财报、公告、会议纪要生成待复盘项。
- 与 company portrait skill 对接：生成受控 prompt，让 agent 写入画像或事件。
- 与 research artifact library 对接：外部研报可转成 report handoff 或继续追问入口。

### Phase 4: OCR、去重和权益

- 为扫描 PDF / 图片引入可选 OCR，默认关闭或受 entitlement 控制。
- 用 sha256 去重同一 actor 重复上传的文件。
- 增加 retention 策略：public 默认 30/90 天，本地 desktop 可设为长期。
- usage entitlement ledger 记录文档数、解析次数、OCR 次数和存储占用。

## 验证方式

- Rust 单元测试：
  - 文档 id 生成、actor 归属、sha256、soft delete、retention policy 序列化。
  - public upload path 只能登记当前用户上传根目录内的文件。
  - IM `ReceivedAttachment` 到 `InvestmentDocument` 的转换保留 kind、size、filename、extract error。
  - PDF extraction success / failure 都能写入 `document_extractions`，API 不因解析失败返回 500。
  - 删除文档后原文和 extraction cache 被移除，review item 标记为 source deleted。
- Web API 测试：
  - Public 用户只能列出和读取自己的文档。
  - Admin actor 可按 actor 查询文档。
  - 不存在、已删除、跨 actor 文档返回明确 404/403。
- 前端测试：
  - 上传后 composer 显示 document id / 解析状态。
  - 文档列表过滤、详情空态、解析失败态、删除确认态可用。
  - 从文档详情生成 review action 时，不直接编辑公司画像。
- 手工验收：
  - Web 上传 PDF、截图、CSV、超限文件，确认 Web 与 IM 的反馈语义一致。
  - Feishu / Telegram / Discord 上传同一 PDF，确认文档页出现记录并能跳回会话。
  - 上传扫描 PDF 时，用户看到明确 OCR 缺口，而不是模型假装读到了全文。
  - 删除文档后，后续 `doc:<id>` 引用被拒绝并提示文件已删除。
- 指标：
  - 上传接受率、解析成功率、扫描件占比、待处理 review item 数、文档到画像更新转化率、存储占用、删除请求数量。

## 风险与取舍

- 隐私风险：用户上传材料可能包含账户、持仓、身份证明或券商信息。第一版必须默认 actor-scoped、默认不跨 workspace 共享、删除可用、原文读取有权限边界。
- 成本风险：OCR、embedding、PDF 全文解析和长期存储会增加成本。第一版不默认启用 OCR / embedding，只保存轻量 extraction summary。
- 产品复杂度：如果文档页变成完整知识库，会与已下线的 KB 入口冲突。边界必须明确：这里只处理用户供给的投资证据和材料治理，不恢复泛知识库或 `kb_search`。
- 模型误读风险：路由 hint 只是建议，不能自动把上传材料写成持仓、画像或投资结论；所有写入 portfolio / company profile 的动作都应经过 preview 或 agent-mediated 更新。
- 兼容风险：历史附件没有 document record。第一版不迁移历史，只对新上传生效；旧会话路径仍按现有历史附件逻辑展示。
- 路径泄漏风险：当前部分 prompt 会包含本地路径。文档 API 和 UI 应使用 document id，用户可见输出应尽量避免暴露 sandbox 外绝对路径。

## 与已有提案的差异

本轮查重范围包括 `docs/proposal/` 和 `docs/proposals/` 下所有现有提案：

- `auto_p0_investment_output_safety_gate.md` 关注投资敏感输出送达前的安全 verdict；本提案关注用户上传材料进入系统后的资产化、解析和治理。
- `auto_p1_delivery_decision_loop.md` 关注通知为什么发送 / 不发送；本提案关注附件文档如何被保留、复用和转入研究流程。
- `auto_p1_evidence_review_queue.md` 关注 event-engine / digest 产生的外部证据待复盘；本提案补齐用户主动上传证据的入口和文档身份。
- `auto_p1_investment_context_intake.md` 关注持仓、偏好、画像缺口初始化；本提案可以把券商截图 / 表格转成 intake draft，但不替代 intake。
- `auto_p1_linked-user-workspace.md` 关注跨渠道真实用户工作区；本提案仍按 actor 隔离落地，未来可被 workspace 聚合。
- `auto_p1_research_artifact_library.md` 关注深度研究报告交付物；本提案关注上传的原始材料，可作为报告输入或外部研报归档，但不是报告状态机。
- `auto_p1_run_trace_workbench.md` 关注 agent run 可观测性；本提案关注文档资产和用户材料治理。
- `auto_p1_usage_entitlement_ledger.md` 关注额度、权益和成本；本提案后续可把文档解析/OCR纳入 usage event，但第一版不做计费。
- `desktop-bundled-runtime-startup-ux.md` 关注桌面 bundled runtime 启动恢复；本提案只复用桌面 Web console 展示文档收件箱。
- `skill-runtime-multi-agent-alignment.md` 关注 skill runtime 与 multi-agent 语义；本提案不改变 skill 调用模型，只通过受控 prompt 使用现有 skill。

因此本提案不是重复已有“证据复盘”“投研上下文初始化”或“研究报告库”，而是补上这些能力共同依赖的用户供给文档入口。
