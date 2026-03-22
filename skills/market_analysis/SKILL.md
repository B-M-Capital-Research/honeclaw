---
name: Market Analysis
description: Analyze macroeconomics, policy trends, and industry momentum, then combine the result with market index data for a broader judgment
tools:
  - web_search
  - data_fetch
---

## Market Analysis Skill

Use the tools according to the user's question and combine macro data with market indices to provide a deeper market view.

### Tool Guide

| Tool call | Purpose |
|---------|------|
| `web_search(query="...")` | Fetch macroeconomic data, policy interpretation, and industry developments |
| `data_fetch(data_type="market")` | Fetch major market index data such as the Dow Jones, Nasdaq, and S&P 500 |

### Analysis Framework

1. **Macro level**: interest rates, inflation, employment, and other economic indicators
2. **Policy level**: Federal Reserve actions, fiscal policy, and regulatory changes
3. **Industry level**: sector trends and capital flows
4. **Market sentiment**: VIX volatility and risk-on / risk-off behavior

### Notes

- Always focus on the dimensions that match the user's question instead of speaking in broad generalities
- Be explicit about the data source, whether it came from `data_fetch` or `web_search`
- Separate hard facts from market expectations or opinions in the final answer
