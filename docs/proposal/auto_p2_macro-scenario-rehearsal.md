# Proposal: Macro Scenario Rehearsal for Portfolio and Thesis Stress Tests

status: proposed
priority: P2
created_at: 2026-05-18 14:04:33 +0800
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
- `docs/proposal/auto_p1_portfolio-exposure-radar.md`
- `docs/proposal/auto_p1_cross-company-thesis-map.md`
- `docs/proposal/auto_p1_evidence_review_queue.md`
- `docs/proposal/auto_p1_trade_discipline_journal.md`
- `docs/proposal/auto_p2_investment-committee-mode.md`
- `memory/src/portfolio.rs`
- `memory/src/company_profile/types.rs`
- `memory/src/company_profile/storage.rs`
- `crates/hone-channels/src/prompt.rs`
- `crates/hone-tools/src/portfolio_tool.rs`
- `crates/hone-event-engine/src/global_digest/mainline_distill.rs`
- `crates/hone-event-engine/src/subscription.rs`
- `crates/hone-event-engine/src/prefs.rs`
- `crates/hone-web-api/src/routes/portfolio.rs`
- `crates/hone-web-api/src/routes/public_digest.rs`
- `packages/app/src/pages/public-portfolio.tsx`
- `packages/app/src/components/user-mainline-view.tsx`
- `packages/app/src/components/portfolio-detail.tsx`
- `packages/app/src/pages/users.tsx`
- `skills/company_portrait/SKILL.md`
- `skills/position_advice/SKILL.md`

## 背景与现状

Honeclaw 已经具备做“投资情景排练”的基础，但这些能力目前分散在不同层：

- `memory/src/portfolio.rs` 以 `ActorIdentity` 保存真实持仓、watchlist、股票/期权、成本、持有期限和策略备注。
- 公司画像以 actor sandbox 下的 `company_profiles/<profile_id>/profile.md` 与 `events/*.md` 保存长期 thesis、风险、证伪条件和事件变化；`memory/src/company_profile/types.rs` 已有 `TrackingConfig`、`ProfileMetadata`、`ProfileEventMetadata` 等结构。
- `crates/hone-event-engine/src/global_digest/mainline_distill.rs` 会从公司画像蒸馏 per-ticker thesis 与整体投资风格，供 digest 个性化使用。
- `crates/hone-event-engine/src/subscription.rs` 从 portfolio 构建事件订阅池，`prefs.rs` 保存 digest、quiet hours、large position 等偏好。
- Public `/portfolio` 和 admin 用户详情页已经能展示持仓、画像、主线蒸馏和通知上下文。
- `crates/hone-channels/src/prompt.rs` 强调宏观/行业叙事必须区分主线与噪音，不能因短期价格或单条新闻频繁切换判断。

现有提案也覆盖了几块相邻基础：`Portfolio Exposure Radar` 关注组合暴露、质量缺口和 guardrail；`Cross-Company Thesis Map` 关注同类公司叙事一致性；`Evidence Review Queue` 关注事件后是否更新画像；`Trade Discipline Journal` 关注用户操作前后的纪律记录；`Investment Committee Mode` 关注多视角研究审查。

但 Hone 还缺一个面向用户决策前的“情景排练”产品层：在重大宏观变量、行业周期或政策假设发生变化前，用户无法把自己的 portfolio、公司画像、证伪条件和通知策略放进一个假设场景里预演。当前系统更擅长“事件发生后提醒和复盘”，还不够擅长“事前知道哪些假设最脆弱、哪些证据出现时应该重新审查”。

## 问题或机会

这是 P2 级机会。它不属于当前核心可用性或安全门禁，但能显著增强 Hone 作为投资纪律助手的差异化：不只回答“现在发生了什么”，也帮助用户提前定义“如果世界变成另一种样子，我的组合和 thesis 哪些地方最该被重新检查”。

主要缺口：

1. **宏观风险和持仓资产没有可执行连接。**  
   用户可以问宏观问题，也可以看 portfolio，但系统没有稳定对象把“利率上行、AI capex 放缓、油价冲击、美元流动性收缩、监管收紧”等假设映射到具体持仓、画像证伪条件和通知敏感度。

2. **现有 guardrail 更偏静态，不是场景化。**  
   `Portfolio Exposure Radar` 适合展示集中度、数据质量和到期窗口；但用户真正需要的是“如果半导体需求下修 20%，哪些 thesis 会先失效，哪些公司画像需要复盘，哪些事件应升级为 immediate”。

3. **公司画像里的证伪条件没有被反向利用。**  
   画像要求保存风险台账和 disconfirming conditions，但这些条件目前主要供阅读和后续分析参考。系统还没有把它们抽成 scenario trigger，用来预演不同宏观或行业路径。

