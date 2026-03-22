---
name: Portfolio Management
description: Manage user holdings, support buy, sell, rebalance, and query actions, and validate ticker symbols before any write operation
tools:
  - portfolio
  - data_fetch
---

## Portfolio Management Skill

Each user has an independent portfolio record. Before any write operation (`add` / `remove`), the ticker must be validated first.

### Ticker Validation Flow

Users often provide abbreviations, shorthand, or Chinese names, so **validation is mandatory**:

1. If the ticker is uncertain, call `data_fetch(data_type="search", symbol="...")` first
2. The search result returns the correct `symbol` and `name`
3. Use the validated ticker and company name for the actual action

**Examples:**
- User says "tem" -> `data_fetch(data_type="search", symbol="TEM")` -> Tempus AI Inc
- User says "NVIDIA" -> `data_fetch(data_type="search", symbol="NVDA")` or search "nvidia"
- User says "Tesla" -> the ticker is already known as TSLA and can be used directly

### Tool Guide

| Tool call | Purpose |
|---------|------|
| `data_fetch(data_type="search", symbol="...")` | Search or validate a ticker and get the correct ticker and company name |
| `portfolio(action="get")` | Get all current holdings |
| `portfolio(action="add", ticker="...", name="...", shares=..., cost_price=...)` | Add or update a holding |
| `portfolio(action="remove", ticker="...")` | Delete a holding |
| `portfolio(action="summary")` | Get a portfolio summary such as total value and P/L |

### Portfolio Workflow

1. Call `portfolio(action="get")` first to inspect the existing holdings
2. If the portfolio is empty, guide the user to add one:
   - Tell the user "You do not have any recorded holdings yet"
   - Ask them to describe it, for example: "Please tell me your holdings, such as: I own 100 shares of Apple at a cost of 175 USD"
3. When adding a holding: validate the ticker first, then add it with the correct ticker and company name
