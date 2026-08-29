---
name: Stock Research
description: Canonical Hone security-research skill covering company and ETF/fund analysis, valuation framing, and criteria-based screening
when_to_use: Use when the user wants company or ETF/fund research, valuation framing, or a small security shortlist based on explicit criteria。宽泛提问触发词：分析、研究、了解、看看、值不值
user-invocable: true
context: inline
aliases:
  - stock research
  - valuation
  - stock screener
  - stock selection
  - OWGZ
  - OWXG
  - 帮我分析
  - 分析一下
  - 分析下
  - 详细分析
  - 怎么看
  - 怎么样
  - 值得买吗
  - 建仓吗
  - 投资机会
  - 公司分析
  - 个股分析
  - 基本面分析
  - 技术分析
  - 技术能力
  - 介绍一下
  - 看下
  - 增长空间
allowed-tools:
  - data_fetch
  - web_search
---

## Stock Research Skill

This is the canonical security-research entrypoint for Hone.

Use it for three closely related user intents:

1. Single-company or ETF/fund research
2. Valuation framing for a named company
3. Criteria-based stock screening that returns a short comparison list

Prefer keeping these modes inside one skill so the model does not have to choose between overlapping prompt variants.

### Tool Guide

| Tool call | Purpose |
|---------|------|
| `data_fetch(data_type="search", query="company name, alias, or ticker")` | Mandatory entity-resolution step before company/security analysis |
| `data_fetch(data_type="snapshot", ticker="ticker")` | Recommended. Fetch a snapshot with price action plus company overview |
| `data_fetch(data_type="earnings_outlook", ticker="ticker")` | Preferred for a named security's earnings preview; includes quote, profile, earnings, estimates, targets, ratings, financials, coverage, and current-listing evidence |
| `data_fetch(data_type="quote", ticker="ticker")` | Fetch detailed latest-available quote data such as price, change, volume, and provider timestamp |
| `data_fetch(data_type="profile", ticker="ticker")` | Fetch company details such as business description, industry, and CEO |
| `data_fetch(data_type="financials", ticker="ticker")` | Fetch financial statements or valuation-relevant fundamentals |
| `data_fetch(data_type="etf_holdings", ticker="ticker")` | Fetch ETF/fund holdings after profile confirms `isEtf` or `isFund` |
| `data_fetch(data_type="news", ticker="ticker")` | Fetch current-turn news for the exact security |
| `data_fetch(data_type="gainers_losers")` | Broader market scan when a screening request needs candidates |
| `data_fetch(data_type="sector_performance")` | Sector strength context for screening or relative positioning |
| `web_search(query="...")` | Search for news, analyst views, and recent events |

### Adapt To The Requested Outcome

Read the complete request and choose the evidence and answer shape that best fits it; these are reusable answer patterns, not a closed intent classifier or a grammar that the user's wording must match:

- **Research mode**: the user asks about one company, ETF/fund, ticker, fundamentals, technicals, or recent developments
- **Valuation mode**: the user asks whether a company looks rich, cheap, stretched, fairly priced, or wants a valuation bridge / peer view
- **Screening mode**: the user asks for a shortlist that matches factors such as AI, dividend yield, value, growth, or momentum

### Non-negotiable Current-turn Pipeline