4. **用户容易在事件发生后才调整纪律。**  
   主动通知和 evidence queue 可以捕获事后证据；但当用户已经在高波动中收到消息时，情绪成本更高。事前 rehearsal 可以把“该看什么、什么才算主线变化”提前写清。

5. **管理端缺少高价值用户的情景覆盖视图。**  
   运维或投研协助者能看到用户持仓和画像，但很难判断“这个用户有没有为核心风险准备情景检查表，哪些持仓完全没有 stress trigger”。

## 方案概述

新增 **Macro Scenario Rehearsal**：actor-scoped 的情景排练层，用一组可保存、可复跑、可转化为提醒/证据队列的 scenario，把 portfolio、company portraits、mainline distill 和 notification preferences 串起来。

核心对象：

- `ScenarioTemplate`：标准情景模板，例如 `higher_for_longer_rates`、`ai_capex_slowdown`、`oil_supply_shock`、`consumer_demand_crack`、`usd_liquidity_tightening`、`china_policy_shift`。模板只描述变量和检查维度，不内置买卖建议。
- `ScenarioRehearsal`：一次用户/管理员发起的排练，包含 actor、scenario、输入假设、覆盖 ticker、使用的画像版本、生成时间和状态。
- `ScenarioImpactMap`：按 ticker / factor / thesis condition 输出的影响矩阵：受影响理由、关联画像段落、需要观察的证据、置信度、数据缺口。
- `ScenarioTrigger`：从排练中产出的观察条件，例如“如果 NVDA capex commentary 连续两次下修，则复盘 AI capex thesis”，可转为 scheduled task、evidence review seed 或 notification preference hint。
- `ScenarioPlaybook`：用户可读的行动框架，限定为研究动作和风险复核动作，例如补画像、设置复盘、降低通知噪音、要求反方研究，不输出直接买卖指令。

第一版目标是“保存和复跑结构化情景”，不是做量化风险模型。所有输出必须保持 Hone 的金融约束：不推荐具体买卖，不替用户决策，只帮助用户提前定义证据、风险和复盘动作。

## 用户体验变化

### 用户端

- Public `/portfolio` 增加 `Scenario rehearsal` 区块：
  - 展示已保存情景、最近一次排练时间、覆盖持仓数、open triggers 数量。
  - 提供 3-5 个默认模板和一个自定义情景输入框。
- 用户选择 “AI capex slowdown” 后，Hone 展示 preview：
  - 将检查哪些持仓和 watchlist。
  - 将读取哪些公司画像和主线蒸馏。
  - 哪些标的缺画像或缺证伪条件，导致结果不可靠。
- 生成结果不是长篇宏观作文，而是矩阵：
  - `最敏感假设`
  - `可能被证伪的画像条件`
  - `需要关注的事实证据`
  - `建议创建的复盘提醒`
  - `当前数据缺口`
- 用户可以把某个 trigger 转成提醒或 evidence queue seed，例如“下次财报后复盘这条假设”。

### 管理端

- `/users/:actor/mainline` 或用户详情页新增 `Scenarios` tab。
- 管理员可以查看：
  - 用户已有 scenario 覆盖率。
  - 哪些持仓没有任何 stress trigger。
  - 哪些画像缺少风险/证伪条件，导致 rehearsal 只能给出低置信度结果。
  - 最近哪些情景被复跑后结论变化最大。
- 管理员可以为高价值用户创建模板化 scenario，但不能直接改用户 portfolio 或画像正文；画像改写仍走 agent-mediated workflow。

### 桌面端

- Desktop bundled 模式可把 scenario rehearsal 做成本地高隐私工作流：读取本地 portfolio JSON 和 actor sandbox 画像，结果保存在本机数据目录。
- Dashboard 显示轻量 badge，例如 `2 个情景待复跑`、`3 个 trigger 等待财报后复盘`。
- Remote mode 明确显示结果来自远端 backend，不在本机重复扫描远端数据。

### 多渠道

- Feishu / Telegram / Discord 私聊支持轻量触发：
  - “帮我排练一下油价冲击对我的组合影响”
  - “如果 AI capex 降速，我该重点复盘哪些持仓”
- IM 回复只返回 top 3 影响和一个 scenario id；完整矩阵引导到 Web/desktop 查看。
- 群聊中不暴露个人持仓和成本，沿用群聊隐私约束，引导到私聊或 Web 处理。

## 技术方案

### 1. 场景模板与实例存储

建议在 `memory` 新增 `scenario_rehearsal` 模块，使用 SQLite 保存元数据和 JSON 结果；模板可以先放在 repo 内 `scenario_templates/` 或作为 Rust 常量，后续再开放管理端配置。

