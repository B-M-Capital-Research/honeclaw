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
- related_prs: none; local change set is not pushed

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
