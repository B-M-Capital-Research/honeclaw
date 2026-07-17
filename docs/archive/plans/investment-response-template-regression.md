# Investment Response Template Regression Repair

- title: Investment response template, current-data, and stream recovery repair
- status: archived
- created_at: 2026-07-17
- updated_at: 2026-07-17
- archived_at: 2026-07-17
- owner: Codex
- related_files: `soul.md`, `skills/stock_research/SKILL.md`, `crates/hone-channels/src/investment_response_guard.rs`, `crates/hone-channels/src/agent_session/{core,emitter,tests}.rs`, `crates/hone-channels/src/{prompt,tool_trace,mcp_bridge}.rs`, `crates/hone-channels/src/runners/acp_common/process.rs`, `crates/hone-tools/src/{data_fetch,web_search}.rs`, `crates/hone-web-api/src/{state.rs,routes/chat.rs}`, `packages/app/src/{lib/public-chat.ts,pages/chat.tsx}`
- related_docs: `docs/invariants.md`, `docs/decisions.md`, `docs/repo-map.md`, `docs/runbooks/backend-deployment.md`, `docs/handoffs/2026-07-17-investment-response-contract-repair.md`
- related_prs: none; direct `main` commits `922007fa`, `d5f1dca0`, `3880d623`, `ce25d0ea`, `010dbae9`, `b0f50a77`, `d75451c3`, `ae8ebc11`, `340b9ee1`, `24c4c48d`, `dea3303d`, `4869ac5c`, `b4874a2c`, and `020c678a`

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

- Identified the prompt regression at `71a4498e686fe1a2f8634958a87c31bbd6a06f11`: it replaced the 291-line / 15,532-byte investment contract with a 36-line compact prompt and explicitly discouraged fixed templates. Restored the full pre-regression prompt and made the first Beijing data-time line plus normalized entity/quote facts server-owned.
- Implemented the five-scope entity state machine, exact ticker fast path, DataFetch quote gate, equity/fund/crypto evidence routing, portfolio read preflight, 15-second auxiliary extraction timeout, and dated-source validation.
- Guarded investment drafts are deferred until validation and then emitted once; refresh recovers the original server run start time, and persistent operations are execute-once across hidden repair attempts.
- Live FMP/DataFetch and Tavily diagnostics succeeded. The observed NBIS/RMBS/INTL failures came from internal entity/asset routing and strict format-repair behavior, not from a provider outage. Production RMBS and ISRG traces each had a valid exact quote before two roughly 60-second synthesis attempts, explaining the 120–125 second empty/failure experience.
- Added a bounded `extended_hours` DataFetch route and exact-symbol guard integration for explicit US premarket/postmarket requests. A fresh matching one-minute bar wins; otherwise the answer explicitly labels the regular-session quote and says extended-hours price was not verified. Crypto/night-session requests do not inherit the US-equity fallback label.
- Replaced the second model repair for supported quote/equity/fund/crypto/market scopes with a server-generated response built only from the prepared contract. The fallback is sanitized and revalidated by the same gate, clears the rejected draft's tool/transcript metadata, and retains the full established template. Runner failures and uncertain/persistent side effects still fail closed; comparisons and sectors retain their specialized repair path.
- Hardened current-price aliases, historical-price rejection, event-subheading boundaries, Markdown-safe provider labels, and English/Chinese execute-once classification. A date, domain, inference label, or coincidentally equal current value cannot authenticate an unsupported historical price. The final extended-price matrix covers local clause scope, signs, currencies, dates/times, ranges, scaled price/non-price metrics, compound movement clauses, and historical/OHLC tables.
- Rebuilt the public/admin bundles and runtime binaries, drained the old supervisor with SIGINT, and completed the final singleton restart at 2026-07-17 09:39 Beijing time. The final supervisor/backend PIDs are `23199`/`23210`; Postgres and S3 are healthy, local durable dependency count and active chats are both zero, local public/origin/Cloudflare public auth each return the expected `401`, and Discord/Feishu each have one child process. An earlier launch under a minimal supervisor `PATH` failed before writing a PID because `--build` could not find Cargo; the final launch used already-built binaries without `--build`, and the runbook now records both safe choices.
- Ran isolated production E2E turns for RMBS, NBIS, INTL, and ISRG. Every turn began with server-owned Beijing data time, used the exact entity and latest same-symbol quote, retained the complete asset-specific template, and produced exactly one `run_started`, one final `assistant_delta`, zero resets/errors, and one successful `run_finished`. After the final restart, a second RMBS turn independently revalidated the deployed runtime with one terminal stream and exactly one persisted assistant response. Cloudflare Pages deployment `53103ef2-eb25-4caa-aafc-f2f8c7a42afd` succeeded for exact code commit `020c678a`, and public `/chat` returned `200`.
- Closed two clean-Linux-only CI defects exposed by the final gate: MCP env/data-dir unit tests no longer depend on a stale ordinary `hone-mcp` binary, and Unix process-group signaling uses `--` before a negative PGID so Ubuntu procps sends TERM/KILL to the intended group.

