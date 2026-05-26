# Proposal: Voice Memo Research Capture for Mobile Investment Workflows

status: proposed
priority: P2
created_at: 2026-05-26 14:07:42 +0800
owner: automation

## related_files

- `README.md`
- `AGENTS.md`
- `docs/repo-map.md`
- `docs/invariants.md`
- `docs/decisions.md`
- `docs/current-plan.md`
- `docs/proposal/auto_p1_investment_document_inbox.md`
- `docs/proposal/auto_p1_investment_context_intake.md`
- `docs/proposal/auto_p1_investment-thread-workbench.md`
- `docs/proposal/auto_p1_research_artifact_library.md`
- `docs/proposal/auto_p1_evidence_review_queue.md`
- `docs/proposal/auto_p1_usage_entitlement_ledger.md`
- `crates/hone-channels/src/attachments/ingest.rs`
- `crates/hone-channels/src/attachments.rs`
- `bins/hone-telegram/src/handler.rs`
- `crates/hone-web-api/src/routes/public.rs`
- `packages/app/src/pages/chat.tsx`
- `packages/app/src/pages/public-me.tsx`
- `packages/app/src/pages/public-portfolio.tsx`
- `memory/src/session.rs`
- `memory/src/company_profile/{storage,types}.rs`
- `skills/company_portrait/SKILL.md`

verification: see `## 验证方式`
risks: see `## 风险与取舍`

## 背景与现状

Hone 已经把投资助手放进用户日常入口：Public Web、Mac desktop、Telegram、Discord、Feishu、iMessage、scheduled task、event-engine digest 和 company portraits。移动端即时通讯尤其适合碎片化输入：用户在通勤、看财报电话会、读新闻或和朋友讨论时，往往更自然地发一段语音，而不是整理成完整文字。

当前代码已经部分承认这个入口存在：

- `bins/hone-telegram/src/handler.rs` 会把 Telegram `audio()` 和 `voice()` 消息视为 supported attachments，并下载成 `RawAttachment`。
- `crates/hone-channels/src/attachments/ingest.rs` 的 `AttachmentKind` 已包含 `Audio` 和 `Video`，`infer_attachment_kind` 识别 `audio/*`、`mp3`、`wav`、`m4a`、`aac`、`ogg`、`flac` 等格式。
- 同一文件的默认附件策略明确写着：音视频在“当前无稳定转写工具”时，只能先告知限制并要求用户补充文字稿。
- Public Web `/api/public/upload` 只做附件数量和大小限制，上传后把本地路径拼进消息；它还没有音频专属处理、转写状态、隐私提示或后续研究入口。
- 公司画像、投资主线、research artifact、evidence queue 和 investment thread 已经在提案与实现中逐步成为长期研究资产，但它们主要假设输入已经是文本、文档、事件或报告。

这说明 Hone 已经能接住音频文件，却不能把它变成可靠投资研究材料。语音当前是“可上传但不可理解”的附件类型，体验上会让用户以为 Hone 支持语音，实际却回到“请补充文字稿”。

## 问题或机会

1. **移动端高频输入没有被产品化。**  
   投资用户经常在非桌面场景产生想法：某个标的为什么需要复盘、某条新闻是否影响 thesis、朋友观点哪里有漏洞、财报电话会听到的疑点、想让 Hone 明天提醒的问题。要求这些内容必须先打字，会降低捕获率。

2. **音频附件缺少受控转写与确认。**  
   直接把 STT 文本塞进 prompt 会引入误听、噪声、标点错误、ticker 混淆和敏感信息泄露。投资场景不能让错误转写直接变成画像更新、持仓判断或通知偏好。

3. **语音备忘录和普通文档不是同一个产品对象。**  
   `Investment Document Inbox` 适合管理 PDF、截图、表格、研报和压缩包。语音更像“用户当下意图 + 未整理研究线索”：它需要转写、说话人确认、ticker disambiguation、行动项抽取和轻量追问，而不是只归档原文件。

4. **多渠道缺少统一能力边界。**  
   Telegram 已下载 voice/audio；Discord、Feishu 和 Web 也可能通过附件上传带来音频。没有统一的 `VoiceMemo` 对象时，各渠道只能各自提示限制，后续很难做到统一 quota、retention、删除、复盘和 UI 展示。

