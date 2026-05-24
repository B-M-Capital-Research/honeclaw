# Proposal: Expert Review Desk for High-Value Human Follow-Up

status: proposed
priority: P2
created_at: 2026-05-24 08:04:17 +0800
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
- `docs/proposal/auto_p0_investment_output_safety_gate.md`
- `docs/proposal/auto_p0_operator-access-audit.md`
- `docs/proposal/auto_p1_invite_activation_funnel.md`
- `docs/proposal/auto_p1_response-feedback-learning-loop.md`
- `docs/proposal/auto_p1_redacted-support-bundle.md`
- `docs/proposal/auto_p1_research_artifact_library.md`
- `docs/proposal/auto_p1_user-data-trust-center.md`
- `docs/proposal/auto_p1_usage_entitlement_ledger.md`
- `docs/proposal/auto_p2_collaborative-research-rooms.md`
- `docs/proposal/auto_p2_investment-committee-mode.md`
- `docs/proposals/desktop-bundled-runtime-startup-ux.md`
- `docs/proposals/skill-runtime-multi-agent-alignment.md`
- `memory/src/web_auth.rs`
- `crates/hone-web-api/src/routes/public.rs`
- `crates/hone-web-api/src/routes/web_users.rs`
- `crates/hone-web-api/src/routes/research.rs`
- `crates/hone-web-api/src/routes/users.rs`
- `crates/hone-web-api/src/routes/llm_audit.rs`
- `crates/hone-channels/src/agent_session/mod.rs`
- `crates/hone-channels/src/prompt.rs`
- `packages/app/src/pages/chat.tsx`
- `packages/app/src/pages/public-me.tsx`
- `packages/app/src/pages/settings.tsx`
- `packages/app/src/components/research-detail.tsx`

## 背景与现状

Hone 已经从本地投资聊天助手演进成多入口产品：public Web 使用手机号与 invite 白名单登录，管理端能创建 invite、查看用户和额度，Hone Cloud API key 可以让外部客户端调用，桌面端支持 bundled/remote backend，多渠道 IM 和 event-engine 能主动触达用户。README 也明确强调 Hone 的价值不是“哄用户开心的聊天玩具”，而是帮助用户保持投资纪律、跟踪公司、沉淀长期研究资产。

当前仓库已经具备几类和人工跟进相邻的基础能力：

- `memory/src/web_auth.rs` 保存 invite user、手机号、登录态、TOS、API key 和 last login，是 public 用户与运营管理的最小身份层。
- `crates/hone-web-api/src/routes/web_users.rs` 允许管理员创建、停用、重置 invite 和 API key，并展示每日 quota / in-flight / session 状态。
- `crates/hone-web-api/src/routes/public.rs` 承载 public `/chat`、上传、历史、OpenAI-compatible chat completions 和 `/me`。
- `crates/hone-channels/src/prompt.rs` 注入全局金融边界，要求避免直接荐股、提醒用户独立思考，并把回答限定在金融投资领域。
- `crates/hone-web-api/src/routes/research.rs` 已有深度研究外部代理入口，前端 `ResearchDetail` 能展示长文研究进度和 Markdown/PDF 交付。
- 既有提案已覆盖输出安全门、回答反馈、invite 激活、研究资产库、技术诊断包、operator 权限审计、协作房间和投委会模式。

这些能力能让 Hone 运行、试用、回答、生成研究、收集反馈和排障，但还缺一个产品层回答：当 AI 明确不能闭环、用户给出负反馈、高价值 trial 用户接近付费转化、研究报告需要人工判断、或合规边界要求“只做教育性解释”时，系统如何把这类信号变成一个可分派、可脱敏、可追踪、可回到用户的人工复核工单。

目前这类跟进只能靠聊天外的人工记忆、截图、私信或管理端手工查用户。对一个投资助手产品而言，这会让“AI 做不到时怎么办”和“高价值用户如何被服务”成为隐性流程，而不是系统能力。

## 问题或机会

这是 P2 级提案：它不应排在 P0 安全、访问控制和核心运行可靠性之前，但它能把 Hone 从纯自助 agent 扩展为可运营的高触感投资研究服务，直接影响 trial 转化、留存、研究质量和商业化包装。

