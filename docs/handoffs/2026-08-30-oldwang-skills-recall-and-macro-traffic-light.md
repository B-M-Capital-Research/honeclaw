# oldwang 观点入 skill、召回体检、宏观红绿灯重做（2026-08-29 夜 ~ 08-30）

## 用户要求

> oldwang 的里面有这么多关于估值、公司看法，都优化到各个 skill 里面，并且要注意 skill 本身
> 有没有被合理的召回，关于一些的公司的看法和观点也可以注入到 skill 里，然后在遇到对应的公司
> 召回就行……**不会老是按照标准模板来答**。
>
> 再就是宏观红绿灯这个功能无论是交互还是数据的质量、数据的准确性时效性方面都再 Review 一下
> 并且做的好一点，**回测一下各个宏观问题有没有走我们拉到的这些数据**，如果 OK 的话把 QQQ、SPY
> 的走势也加入进来。

## 一件必须先纠正的事：我自己判断错了一次

我先报了一个 P0：「`rklb的估值` 这类短问句零取数、估值表是编造的」。**这个结论不成立**，
证据链如下，记在这里以免下次再踩：

- `done` 行的 `tools=N` **只统计 runner 的工具调用**。服务端在 runner 存在之前还跑一轮
  pre-turn 预取（`investment_response_guard.rs` 的 `run_pre_turn_enrichment`），
  直接执行 web_search + `data_fetch(search/snapshot/financials/valuation/extended_hours)`，
  **不产生 `runner.tool` 日志行**。
- 判据在 status trail 里：短问法是「**已取得 7 组资料**，开始撰写分析」（`preloaded_evidence_calls > 0`），
  长问法是「资料已就绪」（预取返回 0 组，Agent 只好自己调 12 次）。**方向与我的判断相反**。
- 我举的「两次数字矛盾」是口径选择：`2.53 = 1.34（长期债务）+ 1.19（租赁负债）`，
  另一次明写「1.34 亿，不含租赁」；股本 `629,681,803` 在 12 工具的对照组里逐字相同。

我据此提的「零取数不得发布带报价首行」的边界层修法也被否掉了，理由成立：它要 key 的前提
在两个举证样本里都是假的，会误杀有 provider 支撑的正确首行；且对已发布散文做价格形状匹配，
正是 `AGENTS.md` 138-158 点名禁止的关键词命中式拒绝。

**那条链上真正存在、且四轮全中的缺陷有两个，已修**：

1. **出处等级被升写**：provider 汇总来的数字被标成「公司 2026 年二季报 10-Q」「SEC 官方披露文件」
   「交易所官方收盘价」，而本轮从未调用 `sec_filings` / `press_releases`。
   `valuation-audit` 现在写明换的是**说法**不是**出处等级**，并点出「精确到个位的股本这种数字精度
   本身就在暗示出处」；`prompt.rs` 的三类出处声明延伸到表格「来源」列与行内标注。
2. **表内恒等式不自洽**：净现金 20.49 + 市值 372.73 应得 EV 352.24，两次都写 352.77。
   新增红旗自检：市值 = 现价 × 股本行、净现金 = 现金行 − 总债务行、EV = 市值 − 净现金。

## 二、召回体检：发现一类系统性缺陷

`skill_runtime` 的 reverse-phrase 回退是为中文写的——按标点切出紧凑触发短语，2 字下限
（注释原文：「so a one-character term cannot activate a broad skill accidentally」）。
**但 2 个拉丁字符不是词**：把英文 `when_to_use` 散文按空白切开会得到 `to` / `an` / `is` / `this`，
于是 `please translate this sentence into English` 一句话**同时命中 5 个投研 skill**
（sector-to-stock 320、image_understanding 280、market_analysis 280、options-analysis 280、
portfolio_management 280），全部来自没人当作触发词写的散文。

修法：拉丁片段只有**出自 aliases / id / allowed-tools、长度 ≥3、且整词匹配**才算数；
中文片段完全不变。这样 `etf`、`vix` 这类作者精选的短别名照常工作，`to` / `of` 不再生效。
顺带删掉两个已在污染全站的片段源：`market_analysis` 的 `why did it move`（切出 `it`）与
`valuation-audit` 的 `margin of safety`（切出 `of`）。

