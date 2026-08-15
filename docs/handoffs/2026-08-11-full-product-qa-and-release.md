# 2026-08-11 HONE 全产品压力、功能测试与上线判断

- title: 2026-08-11 HONE 全产品压力、功能测试与上线判断
- status: `blocked`
- created_at: `2026-08-11`
- updated_at: `2026-08-11`
- owner: `Codex / stress-test Agent / functional-test Agent`
- related_files:
  - `crates/hone-web-api/src/routes/`
  - `packages/app/src/`
  - `tests/regression/ci/`
  - `data/daily_signals/ai/latest.json`
  - `data/portfolio_news/`
- related_docs:
  - `docs/current-plans/full-product-qa-and-release-2026-08-11.md`
  - `docs/runbooks/backend-deployment.md`
  - `docs/runbooks/source-web-startup.md`
- related_prs: none; no commit, push or deployment was performed

## Summary

Two independent Agents tested the 2026-08-10/11 HONE product batch, after which the primary Agent completed the code and test-data remediation authorized by the product owner. The stress Agent found no crash or response corruption under 30,400 local requests. The remediation retest added 6,000 authenticated keep-alive API requests at concurrency 50, again with zero failures. All local repository gates are now green.

The release decision remains **NO-GO**, but the remaining reasons are external/runtime and release-provenance gates rather than known failing code tests: the safe function-calling conversation model is unconfigured, FMP/search/model providers have no usable credentials, and the shared local `main` is 23 commits behind `origin/main` with a large dirty/untracked product batch. HONE now fails closed instead of presenting simulated or incomplete evidence as a formal rating.

No production deployment, commit or push was performed. One test-only research-library record (`hone-research-library-sample`) was deleted through the actor-scoped product API, and the affected portfolio-news report was refreshed. This scoped deletion is not backed up because the record explicitly existed only for verification.

## Verification

### Automated gates

| Gate | Result | Evidence |
|---|---|---|
| Web API library | PASS | 298 passed, 0 failed, 2 credentialed live tests ignored |
| Full Web suite | PASS | 465 passed, 0 failed |
| TypeScript | PASS | `tsc -p packages/app/tsconfig.json` |
| Public production build | PASS with size warning | `research-preview` about 1.44 MB; chat chunk about 198 KB |
| Rust format / diff | PASS | formatting and whitespace checks passed |
| CI-safe regression | PASS | `tests/regression/run_ci.sh` completed successfully |
| Local source deploy contract | PASS in fake boundary only | no real runtime or production was changed |

Remediation covered the English workspace strings and question hooks, the undefined design token, large-text coverage for the composer/tool shelf/hooks, and the CI shell/wording contracts.

### Stress and stability

- Environment: local macOS, 8 logical CPUs / 16 GiB, Vite `3001`, admin/API `8077`, public API/static `8088`.
- Business API requests: 20,400; failures: 0.
- Static requests: 10,000; failures: 0.
- Total requests: 30,400; success rate: 100%.
- No backend crash, port loss, invalid JSON, response-length drift or sustained RSS growth.
- At concurrency 50, core API p95: company ratings 118 ms, key-event chains 80 ms, macro 148 ms, weekly brief 121 ms.
- At mixed concurrency 60, p99 rose to 0.97–1.47 s. The likely local cause is synchronous SQLite `session.last_seen_at` writes on authenticated reads; this inference requires a cloud PostgreSQL retest.
- Peak backend CPU was about 594% and peak RSS 44.5 MiB; post-test RSS returned to about 34.5 MiB.
- Vite development static p95/p99 reached roughly 1.4/3.6 s under concurrency 50. This is a development-server observation, not a production bundle benchmark.

Remediation keep-alive retest, 1,000 requests per authenticated endpoint at concurrency 50:

| Endpoint | Failed | p50 | p95 | p99 | Throughput |
|---|---:|---:|---:|---:|---:|
| Company ratings | 0 | 35 ms | 55 ms | 64 ms | 1,388 req/s |
| AI daily signal | 0 | 6 ms | 10 ms | 12 ms | 7,945 req/s |
| Portfolio news | 0 | 8 ms | 13 ms | 16 ms | 6,135 req/s |
| Position management | 0 | 7 ms | 12 ms | 15 ms | 6,931 req/s |
| Weekly brief | 0 | 27 ms | 43 ms | 50 ms | 1,807 req/s |
| Key-event chains | 0 | 23 ms | 37 ms | 43 ms | 2,073 req/s |

### Browser acceptance

PASS at 1280×720 and 390×844:

