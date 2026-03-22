---
name: Stock Research
description: Analyze a stock's fundamentals, technicals, and market sentiment
tools:
  - data_fetch
  - web_search
---

## Stock Research Skill

Use the right tool combination based on the user's original question. Prefer `snapshot` first to get both price action and company overview, then use `web_search` to supplement news and sentiment.

### Tool Guide

| Tool call | Purpose |
|---------|------|
| `data_fetch(data_type="snapshot", symbol="ticker")` | Recommended. Fetch a snapshot with price action plus company overview |
| `data_fetch(data_type="quote", symbol="ticker")` | Fetch detailed real-time quote data such as price, change, and volume |
| `data_fetch(data_type="profile", symbol="ticker")` | Fetch company details such as business description, industry, and CEO |
| `web_search(query="...")` | Search for news, analyst views, and recent events |

### Research Flow

1. Identify the ticker mentioned by the user. If it is unclear, search first with `data_fetch(data_type="search", symbol="...")`
2. Call `snapshot` for the baseline data
3. Decide whether to add `web_search` for news or causes
4. Output a combined answer covering price action, fundamentals, recent events, and risks
