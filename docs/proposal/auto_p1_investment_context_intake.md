# Proposal: Investment Context Intake and Gap Resolver

status: proposed
priority: P1
created_at: 2026-05-02 05:03:57 CST
owner: automation

## related_files

- `README.md`
- `docs/repo-map.md`
- `docs/invariants.md`
- `docs/decisions.md`
- `docs/current-plan.md`
- `docs/proposal/auto_p1_delivery_decision_loop.md`
- `docs/proposal/auto_p1_evidence_review_queue.md`
- `docs/proposal/auto_p1_linked-user-workspace.md`
- `docs/proposal/auto_p1_run_trace_workbench.md`
- `docs/proposal/auto_p1_usage_entitlement_ledger.md`
- `docs/proposals/desktop-bundled-runtime-startup-ux.md`
- `docs/proposals/skill-runtime-multi-agent-alignment.md`
- `memory/src/portfolio.rs`
- `memory/src/company_profile/{types,storage,markdown,transfer}.rs`
- `crates/hone-tools/src/portfolio_tool.rs`
- `crates/hone-tools/src/cron_job_tool.rs`
- `crates/hone-web-api/src/routes/portfolio.rs`
- `crates/hone-web-api/src/routes/public_digest.rs`
- `crates/hone-web-api/src/routes/notification_prefs.rs`
- `crates/hone-event-engine/src/subscription.rs`
- `crates/hone-event-engine/src/prefs.rs`
- `packages/app/src/pages/public-portfolio.tsx`
- `packages/app/src/pages/users.tsx`
- `packages/app/src/components/portfolio-detail.tsx`
- `packages/app/src/components/notification-preferences-card.tsx`
- `skills/company_portrait/SKILL.md`
- `skills/scheduled_task/SKILL.md`

## 背景与现状

Hone 当前已经具备投资上下文的多个底层部件：

- `PortfolioStorage` 按 `ActorIdentity` 存储持仓和关注列表，`Holding` 支持股票、期权、长期/短期 horizon、策略备注，以及 `tracking_only` 关注标的。
- `PortfolioTool` 允许 agent 在会话中查看、添加、更新、删除持仓与关注。
- Admin 端 `/users/:actorKey/portfolio` 可以手工维护持仓，`/users/:actorKey/profiles` 和 `/users/:actorKey/thesis` 可以查看公司画像与蒸馏结果。
- Public 端 `/portfolio` 读取当前 web actor 的 portfolio、notification prefs 中的 `investment_theses` / `investment_global_style`，以及 actor sandbox 里的 `company_profiles/*/profile.md`，并提示用户通过 `/chat` 维护画像。
- Event engine 的订阅层从 portfolio 构建 watch pool，通知偏好支持 `portfolio_only`、digest slots、quiet hours、source/kind allow/block、per-ticker thesis 和 global style。
- `company_portrait` skill 负责把系统性研究沉淀为 actor sandbox 下的 `company_profiles/<ticker>/profile.md` 和事件时间线。
- `scheduled_task` skill 可以结合 portfolio 创建定时简报、事件提醒和价格提醒。

这些能力已经足够支撑“个人投资研究助理”的核心叙事，但它们仍然分散在聊天、admin 表格、public 只读视图和后台蒸馏任务中。新用户或迁移用户最容易遇到的不是模型不会回答，而是 Hone 还不知道：

- 用户到底持有什么、只是关注什么、哪些标的是长期核心仓位。
- 哪些持仓已经有公司画像，哪些没有。
- 哪些画像可被 thesis 蒸馏识别，哪些因为 ticker/frontmatter/目录名问题被跳过。
- 用户希望多吵、多安静，哪些事件要即时推，哪些只进 digest。
- 第一个有价值的 scheduled task 应该是什么。

当前 public `/portfolio` 在空状态下给出“请在 /chat 里告诉 agent 你持有什么”，admin 端则提供手工表单。这是可用的，但把激活成本交给用户自己组织：用户要知道该说什么，agent 要在一次开放对话里同时完成建仓、建画像、设偏好、设提醒和解释限制。对一个以多渠道、自动化和长期记忆为卖点的产品来说，这条 activation path 还不够产品化。

## 问题或机会

### 问题

1. 投资上下文的“完整度”没有一等状态。
   现在 portfolio、company profile、thesis prefs、notification prefs、cron jobs 分别存在，但没有一个面向用户和管理员的 readiness model 说明哪些前置条件已满足、哪些缺口会影响 digest、推送或长期记忆。

