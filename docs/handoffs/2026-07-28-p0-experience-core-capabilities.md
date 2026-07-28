# P0 Experience Core Capabilities

- title: P0 Experience Core Capabilities
- status: done
- created_at: 2026-07-27
- updated_at: 2026-07-28
- owner: Codex
- related_files:
  - `crates/hone-tools/src/data_fetch.rs`
  - `agents/function_calling/src/lib.rs`
  - `crates/hone-core/src/tool_effect.rs`
  - `crates/hone-channels/src/investment_response_guard.rs`
  - `packages/ui/src/lib/markdown.ts`
  - `packages/app/src/components/markdown-rendering.test.tsx`
  - `tests/regression/ci/test_finance_automation_contracts.sh`
- related_docs:
  - `docs/archive/plans/p0-experience-core-capabilities.md`
  - `docs/invariants.md`
  - `docs/decisions.md`
  - `docs/bugs/feishu_scheduler_watchlist_stockanalysis_abnormal_prices.md`
  - `docs/runbooks/backend-deployment.md`
- related_prs:
  - fix commit `c2edceb7269476c39a3eb23efd25d14d4675aa93` on `main`
  - supervisor readiness fix `e07eadb3a7af01bc71e1240ddd351c8805313eff` on `main`
  - loopback proxy-bypass fix `f56631072a38f32f8f02efa49c5a268156612219` on `main`

## Summary

The nineteen experience records were governed as five reusable capability domains rather than nineteen isolated patches. Three records were solved systemically in this task, four prior systemic fixes were reverified, six are covered by current product contracts, one remains an explicit provider-wide residual risk, four were deferred because no safe general solution exists, and one non-product asset request requires no action.

The private replacement workbook remains outside the repository. It preserves all nineteen source rows and twenty screenshots, adds per-row decisions/evidence and an overall status summary, and has SHA-256 `6b08ba7ff500718373d877cd7ed6e68d15ba65cccc995cd14efe7f9f3f792b0b`.

## What Changed

- Added symbol-scoped `earnings_outlook` evidence with independent quote-field usability, partial component coverage, dimensional validation, and independent-corroboration requirements for extreme target/current ratios.
- Routed interactive and typed scheduled/heartbeat earnings research through the same symbol-scoped contract instead of filtering a global calendar.
- Added central execute-once/read-once reconciliation for ambiguous portfolio, reminder, and notification-preference mutations. Timeouts and operations without a safe actor-scoped read remain fail-closed.
- Made accidental paired-tilde spans render as inert text in the shared Web Markdown layer without changing the established prompt answer format.
- Deliberately did not add company aliases, phrase exceptions, output regexes, screenshot CSS coordinates, or product-navigation redesigns whose only justification was one workbook row.

## Verification

- Full repository contract passed:
  - changed-file formatting and `git diff --check`
  - workspace `cargo check`
  - full workspace Rust tests excluding Apple clients
  - Web `303/303` plus typecheck
  - Public Community Edge `45/45` plus typecheck
  - complete CI-safe regression suite, including finance contracts `44/44`
- Workbook verification passed:
  - all `19` rows and `20` embedded screenshots reconciled
  - LibreOffice open/recalculate/export cycle
  - formula-error scan
  - visual review of all `18` rendered PDF pages
- Immutable deployment:
  - P0 capability fix source `c2edceb7269476c39a3eb23efd25d14d4675aa93`; final runtime source `f56631072a38f32f8f02efa49c5a268156612219`
  - path `target/deploy-f5663107`
  - `502` payloads: five binaries, twenty-seven skill files, `soul.md`, and 469 public Web files
  - manifest SHA-256 `b908de852668a47ea350e8f00dfb8ef09c47e7dcfa494a68a24c4994d32428bd`
  - every recorded payload hash verified
- Production:
  - two independent pre-restart active-chat checks returned zero
  - Web and Feishu supervisors stopped through SIGINT and released ports before restart
  - exact-package process paths and repository-root supervisor working directories verified
  - the first concurrent-head restart revealed that cloud-backed `/api/meta` plus proxy-eligible loopback requests could make the supervisor kill a healthy child after roughly seventy seconds; `e07eadb3`/`f5663107` replaced that readiness path and bypass proxies for loopback supervisor clients
  - the exact final runtime remained healthy across repeated probes beyond the previous self-exit window
  - ports `8077/8088`, version `0.15.3`, cloud mode, PostgreSQL, R2, authoritative storage, zero local durable dependencies, and repeated zero active chats passed
  - local/origin/public unauthenticated auth boundaries returned JSON `401`
  - `/`, `/chat`, `/roadmap`, and `/activate/whop` returned `200`; HSTS, CSP frame denial, `X-Frame-Options`, nosniff, and strict-origin referrer policy were present
  - Feishu held at least one established TCP connection across repeated probes

## Risks / Follow-ups

- A provider-wide split-adjustment or unit error that is internally self-consistent across every upstream endpoint cannot be disproved by dimensional checks alone. The abnormal-price P0 remains `In Progress`; do not label that workbook row fully solved.
- Four suggestions remain deferred because their only current implementation would be a brittle one-off rule or broad product redesign. Revisit only when a typed cross-market mapping, typed entity-span contract, or measured information-architecture requirement exists.
- No authenticated business canary or user-visible Feishu message was sent because no designated test actor was provided. Automated channel contracts and live transport health passed.
- The first post-restart R2 probe failed because the host's selected Clash global node rejected TLS. Both the old and new binaries reproduced it; Clash direct connectivity passed, and changing the reversible `GLOBAL` selector to `DIRECT` restored R2 immediately. Re-selecting a broken global node can make object storage unhealthy again.
- The final package includes the Whop activation code, but production does not configure `HONE_WHOP_WEBHOOK_SECRET` or a transactional email sender. Local, origin, and public webhook probes therefore return intentional JSON `503`; no Whop entitlement can be activated until the runbook configuration and live acceptance are completed.

## Next Entry Point

For finance evidence anomalies, start with `docs/bugs/feishu_scheduler_watchlist_stockanalysis_abnormal_prices.md` and retain the raw provider payload plus field-level usability metadata. For new experience feedback, classify it against the five capability domains and the no-one-off-patch boundary in `docs/decisions.md#d-2026-07-27-01-make-evidence-admission-and-mutation-completion-capability-level` before changing code.

## 2026-07-28 Follow-up: Watchlist Price Anchor Fail-Closed

- status: done
- owner: Codex bug-2 automation
- related_files:
  - `crates/hone-channels/src/scheduler.rs`
  - `docs/bugs/feishu_scheduler_watchlist_stockanalysis_abnormal_prices.md`
  - `docs/bugs/README.md`
- verification:
  - `cargo test -p hone-channels watchlist_price_anchor_guard_detects_quantity_mismatch_against_hit_zone -- --nocapture`
  - `cargo test -p hone-channels watchlist_price_anchor_guard_allows_in_range_prices -- --nocapture`
  - `cargo check -p hone-channels --tests`
- risks:
  - This follow-up only hardens typed scheduler / heartbeat delivery.
  - Interactive direct finance answers can still reuse the same bad price family until the direct evidence-admission path gets an equivalent guard.

- Added a runtime fail-closed guard for watchlist tasks: if a delivered scheduler / heartbeat price is off by an obvious order of magnitude versus the task or restored local hit zone, scheduler now degrades to a failure message and heartbeat suppresses delivery.
- Left the P0 bug open on purpose. The change blocks the most dangerous scheduled push path, but it does not yet close the direct-answer path that still surfaced in the latest bug evidence.
