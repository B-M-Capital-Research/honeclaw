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
- Let the interactive Agent's actual tool choices determine evidence depth and its complete-query reasoning determine answer shape. Runtime guards may add server time/canonical facts and reject unsupported claims, but must not infer a fixed route from phrases, force chapters, replace an answer with a deterministic template, or turn genuine provider no-coverage/ambiguity into a generic entity failure.
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
- 2026-07-18: post-review removed the remaining interactive keyword/depth classifier, fixed-section validator, and deterministic whole-answer fallback. Dynamic contracts now follow only the tools the Agent actually selected and enforce truth rather than presentation. Genuine no-coverage or equal-candidate ambiguity preserves the Agent's specific explanation; only omitted explicit seeds or unique entities lacking exact quote/time remain unsafe incomplete. Regression coverage includes deliberately unmodeled wording plus shallow, financial, news, web, no-coverage, and ambiguity traces.
- This umbrella plan remains `in_progress`, rather than being archived, because the post-restart scheduler window still reproduces the existing task-prose/entity P2 tracked in `docs/bugs/scheduler_finance_entity_guard_misclassifies_instruction_words.md`.

## Risks / Open Questions

- Some real tickers are ordinary English or technical acronyms; provider existence alone is insufficient when the user's context clearly refers to the concept rather than the security.
- Exchange suffixes such as user-facing `.SH` and provider-facing `.SS` require explicit canonicalization without accepting a different listing or share class.
- Unsupported instruments must remain honest failures; architecture work must not degrade into fuzzy first-result selection, web-memory identity, or wrong-symbol quote acceptance.
- Multi-security completeness must never silently drop a named peer: high-confidence explicit code seeds are checked only after the Agent runs, every such seed must appear in the first main-Agent search group, and every searched entity needs later exact quote evidence before a multi-security contract can be built. Seeds remain a minimum rather than a closed entity set, so the Agent can add names and aliases found from the full query.
- Prompt guidance must remain rich enough to produce the established investment style, but runtime truth enforcement and answer composition are separate concerns. Do not reintroduce an interactive phrase dictionary or fixed formatter to compensate for a weak model draft; improve Agent context/tool guidance and test the resulting loop instead.
- Numeric market/asset words must remain bound to their own source span; whole-query hints, value literals, and pre-resolution deduplication are prohibited.
- Scheduler and heartbeat task prose still needs a typed subject boundary: current live failures include `800G`, `NAND`, truncated `AST`, and `SEC` being sent to quote resolution. Keep this separate from the now-closed CRWV/CWY relation bug and do not mark the umbrella plan done until the existing P2 passes a live scheduler window.
