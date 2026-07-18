---
name: Stock Research
description: Canonical Hone security-research skill covering company and ETF/fund analysis, valuation framing, and criteria-based screening
when_to_use: Use when the user wants company or ETF/fund research, valuation framing, or a small security shortlist based on explicit criteria
user-invocable: true
context: inline
aliases:
  - stock research
  - valuation
  - stock screener
  - stock selection
  - OWGZ
  - OWXG
allowed-tools:
  - data_fetch
  - web_search
  - skill_tool
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

1. In the main agent loop, read the complete current user query and retain every possible named security before answering. Treat any pre-scanned ticker as a candidate seed, never as proof that the entity set is complete. Start every named-security request with one batch/parallel discovery round using `data_fetch(data_type="search", query="...")`; after those results return, use the next tool round for exact-symbol quote/profile. A plain ticker such as `NBIS`, `INTL`, or `RMBS` is normal user input: query it directly and require an exact-symbol result instead of asking the user to spell out the company. Only ask for clarification after current-turn tools still show genuine ambiguity or no authoritative coverage.
2. After identity is confirmed, fetch the same-symbol `quote` and preserve its provider timestamp. Never establish identity, price, change, financials, or news from assistant history or model memory.
3. Select the company, ETF/fund, or crypto route only from current-turn structured evidence. A named security takes precedence over broad market words in the same query.
4. Interactive final-answer ownership stays with the main Agent: complete one full final answer inside the current-turn tool loop. After success, the service will not append any user-visible content, rewrite the answer, rerun the main Agent, or reject that successful answer. The Agent itself must emit `数据时间：北京时间 YYYY-MM-DD HH:MM；行情口径：...` as the first visible line, using the current Beijing time from the Session context, the current-turn quote provider timestamp, and the latest-available/non-tick-by-tick basis. Include a market session only when a tool explicitly verified it; otherwise say it was not separately verified and never infer it from an ordinary quote timestamp. Do not emit a preamble before that line.
5. Use absolute-date `web_search` for current events, causes, policy, analyst context, customer/supplier relationships, contracts, purchase scale, or competitive claims. `data_fetch(search)` proves only the entity candidate and profile proves only the company's business description; neither proves a relationship or news causality. A search snippet may support only the limited fact it explicitly states; never expand it into an unstated contract change or cause, and disclose when full text or a primary source was not verified.
6. When a same-symbol quote succeeded, never claim that real-time/current market data was not requested, unavailable, or outside Hone's capability. Describe it accurately as the latest available provider quote, not tick-by-tick data.

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

### Valuation Mode

1. Resolve the ticker first, fetch the same-symbol quote, and read the exact-symbol `profile`; do not attempt valuation before confirming whether the entity is a company or an ETF/fund.
2. For a company, fetch `financials`; add `quote` or `snapshot` if you also need current market context. Use at least two suitable methods only when every numerator, denominator, period, and balance-sheet input is present. Annual FY revenue is not TTM. Without verified cash/debt or enterprise value, label market-cap/EBITDA as such and never call it EV/EBITDA. If only one method is fully supported, use that method, disclose the missing inputs, and do not invent net debt, historical multiples, target prices, or technical support levels to fill the template.
3. For an ETF/fund confirmed by `isEtf/isFund`, fetch `etf_holdings` plus `quote` and frame valuation through underlying holdings/exposures, fees, tracking error, concentration, and applicable portfolio-level multiples. Do not fetch corporate financials or an earnings calendar, and do not apply a single-company DCF to the fund itself.
4. Use `web_search` for the latest operating updates, strategy changes, holdings disclosures, guidance changes, or peer-comparison context appropriate to the confirmed asset type.
5. Do not collapse the result into a simplistic categorical verdict with no assumptions attached.

### Screening Mode

1. Extract the user's explicit criteria before naming companies
2. Use `gainers_losers`, `sector_performance`, or targeted `web_search` to form an initial candidate set
3. Narrow the result to 3-5 names, exact-resolve every final candidate, and fetch a same-symbol `snapshot` or `quote` for each
4. Return a comparison shortlist with why each name matches the screen, plus the main risk or diligence gap for each one
5. Do not output a blunt recommendation list without comparison logic or caveats
