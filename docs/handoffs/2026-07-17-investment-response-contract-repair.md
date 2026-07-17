# Investment Response Contract Repair

- title: Investment response entity, live-data, template, and stream repair
- status: done
- created_at: 2026-07-17
- updated_at: 2026-07-17
- owner: Codex
- related_files: `soul.md`, `crates/hone-channels/src/investment_response_guard.rs`, `crates/hone-channels/src/agent_session/{core,emitter,tests}.rs`, `crates/hone-channels/src/{prompt,tool_trace,mcp_bridge}.rs`, `crates/hone-channels/src/runners/acp_common/process.rs`, `crates/hone-tools/src/{data_fetch,web_search}.rs`, `crates/hone-web-api/src/{state.rs,routes/chat.rs}`, `packages/app/src/{lib/public-chat.ts,pages/chat.tsx}`
- related_docs: `docs/archive/plans/investment-response-template-regression.md`, `docs/invariants.md`, `docs/decisions.md`, `docs/repo-map.md`, `docs/runbooks/backend-deployment.md`, `docs/archive/index.md`
- related_prs: none; direct `main` commits `922007fa`, `d5f1dca0`, `3880d623`, `ce25d0ea`, `010dbae9`, `b0f50a77`, `d75451c3`, `ae8ebc11`, `340b9ee1`, `24c4c48d`, `dea3303d`, `4869ac5c`, `b4874a2c`, and `020c678a`

## Summary

The investment response path has been rebuilt around a deterministic service-owned contract rather than relying on the model to remember the old format. The visible answer begins with Beijing data time, then exact normalized entity and same-symbol DataFetch quote facts. The model body receives asset-appropriate evidence and must follow the restored full response template. Guarded drafts stay hidden until validation and publish as one final assistant message with one terminal stream event.

Live FMP/DataFetch and Tavily diagnostics succeeded. The production NBIS/RMBS/INTL failures were not a general provider outage: they came from internal entity/asset routing, false company-financial requirements for funds, and format validation/repair that could discard a valid quote context or spend a long time retrying. RMBS and ISRG production traces already contained exact same-symbol quotes before two roughly 60-second synthesis attempts, producing the observed 120–125 second failure.

## What Changed

- Restored the full pre-`71a4498e` `soul.md` investment prompt, including task routing, time-first output, fact/inference separation, valuation discipline, Bull/Bear/Base framing, risk/falsification conditions, and the established equity/fund/crypto/market/sector templates.
- Added a five-outcome entity scope state machine for every nonempty turn:
  - `Securities`: exact-resolve every current-turn ticker/company and require a positive same-symbol quote before numeric conclusions.
  - `Portfolio`: read the actor-scoped portfolio/watchlist once as the membership, quantity, and cost truth source; current performance analysis exact-resolves and quotes the explicit ticker, or a bounded portfolio subset when no ticker is named, with totals and omitted coverage disclosed.
  - `Broad`: prepare representative market/sector evidence without inventing a company entity.
  - `ConfirmedNoEntity`: continue a general financial question while forbidding history from injecting an old ticker.
  - `NeedsClarification`: stop unresolved named-security analysis and ask for a company name or ticker.
