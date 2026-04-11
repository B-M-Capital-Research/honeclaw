---
name: Gold Analysis
description: Analyze gold, gold-linked ETFs, and gold miners through macro drivers, positioning, and event risk
aliases:
  - gold analysis
  - gold
  - precious metals
tools:
  - web_search
  - data_fetch
---

## Gold Analysis

Use this skill when the user asks about gold, gold-linked ETFs such as `GLD`, or gold miners and wants a structured market read rather than a generic headline summary.

### Workflow
1. Clarify whether the user means spot gold, a gold ETF, or an individual miner.
2. Use `web_search` to gather the latest macro drivers such as real yields, central-bank tone, U.S. dollar direction, inflation expectations, and geopolitical stress.
3. If the user asks about a tradable gold proxy such as `GLD`, `IAU`, or a miner ticker, use `data_fetch(data_type="snapshot", symbol="...")` for baseline price action and company or fund context.
4. Separate the answer into near-term catalysts, medium-term drivers, and the main invalidation risks.

### Output Goal

Deliver a concise research note that explains what is currently supporting or pressuring gold, what conditions would strengthen or weaken the thesis, and which indicators the user should keep monitoring. Keep the tone analytical and avoid direct buy or sell instructions.
