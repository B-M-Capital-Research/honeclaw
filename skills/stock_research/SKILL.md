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

1. In the main agent loop, read the complete current user query and retain every possible named security before answering. Treat any pre-scanned ticker as a candidate seed, never as proof that the entity set is complete. Do not rely on a service-side grammar that splits natural language on commas, `and`, `和`, `&`, or `/`; names and symbols such as `AT&T`, `S&P Global`, `BRK/B`, and `Berkshire Hathaway, Class B` must remain understandable in context. Assign each named security one stable, distinct, case-sensitive `entity_route` key for this run and reuse it verbatim. Start with one batch/parallel discovery round using one separate `data_fetch(data_type="search", query="...", entity_route="...", identity_match="...")` call per named security; this tool-enabled round returns only tool calls, never a data-time line, summary, draft, or final prose. Calls may run in parallel, but never combine multiple securities into one query. `identity_match` is call-scoped and must be present on every search: set `identity_match="exact_symbol"` when the query is a ticker and `identity_match="name_or_alias"` when it is a company name, Chinese name, or alias. A previous declaration does not authorize a later search, and the service must not guess from case, length, or punctuation. Reuse the exact same route key on that security's refinement, quote, profile/snapshot, and later DataFetch calls. An exact-symbol route keeps its symbol constraint even during a later company-name refinement, with only bounded provider separator equivalence such as `BRK/B`, `BRK-B`, and `BRK.B`; an ETF or product whose name merely embeds CRWV cannot replace CRWV. The key is internal evidence linkage, not a user-visible entity claim. If a Chinese name or alias search is empty, refine it within the same route and use `refines_query` to copy the original empty query verbatim with matching case. If an earlier search omitted the route key, repeat its exact query or set `supersedes_query` to that old query verbatim with matching case so at most that one provisional route is migrated; never guess alias equivalence or erase another entity's gap. `refines_query` and `supersedes_query` are strictly mutually exclusive: provide at most one on any search, because providing both invalidates that identity search and leaves the route pending. After search results return, select one standard symbol per route and give that same symbol both quote and profile/asset-route coverage. A plain ticker such as `NBIS`, `INTL`, `RMBS`, or `CRWV` is normal user input: query it directly instead of asking for a company name. Only ask for clarification after current-turn tools still show genuine ambiguity or no authoritative coverage.
2. After identity is confirmed, fetch the same-symbol `quote` and preserve its provider timestamp. Never establish identity, price, change, financials, or news from assistant history or model memory.
3. Select the company, ETF/fund, or crypto route only from current-turn structured evidence. A named security takes precedence over broad market words in the same query.
4. Interactive final-answer ownership stays with the main Agent. While business tools are present, return only additional tool calls or, once the complete original question and current tool results have been rechecked, the sole structured `finish_research` handoff; do not compose final prose in a tool-enabled round. The handoff contains `answer_scope`, `facts`, `inferences`, and `gaps`. A fact contains only its ID and current-turn evidence references: Web evidence uses `tool_call_id`, a 1-based result number, and a verbatim excerpt; DataFetch evidence uses `tool_call_id` and an RFC 6901 JSON Pointer. Hone resolves the actual Web title/URL or structured scalar field, drops bad references without rejecting or rerunning the answer, and never semantically reviews the final prose. Inferences reference submitted fact IDs; unresolved dimensions remain gaps—absence is never a negative fact. The following no-tool completion is the same Agent's one visible answer, and the service will not append, rewrite, rerun, or reject it. The Agent itself must emit `数据时间：北京时间 YYYY-MM-DD HH:MM；行情口径：...` as the first visible line, using the current Beijing time from the Session context. Quote time, price, and market basis must come from resolved current-turn quote fields; include a market session only when a tool explicitly verified it, and never infer one from an ordinary quote timestamp. Do not emit a preamble before that line.
5. Use absolute-date `web_search` for current events, causes, policy, analyst context, customer/supplier relationships, contracts, purchase scale, ownership, or competitive claims. `data_fetch(search)` proves only the entity candidate and profile proves only the company's business description; neither proves a relationship or news causality. For a broad “A and B relationship” question, let the Agent derive relevant axes from the complete semantics; normally investigate commercial/customer-supplier/technology-contract and investment/ownership separately, preferably in parallel and through SEC, company IR, or both parties' announcements. One generic query is not complete research. A search snippet may support only the limited fact it explicitly states; never expand it into an unstated contract change or cause, and disclose when full text or a primary source was not verified.
   Before finalizing, select the exact evidence snippets/fields needed for the structured ledger. Every relationship fact's number, direction, rank, role, right/obligation, product model, and valuation label must occur directly in resolved current-turn evidence. The service-injected URL locates a source but does not prove unsupported text. Put any judgment beyond literal sources in `inferences`, then render it as a separate sentence beginning `Inference:`; delete details that cannot be paired this way. Relationship answers should be minimal and on-scope rather than filling an unrelated deep-company template.
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
