# Public Community Edge Production Rollout

- title: Public Community Edge Production Rollout
- status: `in_progress`
- created_at: `2026-07-19`
- updated_at: `2026-09-05`
- owner: `Codex / operator`
- related_files:
  - `workers/public-community-edge/`
  - `crates/hone-core/src/config/server.rs`
  - `crates/hone-web-api/src/routes/public_community.rs`
  - `packages/app/src/lib/api.ts`
  - `packages/app/src/pages/public-community.tsx`
- related_docs:
  - `docs/runbooks/backend-deployment.md#public-community-private-r2-edge-rollout`
  - `docs/handoffs/2026-07-19-public-community-edge-delivery.md`
  - `docs/decisions.md#d-2026-07-19-09-deliver-authenticated-community-archives-from-private-r2-at-the-edge`
- related_prs: none; commits `385e35b0`, `100f5608`

## Goal

Restore the production community timeline to the latest authorized source content, repair legitimately accessible images and files, and reduce measured feed/resource latency through the managed backend and private Cloudflare delivery path. Complete the pending edge rollout where verified safe, preserving authentication and rollback.

## 2026-09-05 Recovery Todo

- [x] Pull the clean main checkout with `git pull --ff-only` (baseline `fdb533d4`).
- [x] Compare the authorized Knowledge Planet group `51115212285814` latest posts with canonical PostgreSQL and the production feed; identify the ongoing synchronization owner and missing interval.
- [x] Repair the append/sync workflow in `bins/hone-cli/src/cloud.rs` / `crates/hone-core/src/cloud_runtime.rs` as needed; recover source content without reusing positional bootstrap reconciliation for incremental updates.
- [ ] Audit resource state and source availability; repair accessible originals through the verified immutable-object backfill, then test actual image/PDF/download delivery.
- [ ] Measure GCP runtime, Cloudflare routes/cache, backend feed/resource requests and client request ordering; fix the observed latency causes in `public_community.rs`, `workers/public-community-edge/` and the public frontend.
- [x] Reduce recurring publish latency with a reviewed `scripts/community_production.py --remote` mode for inspect/publish on the same managed runtime; preserve authority checks and full object SHA verification, validate rejection boundaries, and synchronize the daily runbook/saved automation. Real production final dry-run completed in 32.282 seconds with no-op and no conflicts.
- [x] Replace blocked native PDF iframe preview with lazy application-side PDF rendering; retain authenticated/cached Blob downloads and cancellation, verify real rendered page pixels/navigation in Chromium, and synchronize community preview docs. Four production-build E2E scenarios and 567 frontend tests passed; exact Web rollout is tracked below.
- [ ] Run meaningful targeted regressions plus required workspace/Web/Worker gates for the final changes; deploy an exact revision through managed runtime/Pages workflows and verify source freshness, resource access, authentication, latency and rollback on production.
- [ ] Update `docs/runbooks/backend-deployment.md`, `docs/repo-map.md` and applicable long-term decisions/invariants; write `docs/handoffs/2026-09-05-community-freshness-assets-latency.md` with before/after evidence and remaining limitations.
- [ ] On completion remove the active index entry, archive this plan into `docs/archive/plans/`, and add the handoff/plan/commit to `docs/archive/index.md`. If externally blocked, retain explicit status and the exact next action.

The July sections below are retained as history and do not describe the current deployed state; the September 5 handoff is the current operational entry point.

This task meets dynamic-plan admission because it spans source ingestion, archive storage, backend delivery, frontend and production operations, with parallel implementation and verification.

## Historical July 19 Scope

- Completed: published the initial private R2 projection and deployed the brand-new `hone-public-community-edge` Worker as version `e01c1603-7c34-476a-b63b-33ac74244108` on `hone-claw.com/_community/v1/*`, bound to private bucket `honeclaw`, with no secret and no enabling variable.
- Completed: rebased the implementation onto the latest Web mainline in an isolated worktree and pushed commits `385e35b0` and `100f5608` to `main`; later `cb796cce` is a docs-only follow-up.
- Completed: the automatic Pages deployments for `100f5608` and `cb796cce` succeeded. The production bundle contains no `_community`, `edge-session`, or `community_edge` marker, proving discovery remains compiled out; normal users still use the legacy APIs.
- Completed: built all runtime binaries from exact source commit `100f5608` into the new immutable `target/deploy-100f5608` directory, assembled the discovery-off public bundle, exact skills/soul, config symlink, and a hash-verified deployment manifest. The running `d58ef12b` deployment was not modified or restarted.
- Pending: let the external service perform the controlled restart into `target/deploy-100f5608`; require cloud-authority health, zero active chats, legacy compatibility, and `POST /api/public/community/edge-session` returning `200` with `enabled=false` and `mode="off"`.
- Pending: install one shared signing secret in the approved secret manager, backend environment, and Worker while the Worker remains disabled; then move through backend `shadow`, authenticated Worker canaries, backend `prefer`, and a separately enabled Pages discovery build.

## Historical July 19 Validation

- Worker gate: frozen install unchanged; TypeScript passed; Worker tests passed `45/45`; Wrangler dry-run reported `22.93 KiB` and only the reviewed bindings.
- Integration gate: Web typecheck, tests, discovery-off public build, Rust formatting, all workspace runtime-binary build, and gitleaks over the two pushed commits passed.
- Cloudflare gate: Worker deployment list reports version `e01c1603-7c34-476a-b63b-33ac74244108`; secret list is empty; the exact edge route returns Worker-owned `503 {"error":"community_edge_unavailable"}`; production `/community` is `200` while edge discovery markers are absent from its entry and community chunks.
- Current backend boundary: `/api/meta` remains cloud-authoritative with healthy PG/OSS and zero local durable dependencies; active chats are zero; the old process still returns `404` for the new edge-session route, proving no restart occurred.
- Prebuilt runtime gate: every hash in `target/deploy-100f5608/runtime-root/DEPLOYMENT_MANIFEST` matches the staged config, soul, and five production binaries.

## Documentation Sync

- Update the rollout status in `docs/runbooks/backend-deployment.md` and append phase evidence to the existing handoff after every production gate.
- Keep `docs/current-plan.md` linked to this plan while any backend, Worker, Pages, secret, canary, or rollback gate remains pending.
- When rollout acceptance is complete, move this plan to `docs/archive/plans/`, update `docs/archive/index.md`, and remove it from the active index.

## Historical July 19 Risks / Open Questions

- `keep_vars=true` can preserve a remote activation value on later deployments. Before every deploy, inspect the real remote `EDGE_DISABLED` value; disabling an existing Worker must use an explicit deployed `EDGE_DISABLED=true`.
- The backend config intentionally has no explicit `community_delivery` block and therefore uses the safe `mode=off` default. Activation is prohibited until the external restart and the exact `200 enabled=false` probe pass.
- The shared repository may contain unrelated uncommitted work. Restart only the exact immutable deployment; never build or launch the shared working tree for this rollout.
- No signing secret is installed. Never place it in chat, source control, Pages variables, R2 objects, or command output.
- Pages discovery remains off and the Worker must stay fail-closed until the authenticated canary sequence is complete.
