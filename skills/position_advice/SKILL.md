---
name: Position Advice
description: Cost-basis-aware position adjustment advice — add / hold / trim / cut-loss judgments grounded in the user's actual holdings, current-turn quotes, and per-ticker event-risk checks
when_to_use: Use when the user asks whether to add, trim, hold, average down, or cut losses on specific positions — 加仓/减仓/补仓/割肉/回本/被套 — or wants rebalancing advice tied to their cost basis or current holdings
user-invocable: true
context: inline
aliases:
  - 加仓
  - 减仓
  - 补仓
  - 割肉
  - 被套了怎么办
  - 回本
  - 调仓建议
  - position advice
  - rebalance advice
  - OWCW
allowed-tools:
  - portfolio
  - data_fetch
  - web_search
---

## Position Advice Skill

成本价语境（被套/回本/加仓/割肉）的调仓判断。核心纪律：先把持仓事实摆对，再逐只核验证据，最后给**条件化**建议——"若X则A，若Y则B"，并附证伪条件。

### 第一步：实体解析 + 持仓事实复述

1. 逐个解析用户点名的标的：`data_fetch(data_type="search", query="...")`。对 `07709` 这类非美股写法、中文名、缩写尤其要确认——实体解错，后面全错。解析结果向用户复述"你说的 X 我理解为 NAME (SYMBOL)"，有歧义就列候选请用户挑。
2. `portfolio(action="view")` 拉取真实持仓。找到该标的时，先复述持仓事实再谈建议：成本价、数量、当前价（当前轮 quote）、浮动盈亏%（自己算：(现价−成本)/成本）。
3. 标的不在持仓记录里但用户在谈成本（"我 180 买的"）：按用户口述成本计算，标注"以下按你口述的成本 XX 计算"，顺手问是否要记入持仓。
4. 用户完全没给成本、持仓里也没有：仍然作答，但明确说明这是"不含你成本语境的标的判断"，补上成本后结论可能不同。

### 第二步：逐只核验（覆盖完整性）

用户点名 N 只就核验 N 只、表态 N 只。每只至少覆盖：

| 证据 | 调用 | 用途 |
|---|---|---|
| 现价与涨跌 | `data_fetch(data_type="quote", ticker=...)` | 浮盈计算、当日语境 |
| 财报临近度 | `data_fetch(data_type="earnings_status", ticker=...)` | 下次财报日期、最新已发布季度——财报前夕加仓是事件风险，要主动提示 |
| 估值锚 | `data_fetch(data_type="valuation", ticker=...)` | 现价贵贱的粗锚 |
| 边际变化 | `data_fetch(data_type="news", ...)` / `analyst_actions` / `web_search` | 近期利空利好、评级与目标价变动 |

个别标的某项数据取不到：该只单独标注"证据不足（缺X）"，基于已有证据给方向性判断并降低置信度，其余标的照常完整表态。不要因为一只缺数据就整体不答，也不要只核验了两只却对五只给出整体动作暗示——组合层面的话只能建立在全覆盖或明示缺口之上。

### 第三步：条件化建议

对每只标的输出五件套：

1. **持仓事实**：成本/数量/现价/浮盈%，附 quote 时间
2. **关键证据**：2-4 条；财务数字标注期间(季度/FY/TTM)+单位+GAAP/Non-GAAP+性质(历史 actual/公司指引/一致预期/分析师假设)
3. **四态结论**：加仓 / 持有观察 / 减仓·止损 / 证据不足
4. **条件化动作**：一律用若-则结构，如"若想控制事件风险，则等 X 月 X 日财报落地再定；若接受波动且长逻辑未变，则分批、单次不超过现有仓位的一定比例"。不给无条件的"今天可以加仓"
5. **证伪条件**：什么信号出现说明这个判断错了（如毛利率连续两季下滑、指引下修、关键客户流失、财报不及一致预期）

仓位表达用相对/区间语言（"分批""不超过现有仓位的一半"）。目标价只在当前轮证据里真实存在时引用，并标注来源与性质（一致预期/单家分析师观点）；没有就不编造精确点位。

### 数据不全时的出路

固定三步：如实披露缺口 → 基于已核验证据给方向性判断，用降置信度措辞（"倾向于""证据偏向"）→ 给补证清单（等哪天的财报、看哪份 filing、跟用户确认成本价）。每只标的永远落在四态之一，哪怕落点是"证据不足 + 补证清单"——没有结论本身就是一种失败。

### 组合层面追问

用户追问"那整体怎么调"：先回读本对话已给出的逐只结论，在其基础上汇总，不另起炉灶重排；新结论与早先表态矛盾时先承认并修正自己。集中度与相关性要点名提示（如五只全在半导体/硬件链上，同涨同跌，分散意义有限）。

### 表达边界

以研究参考的口吻给建议：证据、条件、证伪写清楚，最终决策留给用户。不堆免责声明，结尾一句"以上为基于当前数据快照的研究参考"即可。
