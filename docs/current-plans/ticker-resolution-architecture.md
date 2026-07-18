# Cross-Market Ticker Resolution Architecture

- title: Cross-market ticker resolution architecture repair
- status: in_progress
- created_at: 2026-07-17
- updated_at: 2026-07-18
- owner: Codex
- related_files: `crates/hone-channels/src/security_identifier.rs`, `crates/hone-channels/src/investment_response_guard.rs`, `crates/hone-tools/src/data_fetch.rs`, `crates/hone-channels/src/agent_session/tests.rs`, `tests/regression/`
- related_docs: `docs/invariants.md`, `docs/decisions.md`, `docs/repo-map.md`, `docs/handoffs/`

## Goal

Replace one-off ticker exceptions with a single deterministic architecture that recognizes ordinary security-code inputs across supported markets, preserves the user's explicit symbol through provider lookup, distinguishes real symbols from weak acronym collisions, and fails accurately only when the exact provider-backed entity or same-symbol quote is genuinely unavailable.

## Scope

- Audit recent production failures and the full ticker path: lexical candidate extraction, confidence/context rules, provider normalization, exact matching, semantic-empty fallback, quote verification, and response-contract routing.
- Cover ordinary US tickers, short symbols that overlap common acronyms, share-class punctuation, exchange-qualified digit-leading symbols, provider suffix aliases, index prefixes, crypto pairs, invalid symbols, and mixed explicit/named comparisons.
- Move market-symbol syntax and provider canonicalization into reusable deterministic helpers rather than scattered residual-language exceptions.
- Preserve false-positive safety: weak finance/technology acronyms must not silently become companies; ambiguous code shapes require stronger ticker context or clarification.
- Add unit, AgentSession, CI-safe contract, live provider, and production Web/SSE regression matrices.
- Keep bounded exact-quote batches, concurrent semantic searches, reusable probe evidence, and concurrent profile enrichment for typed scheduled/heartbeat preparation; interactive turns use the main Agent's current-query `search → exact quote/profile → problem evidence` loop.
- Keep exact ticker identity above weak semantic substring matches: single-stock ETF/ETN/leveraged/yield product names that embed CRWV/RKLB/AAPL-like underlying codes cannot create a false company-name conflict, while Ford/FORD and Apple name fallback remain supported only through word-bounded strong name relations (so Apple does not match Appleseed).
- Treat interactive lexical scans only as candidate seeds. Every nonempty interactive wording—including portfolio/watchlist wording—continues through the same main Agent loop, which reads the complete query, invokes `portfolio(view)` there when membership matters, searches all named securities, then exact-verifies quote/profile evidence; no fixed phrase grammar or closed-ticker shortcut may decide the interactive entity set before the Agent runs.
- Let the interactive Agent's actual tool choices load evidence into context and its complete-query reasoning determine scope, depth, priorities, and answer shape. Runtime guards may add server time/canonical facts and reject deterministic entity/quote/session contradictions, but must not infer a fixed route from phrases, force chapters, parse arbitrary financial/news prose, replace an answer with a deterministic template, or turn genuine provider no-coverage/ambiguity into a generic entity failure. Exact future per-claim validation requires structured provenance rather than Markdown section parsing.
- Push the fix, wait for required CI, rebuild runtime binaries, drain/restart through the supervisor, and verify storage/channel/API health.

## Validation

- Focused extraction/normalization/exact-match tests for every supported symbol class and ambiguity boundary.
- Full `hone-channels` and relevant `hone-tools` tests plus finance CI contracts and proportional repository gates.
- Credentialed live DataFetch search/profile/quote probes for representative US, international, index, and crypto symbols without exposing credentials.
- Live CRWV provider proof plus CRWV/CWY, RKLB/RKLX, AAPL/AAPU, Ford/FORD, Apple/Appleseed, derivative-only, and natural-name fallback regressions.
- Isolated production turns proving time-first output, exact entity and same-symbol quote, Agent-selected valuation evidence and organization, one answer/terminal, no reset/error, and zero active chats afterward.

## Documentation Sync

- Keep this task indexed in `docs/current-plan.md` while active.
- Update `docs/invariants.md`, `docs/decisions.md`, and `docs/repo-map.md` for the durable symbol grammar, confidence boundary, provider normalization, and failure semantics.
- On completion, write a reusable handoff, move this plan to `docs/archive/plans/`, remove the active index entry, and add `docs/archive/index.md` evidence.

## Progress