## Validation

- Focused Rust unit tests for entity resolution, evidence routing, template validation, retry safety, and stream recovery.
- `cargo check --workspace --all-targets --exclude hone-desktop --exclude hone-user-app`
- `cargo test --workspace --all-targets --exclude hone-desktop --exclude hone-user-app`
- `bun run test:web`
- `bash tests/regression/run_ci.sh`
- Live DataFetch provider probes for RMBS, NBIS, INTL, BTCUSD, and mixed markets; isolated production Web E2E cases for RMBS, NBIS, INTL, and ISRG.
- Full runtime restart plus `/api/meta` and active-run health checks.
- Completed automated evidence: DataFetch focused tests `27/27`; `hone-channels` `565/565`, including a fresh `CARGO_TARGET_DIR`; workspace `cargo check` and `cargo test` with `--exclude hone-desktop --exclude hone-user-app`; Web typecheck and `265/265` unit tests; finance static contracts `24/24`; and the full CI-safe regression suite.
- Exact code commit `020c678a45dbc5c202c3d5c7225c8cd1ea7b507d` passed GitHub CI run `29547741054`, including frontend checks, Rust format/compile/tests, and CI-safe regressions.
- Completed live provider probes: exact entity/quote paths for NBIS, RMBS, INTL, and BTCUSD; equity financial/news and fund holdings routes; direct FMP and Tavily health diagnostics.
- Completed production response evidence:
  - RMBS: Rambus Inc., `101.42 USD`, quote time Beijing `2026-07-17 04:00`, about 43 seconds.
  - NBIS: Nebius Group N.V., `171.77 USD`, quote time Beijing `2026-07-17 04:00`, about 57 seconds.
  - INTL: Main International ETF, `30.145 USD`, quote time Beijing `2026-07-17 03:59`, fund/holdings route, about 45 seconds.
  - ISRG explicit postmarket: Intuitive Surgical, `358.93 USD`, fresh exact postmarket bar at Beijing `2026-07-17 07:59`, about 63 seconds of agent execution and no second whole-answer synthesis.
- For all four SSE runs: `run_started=1`, final `assistant_delta=1`, `assistant_reset=0`, `run_error=0`, `run_finished=1`, `success=true`.
- Final post-restart RMBS recheck: one synthesis in about 81 seconds; `run_started=1`, `assistant_delta=1`, `assistant_reset=0`, `run_error=0`, `run_finished=1`; exactly two persisted messages (`user`, `assistant`); first-line Beijing data/quote times; exact Rambus entity and `101.42 USD` quote; all nine required sections; false current-data denial count zero; active chats returned to zero.

## Documentation Sync

- Updated `docs/invariants.md`, `docs/decisions.md`, `docs/repo-map.md`, and `docs/runbooks/backend-deployment.md` with the durable contract and background-supervisor Cargo `PATH` requirement.
- Completed `docs/handoffs/2026-07-17-investment-response-contract-repair.md`, removed this task from `docs/current-plan.md`, moved this plan to `docs/archive/plans/`, and indexed the result in `docs/archive/index.md`.

## Risks / Open Questions

- Raw provider payloads can contain conflicting snapshot fields or entity-ambiguous news; canonical facts must win.
- Format checks must not discard a substantively correct answer or trigger a second persistent write.
- Broad market and sector discovery must fail closed when evidence is insufficient, without treating a common ticker as an acronym.
- Auxiliary extraction is deliberately fail-closed for unresolved company names; the exact ticker path remains independent so common ticker requests do not wait on or inherit auxiliary failures.
- Comparison and sector scopes retain specialized model repair; monitor them separately. The deterministic fallback is intentionally conservative and may reject an ambiguous non-price extended-hours sentence rather than permit a false quote.
- The initial model synthesis can still take roughly 40–85 seconds on a deep query, but the former second whole-answer attempt is removed for supported scopes; ISRG completed in one synthesis instead of the prior 120–125 second failure.
- No database or storage migration is involved. Rollback is code/asset-only: restore the previous server/frontend revision, rebuild, and perform the controlled runtime restart; actor sessions, portfolios, and other durable data do not need transformation or rollback.