- Kept ordinary exact tickers such as `NBIS`, `RMBS`, and `INTL` on a deterministic DataFetch fast path. Named companies/aliases may use auxiliary extraction, but that call is capped at 15 seconds; timeout, provider failure, malformed JSON, or incomplete multi-entity output fails closed rather than analyzing a partial set.
- Classified exact instruments before deep evidence. Equities use exact profile, meaningful financial statements, and entity-matched news; ETFs/funds use structured fund profile, holdings, and news without company-financial or earnings requirements; crypto uses exact market identity, quote, and news without stock-profile requirements.
- Made the server own the first Beijing data-time line, normalized entity, same-symbol current quote, change, quote-source timestamp, and verified-fact labels. A successful DataFetch quote prevents the model from claiming that current/realtime data was unavailable; wording remains “latest available”, not tick-by-tick.
- Added exact-symbol US extended-hours handling for explicit premarket/postmarket questions. DataFetch returns only the latest bounded one-minute bar; the guard accepts it only when its symbol, requested session, New York trading-session boundary, and 45-minute freshness all match. Otherwise it retains the regular quote and visibly discloses that the requested extended-hours price was not verified.
- Reduced provider evidence before prompt injection. Conflicting profile snapshot prices and unsupported financial interpretations are removed. Every claimed event fact must use a verified real absolute date and matching full source domain in the same clause; otherwise it is explicitly inference, hypothesis, or scenario.
- Deferred investment candidate deltas/resets/thoughts/errors behind validation, then emitted exactly one canonical assistant answer. Session `Done` is the sole `run_finished` authority, so late frames cannot create a second flash/run.
- Kept Web run state server-authoritative. Refresh resumes the same `run_id` and original `started_at_ms`; it does not repost the prompt or reset elapsed time. A missing runner with an unanswered persisted user turn becomes an explicit interruption.
- For supported quote/equity/fund/crypto/market scopes, an incomplete successful draft no longer starts a second model run. The server builds a complete answer only from the prepared contract, sanitizes it, and runs it through the same final validator; rejected draft tool/transcript metadata is cleared. Runner failures and persistent/uncertain writes remain failures. Comparisons and sector analysis retain their specialized model repair path. English and Chinese mutation-plus-analysis requests are both execute-once.
- Automatic retry, context-overflow recovery, deterministic fallback, and contract repair now require every observed tool call to match an explicit read-only allowlist. Unknown tools fail closed; `deep_research` is always persistent. A pre-run intent classifier prevents no-trace portfolio writes, completed trades, and research starts from executing twice while keeping ordinary investment-advice questions on the normal path.
- Historical/open/close/high/low prices now fail closed unless a future dedicated verified historical-quote field is added. A date, source domain, inference heading, or a number that happens to equal the current quote cannot launder an unsupported historical price. Inference/condition subheadings apply only to their following Markdown list items and reset before ordinary prose or unknown headings.
- Extended-hours prose is reverse-validated against the exact entity, session, price, and any stated currency, including bare prices, session markers after the number, and English/Chinese modifiers. Markdown historical/OHLC headers carry into their numeric rows; target/scenario tables and same-symbol current-price tables without date, historical, or OHLC semantics remain allowed.
- The extended-price guard regression matrix covers local clause scope, positive and negative ASCII/Unicode signs, prefix/suffix/ISO currencies, date/time/quarter/year/percentage/basis-point tokens, scaled price versus non-price metrics, ranges and regular-close baselines, compound `while`/`whereas`/`而` movements, and historical/OHLC table semantics. Every factual extended/pre/post claim must independently match the prepared contract.

## Verification

Completed before closing this handoff:

