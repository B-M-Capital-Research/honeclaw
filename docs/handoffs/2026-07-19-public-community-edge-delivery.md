# Public Community Private-R2 Edge Delivery

- title: Public Community Private-R2 Edge Delivery
- status: `done`
- created_at: `2026-07-19`
- updated_at: `2026-07-19`
- owner: `Codex`
- related_files:
  - `bins/hone-cli/src/cloud.rs`
  - `crates/hone-core/src/config/server.rs`
  - `crates/hone-web-api/src/routes/public_community.rs`
  - `packages/app/src/lib/api.ts`
  - `packages/app/src/pages/public-community.tsx`
  - `workers/public-community-edge/`
- related_docs:
  - `docs/archive/plans/public-community-edge-delivery.md`
  - `docs/runbooks/backend-deployment.md#public-community-private-r2-edge-rollout`
  - `docs/decisions.md#d-2026-07-19-09-deliver-authenticated-community-archives-from-private-r2-at-the-edge`
- related_prs: none; commits `385e35b0`, `100f5608`

## Summary

The compatible private-R2 edge path is implemented and locally verified. The initial production-derived snapshot is privately published in the existing R2 bucket, but no Worker/Pages deployment, secret or variable mutation, traffic cutover, backend restart, or Git push occurred. Existing users therefore remain on the unchanged legacy community APIs.

The authorized source delta was also reconciled before this delivery work: production contains `662` community contents and `833` resources, exactly `13` more contents than the prior `649`-content archive, with `9` files and `6` images. Browser/source, PostgreSQL, object key, size, SHA-256, content type, and timeline checks matched; no duplicate insertion was performed.

## What Changed

- Backend delivery configuration defaults to `off`, supports staged `shadow`/`prefer`, issues a short-lived scoped HttpOnly HMAC grant, exposes a small personal-state endpoint, and clears both session cookies at logout. The legacy feed/resource APIs are unchanged.
- The frontend discovers edge delivery only in an explicitly enabled production build and only when the backend returns exact `prefer` configuration. Feed, image, PDF, attachment, and download failures immediately fall back to the legacy path.
- The exact-route Worker authenticates before cache/R2, uses the private R2 binding, validates the mutable active-version index, bounds response sizes, and forwards only the legacy web-session cookie to a fixed origin on eligible GET fallback.
- `hone-cli cloud community-publish` is dry-run-first, reads a repeatable read-only PostgreSQL snapshot under an advisory lock, verifies every eligible immutable object before any publication write, writes `latest.json` last, and supports idempotent bounded retry for transient R2 HEAD/GET/PUT failures.
- The production private snapshot contains `34` feed pages, `719` versioned descriptors, one active index, and `754` total publication objects. The remaining `114` resources deliberately stay legacy.

## Verification

- First dry-run after adding bounded retry: `ok=true`, `planned_objects=754`, `conflicts=[]`, `written=0`.
- Apply: `resource_verification=full_bytes_sha256`, `written=754`, `latest_updated=true`, `conflicts=[]`.
- Final dry-run: `existing_objects=754`, `would_write=0`, `no_op=true`, `conflicts=[]`.
- Publisher regression tests: `8/8`; `cargo check -p hone-cli` and changed-file formatting passed.
- Full proof from the implementation task: workspace check/test excluding Apple clients, CI-safe regressions, Web `280/280` plus typecheck and both build-flag variants, Worker `45/45` plus typecheck/frozen install/Wrangler dry-run.
- Final security review found no merge blocker. Authentication precedes cache/R2, R2 stays private, the active index is checked before shared resource cache, missing/unknown disable values fail closed, and the origin fallback never receives the edge cookie.

## Risks / Follow-ups

- Follow `docs/runbooks/backend-deployment.md#public-community-private-r2-edge-rollout` in order. Worker config is already bound to the verified active R2 bucket `honeclaw`; confirm that production `HONE_OSS_BUCKET` has not changed before deploy.
- Keep `EDGE_DISABLED=true` explicitly deployed for an existing/restored Worker. A brand-new Worker may omit it only after confirming no remote value exists; `keep_vars=true` can preserve a stale remote `false`.
- Generate one 32..1024-byte secret in the approved secret manager and install the exact value at backend and Worker. Do not commit, log, or paste it into Pages/R2/chat.
- Backend mode changes require the operator-managed restart that the user explicitly reserved for another service. Pages discovery stays `0` until the authenticated Worker canary and backend `prefer` checks pass.
- Every future snapshot apply currently performs full eligible-resource verification. A dedicated `community-append` command remains necessary before the next weekly source import; `community-contents` must not be used as an incremental importer.

## Next Entry Point

Start at Step 1 of `docs/runbooks/backend-deployment.md#public-community-private-r2-edge-rollout`. Step 5 is already complete for the `662`-content snapshot and only needs repeating if the canonical archive changes before activation.

## 2026-07-19 Disabled Infrastructure And Pre-Restart Phase

- Wrangler OAuth was authorized for Cloudflare account `52dfc1420d779b403c08196b792ce926`; no repository credential or optional Cloudflare AI skill package was added.
- A remote preflight confirmed that `hone-public-community-edge` did not exist, so there was no preserved `EDGE_DISABLED=false` value under `keep_vars=true`. Frozen install made no changes; TypeScript passed; Worker tests passed `45/45`; Wrangler dry-run reported `22.93 KiB` uploaded (`5.86 KiB` gzip) and only the reviewed bindings.
- The disabled first deployment succeeded as Worker version `e01c1603-7c34-476a-b63b-33ac74244108`, with exact route `hone-claw.com/_community/v1/*`, `COMMUNITY_BUCKET -> honeclaw`, `workers_dev=false`, and `preview_urls=false`. `EDGE_DISABLED` remains absent and the Worker secret list is empty.
- The public edge probe returned `503` with `{"error":"community_edge_unavailable"}`, `Cache-Control: private, no-store`, and `Vary: Cookie`, proving the route is Worker-owned and fail-closed before R2 or origin access. Anonymous legacy feed and resource probes remained backend JSON `401`.
- Upstream `main` advanced during delivery, so the implementation was replayed without conflict in an isolated worktree on top of `33f7d4b8`. Gitleaks, Rust formatting, Web typecheck/tests/public build, Worker checks, and the full runtime-binary build passed; commits `385e35b0` and `100f5608` were pushed by fast-forward. Later `cb796cce` is docs-only.
- Cloudflare Pages automatically deployed `100f5608` and the docs-only follow-up. The current production entry is `index-o0hmDXxE.js`; neither it nor `public-community-Cc9Av8gA.js` contains `_community`, `edge-session`, or `community_edge`, proving the compile-time discovery gate remains off.
- Exact source `100f5608` was assembled as a new immutable `target/deploy-100f5608` runtime with all five production binaries, discovery-off public assets, exact skills/soul, the existing config symlink, and a fully verified `DEPLOYMENT_MANIFEST`. Assembly observed `origin/main=cb796cce` and recorded that the follow-up is docs-only.
- The old backend was not restarted or overwritten. It remains cloud-authoritative with healthy PostgreSQL/R2, zero local durable dependencies, and zero active chats; local `POST /api/public/community/edge-session` still returns `404`. The external service must restart into the prepared directory, then require `200` with `enabled=false`, `mode="off"`, no token/user identifier, healthy `/api/meta`, and unchanged legacy behavior before any secret installation.
