# Proposal: Investment Policy Guardrails for Enforceable User Discipline

status: proposed
priority: P1
created_at: 2026-05-22 02:02:56 CST
owner: automation

## related_files

- `README.md`
- `AGENTS.md`
- `docs/repo-map.md`
- `docs/invariants.md`
- `docs/decisions.md`
- `docs/current-plan.md`
- `docs/proposal/auto_p1_investment_context_intake.md`
- `docs/proposal/auto_p1_trade_discipline_journal.md`
- `docs/proposal/auto_p1_portfolio-exposure-radar.md`
- `docs/proposal/auto_p0_investment_output_safety_gate.md`
- `memory/src/portfolio.rs`
- `crates/hone-tools/src/portfolio_tool.rs`
- `skills/portfolio_management/SKILL.md`
- `skills/position_advice/SKILL.md`
- `skills/notification_preferences/SKILL.md`
- `crates/hone-channels/src/prompt.rs`
- `crates/hone-event-engine/src/prefs.rs`
- `crates/hone-event-engine/src/router/policy.rs`
- `crates/hone-event-engine/src/global_digest/mainline_distill.rs`
- `crates/hone-web-api/src/routes/portfolio.rs`
- `packages/app/src/context/portfolio.tsx`
- `packages/app/src/components/portfolio-detail.tsx`
- `packages/app/src/pages/public-portfolio.tsx`
- `packages/app/src/components/notification-preferences-card.tsx`

## 背景与现状

Hone 的产品承诺是帮助用户保持投资纪律，而不是提供一轮轮看似聪明的聊天答案。当前仓库已经具备多块纪律相关能力：

- `README.md` 明确把 Hone 定位为个人投资研究助理和投资纪律守护者。
- `crates/hone-channels/src/prompt.rs` 注入全局金融约束：禁止直接荐股，操作建议必须转为买点、卖点、触发条件、失效条件、仓位和风险分析。
- `memory/src/portfolio.rs` 与 `PortfolioTool` 支持 actor-scoped 持仓和关注列表，`Holding` 已有 `holding_horizon`、`strategy_notes`、`notes`、期权字段和 `tracking_only`。
- `skills/position_advice/SKILL.md` 已要求先读取 portfolio，再评估集中度、催化、流动性、下行情景和仓位调整问题。
- `NotificationPrefs` 已有 `portfolio_only`、`quiet_mode`、`large_position_weight_pct`、价格阈值、digest slots、`mainline_style`、`mainline_by_ticker` 等偏好字段，event-engine router 会按这些偏好调整送达和 severity。
- `global_digest/mainline_distill.rs` 会从 company portraits 蒸馏 per-ticker 投资主线和全局风格，public `/portfolio` 会展示这些长期主线。
- 现有 proposal 已覆盖投资上下文收集、单次交易纪律日志、组合暴露雷达和输出安全门禁。

这些能力共同说明：Hone 已经能保存“用户持有什么”、展示“每个标的的主线是什么”、提醒“哪些事件值得看”、并在单次回答里执行全局安全规则。但长期投资纪律仍然缺少一个一等真相源：用户自己的投资政策。

现在，纪律规则散落在 portfolio 的自由文本备注、notification prefs 的推送偏好、company portrait 的用户视角、prompt 的通用规则，以及 position advice 的临时输出里。系统没有一个稳定对象回答：

- 这个用户允许哪些资产类别、仓位上限、期权/杠杆边界和观察名单规则？
- 哪些行为是用户明确想避免的，例如追高、单一催化重仓、财报前加杠杆、亏损后补仓？
- 哪些情形必须触发冷静期、二次确认或只允许进入复盘，不应马上写入 portfolio 或创建强提醒？
- 当通知、digest、chat、portfolio 写入和 position advice 发生冲突时，应以哪套用户纪律为准？

## 问题或机会

这是 P1 级产品/架构机会。它不属于 P0 安全底线，但会显著提升 Hone 的核心差异化、用户信任、留存和付费感知。

1. **用户纪律没有可机读源头。**
   当前全局 prompt 只能提供通用金融边界，无法表达某个用户自己的投资政策。例如“单一股票成本权重不超过 25%”“期权只允许保险性保护仓”“不做财报前短线押注”“只关注美股和现金类资产”。