5. **商业化和留存机会没有承接。**  
   语音捕获是个人助理类 AI 产品的重要体验：用户越容易把碎片想法交给 Hone，Hone 越能成为长期投资工作台。第一版不需要实时语音对话，只需可靠处理短语音备忘录，就能显著降低输入摩擦。

本提案列为 P2：它不是核心可用性 blocker，也不应抢在权限、额度、输出安全和数据治理之前；但它有明确用户体验收益，能强化“多渠道个人投资助理”的产品差异，并且可以在现有附件、session、company portrait 和 public/admin UI 上增量实现。

## 方案概述

新增 **Voice Memo Research Capture**：一个 actor-scoped 的语音备忘录入口，把短音频从“附件限制提示”升级为“可转写、可确认、可转研究行动”的产品对象。

核心对象：

- `VoiceMemo`：原始音频元数据，包含 actor、source channel、session_id、filename、duration、mime、size、sha256、storage_path、created_at、retention_policy。
- `VoiceTranscript`：转写结果，包含 transcript text、language、confidence、segment timestamps、detected tickers、uncertain spans、provider metadata、redaction status。
- `VoiceCaptureDraft`：从转写中提取的用户意图草稿，例如 `question`、`profile_update_candidate`、`reminder_candidate`、`evidence_note`、`research_thread_note`。
- `VoiceConfirmation`：用户确认层，记录哪些 draft 被接受、修正、丢弃或转给 agent。
- `VoicePrivacyPolicy`：音频保存、转写文本保存、删除、是否允许外部 STT provider、是否允许进入长期记忆的策略。

第一版目标应保持窄：

1. 支持短音频备忘录异步转写，不做实时语音对话。
2. 转写结果默认只进入草稿/预览，不自动改写公司画像、portfolio、notification prefs 或 scheduled task。
3. Web/IM 都返回同一种状态：received -> transcribing -> ready_for_review -> confirmed / discarded / failed。
4. 后续行动复用既有 agent-mediated 路径：company portrait 更新继续由 `company_portrait` skill 执行，提醒/任务继续走 scheduled task，研究议题可进入 investment thread。

## 用户体验变化

### 用户端

- 在 Telegram 或 Web 发一段短语音后，Hone 先回复“已收到语音，正在转写”，而不是要求用户手工补文字稿。
- 转写完成后，用户看到简短确认卡：
  - 识别出的文字摘要。
  - 系统认为的 ticker / company / 时间 / 行动项。
  - 不确定片段，例如“TSLA 还是 TLSA？”。
  - 可选动作：`作为问题继续问`、`加入研究议题`、`更新公司画像草稿`、`创建提醒草稿`、`丢弃`。
- Public `/me` 或 `/portfolio` 增加“语音备忘录”入口，只展示当前用户自己的待确认和已处理 memo。
- 用户可以删除原始音频；删除后保留的 transcript / 派生成果必须明确标注来源音频已删除。
- 对转写低置信度的 memo，Hone 不直接回答投资问题，而是先要求用户确认关键 ticker 和结论。

### 管理端

- 用户详情页增加 Voice Memos tab 或并入未来 Documents/Threads 工作台，支持按 actor、channel、status、detected ticker、created_at、failure reason 过滤。
- 管理员能看到转写失败原因：文件超限、格式不支持、STT 未配置、provider 超时、语言不支持、低置信度。
- 管理端可以帮助用户把一条确认后的 memo 转成研究线程或画像更新请求，但不能直接把未确认转写写入长期资产。
- Settings 显示语音能力状态：STT provider 是否配置、最大时长/大小、保留策略、外部转写隐私提示。

### 桌面端

- Desktop bundled 模式可从文件拖入短音频，复用同一上传和转写状态；如果本地未配置 STT，展示明确的 missing capability。
- Remote backend 模式只展示服务端 capability，不假设本机有音频处理能力。
- 桌面 Dashboard 可显示待确认语音备忘录数量，作为“稍后整理研究想法”的轻量 inbox。

### 多渠道

- Telegram voice/audio 先作为第一优先入口，因为代码已经下载这两类附件。
- Feishu / Discord 若后续能拿到音频附件，也走同一 `VoiceMemo` 登记和转写流程。
- 群聊语音默认只归属于 group scope，不自动写入个人投资记忆；若用户要保存到个人空间，需要私聊确认或显式命令。
- IM 不承载长篇编辑器，只返回短摘要和确认命令；复杂 review 在 Web/desktop 完成。

