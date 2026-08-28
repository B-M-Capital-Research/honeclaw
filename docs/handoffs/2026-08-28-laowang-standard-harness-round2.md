# 老王标准评测（8/22–8/28）驱动的第二轮 harness 改造

- title: 按人工评测重做深度个股收口、估值定倍数、荐股链、ETF 五步与中文路由
- status: done
- created_at: 2026-08-28
- updated_at: 2026-08-28
- owner: ecohnoch
- related_files:
  - `skills/sector-to-stock/SKILL.md`（新增）
  - `skills/etf-analysis/SKILL.md`（新增）
  - `skills/stock_research/SKILL.md`
  - `skills/valuation-audit/SKILL.md`
  - `skills/market_analysis/SKILL.md`
  - `skills/scheduled_task/SKILL.md`
  - `skills/chart_visualization/SKILL.md`
  - `skills/company_portrait/SKILL.md`
  - `skills/notification_preferences/SKILL.md`
  - `crates/hone-tools/src/skill_runtime.rs`
- related_docs: `docs/handoffs/2026-08-28-direct-session-audit-harness-tuning.md`

## Summary

输入是一份人工评测表：131 条真实用户问题、84 条在生产站复现出的回答、43 条人工点评，
其中 39 条给了 `N/10`。均分 **5.95**，分布 3×1 / 4×3 / **5×17** / 6×5 / 7×2 / 8×11。

按类型看，差的集中在深度个股：

| 类型 | n | 均分 |
| --- | --- | --- |
| P2 基本面/财报 | 11 | 5.3 |
| P7 上下文/任务 | 7 | 5.3 |
| P1 估值/买入点 | 10 | 5.7 |
| P5 组合/选股 | 1 | 6.0 |
| P3 新闻/异动 | 6 | 7.0 |
| P4 宏观 / P6 期权 / P8 资料 | 4 | 8.0 |

**17 条 5 分是同一个失败**：结构走完了九段，却没落到一个算出来的数。点评里 9 行写的是同一句
`和上面一样，没有结论。M47`。逐条核对那 12 条被判「没有结论」的回答：只有 4 条同时出现了
四态标签和合理估值/目标价，5 条两样都没有。而即使两样都有的那几条仍是 5 分——老王要的不是
标签加倍数罗列，是「分母 × 倍数 = 价格区间」以及现价站在区间哪个位置。

对全部 44 条深度个股回答做的确定性统计（关键词命中）：

| 维度 | 命中率 |
| --- | --- |
| 四态标签（机会/持有/风险/数据不足） | 54% |
| 合理估值 / 目标价 / 估值区间 | 43% |
| 二者同时出现 | 31% |
| 稀缺度 / 差异化 | 27% |
| 第一性原理 / 胜负手 | 6% |
| 对标 / 可比公司 | 6% |

## 两个把这轮分数压住的结构性原因

**一、评测测的是一台没有纪律层的 Hone。** 表里 83 条复现全部发生在 2026-08-28 02:39–03:34；
生产直到当天 18:43 才第一次装上 `hari-invest` 与 `company-thesis-ratings`（见上一份 handoff）。
而 `hari-invest/references/investment-frameworks.md` 里，老王这次点名要的东西早就写好了——
框架 1 第一性原理与现实验证、框架 2 稀缺与差异化、框架 4 动态杠铃、框架 5 板块优先的仓位分配。
它们从没被加载过。所以本轮的原则是：**hari-invest 已经写了的不再重写，只补它没写的**。

**二、一半的问题根本没被指到任何场景 skill。** `turn_builder.rs:186-198` 每轮用整句问题调
`search_for_stage(...)` 取前五个 skill 注入【本轮相关技能提示】。`skill_runtime.rs:919` 的打分
对中文只有一条通路（`reverse_phrase_match`）：把 skill 自己的字段按标点切成 2–16 字片段，
片段必须**整段出现在用户问句里**才得分。按这个规则把 131 条问题跑了一遍：

