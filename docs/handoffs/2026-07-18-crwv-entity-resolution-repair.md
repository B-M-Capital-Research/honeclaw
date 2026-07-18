# CRWV Exact-Ticker Entity Resolution Repair

- title: CRWV exact-ticker versus embedded-product entity resolution repair
- status: in_progress
- created_at: 2026-07-18
- updated_at: 2026-07-18
- owner: Codex
- related_files: `crates/hone-channels/src/investment_response_guard.rs`, `crates/hone-channels/src/agent_session/core.rs`, `crates/hone-channels/src/agent_session/tests.rs`, `crates/hone-channels/src/tool_trace.rs`, `crates/hone-channels/src/prompt.rs`, `crates/hone-core/src/config/server.rs`, `crates/hone-core/src/config/tests.rs`, `soul.md`, `skills/stock_research/SKILL.md`, `tests/regression/ci/test_finance_automation_contracts.sh`, `tests/regression/manual/test_entity_search_live.sh`
- related_docs: `docs/current-plans/ticker-resolution-architecture.md`, `docs/decisions.md#d-2026-07-17-04-resolve-securities-through-a-span-aware-exact-first-pipeline`, `docs/decisions.md#d-2026-07-18-01-keep-entity-discovery-inside-the-main-agent-tool-loop`, `docs/invariants.md`, `docs/repo-map.md`, `docs/bugs/scheduler_finance_entity_guard_misclassifies_instruction_words.md`, `docs/runbooks/backend-deployment.md`
- related_prs: commits `4d419770`, `b87c4cb7`, `2d6b4be8`, `8d4fcdd6`

## Summary

The production failure was not an FMP/DataFetch outage. For lowercase `crwv`, the exact quote probe correctly verified CoreWeave (`CRWV`), while semantic search also returned GraniteShares YieldBOOST CRWV ETF (`CWY`). The old reconciliation score treated any candidate name containing the query as a competing company name, so a product that merely referenced its underlying ticker generated a false identity conflict.

The repair classifies exact ticker identity, strong natural-name relations, and embedded-ticker products separately. Provider-verified `CRWV` now wins over `CWY`; the same rule covers `RKLB/RKLX` and `AAPL/AAPU`. Genuine code-versus-company conflicts such as Ford Motor (`F`) versus ticker `FORD` still clarify. Natural-name fallback uses word boundaries, so Apple resolves to `AAPL` without treating Appleseed Fund (`APPLX`) as an Apple-name match.

## What Changed

- Tentative ticker reconciliation now considers only strong full-name or word-bounded name-prefix relations; low semantic scores are unresolved instead of arbitrary ambiguity lists.
- ETF, ETN, fund, leveraged, yield, long/short, covered-call, option, warrant, and similar names that embed a different requested ticker are classified as reference products and cannot challenge an exact same-symbol quote.
- Derivative-only semantic results cannot replace a missing exact ticker, while full product names can still resolve to the product itself through the normal named-entity path.
- Added deterministic regressions for `CRWV/CWY`, `RKLB/RKLX`, `AAPL/AAPU`, `Ford/FORD`, `Apple/Appleseed`, derivative-only search, and named-company fallback.

## Verification

- `cargo test -p hone-channels investment_response_guard::tests::` = 83 passed after the word-boundary repair.
- `cargo test -p hone-channels` = 596 passed; the proportional full workspace run also completed successfully before the final word-boundary refinement.
- `bash tests/regression/run_ci.sh` passed; finance automation contracts = 26/26.
- Credentialed `tests/regression/manual/test_entity_search_live.sh` passed. Live DataFetch returned CoreWeave for exact `CRWV`, quote `73.21`, a non-ETF CRWV profile, and three different-symbol products whose names reference CRWV, proving the provider was healthy and the ambiguity was local logic.
- Rebuilt `hone-cli`, `hone-console-page`, `hone-discord`, `hone-feishu`, and `hone-mcp`. Gracefully stopped supervisor `39101` only after active chat count reached zero, then started supervisor `85148` with backend `85163`, Discord `85418`, and Feishu `85439`.
- Production `/api/chat` probe `crwv当前价` completed in 11.8 seconds with CoreWeave/CRWV, current quote `73.21`, time-first output, one `assistant_delta`, one successful `run_finished`, and no reset/error.
- Production probe `crwv预计估值多少` completed in 69.6 seconds with CoreWeave/CRWV, server-owned time and quote, the nine-section equity template, one final answer, and one successful terminal event. Active chat count returned to zero.
- Final health: Postgres and S3 connected; 8077 and 8088 served from backend `85163`; Web, Discord, and Feishu each reported one current process; public root returned HTTP 200.

