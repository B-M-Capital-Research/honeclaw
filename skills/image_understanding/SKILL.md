---
name: Image Understanding
description: Analyze images sent by the user, such as portfolio screenshots or candlestick charts, and combine the result with tools for a fuller assessment
when_to_use: "Only invoke when the current message contains an actual image attachment (photo, screenshot, chart). Never call this skill for text-only requests, even if the user mentions watchlists, portfolios, or charts in text."
tools:
  - portfolio
  - web_search
  - data_fetch
---

## Image Understanding Skill

When the user sends an image attachment, first consume the current-turn
`【图片文字提取】` block produced by the attachment service. It is trusted
current-attachment evidence and is grouped by filename. If that block is empty
or absent, follow the current-turn attachment policy and actually try the
read-only local file tools on the attachment path before concluding anything;
do not call this skill repeatedly to try to make the same image visible.

If extraction is partial, answer from the fields that are actually present and
ask one minimal confirmation only for the specific number or label that matters
to the user's decision. Never replace the whole answer with a generic tool,
OCR, or research failure.

Nothing about this pipeline is user-facing. The reply must not mention this
skill, `【图片文字提取】`, the runner, local paths, the image's file type, or
why extraction failed. When an image genuinely cannot be read, say only that
this picture did not come through clearly and name the one value you need
("看不清持仓那一栏，成本价是多少？") — never ask the user to re-send it in a
different format.

### Supported Scenarios

#### 1. Identify Portfolio Screenshots

- Extract the ticker, company name, share count, cost basis, and similar details from the image
- Keep the filename boundary when several screenshots overlap
- Only offer `portfolio(action="add")` after summarizing the extracted values
  and receiving confirmation; image analysis itself is read-only

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
- Use other current-turn evidence such as `portfolio(action="view")`,
  `data_fetch`, or `web_search` when it helps answer the question; a missing
  image field is a narrow evidence gap, not authority to refuse the task
