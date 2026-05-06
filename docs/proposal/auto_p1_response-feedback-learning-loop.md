# Proposal: Response Feedback Learning Loop for Answer Quality

status: proposed
priority: P1
created_at: 2026-05-07 05:03:32 CST
owner: automation

## related_files

- `README.md`
- `AGENTS.md`
- `docs/repo-map.md`
- `docs/invariants.md`
- `docs/decisions.md`
- `docs/current-plan.md`
- `docs/proposal/auto_p0_investment_output_safety_gate.md`
- `docs/proposal/auto_p1_automation_intent_control_plane.md`
- `docs/proposal/auto_p1_delivery_decision_loop.md`
- `docs/proposal/auto_p1_evidence_review_queue.md`
- `docs/proposal/auto_p1_investment_context_intake.md`
- `docs/proposal/auto_p1_investment_document_inbox.md`
- `docs/proposal/auto_p1_linked-user-workspace.md`
- `docs/proposal/auto_p1_research_artifact_library.md`
- `docs/proposal/auto_p1_run_trace_workbench.md`
- `docs/proposal/auto_p1_trade_discipline_journal.md`
- `docs/proposal/auto_p1_usage_entitlement_ledger.md`
- `docs/proposals/desktop-bundled-runtime-startup-ux.md`
- `docs/proposals/skill-runtime-multi-agent-alignment.md`
- `packages/app/src/pages/chat.tsx`
- `packages/app/src/lib/messages.ts`
- `packages/app/src/lib/public-chat.ts`
- `packages/app/src/lib/api.ts`
- `crates/hone-web-api/src/routes/public.rs`
- `crates/hone-web-api/src/routes/chat.rs`
- `crates/hone-web-api/src/routes/history.rs`
- `crates/hone-web-api/src/routes/llm_audit.rs`
- `crates/hone-web-api/src/types.rs`
- `crates/hone-channels/src/agent_session/core.rs`
- `crates/hone-channels/src/agent_session/emitter.rs`
- `memory/src/session.rs`
- `memory/src/llm_audit.rs`
- `skills/company_portrait/SKILL.md`
- `skills/position_advice/SKILL.md`
- `skills/stock_research/SKILL.md`

## 背景与现状

Hone 当前已经有较强的运行证据和会话恢复基础：

- Public Web chat 通过 `packages/app/src/pages/chat.tsx` 流式发送消息，前端为本地新消息生成临时 `messageId()`，并在 `run_finished` 后重新拉取 `/api/public/history` 恢复服务端历史。
- `packages/app/src/lib/messages.ts` 会用 `index + role + content hash` 为历史消息生成稳定前端 key，但这个 key不是服务端消息 id，也不能作为跨设备、跨 API、跨渠道反馈锚点。
- `crates/hone-web-api/src/routes/history.rs` 把 session messages 投影成 `HistoryMsg`，目前只包含 role、content、subtype、synthetic、transcript_only 和 attachments，没有返回 message timestamp、message metadata、runner、trace 或可评价 id。
- `memory/src/session.rs` 的 `SessionMessage` 已有 `timestamp` 和可选 `metadata`，`AgentSession` 也支持 `with_message_id` / `with_message_metadata`，但 Web public/admin chat 主路径目前没有把用户可见的 assistant turn 锚定为可反馈对象。
- `crates/hone-channels/src/agent_session/core.rs` 会把成功 assistant turn 落盘，并保留 tool-call metadata、runner metadata、session metadata updates；这说明“反馈指向哪一轮回答”可以落到现有会话模型上，而不是重新发明聊天存储。
- `memory/src/llm_audit.rs` 和 `/api/llm-audit` 已记录模型调用、token、latency、错误和请求响应详情；但这些是运维审计，不是用户对最终答案质量的显式信号。
- Public Web 用户已通过邀请码、手机号、TOS、cookie session 和日额度进入产品化路径；这类用户最适合在答案层收集轻量反馈，帮助 Hone 判断留存风险、回答质量和付费转化前的阻塞。
- 多渠道 IM 端、桌面 bundled runtime 和管理端已经共享 `AgentSession` / `ActorIdentity` / session storage，但没有统一的“这条回复有用吗、哪里错了、是否解决了问题”的信号入口。