## Risks / Follow-ups

- This incident is closed, but the umbrella cross-market ticker plan stays `in_progress`: the post-restart scheduler window still reproduced the existing P2 where task prose can surface `800G`, `AST`, `SEC`, `NAND`, or named-company listing ambiguity. That separate live defect remains tracked in `docs/bugs/scheduler_finance_entity_guard_misclassifies_instruction_words.md`; it is not an FMP outage and was not hidden by this CRWV handoff.
- The deep CRWV valuation request used the deterministic evidence-safe fallback because the model draft failed the full response contract. It intentionally disclosed unavailable valuation inputs instead of inventing a target price.
- No database, session, portfolio, quota, or storage migration was required.

## Next Entry Point

For tentative ticker/name reconciliation, start at `resolve_tentative_named_match`, `reconcile_tentative_entity_match`, `candidate_is_embedded_ticker_reference`, and `tentative_name_candidate_score` in `crates/hone-channels/src/investment_response_guard.rs`. Continue the remaining scheduler/entity work from `docs/current-plans/ticker-resolution-architecture.md` and the linked P2 bug record.

## Phase 2 — Multi-Ticker Agent-Loop Entity Discovery

### Incident and Root Cause

The production turn `分析下crwv和nbis的估值` failed at 2026-07-18 15:03 Beijing time after about 16.2 seconds. The lexical scanner had already captured both `CRWV` and `NBIS`, but residual-language deletion left `下`; the comparison-binding rule then treated `和` as evidence that the set was incomplete even though both sides were present. The request entered the auxiliary entity LLM, hit its fixed 15-second timeout, and returned the generic entity-recognition failure. Logs contained no DataFetch start, batch probe, or `agent.run start` for the turn. Same-window failures across unrelated symbols confirmed an architecture-level single point, while live FMP results showed both securities were healthy.

### Architecture Change

- Removed the current-turn blocking auxiliary entity extraction, timeout, generic availability error, and `NeedsClarification` pre-run outcome.
- Lexical results are now explicitly candidate seeds. For interactive traffic they never close the scope, trigger clarification, or start a DataFetch/portfolio preflight; every nonempty wording becomes `AgentToolDiscovery` so the configured main runner reads the complete query. Portfolio membership is loaded by the Agent's own `portfolio(view)` call in that same loop.
- If the Agent finds named securities, its tool loop searches every candidate—including explicit tickers—and exact-verifies all selected symbols with quote/profile before problem-specific evidence. High-confidence explicit code seeds are checked only after the run as a minimum that cannot be silently omitted; they are not a closed entity list. Search may refine across multiple assistant rounds, and an earlier empty broad query cannot hide later exact evidence. Ordinary finance or non-security turns can simply continue without an irrelevant DataFetch call.
- After the same run completes, the service derives an optional entity contract from all current-run search/refinement calls and structured tool facts. Every contracted entity must intersect a later positive same-symbol quote with a usable provider timestamp. The actual financials, holdings, news, web, earnings, sector, market, or extended-hours results selected by the Agent remain in its context for synthesis; user wording does not select a server-side depth route, and tool presence does not authorize a fixed chapter parser.
- DataFetch response envelopes are excluded from candidate rows, fixing the Chinese-name case where the wrapper query `英伟达` previously hid the returned provider symbol `NVDA`. CRWV/CWY is resolved only when CRWV itself has later exact quote evidence; quoting only CWY fails. A complete current trace must cover named NBIS and later exact quote evidence before it can create a two-sided verified comparison contract.
- Dynamic contracts follow observed search/quote scope only. The service no longer infers tool calls from “估值”, “最近”, “盘前”, or any other wording vocabulary, and it does not force a fixed comparison/deep template. Financials, holdings, news, and web results are evidence in Agent context, not cues for parsing its natural-language chapters.
- Dynamic interactive validation is limited to deterministic boundaries: server time, resolved entity, exact current quote/provider time, exact extended-session identity/session/freshness when present, false market-data denial, and one session/history/SSE result. The Agent retains control of scope, depth, structure, length, and priorities. Typed scheduled/heartbeat contracts retain their deterministic formats and strict field-aware validators. Any future strict validation of individual financial, holding, valuation, or event claims requires structured provenance tied to tool results.
- Genuine provider no-coverage/failure, equal-candidate ambiguity, omitted explicit seeds, and missing exact quote/time all preserve a successful Agent response. Omitted seeds may receive one hidden read-only continuation, but failure to build the optional contract afterward is diagnostic only and never authorizes a fixed refusal.
- All interactive drafts use the deferred output boundary. Tool progress remains visible, but draft deltas/resets are withheld; one deterministic-fact, read-only repair may preserve and correct the Agent answer, and the dynamic path never replaces it with a service-authored whole-answer fallback. Exactly one final answer is published.
- Repair retains the initial and retry tool traces in execution order and rejects unknown or persistent repair calls before accepting a revised answer.
- Programmatic `FmpConfig::default()` now matches serde defaults (`base_url` plus a 60-second timeout). This closes a separate zero-timeout path exposed by the strict tool-loop regression fixture; credentialed live checks continued to show the production FMP provider itself was healthy.
- The same main-agent-loop rule is now encoded in `soul.md`, the runtime finance policy, the canonical stock-research skill, repository invariants, repo map, and `D-2026-07-18-01`.