- DataFetch focused tests: `27/27`.
- Full `hone-channels` tests: `565/565`, including a fresh `CARGO_TARGET_DIR` run without a prebuilt ordinary `hone-mcp` binary.
- `hone-web-api`: `117 passed`, `2 ignored`.
- Prompt tests: `12/12`.
- Finance static contract checks: `24/24`.
- Frontend tests: `265/265`.
- `bun run typecheck:web`, `cargo check --workspace --all-targets --exclude hone-desktop --exclude hone-user-app`, and `cargo test --workspace --all-targets --exclude hone-desktop --exclude hone-user-app` passed locally.
- `bash tests/regression/run_ci.sh` passed, including finance automation contracts `24/24`.
- Live provider probes passed exact entity/quote paths for NBIS, RMBS, INTL, and BTCUSD, including equity financial/news and fund holdings routes.
- `scripts/diagnose_fmp_tavily.sh` reported both FMP and Tavily healthy during the incident investigation.
- The manual live DataFetch regression returned exact current entities and quotes for NBIS (`171.77 USD`), INTL (`30.145 USD`, ETF/fund route), RMBS (`101.42 USD`), and BTCUSD (`63800.99 USD`).
- Production Web E2E completed for RMBS, NBIS, INTL, and explicit ISRG postmarket. The first line was Beijing data time, the normalized entity and quote matched the prepared contract, and every answer used the full asset-specific template without false availability denial. ISRG used the exact fresh postmarket bar `358.93 USD` at Beijing `07:59`; RMBS/NBIS/INTL used `101.42`/`171.77`/`30.145 USD` with their provider quote times.
- Each of the four initial production SSE runs had `run_started=1`, final `assistant_delta=1`, `assistant_reset=0`, `run_error=0`, `run_finished=1`, and `success=true`.
- After the final guard patch, code commit `020c678a45dbc5c202c3d5c7225c8cd1ea7b507d` passed GitHub CI run `29547741054`: frontend checks, Rust format/compile/tests, and CI-safe regressions were all successful. Gitleaks and the relevant completed CodeQL checks were also green.
- Cloudflare Pages successfully deployed that exact code commit as deployment `53103ef2-eb25-4caa-aafc-f2f8c7a42afd`. The public `/chat` route returned `200`, while local public, origin, and Cloudflare public unauthenticated auth probes each returned the expected `401`.
- The old supervisor was drained with SIGINT at zero active chats. The final replacement started at Beijing `2026-07-17 09:39`; supervisor/backend PIDs are `23199`/`23210`, ports 8077/8088 each have one listener, Postgres and S3 are healthy, local durable dependency count and active chat count are zero, and Discord/Feishu each have one child. An earlier background launch failed before writing `current.pid` because its minimal `PATH` omitted Cargo while using `--build`; the corrected final launch used already-built binaries without `--build`, and the deployment runbook records both safe choices.
- A fresh post-restart `现在rmbs怎么看` production turn completed in one synthesis in about 81 seconds. The persisted answer was 1,878 characters and exactly two history messages existed (`user`, `assistant`). Its first line was `数据时间：北京时间 2026-07-17 09:42；行情口径：报价源时间：北京时间 2026-07-17 04:00（最新可得，非逐笔）`; it identified Rambus Inc. (`RMBS`, NASDAQ Global Select, equity), used `101.42 USD` and `-1.42871%`, included all nine required sections, and contained no false current-data denial. SSE counts were `run_started=1`, `assistant_delta=1`, `assistant_reset=0`, `run_error=0`, and `run_finished=1`; active chats returned to zero.
- Clean Ubuntu CI exposed and drove two hermetic test fixes: env/data-dir tests now inject a dummy `HONE_MCP_BIN`, and Unix process-group signaling places `--` before a negative PGID to avoid the procps 4.0.4 parsing defect.

## Risks / Follow-ups

- The auxiliary company-name extractor intentionally fails closed after 15 seconds. The deterministic exact ticker path is independent, but ambiguous company names may require the user to retry or provide a ticker when the auxiliary route is unavailable.
- Comparisons and sector analysis still use specialized model repair when their first draft is incomplete. Monitor those scopes separately; ordinary quote/equity/fund/crypto/market failures must not regress to a second full model synthesis.
- Deep production turns still spend roughly 40–85 seconds in the one allowed initial synthesis. The duplicate 60-second repair is gone, but first-token/total latency should continue to be monitored independently.
- Portfolio preflight is read-only. An explicit add/update/delete request must still execute its mutation once; never report the preflight read as mutation success.
- This change has no database, schema, or durable-storage migration. Rollback is code/asset-only: restore the previous server/frontend revision, rebuild, drain active runs, and perform the controlled restart. Do not delete or transform actor sessions, portfolios, or other durable data.

## Next Entry Point

Use `docs/archive/plans/investment-response-template-regression.md` for the full evidence trail. Future investment-response work should begin at `crates/hone-channels/src/investment_response_guard.rs`, `crates/hone-channels/src/agent_session/core.rs`, and `crates/hone-channels/src/tool_trace.rs`; deployment/rollback remains governed by `docs/runbooks/backend-deployment.md`.