实测（`route_sim.py`，已同步新规则）：

| 问句 | 之前 | 现在 |
|---|---|---|
| please translate this sentence into English | 5 个投研 skill | **0** |
| help me write a python function | 4 个 | **0** |
| what is the weather today | 命中 | **0** |
| 结合利率、美债、就业、通胀和 VIX 判断宏观环境 | 只有 hari-invest | market_analysis |
| 降息概率现在多少 | **0 命中** | market_analysis |
| Citi 上调了AAOI的评级 | market_analysis（`it` 误命中） | analyst-coverage |
| 内部人在减持吗 | **0 命中** | company-latest-developments |
| AAOI 这次财报质量有没有问题 | 只有基线 skill | fundamentals |
| MU 下周财报怎么看，市场预期差在哪 | 没有财报口径 | earnings-readout |
| AAOI 最近有什么进展 | **0 命中** | company-latest-developments |

召回缺口的成因几乎都是同一种：**别名写成合成词，用户实际说法多一个字就断**——
`美债利率` vs「利率、美债」分开写；`上调评级` vs「上调**了** AAOI 的评级」；
`财务质量` vs「**财报**质量」；`最新进展` vs「最近**有什么**进展」；
`值得买吗` vs「值**不**值得买」。

## 三、oldwang 的四个分析引擎入库

生产上有 11 个**不在版本控制里**的自定义 skill。其中 `OWSEC` / `OWTI` / `OWERN` / `OWFA`
是老王的四个分析引擎，**aliases 全空、无 when_to_use、id 是英文缩写**，中文问题几乎召回不到。

| 引擎 | 去向 | 搬了什么 |
|---|---|---|
| OWFA 财报法证 | 并入 `fundamentals` | DSO/DIO/DPO/CCC 算式（要 4 个季度序列）、合同负债、SBC 占营收与占经营现金流 |
| OWSEC 监管公告雷达 | 并入 `company-latest-developments` | Form 4 的四项判据（是否 10b5-1、占本人持股比例、几名高管同期、同期有无回购或内部人买入）、S-3「注册额度 ≠ 已发行」；**并第一次让 skill 层指向 `data_fetch(ownership)`**——此前全仓无人引用该数据源 |
| OWERN 财报战役 | 新建 `earnings-readout` | 预期差的四个来源分开列（一致预期 / 公司指引 / 同产业链读数 / 买方定价）、复盘时「幅度」与「中枢」分行、transcript 受限时管理层措辞怎么落笔 |
| OWTI 科技前沿 | 并入 `market_analysis` | 技术代际要落到可对比的物理维度；三档映射必须点名**受损方**（全仓此前 0 命中） |

**四份的固定输出版面和它们自造的「利多 / 中性 / 利空」三档全部丢弃。** 那正是用户说的
「老是按标准模板来答」；而且仓库唯一的四档口径在 `hari-invest/references/decision-rubric.md`，
两套分档词并存会让同一轮出现两种结论体系。

`us_stock_deep_analysis`（aliases `详细分析` / `深度分析`）正文写着「**必须严格按照以下 SNDK
模板结构输出**」，第 5 段还要求做均线支撑位阻力位 RSI——与 `stock_research` 的
「X 技术分析默认指技术能力」直接冲突，而那条规则本身就是从老王「Lite 技术分析 4/10」的点评来的。
**本轮未下线它**（实测它当前没有被模型加载），留给你决定；一条命令可停：
`PATCH /api/skills/us_stock_deep_analysis/state {"enabled": false}`。

公司观点这一侧只做了零 token 成本的一项：`company-index.json` 给 RKLB 补「火箭实验室」、
SPCX 补「太空探索技术」、AMPX 补「安普瑞斯」。**没有往公司卡正文里加观点**——
单卡平均 335→565 字、命中 8 家约 +1286 token 是上一轮实测，而卡片按 `include_str!`
编进二进制并每轮按命中公司注入，加正文的代价是持续的。

## 四、宏观：回测结论是「没有走」，根因不是模型不听话

评测题「结合最新的**利率、10 年与 30 年美债、就业、通胀和 VIX**……」在 r3 / r4 / 单独探针
**三轮都是 0 次 `data_fetch(macro)`**，改用 quote/search/web_search 各 6-10 次。

