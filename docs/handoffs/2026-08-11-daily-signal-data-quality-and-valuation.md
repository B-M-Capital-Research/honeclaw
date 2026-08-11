# Daily Signal Data Quality And Company Valuation Handoff

## Outcome

The three existing authenticated dashboards were tightened without starting another homepage feature. Macro and AI snapshots now use model v2. Company ratings use methodology v2 and fail closed when a current, reviewed Hari valuation is unavailable.

## Macro

- Added FRED `DGS10`, `DGS30`, `FEDFUNDS`, `EMRATIO` and `VIXCLS`.
- Rebalanced macro weights to 1.0.
- Treasury yields, the policy rate and VIX use risk-direction scoring rather than the ordinary “higher is healthier” growth rule.
- Old v1 snapshots are normalized: missing v2 dimensions remain unknown rather than silently inheriting an old complete score.
- The 2026-08-11 local refresh was live at 61.4 and displayed all five additions.

## AI

- Removed hardware quote fetching, hardware cards and the hardware layer.
- Removed unsupported AI revenue, RPO/order visibility and specialized monetization placeholders.
- Each cloud-company score now uses seven standard financial factors: revenue growth, gross margin, operating margin, FCF margin, capex growth, liquidity and debt load.
- Coverage uses `metric_total=7`; missing standard financial values remain unknown and never become zero.
- Old v1 snapshots are normalized before presentation. The 2026-08-11 local refresh was live at 72.6 with no hardware placeholder.

## Company valuation

- Removed static transcript valuation risk and generic P/E buckets from the live score.
- `dimensions.valuation` is nullable. Missing valuation removes the 15% weight and normalizes the remaining 85%.
- Optional input path: `data/company_ratings/valuations/latest.json`.
- Required report contract: `framework_version="hari-invest-v1"`, current Beijing `report_date`, `generated_at`, and `items`.
- Required item contract: exact ticker, current-date `as_of`, `review_status="verified"`, ordered positive `bear_case <= base_case <= bull_case`, currency, method, non-empty assumptions, at least two sources and optional current price. Generation must be under 36 hours old; a supplied valuation price must be within 5% of the current quote.
- A passing artifact displays bear/base/bull/current, range position, method, Beijing valuation time and data date. Without one, the UI says “今日不计估值分”.
- The local environment has no verified current-day valuation artifact and no live FMP quote coverage, so all 52 rows correctly exclude valuation instead of fabricating it.

## Verification

- `cargo fmt --all -- --check`
- `cargo test -p hone-web-api --lib --no-fail-fast`: 229 passed, 2 ignored
- Full Web suite: 420 passed
- Focused dashboard contracts: 21 passed
- `bun run typecheck`
- `bun run build`
- Local Web-only runtime: backend 8077/8088 and public Vite 3001; `MultiChannelSink` reported `channels=["web"]`.
- Authenticated browser inspection at `http://127.0.0.1:3001/chat` confirmed all requested behavior.

## Current local runtime

- User UI: `http://127.0.0.1:3001/chat`
- Local dev login is enabled for this process.
- No Feishu or Discord channel was started.
- No remaining homepage button or new feature was implemented in this task.
