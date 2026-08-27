---
name: Valuation Audit
description: Auditable valuation and entry-point methodology - reconcile every input line by line, pick methods by business model, run scenarios plus reverse valuation, close with a four-state verdict
when_to_use: Use when the user asks whether a stock is cheap or expensive, a fair value or target price, a buy/entry/add price, margin of safety, PE/EV multiple sanity, DCF, or challenges a specific valuation number; also for open-ended single-stock research (分析下X、X怎么样、X如何) and cross-stock preference questions (X和Y更看好哪个) — the reconciliation and red-flag discipline applies there too
user-invocable: true
context: inline
aliases:
  - 估值
  - 估值分析
  - 买入点
  - 建仓价
  - 加仓价
  - 安全边际
  - 贵不贵
  - 便宜吗
  - 合理价位
  - 目标价
  - valuation
  - fair value
  - entry point
  - margin of safety
  - 怎么样
  - 更看好
  - 对比分析
allowed-tools:
  - data_fetch
  - web_search
---

## 估值审计工作流

估值结论必须可复算：读者拿着你的答案应能逐行验算出同一个数。先对账、再选方法、再给情景和反向估值，最后落到四态结论。同一 as-of 基准内只允许一个价格。

### 第一步：取数与对账（先做完这步再谈任何倍数）

1. `data_fetch(search)` 解析实体 → `quote` 取现价与市值 → `earnings_status` 确认最新已发布季度和下次财报日。未发布季度的数字一律标"预期"，不当 actual 用。
2. 并行拉取：`income_quarter` + `income_annual`（稀释EPS、稀释股本、财年截止月）、`balance_sheet_quarter`（现金、总债务→净债务→EV）、`cash_flow_quarter`（经营现金流、CapEx→FCF）、`analyst_estimates`（FY1/FY2 一致预期）、`valuation`、`financial_growth`。
3. 公司指引：优先 `earnings_outlook` / `transcript` / `press_releases`；GAAP→Non-GAAP 调节项和一次性项目用 `sec_filings` 索引的原文核对；不足时用带绝对日期的 `web_search` 补最新指引原文。
4. 一致预期必须与公司最新指引对账：两者差异超过约 5% 时点名差异、说明采用哪个、为什么（例：一致预期 FY1 EPS 11.05 vs 公司指引 11.30 Non-GAAP，采用指引因更新）。

### 估值对账表（终稿必须展示，逐行可复算）

| 项目 | 数值 | 口径/期间 | 来源 |
|---|---|---|---|
| As-of 日期 | | 报价时间戳 | quote |
| 现价 | | 币种/股 | quote |
| 稀释股本 | | 最新季报加权稀释 | income_quarter |
| 市值 | 现价×稀释股本 | 与 quote 市值互验，偏差>3%要解释 | 计算 |
| 现金及等价物 | | 最新季末 | balance_sheet_quarter |
| 总债务 | 含租赁与可转债本金 | 最新季末 | balance_sheet_quarter |
| EV | 市值+净债务 | | 计算 |
| 营收 | | TTM 或 FY1，注明单位 | income_quarter / income_annual / analyst_estimates |
| EPS 分母 | | TTM actual / FY1E / FY2E；GAAP 或 Non-GAAP | income_quarter / income_annual / analyst_estimates |
| 公司指引 | | 期间+口径+性质 | earnings_outlook / transcript |
| FCF | 经营现金流−CapEx | TTM | cash_flow_quarter |
| 预测窗口 | | FY1=哪个自然年段（写财年截止月） | earnings_status |

### 红旗自检（对账表填完后逐条过一遍，命中就先修数再往下走）

- 数量级：市值/营收/EPS 单位与币种一致（百万 vs 十亿）。
- 财年 vs 自然年：财年截止月非 12 月时（如 MU/AMAT/NVDA），写明 FY1 对应的自然年段；预测窗口默认止于 FY2，不越权外推到更远年份。
- GAAP vs Non-GAAP：P/E 分子分母同口径；说"15 倍 PE"必须指明是哪个 EPS（TTM/FY1/FY2、GAAP 与否）。
- 市值 vs EV：EV/EBITDA、EV/S 用 EV；P/E、FCF yield 对市值。缺净债务时不把市值/EBITDA 冒充 EV/EBITDA。
- 季度年化：单季×4 不等于 TTM，周期与季节性业务尤其失真。
- 周期位置：先判断当前利润处于峰/谷/中段，峰值利润×正常倍数不能当买点（见周期股一节）。
- 价值锚：订单/backlog 不是收入；可转债转股价、期权行权价只是合约条款，不是股票价值锚或"安全边际"。
- 单一价格基准：全文所有计算只用对账表那一个 as-of 价格；发现两处价格不一致，回表改成一个，不允许并存。用户点名历史时点（如“以 8 月 6 日盘后 1244 美元为基准”）时，as-of 就冻结在该时点：价格用用户给定值或该日行情，分母用当时已发布的最近一期财报（用 earnings_status 核对该日之前哪期已发布），全文明示“历史时点估值”，不与今日现价混算。
- 字段语义：growth 类字段（如 ebitdaGrowth）是增长率不是绝对额；任何字段代入公式前先确认定义。