这意味着 Hone 已能回答“系统跑了什么”，但还不能系统性回答“用户觉得这次回答是否有用，为什么无用，下一次应该怎么改”。对投资研究助手来说，这个缺口很关键：用户未必会主动报告“这次分析忽略了我的持仓上下文”“这条宏观归因太武断”“这份回答没有给出证据来源”“我想要更短的行动清单”。如果这些信号只留在聊天文本或流失行为里，产品很难形成可执行的质量改进闭环。

## 问题或机会

这是 P1，因为它直接影响核心体验、留存、付费转化、模型/runner 评估和运维优先级，但不要求重构 agent runtime。它可以先作为只追加的反馈层落地，再逐步把高价值负反馈转为 safety eval、run trace、公司画像修正或产品 backlog。

当前缺口主要有六类：

1. 用户满意度没有一等数据。
   Public chat 页面没有 thumbs up/down、纠错、原因分类或“已解决”状态。管理员能看会话、日志和 LLM audit，但无法按用户、ticker、runner、skill 或功能场景看回答质量。

2. 前端消息 id 不是服务端反馈锚点。
   历史消息 key 来自内容 hash，适合渲染稳定性，不适合作为持久反馈 id。内容相同、compaction、history window、跨端拉取或未来 message metadata 扩展都可能让反馈无法准确回写。

3. 负反馈不能进入改进队列。
   用户觉得回答错误时，系统无法结构化记录原因：事实错、忽略上下文、太啰嗦、不够具体、未遵守投资边界、没有引用证据、工具失败、格式难读、未解决问题。维护者只能靠主动访谈或手工 grep。

4. 回答质量与运行证据断开。
   LLM audit、prompt audit、session metadata、未来 Run Trace Workbench 都能解释运行过程，但缺少用户打分作为结果标签。没有标签，就很难比较 runner、model、prompt、skill 或 safety gate 调整是否真正改善用户体验。

5. 个性化偏好无法从显式反馈学习。
   Hone 有公司画像、portfolio notes、notification prefs 和 skills，但没有一个低风险的 per-actor preference memory 表达“这个用户偏好短答案 / 更重视反证 / 需要先看结论 / 不喜欢未经证据的宏观归因”。当前只能靠用户在自然语言里反复提醒。

6. 商业化和增长缺少质量漏斗。
   Public trial 用户的转化不只取决于剩余额度，也取决于前几次回答是否解决问题。如果系统无法衡量“首轮有用率”“负反馈后是否挽回”“哪些能力最常被批评”，就很难把增长优化落到产品和工程任务上。

## 方案概述

新增 actor-scoped 的 `ResponseFeedbackLearningLoop`：围绕每条用户可见 assistant answer 建立轻量反馈、原因分类、管理端复盘、质量指标和受控偏好学习。

核心对象建议：

1. `AssistantAnswerRef`
   稳定引用一次用户可见 assistant turn，包含 `answer_id`、actor、session_id、session_identity、message_index 或 message timestamp、channel、origin、runner/model、optional trace_id、optional llm_audit_ids、created_at。

2. `ResponseFeedback`
   用户或管理员提交的反馈，包含 `feedback_id`、answer_id、rating、reason_codes、free_text、resolved_state、source、created_at、created_by_actor。

3. `FeedbackReasonCode`
   稳定分类，例如 `helpful`、`solved`、`fact_error`、`ignored_context`、`missing_evidence`、`too_generic`、`too_verbose`、`unsafe_investment_tone`、`wrong_format`、`tool_or_data_failure`、`not_finance_boundary`、`other`。

4. `FeedbackReviewItem`
   管理端复盘队列项，用于把高严重度负反馈转成后续动作：创建 safety eval case、关联 run trace、补公司画像、更新 skill prompt、调整 public chat UX、标记为已处理。

5. `PreferenceCandidate`
   从重复反馈中提取的 per-actor 偏好草稿，例如“回答先给结论后给证据”“默认列出反证条件”“宏观问题必须说明数据时间”。第一版只作为用户可确认的草稿，不自动写入系统 prompt。

第一版应保持保守边界：

- 不把 thumbs down 自动喂进 live prompt。
- 不把用户自由文本反馈直接写入公司画像或长期系统指令。
- 不让管理员跨 actor 查看 public 用户隐私内容，除非已经具备相同的 admin 会话查看权限。
- 不把反馈当成事实真相；它是质量信号，需要和 trace、audit、session、工具结果一起判断。

## 用户体验变化

### Public 用户端

