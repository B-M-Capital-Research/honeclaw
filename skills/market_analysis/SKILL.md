---
name: Market Analysis
description: Analyze macroeconomics, policy trends, and industry momentum, then combine the result with market index data for a broader judgment
allowed-tools:
  - web_search
  - data_fetch
  - skill_tool
---

## Market Analysis Skill

Use the tools according to the user's question and combine macro data with market indices to provide a deeper market view.

This skill must always anchor the analysis to the current session time before making any macro judgment. Treat the session's current time and date as the source of truth for words such as "today", "latest", "this week", "tonight", or "just announced".

### Tool Guide

| Tool call | Purpose |
|---------|------|
| `web_search(query="...")` | Fetch macroeconomic data, policy interpretation, and industry developments |
| `data_fetch(data_type="market")` | Fetch major market index data such as the Dow Jones, Nasdaq, and S&P 500 |

### Analysis Framework

1. **Time anchor first**: explicitly identify the current Beijing date, and if needed the current hour and minute, before analyzing the question
2. **Query rewrite first**: convert relative-time user wording into an absolute-date search query before calling `web_search`
3. **Macro level**: interest rates, inflation, employment, and other economic indicators
4. **Policy level**: Federal Reserve actions, fiscal policy, and regulatory changes
5. **Industry level**: sector trends and capital flows
6. **Market sentiment**: VIX volatility and risk-on / risk-off behavior

### Mandatory Query-Rewrite Rules

1. If the user asks about macro data, policy headlines, geopolitical events, or uses relative time such as "today", "latest", "this week", or "just announced", first read the current session date.
2. Rewrite the search query into an absolute-time form before search. Do not search with ambiguous wording.
3. The rewritten query must include the exact year, month, and day. Add "latest", "today's release", or the event name only after the absolute date is present.
4. Example rewrite:
   User asks: "How was today's nonfarm payroll?"
   Search query should become: "2026-04-04 latest US nonfarm payroll release"
5. When the answer is time-sensitive, the first line of the final answer should state the current Beijing time before giving the judgment.

### Notes

- Always focus on the dimensions that match the user's question instead of speaking in broad generalities
- Be explicit about the data source, whether it came from `data_fetch` or `web_search`
- Separate hard facts from market expectations or opinions in the final answer
- For macro search, never issue a `web_search` query that omits the absolute date when the user intent is time-sensitive
- If the user asks for a trend, curve, distribution, or side-by-side visual and you already have the numbers, call `skill_tool(skill_name="chart_visualization", execute_script=true, ...)` instead of describing the chart only in prose