2. **偏好、主线和操作建议之间缺少仲裁层。**
   `NotificationPrefs` 管推送，portfolio 管持仓，company portrait 管公司主线，position advice 管一轮建议。它们都重要，但没有一个 policy verdict 说明本轮输出或状态变更是否符合用户长期纪律。

3. **新用户激活后仍容易回到自由聊天。**
   `investment_context_intake` 可以补齐数据，但补齐数据不等于建立纪律。用户填完持仓后，Hone 仍缺一张“以后所有分析和提醒都要遵守这些规则”的政策卡。

4. **单次交易日志缺少上游政策约束。**
   `trade_discipline_journal` 可以记录一次操作决策，但它需要一个判断基准：这次操作是违反用户已定政策，还是在政策允许范围内需要补证据？

5. **商业化叙事还停留在功能集合。**
   对投资助理来说，“帮你执行自己的投资政策”比“更多模型调用、更多提醒、更多图表”更容易形成高信任价值。它也能自然连接团队版、顾问协作、审计和高阶权益。

机会是新增 **Investment Policy Guardrails**：一个 actor-scoped、版本化、可解释、可被 agent 和 UI 共同引用的用户投资政策对象。它不替代 portfolio、company portrait、notification prefs 或 journal，而是给这些能力提供用户纪律基线。

## 方案概述

新增 `InvestmentPolicy` 产品层，保存用户自己确认过的投资纪律规则，并提供统一的 `PolicyVerdict` 给 chat、portfolio、notification、digest、position advice 和未来 journal 使用。

建议核心对象：

1. `InvestmentPolicy`
   actor-scoped 的当前生效政策，包含适用资产、仓位边界、期权/杠杆规则、时间跨度、允许/禁止行为、确认门槛、通知敏感度和复盘节奏。

2. `PolicyVersion`
   每次政策变更形成版本。旧版本只读保留，用于解释历史 journal 或历史通知为什么当时被允许/阻止。

3. `PolicyRule`
   可机读规则，例如 `max_single_position_cost_weight_pct`、`options_requires_explicit_expiry`、`no_new_position_before_earnings_days`、`cooldown_required_for_fomo_language`、`disallow_margin_or_leverage`。

4. `PolicyVerdict`
   面向一次用户意图、一次 portfolio mutation、一次 position advice、一次 notification upgrade 或一次 scheduled task 创建，输出 `allow`、`warn`、`require_confirmation`、`cooldown`、`block`、`not_applicable` 以及稳定 reason codes。

5. `PolicyEvidenceRef`
   verdict 可引用 portfolio、company portrait、mainline、exposure snapshot、recent messages、trade journal entry 或 user-confirmed policy version，避免模型自由发挥。

边界：

- 不接交易执行，不替用户做投资决策。
- 不把 policy 变成强制合规系统；用户可以修改或临时豁免，但豁免要有记录。
- 不把 company portrait 改成政策源。画像描述某家公司和用户视角，policy 描述用户跨标的的长期纪律。
- 不直接覆盖 `NotificationPrefs`。policy 可以生成建议和 guardrail，但通知偏好仍是送达配置真相源。

## 用户体验变化

### 用户端

- Public `/portfolio` 增加“投资政策”卡片，展示当前生效纪律：
  - 资产范围：美股、ETF、期权、现金、加密等。
  - 仓位上限：单一标的、主题、短线仓、期权风险预算。
  - 行为边界：是否允许追涨、财报前开仓、卖裸期权、杠杆、短线事件交易。
  - 冷静期：哪些语言或行为会触发延迟确认。
  - 复盘节奏：默认 T+7、T+30、财报后或 thesis 失效条件。
- 用户可以通过 guided flow 创建第一版 policy，也可以在 chat 中说“以后不要让我因为单日大涨追高”“期权提醒我必须写明到期和最大亏损”。
- 当用户提出操作意图时，Hone 先引用 policy：
  - “这符合你的长期核心仓规则，但超过单一标的 25% 的提醒阈值。”
  - “这触发你设置的财报前不开新仓规则，建议先进入冷静期或补充反方证据。”
  - “你没有设置期权政策，我只能按默认风险问题拆解，不能假定允许。”
