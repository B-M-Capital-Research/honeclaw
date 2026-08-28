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

## 追加：数据能力接入与可复用视角 skill

同一份评测的第二轮跟进，回答的是「FMP 有没有评级数据」「能不能把研报和评级做成 skill」
「多建立和复用各种 SKILL」。

### 数据层：枚举挡住了已经实现的能力

`data_fetch` 的 `data_type` 是硬枚举，只列 15 个值，而代码里实现了 36 个。评级家族全在缺口里：
`ratings_snapshot` / `grades_consensus` / `price_target_consensus` / `price_target_summary` /
`analyst_actions` / `analyst_estimates` 都可达、都不在 schema、`skills/` 引用次数为 **0**
（`analyst_actions` 3 次，其余 0）。逐个打到线上 API 实测过返回，字段写进了工具描述：

- `grades_consensus` → `strongBuy/buy/hold/sell/strongSell` 家数 + `consensus`
- `price_target_consensus` → `targetHigh/targetLow/targetConsensus/targetMedian`
- `price_target_summary` → 近一月/一季/一年/全期家数与均值 + `publishers`
- `ratings_snapshot` → `rating`(A+~F) + `overallScore` + 六个分项 1–5，**是 provider 量化打分卡不是投行观点**
- `analyst_actions` → `grades` 逐条动作 + `grades_news`/`price_target_news`（`newsTitle`/`newsURL`/
  `gradingCompany`/`previousGrade→newGrade`/`priceWhenPosted`），**最接近研报的结构化源**

新增 `price_history`：`historical-price-eod/dividend-adjusted`，按 `from`/`to` 返回除权除息调整后
日线（`adjOpen/adjHigh/adjLow/adjClose/volume`），缺省最近一年。评测里「一年前投入 100 万到今天
总收益」那条，Hone 凭空取了 9.48 港元买入价推出「总收益约 11.7 万」——有了真实序列才谈得上算。
用调整后价而非原始收盘价：区间内一次拆股会让原始价算出的收益错得看不出来。
`portfolio_management` 里「当前没有历史价格序列与回测工具」这句已不成立，拆成「能算的区间收益
与逐笔复盘」和「仍需回测框架的策略绩效」两段。

`earning-call-transcript` 正文实测是 `Restricted Endpoint`（当前订阅只给日期列表），
限制写进了工具描述，避免 skill 承诺「管理层原话」。

同时把新暴露的类型补进 `data_fetch_data_type_uses_security_target`——否则一次 `price_history`
调用不会被记成覆盖了那只证券。

### 五个可复用的分析视角 skill

`analyst-coverage`（投行评级/目标价/研报）、`fundamentals`、`moat`、`scarcity-differentiation`、
`first-principles`。估值**不新建**：`valuation-audit` 就是估值 skill。

它们有两个入口：用户点名该视角时由路由命中；`stock_research` / `valuation-audit` /
`sector-to-stock` / `etf-analysis` 需要展开某一维时用 `skill_tool` 拉起来——
`stock_research` 里新增的「哪一维要展开时加载哪个 skill」对照表就是这个复用点。

判断口径一律引用 `hari-invest` 框架 1/2，不重写定义；这批只写「怎么跑出来、产出什么格式」。
几条代表性的可判定规则：护城河至少两项当轮数字且来自分型表不同两行；基本面用「遮数字自查」
区分罗列与判断（把数字遮住后仍能看出公司处在什么状态才算判断）；稀缺度与差异化各给 1–5 整数分
且每分挂一条当轮证据；第一性原理必须写成可代入数字的等式并含供给段。

### 路由影响

新增 5 个 skill 后仓库共 29 个，而每轮只注入前 5 个。因此这批的 alias 只放**点名该视角**的词
（护城河/壁垒、稀缺/差异化、第一性原理/底层逻辑、投行/研报/评级、基本面），不放「分析」「怎么样」。
按 131 条真实问题模拟：**原有 top-1 掉出前五的 0 条**，空提示率 28 → 27；视角问题各自命中：

| 问句 | top-3 |
| --- | --- |
| 分析下 AAOI 的护城河 | moat / stock_research / company-thesis-ratings |
| MU 的底层逻辑是什么 | first-principles |
| 存储的稀缺性怎么样 | scarcity-differentiation / valuation-audit |
| NVDA 投行给的目标价是多少 | analyst-coverage / valuation-audit |
| 看看 TSLA 的基本面 | fundamentals / stock_research |
| 帮我分析一下MU（对照） | stock_research（未被抢） |

`colloquial_chinese_questions_surface_their_scenario_skill` 回归测试扩到 16 条，含这 5 条视角问句
与「帮我分析一下MU」的对照。`cargo test -p hone-tools --lib` 200 passed / 5 failed，
失败仍是既有的同 5 条 `skill_tool::tests::*` 漂移。

### 这一部分的未尽事项

- 仍然没有研报全文数据源。`analyst-coverage` 的做法是两层分开：第一层 `analyst_actions` 的结构化
  记录（谁、何时、评级/目标价怎么变、当时股价、原文链接），第二层带绝对日期的 `web_search` 补论点
  并标明是转述。「今天各大投行研报摘要」那条 3/10 能改善到什么程度，要看下一轮评测。