- 每条完成的 assistant 消息下方增加轻量反馈控件：
  - `有帮助`
  - `没解决`
  - `有错误`
  - `太泛 / 太长`
- 用户点负反馈后出现一个简短原因面板，可选择原因并补一句说明。
- 对负反馈的即时响应不是重新跑模型，而是确认已记录，并提供两个清晰动作：
  - `补充说明继续问`
  - `把这条发给管理员复盘`
- 如果用户多次选择同类偏好，例如“太长”，系统可以提示“以后默认更简短吗？”用户确认后才写入偏好草稿。
- Public `/me` 或账户页展示最近反馈状态：哪些已记录、哪些已被处理、是否已应用个人回答偏好。

### 管理端

- 新增 `Feedback` 页面或在 `/sessions` / 未来 `/traces` 中增加反馈 tab。
- 列表支持按 actor、channel、rating、reason、runner、model、skill、created_at 过滤。
- 详情页展示：
  - 用户问题摘要与 assistant 回复摘要。
  - 反馈原因和用户补充说明。
  - session message metadata、LLM audit 链接、prompt audit / trace 链接。
  - 是否涉及公司画像、portfolio、document、scheduled task 或 notification。
- 管理员可以把反馈标记为：
  - `needs_investigation`
  - `model_quality`
  - `missing_context`
  - `product_ux`
  - `safety_eval_candidate`
  - `resolved`
- 管理端质量仪表盘展示 7 天趋势：回答有用率、负反馈原因分布、首轮解决率、按 runner/model 的负反馈率、负反馈后继续对话率。

### 桌面端

- Desktop bundled/remote 模式复用同一 Web feedback UI。
- 桌面 dashboard 可显示“最近负反馈”和“待复盘反馈数”，帮助本地维护者从桌面进入问题定位，而不是只看日志。
- 本地个人用户可以关闭产品反馈上传，但本地仍可保留私有质量笔记，用于调整自己的回答偏好。

### 多渠道

- Feishu / Telegram / Discord 不需要一开始做复杂交互。第一版可以支持轻量回复：
  - `/feedback helpful`
  - `/feedback wrong`
  - `/feedback too long`
  - 回复某条 bot 消息并写“反馈：缺少证据”
- 渠道 adapter 把 reply-to-bot message id 或最近 assistant turn 解析成 `AssistantAnswerRef`。
- 群聊必须记录触发 actor 和 `SessionIdentity`，避免一个群成员的偏好自动影响整个群。

## 技术方案

### 1. 给用户可见 assistant turn 增加稳定 answer id

在 `AgentSession` 成功持久化 assistant turn 时生成 `answer_id`，写入 assistant message metadata。建议格式为 `ans_<timestamp>_<random>`。

兼容策略：

- 旧消息没有 `answer_id` 时，`/api/history` 可以临时返回 inferred id，例如 `legacy:<session_id>:<index>:<hash>`，但写反馈时应提示这是 best-effort，不能作为长期训练标签。
- 新消息同时保留 timestamp、runner、model、origin、message_id、future trace_id 等 metadata，方便后续和 Run Trace Workbench 对齐。
- Public/admin history API 扩展 `HistoryMsg`，返回 `id`、`timestamp`、`metadata_summary` 或至少 `answer_id`。前端渲染 key 可以继续用稳定 hash，但反馈提交必须使用服务端 `answer_id`。

### 2. 新增反馈存储

在 `memory` 增加 `response_feedback.rs`，优先使用 SQLite，便于按 actor、reason、时间和 answer_id 查询：

```text
response_feedback (
  feedback_id TEXT PRIMARY KEY,
  answer_id TEXT NOT NULL,
  actor_channel TEXT NOT NULL,
  actor_user_id TEXT NOT NULL,
  actor_channel_scope TEXT,
  session_id TEXT NOT NULL,
  channel TEXT NOT NULL,
  source TEXT NOT NULL,
  rating TEXT NOT NULL,
  reason_codes TEXT NOT NULL,
  free_text TEXT,
  created_by_actor TEXT,
  created_at TEXT NOT NULL,
  resolved_state TEXT NOT NULL DEFAULT 'open',
  admin_tags TEXT NOT NULL DEFAULT '[]',
  linked_trace_id TEXT,
  linked_llm_audit_ids TEXT NOT NULL DEFAULT '[]'
)
```

写入必须幂等：同一 actor 对同一 answer 的同一 source 默认 upsert，避免用户重复点击造成统计膨胀。管理员补充 review note 可以追加单独事件表，保留审计历史。