1. In the main agent loop, read the complete current user query and retain every possible named security before answering. Treat any pre-scanned ticker as a candidate seed, never as proof that the entity set is complete. Do not rely on a service-side grammar that splits natural language on commas, `and`, `和`, `&`, or `/`; names and symbols such as `AT&T`, `S&P Global`, `BRK/B`, and `Berkshire Hathaway, Class B` must remain understandable in context. Assign each named security one stable, distinct, case-sensitive `entity_route` key for this run and reuse it verbatim. Start with one batch/parallel discovery round using one separate `data_fetch(data_type="search", query="...", entity_route="...", identity_match="...")` call per named security; this tool-enabled round returns only tool calls, never a data-time line, summary, draft, or final prose. Calls may run in parallel, but never combine multiple securities into one query. `identity_match` is call-scoped and must be present on every search: set `identity_match="exact_symbol"` when the query is a ticker and `identity_match="name_or_alias"` when it is a company name, Chinese name, or alias. A previous declaration does not authorize a later search, and the service must not guess from case, length, or punctuation. Reuse the exact same route key on that security's refinement, quote, profile/snapshot, and later DataFetch calls. An exact-symbol route keeps its symbol constraint even during a later company-name refinement, with only bounded provider separator equivalence such as `BRK/B`, `BRK-B`, and `BRK.B`; an ETF or product whose name merely embeds CRWV cannot replace CRWV. The key is internal evidence linkage, not a user-visible entity claim. If a Chinese name or alias search is empty, refine it within the same route and use `refines_query` to copy the original empty query verbatim with matching case. If an earlier search omitted the route key, repeat its exact query or set `supersedes_query` to that old query verbatim with matching case so at most that one provisional route is migrated; never guess alias equivalence or erase another entity's gap. `refines_query` and `supersedes_query` are strictly mutually exclusive: provide at most one on any search, because providing both invalidates that identity search and leaves the route pending. After search results return, select one standard symbol per route and give that same symbol both quote and profile/asset-route coverage. A plain ticker such as `NBIS`, `INTL`, `RMBS`, or `CRWV` is normal user input: query it directly instead of asking for a company name. Only ask for clarification after current-turn tools still show genuine ambiguity or no authoritative coverage.
2. After identity is confirmed, fetch the same-symbol `quote` and preserve its provider timestamp. Never establish identity, price, change, financials, or news from assistant history or model memory.
3. Select the company, ETF/fund, or crypto route only from current-turn structured evidence. A named security takes precedence over broad market words in the same query.
   When `hone_security_listing_evidence.status=active_listing`, treat the same-symbol security as currently listed and trading. Never override it with model memory about an earlier acquisition, delisting, or former parent, and never redirect the user to that former parent. If current regulatory evidence conflicts with the provider, fetch and disclose the authoritative conflict instead of deciding from memory.
4. Interactive final-answer ownership stays with the main Agent and the ordinary function-calling loop. Current-turn business-tool results remain directly in the same Agent context. Before the entity/evidence floor is complete, return only the missing real tool calls. After that floor, either call another real business tool when the user's actual question still lacks key evidence, or return one complete natural `Stop + Done` final answer. Tool-enabled rounds remain buffered because a provider can emit prose before a later tool call; discard that preamble when tools appear, and publish/persist only the one completed DirectFinal. The Agent itself must emit `数据时间：北京时间 YYYY-MM-DD HH:MM；行情口径：...` as the first visible line, using the current Beijing time from the Session context. Quote time, price, and market basis must come from current-turn quote fields. Prefer `hone_quote_time.beijing`; `market_date_new_york` and `new_york` only represent New York local date/time and never prove an exchange, a market session, or a closing price. Take the exchange only from `exchange` / `exchangeShortName`, include a session only when a tool explicitly verified it, and do not emit a preamble before the first line.
5. Use absolute-date `web_search` for current events, causes, policy, analyst context, customer/supplier relationships, contracts, purchase scale, ownership, or competitive claims. `data_fetch(search)` proves only the entity candidate and profile proves only the company's business description; neither proves a relationship or news causality. For a broad “A and B relationship” question, let the Agent derive relevant axes from the complete semantics; normally investigate commercial/customer-supplier/technology-contract and investment/ownership separately, preferably in parallel and through SEC, company IR, or both parties' announcements. One generic query is not complete research. A search snippet may support only the limited fact it explicitly states; never expand it into an unstated contract change or cause, and disclose when full text or a primary source was not verified.
   Before finalizing, reread the exact current-turn evidence snippets/fields. Every relationship fact's number, direction, rank, role, right/obligation, product model, and valuation label must occur directly in current-turn evidence. A URL locates a source but does not prove unsupported text. Render any judgment beyond literal sources as a separate sentence beginning `Inference:`; when the premises are insufficient, delete it. In particular, do not summarize a relationship as core/largest, a major customer, highly dependent, locked in, or multiply bound unless the current source text directly supports that strength and direction. Relationship answers should be minimal and on-scope rather than filling an unrelated deep-company template.