- Ten tools remain in one horizontally scrollable row; desktop exposes six full buttons plus a seventh affordance.
- Five new-conversation question hooks render.
- Tool shelf, messages and composer do not overlap; no page/dialog horizontal overflow was observed.
- Macro dashboard exposes 16 evidenced factors including 10Y, 30Y, Fed Funds, employment-population ratio, unemployment, payrolls and VIX.
- AI dashboard is truthfully `部分数据`: only MSFT is 7/7 complete, the summary states 1/4 coverage, and missing gross-margin inputs remain blank. Saved snapshots are normalized defensively so stale `live` metadata cannot leak into the page.
- Company rating simulation is disabled. All 52 transcript-only records use a neutral `研究基线` state, show `—` for formal rating, and separately expose the research-structure score and 3/8 factor coverage. No red/yellow/green daily investment conclusion or simulated valuation is presented.
- Weekly Brief is structured text, includes Hot Chips and NVIDIA's official schedule, and no longer lives inside the key-event chain.
- Key-event chain exposes 12 first-principles topics and 40 source-linked events.
- Research Library, My and Community retain the intended trust separation.
- News, position, influencer and event-chain products fail closed when the analysis model is unavailable.

## Remediation Completed

1. AI snapshot/API/UI status now requires every required company metric before claiming `live`; partial data is labeled and counted explicitly.
2. The verification-only research item was removed through the actor-scoped API, portfolio news was regenerated, and no sample marker remains in the research library or portfolio-news response.
3. Company-rating simulation was disabled. Transcript research baselines cannot claim formal traffic lights or formal scores; valuation remains excluded until fresh quotes, fundamentals, sources and multi-method valuation pass their gates.
4. English workspace strings and new-conversation hooks were localized; Chinese internal role/source tokens were translated for presentation.
5. Large-text mode now covers the composer, daily-tool titles and new-conversation question hooks.
6. Full Web, Web API, TypeScript, build and CI-safe regression gates all pass after remediation.

## Remaining Blocking Findings

### P1 — must fix before release

1. **Core conversation runtime is unconfigured.** `agent_runner=codex_acp`, but the inspected local configuration has zero usable OpenRouter/provider keys and no configured function-calling model. A normal public turn therefore correctly returns the safe-executor unavailable state. Configure the actor-safe model/API and pass real golden conversational canaries before release.
2. **Real evidence providers are unconfigured.** The inspected local configuration has zero FMP keys and zero search-provider keys. Consequently current quotes, financials and valuation cover 0/52 companies; position advice remains `insufficient_data`, while news/influencer/event outputs can remain source-only. Missing credentials must not be replaced with simulated facts.
3. **There is no releasable revision provenance.** Local `main` is 23 commits behind `origin/main` and contains a large dirty/untracked batch. Integrate onto a clean `codex/` review branch, resolve overlap with current upstream, rerun all gates on the exact revision, then use the approved deployment path.

### P2 — fix in the same hardening pass when practical

- Chinese UI exposes internal tokens such as `leading`, `confirmation`, `risk`, `lagging`, `financial_conditions`, `market_risk` and `ai_layer`; English mode leaves most new tools in Chinese.
- Large text increases message prose but leaves the 16 px composer, 13.5 px tool titles and hook cards largely unchanged.
- One valid two-page SpaceX PDF has an abnormal 1486×14400 pt page and fails text extraction; the Seeking Alpha PDF retains navigation/legal noise.
- Influencer source-only mode admits low-value posts such as `yes` and follower-count chatter; add source-content quality filters.
- Portfolio contains `APPL`, which may be a typo for `AAPL`; add ticker validation/confirmation rather than silently treating it as ordinary missing coverage.
- Mixed authenticated reads show a local SQLite tail-latency risk; throttle or batch `last_seen_at` updates and retest against production PostgreSQL/proxy topology.
- Public bundle chunk sizes merit later code splitting, especially `research-preview`.

## Release Decision

**NO-GO. No deployment was attempted.** The implementation and local automated gates now pass, but publishing without the conversation/data-provider configuration would release a visibly incomplete investment product. Publishing directly from the dirty, behind local `main` would also make the deployed revision unauditable. Only after the three remaining P1 gates are closed may the normal public Pages and managed-backend runbook proceed.

## Next Entry Point

Continue in this order:

1. Configure the safe function-calling conversation model, FMP and search/analysis providers without recording secrets in Git.
2. Run golden conversational canaries and verify fresh quote/fundamental/valuation coverage, company-rating participation, position advice, earnings calendar and evidence citations.
3. Validate whether portfolio ticker `APPL` is intentional or should be corrected to `AAPL`; do not silently rewrite user holdings.
4. Put the product batch on a clean `codex/` review branch based on current upstream, then rerun both Agent suites and every repository gate on the exact candidate revision.
5. Perform exact-revision production deployment and authenticated/unauthenticated canaries only after all above checks pass.