### 第二步：按商业模式选 2-4 种方法

- 成熟盈利：正常化 P/E + EV/EBIT + FCF yield，至少两种互验。
- 亏损/微利成长：EV/S + 利润率桥（目标年利润率 × 届时营收 → 隐含利润 → 合理市值），不硬套 P/E。
- 重资产/高折旧：EV/EBITDA 不单独机械使用，须与 EV/EBIT 或 FCF 互验（折旧是真实成本）。
- DCF：只有能逐项列出 FCFF 起点、增长假设、CapEx、WACC、终值方法时才展示；拆不开就改用倍数法并如实说明。
- 资产持有型（MSTR 类币股、控股/投资公司、封闭式基金）：先算每股 NAV（持有资产市值 − 净债务 − 可转债/优先股 ÷ 完全摊薄股本），再讨论 NAV 溢价/折价的历史区间与理由；只看股票 P/E 必然失真，不适用上面的利润倍数法。可转债按转股价核算潜在稀释，但转股价本身不是价值锚。
- 跨公司比较：先判断商业模式是否可比。不可比（如器件商 vs 存储商）就各自选各自的方法算完，再在结论层比较风险收益，先说明口径差异，不共用同一倍数直接排序。

### 第三步：三情景 + 反向估值

- 悲观/基准/乐观各写一行完整算式，公式与代入值齐全，例：基准 = FY2E EPS 6.50 美元（一致预期，Non-GAAP）× 18x = 117 美元。
- 每个情景注明关键假设（营收增速、利润率、倍数）及其性质（历史/指引/一致预期/自设）。
- 反向估值必做一句：当前价隐含了怎样的增长或利润率，与历史区间和公司指引比是宽松还是苛刻。纯叙事（"前景大所以值"）不构成估值。

### 周期股专节（存储、半导体及设备、能源、航运、化工等）

1. 用 `income_annual` + `financial_growth` 拉 5-10 年营业利润率/毛利率区间，标出当前处于峰/谷/中段。
2. 给中周期正常化 EPS：区间中位利润率 × 当前营收规模 ÷ 稀释股本，并用它再算一遍 P/E。
3. 结论里同时呈现"现价/峰值盈利"和"现价/正常化盈利"两个倍数；只在正常化口径上讨论买点。周期高点的低 P/E 是陷阱信号而非便宜信号，明确提示。

### 数据不全时怎么办（永远有出路，不空手而归）

- 缺哪项写哪项：在对账表对应行标"本轮未核验"，说明影响（如"净债务未核验，EV 口径未算，以下仅用市值口径"）。
- 用已核验部分给方向性判断，并明确降低置信度（如"仅市值/TTM 口径看偏贵，置信度中低"）。
- 附补证清单：列出还差哪几笔 `data_fetch` 或原文（如 balance_sheet_quarter、最新指引原文）即可补齐。
- 证据不足本身是合法的四态结论之一，但要附"补齐什么即可升级结论"的触发条件；不要停在"无法判断"四个字。

### 结论纪律

- 必给四态之一：机会 / 持有观望 / 风险 / 证据不足，并附 1-3 个会改变结论的触发条件（价格水平、下季财报、指引修订）。
- 关键数字四要素标注贯穿全文：期间（季度/财年/TTM）+ 单位 + GAAP/Non-GAAP + 性质（历史 actual / 公司指引 / 一致预期 / 分析师假设）。
- 不用未核实数字制造虚假精确：宁可给区间和口径，不给无来源的精确目标价。
- 终稿结构：数据时间与现价 → 估值对账表 → 方法与情景算式 → 反向估值一句 →（周期股加正常化段）→ 四态结论 + 触发条件 +（如有）补证清单。