6. When a same-symbol quote succeeded, never claim that real-time/current market data was not requested, unavailable, or outside Hone's capability. Describe it accurately as the latest available provider quote, not tick-by-tick data.

### Evidence Floor — What The First Line Is Allowed To Claim

The `数据时间 / 行情口径` first line is a claim about **this turn's** tool results, not a formatting ritual. Before writing it, check what this turn actually returned:

- **A quote/snapshot returned for this symbol** → write the normal quote basis, and name the symbol plus the provider time you are quoting (`hone_quote_time.beijing`). Every price, change, range, market cap, and multiple in the answer must trace to that payload.
- **No quote returned this turn** (nothing was called, every call failed, or only a skill/file/registry tool ran) → the first line must be `数据时间：北京时间 YYYY-MM-DD HH:MM；行情口径：本轮未取到行情`, and the whole answer must then contain **no** price, change percentage, intraday range, market cap, PE/PS/EV multiple, quarterly financial figure, or price band. Answer with the framework, the falsification conditions, and what you would need to check — that is a complete, acceptable answer.

A failed or missing skill load, a local-file listing, and a company-profile read are **not** market evidence. If loading a skill fails, keep going with the tools you do have; never treat the failed load as the turn's research step.

Three claims are only permitted when this turn's tools actually produced them, and each is a fabrication otherwise: **「本轮已核验 / 已取得」** (needs a returned payload), **「本轮检索到」plus a URL** (needs a `web_search` result this turn — a `snapshot` never returns article URLs), and **「根据 SEC 8-K / 公司公告原文」** (needs `sec_filings`, `press_releases`, or a search result you actually read).

Reusing an earlier turn's quote is allowed only as history: label it with its original date (`8/8 分析时价格 $82.10`). Never relabel it as the current price, and never restamp it with the current time.

### Research Mode

1. Resolve every named security discovered from the complete current query with current-turn tools, preferably in one batch/parallel first round. A ticker is a first-class search input but becomes an entity only after exact-symbol confirmation; names, aliases, Chinese names, multiple securities, and share classes must all produce explicit resolution results. A pre-scan miss must fall through to this agent loop, not become a user-facing failure. Never take the first approximate result silently, and clarify only when tool evidence remains genuinely ambiguous.
2. Verify the current-turn same-symbol `quote`, then select the route from structured exact-symbol evidence. A company uses `profile`, `financials`, and `news`; an ETF/fund confirmed by profile `isEtf/isFund` uses `etf_holdings` and `news`; a crypto asset confirmed by exact search market evidence such as `exchangeShortName=CRYPTO` uses the same-symbol quote and relevant news. Never request corporate financials or an earnings calendar for a confirmed ETF/fund, and never request corporate financials, an earnings calendar, or ETF holdings for crypto. Treat provider errors separately from a successful empty response. Do not infer an asset type from an empty response.
3. A quote-only question may stay concise. A deep single-company, quarter-outlook, “can it take off”, fundamentals, valuation, or buyability question must use these nine numbered sections in order:
   1. Conclusion
   2. What the company is and how it makes money
   3. Moat and competitive barriers
   4. Industry position and key competitors
   5. Financial quality
   6. Valuation using at least two suitable methods only when the current-turn inputs are complete; otherwise use the method that can be calculated rigorously and state the missing inputs
   7. Bull / Bear / Base Case
   8. Catalysts, risks, and falsification conditions
   9. Action: buy / wait / reduce / sell / observe, with triggers
4. Preserve the Agent-authored first-line data timestamp and quote basis, and distinguish verified facts, inference, conclusion, and action. Do not ask for the user's cost basis as a substitute for completing the analysis.
5. If required live evidence is missing or mismatched, stop numeric conclusions instead of filling gaps from memory, history, profiles, or another symbol.
6. If the user explicitly asks for a chart, trend line, comparison visual, or the answer would be materially clearer as a chart, hand off to `chart_visualization` with the concrete numbers you already fetched.

