---
name: Company Latest Developments
description: Build a dated, first-party-sourced timeline for "某公司/某项目最新进展、最近动态、最近发生了什么" questions by combining SEC filings, official press releases, dated analyst actions, ticker news, and targeted web search
allowed-tools:
  - data_fetch
  - web_search
---

# Company Latest Developments（个股/项目最新进展）

当用户问"XX 最近有什么进展 / 最新动态 / 最近发生了什么 / 有什么新消息"时，
回答的骨架是**带日期的事件时间线**，而且每个关键事件要标注一手/二手来源。
不要只靠 news + web_search 拼凑：一手来源工具已经存在，必须先用。

## 取证顺序（在 search 定妥 symbol 之后）

1. `data_fetch sec_filings`：最近 90 天 SEC 申报索引（8-K/6-K/10-Q 等）。
   融资（可转债、增发）、并购、重大合同、管理层变动的**一手确认**在这里。
   引用时给出 formType + filingDate + 原文链接；模型记忆或媒体转述不能替代。
2. `data_fetch press_releases`：公司官方新闻稿。项目里程碑（开工、审批、投产）
   的公司口径以此为准。
3. `data_fetch analyst_actions`：带日期的评级/目标价动作流（grades +
   评级新闻 + 目标价新闻）。"机构怎么看"必须引用具体机构、日期、前后值，
   不要写"多家机构看好"这类无法核验的话。
4. `data_fetch news`（symbol 维度）：第三方报道补充时间线空档。
5. `web_search`：只用于以上一手来源覆盖不到的缺口——地方监管、社区听证、
   政府文件、行业媒体深挖（例如市政规划会、停工令、当地报纸）。搜索词带上
   具体地名/项目名和时间限定。

## 输出纪律

- 主体是时间线：`日期 — 事件 —（来源：8-K 链接 / 公司新闻稿 / 媒体名）`。
- 一手（filings、press releases）与二手（媒体、博客）明确分层；只有二手
  来源的关键结论要标注"未见一手确认"。
- 悬而未决的事项（如审批是否通过、禁令是否解除）明确写"截至检索时间未有
  一手确认"，并说明下一个可观察节点（下次听证日期、财报日等）。
- 股价归因必须对齐事件日期与行情窗口；无法对齐时明确说不能归因。
- 结尾按仓位视角给一句"对持有者意味着什么"，遵循 hari-invest 的决策纪律
  （若已加载）。