### 3. API 设计

Public 路由：

- `POST /api/public/feedback`
  - 从 cookie session 推导 actor，只允许给当前 actor 的 answer_id 提交反馈。
- `GET /api/public/feedback?answer_id=...`
  - 返回当前用户对该 answer 的反馈状态。

Admin 路由：

- `GET /api/feedback`
  - 支持 actor、channel、rating、reason、status、time range、runner/model 查询。
- `GET /api/feedback/:id`
  - 返回反馈、answer 摘要、session link、audit links。
- `POST /api/feedback/:id/review`
  - 更新处理状态、admin tags、关联 bug/eval/proposal。

IM 工具：

- 新增轻量 `response_feedback` tool 或 pre-session command handler。
- 对 reply-to-bot 消息优先使用 channel message id；没有 reply 时使用该 actor 最近一个 assistant answer。

### 4. 前端与产品面

- `packages/app/src/pages/chat.tsx` 在 assistant bubble 完成后渲染反馈按钮。
- `packages/app/src/lib/api.ts` 增加 public/admin feedback API client。
- `packages/app/src/lib/public-chat.ts` 将 `HistoryMsg.answer_id` 映射到 `PublicChatMessage.answerId`。
- 管理端新增 feedback 页面，第一版可复用 sessions/logs 的简洁列表风格，不需要复杂分析图。

### 5. 偏好学习边界

第一版不把反馈自动注入 prompt。建议先实现两个只读或确认式能力：

- `FeedbackPatternSummary`：按 actor 汇总最近 30 天重复原因，例如“5 次选择 too_verbose”。
- `PreferenceCandidate`：当重复信号达到阈值，生成用户可确认的偏好草稿，存入未来的 profile/prefs 层，而不是直接改全局 prompt。

如果后续要让偏好进入 prompt，必须遵守 `docs/invariants.md` 的 prompt layering 约束：用户偏好属于 mutable session/actor context，不能写回 static system prefix，也不能覆盖金融安全边界。

## 实施步骤

### Phase 1: Answer id 与只写反馈

- 给新 assistant turns 增加 `answer_id` metadata。
- 扩展 public/admin history projection，返回 `answer_id` 与 timestamp。
- 新增 `memory::response_feedback` SQLite 存储和 public `POST /api/public/feedback`。
- Public chat 增加 thumbs / reason 面板。
- 增加单元测试覆盖 answer_id 生成、history 投影、actor 权限校验、重复提交幂等。

### Phase 2: 管理端复盘队列

- 增加 admin `GET /api/feedback` / detail / review route。
- 新增管理端 Feedback 页面，支持筛选、状态流转和跳转 session / llm-audit。
- 把高严重度 reason，例如 `fact_error`、`unsafe_investment_tone`、`tool_or_data_failure`，默认进入 `needs_investigation`。
- 与未来 Run Trace Workbench 对齐：有 trace_id 时从反馈详情跳转到 trace。

### Phase 3: 多渠道轻量反馈

- 在 channel pre-session intercept 或共享 command 层支持 `/feedback`。
- Feishu / Telegram / Discord reply-to-bot 场景解析最近 assistant answer。
- 对群聊反馈保持 actor 与 session 分离，禁止个人偏好自动污染群 session。
- 增加手工回归脚本，覆盖至少一个 IM channel 的最近回答反馈链路。

### Phase 4: 质量指标与偏好候选

- 增加 7/30 天质量聚合：helpful rate、negative reason distribution、first-answer solved rate、negative feedback follow-up rate。
- 生成 `FeedbackPatternSummary` 和 `PreferenceCandidate`，只展示待确认偏好。
- 将被确认的偏好接入 actor-level mutable context，并建立明确关闭/重置入口。

## 验证方式

- 单元测试：
  - `AgentSession` 成功 assistant turn 自动写入唯一 `answer_id`。
  - `history_from_messages` 返回 answer_id/timestamp，旧消息可 degraded。
  - feedback storage upsert 幂等，reason code 校验严格。
  - public feedback 只能写当前 cookie actor 的 answer。
  - admin review 状态流转保留审计。
- 前端测试：
  - assistant bubble 完成后显示反馈控件。
  - 正反馈一键提交，负反馈弹出原因面板。
  - 历史消息恢复后仍能显示该 answer 的反馈状态。
  - 反馈失败时不影响聊天主流程。