If the exact quote and profile are valid but current company financial statements are empty, failed, mismatched, or limited to an income statement, do not fail the whole response and do not fabricate values. Keep all nine sections, state `本轮公司财务数据未核验` in section 5 with the exact missing scope, and base the remaining sections only on verified quote/profile/news evidence. An income statement does not prove cash, debt, net debt, or free cash flow. Financial-data absence must never be rewritten as an absence of current quote capability.

### ETF / Fund Research Route

When the exact-symbol profile confirms `isEtf=true` or `isFund=true`, use these nine numbered sections instead of the company template:

1. Conclusion
2. Fund objective, strategy, and tracked exposure
3. Holdings, concentration, and primary exposures
4. Geographic, sector, and currency risk
5. Liquidity, fund size, and trading characteristics
6. Fees, tracking error, and underlying-asset valuation framing
7. Bull / Bear / Base Case
8. Catalysts, risks, and falsification conditions
9. Action: buy / wait / reduce / sell / observe, with triggers

Preserve the Agent-authored first-line data timestamp and quote basis, and separate verified facts from inference and action. If holdings, fees, size, or tracking-error evidence is absent, label that item as not verified in the current turn; do not fill it from memory. A successful empty company financial response for a confirmed ETF/fund is not a provider outage and must not block this route.

### Crypto Research Route

Only classify crypto from exact-symbol structured market evidence such as `exchangeShortName=CRYPTO`; do not infer it from a `USD` suffix. A confirmed crypto asset uses quote and relevant news, not stock profile, company financials, an earnings calendar, or ETF holdings. Use nine substantive numbered sections: conclusion with verified current price; asset/network/use case; supply/tokenomics/concentration; adoption/liquidity/market structure; on-chain/network/ecosystem evidence; valuation framework and assumptions; Bull/Bear/Base; catalysts/regulation/risks/falsification; and an action with trigger conditions. Label absent on-chain, supply, or ecosystem evidence as not verified in the current turn.

### 深度个股分析的收口纪律

适用范围：走上面 Research Mode 第 3 条九段模板的深度个股问题（`帮我分析一下 X`、`X 怎么看`、`X 还有增长空间吗`、`X 值得现在建仓吗`、`X 和 Y 怎么样`）。行情速查、ETF/基金、加密、纯新闻和宏观问题不适用。

四档区间（机会区 / 持有区 / 风险区 / 数据不足）、置信度分级、「只要证据足以区分就必须选边」、稀缺性与差异化的判断口径，都在 `hari-invest` 的「判断主干」和 `references/decision-rubric.md` 里；「首行之后第一段直接给结论」的顺序由运行时的 Hari Invest 框架规定；估值方法选择、对账表、三情景算式与反向估值在 `valuation-audit`。这些不在这里重复，需要时直接照那边的口径做。这一节只补那边没写的两件事：**结论必须落到算出来的价格区间**，以及**结论之前必须先点出胜负手**。

#### 0. 哪一维要展开时，加载哪个 skill

九段不是九段各写两行就算完。某一维需要真正展开时，加载对应的视角 skill（函数调用运行时用
`skill_tool(skill_name="...")`），它们各自规定了取哪些字段、判定锚点和产出格式，这里不重复：

| 九段里的哪一段 | 加载 | 它负责什么 |
|---|---|---|
| 结论之前的胜负手 | `first-principles` | 需求量 × 单位用量 × 单价 → 供给约束 → 公司拿到多少，以及证伪句 |
| 3 护城河与竞争壁垒 | `moat` | 壁垒分型、每型的取数字段、两项数字底线、护城河到倍数上下沿的传导 |
| 5 财务质量 | `fundamentals` | 收入结构、盈利质量、偿债、增长来源四块，每块要有带方向和后果的判断 |
| 稀缺度与差异化 | `scarcity-differentiation` | 1–5 打分锚点、每分要挂什么证据、这两个分把倍数往哪调 |
| 6 估值 | `valuation-audit` | 先定类型 → 找同类型对标 → 稀缺度调倍数 → 交叉验证 → 合理价区间 |
| 机构口径（评级、目标价） | `analyst-coverage` | 卖方分布家数、目标价低/中/高、近期评级动作与迁移方向 |