- 2026-07-18: direct interactive `CRWV` false ambiguity is fixed and deployed in `b87c4cb7`. Exact CRWV now outranks CWY and other product names that only reference the underlying ticker; word-bounded Apple/Appleseed and genuine Ford/FORD behavior are regression-covered. Production quote-only and deep valuation turns both completed with one answer and one successful terminal event. See `docs/handoffs/2026-07-18-crwv-entity-resolution-repair.md`.
- 2026-07-18: the production multi-ticker failure `分析下crwv和nbis的估值` was traced to residual-word/comparison completeness logic followed by the shared 15-second auxiliary entity timeout; neither DataFetch nor the main runner had started. Every nonempty interactive request now enters `AgentToolDiscovery`: the scanner supplies non-factual seeds only, the configured main runner determines the actual scope from the full query, and structured search/quote/profile/problem-evidence results from that same run build the server validation contract afterward. Explicit code seeds are a post-run minimum—never a complete set, but they cannot be silently omitted. The auxiliary pre-run extraction/failure path is removed. Prompt, skill, CI, live-provider, and production restart evidence is tracked in the same-day handoff.
- 2026-07-18: post-review removed the remaining interactive keyword/depth classifier, fixed-section validator, and deterministic whole-answer fallback. Dynamic contracts derive entity/quote scope from the tools the Agent selected and enforce deterministic truth rather than presentation. A later incident proved that even omitted seeds or missing exact quote/time may not become a service-owned publication ban: the optional contract can withhold server-certified facts, while the successful Agent response remains intact. Regression coverage includes deliberately unmodeled wording plus shallow, financial, news, web, no-coverage, ambiguity, repeated-search, and contract-none traces.
- 2026-07-18: the first exact production replay after deployment confirmed the new entity/tool architecture but exposed a separate output-boundary regression: the dynamic repair path indirectly reused typed comparison instructions and legacy validators parsed unconstrained valuation/news prose as fixed chapters. Commit `8d4fcdd6` removed typed-format injection and arbitrary prose financial/event parsing from Interactive validation while retaining deterministic time/entity/quote/session guards. After rebuild and a graceful zero-active-chat restart, a fresh exact replay made all 10 expected CRWV/NBIS search/quote/profile/financials/news calls, logged `contract_built=true entities=CRWV,NBIS`, emitted one time-first answer and one successful terminal event with no reset/error, persisted exactly one user/assistant pair, and returned active chats to zero in 78.405 seconds.
- 2026-07-18: a later production run exposed a second regression introduced by `2d6b4be8`, not a provider outage or configured policy. The retry first searched `CRWV CoreWeave` / `NBIS Nebius` and received empty rows, then successfully refined to exact `CRWV` / `NBIS`, loaded same-symbol quote/profile/financials/web evidence, and generated a 6037-character answer. The runtime reconstructed only the first assistant search group, returned `contract_built=false`, and an unrequested `UnsafeIncomplete` branch converted the successful response into a fixed refusal. The fix removes that publication veto and disposition, aggregates all current-run search refinements, skips earlier empty attempts, restricts explicit-ticker contracts to the requested exact seeds so auxiliary products are not absorbed, and treats contract construction as optional observation rather than answer authorization. Focused tests now reproduce the production sequence and prove that even deliberate contract failure preserves the Agent body, success state, and one-output boundary.
- 2026-07-18: commits `fcca5a35` / `54b14068` completed that repair plus an independent mixed-alias subset safeguard. All 615 channel tests, workspace check/test, 265 Web tests, and CI-safe regressions passed. After a zero-active-chat SIGINT restart, both fresh production queries `分析下crwv和nbis的估值` and `crwv和nbis的估值怎么看` logged `contract_built=true entities=CRWV,NBIS`, returned time-first answers with `73.21` / `177.71`, emitted exactly one answer and successful terminal event with no reset/error/refusal, persisted one user/assistant pair, and returned active chats to zero.
- This umbrella plan remains `in_progress`, rather than being archived, because the post-restart scheduler window still reproduces the existing task-prose/entity P2 tracked in `docs/bugs/scheduler_finance_entity_guard_misclassifies_instruction_words.md`.

## Risks / Open Questions

- Some real tickers are ordinary English or technical acronyms; provider existence alone is insufficient when the user's context clearly refers to the concept rather than the security.
- Exchange suffixes such as user-facing `.SH` and provider-facing `.SS` require explicit canonicalization without accepting a different listing or share class.
- Unsupported instruments must remain honest failures; architecture work must not degrade into fuzzy first-result selection, web-memory identity, or wrong-symbol quote acceptance.
- Multi-security completeness must never silently drop a named peer: high-confidence explicit code seeds are checked after the Agent runs, every such seed must receive exact provider coverage somewhere in the current search/refinement trace, and every entity entering the verified contract needs later exact quote evidence. The runtime must not freeze the first search group. Seeds remain a minimum rather than a closed entity set, while auxiliary products/benchmarks cannot silently expand a strong explicit-ticker contract without future structured scope provenance.
- Prompt guidance must remain rich enough to produce the established investment style, but runtime truth enforcement and answer composition are separate concerns. Do not reintroduce an interactive phrase dictionary or fixed formatter to compensate for a weak model draft; improve Agent context/tool guidance and test the resulting loop instead.
- Free-form interactive financial/news claims cannot be reliably approved or rejected by looking for ticker headings, numbered sections, or matching numbers anywhere in a clause. Keep those parsers on typed scheduled/heartbeat output only; require structured claim provenance before adding strict per-claim enforcement to the interactive path.
- Numeric market/asset words must remain bound to their own source span; whole-query hints, value literals, and pre-resolution deduplication are prohibited.
- Scheduler and heartbeat task prose still needs a typed subject boundary: current live failures include `800G`, `NAND`, truncated `AST`, and `SEC` being sent to quote resolution. Keep this separate from the now-closed CRWV/CWY relation bug and do not mark the umbrella plan done until the existing P2 passes a live scheduler window.