根因两层，都不在模型：

1. **`VIX` 在整个 `crates/hone-tools/src/` 里没有任何来源**（grep 0 命中）。macro bundle 只有
   treasury_rates / gdp / cpi / unemployment / federal_funds / economic_calendar 六个组件。
2. **工具描述没把 macro 的能力说全**：只写「国债收益率曲线 + GDP/CPI/失业率/联邦基金利率」，
   没提经济日历、没提 VIX、也没写明含 10 年/30 年。模型读了这行，判断 macro 答不了 VIX，
   于是整题降级成逐项拼。**它是照描述读的。**

修法：macro bundle 补 `^VIX`（并单独给它 quote 的 5 分钟缓存策略——macro 的 TTL 是 24 小时，
直接塞进去会让 VIX 变成隔夜数据，还会污染普通 `data_fetch(quote, "^VIX")`）；
描述改成把七个组件列全；`market_analysis` 的宏观流程第 1 条改成可判定的取数顺序
（第一次工具调用是 macro，拿到之前不要逐项拼），并明写**这不是拒答门禁**、
provider 无覆盖时仍要给方向性判断。

上线后同一道题实测：**首行就带 `^VIX` 14.43 与 provider 报价时间**，正文有
10Y 4.67% / 30Y 5.19%（数据期 2026-08-27）、联邦基金 3.88%、失业率 4.5%、2Y/10Y 利差 +47bp，
三情景带概率，升降级触发条件带具体点位（10Y 下破 4.50% / 30Y 上破 5.35% / VIX 破 20）。

## 五、宏观红绿灯

**时效性（这是最严重的一处）**：顶行写「数据截止 2026-08-28」，而按 `macro_specs` 的权重统计，
离该日的天数分布是 `{0天: 0.06, 1天: 0.14, 58天: 0.54, 149天: 0.26}`——
**总分 80% 的权重来自至少 58 天前的数据，26% 来自 149 天前**（实际周薪是季频，口径日 2026-04-01）。
而 `dimensions[]` 根本没有日期字段（只在折叠起来的 `evidence[]` 里），前端渲染的正是 dimensions。

现在：顶行印区间并点名最老的那一维；summary 印口径分布
（实测「日频 2026-08-28（占 20%）、月频 2026-07-01（占 54%）、季频 2026-04-01（占 26%）；
加权中位口径日 2026-07-01」）；每一维带 `period` / `frequency_label` / `lag_days`，
折叠态就能看到。**`lag_days` 只作展示，有测试钉死它永远不影响分数**——
否则就是把发布日历变成对经济的判断。

**准确性**：`核心 PCE 价格` 与 `失业率` 的**打分**本来就是反向的（前者按通胀水平分档，
后者取 `100 − 增长分`），但 `trend_label` 走的是通用增长分支——于是 3.3% 的核心通胀在屏幕上
写着「**改善**」，旁边挂着一个正在下降的健康分；`reason` 还写着「健康分只反映增长方向与动量」，
对这两维是假话。现在这两维用「压力上升 / 压力缓解」，reason 各自说明自己的极性。
**分数一个都没动**（上线后仍是 61.5）。

**告警**：规则只看 `role == "leading"`，于是全表唯一的红卡 DGS30（23.6，role 是
financial_conditions）**根本不可能报警**，`alerts` 为空却同屏显示「2 个领先维度处于收缩区」。
改成逐维度判定 + 前端空态渲染。上线后实测告警：
「美国 30 年期国债收益率 亮红灯：健康分 23.6，口径 2026-08-27（日频）。」

**QQQ / SPY**：FRED **没有 QQQ / SPY 本身**（只收指数不收 ETF），但 `NASDAQ100` 可用且与
已在用的 `SP500` 同为 T+1。所以做成独立的相对走势图，两条线按共同基期归一化到 100，
带 1 年 / 3 年 / 10 年切换，文案写「纳斯达克 100 指数（QQQ 跟踪标的）」，
**display_only、不进健康分**——`SP500` 已作为「市场确认」占 0.06 权重，再加一条就是把
「美股涨不涨」记两遍；而且 `macro_specs` 的权重合计必须为 1.0（有测试断言），
加带权重的 spec 会直接让测试失败。前置修掉了 `downsample` 丢末点的问题
（实测 sp500 的 trend 末点比 evidence 老 18 天、vixcls 老 27 天）。