### Verification So Far

- Focused pure tests cover whole-trace search refinement, CRWV/CWY exact disambiguation, CWY-only rejection, explicit NBIS omission, CRWV+NBIS provider timestamps, per-equity financial/news tool-trace capture, tool aliases, partial quote rejection, contract-none answer preservation, and a generic no-search turn.
- AgentSession tests prove arbitrary surrounding wording, the exact production phrase, Chinese company names, and the dedicated financial-evidence round reach the real function-calling tool loop with `chat_calls() == 0`; exact DataFetch results produce a provider-time prefix and one visible answer. Repair tests cover merged traces plus unknown/persistent retry rejection.
- `FmpConfig::default()` is regression-covered against empty serde deserialization, including the nonzero timeout.
- Finance CI contract = 27/27, including a guard that rejects restoration of the auxiliary timeout/failure path.
- Credentialed live DataFetch regression passed. CRWV exact-resolved CoreWeave at `73.21`, NBIS exact-resolved Nebius at `177.71`, and one `CRWV,NBIS` batch quote returned fresh positive same-symbol records for both. Provider health was not the incident cause.
- `cargo test -p hone-channels --all-targets` = 611 passed after the output-boundary hotfix. Added regressions preserve Agent-owned no-coverage and equal-candidate clarifications, emit one final delta, prove shared-heading/free-form CRWV+NBIS valuation prose passes with observed financial/news evidence, reject a forged CRWV `15 USD` current price, and cover omitted-seed read-only continuation while retaining the complete audit trace.
- `cargo check --workspace --all-targets --exclude hone-desktop --exclude hone-user-app` and `cargo test --workspace --all-targets --exclude hone-desktop --exclude hone-user-app` passed. The only compiler diagnostic was the pre-existing unused-function warning for `feishu_direct_actor_contact_targets_from_records`.
- `PATH="$HOME/.bun/bin:$PATH" bun run test:web` passed 265/265; `bash tests/regression/run_ci.sh` passed every CI-safe regression, including finance contracts 27/27.
- Main-agent discovery commit `2d6b4be8` and output-boundary commit `8d4fcdd6` were pushed. The five runtime binaries were rebuilt; with active chat count at zero, supervisor `62866` was stopped by SIGINT without killing children individually and replaced by supervisor `78710`, backend `78722`, Discord `78978`, and Feishu `78997`. Ports 8077/8088 returned HTTP 200, and `/api/meta` reported healthy Postgres and S3 connections.
- The first deployed production replay of the exact phrase `分析下crwv和nbis的估值` reached the intended Agent loop. It searched, quoted, profiled, loaded financials, and loaded news independently for `CRWV` and `NBIS`; the log recorded `entity_resolution.agent_loop contract_built=true entities=CRWV,NBIS`. Provider/entity discovery therefore succeeded.
- That first replay was a failed acceptance result: legacy free-prose financial/event checks rejected the Agent answer, no final `assistant_delta` was published, and the run ended with one unsuccessful terminal event. Review also found the repair suffix indirectly reused the typed fixed comparison enforcement block. Commit `8d4fcdd6` removed both strong-interference paths while retaining exact quote/time/session checks.

### Final Production Acceptance

