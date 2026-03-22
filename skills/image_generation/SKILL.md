---
name: Image Generation
description: Generate dynamic portfolio screenshots, stock-analysis infographics, or general-style images. Real data must be fetched first
tools:
  - image_gen
  - portfolio
  - data_fetch
  - web_search
---

## Image Generation Skill

Three image types are supported. The `portfolio_snapshot` and `stock_analysis` types must use real data before generation.

### Image Types

#### 1. Portfolio Snapshot (`portfolio_snapshot`)

Fetch the portfolio first, organize the data, and then generate the image:

```text
portfolio(action="get")
-> organize the data (company name, ticker, price, percent change, shares, and so on)
-> image_gen(image_type="portfolio_snapshot", content="AAPL Apple $189.50 +1.2%\nTSLA Tesla $245.30 -0.8%\n...")
```

#### 2. Stock Analysis Graphic (`stock_analysis`)

Fetch the stock data first, organize it, and then generate an infographic:

```text
data_fetch(data_type="snapshot", symbol="ticker")
-> optionally use web_search to gather moat, competitive advantage, and similar context
-> image_gen(image_type="stock_analysis", content="Company: NVIDIA NVDA\nMarket cap: 3.2T\nPE: 65x\nMoat: CUDA ecosystem + AI training dominance\n...")
```

#### 3. General Image (`general`)

Generate directly from the user's description:

```text
image_gen(image_type="general", prompt="A futuristic image showing the global financial market")
```

### Important Rules

- `portfolio_snapshot` and `stock_analysis` **must** fetch real data first, and the content must include detailed, concrete numbers
- You may use the `prompt` parameter to specify extra visual style preferences