当前缺口主要有六个：

1. **AI 失败或拒答没有人工接力。**
   安全门、反馈和 run trace 可以说明某条回答被拦截、失败或被用户认为不好，但没有一等对象把“这需要人看”交给 support / research / product operator。

2. **高价值用户信号分散。**
   Invite 激活、usage entitlement、response feedback、research artifact、public API key 使用、portfolio 上下文完整度都可能表示用户值得跟进。现在没有一个统一 review queue 汇总这些信号。

3. **人工跟进容易过度暴露隐私。**
   直接把聊天记录、持仓、上传文件或公司画像转发给人看，会破坏 local-first 和 actor isolation 的信任。缺少默认脱敏、最小上下文和用户授权边界。

4. **研究复核与客服排障混在一起。**
   技术支持需要日志和配置；投资研究复核需要用户问题、已引用证据、画像摘要和安全边界。二者不应共用一个“发截图给管理员”的临时流程。

5. **管理端没有跟进 SLA 和结果闭环。**
   管理员可以看到用户、日志、LLM audit 和研究页，但无法给某个用户问题创建状态机：open、triaged、waiting_user、answered、declined、closed。

6. **商业化体验缺少“专业服务”承接。**
   深度研究、协作房间、投委会模式和自助 billing 之后，高价值用户自然会期待“我能否让人复核一次这份 thesis / 这次回答”。没有受控人工复核层，升级体验只能卖更多模型调用，而不是卖更可靠的研究服务。

机会是新增 **Expert Review Desk**：一个 actor-scoped、脱敏优先、非交易执行的人工复核队列。它不替代 AI 回答，不让人直接下投资建议，不连接券商交易；它只为高价值或高风险场景提供可审计的人工跟进工作流。

## 方案概述

新增 `ExpertReviewDesk` 产品层，核心是把多来源信号转成可处理的 review case。

核心对象：

1. `ExpertReviewCase`
   一条人工复核工单，包含 case id、actor、source、category、priority、status、subject、created_at、due_at、assigned_operator、redaction level 和用户授权状态。

2. `ReviewCaseSource`
   来源类型：`user_requested_review`、`negative_feedback`、`safety_gate_escalation`、`research_artifact_handoff`、`invite_activation_followup`、`api_user_support`、`operator_created`、`billing_or_entitlement_followup`。

3. `ReviewCaseSnapshot`
   脱敏上下文包：最近一条问题/回答摘要、相关 session id、research artifact id、company profile summary、portfolio exposure summary、run trace/support bundle reference。默认不含完整聊天、上传原文、成本价或 sandbox 路径。

4. `ReviewResponseDraft`
   人工或 agent-assisted 生成的回复草稿。必须标记为教育性研究反馈、风险提示或客服说明，不能伪装成 Hone 自动回答，也不能成为交易执行指令。

5. `ReviewOutcome`
   结果：`answered`、`declined_out_of_scope`、`needs_more_user_context`、`converted_to_research_task`、`converted_to_bug`、`converted_to_profile_handoff`、`closed_duplicate`。

原则：

- 默认 actor-scoped，不跨用户合并。
- 默认脱敏，只有用户显式请求人工复核时才附加更多上下文。
- 人工回复仍遵守 `prompt.rs` 的投资安全边界，不提供确定性买卖指令。
- 管理员访问正文必须经过未来 operator scope / audit；第一版可以先在本地或 owner 模式开放。
- 不接交易、不做券商账户、不做受监管投顾承诺。

## 用户体验变化

### 用户端

- Public `/chat` 在以下场景提供轻量入口：
  - 用户对回答点负反馈并选择“希望有人复核”。
  - Hone 拒绝高风险交易指令后，提示可提交“研究复核请求”，但明确不是投资建议。
  - 深度研究报告完成后，用户可请求“人工检查这份报告是否有明显遗漏”。
- Public `/me` 增加 `Review requests` 区块：
  - 查看已提交 case、状态、预计回复时间、是否需要补充信息。
  - 撤回请求或补充上下文。
  - 下载/删除已关闭 case 的用户可见记录，后续接入 Data Trust Center。
- 用户提交时看到清晰边界：
  - 将共享哪些内容。
  - 是否包含持仓摘要。
  - 人工复核只提供研究流程与风险提示，不代替独立决策。