建议表：

```text
scenario_rehearsals (
  rehearsal_id TEXT PRIMARY KEY,
  actor_channel TEXT NOT NULL,
  actor_user_id TEXT NOT NULL,
  actor_scope TEXT,
  template_id TEXT NOT NULL,
  title TEXT NOT NULL,
  status TEXT NOT NULL,
  input_json TEXT NOT NULL,
  impact_json TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  source_session_id TEXT
)

scenario_triggers (
  trigger_id TEXT PRIMARY KEY,
  rehearsal_id TEXT NOT NULL,
  actor_channel TEXT NOT NULL,
  actor_user_id TEXT NOT NULL,
  actor_scope TEXT,
  symbol TEXT,
  condition TEXT NOT NULL,
  suggested_route TEXT NOT NULL,
  status TEXT NOT NULL,
  created_at TEXT NOT NULL,
  converted_target_id TEXT
)
```

第一版只保存结构化结果和用户确认的 trigger，不迁移 portfolio 或 company profile。

### 2. 输入聚合

Rehearsal evaluator 读取：

- `PortfolioStorage::load(actor)`：真实持仓、watchlist、期权字段、horizon、strategy notes。
- Company profile storage：相关 `profile.md` 和 `events/*.md`，优先提取风险、证伪条件、估值框架、关键经营指标。
- `NotificationPrefs` / mainline distill：per-ticker thesis、global style、digest slots、large position hints。
- 可选 market/event context：只在用户明确要求“基于当前最新事实复跑”时调用 agent/tool；默认 rehearsal 可以离线使用已存资料。

输入聚合必须把缺口显式输出，例如缺画像、缺 ticker、缺证伪条件、portfolio 成本不可计算、watchlist-only。

### 3. 评估方式

v1 采用“确定性框架 + agent synthesis”的混合模型：

1. 确定性候选：按 portfolio symbols 找画像，按模板 factor 找相关 thesis/risk 关键词，按持仓/关注/期权窗口标记重要性。
2. Agent synthesis：把候选输入给 runner，要求输出严格 JSON：
   - impacted_symbols
   - thesis_conditions_to_watch
   - evidence_needed
   - data_gaps
   - trigger_suggestions
   - user_facing_summary
3. Safety constraints：复用 `prompt.rs` 的金融约束，额外要求不输出买卖指令，不把 scenario 当作预测，不把旧数据伪装成最新事实。
4. Result validator：检查 JSON schema、symbol 是否来自输入、trigger 是否有可执行 condition、是否泄漏本地绝对路径。

### 4. API 与前端

新增 API：

- `GET /api/scenarios/templates`
- `GET /api/scenarios?actor=...`
- `POST /api/scenarios/rehearse`
- `GET /api/scenarios/:id`
- `POST /api/scenarios/:id/triggers/:trigger_id/convert`
- `POST /api/public/scenarios/rehearse`
- `GET /api/public/scenarios`

Public API 从 web session 推导 actor；admin API 可指定 actor。Convert 动作只创建草稿或调用现有受控路径：

- 转 scheduled task：生成 `scheduled_task` skill prompt 或 cron draft。
- 转 evidence review：创建 open review seed。
- 转 chat follow-up：打开 `/chat` 并预填 scenario context。

前端新增：

- `packages/app/src/pages/public-portfolio.tsx` 的 scenario section。
- `packages/app/src/components/scenario-rehearsal-panel.tsx`。
- admin 用户详情或 mainline 页的 scenario tab。
- 共享 model helpers 放在 `packages/app/src/lib/scenario-rehearsal.ts`，避免页面内拼业务规则。

### 5. 与现有系统边界

- 不替代 `Portfolio Exposure Radar`：exposure radar 回答“当前组合暴露和数据质量是什么”；scenario rehearsal 回答“在某个假设场景下，哪些 thesis 和证据要提前观察”。
- 不替代 `Investment Committee Mode`：committee 是一次研究审查的多视角输出；scenario rehearsal 是可保存、可复跑、可转 trigger 的情景对象。
- 不替代 `Evidence Review Queue`：evidence queue 处理已经出现或被标记的证据；scenario rehearsal 产出未来应关注的 evidence triggers。
- 不替代 `Trade Discipline Journal`：journal 记录用户具体操作意图和复盘；scenario rehearsal 只做事前情景检查，不记录交易行为。

## 实施步骤

### Phase 1: 模板和只读排练

- 定义 3 个内置模板：`higher_for_longer_rates`、`ai_capex_slowdown`、`oil_supply_shock`。
- 实现 actor 输入聚合：portfolio、company profiles、mainline distill 摘要、数据缺口。
- 新增 admin-only `POST /api/scenarios/rehearse`，返回未持久化 preview。
- 增加 schema validator 和单元测试，确保 symbol、trigger、data gap 输出可判定。