- **69/131 = 53% 的问题命中 0 个 skill**，提示列表为空。
- 最常见的问法「帮我分析一下 MU」「rklb 怎么看」「当前最适合建仓的美股」「给我出一张饼图」全部为空。
- 原因很直接：`stock_research`（规范的个股研究入口）与 `chart_visualization`、`scheduled_task`
  的 description / when_to_use / aliases **全是英文**，中文句子切不出共同片段。
  8/27 改过的 `market_analysis` / `valuation-audit` / `position_advice` 有中文 alias，就能命中。

## What Changed

**新增两个 skill**（老王点名「应该建立一套 SKILL」的两条）

- `sector-to-stock`：板块/主题 → 选股 → 配比。稀缺度与差异化按 1–5 打分并排序（判断口径引用
  hari-invest 框架 2），标的对比表含基本面/护城河/财务/估值，最后按杠铃原则给比例区间。
  对应 4/10 的「存储太空光芯片占比例多少」、6/10 的「黄金股票推荐一些」、4/10 的「当前最适合建仓的美股」。
  边界：候选池已是用户持仓/关注时走 `portfolio_management`；单只公司深研走 `stock_research`。
- `etf-analysis`：老王给的五步。重心在第 4 步——前十成分股逐家取机构目标价与自算区间，
  按持仓权重加权汇总成 ETF 层面的合理价区间，并披露覆盖了多少权重。组合 PE 用加权调和平均
  （`1 / Σ(ŵᵢ/PEᵢ)`）而不是算术平均。对应 QQQ 那条 5/10。

**扩展四处**

- `stock_research` 新增「深度个股分析的收口纪律」：① 结论段必须同时有四态+置信度、一个带算式的
  合理价值区间、现价相对该区间的位置，三者缺一即未完成；输入不足时写出缺的那一项与补齐后的分档走向。
  ② 结论之前先用一两句点出**胜负手**（哪一个变量的走向直接决定这次判断对错），并附评测里三个
  形状不同的真实标定例子（腾讯＝护城河深但被 AI 落后+Capex 抬升压住估值、光模块三家＝产能、
  零跑＝行业太卷且差异化弱本身就是结论）。③ Bull/Bear 各自要落到「产能→收入→估值」链上的至少一环数字。
  ④「X 技术分析」默认按技术能力/技术路线理解，不默认做 K 线。
- `valuation-audit` 新增「先定类型、再定倍数」：类型判断（消费/高科技/周期/IP 内容/公用事业/平台）
  要附主营收入构成、增长来源、竞争格局三项依据；必须点名 2–3 家同类型可比公司并本轮取数拿倍数
  （PPMT 对标迪士尼/梦工厂，特斯拉的高倍数来自无人驾驶叙事而不是卖车）；稀缺度与差异化要显式把
  倍数往上或往下调并说明理由（VST：电力紧张 + AI 数据中心缺电 → 稀缺度拉高倍数）。
- `market_analysis` 新增「归因之后的三段」：把事件换算成数字（ATM 融资额 ÷ 市值 = 稀释比例 →
  EPS 摊薄 → 合理价下移）、判断动没动长期公式（一次性冲击 vs 结构改变）、落到分持仓状态的条件化应对；
  另加「板块级分化先回答 Price In 还是基本面变了」。
- `scheduled_task` 新增「时刻优先于条件轮询」+「建完回读真正落库的时刻」：用户说出任何可解析钟点，
  就必须建常规任务、不得传 `heartbeat`；`add`/`update` 返回后复述的时刻必须取自返回体而不是入参，
  读到 `00:00` 而用户并没要午夜推送时当场 `update` 修回来。

**中文路由层**（本轮真正的杠杆）

给 `stock_research`、`chart_visualization`、`scheduled_task`、`company_portrait`、`valuation-audit`、
`market_analysis`、`sector-to-stock` 补短触发词 alias；`chart_visualization` 的 description /
when_to_use 从纯英文改写成中文并列出饼图/占比图/画图/出一张图等说法；三个 skill 的 when_to_use
末尾加了用 `、` 分隔的触发词段（这样每个词才会被切成独立片段），把「分析」这类过宽的词放在
when_to_use（权重 80）而不是 aliases（权重 110），让更具体的 skill 仍能拿到 top-1。

