# CRWV Exact-Ticker Entity Resolution Repair

- title: CRWV exact-ticker versus embedded-product entity resolution repair
- status: done
- created_at: 2026-07-18
- updated_at: 2026-07-18
- owner: Codex
- related_files: `crates/hone-channels/src/investment_response_guard.rs`, `tests/regression/ci/test_finance_automation_contracts.sh`, `tests/regression/manual/test_entity_search_live.sh`
- related_docs: `docs/current-plans/ticker-resolution-architecture.md`, `docs/decisions.md#d-2026-07-17-04-resolve-securities-through-a-span-aware-exact-first-pipeline`, `docs/invariants.md`, `docs/repo-map.md`, `docs/bugs/scheduler_finance_entity_guard_misclassifies_instruction_words.md`, `docs/runbooks/backend-deployment.md`
- related_prs: commits `4d419770`, `b87c4cb7`

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