- Policy 不应阻塞普通研究。用户问公司基本面、行业主线或财报解读时，只作为风格和边界上下文，不把页面变成合规表单。

### 管理端

- 用户详情页增加 `Investment policy` 区域：
  - 当前版本、最近更新时间、确认来源、待确认草稿。
  - 规则覆盖度：仓位、期权、杠杆、事件交易、通知、复盘。
  - 最近 policy verdict：哪些回答、portfolio 变更或任务创建触发了 warning / confirmation / cooldown。
- Admin 可代用户发起 policy draft，但不能静默生效；需要用户确认或明确标记为 operator-applied。
- 对运营和支持来说，policy 能解释“为什么这条提醒没有即时推”“为什么 agent 一直追问仓位上限”“为什么某个持仓改动要求确认”。

### 桌面端

- Desktop dashboard 可显示本地 actor 的 policy readiness：未设置、草稿待确认、已生效、规则过期。
- Bundled 模式保留本地隐私优势：policy 存在本机数据目录，可导出/备份。
- Remote 模式明确标注 policy 属于当前远端 backend 账号，避免用户误以为本地桌面 policy 会自动同步到云端。

### 多渠道

- IM 私聊支持轻量命令：
  - “查看我的投资政策”
  - “把单一标的上限设为 20%”
  - “期权必须有到期日、最大亏损和退出条件”
- 群聊不展示个人 policy 细节；如果用户在群里触发敏感操作，应引导私聊。
- 定时任务和 event-engine 可在用户态文案中简短解释 policy reason，例如“按你的少打扰+长期仓政策，本条进入 digest”。

## 技术方案

### 1. 存储与类型

建议在 `memory` 新增 `investment_policy` 模块，第一版使用每 actor 一个 JSON 文件，后续再迁入 SQLite：

```text
data/investment_policies/<actor_storage_key>.json
```

建议类型：

```rust
pub struct InvestmentPolicy {
    pub actor: ActorIdentity,
    pub active_version_id: String,
    pub status: PolicyStatus,
    pub rules: Vec<PolicyRule>,
    pub default_review_cadence: Option<ReviewCadence>,
    pub created_at: String,
    pub updated_at: String,
}

pub struct PolicyRule {
    pub rule_id: String,
    pub category: PolicyRuleCategory,
    pub severity: PolicyRuleSeverity,
    pub scope: PolicyRuleScope,
    pub params: serde_json::Value,
    pub user_text: String,
    pub enabled: bool,
}

pub struct PolicyVerdict {
    pub verdict: PolicyVerdictKind,
    pub reason_codes: Vec<String>,
    pub user_summary: String,
    pub evidence_refs: Vec<PolicyEvidenceRef>,
    pub requires_confirmation: bool,
    pub cooldown_until: Option<String>,
}
```

第一批规则保持确定性，避免 LLM 任意解释：

- `max_single_position_cost_weight_pct`
- `max_theme_or_strategy_cluster_pct`
- `disallow_margin_or_leverage`
- `options_require_expiration_and_max_loss`
- `no_new_position_before_earnings_days`
- `require_exit_rule_for_short_term_trade`
- `cooldown_on_fomo_or_panic_language`
- `large_position_requires_second_confirmation`
- `watchlist_no_position_sizing_advice`

### 2. Policy evaluator

新增纯逻辑 evaluator，可放在 `memory` 或 `hone-core` 类型层加 `hone-web-api` 服务层。输入是 `PolicyCheckSubject`：

```rust
pub enum PolicyCheckSubject {
    ChatIntent(ChatIntentSummary),
    PortfolioMutation(PortfolioMutationPreview),
    ScheduledTaskDraft(ScheduledTaskDraftSummary),
    NotificationDecision(NotificationDecisionSummary),
    PositionAdviceRequest(PositionAdviceContext),
}
```

第一版只对 `PortfolioMutation` 和 `PositionAdviceRequest` enforce，其它路径先 observe：

