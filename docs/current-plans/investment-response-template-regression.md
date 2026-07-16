# Investment Response Template Regression Repair

- title: Investment response template, current-data, and stream recovery repair
- status: in_progress
- created_at: 2026-07-17
- updated_at: 2026-07-17
- owner: Codex
- related_files: `soul.md`, `skills/stock_research/SKILL.md`, `crates/hone-channels/src/investment_response_guard.rs`, `crates/hone-channels/src/agent_session/{core,emitter,tests}.rs`, `crates/hone-channels/src/prompt.rs`, `crates/hone-tools/src/{data_fetch,web_search}.rs`, `crates/hone-web-api/src/{state.rs,routes/chat.rs}`, `packages/app/src/{lib/public-chat.ts,pages/chat.tsx}`
- related_docs: `docs/invariants.md`, `docs/decisions.md`, `docs/repo-map.md`, `docs/handoffs/2026-07-17-investment-response-contract-repair.md`

## Goal

Restore the long investment prompt as an enforced runtime contract: server-owned Beijing data time first, entity-first exact security resolution, current same-symbol DataFetch quote and timestamp, asset-appropriate evidence, and the complete prior single-security / fund / crypto / market / sector response templates. Eliminate false current-data denial, fragile whole-answer retries, duplicate terminal streams, and refresh-time run loss.

## Scope

- Audit the regression commits and production RMBS / NBIS / INTL evidence.
- Route every nonempty turn through one explicit entity scope: securities, actor portfolio, broad market/sector, confirmed no entity, or needs clarification.
- Make time, resolved entity, canonical quote, quote timestamp, and fact labels deterministic server output.
- Resolve exact/common tickers through current-turn DataFetch, route evidence by equity/fund/crypto type, and require dated event facts to carry their verified source domain in the same clause.
- Read personal holdings/watchlists from the actor-scoped portfolio tool before analysis; when the question asks about current performance, continue from the explicit ticker or a bounded, disclosed portfolio subset into exact entity and quote verification instead of treating stored symbols or prices as current market evidence.
- Bound auxiliary named-entity extraction to 15 seconds and fail closed on timeout, provider failure, or malformed output without accepting a partial entity set.
- Preserve a valid draft during format repair and prevent persistent operations from re-executing.
- Remove conflicting profile prices, entity-mismatched news, and ambiguous raw financial evidence.
- Cover lowercase/common tickers and broad market/sector queries without promoting theme acronyms.
- Rebuild, restart all runtime services, and run isolated real-data E2E checks.

## Current Progress

- Restored the full pre-`71a4498e` investment prompt and made the first Beijing data-time line plus normalized entity/quote facts server-owned.
- Implemented the five-scope entity state machine, exact ticker fast path, DataFetch quote gate, equity/fund/crypto evidence routing, portfolio read preflight, 15-second auxiliary extraction timeout, and dated-source validation.
- Guarded investment drafts are deferred until validation and then emitted once; refresh recovers the original server run start time, and persistent operations are execute-once across hidden repair attempts.
- Live FMP/DataFetch and Tavily diagnostics succeeded. The observed NBIS/RMBS/INTL failures came from internal entity/asset routing and strict format-repair behavior, not from a provider outage.
- Final workspace CI, post-merge live E2E, controlled restart, and deployment health checks remain pending in the parent task.

## Validation

- Focused Rust unit tests for entity resolution, evidence routing, template validation, retry safety, and stream recovery.
- `cargo check --workspace --all-targets --exclude hone-desktop --exclude hone-user-app`
- `cargo test --workspace --all-targets --exclude hone-desktop --exclude hone-user-app`
- `bun run test:web`
- `bash tests/regression/run_ci.sh`
- Live DataFetch and isolated Web E2E cases for RMBS, NBIS, INTL, crypto, market, and sector prompts.
- Full runtime restart plus `/api/meta` and active-run health checks.
- Completed evidence so far: investment-guard focused tests `56/56`, AgentSession focused tests `79/79`, full `hone-channels` tests `549/549`, `hone-web-api` tests `117 passed / 2 ignored`, prompt tests `12/12`, finance static contracts `24/24`, and frontend tests `265/265`.
- Completed live provider probes: exact entity/quote paths for NBIS, RMBS, INTL, and BTCUSD; equity financial/news and fund holdings routes; direct FMP and Tavily health diagnostics.
- TODO before closure: record final post-rebase workspace check/test/regression counts, deployed RMBS/NBIS/INTL response samples, SSE terminal counts, and restart health evidence.

## Documentation Sync

- Update `docs/current-plan.md`, `docs/invariants.md`, `docs/decisions.md`, and `docs/repo-map.md` while active.
- On completion, write one handoff, archive this plan, update `docs/archive/index.md`, and remove the active index entry.

## Risks / Open Questions

- Raw provider payloads can contain conflicting snapshot fields or entity-ambiguous news; canonical facts must win.
- Format checks must not discard a substantively correct answer or trigger a second persistent write.
- Broad market and sector discovery must fail closed when evidence is insufficient, without treating a common ticker as an acronym.
- Auxiliary extraction is deliberately fail-closed for unresolved company names; the exact ticker path remains independent so common ticker requests do not wait on or inherit auxiliary failures.
- No database or storage migration is involved. Rollback is code/asset-only: restore the previous server/frontend revision, rebuild, and perform the controlled runtime restart; actor sessions, portfolios, and other durable data do not need transformation or rollback.
