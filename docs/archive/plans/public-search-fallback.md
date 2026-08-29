# Public Search Fallback

- title: Tavily 失效后的零密钥公开搜索兜底
- status: archived
- created_at: 2026-08-29
- updated_at: 2026-08-29
- owner: Codex
- related_files: `crates/hone-tools/src/web_search.rs`, `crates/hone-tools/Cargo.toml`, `crates/hone-web-api/src/routes/portfolio_news.rs`, `config.example.yaml`
- related_docs: `docs/invariants.md`, `docs/repo-map.md`, `docs/decisions.md`, `docs/handoffs/2026-08-29-public-search-fallback.md`

## Goal

让 HONE 在 Tavily 未配置、额度耗尽、鉴权失败或短时故障时仍能获得真实网络搜索摘要，同时不降低金融事实的证据标准。

## Scope

- Tavily 保持首选和原有 key/cooldown 行为。
- 自动降级到 DuckDuckGo 官方无 JavaScript HTML 搜索，无需第二个密钥。
- 公开搜索使用 15 秒上限、失败后 5 分钟冷却、最多 3 条结果。
- 两条路线共用本地覆盖的 snippet-only 证据合同，并显式记录实际 provider。
- 持仓新闻保存实际 provider，不再把 DuckDuckGo 结果错误标成 Tavily。

## Validation

- `cargo test -p hone-tools --lib`: 193 passed, 1 ignored.
- `cargo check -p hone-channels -p hone-web-api`: passed with four pre-existing dead-code warnings.
- `cargo test -p hone-web-api portfolio_news::tests`: 12 passed.
- `bash tests/regression/ci/test_finance_automation_contracts.sh`: 49 passed.
- Local runtime with an exhausted Tavily key logged repeated `provider="duckduckgo_html" public search fallback succeeded` events for scheduled products and an interactive SNDK query.
- Browser acceptance returned one official Sandisk press release and bounded SEC results rather than a provider-unavailable failure.

## Documentation Sync

Updated configuration comments, repository map, evidence invariants, the architecture decision log, completion handoff and archive index. The task completed in one working session, so no active entry remains in `docs/current-plan.md`.

## Risks / Open Questions

- Public HTML layouts and automated-access challenges can change; parser mismatch fails closed and activates cooldown.
- Search snippets never become page content or first-party financial truth. SEC, Nasdaq and company IR remain the preferred routes for current filings, quotes and reported financials.
- The configured Luna background profiles still return HTTP 403; that is a separate model-enrichment issue and does not block source-only search results.