**交互**：刻度条印出真实分界（宏观 40/55/75、AI 60/80）；溯源行改成区间 + 点名最老维度 +
下次刷新时刻（`next_refresh_at` 此前一直在 payload 里没人用）；删掉那个永远为假的
`market_date !== data_cutoff` 死分支；alerts 空态显式渲染。

**未做**：`released_at` 仍是 `None`。`fredgraph.csv` 没有发布日期，取 `last_updated` 需要
FRED api key 与一个新的外部依赖，本轮不引入；代码里写清了原因。

## 上线与验证

提交 `481963f5`，镜像 `sha256:220d7798…`。harness 63 文件清单，7 个 skill 变更
（含新增 `earnings-readout`），`soul.md` 未变。两次 `active-chat-runs` 空闲读后原子换链。

核验：`git_sha` 一致、`NRestarts=0`、`cloud_mode=cloud`、`local_durable_dependency_count=0`、
**41 skills**（`earnings-readout` 在位）、公网 401、重启后无错误日志。
注意：**宏观红绿灯的代码在 `hone-console-page`，不在 `hone-cli`**——
它才是监听 8077/8088 的那个进程；核验新串要 grep 它。

测试：`hone-web-api` **321 passed / 0 failed**（基线 313，+8）；`hone-tools` 205/5（既有 5 条
`skill_tool::tests::*` 漂移）；`hone-channels` 833/37（基线带内）；`hone-agent` 160/2（既有两条）；
hari-invest 契约 PASS；前端 `525 pass / 0 fail`、`tsc` 干净、`vite build` ✓。

上线后探针（8 题）：

| 题 | 结果 |
|---|---|
| 宏观旗舰题 | `data_fetch macro` + market_analysis，首行带 ^VIX |
| 降息概率现在多少 | macro + market_analysis（此前 0 命中任何 skill） |
| 宏观红绿灯现在是什么颜色 | macro + market_analysis |
| 火箭实验室现在值得买吗 | 中文名命中公司卡，14 个工具 |
| MU 下周财报怎么看，市场预期差在哪 | **earnings-readout** 加载 |
| AAOI 内部人最近在减持吗 | **`data_fetch ownership`**（此前无人指向该数据源） |
| 帮我把这句话翻译成英文 | 零工具、177 字，无投研 skill |

## 留给下一轮

1. **`rklb的估值` 这一题上线后交付了 214 字**，日志显示 listing 终稿检查两轮纠正后预算用尽，
   `deterministic_listing_gap_response` **整体替换**了 14 次取数换来的估值分析，
   并且**把数据时间首行重复了两遍**。违规文案是
   `X: 本轮没有 inactive_listing 结构化证据，终稿却用历史记忆断言退市或未上市`——
   主语 `X` 不是本轮任何实体，像是把大写单字母当 ticker 的误判。这与上一轮 market-move 的
   失败形态同构（预算耗尽 → deterministic 兜底毁掉真答案），修法也应同构：让违规点名
   symbol 与命中原文。**正在修，尚未上线。**
2. **pre-turn 预取的 identity 阶段有个 6 秒 `join3`，超时就丢弃全部已返回结果**
   （`investment_response_guard.rs` 约 3342-3367）。同段注释恰恰说分阶段预算是为了避免
   「一次慢调用把已经返回的 identity search 和 quote 全丢掉」，identity 阶段自己仍在犯这个错。
   实测长问法与 `rklb的估值` 都命中过这条超时。修它会改变长问法行为，本轮未动。
3. `market_trend` 让 payload 增加约 29KB。现有 16 条 sparkline 已经在传 ~68KB 而前端每条只画
   36 个点，更划算的是先裁掉那部分。
4. `us_stock_deep_analysis` 的处置（见上）需要你拍板。
5. 生产上另外几个自定义 skill（`fed_rate_cut_analysis` / `business_model_analysis` /
   `gold_analysis`）都要求「纯文本、不要 Markdown」，与其它所有 skill 的输出契约冲突；
   `gold_analysis` 还与仓库 `gold-analysis` 重复。本轮未动。
