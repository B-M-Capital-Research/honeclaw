# Daily Valuation Lab

- status: done_locally
- date: 2026-08-11
- scope: daily reproducible valuation, rating integration, and public audit page

## Result

HONE now exposes a signed-in `/valuation-lab` from the chat home. The global 19:20 Beijing worker covers the existing 52-company research universe and, when FMP is configured, reads current quotes, eight quarterly cash-flow periods, the latest balance sheet and annual analyst estimates. It calculates HONE-owned bear/base/bull DCF values, a forward-EPS multiple cross-check and the current market's reverse-DCF implied growth.

Every result is evidence-gated. Stale price or financial data, missing shares or net cash, non-positive current/prior TTM free cash flow, missing forward EPS, a cross-check gap above 50%, unordered scenarios or a failed reverse solve yields no target price. Only eligible same-day rows are written as `hone-valuation-v1 / computed` into the existing company-rating valuation store; company ratings independently recheck freshness and quote consistency at 19:30.

The page exposes current position against the range, assumptions, method, source endpoints and dates. With the current local environment lacking an FMP key, the accepted snapshot truthfully reports 52 unavailable companies and zero model values rather than rendering sample data.

## Framework Boundary

The internal Hari material influenced the evidence discipline: growth assumptions should ultimately be challenged against scarcity, differentiation, real execution and remaining upside. It does not contain reviewed exact DCF discount rates or multiple thresholds. Those numbers are therefore explicitly versioned as HONE model defaults and must not be quoted as old Wang's personal valuation formula.

## Verification

- Valuation Rust tests: 5/5.
- Full Web API: 268 passed, 2 ignored.
- Web: 444/444.
- TypeScript, Rust formatting, public production build and console binary build passed.
- Authenticated API smoke confirmed fail-closed behavior without FMP.
- Authenticated desktop browser acceptance covered the page, summary, filters and an expanded unavailable reason.

## Operations

- Worker schedule: 19:20 Asia/Shanghai.
- Company rating schedule: 19:30 Asia/Shanghai.
- Snapshot: `data/valuation_lab/daily.json` under the configured data directory.
- Rating projection: `data/company_ratings/valuations/latest.json`.
- API: authenticated read-only `GET /api/public/valuation-lab`.
- No broker connection, order mutation, user portfolio data or model API is introduced by this feature.

## Follow-up

Configure the existing HONE FMP key pool in the target environment, inspect the first live coverage distribution and manually review the first eligible names. Add business-model-specific valuation methods before admitting banks, insurers, REITs or negative-FCF companies; do not weaken the current fail-closed gate to improve headline coverage.
