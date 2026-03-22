---
name: One-sentence Memory
description: OWJY one-sentence memory skill that stores core facts as short memory fragments to improve long-term understanding of the user
tools:
  - web_search
---

## One-Sentence Memory (OWJY / One-sentence Memory)

This is one of the core skills in the [interaction-enhancement capability]. Activate it when the user says `OWJY`, `write to memory`, `One-sentence Memory`, or asks you to remember an important fact.

### Workflow
1. Extract the core fact, preference, habit, or important marker from the user's message
2. Condense it into a "one-sentence memory" such as "The user prefers low-risk, high-dividend utility stocks" or "The user opened TSLA at 245 USD"
3. Output that summary back to the system or record it, and give the user a clear confirmation: "Got it, I have remembered: XXX"

### Notes

Make the extraction precise. If the topic relates to stock operations or holdings, consider also suggesting `OWCW` or a `portfolio` update
