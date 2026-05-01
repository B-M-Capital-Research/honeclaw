# Bug: Feishu 直聊非金融新话题仍误入 `stock_research` 并沿用旧 `LITE` 上下文

- **发现时间**: 2026-04-30 19:08 CST
- **Bug Type**: Business Error
- **严重等级**: P3
- **状态**: Fixed
- **证据来源**:
  - `data/sessions/Actor_feishu__direct__ou_5f62439dbed2b381c0023e70a381dbd768.json`
    - `2026-04-30T18:59:08.334712+08:00` 用户真实输入：`AMD的电脑CPU是什么名字`
    - `2026-04-30T18:59:37.137018+08:00` assistant 最终回复是 AMD CPU 命名科普，并未保持金融助手角色边界，也没有拒绝非金融问题
    - 这轮用户只问了一个泛硬件问题，最终仍耗时约 `29s` 才完成
  - `data/runtime/logs/acp-events.log`
    - `2026-04-30T10:59:12.087086Z` 同一 session 先执行 `local_search_files path=company_profiles query="LITE OR Lumentum OR optical OR photonics"`
    - `2026-04-30T10:59:12.094974Z` 与 `2026-04-30T10:59:12.099779Z` 同窗继续展开 `Skill: Stock Research (stock_research)`，提示文本明确写着“active skill context for this turn and future compaction restores until replaced”
    - `2026-04-30T10:59:12.087200Z` 同轮 `rawOutput` 里直接回灌整段 `stock_research` 技能说明
    - `2026-04-30T10:59:12.088062Z` 同轮 `local_search_files` 返回 `{"query":"LITE OR Lumentum OR optical OR photonics"}`
    - `2026-04-30T10:59:12.102225Z` 与 `2026-04-30T10:59:12.102429Z` 同窗仍继续出现 `Lumentum (LITE)` 新闻/画像结果，以及 `天孚通信 / 新易盛` 财报搜索结果，说明非金融问句进入回答前仍在消耗上一轮光通信股票研究链路
  - 对照当前约束：
    - 同一 session 的系统指令明确要求“如用户问题与金融无关，直接礼貌拒绝，并提醒仅支持金融相关话题”
    - 当前 `docs/bugs/` 里已有 [`feishu_direct_stale_symbol_context_hijacks_new_query.md`](./feishu_direct_stale_symbol_context_hijacks_new_query.md)，但那条是新金融问题被旧 ticker 直接答偏；本单是非金融新话题没有被拒绝，且仍误入旧金融 skill / ticker 链路，受影响面和期望行为不同

## 端到端链路

1. 用户在 Feishu 直聊连续讨论股票后，于 `2026-04-30 18:59:08 CST` 切到新话题：`AMD的电脑CPU是什么名字`。
2. 该问题不是金融研究请求，按当前角色边界应被礼貌拒绝或至少提示“仅支持金融相关话题”。
3. 实际运行中，session 先继续调用 `stock_research`，并围绕旧上下文里的 `LITE / Lumentum / 光模块` 执行本地检索与财经搜索。
4. 最终 assistant 在 `18:59:37 CST` 给出了一个普通硬件科普答案，表面上可读，但整轮既没有遵守金融领域边界，也没有清理旧金融上下文。

## 期望效果

- 当用户切到非金融问题时，直聊应优先命中领域边界约束，直接简短拒绝或引导回金融相关话题。
- 新 user turn 若已明显脱离金融域，不应继续展开 `stock_research`、旧 ticker 检索或财经搜索。
- 即使最终选择回答，也应先清除旧金融 skill 上下文，避免把上一轮股票研究链路带入新问题。

## 当前实现效果

- 当前轮没有按领域边界拒绝非金融问题，而是输出了一条正常硬件回答。
- 在输出前，链路继续展开 `stock_research`、`LITE` 本地检索和光通信财报搜索，说明旧金融上下文没有被当前 user turn 及时覆盖。
- 最终答案虽然没有直接把 `LITE` 错答成 AMD CPU，但整轮仍出现明显的技能误路由、上下文污染和额外延迟。
- 之所以定级为 `P3`，是因为主功能链路没有中断、没有错误投递、没有系统崩溃，且最终用户看到的文本本身基本正确；问题主要是角色边界失守和旧上下文造成的质量退化，不直接阻断功能链路。

## 用户影响

- 用户问一个简单硬件问题，本应快速得到拒绝或简短说明，却经历了不必要的金融 research 链路和约半分钟等待。
- 这会降低用户对“新话题切换是否干净”和“系统是否真的按角色边界工作”的信任。
- 无关的 `stock_research` / `web_search` / 本地检索还会白白消耗工具预算，放大后续浅答、慢答或错误路由的概率。

## 根因判断

- 直聊链路对“非金融新话题”的领域边界约束没有在路由前生效，导致模型继续尝试回答。
- 搜索/技能选择阶段对“当前 turn 已脱离上一轮金融主题”的约束不够强，旧 skill context 仍被继承到本轮。
- 目前缺少“当前 user turn 与首个技能/工具目标必须语义一致”的前置校验，所以即便最终文本答对，前面的 skill/tool 仍可能沿用旧金融上下文偏航。

## 下一步建议

- 在 direct 路由前增加非金融意图短路，优先执行领域边界拒绝，而不是先进入股票研究技能。
- 为“长金融会话后切到泛知识问题”补一条回归，验证不会再触发 `stock_research`、旧 ticker 或财经搜索。
- 在 search/skill 入口增加一致性检查：若当前 user turn 不含金融研究意图，禁止继承上一轮 active skill context。

## 修复记录

- 2026-05-01 15:07 CST：当前 user turn 在 `PromptBundle::compose_user_input` 与带接收元信息的 `compose_runtime_input` 中统一放到历史摘要、旧 skill context 与 session context 之后，避免旧 `stock_research` / `LITE` 上下文在提示末尾压过本轮问题。
- 领域边界策略新增明确约束：本轮用户输入优先于历史摘要、旧技能上下文和上一轮标的；当前问题明显不是金融/投研请求时必须先短路回复，不得调用 `stock_research`、`data_fetch`、`web_search` 或沿用旧 ticker / 旧 skill context。
- 回归验证：`cargo test -p hone-channels prompt::tests:: --lib -- --nocapture`、`cargo test -p hone-channels turn_builder::tests::runtime_input_with_recv_extra_keeps_current_turn_last --lib -- --nocapture`。
- 关联 GitHub Issue：无。
