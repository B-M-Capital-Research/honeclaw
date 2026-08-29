# Tavily 失效后的零密钥公开搜索兜底

- title: Public search fallback
- status: done
- created_at: 2026-08-29
- updated_at: 2026-08-29
- owner: Codex
- related_files: `crates/hone-tools/src/web_search.rs`, `crates/hone-web-api/src/routes/portfolio_news.rs`, `config.example.yaml`
- related_docs: `docs/archive/plans/public-search-fallback.md`, `docs/invariants.md`, `docs/repo-map.md`, `docs/decisions.md#d-2026-08-29-198-keep-web-search-available-without-weakening-evidence-authority`
- related_prs: local branch `codex/unified-hone-sndk`; no push or production deployment

## Summary

HONE no longer treats Tavily as a single point of failure. An exhausted local Tavily key now automatically falls through to DuckDuckGo's official non-JavaScript search and returns the same bounded snippet evidence shape.

## What Changed

- Added zero-key DuckDuckGo HTML retrieval, redirect URL recovery and DOM parsing.
- Added provider recency parameters, 15-second timeout, challenge detection and five-minute failure cooldown.
- Preserved the three-result cap and locally overwritten citation/evidence contract.
- Propagated the actual provider into portfolio-news source labels and payloads.
- Documented that first-party DataFetch sources remain authoritative for company and financial facts.

## Verification

- HONE Tools: 193 passed, 1 ignored.
- Portfolio News: 12 passed.
- Finance automation contracts: 49 passed.
- Channels/Web API compile check passed.
- Real local runtime and interactive Web chat both exercised Tavily quota failure followed by successful `duckduckgo_html` results.

## Risks / Follow-ups

- DuckDuckGo can present a human challenge after bursts of traffic; the parser rejects it and cools down instead of returning a false success.
- If production volume grows, add a contracted secondary API provider (for example Brave Search API) ahead of the public HTML route; do not remove the evidence-authority boundary.
- Luna HTTP 403 background-model errors remain separate from search availability.

## Next Entry Point

Use `WebSearchTool::search_public_fallback` and `parse_duckduckgo_html` for fallback maintenance. When adding another provider, normalize into the same `results` shape and pass through `annotate_basic_search_evidence` rather than creating a looser evidence path.