- 回归脚本：
  - CI-safe：public login mock 或 storage-level API test，验证提交反馈、重复提交、跨 actor 拒绝。
  - Manual：Feishu/Telegram 回复 bot 消息后 `/feedback wrong` 能定位最近 assistant answer。
- 指标：
  - Public chat 最近 7 天至少 20% 的完成回答获得显式或隐式反馈入口曝光。
  - 负反馈中 `reason_codes` 非空比例超过 70%。
  - 管理端能在 2 次点击内从反馈跳到 session/audit/trace 证据。
  - 高严重度反馈从创建到 triage 的中位时间可被统计。
- 手工验收：
  - 新用户在 public chat 中能对回答打分，不需要理解 session id。
  - 管理员能按 actor 找到某条负反馈，并看到对应问题和回答摘要。
  - 本地 desktop remote/bundled 都能显示相同反馈状态。

## 风险与取舍

- 反馈会引入隐私和信任问题。默认只保存必要摘要和 answer id；自由文本反馈要遵守现有 admin 可见边界，不额外扩大跨 actor 读取权限。
- 负反馈不是事实。不能因为用户点错或情绪化反馈就自动改 prompt、画像或 safety policy；必须经过 review 或用户确认。
- Answer id 需要谨慎兼容旧 session。旧消息可以 best-effort 反馈，但高质量统计应区分 `stable` 与 `legacy_inferred`。
- 过多反馈按钮会干扰聊天体验。Public 端应保持轻量，默认折叠原因面板，不在 streaming 中展示。
- 多渠道 reply-to-bot 定位可能不稳定。第一版允许 fallback 到最近 assistant answer，并在提交结果中明确说明定位对象。
- 质量指标可能诱导模型迎合用户。Hone 的投资安全边界高于用户满意度；`unsafe_investment_tone` 等反馈只能帮助复盘，不能降低全局金融约束。

## 与已有提案的差异

- 与 `auto_p0_investment_output_safety_gate.md` 不重复：安全门禁关注送达前是否允许输出、降级或拦截；本提案关注送达后的用户反馈、质量标签和改进闭环。
- 与 `auto_p1_run_trace_workbench.md` 不重复：run trace 解决一次运行怎么跑、哪里失败；本提案给 trace 增加用户结果标签，帮助判断哪类运行真的伤害体验。
- 与 `auto_p1_delivery_decision_loop.md` 不重复：delivery decision 解释事件为什么推/不推；本提案评价 assistant answer 本身是否解决问题。
- 与 `auto_p1_evidence_review_queue.md` 不重复：evidence queue 处理市场事件是否应更新 thesis；本提案处理用户对回答质量的显式评价，不直接改画像。
- 与 `auto_p1_investment_document_inbox.md` 不重复：document inbox 管理用户上传材料；本提案管理回答反馈和偏好信号。
- 与 `auto_p1_investment_context_intake.md` 不重复：context intake 补齐持仓、画像、偏好和自动化前置条件；本提案补齐回答后质量反馈。
- 与 `auto_p1_trade_discipline_journal.md` 不重复：trade journal 记录用户投资动作前后的纪律流程；本提案记录对 assistant 回复本身的满意度和问题类型。
- 与 `auto_p1_usage_entitlement_ledger.md` 不重复：entitlement ledger 处理权益、用量和成本；本提案处理质量、留存和学习信号，可作为未来转化分析的输入。
- 与 `auto_p1_automation_intent_control_plane.md` 不重复：automation intent 关注 cron/heartbeat 变更前的 preview/approval；本提案关注任意回答的用户评价。
- 与 `auto_p1_linked-user-workspace.md` 不重复：linked workspace 解决跨渠道资产归属；本提案可在未来迁移到 workspace 级反馈，但第一版仍按 actor 隔离。
- 与 `auto_p1_research_artifact_library.md` 不重复：research artifact 保存深度研究交付物；本提案保存用户对回答和研究体验的反馈标签。
- 与 `docs/proposals/skill-runtime-multi-agent-alignment.md` 不重复：skill/runtime alignment 关注 agent 能力执行；本提案只采集执行结果的用户质量信号。
- 与 `docs/proposals/desktop-bundled-runtime-startup-ux.md` 不重复：desktop startup UX 处理 bundled runtime 接管和启动恢复；本提案只在桌面复用反馈 UI 与质量入口。
