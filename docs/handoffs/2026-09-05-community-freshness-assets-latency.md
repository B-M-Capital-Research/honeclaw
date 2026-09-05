# Community freshness, assets and latency recovery

- title: Community freshness, assets and latency recovery
- status: `in_progress`
- created_at: `2026-09-05`
- updated_at: `2026-09-05`
- owner: `Codex`
- related_files:
  - `bins/hone-cli/src/cloud.rs`
  - `crates/hone-core/src/cloud_runtime.rs`
  - `crates/hone-web-api/src/routes/public_community.rs`
  - `scripts/community_production.py`
  - `packages/app/src/pages/public-community.tsx`
  - `packages/app/src/lib/public-community-timeline.ts`
- related_docs:
  - `../current-plans/public-community-edge-production-rollout.md`
  - `../runbooks/community-insights-daily-sync.md`
  - `../runbooks/backend-deployment.md`
  - `2026-08-22-community-insights-refresh.md`
- related_prs: pending exact recovery revision and deployment

## Summary

The September 5 production audit found two different community archives: the managed API had 718 topics ending July 31, while a local automation had updated a different PostgreSQL database and shared private R2 projection. Restored uncommitted August capture/append tooling, reviewed its identity and transaction boundaries, and bound future community commands to the managed production environment through a fingerprint-checked IAP wrapper.

A contiguous source recovery manifest added 151 topics and 194 resource metadata rows to the live database, reaching 869 topics and source head `2026-09-04 00:51` (`content_id=976`). Recovered historical source captures are evidence, not proof that the August task had updated production. Its handoff now carries an explicit correction.

## What Changed

- `community-inspect --anchor-only` emits exact machine-generated anchors. Append validates the original anchor on every replay, verifies existing stable identities, preserves same-minute source order and accepts legitimate file identity completion by backfill.
- Production wrapper keeps credentials in memory, pins the database identity and excludes the repository dotenv. The ignored operator configuration stores only host/identity metadata.
- Publisher metadata preflight uses 16 concurrent reads, preserving ordered writes, immutable-key conflict refusal, apply-time byte/SHA checks and latest-pointer-last semantics.
- Legacy HEAD and matching conditional GET use object metadata/size verification without downloading entire files. Full GET still verifies SHA-256.
- Frontend overlaps grant/state requests, rejects stale edge heads by equality with canonical IDs, refreshes on visible intervals/focus/online, merges pagination in source order, catches body-stream failures, cancels closed previews and reuses PDF bytes for download.
- Preserve the deployed Worker code and managed-origin fallback; repository July rollout configuration is behind the live deployment. Do not deploy it over production.

## Verification

- Import: first production dry-run 151 inserts; apply inserted 151 in one transaction, source newest September 4. Replay and final publication evidence will be appended after completion.
- Stored-object audit: all 769 existing stored objects exist with matching size; sampled file/image GETs pass full SHA. R2 HEAD p50 140 ms / p95 222 ms from the managed host. These figures are origin object probes, not browser page-load timings.
- Append: real PostgreSQL transaction regressions 5/5; CLI community tests 22/22. Legacy resource route tests 15/15, including HEAD/304/object missing/size/full-byte integrity boundaries.
- Frontend: 563 unit tests, typecheck and two Chromium community E2E scenarios passed. Worker typecheck and 45/45 tests passed. Public production-mode build passed.
- Workspace check passed. Full workspace tests with live isolated PostgreSQL: 2824 passed, 113 ignored, three unrelated failures in existing agent routing and the soul prompt contract. No affected community regression failed.
- CI-safe suite: 21/22 scripts passed. `finance_automation_contracts` has nine pre-existing checks failing identically when its script and 45 inputs are reconstructed from HEAD. Initial PG availability failures were rerun successfully after restoring the database.
- Ignored operational evidence: `data/community-imports/2026-09-05/`, including source/append manifests, production object audit, CLI reports and workspace log. Do not publish source bodies or credentials with the handoff.

## Risks / Follow-ups

- Normal source PDF download is blocked in Chrome at `files.zsxq.com` with `ERR_BLOCKED_BY_CLIENT`. The user has been asked to resolve the browser block. Protected/inaccessible files remain metadata-only; do not claim they are repaired or substitute invented content.
- Historical recovered images are rendered variants; newly captured originals are recorded separately with exact size and SHA before backfill. Production resource IDs must come from the production append report, never the old local database.
- Code deployment, managed environment repair, publication convergence, real browser acceptance and automation update are still pending in this phase record.
- Existing unrelated data-center/frontend edits in the shared checkout belong to another task and must remain outside the community recovery commit.

## Next Entry Point

Finish append replay, image backfill and production-only R2 publish with conflict-free dry-run/apply/no-op. Load the existing shared edge secret into the managed process environment and deploy the exact GHCR/Pages revision. Verify production latest ID, resource bytes, authentication and measured delivery; append the results here and update the active plan.