## 技术方案

### 1. 存储与状态机

建议在 `memory` 新增 `voice_memo` 模块，使用 SQLite 存 metadata，原始音频放在 actor sandbox：

```text
voice_memos (
  memo_id TEXT PRIMARY KEY,
  actor_channel TEXT NOT NULL,
  actor_user_id TEXT NOT NULL,
  actor_scope TEXT,
  source_channel TEXT NOT NULL,
  source_session_id TEXT,
  source_message_id TEXT,
  filename TEXT NOT NULL,
  mime TEXT,
  size_bytes INTEGER NOT NULL,
  duration_ms INTEGER,
  sha256 TEXT NOT NULL,
  storage_path TEXT NOT NULL,
  status TEXT NOT NULL,
  retention_policy TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  deleted_at TEXT
)

voice_transcripts (
  memo_id TEXT PRIMARY KEY,
  transcript_status TEXT NOT NULL,
  language TEXT,
  transcript_text TEXT,
  confidence REAL,
  segments_json TEXT,
  detected_symbols_json TEXT,
  uncertain_spans_json TEXT,
  provider TEXT,
  provider_metadata_json TEXT,
  redaction_report_json TEXT,
  error_message TEXT,
  updated_at TEXT NOT NULL
)

voice_capture_drafts (
  draft_id TEXT PRIMARY KEY,
  memo_id TEXT NOT NULL,
  action_kind TEXT NOT NULL,
  target_symbol TEXT,
  draft_text TEXT NOT NULL,
  status TEXT NOT NULL,
  agent_session_id TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
)
```

状态建议：

- `received`
- `rejected_policy`
- `transcribing`
- `transcript_ready`
- `needs_confirmation`
- `confirmed`
- `discarded`
- `failed`
- `deleted`

### 2. 附件摄取改造

在 `crates/hone-channels/src/attachments/ingest.rs` 保留现有准入限制，但对 `AttachmentKind::Audio` 增加专门路径：

- policy 增加音频最大大小和最大时长。第一版建议仍沿用 5MB 通用上限，再额外限制时长，例如 180 秒。
- accepted audio 生成 `VoiceMemo`，而不是只作为普通附件 note。
- `build_attachment_strategy_note_from_refs` 的音视频提示改为根据 capability 分支：
  - STT 未配置：保留当前“请补文字稿”的限制提示。
  - STT 已配置且 memo 正在转写：提示用户稍后确认。
  - transcript 已确认：把确认后的 transcript summary 注入本轮 prompt。
- 对 Web public upload，`classify_attachment_kind` 与 `/api/public/chat` 应能返回/引用 `memo_id`，避免只传本地 path。

### 3. STT Provider 抽象

新增轻量 trait，不把具体 provider 绑死在 channel 里：

```rust
pub trait SpeechToTextProvider: Send + Sync {
    async fn transcribe(&self, request: TranscriptionRequest) -> HoneResult<TranscriptionResult>;
}
```

配置上可以先支持一种 OpenAI-compatible audio transcription provider 或本地 CLI provider，但必须遵守现有密钥治理方向：

- 凭证来自 `config.yaml` 或未来 secret ref，不从父进程环境变量兜底。
- provider capability 出现在 `/api/meta` 或专门 `/api/voice/capabilities`。
- 转写请求和响应写入最小 audit metadata；不要把完整音频或完整 transcript 写入普通日志。
- 如果 `Usage Entitlement Ledger` 后续落地，记录 `voice_transcription_seconds` 或 `voice_transcription_minutes` 用量。

### 4. Draft 抽取与确认

转写完成后，不直接让模型执行用户意图。建议使用两段式：

1. 规则层先做 ticker/date/action 粗提取：
   - ticker-like tokens：`AAPL`、`TSLA`、`NVDA`。
   - 常见动作：复盘、提醒、更新画像、查证、比较、加入观察。
   - 高风险词：买入、卖出、加仓、清仓、期权。
2. LLM 或现有 agent 只生成 `VoiceCaptureDraft`，并标注是否需要确认。

确认后再进入对应流程：

