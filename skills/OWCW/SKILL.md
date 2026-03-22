---
name: Position Advice
description: OWCW position advice skill that combines market conditions, stock-specific developments, and the user's current holdings to provide professional position-adjustment suggestions
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
3. Evaluate whether the position is too concentrated or whether the risk/reward is out of balance, then give a specific recommendation such as rebalance, trim, add, or hold.

### Output Goal

Give actionable, explicit advice: which stock should be trimmed on strength, which one should be held patiently, and whether the overall position size should be reduced.