### Phase 2: 持久化和用户端入口

- 新增 `scenario_rehearsals` / `scenario_triggers` 存储。
- Public `/portfolio` 增加 scenario section，允许当前 web actor 创建和查看 rehearsal。
- Admin 用户详情增加 scenario tab。
- 将结果中的 trigger 转成 chat draft 或 scheduled task draft，不自动创建高副作用任务。

### Phase 3: 与事件/画像闭环

- 将 scenario trigger 与 `Evidence Review Queue` 串联：未来命中相关事件时标记为“来自某个 rehearsal 的观察条件”。
- 将 rehearsal 结果中的画像缺口转成 company portrait 更新建议，但仍由 agent 执行写入。
- 支持手动复跑并对比上次结果，记录“哪些假设变得更脆弱/更稳固”。

### Phase 4: 多渠道轻量触发

- 增加 IM 私聊触发语义和短结果格式。
- 群聊只提示切换私聊，不在群里展开个人组合。
- Desktop dashboard 增加 pending trigger / stale rehearsal badge。

## 验证方式

- 单元测试：
  - 模板解析和默认输入 schema。
  - portfolio + profile + mainline 的聚合逻辑。
  - 缺 portfolio、缺画像、watchlist-only、期权字段不完整时的 data gap 输出。
  - agent JSON validator 拒绝未知 symbol、空 trigger、买卖指令式 action。
- API 测试：
  - public route 只能访问当前 web actor。
  - admin route 可以指定 actor，但无效 actor 返回可解释错误。
  - rehearsal preview 不写盘，persisted rehearsal 才进入列表。
- 前端测试：
  - public `/portfolio` 空态、无画像、已有 rehearsal、trigger 转 draft 的状态模型。
  - admin scenario tab 的筛选和错误态。
- 手工验收：
  - 对一个包含真实持仓、watchlist、至少两个公司画像的 actor 运行 `ai_capex_slowdown`。
  - 确认输出引用的是画像中的风险/证伪条件，而不是泛泛宏观话术。
  - 确认结果只给研究动作和观察条件，不给买卖指令。
  - 确认 IM short mode 不泄露持仓明细到群聊。
- 指标：
  - rehearsal 创建数。
  - trigger 转化为复盘/任务/证据队列的比例。
  - 有画像但无证伪条件的持仓比例下降。
  - 用户在重大事件前已存在相关 scenario trigger 的比例。

## 风险与取舍

- **风险：用户把情景排练误解成预测。**  
  输出必须明确这是压力测试和观察清单，不是市场预测或交易建议。

- **风险：LLM 生成泛泛宏观作文。**  
  通过结构化输入、JSON schema、必须引用 portfolio/profile/mainline 条件、限制输出章节来降低空泛度。

- **风险：与 exposure radar / committee mode 边界混淆。**  
  第一版 UI 应只放在“场景”入口下，结果以 trigger 和 impact map 为核心，不展示为普通风险评分或投委会合议。

- **风险：触发过多提醒造成噪音。**  
  Trigger 转任务必须二次确认；默认只生成 chat draft 或 evidence seed，不直接启用 immediate notification。

- **风险：画像质量不足导致低置信度。**  
  这是产品机会：结果应把缺口显式暴露，引导补画像或补证伪条件，而不是假装有完整判断。

## 与已有提案的差异

- 不重复 `auto_p1_portfolio-exposure-radar.md`：该提案解决当前组合暴露、数据质量和静态 guardrail；本提案解决某个宏观/行业假设下的可保存、可复跑情景排练和 trigger 转化。
- 不重复 `auto_p1_cross-company-thesis-map.md`：该提案关注跨公司主线一致性；本提案关注外部场景如何冲击用户当前 portfolio 与画像证伪条件。
- 不重复 `auto_p1_evidence_review_queue.md`：该提案处理已经出现的证据项；本提案提前定义未来出现哪些证据才值得进入复盘。
- 不重复 `auto_p1_trade_discipline_journal.md`：该提案围绕用户具体操作意图和事后复盘；本提案不记录交易行为，只产出研究情景和观察条件。
- 不重复 `auto_p2_investment-committee-mode.md`：该提案是多角色研究审查模式；本提案是 actor-scoped 的 scenario 对象、impact map 和 trigger 生命周期。
- 不重复 `auto_p1_investment_playbook_launcher.md`：playbook launcher 是启动可重复 workflow；scenario rehearsal 是一个特定的投资风险排练产品面，可被 playbook 后续调用但不等同于 playbook 系统。