- `question`：把确认 transcript 作为用户消息进入 `AgentSession::run()`。
- `profile_update_candidate`：生成带 memo ref 的 `company_portrait` prompt，要求只写长期 thesis、证据、风险或证伪条件。
- `reminder_candidate`：生成 scheduled task draft，不直接启用。
- `research_thread_note`：接入 Investment Thread Workbench 的 future store；未落地前可只作为 memo draft。

### 5. API 与前端

Admin API：

- `GET /api/voice-memos?actor=&status=&symbol=`
- `GET /api/voice-memos/:id`
- `POST /api/voice-memos/:id/retry-transcription`
- `POST /api/voice-memos/:id/drafts/:draft_id/confirm`
- `POST /api/voice-memos/:id/discard`
- `DELETE /api/voice-memos/:id`

Public API：

- `GET /api/public/voice-memos`
- `GET /api/public/voice-memos/:id`
- `POST /api/public/voice-memos/:id/drafts/:draft_id/confirm`
- `POST /api/public/voice-memos/:id/discard`
- `DELETE /api/public/voice-memos/:id`

Public API 必须从 `hone_web_session` 推导 actor，不接受 query actor。IM 入口只返回当前消息相关 memo 的短状态和确认命令。

前端改动：

- Public chat composer 对音频附件显示“将转写为语音备忘录”。
- `/me` 或 `/portfolio` 增加待确认 voice memo 列表。
- Admin 用户详情页显示 memo status、transcript preview、draft action 和失败原因。

### 6. 迁移与兼容

- 不迁移旧 audio/voice 附件；旧 session 仍按普通附件历史显示。
- STT 未配置时行为保持现状，但提示更明确：系统已收到音频，当前未启用转写。
- 第一版只对新上传且通过 policy 的音频生成 `VoiceMemo`。
- 原始音频删除后，不删除已经确认并进入 session/company profile 的派生文本，但派生文本应保留 `source_deleted=true`。
- 群聊 memo 不默认进入个人 actor；后续 linked workspace 落地后再做共享策略。

## 实施步骤

### Phase 1: Voice memo registry and policy

1. 在 `memory` 增加 `voice_memo` SQLite store、类型和单元测试。
2. 在附件摄取层对 `AttachmentKind::Audio` 生成 `VoiceMemo` 记录。
3. 为 Telegram voice/audio 入口返回 memo id；Web upload 增加音频类型识别。
4. 增加 capability API，STT 未配置时保持现有限制提示。

### Phase 2: Transcription provider and review UI

1. 增加 `SpeechToTextProvider` trait 和一个可配置 provider。
2. 实现异步转写任务、失败重试和 transcript metadata。
3. Public `/me` 或 `/portfolio` 展示待确认 memo；admin 用户详情展示过滤列表。
4. 增加删除、丢弃和 retry 操作。

### Phase 3: Draft extraction and agent handoff

1. 实现规则优先的 action/ticker/date 抽取。
2. 生成 `VoiceCaptureDraft`，不直接执行。
3. 用户确认后可转为 chat question、company portrait update draft 或 scheduled task draft。
4. IM 入口增加简短确认命令；复杂编辑跳 Web/desktop。

### Phase 4: Governance and metrics

1. 接入 usage/cost 记录：音频秒数、STT 成功率、失败原因。
2. 接入 data trust/export/delete：用户能导出 memo metadata 和 transcript，删除原始音频。
3. 增加 low-confidence 和 risky-action review 指标。
4. 根据真实使用决定是否支持更长音频、会议录音或本地 STT。

## 验证方式

### 自动化测试

- Rust unit tests：
  - `infer_attachment_kind` 继续识别 mp3/ogg/m4a/wav。
  - audio policy 对超大小、超时长、未知 mime 给出稳定 reason code。
  - `VoiceMemoStorage` create/list/update/delete/retry 状态转换。
  - public actor 权限：用户只能读取自己的 memo。
  - draft confirmation 不能跨 actor 执行。
- CI-safe regression：
  - 用 fake STT provider 跑一个短音频 fixture，验证状态从 `received` 到 `needs_confirmation`。
  - 低置信度 transcript 只生成 draft，不触发 `AgentSession::run()`。
  - 删除原始音频后，metadata 标记 `deleted_at`，路径不可再读。