- `web_search` 没有暴露 `include_domains`。Tavily 支持，但仓库当天刚加了「Tavily 兼容端点」配置，
  兼容实现未必支持该字段，本轮没动；需要定向到 thefly/benzinga 这类站点时先用 query 里的机构名。
- `key-executives` 端点实测可用（评测里有一条问「高管团队」），本轮没接，属可选。

## 生产部署记录（2026-08-28 16:39–16:55 北京时间）

用户批「都可以上生产」。这次是**二进制与 harness 分开发布**，因为两者的最后改动点不同：

| 层 | 版本 | 理由 |
|---|---|---|
| 二进制 | `897f66fd` | 最后一个构建出镜像的 revision。`f047326d` 只改了非 `stock_research` 的 skill，而 `runtime-image.yml` 的 path filter 只含 `skills/stock_research/**`，所以没触发构建。`git diff --stat 897f66fd f047326d -- crates/ bins/ agents/ memory/ soul.md Cargo.toml Cargo.lock` 为空，二进制等价已证明 |
| harness（skills + soul.md） | `f047326d`（HEAD） | harness 不随镜像走：`HONE_SKILLS_DIR=/srv/honeclaw/skills`，需单独投递 |

### 发布前检查

- `cargo test -p hone-tools --lib`：200 passed / 5 failed，与基线同 5 条 `skill_tool::tests::*` 漂移。
- `cargo test -p hone-channels --lib`：826 passed / 40 failed，比暂存基线多 3 条；3 条单线程复跑全部通过，属已知并行漂移。
- 生产磁盘 4.8G 空闲（门槛 2G），`hone-web` active。

### harness 投递

`git archive f047326d skills soul.md` → 29 个 skill、0 符号链接、60 文件 SHA-256 清单
（bundle sha `ef4a174a2b7a6d5f35411bdd`），上传后本地/远端 sha 一致。dry-run 报 15 个 skill 变更
（新增 7：analyst-coverage / etf-analysis / first-principles / fundamentals / moat /
scarcity-differentiation / sector-to-stock；改动 8），`soul.md unchanged`。apply 输出
`manifest OK (60 files)`，备份落在
`/srv/honeclaw/skills/backups/pre-f047326d...-20260828T163936Z`。

### 二进制切换

摘要 `ghcr.io/b-m-capital-research/honeclaw-runtime:897f66fd...`
→ `sha256:b270696064bb541dd35c107d5315bcfc9ff8748eaad1acc522bd900f0f0a1e45`，
`stage_ghcr_runtime.sh` 报 `[PASS] verified`。两次 `/api/runtime/active-chat-runs` 均 `{"count":0}`
后，先把 `previous` 指到 `a43f99c8`，再原子换 `current` 到 `897f66fd`，`systemctl restart hone-web`。

### 发布后核对

- `/proc/<MainPID>/exe` → `.../897f66fd.../bin/hone-cli`；`RELEASE_METADATA.git_sha=897f66fd...`。
- `/api/meta`：`cloud_mode=cloud`、`local_durable_dependency_count=0`、`version=0.15.3`。
- `/api/skills`：40 个，7 个新 skill 全部在位并能取到正文（3.6–4.2k 字）。
- `127.0.0.1:8088/api/public/auth/me` → 401。
- 重启后 4 分钟内 `journalctl -p err` 无条目，`skill not found` 0 次（对照发布前两天 990 次）。
- 逐文件 SHA-256 比对：仓库 `f047326d` 的 59 个 skill 文件，生产端**全部存在且内容一致**。

### 顺手处理与遗留

- 清掉了生产 skills 目录里 59 个 macOS AppleDouble 垃圾文件（`._*`，时间戳 08-23 / 08-27，
  是更早几次从 Mac 打包投递留下的，非本次产生）。清理前全部用 `file(1)` 确认类型并打包备份到
  `backups/appledouble-20260828T165459Z.tgz`；清理后 `/api/skills` 仍是 40。它们本来也不影响加载
  （loader 只遍历目录读 `SKILL.md`），清掉是为了让今后的逐文件比对不再有噪声。
- `skills/README.md` 生产端比仓库少 6 行（介绍 `company-thesis-ratings` / `hari-invest` 的那段）。
  安装脚本只投递 skill 目录、不投递顶层 README，运行时也不读它，本次未动。
- 清理最老的 release `dfa9d8cd`，保留 `current=897f66fd` / `previous=a43f99c8` / `1e7cfc15` 三份，
  磁盘回到 4.8G。
- 既有偏差照旧：`/root/.docker/config.json` 里的 GHCR 凭据是手工放的，不在配置管理里。

## Next Entry Point

- 同步到生产后，用同一份 131 条问题重跑路由回归（`route_sim.py`），确认线上 skills 与仓库一致。
- 下一轮人工评测重点看这三条是否变化：深度个股「四态 + 估值区间同时出现」（基线 31%）、
  「稀缺度/差异化」（基线 27%）、「胜负手/第一性原理」（基线 6%）。
