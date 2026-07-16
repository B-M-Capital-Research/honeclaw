---
name: Market Analysis
description: Analyze macroeconomics, policy trends, and industry momentum, then combine the result with market index data for a broader judgment
allowed-tools:
  - web_search
  - data_fetch
  - skill_tool
---

## Market Analysis Skill

Use the tools according to the user's question and combine current-turn market quotes, macro evidence, and dated news to provide a deeper market view. This skill covers broad or regional markets and sector or industry themes; it must not fall back to generic chat merely because the request has no single ticker.

This skill must always anchor the analysis to the current session time before making any macro judgment. Treat the session's current time and date as the source of truth for words such as "today", "latest", "this week", "tonight", or "just announced".

### Tool Guide

| Tool call | Purpose |
|---------|------|
| `web_search(query="...")` | Fetch macroeconomic data, policy interpretation, and industry developments |
| `data_fetch(data_type="search", query="ticker or name")` | Resolve every representative index proxy, ETF, or listed company before using it |
| `data_fetch(data_type="quote", ticker="comma-separated exact symbols")` | Fetch same-symbol latest-available price, change, and provider timestamps for representative benchmarks |
| `data_fetch(data_type="sector_performance")` | Fetch current sector breadth when the requested scope supports it |
| `data_fetch(data_type="gainers_losers")` | Inspect current market leaders/laggards as breadth context, not as entity proof |

### Analysis Framework

1. **Time anchor first**: every market or sector answer starts with the server-provided Beijing data-time/quote-basis line; do not emit any preamble or duplicate model-authored time line before it
2. **Subject and entity discovery first**: identify every requested market scope; broad-market turns resolve representative benchmarks, while sector turns discover listed representatives from current theme evidence and exact-resolve at least three same-theme securities
3. **Current quote first**: fetch same-symbol quotes and provider timestamps for every representative before analyzing direction; never reuse prior assistant prices or let one market overwrite another in a mixed-scope request
4. **Query rewrite first**: convert relative-time wording into absolute market-local civil dates before calling `web_search`; keep Beijing time as the user-visible anchor
5. **Macro and policy**: interest rates, inflation, employment, central-bank, fiscal, and regulatory evidence only when relevant
6. **Industry and breadth**: sector trends, representative-company dispersion, and capital flows
7. **Fact/inference split**: dated source facts and causal inference must be visibly separate

### Broad / Regional Market Output Contract

Use exactly five substantive numbered sections after the server-owned time line:

1. Conclusion
2. Verified market facts: one independent line per representative, with exact symbol, current-turn price, change, and quote timestamp basis
3. Market-move reasons: dated verified events with a current-turn source domain, followed separately by causal inference; if evidence is insufficient, say `原因本轮未完全核验`
4. Bull / Bear / Base Case and primary risks
5. Action framing, triggers, and falsification conditions

Do not ask “which stock?” as a substitute for a broad-market answer. In mixed-market requests, retain separate local dates, benchmark entities, and evidence for every scope.

### Sector / Industry Output Contract

Use exactly nine substantive numbered sections after the server-owned time line:

1. What the technology or theme is
2. Its core change versus alternatives
3. Why it matters now and the adoption timeline
4. The next 2–3 years of market space and mainstream views; label unsupported numbers as not verified in the current turn
5. Value-chain layers and bargaining power
6. Listed-company comparison, with an independent current-turn same-symbol quote for every verified representative
7. High-certainty, high-beta, and concept-only mappings
8. Risks and falsification conditions, including Bull / Bear / Base scenarios and verifiable catalysts
9. Final investment framing and trigger conditions

Never use SPY/QQQ or a previous-turn ticker merely to fill a sector list. Every representative must be supported by current theme evidence plus exact-symbol search and quote results.

### Mandatory Query-Rewrite Rules

1. If the user asks about macro data, policy headlines, geopolitical events, or uses relative time such as "today", "latest", "this week", or "just announced", first read the current session date.
2. Rewrite the search query into an absolute-time form before search. Do not search with ambiguous wording.
3. The rewritten query must include the exact year, month, and day. Add "latest", "today's release", or the event name only after the absolute date is present.
4. Example rewrite:
   User asks: "How was today's nonfarm payroll?"
   Search query should become: "2026-04-04 latest US nonfarm payroll release"
5. The first visible line is always the server-owned Beijing data-time and quote-basis line for market and sector answers. Do not generate a second time line.

### Notes

- Always focus on the dimensions that match the user's question instead of speaking in broad generalities
- Use current-turn DataFetch quotes for market facts and dated `web_search` evidence for causes; do not expose raw tool names or payloads in the user-facing answer
- Separate hard facts from market expectations or opinions in the final answer
- For macro search, never issue a `web_search` query that omits the absolute date when the user intent is time-sensitive
- If quotes succeeded, never claim that Hone lacks real-time/current market data or did not request it. Describe the provider result as latest available and non-tick-by-tick.
- A failed news result does not erase valid quotes: report the verified prices and say the cause is not fully verified. A failed quote for one scope does not permit copying another scope's quote.
- If the user asks for a trend, curve, distribution, or side-by-side visual and you already have the numbers, call `skill_tool(skill_name="chart_visualization", execute_script=true, ...)` instead of describing the chart only in prose
