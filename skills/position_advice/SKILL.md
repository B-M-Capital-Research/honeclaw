---
name: Position Advice
description: OWCW position advice skill that combines market conditions, stock-specific developments, and the user's current holdings to provide professional position-adjustment suggestions
aliases:
  - OWCW
  - rebalance advice
tools:
  - portfolio
  - web_search
  - data_fetch
---

## Position Advice (OWCW / Position Advice)

This is one of the core skills in the [US-stock specialist capability]. Activate it when the user says `OWCW`, `Position Advice`, or `position advice`.

### Workflow
1. Use `portfolio(action="get")` first to fetch the user's current holdings.
2. Combine `data_fetch(data_type="sector_performance")` with current sector strength, or use `web_search` directly on the user's concentrated holdings to find risk notes.
3. Evaluate concentration, liquidity, catalyst exposure, and downside scenarios, then explain what would need to happen for the user to consider reducing, maintaining, or restructuring exposure.

### Output Goal

Provide a risk-management oriented assessment: where the portfolio is concentrated, which names carry elevated event risk, what trigger conditions deserve attention, and what position-sizing or hedging questions the user should review before making any change.
