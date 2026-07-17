# RKLB Entity And Market Data Resolution Regression

- title: RKLB entity and current-market-data resolution regression repair
- status: done
- created_at: 2026-07-17
- updated_at: 2026-07-17
- owner: Codex
- related_files: `crates/hone-channels/src/investment_response_guard.rs`, `crates/hone-channels/src/agent_session/tests.rs`, `tests/regression/ci/test_finance_automation_contracts.sh`, `tests/regression/manual/test_entity_search_live.sh`
- related_docs: `docs/invariants.md`, `docs/decisions.md`, `docs/repo-map.md`, `docs/handoffs/2026-07-17-rklb-entity-resolution-repair.md`

## Goal

Make an ordinary `RKLB` request deterministically resolve Rocket Lab USA, Inc., retrieve the exact same-symbol latest available quote through DataFetch, and produce the complete time-first investment response without a false data-unavailable result.

## Completed Scope

- Compared the three failing production turns, persisted replies, tool timing, and direct FMP/DataFetch results.
- Removed the auxiliary-LLM dependency from complete current-turn ticker requests and ticker identity-binding phrases.
- Preserved the explicit ticker as the provider query and added one exact-symbol profile fallback for semantic-empty or derivative-only search results.
- Classified safe-range, margin-of-safety, fair-value, recommended-entry, and equivalent decision questions as deep valuation requests so they cannot fall through the quote-only validator.
- Added unit, AgentSession, CI contract, and live provider regression coverage for RKLB and structural boundaries.
- Pushed both fixes, passed required CI, rebuilt all runtime packages, completed two controlled drain/restart cycles, and ran final isolated production Web/SSE cases.

## Verification

- Root cause evidence: the original turns either waited for the 15-second auxiliary entity extractor or let its company alias replace the exact `RKLB` query; FMP/DataFetch itself was healthy.
- Live DataFetch: exact Rocket Lab USA, Inc. search/profile, `67.35 USD`, `-11.61417%`, provider timestamp `1784232000`, non-ETF equity classification, and four annual financial periods. RMBS, NBIS, INTL, BTCUSD, and mixed-market probes also passed.
- Local automated: `hone-channels` 569/569, `hone-tools` 136/136 with one expected ignored test, finance contracts 24/24, focused entity/deep-intent tests, format, and diff checks.
- GitHub: CI `29570821727`, Secret Scan `29570821700`, and Code Quality `29570820863` succeeded for final code commit `7d14c87f`.
- Production: all three original RKLB phrases exact-resolved to `deep_analysis=Equity`; every SSE had one start, one assistant answer, one successful terminal, zero reset/error, all nine sections, one persisted user plus one assistant, and active-run count zero afterward.
- Runtime health: supervisor/backend `74062`/`74073`; Discord `74274`; Feishu `74292`; one shared backend listener on 8077/8088; Postgres and S3 healthy; cloud storage authoritative; zero local durable dependencies; local/origin/public auth probes returned the expected 401.

## Documentation Sync

- Updated `docs/invariants.md`, `docs/decisions.md`, and `docs/repo-map.md` with the exact-ticker, query-integrity, profile-fallback, and deep valuation-intent contracts.
- Added `docs/handoffs/2026-07-17-rklb-entity-resolution-repair.md` and this archive entry; removed the task from the active plan index.

## Risks / Follow-ups

- A real provider error still fails closed; the profile fallback is only for a successful semantic-empty or derivative-only search and never accepts a different symbol.
- When verified valuation inputs are insufficient, the deterministic nine-section fallback deliberately refuses to invent a numeric “safe range” and tells the user which evidence is missing.
- No schema or durable-data migration occurred. Rollback is the prior binaries plus the controlled supervisor restart.
