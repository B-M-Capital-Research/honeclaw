---
name: Portfolio Management
description: Manage user holdings and watchlist (add/update/remove/watch/unwatch with ticker validation), and answer portfolio-level screening, ranking, and multi-ticker batch questions with reproducible scorecards
when_to_use: Use when the user adds/updates/removes holdings or watchlist entries, asks what is in their portfolio, or asks portfolio-level questions — screening or ranking candidates (Top N) or applying previously defined filter rules from this conversation; for add/trim/cut decisions on named positions (加仓/减仓/割肉), load position_advice instead and keep this skill for the portfolio-level constraints
user-invocable: true
context: inline
aliases:
  - 持仓管理
  - 组合管理
  - 我的持仓
  - 帮我关注
  - 加自选
  - 关注列表
  - 组合筛选
  - 选股排序
  - portfolio management
  - watchlist
  - top picks
allowed-tools:
  - portfolio
  - data_fetch
  - web_search
---

## Portfolio Management Skill

两类任务共用本 skill：**记录管理**（持仓/关注的增删改查）与**组合分析**（组合内筛选、排序、多标的批量表态）。先判断用户属于哪类再走对应流程；两类都以 ticker 验证为第一步。

### 心智模型：持仓 vs 关注

| | 持仓 (Holding) | 关注 (Watchlist) |
|---|---|---|
| 何时使用 | 用户真实买入了标的 | 想追踪但未买入 |
| 必填字段 | ticker + shares + cost_basis | 仅 ticker |
| 资金统计 | 计入 total_shares / P&L | 不计入任何资金口径 |
| 主动推送 | 收新闻/价格异动 | 与持仓同级，同样收推送 |
| 底层字段 | `tracking_only: false/缺省` | `tracking_only: true` |

用户没说明 shares 时一律走 `watch`，不要 `add shares=0`——后者会在资金统计里留下 0 股的垃圾持仓。

### Ticker 验证（写操作与分析共用的第一步）

用户常给缩写/中文名/口误/非美股写法（如 `07709` 这类港股代码），先解析再动手：

1. 每次写入或表态前先 `data_fetch(data_type="search", query="...")`；用户给的 ticker 只是当前轮实体线索，不是免检凭据
2. 从搜索结果确认 `symbol` + `name`，在回答里复述"你说的 X，我理解为 NAME (SYMBOL)"
3. 搜索无果或多个候选相近时，列出候选请用户确认，不要默默取第一个近似结果继续分析

### Tool 调用速查

| 自然语言 | Tool 调用 |
|---|---|
| 查看我的持仓/关注 | `portfolio(action="view")` → `{holdings: [...], watchlist: [...]}` |
| 帮我关注 NVDA | `portfolio(action="watch", ticker="NVDA")`（幂等） |
| 我以 175 买了 100 股苹果 | `portfolio(action="add", ticker="AAPL", quantity=100, cost_basis=175)` |
| 买了之前关注的标的 | `add` 自动转持仓，返回 `promoted_from_watchlist: true` 时告知用户"已从关注升级为持仓" |
| 取消关注 | `portfolio(action="unwatch", ticker=...)`（仅删关注；真实持仓会拒绝并提示用 remove） |
| 我不持有了 | `portfolio(action="remove", ticker=...)`（持仓/关注通用） |
| 更新成本价 | `portfolio(action="update", ticker=..., cost_basis=...)` |

### 记录管理工作流

1. 先 `view` 查当前状态，判断意图是加关注还是加持仓
2. 没提 shares/cost → `watch`；提了 → `add`；发生 promote 时务必向用户汇报
3. 删除区分：只想停推送 → `unwatch`；清掉真实持仓 → `remove`
4. 期权关注同样支持：`watch` 时 ticker 可由 `underlying/expiration_date/option_type/strike_price` 自动生成

### 组合分析纪律（筛选 / 排序 / 批量表态）

**快照纪律**。回答第一行声明数据快照时间与口径（例："数据快照：北京时间 YYYY-MM-DD HH:MM，基于最新可得报价与已披露财报"）。同一输入应产出同一排序；本次排序与本对话早先结果不同时，明确指出是哪项数据更新导致的（新财报、价格变动、评级调整），不让结果无解释地漂移。

**候选池先行**。"哪家是买入点""帮我挑一个"这类开放问题，先定义候选池再比较，优先级：用户点名的标的 > 用户持仓+关注列表 > 用户指定的板块/主题（用 `gainers_losers` / `sector_performance` / `search` 成池）。用户没给池时默认用其持仓+关注，并在回答里声明"本轮候选池为你的持仓与关注共 N 只"。跳过候选池直接空降一个池外答案，结果既不可复现也不可审计。