2. Public 端空态没有下一步编排。
   `/portfolio` 能展示蒸馏后的投资上下文，但当 portfolio 为空、profile 缺失或 thesis 被跳过时，用户只能被动跳到 `/chat`。系统没有把用户引导进一个低摩擦的 intake 流程。

3. Agent 写入能力强，但缺少结构化 intake 护栏。
   `PortfolioTool` 和 `company_portrait` skill 已能修改长期状态。如果用户在自由聊天里一次性给出复杂持仓，agent 需要自己决定哪些字段写入、哪些追问、是否建画像、是否设置任务。缺少“收集草稿 -> 用户确认 -> 原子应用 -> 生成缺口清单”的产品契约。

4. Admin 端能代管数据，但很难判断一个用户是否已经“可用”。
   `/users` 聚合了持仓、画像、thesis、会话、研究任务，但没有按 actor 汇总“激活完成度”或“为什么这个用户收不到高质量 digest”。

5. 多渠道体验不一致。
   Web 用户从 public `/chat` 和 `/portfolio` 进入；Feishu/Telegram/Discord 用户从私聊或群聊进入；Desktop 用户从 bundled/remote backend 进入。它们共享 actor 数据，但缺少同一个 intake/gap resolver 语义。

### 机会

把“投资上下文初始化”做成 Hone 的 P1 产品层，可以直接提升：

- 新用户激活：从第一次登录到拿到有用 digest 的时间更短。
- 留存：用户看到缺口被逐步补齐，Hone 变成持续维护的研究工作台，而不是一次性聊天窗口。
- 推送质量：portfolio、profile、thesis、prefs 和 cron 都有明确来源，少发无关消息，少漏关键事件。
- 商业化承接：未来 entitlement/付费可以围绕“可监控标的数、画像数、自动任务数、digest 个性化”表达，而不是只卖对话次数。
- 运维效率：admin 能看出问题是无 portfolio、无 profile、蒸馏失败、通知关闭、quiet held，还是 channel delivery 问题。

## 方案概述

新增一个“Investment Context Intake and Gap Resolver”产品/架构层，核心不是新建另一套投资数据，而是把现有 portfolio、company portraits、notification prefs、cron jobs 和 digest thesis 串成一个可恢复的初始化与缺口修复流程。

建议包含四个对象：

1. `InvestmentContextStatus`
   按 actor 聚合当前 readiness：portfolio 数量、watchlist 数量、profile 覆盖率、thesis 蒸馏状态、prefs 状态、digest slots、quiet hours、cron 任务、最近一次上下文变更时间。

2. `InvestmentContextGap`
   可执行缺口项，例如 `missing_portfolio`、`missing_profile`、`profile_unrecognized_ticker`、`thesis_distill_skipped`、`notification_disabled`、`no_digest_slot`、`no_first_briefing_task`。

3. `IntakeDraft`
   用户在 public/chat/admin/desktop 任一入口补充的信息先进入草稿：持仓、关注、长期偏好、事件敏感度、勿扰时段、想跟踪的公司、是否允许建立画像、是否创建首个 scheduled task。

4. `ApplyPlan`
   用户确认后由现有工具和存储完成写入：`portfolio` upsert、`company_portrait` 建档或补 ticker/frontmatter、`NotificationPrefs` 更新、`cron_job` 添加首个任务，并触发一次 thesis distill refresh。

关键原则：

- 不绕过现有 actor 隔离，所有状态仍按 `ActorIdentity` 归属。
- 不把 company portrait 变成 UI 直接编辑器，画像创建和更新仍由 agent-mediated file operation 完成。
- 不把 gap resolver 做成自动买卖建议。它只帮助补上下文、偏好、监控和研究记忆。
- 第一版只做“一个 actor 的投资上下文”，不解决跨渠道身份合并。跨渠道共享应等待 linked workspace 提案落地。

## 用户体验变化

### Public 用户端

- `/portfolio` 的空态从“请去 chat 告诉 agent”升级为“开始配置投资上下文”。
- 用户可以通过一个短流程输入：
  - 持仓或关注标的，支持“我持有 AAPL、NVDA，关注 TSM”这种自然语言粘贴。
  - 长期/短期倾向、核心仓位、策略备注。
  - 希望收到什么：重大财报、SEC、价格异动、每日/盘前/盘后 digest。
  - quiet hours 和本地时区。