- `PortfolioMutation`：创建/更新持仓前检查 shares、avg_cost、asset_type、期权字段、仓位阈值和 watchlist 状态。
- `PositionAdviceRequest`：输出前把 policy summary 注入当前 turn，不直接改最终文案。
- `ScheduledTaskDraft`：如果用户要创建高频价格/财报前押注型任务，给出 policy warning。
- `NotificationDecision`：先只记录 reason，未来再参与 router 降级或解释。

### 3. API 与 UI

新增 Web API：

- `GET /api/investment-policy?channel=&user_id=&channel_scope=`
- `POST /api/investment-policy/draft`
- `POST /api/investment-policy/confirm`
- `POST /api/investment-policy/check`
- `GET /api/public/investment-policy`
- `POST /api/public/investment-policy/draft`
- `POST /api/public/investment-policy/confirm`

UI 落点：

- `packages/app/src/pages/public-portfolio.tsx`：在投资主线和持仓列表之间展示 policy card。
- `packages/app/src/components/portfolio-detail.tsx`：保存持仓前展示 policy preview warning。
- `packages/app/src/components/notification-preferences-card.tsx`：说明哪些通知设置与 policy guardrails 有关联，但不混为同一个配置。
- `packages/app/src/pages/users.tsx`：用户详情页增加 policy tab 或 section。

### 4. Agent 与 skill 集成

新增 `investment_policy` tool 或 skill：

- `policy(action="view")`
- `policy(action="draft_rule", ...)`
- `policy(action="check", subject=...)`
- `policy(action="confirm", draft_id=...)`

`position_advice` skill 应在读取 portfolio 后读取 policy，并将 verdict 纳入输出结构：

1. 当前政策摘要。
2. 本次请求触发的 policy checks。
3. 需要补充的问题。
4. 非指令式风险管理建议。

`portfolio_management` skill 在写入前使用 policy preview：

- 普通关注标的：通常 allow。
- 新增真实持仓但缺 shares/cost：保持现有 watchlist 逻辑。
- 期权缺到期日或最大亏损：warn 或 require_confirmation。
- 单一标的明显超过用户上限：require_confirmation 或 cooldown。

### 5. 与 event-engine 的关系

第一版不要让 policy 直接阻断所有通知，否则容易引入误拦截。建议先提供只读 hints：

- `policy_digest_style`: digest 中突出用户关心的纪律项。
- `policy_immediate_guard`: 某些通知触发“需要用户复核”，但不自动给买卖指令。
- `policy_suppression_reason`: 对明显违反少打扰/非持仓/非政策范围的事件，只在 digest 或日志里解释。

后续可以与 `Delivery Decision Loop`、`Portfolio Exposure Radar`、`Investment Output Safety Gate` 联动，但本提案第一阶段只做 policy source 和 verdict。

## 实施步骤

### Phase 1: Policy model and read-only surface

- 在 `memory` 新增 `investment_policy` JSON storage、类型和 roundtrip 单测。
- 新增 Web API 的 get/draft/confirm 基础接口。
- Public `/portfolio` 和 admin 用户详情显示 policy readiness。
- 提供默认空态：尚未设置政策时，不影响聊天和 portfolio。

### Phase 2: Portfolio and position-advice preview

- `PortfolioTool` 和 admin `PortfolioDetail` 写入前生成 policy preview。
- `position_advice` skill 读取 policy 并输出 policy check section。
- 对高风险规则先 `warn` / `require_confirmation`，不直接 block。
- 增加回归测试：watchlist、普通股票、期权缺字段、超过仓位阈值、禁用杠杆。

### Phase 3: Chat draft flow and multi-channel commands

- 增加 `investment_policy` skill，把自然语言映射为 policy draft。
- Public chat/IM 私聊支持查看和修改 policy 草稿。
- 用户确认后生成新 `PolicyVersion`，旧版本保留。
- 群聊敏感场景引导私聊。

### Phase 4: Event-engine hints and journal integration

- 给 event-engine router/digest 增加 policy hints，只做解释和降级建议。
- `trade_discipline_journal` 创建草稿时引用 active policy version。
- `portfolio_exposure_radar` 可使用 policy 阈值解释 concentration flags。
- 管理端展示最近 policy verdict 和 false-positive/override 反馈。

