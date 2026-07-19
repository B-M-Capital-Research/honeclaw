# Public Community Edge Production Rollout

- title: Public Community Edge Production Rollout
- status: `in_progress`
- created_at: `2026-07-19`
- updated_at: `2026-07-19`
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

Complete the staged production rollout of authenticated community feed and resource delivery through the exact-route Cloudflare Worker and private R2, without a visible regression for existing users and without bypassing the externally managed backend restart.

## Scope

- Completed: published the initial private R2 projection and deployed the brand-new `hone-public-community-edge` Worker as version `e01c1603-7c34-476a-b63b-33ac74244108` on `hone-claw.com/_community/v1/*`, bound to private bucket `honeclaw`, with no secret and no enabling variable.
- Completed: rebased the implementation onto the latest Web mainline in an isolated worktree and pushed commits `385e35b0` and `100f5608` to `main`; later `cb796cce` is a docs-only follow-up.
- Completed: the automatic Pages deployments for `100f5608` and `cb796cce` succeeded. The production bundle contains no `_community`, `edge-session`, or `community_edge` marker, proving discovery remains compiled out; normal users still use the legacy APIs.
- Completed: built all runtime binaries from exact source commit `100f5608` into the new immutable `target/deploy-100f5608` directory, assembled the discovery-off public bundle, exact skills/soul, config symlink, and a hash-verified deployment manifest. The running `d58ef12b` deployment was not modified or restarted.
- Pending: let the external service perform the controlled restart into `target/deploy-100f5608`; require cloud-authority health, zero active chats, legacy compatibility, and `POST /api/public/community/edge-session` returning `200` with `enabled=false` and `mode="off"`.
- Pending: install one shared signing secret in the approved secret manager, backend environment, and Worker while the Worker remains disabled; then move through backend `shadow`, authenticated Worker canaries, backend `prefer`, and a separately enabled Pages discovery build.

## Validation

- Worker gate: frozen install unchanged; TypeScript passed; Worker tests passed `45/45`; Wrangler dry-run reported `22.93 KiB` and only the reviewed bindings.
- Integration gate: Web typecheck, tests, discovery-off public build, Rust formatting, all workspace runtime-binary build, and gitleaks over the two pushed commits passed.
- Cloudflare gate: Worker deployment list reports version `e01c1603-7c34-476a-b63b-33ac74244108`; secret list is empty; the exact edge route returns Worker-owned `503 {"error":"community_edge_unavailable"}`; production `/community` is `200` while edge discovery markers are absent from its entry and community chunks.
- Current backend boundary: `/api/meta` remains cloud-authoritative with healthy PG/OSS and zero local durable dependencies; active chats are zero; the old process still returns `404` for the new edge-session route, proving no restart occurred.
- Prebuilt runtime gate: every hash in `target/deploy-100f5608/runtime-root/DEPLOYMENT_MANIFEST` matches the staged config, soul, and five production binaries.

## Documentation Sync

- Update the rollout status in `docs/runbooks/backend-deployment.md` and append phase evidence to the existing handoff after every production gate.
- Keep `docs/current-plan.md` linked to this plan while any backend, Worker, Pages, secret, canary, or rollback gate remains pending.
- When rollout acceptance is complete, move this plan to `docs/archive/plans/`, update `docs/archive/index.md`, and remove it from the active index.

## Risks / Open Questions

- `keep_vars=true` can preserve a remote activation value on later deployments. Before every deploy, inspect the real remote `EDGE_DISABLED` value; disabling an existing Worker must use an explicit deployed `EDGE_DISABLED=true`.
- The backend config intentionally has no explicit `community_delivery` block and therefore uses the safe `mode=off` default. Activation is prohibited until the external restart and the exact `200 enabled=false` probe pass.
- The shared repository may contain unrelated uncommitted work. Restart only the exact immutable deployment; never build or launch the shared working tree for this rollout.
- No signing secret is installed. Never place it in chat, source control, Pages variables, R2 objects, or command output.
- Pages discovery remains off and the Worker must stay fail-closed until the authenticated canary sequence is complete.