- 提交流程不立即静默写入全部状态，而是展示 apply plan：
  - 将新增哪些持仓/关注。
  - 哪些 ticker 会创建空画像骨架或触发 agent 建首版画像。
  - 哪些通知偏好会改变。
  - 是否创建首个“每日投资简报”或“持仓事件提醒”任务。
- 成功后 `/portfolio` 展示 readiness 卡片，例如“4/5 个持仓已有画像，2 个 thesis 已蒸馏，1 个标的需要补 profile frontmatter”。

### Admin 管理端

- `/users` 左侧或用户详情页增加 `Context readiness` 概览。
- 在用户 portfolio/profiles/thesis tabs 中突出缺口：
  - 持仓存在但没有画像。
  - 画像存在但 ticker/frontmatter 无法被蒸馏识别。
  - thesis distill skipped 的原因和建议操作。
  - 通知关闭、digest slots 为空、quiet hours 覆盖全部 digest slot。
- 管理员可以为单个 actor 发起 intake repair：
  - 仅生成修复建议。
  - 代用户应用非画像类结构化修复。
  - 对画像类修复生成一条 agent task 或 chat handoff，而不是 UI 直接改 profile 正文。

### 桌面端

- Desktop bundled/remote 模式启动后可以显示“当前连接用户/本地 actor 的投资上下文是否就绪”。
- 当本机首次配置成功但 portfolio 为空时，桌面端优先引导到 intake，而不是把用户直接丢到空 dashboard。
- Desktop 不需要独立存储；它只消费 backend capabilities 和同一组 status/gap API。

### 多渠道

- 在 Feishu/Telegram/Discord 私聊里，用户说“帮我开始监控我的持仓”或“我刚注册，怎么配置”时，agent 触发同一个 intake prompt contract。
- 群聊不自动写个人投资上下文。若用户在群里触发，应要求切换到私聊或明确 actor scope，保持当前 `ActorIdentity` / `SessionIdentity` 边界。
- 对 iMessage 保持谨慎：由于本地权限和默认关闭策略，第一版只复用私聊文本流程，不要求额外系统能力。

## 技术方案

### 后端 API

新增或扩展 web API，建议放在 `crates/hone-web-api/src/routes/investment_context.rs`：

- `GET /api/investment-context/status?channel=&user_id=&channel_scope=`
  - admin 端查询任意 actor。
  - 聚合 `PortfolioStorage`、company profile scan、`FilePrefsStorage`、cron jobs 和 thesis distill metadata。

- `GET /api/public/investment-context/status`
  - public 端从 `hone_web_session` 推导 web actor。
  - 返回同一 status/gaps，但只包含当前用户可见信息。

- `POST /api/public/investment-context/intake/preview`
  - 输入自然语言或结构化表单。
  - 输出 `IntakeDraft` 和 `ApplyPlan`，不写盘。
  - 第一版可以先要求结构化 JSON 表单，后续再接 agent 解析。

- `POST /api/public/investment-context/intake/apply`
  - 应用用户确认后的 plan。
  - 写 portfolio 与 notification prefs。
  - 画像类动作只创建 agent handoff/task 或触发受控 chat prompt，不直接在 API 写主画像正文。
  - 可选触发 thesis distill refresh。

Admin 端可增加同构接口：

- `POST /api/investment-context/intake/preview`
- `POST /api/investment-context/intake/apply`

### 数据模型

第一版尽量不引入新持久表。`InvestmentContextStatus` 可以完全由现有真相源派生：

- portfolio: `memory/src/portfolio.rs`
- company profile: actor sandbox `company_profiles/*/profile.md`
- thesis metadata: `NotificationPrefs.investment_theses`、`investment_global_style`、`last_thesis_distilled_at`、`thesis_distill_skipped`
- notification prefs: `NotificationPrefs`
- scheduled tasks: `CronJobStorage`

如果需要保存 intake 草稿，可放在 `data/runtime/intake_drafts/` 或 SQLite，TTL 7 天即可。草稿不是投资真相源，丢失后可重建。

建议 status 结构：