只在这一维确实是本轮问题的重心时加载。用户问「现在多少钱」不需要拉起六个 skill。

#### 1. 先点胜负手，再展开

在数据时间首行与「结论：」之后的第一段里，用一到两句写清楚：**这家公司当下哪一个变量的走向，直接决定这次判断对错**。九段的其余部分都围绕它取证——护城河、财务质量、估值、Bull/Bear 都要回答这个变量现在走到哪一步、还差什么才算兑现。

写法是「这次判断的成败取决于 X；X 往 A 走则……，往 B 走则……」，不是「公司有 A、B、C 三块业务，面临 D、E 两个风险」。把业务板块、驱动因素、风险点平铺成并列清单，通篇不指出哪一个是决定性的，视为未完成——这是评测里多次出现的「逻辑顺序对，但没抓住核心」。

评测里被点名漏掉的胜负手，作为标定用的真实例子：

| 标的 | 胜负手 | 平铺式回答漏掉了什么 |
|---|---|---|
| 腾讯 | 传统互联网护城河极深，但 AI 落后、Capex 明显抬升——这才是估值被压低、股价下跌的原因 | 按基本面→护城河→财务→估值铺完，没说清低估值正是市场对「AI 落后 + Capex 抬升」的定价，也就找不到合理倍数 |
| Credo / Lumentum / AAOI | 光模块与 AEC 现在都是产能问题（AAOI 另加高债务 + 高承诺），公司 earnings call 里就有 | 护城河与稀缺性只做了定性评价，没落到份额、毛利、积压订单和产能 |
| 零跑 | 行业极卷、产业稀缺性差、公司差异化不强——这本身就是结论 | 只做了财报复盘和情景估值，没有把行业稀缺度与公司差异化接进持仓建议 |

胜负手必须由本轮证据推出，不是从这张表里挑现成答案。表上没有的公司，自己按「哪一个变量一变、结论就得改」去定。

#### 2. 结论必须给出算出来的价格区间

深度个股分析的结论段必须同时出现三样东西：

1. **四态之一 + 置信度**（口径见 `hari-invest`）；
2. **一个由本轮数字推出的合理股价区间（每股口径），带算式**——写成「下沿 = 分母 × 倍数 = $A，上沿 = 分母 × 倍数 = $B」，$A/$B 是每股价格；算式走市值或 EV 口径时，要再除以本轮稀释股本折成每股才算完。分母注明期间、口径（TTM / FY1E / FY2E、GAAP / Non-GAAP）和性质（历史 actual / 公司指引 / 一致预期 / 自设假设）；倍数注明来自哪里（公司历史中枢、可比公司、正常化利润对应的倍数）。算式格式与情景拆法沿用 `valuation-audit` 第四步，不另立一套；
3. **当前价相对该区间的位置**——写成「现价 $P，位于区间约 X% 分位 / 低于下沿 Y% / 高于上沿 Z%」，用的价格必须是本轮 quote 的那一个（同一 as-of 基准只允许一个价格）。

三者缺一，这篇就没有结论。「当前评级：持有区（中置信度）」后面接一串 TTM P/E、Forward P/E，只是分档标签加倍数罗列——评测里这样写的九篇个股分析被逐条判为「没有结论」。倍数是中间量，用户要的是把倍数乘完之后的价格，以及现价站在这个价格带的哪个位置。

**输入不足时怎么写**：不要退回「无法判断」。显式写出缺的是哪一项（例如「缺 FY2 一致预期 EPS」「缺净债务，EV 口径算不出来」「未取得长协覆盖比例，正常化力度定不了」），再写出**补上后结论会怎么变**：「若 FY2E EPS 站上 X，按 18x 对应 $A，现价即进入机会区；低于 X 则维持持有区」。缺项 + 补齐后的分档走向，本身就是一个合格的结论。