**评分卡展示**。筛选/排序类回答要让读者能用表里的数字自己重算出同样的排序：

- 列出筛选维度和每个维度的规则（阈值/方向；有权重就写出数值，没有就说明是等权或定性排序）
- 表格展示每只候选在各维度的实际取值；关键财务数字标注期间(季度/FY/TTM)+单位+GAAP/Non-GAAP+性质(历史 actual/公司指引/一致预期/分析师假设)
- 排序结论直接由表中数值推出。可复现性来自透明度，不来自"保证结果一致"的口头承诺

**覆盖完整性**。用户点名 N 只（如"BE COHR AMD MU SNDK 今天可以加仓吗"）时逐只核验、逐只表态，表态条数等于 N。个别标的数据取不到就单独标注该只"证据不足 + 缺什么"，其余照常给结论；不要静默跳过任何一只，也不要在只核验了部分标的时给"整体可以加仓/整体观望"这类组合层面动作暗示——组合结论只能建立在全覆盖或明示缺口之上。涉及成本价语境的逐只加减仓判断，按 `position_advice` skill 的证据标准执行。

**追问锚定**。追问（"那XX呢""按你说的因子三过滤""用刚才那份清单"）先回读本对话已经给出的定义、清单、评分卡：找到了就直接沿用并点明"沿用上文定义：因子三=……"；确实找不到再请用户补充。当新回答与本对话早先内容矛盾时，先承认并修正早先内容，不要声称"未定义"或"没说过"。

### 结论纪律

每只候选落在四态之一：**机会 / 持有 / 风险 / 证据不足**；组合层面另给一句总结（可加仓方向、需减配方向、集中度提示）。数据不全时的出路固定为三步：如实披露缺口 → 基于已核验部分给方向性判断并用降置信度措辞（"倾向于""证据偏向"）→ 列补证清单（还差哪个 data_fetch、哪份财报、哪项确认）。宁可给"方向 + 缺口"，不要用未核实数字凑出虚假精确的排名。

### 与通知偏好联动

用户说"只收持仓和关注标的的推送" → 去 `notification_preferences` skill 设 `set_portfolio_only=true`。关注标的与持仓同级触发 registry，自动进入白名单。

### 组合数字的取数纪律

**用户贴的明细优先**：用户在本会话粘贴了持仓/交易明细时，回测、过滤、复盘一律以那份明细为准；要改用系统存储的持仓，先写明两者不一致再请用户确认。

**盈亏方向先对表**：给任一仓位定性前，把同一张表里的现价与成本价相减——现价低于成本一律写「浮亏」，不允许表格显示浮亏而正文称其「真正盈利」「带来正收益」。复述本会话已给过的笔数、胜率、盈亏幅度之前先逐行重算，与上文不一致必须写明以哪个为准。

**精确数字要落到本轮工具**：估值倍数来自本轮 `valuation`，利率/短债收益率来自本轮 `macro`，股价来自本轮 `quote`/`snapshot`。没调对应工具就只能定性表述（「短债提供低风险收益」「估值低于同行中枢」），不要写「静态 PE 约 17 倍」「4.5%~5.0% 无风险收益」这类精确值或区间。

### 回测与收益声明

历史价格序列是有的：`data_fetch(data_type="price_history", ticker="X", from="YYYY-MM-DD", to="YYYY-MM-DD")` 返回除权除息调整后的日线（`date` / `adjOpen` / `adjHigh` / `adjLow` / `adjClose` / `volume`），缺省取最近一年。所以下面两类要分开：

**能算的：区间收益与逐笔复盘。** 用户给了买入日期或成交价（"一年前投入 100 万""我 70 买的"），或要按本对话已定义的条件回放自己的历史买点时，取真实序列算：用调整后价而不是原始收盘价，写出取的是哪一天的哪个字段，区间内有拆股或分红时点名说明。**不得假设一个买入价再往下推导收益**——评测里出现过凭空取 9.48 港元算出"总收益约 11.7 万"，这是编造。取不到序列（非美股覆盖、区间超出范围）就说明缺口并给公式，不要给数字。

**不能算的：带统计结论的策略回测。** "这个因子策略胜率多少""年化多少"需要样本外验证、幸存者偏差、未来函数、交易成本与换手、行业暴露的处理，逐日序列本身不构成回测框架。这类照实说明：① 能给的是这几笔在真实价格下的实际结果，不是经过检验的策略绩效；② 给出规范回测所需的前提清单，用户可以拿去任何回测平台执行；③ 未经回测的筛选阈值照常可用，但标注"未经历史回测，存在过拟合近期行情的风险"。