### 手工验收

- Telegram 发送 30 秒 voice message：
  - 收到“正在转写”反馈。
  - Web `/me` 能看到待确认 memo。
  - 确认后能把 transcript 作为问题继续问 Hone。
- Public Web 上传 mp3：
  - 未配置 STT 时提示 capability 缺失。
  - 配置 fake/local STT 后出现 transcript preview。
- 管理端按 actor/status 过滤 memo，能看到失败原因和 retry。
- 含错误 ticker 的 transcript 必须要求用户确认，不得自动写入公司画像。

### 指标

- voice memo received -> transcript_ready 成功率。
- transcript_ready -> confirmed 转化率。
- low-confidence rate / user correction rate。
- voice memo -> chat question / profile draft / reminder draft 的转化。
- 平均转写耗时和每 actor 转写用量。

## 风险与取舍

- 风险：STT 误听导致投资结论错误。  
  取舍：第一版所有语音都先进入确认层；低置信度、ticker 冲突和交易动作词必须人工确认。

- 风险：音频包含敏感个人信息。  
  取舍：默认短 retention，提供删除原始音频；日志只写 memo id 和状态，不写完整 transcript。

- 风险：外部 STT provider 增加成本和隐私边界。  
  取舍：provider 必须显式配置；未配置时保留当前限制提示；后续可支持本地 STT。

- 风险：与 Investment Document Inbox 范围重叠。  
  取舍：Document Inbox 管文件资产和解析；本提案只管短语音备忘录、转写确认和行动草稿。音频原文件可作为 document-like artifact 被治理，但产品对象仍是 `VoiceMemo`。

- 风险：多渠道实现不一致。  
  取舍：先以 Telegram 和 Web upload 为最小闭环，Feishu/Discord 后续只接统一 store/API，不各自实现转写逻辑。

- 不做：不做实时语音通话，不做长会议全量纪要，不把未确认 transcript 写进公司画像，不做自动交易建议，不用语音绕过投资输出安全约束。

## 与已有提案的差异

查重范围覆盖 `docs/proposal/` 与 `docs/proposals/` 下全部现有提案，并重点全文检索了 `voice`、`audio`、`speech`、`transcript`、`document`、`inbox`、`research artifact`、`thread`、`evidence`、`capture` 等关键词。

- 不重复 `auto_p1_investment_document_inbox.md`：该提案解决用户上传文档如何成为长期证据资产，覆盖 PDF、截图、表格、压缩包和普通文件；本提案聚焦短语音备忘录的 STT、确认、ticker 消歧和行动草稿。
- 不重复 `auto_p1_investment_context_intake.md`：该提案补齐持仓、偏好、画像缺口等结构化上下文；本提案解决移动端语音输入如何进入这些结构化流程。
- 不重复 `auto_p1_investment-thread-workbench.md`：该提案管理长期研究议题；本提案可以把确认后的 memo 转为 thread note，但不设计 thread store 本身。
- 不重复 `auto_p1_research_artifact_library.md`：该提案管理深度研究报告和 PDF/Markdown 交付物；本提案处理用户碎片化语音输入，不生成正式研究报告。
- 不重复 `auto_p1_evidence_review_queue.md`：该提案把 event-engine/digest 外部事件送入复盘队列；本提案来源是用户主动语音输入。
- 不重复 `auto_p1_usage_entitlement_ledger.md`：该提案管理权益和成本；本提案只预留语音秒数/分钟数用量事件，不设计计费层。

差异结论：当前仓库已能识别和下载音频附件，但没有把语音变成可确认、可治理、可进入研究工作流的产品对象。Voice Memo Research Capture 是一个独立、可落地的 P2 增量方向。

## 文档同步说明

本轮只创建 proposal，不开始实施，不修改业务代码、测试代码、运行配置或 `docs/current-plan.md`。该任务属于定期产品/架构提案产出，未进入执行态，因此不满足动态计划落盘准入标准，也无需归档计划页。若后续开始实施，应按动态计划准入标准新增或复用 `docs/current-plans/voice-memo-research-capture.md`，并在改变附件摄取、STT provider、API、隐私保留策略或多渠道行为后同步更新 `docs/repo-map.md`、`docs/invariants.md`、相关 runbook 和必要的 handoff/archive 索引。