### 管理端

- 新增 `Expert Review` 页面或在 Users 详情中增加 tab：
  - 按 priority、category、status、due_at、actor、source 过滤。
  - 显示脱敏 snapshot、相关 feedback/run trace/research artifact/mainline/profile summary 链接。
  - 支持分派 operator、添加内部 note、生成回复草稿、关闭或转换为 bug / research task / profile handoff。
- Settings 的 invite table 可显示未处理 review case 数，帮助运营识别高意向用户。
- Operator 权限提案落地后，case 正文读取、回复发送、关闭都写 admin audit event。

### 桌面端

- Desktop remote mode 复用远端 Expert Review API。
- Desktop bundled/local owner mode 可以显示“本机 review inbox”，用于个人把复杂问题标记为稍后复盘，而不是提交给远端人。
- 当 desktop 连接 remote backend 时，UI 明确提示提交 review 会把脱敏上下文上传给远端服务。

### 多渠道

- Feishu / Telegram / Discord 私聊支持：
  - “转人工复核这条回答”
  - “查看我的复核请求”
  - “补充刚才那个 case：我关注的是毛利率而不是短期股价”
- 群聊中不直接创建包含个人持仓的 case；默认引导用户私聊确认共享范围。
- IM 只返回短状态和 case id，不推送完整人工回复中的敏感上下文到群聊。

## 技术方案

### 1. 存储模型

建议在 `memory` 新增 `expert_review.rs`，使用 SQLite：

```text
expert_review_cases (
  case_id TEXT PRIMARY KEY,
  actor_channel TEXT NOT NULL,
  actor_user_id TEXT NOT NULL,
  actor_scope TEXT,
  source TEXT NOT NULL,
  category TEXT NOT NULL,
  priority TEXT NOT NULL,
  status TEXT NOT NULL,
  subject TEXT NOT NULL,
  redaction_level TEXT NOT NULL,
  user_authorized_full_context BOOLEAN NOT NULL DEFAULT FALSE,
  assigned_operator_id TEXT,
  due_at TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  closed_at TEXT
)

expert_review_snapshots (
  snapshot_id TEXT PRIMARY KEY,
  case_id TEXT NOT NULL,
  snapshot_kind TEXT NOT NULL,
  summary_markdown TEXT NOT NULL,
  source_refs_json TEXT NOT NULL,
  redaction_report_json TEXT NOT NULL,
  created_at TEXT NOT NULL
)

expert_review_messages (
  message_id TEXT PRIMARY KEY,
  case_id TEXT NOT NULL,
  author_kind TEXT NOT NULL,
  author_ref TEXT,
  visibility TEXT NOT NULL,
  content_markdown TEXT NOT NULL,
  created_at TEXT NOT NULL
)

expert_review_outcomes (
  outcome_id TEXT PRIMARY KEY,
  case_id TEXT NOT NULL,
  outcome TEXT NOT NULL,
  linked_ref_json TEXT NOT NULL,
  created_at TEXT NOT NULL
)
```

`visibility` 区分 `internal_note`、`user_visible_draft`、`user_visible_sent`。不要把 operator 内部 note 写进用户聊天历史。

### 2. Case 创建入口

第一版支持四类入口：

- Public feedback escalation：从 Response Feedback Learning Loop 或 chat UI 创建。
- User explicit request：public `/chat`、IM 私聊或 `/me` 表单。
- Research artifact handoff：从报告详情创建人工复核 case。
- Operator created：管理员从用户详情、LLM audit 或 research 页面创建 case。

后续再接安全门、billing、entitlement、activation funnel 和 collaborative rooms。

### 3. 脱敏与上下文快照

新增 `ReviewSnapshotBuilder`，复用 support bundle 和 future data trust 的 redaction 思路：

- `minimal`: subject、用户原问题摘要、回答摘要、source ids、错误/反馈 reason。
- `research`: 加入 research artifact 摘要、公司画像摘要、引用章节，不含完整原文。
- `portfolio_summary`: 只加入 ticker、关注/持仓方向、风险标签，不含 shares、avg_cost、成本和交易意图。
- `full_user_authorized`: 用户显式确认后，允许附加更完整的聊天片段或报告 excerpt，但仍不含 secret、绝对路径、session token。

