# Public API Connection Performance

- title: Reduce shared API CPU cost with isolated PostgreSQL connection reuse
- status: in_progress (post-rollout production hang; restore service and diagnose)
- created_at: 2026-09-20
- updated_at: 2026-09-20
- owner: Codex
- related_files: Cargo.toml; crates/hone-core/src/cloud_runtime.rs; crates/hone-core/src/cloud_runtime/; tests/regression/ci/test_runtime_image_contract.sh
- related_docs: docs/handoffs/2026-09-20-public-api-latency-diagnosis.md; docs/invariants.md; docs/repo-map.md; docs/decisions.md; docs/runbooks/source-web-startup.md

## Goal

Remove repeated PostgreSQL authentication from ordinary API queries and enable optimized production builds. User/actor isolation, correct response routing, exclusive transactions and clean failure handling are acceptance requirements, not performance tradeoffs.

## Scope

- [x] Audit connection call sites and preserve existing unrelated working-tree changes.
- [x] Add bounded, exclusively borrowed connections for ordinary autocommit operations; retain dedicated connections for transactions and advisory locks and preserve connection-local test schemas.
- [x] Enable optimization in the existing source-runtime build profile while retaining line-level debugging and non-incremental builds.
- [x] Do not change auth, actor predicates, session semantics or production deployment in this implementation step.

## Validation

- [x] Real PostgreSQL tests: concurrent acquisition is bounded, responses/actor data cannot cross, transactions remain isolated and roll back on cancellation, failed connections recover, and test namespaces stay separate. Eight optimized-profile regressions passed.
- [x] Focused core/memory/web-api regressions and applicable CI checks; report baseline failures separately. 93 explicit PG tests passed; workspace check passed; workspace tests had 2842 passes and 3 documented unrelated failures; 21/22 CI scripts passed (finance static failure documented); 552 web and 45 Worker tests passed.
- [x] Reproducible local before/after connection/query benchmark without a flaky timing assertion in CI. SCRAM-authenticated local 8-query batches: dev/fresh 314.097 ms, optimized/fresh 39.217 ms, optimized/pooled 2.545 ms medians; not an HTTP speedup claim.

## Documentation Sync

- [x] Update docs/invariants.md, docs/repo-map.md, docs/decisions.md and docs/runbooks/source-web-startup.md for connection ownership and optimized source-runtime semantics.
- [x] Append implementation and verification to the existing same-day diagnostic handoff, update docs/archive/index.md, archive this plan and remove its active index entry when local work is complete.

## Risks / Open Questions

- Production runs a newer revision than this dirty checkout; no unrelated changes may be silently deployed.
- Ordinary reusable clients must not share transaction/session state with schema migrations or advisory locks.
- Docker is unavailable locally; safety tests used the existing native development PostgreSQL. A separate temporary SCRAM-enabled PostgreSQL served the benchmark and was stopped after use.
- Production HTTP gains are recorded in the handoff. High-load pool queueing and a live multi-account write canary remain outside this short acceptance run; isolation/transaction behavior was verified with isolated PostgreSQL regressions.

## Production Rollout — 2026-09-20

User authorized: “发上线测一下看看”. No formal version/tag requested.

- [x] Re-read live revision, service topology, active chats and rollback target; capture matched pre-deploy API timings.
- [x] Create an isolated production-based checkout containing only this optimization, regression tests and matching documentation. Review diff and run candidate checks.
- [x] Publish the reviewed candidate branch and build the exact immutable Linux GHCR artifact through Runtime Image; no macOS binary or production-host compilation.
- [x] Add a bounded Runtime Image export mode using job-scoped package read permission and a checksummed Actions artifact, because the operator credential cannot pull the private registry. Do not expand operator token scopes or transfer a broad credential to production.
- [x] Verify bundle, environment, free disk and two idle-chat reads; atomically switch and restart with rollback on failed acceptance.
- [x] Validate exact live revision, PostgreSQL/object-store authority, expected channel workers, account/session isolation and before/after API latency.
- [x] Append rollout evidence to the same-day handoff, update decision/archive index, archive this plan and remove the active index entry.

Completion: deployed exact `3e26eb4fe574ff9aae94ddb2b21732c9f8ede416`; authenticated origin samples and 140 auth rejection checks passed, 13 live message IDs/order/content unchanged, previous artifact retained. See the handoff for baseline test failures, the restart-window 502s and follow-ups.

## Withdrawal Follow-up

- [x] Restore the previous runtime and verify the public API and existing browser history.
- [x] Withdraw the earlier deployment acceptance in the handoff, decision and archive index; keep detailed operator evidence outside version control.
- [ ] Reproduce the HTTP responsiveness regression in isolation and identify the cause before modifying runtime code.
- [ ] Add a cause-specific regression and validate sustained responsiveness plus account/session isolation before another rollout.
- [ ] Archive only after the causal investigation and required fix/verification are complete; the performance rollout remains withdrawn.

Affected files: cloud_runtime.rs, cloud_runtime/query_pool.rs and tests if implicated; the same-day handoff, decisions, current-plan index and archive index. No speculative runtime change was made during recovery.