```json
{
  "actor": { "channel": "web", "user_id": "..." },
  "portfolio": { "holdings_count": 3, "watchlist_count": 2, "updated_at": "..." },
  "coverage": {
    "symbols": ["AAPL", "NVDA", "TSM"],
    "profiled_symbols": ["AAPL"],
    "thesis_symbols": ["AAPL"],
    "skipped_symbols": ["NVDA"]
  },
  "notifications": {
    "enabled": true,
    "portfolio_only": false,
    "digest_slots_count": 2,
    "quiet_hours": null
  },
  "automation": { "enabled_jobs_count": 1 },
  "gaps": [
    {
      "code": "missing_profile",
      "symbol": "NVDA",
      "severity": "warning",
      "action": "create_company_profile"
    }
  ]
}
```

### Agent 与 skill 契约

新增一个轻量 skill 或在现有 skills 中增加约定：

- `investment_context_intake` skill 只负责收集和确认上下文，不直接给交易建议。
- 可调用工具：`portfolio`、`cron_job`、必要时 `skill_tool(company_portrait)` 或 runner 原生文件操作。
- 强制两阶段：
  1. summarize draft and ask confirmation
  2. after confirmation, apply tools and report gaps
- 如果用户只给 ticker 而没有长期 thesis，先创建 portfolio/watchlist，不强行生成画像结论。
- 如果用户明确要求“帮我建立长期画像”，再进入 `company_portrait` workflow。

这个 skill 需要和现有 `company_portrait` 分工清晰：intake 处理投资上下文激活与缺口，company_portrait 处理单公司长期研究文档。

### 前端实现

Public 端：

- 在 `packages/app/src/pages/public-portfolio.tsx` 增加 readiness panel、gap list、intake modal。
- 在 `packages/app/src/lib/api.ts` 增加 status/preview/apply 客户端。
- 空态和 skipped thesis 区域提供直接行动入口。

Admin 端：

- 在 `packages/app/src/pages/users.tsx` 用户详情顶部增加 readiness summary。
- 在 `PortfolioDetail`、`CompanyProfileDetail`、`UserThesisView` 的相邻区域展示 gap badges。
- 在 `NotificationPreferencesCard` 中展示“当前设置是否会让 digest 永远不发”的可解释检查，例如 digest slots 全落在 quiet hours 内。

Desktop：

- 只需要在已有 backend capability negotiation 后显示同一 readiness summary。
- 不新增桌面专属 API。

### 兼容与迁移

- 不迁移现有 portfolio JSON。
- 不改变 `NotificationPrefs` 默认行为。文件缺失仍代表 default prefs。
- 不直接修改已存在画像正文，只识别缺口并通过 agent-mediated flow 修复。
- 对旧画像缺 frontmatter 的兼容读取规则保持不变，但 gap resolver 应提示“可读但不可稳定蒸馏”，引导补上 ticker frontmatter。

## 实施步骤

1. 定义派生 status/gap 类型
   - 在 web API 层先实现只读 status。
   - 覆盖 portfolio 空、profile 缺失、thesis skipped、prefs disabled、digest slot empty、quiet hours conflict。

2. 接入 public `/portfolio`
   - 展示 readiness panel 和 gaps。
   - 空态入口改为 intake modal。
   - 仍保留去 `/chat` 的路径。

3. 接入 admin `/users`
   - 用户详情页顶部展示 readiness。
   - portfolio/profile/thesis tab 展示对应 gap。
   - 支持 admin 复制一段“建议用户发送给 agent 的修复 prompt”。

4. 实现 preview/apply v1
   - 第一版使用结构化表单，不依赖 LLM 解析。
   - apply 只写 portfolio 和 prefs。
   - company profile 动作生成 chat prompt 或 task，不直接写正文。

5. 接入 agent intake skill
   - 支持多渠道自然语言触发。
   - 强制确认后写入。
   - 与 `company_portrait`、`scheduled_task` 保持工具边界。

6. 增加 thesis refresh 与任务建议
   - 用户确认后可手动触发一次 distill。
   - 推荐但不强制创建首个 daily/weekly briefing task。

7. 运营指标和灰度
   - 先仅对 web public 用户开启。
   - 再开放 admin repair。
   - 最后开放 IM 私聊触发。

## 验证方式

自动化验证：

- Rust unit tests:
  - status 派生：空 portfolio、仅 watchlist、holding 有/无 profile、profile 无 frontmatter、thesis skipped。
  - quiet hours 与 digest slot 冲突检测。
  - notification disabled / digest slots empty gap。
- Web API tests:
  - public status 必须只能返回当前 web session actor。
  - admin status 可查询任意 actor，但必须走已有 actor 参数校验。
  - preview 不写盘，apply 才写盘。
