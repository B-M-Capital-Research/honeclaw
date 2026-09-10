---
name: Influencer Views
description: 把大V速报（Serenity/白毛 @aleabitoreddit、SemiAnalysis 等注册作者近期公开推文与文章，含中文翻译与 HONE 观点整理）作为「观点线索」接进个股与产业问答：用户问某只股票最近的新闻、消息、市场怎么看，或点名 Serenity/白毛/大V 时，先取 influencer_views，再把带作者、日期与原链的观点写成独立一段，与已核验事实分层。
when_to_use: 用户问某只股票或某个环节最近有什么新闻、消息、传闻、市场/大V/推特上怎么看，或直接问 Serenity/白毛/大V 最近说了什么、看多看空什么时使用。带日期的事件时间线走 company-latest-developments，异动归因走 market_analysis，倍数与合理价走 valuation-audit；本 Skill 只负责把大V近期观点当线索补进来。
user-invocable: true
context: inline
aliases:
  - 大V
  - 大V速报
  - 大V观点
  - 大V怎么看
  - 大V最近
  - 白毛
  - 白毛怎么看
  - 白毛速报
  - 白毛最近
  - Serenity
  - serenity怎么看
  - serenity最近
  - 推特怎么看
  - 推特上怎么说
  - X上怎么说
  - 网红观点
  - KOL观点
  - 大家怎么看
  - 市场怎么看
  - 最近有什么消息
  - 有什么消息
  - 有什么新闻
  - 最新消息
  - 最近的新闻
  - influencer
allowed-tools:
  - influencer_views
  - data_fetch
  - web_search
---

# Influencer Views（大V观点线索）

研究台的「大V速报」每十几分钟同步一次注册作者的公开内容：Serenity / 白毛
（@aleabitoreddit，AI 与半导体供应链一线观察者，中文翻译来自 aichainmap 的白毛速报页，
原文以 X 链接为准）和 SemiAnalysis 官方 feed。`influencer_views` 工具只读这份快照，
不联网、不调模型。**它给的是作者观点线索，不是事实，也不是 HONE 结论。**

## 什么时候取、怎么取

1. 用户问某只股票「最近有什么消息 / 新闻 / 市场怎么看 / 大V怎么看」，或直接点名
   Serenity / 白毛 / 大V：先 `influencer_views(symbol=<代码>)`。
2. 没命中就按它所在的环节再取一次：`influencer_views(query=<行业词>)`，例如
   `HBM`、`内存`、`光模块`、`CPO`、`Rubin`、`衬底`。
3. 用户问「白毛最近在看什么 / 最近关注什么」这类不带标的的问题：直接
   `influencer_views()`，读 `items` 与 `focus`（窗口内被点名最多的 ticker 与话题）。
4. 默认回看 14 天；用户明确问「这个月」再把 `days` 放到 30。窗口内没有命中，
   就如实写「大V近期没有直接提到 X」，不要拿别的公司的话顶替，也不要用搜索摘要
   或记忆里的「某某说过」来补。

这一步不替代本轮该做的事实取证：行情、财报、公告、新闻仍走 `data_fetch` /
`web_search`；`company-latest-developments` 的时间线、`market_analysis` 的归因
照常执行，本 Skill 只是在它们之后补一段。

## 输出纪律

- 独立成段，标题写清是谁的观点，例如 **「大V观点线索（Serenity / 白毛）」**；
  放在已核验事实与时间线之后、结论之前。不要把作者的话揉进「已核验事实」里。
- 每条一行：`MM-DD · 作者 · 一句话观点（原话或忠实压缩）· 立场（偏多/偏空/中性）·
  未证实处或反方 · [原文](X 链接)`。有 `hone_reading` 就用它的 summary / stance /
  counterpoint；没有就自己压缩原话，不要替作者补论据。
- 一般引用 2–4 条，最多 6 条；同一观点多次重复只留最新一条并注明「多次重申」。
- 观点与事实分层：作者说的「供给缺口 40–60%」是**作者转述**，除非本轮有一手来源，
  否则写「据作者转述，未核验」。作者观点与本轮已核验事实冲突时，以事实为准，并明确指出
  冲突在哪。
- 不把作者立场写成 HONE 判断，不据此给买卖、仓位或目标价；`hari-invest` 的机会 / 持有 /
  风险分档仍然只能由本轮证据推出来，这一段只作为「市场叙事在关注什么」的旁证。
- 说明翻译来源：中文译文来自 aichainmap 白毛速报，原文以 X 为准；作者观点仅供参考，
  不构成投资建议。这句话在一次回答里写一次即可。
- 不要泄露快照文件路径、工具内部字段名或 Skill 路径。

## 与其它 Skill 的配合

- `company-latest-developments`：时间线写完后，追加本段作为「市场叙事」补充。
- `stock_research` / `valuation-audit`：在「催化剂与风险」或「市场预期」段引用，
  不进入估值输入。
- `market_analysis`（为什么涨跌）：作者对该异动的解释只能作为「一种市场解释」，
  不是归因结论；归因仍要落到本轮来源与日期。