## Verification

- 用 Python 复刻 `score_skill` / `score_field` 打分（`<scratch>/route_sim.py`），对 131 条真实问题
  回归：**空提示 69/131 = 53% → 28/131 = 21%**，top-1 分布从 valuation-audit 独大变成
  stock_research 31 / valuation-audit 24 / sector-to-stock 12 / market_analysis 7 /
  portfolio_management 6 / chart_visualization 4。
  三条饼图问题全部 top-1 命中 `chart_visualization`；九条「帮我分析一下 X」全部命中 `stock_research`。
- 新增 Rust 回归测试 `skill_runtime::tests::colloquial_chinese_questions_surface_their_scenario_skill`：
  直接加载仓库真实 `skills/` 目录，断言 11 条真实问题各自的场景 skill 出现在前五提示里。
  这条测试用的是生产同一套 `search_for_stage`，不会随 Python 复刻漂移。
- `cargo test -p hone-tools --lib -- --test-threads=1`：199 passed / 5 failed，
  失败的仍是既有的同 5 条 `skill_tool::tests::*` 漂移，本轮零新增。

## Risks / Follow-ups

1. **未上线**。本轮只改了仓库。harness 不随镜像发布，要按
   `docs/handoffs/2026-08-28-direct-session-audit-harness-tuning.md` 的 staging 流程同步到
   `/srv/honeclaw/skills`；`skill_runtime.rs` 的回归测试属于二进制，要走镜像。
2. **规则总量**。两个新 skill 各约 1 万字节，四处扩展合计约 230 行。评审（第 7 个 agent）
   逐条比对后砍掉了与 `prompt.rs` / `soul.md` / 同文件已有条款重复的部分：
   sector-to-stock 的「常见失败形态」整节、etf-analysis 的九段映射表与「表达」节、
   stock_research 里第二次出现的「不得编造净债务/远期利润」禁令、胜负手示例表 5 行砍到 3 行。
   这些都是按需加载的场景层，`soul.md` 本轮一个字未动。但 sector-to-stock 与 etf-analysis
   会参与每轮前五的竞争，下一轮评测要观察它们有没有挤掉更合适的 skill。
   评审还纠正了一处事实错误：草稿写「heartbeat 落库时 hour/minute 一律记 0」，
   而 `memory/src/cron_job/storage.rs:437-442` 是 `hour.unwrap_or(0)` 且只对 heartbeat
   跳过校验——传了 20:30 就存 20:30，丢时刻发生在模型**省略** hour/minute 时。已按代码改正。
3. **P1 画图仍有能力缺口**。渲染器支持 line/area/bar/scatter/histogram/horizontal_bar，**没有 pie**。
   本轮解决的是「请求根本没路由到画图 skill」，以及路由到之后用 horizontal_bar 顶替并说明。
   老王要的真饼图需要在 `skills/chart_visualization/scripts/render_chart.py` 里加 pie 类型，属于另一单。
4. **P0 图片识别**未在本轮处理：评测行 `港股长飞…【附件内容未在管理页复传】` 没有复现证据，
   上一轮 handoff 已记录飞书图片附件链路的既有缺陷。
5. **知识/宏观/新闻类不要再加规则**。这三类在评测里是 7–8 分（`宏观类的确实回答还行`、
   `新闻类还行`、`方法的回复正常就好`），本轮刻意没碰。
6. 「今天各大投行研报摘要」3/10 是**数据源缺口**（没有研报库，只能靠搜索），提示词层解决不了，
   老王自己也写了「感觉要接入 IMA 那样的数据库」。

## Next Entry Point

- 同步到生产后，用同一份 131 条问题重跑路由回归（`route_sim.py`），确认线上 skills 与仓库一致。
- 下一轮人工评测重点看这三条是否变化：深度个股「四态 + 估值区间同时出现」（基线 31%）、
  「稀缺度/差异化」（基线 27%）、「胜负手/第一性原理」（基线 6%）。
