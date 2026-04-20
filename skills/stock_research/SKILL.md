---
name: Stock Research
description: Canonical Hone equity-research skill covering single-stock analysis, valuation framing, and criteria-based screening
when_to_use: Use when the user wants company research, valuation framing, or a small stock shortlist based on explicit criteria
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

This is the canonical equity-research entrypoint for Hone.

Use it for three closely related user intents:

1. Single-company research
2. Valuation framing for a named company
3. Criteria-based stock screening that returns a short comparison list

Prefer keeping these modes inside one skill so the model does not have to choose between overlapping prompt variants.

### Tool Guide

| Tool call | Purpose |
|---------|------|
| `data_fetch(data_type="snapshot", symbol="ticker")` | Recommended. Fetch a snapshot with price action plus company overview |
| `data_fetch(data_type="quote", symbol="ticker")` | Fetch detailed real-time quote data such as price, change, and volume |
| `data_fetch(data_type="profile", symbol="ticker")` | Fetch company details such as business description, industry, and CEO |
| `data_fetch(data_type="financials", symbol="ticker")` | Fetch financial statements or valuation-relevant fundamentals |
| `data_fetch(data_type="gainers_losers")` | Broader market scan when a screening request needs candidates |
| `data_fetch(data_type="sector_performance")` | Sector strength context for screening or relative positioning |
| `web_search(query="...")` | Search for news, analyst views, and recent events |

### Mode Selection

Choose the mode from the user's request before fetching data:

- **Research mode**: the user asks about one company, ticker, fundamentals, technicals, or recent developments
- **Valuation mode**: the user asks whether a company looks rich, cheap, stretched, fairly priced, or wants a valuation bridge / peer view
- **Screening mode**: the user asks for a shortlist that matches factors such as AI, dividend yield, value, growth, or momentum

### Research Mode

1. Identify the ticker mentioned by the user. If it is unclear, search first with `data_fetch(data_type="search", symbol="...")`
2. Call `snapshot` for the baseline data
3. Decide whether to add `web_search` for news or causes
4. Output a combined answer covering price action, fundamentals, recent events, and risks
5. If the user explicitly asks for a chart, trend line, comparison visual, or the answer would be materially clearer as a chart, hand off to `chart_visualization` with the concrete numbers you already fetched

### Valuation Mode

1. Resolve the ticker first; do not attempt valuation without confirming the company
2. Fetch `financials`; add `quote` or `snapshot` if you also need current market context
3. Use `web_search` for the latest operating updates, guidance changes, or peer-comparison context
4. Explain the valuation through assumptions, peer multiples, and business quality, and state which conditions would make the company look richer, more balanced, or more compelling relative to peers
5. Do not collapse the result into a simplistic categorical verdict with no assumptions attached

### Screening Mode

1. Extract the user's explicit criteria before naming companies
2. Use `gainers_losers`, `sector_performance`, or targeted `web_search` to form an initial candidate set
3. Narrow the result to 3-5 names and fetch `snapshot` for each final candidate
4. Return a comparison shortlist with why each name matches the screen, plus the main risk or diligence gap for each one
5. Do not output a blunt recommendation list without comparison logic or caveats