## 验证方式

- 单元测试：
  - policy JSON roundtrip、actor isolation、version append-only。
  - evaluator 对每类初始规则返回稳定 reason code。
  - disabled rule 不参与 verdict。
- API 测试：
  - public API 只能访问当前 web actor。
  - admin API 必须显式 actor 参数。
  - draft 不生效，confirm 后 active version 更新。
- 前端测试：
  - policy empty state、draft preview、confirmation state、portfolio warning banner。
  - mobile viewport 下 policy card 不挤压 public portfolio 主线。
- 回归脚本：
  - 使用 fixtures 覆盖新用户无 policy、普通股票、期权缺到期、超过单标的阈值、禁用杠杆。
- 手工验收：
  - 在 public chat 设置一条政策，回到 `/portfolio` 可见。
  - 在 admin 端新增超过阈值的持仓，看到 warning 且可取消。
  - 在 IM 私聊查询 policy，只返回摘要，不泄漏完整 portfolio。
- 指标：
  - policy 设置率。
  - portfolio mutation 中 policy warning 的确认/取消比例。
  - position advice 中触发 policy check 的比例。
  - 因 policy 引导进入 trade journal 或 follow-up 的比例。

## 风险与取舍

- **风险：用户把 policy 当成合规或投资保证。**
  取舍：所有文案强调“执行你自己确认的纪律”，不是收益承诺或交易系统。

- **风险：规则太多导致新用户负担重。**
  取舍：第一版只提供 5 到 8 个高价值规则，允许以后逐步补齐；空 policy 不阻塞使用。

- **风险：LLM 把自然语言 policy 解释过度。**
  取舍：自然语言只能生成 draft，生效规则必须落成结构化 `PolicyRule` 并由用户确认。

- **风险：与 notification prefs、portfolio notes、company portraits 重叠。**
  取舍：policy 只保存跨标的、跨渠道、长期生效的纪律规则；单标的 thesis 留在 company portrait，送达配置留在 prefs，持仓备注留在 portfolio。

- **风险：enforce 太早伤害体验。**
  取舍：v1 以 warning 和 confirmation 为主，只对极少数用户明确禁用的行为 block。

- **风险：历史行为解释复杂。**
  取舍：保留 `PolicyVersion`，journal 和 verdict 引用版本号，避免用当前政策回头解释旧决策。

## 与已有提案的差异

本轮查重范围包含 `docs/proposal/` 与 `docs/proposals/` 下全部现有提案，并重点核对了以下相邻主题：

- 不重复 `auto_p1_investment_context_intake.md`：该提案解决“如何收集 portfolio、画像、通知偏好和首个任务”；本提案解决“收集后哪些长期纪律规则应成为可机读约束”。
- 不重复 `auto_p1_trade_discipline_journal.md`：该提案记录单次操作意图、检查和复盘；本提案提供 journal 判断一次操作是否违反用户长期政策的上游基线。
- 不重复 `auto_p1_portfolio-exposure-radar.md`：该提案生成组合风险快照；本提案保存用户自己确认的风险边界，并可被 exposure radar 用来解释阈值。
- 不重复 `auto_p0_investment_output_safety_gate.md`：该提案是系统级输出安全门禁；本提案是用户级投资纪律偏好，不替代全局安全边界。
- 不重复 `auto_p1_notification-control`、`delivery_decision_loop`、`end-user-notification-control` 类提案：这些处理消息送达和偏好；本提案只在通知决策中提供 policy hints，不直接承担 delivery router。
- 不重复 `auto_p1_agent-mutation-ledger.md`：mutation ledger 记录状态变更确认和撤销；本提案定义投资政策本身和 policy verdict，后续可把 policy 修改作为 mutation ledger 的一种事件。

查重结论：现有 proposal 覆盖了数据收集、运行安全、通知控制、组合风险和单次交易复盘，但还没有覆盖“用户长期投资政策作为跨 chat / portfolio / notification / journal 的可版本化纪律源头”。因此本主题是新的、可落地的 P1 产品/架构提案。
