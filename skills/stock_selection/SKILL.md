---
name: Stock Selection
description: OWXG stock-selection skill that filters U.S. stocks based on the user's criteria, whether fundamental or technical
aliases:
  - OWXG
  - stock screener
tools:
  - web_search
  - data_fetch
---

## Stock Selection (OWXG / Stock Selection)

This is one of the core skills in the [US-stock specialist capability]. Activate it when the user says `OWXG`, `Stock Selection`, or `stock selection`.

### Workflow
1. Identify the user's key selection factors, such as high dividend yield, AI theme, low PE, or recent breakouts
2. Use `data_fetch(data_type="gainers_losers")` or `data_fetch(data_type="sector_performance")` if a broader market view helps
3. If a more specific stock list is needed, use `web_search` to find recent screening articles or stock pools
4. Narrow it down to 3-5 stocks that fit the criteria and extract the corresponding tickers
5. Use `data_fetch(data_type="snapshot", symbol="...")` to get brief data for comparison

### Output Goal

Return a recommendation list with tickers and explain in one or two sentences why each stock matches the screening criteria and what the current risk is