- A fresh production Web actor replayed the exact query `分析下crwv和nbis的估值` after the hotfix deployment. The main Agent made 10 DataFetch calls: independent search, quote, profile, financials, and news calls for both CRWV and NBIS. The log recorded `contract_built=true entities=CRWV,NBIS`.
- Run `8da0538d-8bad-42f0-a700-373f0a9edb83` completed successfully in 78.405 seconds. Its visible answer began with server Beijing time, named both verified entities, and included exact current quotes `73.21 USD` and `177.71 USD`. SSE contained exactly one `run_started`, one `assistant_delta`, and one successful `run_finished`, with no `assistant_reset`, `run_error`, or generic `error`.
- Persisted history contained exactly `user,assistant`; the user text matched byte-for-byte, and active chat count returned to zero. This closes the Interactive CRWV+NBIS phase. The umbrella ticker plan remains active only for the separately documented scheduler task-prose P2.

## Phase 3 — Remove The Unrequested Interactive Publication Ban

### Reopened Incident

At 20:06 Beijing time, production received `分析下crwv和nbis的估值` again. This was not an FMP/DataFetch outage: the retry trace contained empty exploratory searches for `CRWV CoreWeave` and `NBIS Nebius`, followed by successful exact `CRWV` and `NBIS` searches, exact quotes `73.21` and `177.71` with provider timestamps, both profiles and financial statements, and two web searches. The model completed a 6037-character valuation answer.

The response was lost locally. Commit `2d6b4be8` had introduced two coupled assumptions: dynamic reconstruction selected only the first assistant search group, and `AgentDiscoveryDisposition::UnsafeIncomplete` then converted an otherwise successful Interactive result into failure and replaced its content with a fixed “cannot safely publish” sentence. The first assumption ignored the Agent's successful refinement; the second was an unrequested publication ban that did not exist in the product prompt or configuration.

### Architecture Correction

- Interactive search evidence is now collected from the entire current runner trace. Earlier empty/failed broad or enriched attempts are skipped; later exact-symbol refinements remain authoritative when joined to positive same-symbol quotes and usable provider timestamps.
- Explicit ticker seeds bound the strong dynamic-contract scope. A CRWV request can use later CRWV refinement evidence without absorbing a separately researched CWY ETF; a CRWV+NBIS request must retain both exact seeds.
- The `UnsafeIncomplete` enum, fixed refusal constant, and success-to-failure response mutation are deleted. `contract_built=false` is logged as `answer_preserved=true`, then the existing normal path returns the successful Agent body unchanged.
- A dynamic Interactive contract is optional server fact enhancement, not permission to answer. Typed scheduled/heartbeat contracts, real runner/auth/quota errors, persistent-side-effect protection, and deterministic checks on a contract that was successfully built remain unchanged.
- The hidden omitted-seed continuation now explicitly tells the Agent to search an explicit ticker using the original code alone rather than concatenating a company name. The implementation still does not depend on prompt compliance: all later search refinements are considered structurally.
- Finance CI now rejects restoration of first-search freezing, `AgentDiscoveryDisposition`, or `UNSAFE_AGENT_DISCOVERY_MESSAGE` and requires both production-shape refinement coverage and contract-none answer preservation.

### Regression Evidence

- Guard coverage reproduces `CRWV CoreWeave` / `NBIS Nebius` empty results followed by exact CRWV/NBIS results, quotes, profiles, and financials; the rebuilt contract contains exactly CRWV and NBIS with `73.21` / `177.71`.
- AgentSession coverage starts with a no-tool historical draft, triggers the bounded seed retry, performs both empty enriched searches and later exact searches, builds the two-entity contract, drops the stale draft, and emits one successful final body.
- A separate AgentSession regression deliberately returns a CRWV quote without a usable timestamp so the optional contract cannot build; the Agent's concrete valuation/data-gap answer remains `success=true`, byte-for-byte intact, with one visible delta and no error.
- Existing Agent-owned no-coverage and equal-candidate clarification tests continue to pass. Finance automation contracts pass 27/27.
- `cargo test -p hone-channels --all-targets` passes 614/614; workspace `cargo check` and `cargo test` pass with the existing unrelated Feishu dead-code warning only. Web tests pass 265/265 and `tests/regression/run_ci.sh` passes every CI-safe contract.

### Remaining Acceptance

The implementation and focused regressions are complete. Before marking this reopened phase done, run the full repository gates, push `main`, rebuild the five runtime binaries, drain active chats, restart the supervisor, verify Postgres/S3 and ports 8077/8088, and replay both `分析下crwv和nbis的估值` and `crwv和nbis的估值怎么看` with fresh actors. Each production run must yield a normal answer, one successful terminal event, no fixed refusal/reset/error, and zero active chats afterward. First-visible-text latency remains a separate follow-up because the current Interactive path still defers body deltas until the tool loop and optional contract handling finish.