第 6 段自己就要把区间算到每股并标出现价位置，结论段引用的是同一个区间、同一个现价，不是第 6 段列一串倍数、结论段另起一个数。第 6 段里的悲观/基准/乐观三行算式是全篇唯一的价格来源：结论段的区间下沿逐字取悲观那行的结果、上沿逐字取乐观那行的结果，Bull / Bear 段里出现的价位也必须是同样这三个数。三处对不上，读者拿到的是三个互相打架的锚，等于没有结论。第 6 段只有一种方法能严格算出时，按 `valuation-audit` 的口径用那一种把区间算完并披露缺项。只给单点目标价不给区间，或给了区间却不说现价在哪，都算没收口。

#### 3. Bull / Bear 要落到数字链

第 7 段的 Bull 与 Bear 各自至少要落到「兑现／证伪后大致是多少产能 → 多少收入 → 多少估值」这条链上的**至少一环具体数字**，并说明它把第 2 条的价格区间推向哪一侧。

示例（AAOI 增发那轮点评要的推演）：Bull——若 6 亿美元 ATM 全部投入产能扩张、且新增产能按公司披露的单位经济兑现，对应约 X 万只/月的 800G 出货、约 $Y 亿年化收入，按 Z 倍 PS 对应市值 $W，即区间上沿；Bear——若稀释后产能爬坡延后一年、债务成本继续上行，收入停在 $Y′ 亿，同一倍数下对应 $W′，即区间下沿甚至以下。

两边都只写「AI 需求强劲、份额有望提升」对「竞争加剧、毛利承压」，通篇不带数字，视为未完成。某一环的数字确实拿不到时，写清楚缺的是哪个量（产能、单价/ASP、合约覆盖比例、客户订单、backlog），不要用形容词补位。

#### 4. 「X 技术分析」默认指技术能力

用户说「X 技术分析」「分析一下 X 的技术」时，默认理解为**技术能力 / 技术路线分析**，归入基本面来做：产品代际与路线图、良率与制程、专利与认证壁垒、客户验证进度、与竞品的技术代差、研发投入与产出效率。不要默认展开均线、支撑位、阻力位、量价形态这类 K 线走势分析。

只有用户明确写了 K 线、走势、均线、支撑位、阻力位、技术面、形态、MACD/RSI 这类词，才做价格走势分析。两种理解都说得通而问题又很短时，按技术能力回答，并在结尾用一句话问是否也要看价格走势。技术能力这一维回答完，仍要按上面第 2 条把合理价格区间与现价所处位置一并给出——技术尽调不替代结论，全文零个价格、零个倍数的回答不算答完。

### Valuation Mode

1. Resolve the ticker first, fetch the same-symbol quote, and read the exact-symbol `profile`; do not attempt valuation before confirming whether the entity is a company or an ETF/fund.
2. For a company, fetch `financials`; add `quote` or `snapshot` if you also need current market context. Use at least two suitable methods only when every numerator, denominator, period, and balance-sheet input is present. Annual FY revenue is not TTM. Without verified cash/debt or enterprise value, label market-cap/EBITDA as such and never call it EV/EBITDA. If only one method is fully supported, use that method, disclose the missing inputs, and do not invent net debt, historical multiples, target prices, or technical support levels to fill the template. "Only one method" applies only when that denominator is genuinely unavailable this turn (see `valuation-audit`).
3. For an ETF/fund confirmed by `isEtf/isFund`, fetch `etf_holdings` plus `quote` and frame valuation through underlying holdings/exposures, fees, tracking error, concentration, and applicable portfolio-level multiples. Do not fetch corporate financials or an earnings calendar, and do not apply a single-company DCF to the fund itself.
4. Use `web_search` for the latest operating updates, strategy changes, holdings disclosures, guidance changes, or peer-comparison context appropriate to the confirmed asset type.
5. Do not collapse the result into a simplistic categorical verdict with no assumptions attached.

### Screening Mode

1. Extract the user's explicit criteria before naming companies
2. Use `gainers_losers`, `sector_performance`, or targeted `web_search` to form an initial candidate set
3. Narrow the result to 3-5 names, exact-resolve every final candidate, and fetch a same-symbol `snapshot` or `quote` for each
4. Return a comparison shortlist with why each name matches the screen, plus the main risk or diligence gap for each one
5. Do not output a blunt recommendation list without comparison logic or caveats