每次 snapshot 都保存 `redaction_report_json`，列出哪些字段被省略，方便 operator 知道信息缺口。

### 4. API

Public API：

- `GET /api/public/expert-reviews`
- `POST /api/public/expert-reviews`
- `GET /api/public/expert-reviews/:case_id`
- `POST /api/public/expert-reviews/:case_id/messages`
- `POST /api/public/expert-reviews/:case_id/withdraw`

Admin API：

- `GET /api/expert-reviews?status=&priority=&category=&actor=`
- `GET /api/expert-reviews/:case_id`
- `POST /api/expert-reviews/:case_id/assign`
- `POST /api/expert-reviews/:case_id/messages`
- `POST /api/expert-reviews/:case_id/send-response`
- `POST /api/expert-reviews/:case_id/close`
- `POST /api/expert-reviews/:case_id/convert`

Public API 从 `hone_web_session` 推导 actor，不接受任意 actor query。Admin API 在 operator access 落地前沿用现有 bearer token，但应在提案实施时优先和 operator scope/audit 对齐。

### 5. Agent 与人工回复边界

Expert Review 可以使用 agent 生成回复草稿，但发送前必须由 operator 确认：

1. Operator 打开 case，选择 `draft response`。
2. 系统把脱敏 snapshot、Hone 安全边界、用户问题和相关研究摘要交给 runner。
3. Agent 生成 `ReviewResponseDraft`，标注引用、边界和不确定性。
4. Operator 编辑后发送。
5. 用户在 `/me`、chat 或私聊收到人工复核回复。

不要把人工回复写成普通 assistant message。可以在会话历史中追加一条 typed system/user-visible event，例如 `expert_review_response(case_id)`，前端单独渲染，避免用户误以为这是实时模型回答。

## 实施步骤

### Phase 1: Case storage and admin queue

- 新增 `memory/src/expert_review.rs`、类型、SQLite schema 和单元测试。
- 新增 admin list/detail/create/close API。
- 管理端 Settings 或 Users 详情显示 open case count。
- 只支持 operator 手工创建 case，不接 public 提交。

### Phase 2: Public request flow

- Public `/me` 增加 review request 列表和详情。
- Public `/chat` 负反馈或用户显式点击后可创建 case。
- Snapshot builder 默认 minimal redaction，用户可选择是否附加更多上下文。
- IM 私聊支持创建和查看 case 状态。

### Phase 3: Research and safety integrations

- Research artifact 详情支持创建 `research_review` case。
- Safety gate 或 high-risk refusal 可创建 `safety_explanation_review` case，但只在用户请求时提交。
- Case 可转换为 research task、profile handoff、bug/handoff 或 support bundle reference。

### Phase 4: Operator workflow and metrics

- 增加 assign、due_at、SLA、internal note、response draft、send-response。
- 与 operator audit 记录正文读取、发送和关闭。
- 指标：open case、time to first response、case category、conversion to paid/research task、decline reason。

## 验证方式

- Rust 单元测试：
  - case 状态转换：open -> assigned -> waiting_user -> answered/declined/closed。
  - public actor 只能读取自己的 case。
  - snapshot redaction 不包含 API key、session token、绝对 sandbox 路径、avg_cost、shares。
  - user-authorized full context 仍保留 secret/path redaction。
- Web API 测试：
  - public create 忽略 actor query，只使用当前 session actor。
  - admin list 可按 status/priority/category 过滤。
  - send-response 后用户可见消息出现，internal note 不出现在 public detail。
- 前端测试：
  - `/me` 渲染空状态、open case、waiting_user、answered、withdrawn。
  - 管理端 case list 在移动和桌面视口不溢出，priority/status badge 可读。
  - chat 负反馈创建 case 后不丢失当前对话。
- 手工验收：
  - 创建 invite 用户，完成一次 chat，提交人工复核请求，admin 看到脱敏 snapshot，发送回复，public `/me` 可读。
  - 从研究报告创建复核 case，只共享报告摘要，不暴露后端本地 PDF 路径。
  - Telegram/Discord/Feishu 私聊创建 case，群聊中请求包含持仓时被引导到私聊。
