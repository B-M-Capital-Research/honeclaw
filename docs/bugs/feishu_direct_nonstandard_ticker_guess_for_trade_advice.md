# Bug: Feishu 直聊对非标准 ticker 拼写直接猜测实体并给出建仓建议

- 发现时间：2026-06-04 03:02 CST
- Bug Type：Business Error
- 严重等级：P3
- 状态：New
- GitHub Issue：无，非 P1

## 证据来源

### 2026-07-07 19:03 CST 运行态回退

- `data/sessions.sqlite3`
  - 时间窗：2026-07-07 15:00 到 19:03 CST。
  - 本窗共有 6 个 user turn 与 6 条 assistant final，Feishu / Web direct 与 1 条普通 scheduler 均以 assistant 收口。
  - assistant final 污染扫描未命中空回复、`reasoning_content`、`<think>`、本机绝对路径、`company_profiles/`、raw tool 字段、provider 原始错误、`data_fetch`、`quote_short`、`stock_research`、`mcpServers`、`Param Incorrect`、panic 或资源耗尽。
- 真实会话：`session_id=Actor_web__direct__web-user-5bb05078acd4`
  - `2026-07-07T18:10:10.265721+08:00` 用户输入摘要：`CBRS`。
  - `2026-07-07T18:10:33.033094+08:00` assistant 回复摘要：明确说 `CBRS 这个代码我不能直接确认唯一标的`，要求用户补充是美股 ticker、基金、通信频谱主题或券商页面具体股票名称。
  - `2026-07-07T18:36:11.937355+08:00` 用户追问摘要：`美股CBRS要加仓还是减仓？`。
  - `2026-07-07T18:37:12.155036+08:00` assistant 回复摘要：改按 `美股 Cerebras Systems` 分析，虽然写明“本轮未稳定核到 7 月 7 日实时价”，仍给出“不建议现在加仓”“更适合减仓或至少不再加”的明确交易动作建议，并引用 6 月下旬 / 6 月 24 日旧价口径。
- `cron_job_runs`
  - 同窗普通 scheduler 1 条为 `completed + sent + delivered=1`；heartbeat 异常另归入 heartbeat 结构化缺陷。
- 最近四小时无非文档代码提交。

#### 本轮判断

- 这是既有“非标准 / 高歧义 ticker + 交易动作建议未先确认”的同根复发；虽然原始样本来自 Feishu direct，本轮样本发生在 Web direct，但两者共享金融 prompt / multi-agent search-stage 交易建议边界。
- 本轮 reply 主体正常收口，未见错投、空回复、系统失败或原始工具输出外泄；问题主要是实体确认不足和强时效价格不足时仍给加仓 / 减仓建议，因此仍为质量性 `P3`，而不是 `P1/P2`。
- 状态从代码级 `Fixed` 回退为运行态 `New`；非 P1，不创建 GitHub Issue。

- `data/sessions.sqlite3`
  - 时间窗：2026-06-03 23:02 到 2026-06-04 03:02 CST。
  - 本窗共有 14 个 Feishu user turn 与 14 个 assistant final，均成对收口。
  - assistant final 污染扫描未命中空回复、`hone-mcp binary not found`、本机绝对路径、`data/agent-sandboxes`、`rawOutput`、`tool_call`、`assistant.tool_calls`、`session/update`、`reasoning_content`、`<think>`、provider 原始错误、`HTTP 400`、`Param Incorrect`、`Resource temporarily unavailable`、`quota exhausted`、panic、`index out of bounds` 或 `company_profiles/...`。
- `data/runtime/logs/acp-events.log`
  - 同窗 Feishu / Web direct prompt 均有对应 `stopReason=end_turn`，未见 `stream disconnected before completion`、runner no-final 或通用失败收口。
- 真实会话：`session_id=Actor_feishu__direct__ou_5f680322a6dcbc688a7db633545beae42c`
  - `2026-06-03T23:30:36.122592+08:00` 用户输入摘要：`MFST的建仓建议`。
  - `2026-06-03T23:31:40.344069+08:00` assistant 回复摘要：先写 `若你说的“MFST”是 Microsoft（MSFT）`，随后直接给出 `现在 427 美元附近可以开始建首仓`、计划仓位比例、买入区间、Bull / Bear 与证伪条件。
  - `2026-06-03T23:42:24.591853+08:00` 用户输入摘要：`MPVL可以加仓吗`。
  - `2026-06-03T23:43:37.176739+08:00` assistant 回复摘要：先写 `“MPVL”我未查到常见美股代码，以下按你大概率想问的 MRVL（Marvell）回答`，随后直接给出 `现在不适合明显加仓`、仓位比例、价格区间和证伪条件。
