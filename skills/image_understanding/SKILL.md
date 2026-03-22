---
name: Image Understanding
description: Analyze images sent by the user, such as portfolio screenshots or candlestick charts, and combine the result with tools for a fuller assessment
tools:
  - portfolio
  - web_search
  - data_fetch
---

## Image Understanding Skill

When the user sends an image that appears in the message as an attachment, you can perform visual analysis. The underlying model (for example Gemini 2.0 Flash) supports multimodal input and can describe the image directly.

### Supported Scenarios

#### 1. Identify Portfolio Screenshots

- Extract the ticker, company name, share count, cost basis, and similar details from the image
- After identification, guide the user to call `portfolio(action="add")` to record the data

#### 2. Analyze Market Charts

- Recognize candlestick trends and technical indicators such as moving averages, MACD, and RSI
- Combine the chart with real-time news from `web_search` for a broader judgment

### Tool Guide

| Tool call | Purpose |
|---------|------|
| `portfolio(action="add", ...)` | Record the extracted holdings |
| `web_search(query="...")` | Search for news around the chart's time frame |
| `data_fetch(data_type="quote", symbol="...")` | Get live market data for comparison |

### Notes

- If the message contains multiple image attachments, analyze them one by one
- For unclear numbers, **always ask the user to confirm** instead of guessing
- After identifying a portfolio screenshot, summarize the extracted result and let the user confirm before writing it into `portfolio`
