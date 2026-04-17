---
name: Professional Valuation
description: OWGZ professional valuation skill for analyzing a company's financial data and market performance
aliases:
  - OWGZ
  - valuation
allowed-tools:
  - web_search
  - data_fetch
  - skill_tool
---

## Professional Valuation (OWGZ / Professional Valuation)

This is one of the core skills in the [US-stock specialist capability]. Activate it when the user says `OWGZ`, `Professional Valuation`, or `valuation`.

### Workflow
1. Fetch the company's financial statements, market cap, PE ratio, and similar data with `data_fetch(data_type="financials", ticker="...")`; add `quote` if needed
2. Use `web_search` to fetch the latest operating updates or Wall Street research summaries
3. Combine macroeconomics and industry context to produce a professional valuation summary, including a brief DCF view and relative-valuation comparison, and then explain which assumptions would make the stock look richer, more balanced, or more compelling relative to peers
4. If the user asks for a valuation bridge, peer multiple comparison chart, or any visual trend, call `chart_visualization` with the exact data instead of only describing it verbally

### Note

The answer should sound highly professional, avoid casual wording, use finance terminology, and still remain logically clear