- 产品指标：
  - review request 创建率、回复时长、用户补充信息率、closed outcome 分布。
  - trial 用户提交 review 后的 7 日留存和付费转化。

## 风险与取舍

- 风险：用户把人工复核理解成受监管投顾服务。
  取舍：所有文案与回复模板都限定为研究流程、证据检查、风险提示和产品支持，不输出交易指令，不承诺收益或适当性判断。

- 风险：人工队列扩大隐私暴露面。
  取舍：默认最小 snapshot，完整上下文需用户授权；operator 读取正文要接入 scope/audit；群聊默认不收集个人敏感数据。

- 风险：运营成本过高。
  取舍：P2 优先级，第一版只处理显式用户请求和高价值入口，不自动把所有负反馈都变成人工工单。

- 风险：和技术支持工单混淆。
  取舍：Expert Review 处理研究/回答/高价值用户跟进；技术诊断继续走 Redacted Support Bundle 和 Run Trace。

- 风险：AI 草稿被未经审阅发送。
  取舍：所有 expert response draft 默认需要 operator 确认；自动发送只允许状态通知，不允许自动发送研究判断。

- 不做：不接券商交易，不做正式投顾合规系统，不创建公开专家市场，不把 operator 内部 note 写进普通 chat history，不替代协作房间或投委会模式。

## 与已有提案的差异

查重范围覆盖 `docs/proposal/` 与 `docs/proposals/` 下全部现有提案，重点比对了 safety gate、operator access、response feedback、invite activation、research artifact library、support bundle、collaborative rooms、investment committee mode、entitlement、data trust 和 skill runtime。

- 不重复 `auto_p0_investment_output_safety_gate.md`：安全门决定高风险输出是否可送达或需拦截；本提案处理用户或 operator 如何把特定问题提交给人工复核并完成回复。
- 不重复 `auto_p1_response-feedback-learning-loop.md`：feedback loop 收集单条回答质量信号；本提案把少量高价值或高风险反馈升级为可分派、可回复的人工 case。
- 不重复 `auto_p1_redacted-support-bundle.md`：support bundle 面向技术诊断证据包；本提案面向研究/产品跟进工单，只引用诊断包，不打包运行环境。
- 不重复 `auto_p1_research_artifact_library.md`：artifact library 保存研究报告和画像交接；本提案允许针对报告创建人工复核 case，但不定义报告存储本身。
- 不重复 `auto_p0_operator-access-audit.md`：operator access 解决后台人员身份、权限和审计；本提案是 operator 可处理的一类业务队列，后续依赖其权限底座。
- 不重复 `auto_p1_invite_activation_funnel.md`：activation funnel 判断用户是否跨过价值里程碑；本提案处理需要人跟进的具体 case 和回复闭环。
- 不重复 `auto_p2_collaborative-research-rooms.md`：research rooms 是多用户协作空间；expert review 是用户与服务方之间的一对一或后台队列，不共享 room assets。
- 不重复 `auto_p2_investment-committee-mode.md`：committee mode 是多 agent/角色审查；expert review desk 是人工复核和运营跟进流程，可以把 committee verdict 作为 case 附件但不替代它。
- 不重复 `auto_p1_user-data-trust-center.md`：data trust 处理导出/删除/隐私权利；expert review 需要接入其数据治理，但核心是 case 状态机与人工回复。

查重结论：现有 proposal 已覆盖自动回答质量、安全拦截、技术诊断、研究资产、团队协作和后台权限，但尚未覆盖“AI 无法闭环或高价值用户需要人工跟进时，如何用脱敏 snapshot、case queue、operator workflow 和用户可见回复形成闭环”。因此本主题是新的、可落地的 P2 产品/架构提案。

## 文档同步说明

本轮只新增 proposal，不开始实施方案，因此不更新 `docs/current-plan.md`、`docs/repo-map.md`、`docs/invariants.md`、运行配置、业务代码或测试代码。若后续实际落地，应按动态计划准入标准新增或复用 `docs/current-plans/expert-review-desk.md`，并在新增 review storage、public/admin API、operator workflow、IM 命令、用户隐私授权或人工回复审计后同步更新 `docs/repo-map.md`、`docs/invariants.md`、`docs/decisions.md` 和必要 handoff。
