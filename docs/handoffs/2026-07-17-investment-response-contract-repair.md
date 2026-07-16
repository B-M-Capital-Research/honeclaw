# Investment Response Contract Repair

- title: Investment response entity, live-data, template, and stream repair
- status: in_progress
- created_at: 2026-07-17
- updated_at: 2026-07-17
- owner: Codex
- related_files: `soul.md`, `crates/hone-channels/src/investment_response_guard.rs`, `crates/hone-channels/src/agent_session/{core,emitter,tests}.rs`, `crates/hone-channels/src/prompt.rs`, `crates/hone-tools/src/{data_fetch,web_search}.rs`, `crates/hone-web-api/src/{state.rs,routes/chat.rs}`, `packages/app/src/{lib/public-chat.ts,pages/chat.tsx}`
- related_docs: `docs/current-plans/investment-response-template-regression.md`, `docs/invariants.md`, `docs/decisions.md`, `docs/repo-map.md`
- related_prs: none yet

## Summary

The investment response path has been rebuilt around a deterministic service-owned contract rather than relying on the model to remember the old format. The visible answer begins with Beijing data time, then exact normalized entity and same-symbol DataFetch quote facts. The model body receives asset-appropriate evidence and must follow the restored full response template. Guarded drafts stay hidden until validation and publish as one final assistant message with one terminal stream event.

Live FMP/DataFetch and Tavily diagnostics succeeded. The production NBIS/RMBS/INTL failures were not a general provider outage: they came from internal entity/asset routing, false company-financial requirements for funds, and format validation/repair that could discard a valid quote context or spend a long time retrying.

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
- Reduced provider evidence before prompt injection. Conflicting profile snapshot prices and unsupported financial interpretations are removed. Every claimed event fact must use a verified real absolute date and matching full source domain in the same clause; otherwise it is explicitly inference, hypothesis, or scenario.
- Deferred investment candidate deltas/resets/thoughts/errors behind validation, then emitted exactly one canonical assistant answer. Session `Done` is the sole `run_finished` authority, so late frames cannot create a second flash/run.
- Kept Web run state server-authoritative. Refresh resumes the same `run_id` and original `started_at_ms`; it does not repost the prompt or reset elapsed time. A missing runner with an unanswered persisted user turn becomes an explicit interruption.
- Contract repair reuses the sanitized draft once and never replays the original operation. Requests that can persist, send, schedule, update, or delete are execute-once when traces are absent or uncertain.

## Verification

Completed before this handoff:

- Investment guard focused tests: `56/56`.
- AgentSession focused tests: `79/79`.
- Full `hone-channels` tests: `549/549`.
- `hone-web-api`: `117 passed`, `2 ignored`.
- Prompt tests: `12/12`.
- Finance static contract checks: `24/24`.
- Frontend tests: `265/265`.
- Live provider probes passed exact entity/quote paths for NBIS, RMBS, INTL, and BTCUSD, including equity financial/news and fund holdings routes.
- `scripts/diagnose_fmp_tavily.sh` reported both FMP and Tavily healthy during the incident investigation.

TODO before marking the plan done:

- Record final post-rebase `cargo check`, full workspace tests, frontend typecheck/tests, and `tests/regression/run_ci.sh` results.
- After deployment, capture isolated RMBS, NBIS, and INTL user-response samples proving first-line time, exact quote, asset-specific template, and no false live-data denial.
- Verify SSE counts for a guarded turn: one `run_started`, one final `assistant_delta`, zero `assistant_reset`, zero `run_error`, and one successful `run_finished`.
- Record controlled restart evidence: new PID, `/api/meta`, active-run drain/count, public-port auth behavior, and singleton backend/channel processes.

## Risks / Follow-ups

- The auxiliary company-name extractor intentionally fails closed after 15 seconds. The deterministic exact ticker path is independent, but ambiguous company names may require the user to retry or provide a ticker when the auxiliary route is unavailable.
- Strict nine-section validation can still add model latency. Keep monitoring final-answer latency and missing-section repair rates without weakening exact quote/entity correctness.
- Portfolio preflight is read-only. An explicit add/update/delete request must still execute its mutation once; never report the preflight read as mutation success.
- This change has no database, schema, or durable-storage migration. Rollback is code/asset-only: restore the previous server/frontend revision, rebuild, drain active runs, and perform the controlled restart. Do not delete or transform actor sessions, portfolios, or other durable data.

## Next Entry Point

Continue from `docs/current-plans/investment-response-template-regression.md`: complete the final workspace gates, merge/rebase, rebuild and deploy, perform the controlled restart and health checks, then run the real RMBS/NBIS/INTL response and SSE regressions. Once all evidence is recorded, mark this handoff done, archive the plan, remove its active index entry, and add the archive index link.