- 最近四小时无非文档代码提交。
- `cron_job_runs.max(executed_at)` 仍停在 `2026-06-01T00:26:00.908925+08:00`，本轮没有新的 scheduler 执行记录；该观测沿用既有 scheduler / actor scope 缺陷，不作为本单新根因。

## 端到端链路

1. Feishu direct 用户用非标准 ticker 拼写询问建仓或加仓建议。
2. 系统识别出输入不是常见美股代码，或只能推测一个相近 ticker。
3. assistant 没有先短句澄清目标实体，而是直接按相近实体展开完整操作建议。
4. 用户若原本不是想问该实体，会收到答非所问的交易分析，并可能基于错误实体继续决策。

## 期望效果

- 当用户输入 `MFST`、`MPVL` 这类非标准 ticker 或明显拼写疑似错误的代码时，系统应先要求用户确认目标实体。
- 若只能高概率猜测一个实体，也只能用一句话澄清，例如“你是指 MSFT / Microsoft 吗？确认后我再给建仓建议。”
- 涉及建仓、加仓、减仓、买点和仓位比例时，实体确认门槛应高于普通科普问答，不能在未确认实体下给完整操作建议。

## 当前实现效果

- assistant 已意识到 `MFST` / `MPVL` 不是标准或常见 ticker，但仍直接按 `MSFT` / `MRVL` 输出完整建仓或加仓建议。
- 两条回复都包含具体价格区间、仓位比例和操作节奏，属于会影响用户投资动作的回答。
- 本轮没有用户后续纠正，因此不能证明实际误导已发生；但链路确实越过了“模糊实体先确认”的安全边界。

## 用户影响

- 这是质量性 bug，不是功能性 bug。
- 本轮 Feishu direct 消息正常投递、assistant final 正常收口，工具调用和日志没有显示系统失败、错投、空回复、重复回复或内部错误外泄。
- 问题主要是实体确认不足导致潜在答非所问和错误投资建议风险；它影响回答质量与可信度，但不阻断功能链路，因此定级为 `P3`，而不是 `P1/P2`。

## 根因判断

- 金融领域 prompt 已有“实体歧义约束”，但模型对“非标准 ticker 拼写 + 单一高概率相近实体”的场景仍倾向直接猜测并作答。
- 历史归档 `docs/bugs/archive/feishu_ambiguous_lite_entity_guessed_as_litecoin.md` 覆盖的是 `lite` 在股票 / 加密资产之间直接猜错实体；本轮是相近 ticker 拼写疑似错误时直接给交易建议，受影响链路相邻但样本与风险形态不同。
- 初步判断需要把“非标准 ticker / 拼写疑似错误 + 操作建议”纳入更严格的澄清门槛，而不是只处理多资产歧义。

## 下一步建议

- 在金融系统 prompt 或实体解析前置逻辑中补充约束：未识别 ticker、拼写疑似错误、编辑距离相近但未唯一确认时，涉及交易动作的问题必须先澄清。
- 对 `MFST -> MSFT`、`MPVL -> MRVL` 这类样本增加回归或 prompt fixture：最终回复不得给仓位比例、买入区间或直接建仓建议，只能请求确认。
- 后续巡检继续关注用户是否因同类猜测而纠正实体；若出现已确认答错且影响交易动作，可按影响范围重新评估严重等级。

## 修复记录

- 2026-06-05 03:03 CST 已修复：
  - `crates/hone-channels/src/prompt.rs` 的金融系统 prompt 新增“非标准 ticker 约束”：当输入是疑似拼写错误、少字母/多字母或并非常见证券代码的 ticker，且问题涉及建仓、加仓、减仓、买点、卖点、止损、仓位等交易动作时，必须先确认具体标的，禁止按“最像的代码”直接给出价格区间、仓位比例或交易建议。
  - `crates/hone-channels/src/runners/multi_agent.rs` 的 search-stage guidance 同步加入相同护栏，要求遇到 non-standard / near-match ticker 时先发一条澄清问题，不得在确认前输出 price targets、position sizing 或 trade advice。
  - 这次修复不依赖新增工具或运行态重启，直接收紧所有共享 prompt / multi-agent 搜索阶段的行为边界。
- 验证：
  - `cargo test -p hone-channels build_prompt_bundle_always_includes_finance_domain_policy --lib -- --nocapture`
  - `cargo test -p hone-channels search_input_guidance_allows_direct_replies_for_greetings --lib -- --nocapture`
  - `cargo check -p hone-channels --tests`
- 文档同步：
  - 已同步 `docs/bugs/README.md` 活跃计数、状态与修复表。
  - 本修复只收紧共享 prompt 与 search-stage 行为约束，不改变模块边界、长期约束或运行工作流，无需更新 `docs/repo-map.md`、`docs/invariants.md` 或新增 handoff。
