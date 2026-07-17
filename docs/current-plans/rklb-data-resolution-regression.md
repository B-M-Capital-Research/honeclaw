# RKLB Entity And Market Data Resolution Regression

- title: RKLB entity and current-market-data resolution regression repair
- status: in_progress
- created_at: 2026-07-17
- updated_at: 2026-07-17
- owner: Codex
- related_files: `crates/hone-channels/src/investment_response_guard.rs`, `crates/hone-channels/src/agent_session/core.rs`, `crates/hone-tools/src/data_fetch.rs`, `tests/regression/`
- related_docs: `docs/invariants.md`, `docs/decisions.md`, `docs/repo-map.md`, `docs/handoffs/`

## Goal

Make an ordinary `RKLB` request deterministically resolve Rocket Lab USA, Inc., retrieve the exact same-symbol latest available quote through DataFetch, and produce the complete time-first investment response without a false data-unavailable result.

## Scope

- Compare the newest production RKLB request, persisted response, tool trace, and timing against direct DataFetch/FMP probes.
- Identify whether the failure is ticker extraction, search/profile/quote normalization, provider/cache behavior, evidence routing, or final response validation/fallback.
- Implement the smallest durable fix for RKLB and structurally equivalent symbols without weakening entity or price verification.
- Treat safe-range, margin-of-safety, fair-value, recommended-entry, and equivalent decision questions as deep valuation requests so they cannot fall through the quote-only validator or bypass the established nine-section answer contract.
- Add automated regression coverage for the observed failure and nearby false-positive/false-negative boundaries.
- Push the fix, wait for required CI, drain/restart the runtime safely, then run an isolated production RKLB Web/SSE turn and health checks.

## Validation

- Focused unit tests for the failing entity/data/guard boundary.
- Relevant `hone-tools` and `hone-channels` test subsets, then the proportional workspace/CI-safe gates.
- Direct live DataFetch RKLB search, quote, profile, and applicable financial/news probes.
- Production RKLB E2E proving first-line Beijing time, exact entity/current quote, required template, one terminal stream, no false data denial, and zero active chats afterward.
- Repeat the original production phrases after the valuation-intent follow-up and reject any answer that contains an unverified price range, omits the nine-section template, or exposes more than one terminal stream.

## Documentation Sync

- Keep this task indexed in `docs/current-plan.md` while active.
- Update `docs/invariants.md`, `docs/decisions.md`, or `docs/repo-map.md` only if the durable contract or module flow changes.
- On completion, write or update a reusable handoff, move this plan to `docs/archive/plans/`, remove the active index entry, and add `docs/archive/index.md` evidence.

## Risks / Open Questions

- A provider outage or semantic-empty response must remain distinguishable from internal entity-routing failure.
- The repair must not accept fuzzy first-result matches, stale cached empties, conflicting profile prices, or wrong-symbol quotes.
- Production validation must not replay persistent user operations or create duplicate terminal stream events.
