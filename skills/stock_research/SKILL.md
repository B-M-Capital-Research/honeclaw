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
| `data_fetch(data_type="search", query="company name, alias, or ticker")` | Mandatory entity-resolution step before company/security analysis |
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

1. Resolve every named security with a current-turn `search` call. A ticker is a hint; names, aliases, Chinese names, multiple securities, and share classes must all produce explicit resolution results. Never take the first approximate result silently.
2. Verify the current-turn same-symbol `quote`; for deep research also fetch `profile`, `financials`, and `news`. Forward quarter/outlook questions additionally require a current earnings-calendar check.
3. A quote-only question may stay concise. A deep single-company, quarter-outlook, “can it take off”, fundamentals, valuation, or buyability question must use these nine numbered sections in order:
   1. Conclusion
   2. What the company is and how it makes money
   3. Moat and competitive barriers
   4. Industry position and key competitors
   5. Financial quality
   6. Valuation using at least two suitable methods with assumptions
   7. Bull / Bear / Base Case
   8. Catalysts, risks, and falsification conditions
   9. Action: buy / wait / reduce / sell / observe, with triggers
4. State the data timestamp and distinguish verified facts, inference, and action. Do not ask for the user's cost basis as a substitute for completing the analysis.
5. If required live evidence is missing or mismatched, stop numeric conclusions instead of filling gaps from memory, history, profiles, or another symbol.
6. If the user explicitly asks for a chart, trend line, comparison visual, or the answer would be materially clearer as a chart, hand off to `chart_visualization` with the concrete numbers you already fetched.

### Valuation Mode

1. Resolve the ticker first; do not attempt valuation without confirming the company
2. Fetch `financials`; add `quote` or `snapshot` if you also need current market context
3. Use `web_search` for the latest operating updates, guidance changes, or peer-comparison context
4. Use at least two suitable methods (for example P/S plus scenario analysis for a high-growth cloud company), show assumptions, and state which conditions would expand or compress the valuation
5. Do not collapse the result into a simplistic categorical verdict with no assumptions attached

### Screening Mode

1. Extract the user's explicit criteria before naming companies
2. Use `gainers_losers`, `sector_performance`, or targeted `web_search` to form an initial candidate set
3. Narrow the result to 3-5 names and fetch `snapshot` for each final candidate
4. Return a comparison shortlist with why each name matches the screen, plus the main risk or diligence gap for each one
5. Do not output a blunt recommendation list without comparison logic or caveats
