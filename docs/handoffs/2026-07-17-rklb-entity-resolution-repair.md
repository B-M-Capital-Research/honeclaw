# RKLB Entity And Deep Valuation Response Repair

- title: RKLB entity, market-data, and deep valuation response repair
- status: done
- created_at: 2026-07-17
- updated_at: 2026-07-17
- owner: Codex
- related_files: `crates/hone-channels/src/investment_response_guard.rs`, `crates/hone-channels/src/agent_session/tests.rs`, `tests/regression/ci/test_finance_automation_contracts.sh`, `tests/regression/manual/test_entity_search_live.sh`
- related_docs: `docs/archive/plans/rklb-data-resolution-regression.md`, `docs/invariants.md`, `docs/decisions.md`, `docs/repo-map.md`, `docs/runbooks/backend-deployment.md`
- related_prs: commits `ff3852c3`, `7d14c87f`; GitHub CI `29570821727`

## Summary

RKLB was not failing because FMP, DataFetch, Tavily, credentials, or cache were down. Hone already found the lexical ticker but still required an auxiliary LLM to consume every remaining business word. “推荐 / 安全区间 / 中子” therefore caused a 15-second entity-extraction wait, and an auxiliary alias could replace the reliable provider query `RKLB` with an FMP-sensitive company-name string. `RKLB 是前面提到的 火箭实验室` was also missing deterministic ticker-binding context.

The first production deployment fixed entity and quote resolution, then exposed a second routing defect during acceptance: “推荐的安全区间价格” was classified as quote-only because deep intent did not recognize safe-range language. That allowed an incomplete, unverified non-nine-section draft through. The follow-up makes valuation/action wording deep before quote-only detection and keeps the code-level nine-section validator authoritative.

## What Changed

- Interactive requests containing only explicit ticker entities bypass auxiliary extraction unless an explicit incomplete comparison still needs another named security.
- The exact symbol is the immutable DataFetch search key; auxiliary aliases cannot rewrite it. A successful search with no exact candidate may use one exact-symbol profile fallback, but wrong/derivative symbols and provider errors remain rejected, and same-symbol quote verification is still mandatory.
- Uppercase and contextual lowercase ticker identity bindings such as `RKLB 是...` now enter the deterministic path without weakening the DCF/API/GPU acronym denylist.
- Safe-range, safety-margin, fair-value, buy/entry-range, and recommendation-decision phrases enter the deep equity/fund/crypto contract and cannot be treated as quote-only merely because they contain “price”. Explicit quote-only wording such as `只报现价，不要推荐` stays concise.
- Regression coverage preserves the three production phrases, lowercase binding, comparisons, alias merging, profile fallback, quote-only negation, and live RKLB market data.

## Verification

- Local tests: `cargo test -p hone-channels` = 569 passed; `cargo test -p hone-tools` = 136 passed with one expected ignored test; finance automation contracts = 24/24.
- Live provider: Rocket Lab USA, Inc. / RKLB exact search and non-ETF profile; quote `67.35 USD`, change `-11.61417%`, timestamp `1784232000`; four annual financial periods.
- Final GitHub code SHA `7d14c87fe980e12be552a00dd45e0b8af7d08e62`: CI `29570821727`, Secret Scan `29570821700`, and Code Quality `29570820863` succeeded.
- Final runtime: supervisor `74062`, backend `74073`, Discord `74274`, Feishu `74292`; PG/S3 healthy, cloud authoritative, zero local durable dependencies, one backend listener, active chat count zero.
- Production actor base `codex-rklb-final-1784281587`: all three isolated cases resolved `entities=RKLB deep_analysis=Equity`. Each had `run_started=1`, `assistant_delta=1`, `run_finished=1`, `assistant_reset=0`, `run_error=0`, terminal success, nine numbered sections, and a 1-user/1-assistant persisted transcript.

## Risks / Follow-ups

- The response may use the server-owned deterministic nine-section fallback when the model draft contains unsupported historical prices, forward figures, incomplete sections, or improperly sourced event claims. This is intended; it favors a disclosed evidence gap over an invented safe-price range.
- Semantic-empty profile fallback is deliberately narrow. Do not broaden it to fuzzy first-result matching or invoke it after provider/search errors.
- No database, session, quota, portfolio, or storage migration was required.

## Next Entry Point

Start at `extract_entity_scope`, `prepare_verified_investment_turn`, `response_intent`, and `is_strict_quote_only_request` in `crates/hone-channels/src/investment_response_guard.rs`. For production restart and active-run drain semantics, use `docs/runbooks/backend-deployment.md` and send SIGINT only to the `hone-cli start` supervisor.