- Frontend tests:
  - public `/portfolio` 空态显示 intake 入口。
  - skipped thesis 显示具体 gap。
  - admin users 页在 actor 切换后刷新 readiness。

手工验收：

- 新建 web invite 用户，登录后 portfolio 为空，能完成 intake 并看到 holdings 出现在 `/portfolio`。
- 添加一个持仓但不建画像，status 显示 `missing_profile`。
- 添加一个 profile 但缺 ticker frontmatter，status 显示可读但蒸馏风险。
- 关闭 notification prefs，status 显示推送关闭。
- 设置 digest slots 全落入 quiet hours，status 给出冲突提示。
- 在 Feishu/Telegram/Discord 私聊发起 intake，确认前不写盘，确认后 portfolio 可在 admin 端看到。

成功指标：

- 新用户从登录到首个 portfolio 记录的转化率。
- portfolio 非空用户中 profile 覆盖率。
- thesis distill skipped 比例。
- 首个 digest 成功生成和投递的时间。
- 用户手动问“为什么我没收到推送/没有投资上下文”的次数下降。

## 风险与取舍

- 风险：intake 流程过重，反而拖慢首次体验。
  - 取舍：第一版只问最少字段，允许“只关注 ticker，不填成本”。

- 风险：自然语言解析持仓容易误写。
  - 取舍：所有写入必须经过 preview/confirmation；第一版优先结构化表单。

- 风险：画像骨架自动创建导致低质量空文档。
  - 取舍：默认只标记 gap，不自动生成长期画像结论；只有用户确认系统性研究时才进入 `company_portrait`。

- 风险：与 linked workspace 提案重叠。
  - 取舍：本提案只处理单 actor 的上下文完整度，不做跨渠道身份合并。

- 风险：与 entitlement ledger 的商业化字段耦合。
  - 取舍：本提案只输出 readiness 和 gaps，未来可被 entitlement 使用，但不引入计费逻辑。

- 风险：admin 代改用户上下文可能越权。
  - 取舍：沿用已有 admin API 权限边界；public API 必须从 session 推导 actor，不能接受任意 user_id。

## 与已有提案的差异

已检查：

- `docs/proposal/auto_p1_delivery_decision_loop.md`
- `docs/proposal/auto_p1_evidence_review_queue.md`
- `docs/proposal/auto_p1_linked-user-workspace.md`
- `docs/proposal/auto_p1_run_trace_workbench.md`
- `docs/proposal/auto_p1_usage_entitlement_ledger.md`
- `docs/proposals/desktop-bundled-runtime-startup-ux.md`
- `docs/proposals/skill-runtime-multi-agent-alignment.md`

差异结论：

- 与 `auto_p1_delivery_decision_loop.md` 不重复：该提案关注通知为什么发送、降级、digest 或跳过；本提案关注通知之前的投资上下文是否完整。
- 与 `auto_p1_evidence_review_queue.md` 不重复：该提案处理市场证据进入人工复盘再更新画像；本提案处理新用户或缺口用户如何建立 portfolio/profile/prefs/task 的初始上下文。
- 与 `auto_p1_linked-user-workspace.md` 不重复：该提案处理跨渠道 identity/workspace 归并；本提案明确限定单 actor，不解决身份合并。
- 与 `auto_p1_run_trace_workbench.md` 不重复：该提案处理 agent run 的排障证据；本提案处理投资产品上下文的 readiness 和 activation。
- 与 `auto_p1_usage_entitlement_ledger.md` 不重复：该提案处理试用、付费、成本和用量权益；本提案不计费，只补投资上下文。
- 与 `desktop-bundled-runtime-startup-ux.md` 不重复：该提案处理 desktop sidecar 启动冲突和接管体验；本提案只让 desktop 消费同一投资上下文 readiness。
- 与 `skill-runtime-multi-agent-alignment.md` 不重复：该提案处理 skill runtime 与 multi-agent 执行模型；本提案只新增一个可选的投资上下文 intake 契约，复用现有 skill/runtime 机制。

本轮选择该主题，是因为当前仓库已经具备 portfolio、company portrait、public portfolio、notification prefs、cron jobs 和 digest thesis 的基础设施，但缺少把这些能力转化为“用户首次可用”和“持续缺口修复”的产品层。它能直接提升激活、留存和推送质量，且可以分阶段落地，不要求重构核心 runner 或存储真相源。
