# Production GCE Rollout — 2026-08-08

- title: Production GCE rollout for post-`beaf05c3` changes
- status: archived
- created_at: 2026-08-08
- updated_at: 2026-08-08
- owner: Codex
- related_files:
  - `crates/hone-channels/src/agent_session/core.rs`
  - `crates/hone-channels/src/investment_response_guard.rs`
  - `crates/hone-web-api/src/routes/chat.rs`
  - `crates/hone-web-api/src/state.rs`
  - `packages/app/src/`
  - `tests/regression/ci/`
- related_docs:
  - `docs/runbooks/backend-deployment.md`
  - `docs/handoffs/2026-08-08-production-gce-rollout.md`
  - `docs/archive/index.md`

## Goal

Review the changes from production revision `beaf05c360a7397ce6335ce177fdb74380756662` through `7efdc01d9c8d8d247c9d00d163585743274b2b2e`, fix deployment blockers, validate the affected backend and public Web behavior, and deploy the exact reviewed successor revision to production GCE.

## Scope

- Reviewed backend preturn evidence timing, scheduled entity handling, heartbeat delivery, event-engine earnings continuity, active-run recovery, and public frontend navigation/session/theme changes.
- Fixed the preturn outer deadline so it cannot preempt the sequential bounded phases.
- Made public user/community caches populate from bootstrap and clear synchronously on logout, unauthorized bootstrap, and account replacement.
- Validated the complete repository contract and deployed exact revision `d379cccc6e909129d02e726c04919e7c7ec250e1` from its immutable GHCR digest.
- Verified GCE runtime provenance/cloud authority and the exact Cloudflare Pages commit plus active-run protocol markers.

## Validation

- Passed changed-file format/diff checks, focused Rust/Web tests, complete Rust workspace check/test, Web unit/typecheck/public build, Edge Worker typecheck/45 tests, and all CI-safe regressions.
- GitHub CI `31241183462`, Secret Scan, Runtime Image `31241183454`, and Cloudflare Pages exact-commit check passed.
- GCE staging verified the bundle twice; disk had more than the required 2 GiB before staging and about 3.8 GiB after cutover.
- Two pre-cutover idle reads and post-cutover readback were zero. `/api/meta` reports exact `d379cccc`, `source=ghcr_linux_oci`, healthy PostgreSQL/R2, cloud authority and zero local durable dependencies. `hone-web.service` is active with `NRestarts=0`; warning-level journal count was zero.
- Public `/`, `/chat`, and `/roadmap` return `200`; unauthenticated auth/events return application `401`; required HSTS/CSP/frame/content-type/referrer headers are present. Pages entry `index-CH0a9V6L.js`, chat chunk `chat-hWm9xgdp.js`, and shared recovery chunk `public-chat-DRmjVRIM.js` contain the current recovery markers without the legacy `in_flight + Date.now()` branch.

## Documentation Sync

- Removed this task from `docs/current-plan.md`, archived this plan, added the production handoff, and indexed the result in `docs/archive/index.md`.
- No `docs/repo-map.md`, `docs/invariants.md`, or decision update was needed: the review fixes restore existing behavior and session-isolation contracts without changing module boundaries or architecture.

## Risks / Open Questions

- Immediate rollback remains `/opt/hone/releases/beaf05c360a7397ce6335ce177fdb74380756662-ghcr-runtime`.
- `origin.hone-claw.com` still returns the previously recorded stale tunnel response; the public Worker API route is healthy and this rollout did not change origin routing.
- The GHCR bundle remains executable-oriented and does not provide the loopback static public fallback configured in systemd; Cloudflare Pages is the verified public frontend authority.
- Automatic legal full-transcript acquisition is still a future event-engine dependency; this rollout does not claim the saved transcript baseline became an automatic production source.
