# Public Community Private-R2 Edge Delivery

- title: Public Community Private-R2 Edge Delivery
- status: `archived`
- created_at: `2026-07-19`
- updated_at: `2026-07-19`
- owner: `Codex`
- related_files:
  - `crates/hone-core/src/config/server.rs`
  - `crates/hone-web-api/src/routes/public_community.rs`
  - `bins/hone-cli/src/cloud.rs`
  - `packages/app/src/lib/api.ts`
  - `packages/app/src/pages/public-community.tsx`
  - `workers/public-community-edge/`
  - `docs/runbooks/backend-deployment.md`
- related_prs: none
- related_docs:
  - `docs/handoffs/2026-07-19-public-community-edge-delivery.md`
  - `docs/handoffs/2026-07-12-public-community-readonly.md`
  - `docs/decisions.md#d-2026-07-19-09-deliver-authenticated-community-archives-from-private-r2-at-the-edge`
- supersedes: none
- superseded_by: none

## Goal

Move authenticated public-community feed and stored resource delivery off the slow backend hot path by serving a published snapshot and private R2 objects through a narrowly routed Cloudflare Worker. Preserve the existing API, authentication, pagination, read state, resource preview, and download paths as automatic fallbacks. Do not restart the backend or activate production traffic in this implementation task.

## Plan

- [x] Read the existing community/API/OSS/Pages paths and anonymously probe the current production route.
- [x] Add fail-closed backend delivery configuration, short-lived edge-session grants, and the small user-specific community state endpoint.
- [x] Add a dry-run-first R2 snapshot publisher whose final `latest.json` write is the only publication pointer update.
- [x] Add the private-R2 Worker with exact path routing, token verification, authenticated edge caching, safe resource streaming, and legacy-origin fallback.
- [x] Add compile-time-disabled frontend discovery with per-request fallback to every legacy API/resource path.
- [x] Add focused tests and run proportionate Rust, Web, Worker, build, and regression verification.
- [x] Publish the initial private R2 snapshot and prove the result is idempotent without deploying a route or switching traffic.
- [x] Synchronize repository maps, invariants, decisions, the deployment runbook, handoff, and archive index.

## Verification

- Production archive audit: `662` contents and `833` resources. The latest authorized delta is exactly `13` contents with `9` files and `6` images; it was already present and was not inserted a second time.
- Snapshot dry-run: `662` contents, `34` pages, `833` resources, `719` edge resources, `114` legacy resources, `754` planned objects, and zero conflicts.
- Snapshot apply: all `719` edge resources passed full byte-size, SHA-256, and content-type validation before publication; `754/754` objects were written and read back; `latest.json` was updated last.
- Final snapshot dry-run: `existing_objects=754`, `would_write=0`, `no_op=true`, and `conflicts=[]`.
- Rust: changed-file formatting, focused publisher tests (`8/8` after the transient-R2 retry regression), workspace check, full workspace test excluding the two Apple clients, and CI-safe regressions passed.
- Web: `280/280` tests, typecheck, and public builds with edge discovery both disabled and enabled passed.
- Worker: `45/45` tests, typecheck, frozen install, and Wrangler dry-run passed.
- Read-only production probes confirmed the existing legacy authentication boundary and fixed origin; the new backend endpoints and Worker route remain inactive until operator deployment/restart.

## Risks

- Worker deployment, R2 binding selection, shared HMAC secret installation, backend `off -> shadow -> prefer` restarts, Pages activation, authenticated canaries, monitoring, and rollback drills remain operator work in `docs/runbooks/backend-deployment.md`.
- `114` metadata-only/source-protected resources deliberately remain on the legacy compatibility path.
- The publisher performs full eligible-resource verification on every apply, so a weekly publication currently reads several GiB. Do not weaken this before a separately reviewed incremental integrity design exists.
- `community-contents` is bootstrap-only. A dedicated idempotent `community-append` workflow is required before the next weekly source import; shifting source positions must not be fed to the bootstrap command.
