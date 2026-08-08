# Production GCE Rollout Handoff — 2026-08-08

- title: Reviewed production GCE rollout through `d379cccc`
- status: done
- created_at: 2026-08-08
- updated_at: 2026-08-08
- owner: Codex
- related_files: `crates/hone-channels/src/investment_response_guard.rs`, `packages/app/src/lib/api.ts`, `packages/app/src/lib/public-session-cache.ts`, `tests/regression/ci/test_navigation_responsiveness_contract.sh`
- related_docs: `docs/archive/plans/production-gce-rollout-2026-08-08.md`, `docs/runbooks/backend-deployment.md`, `docs/archive/index.md`
- related_prs: direct `main` commit `d379cccc6e909129d02e726c04919e7c7ec250e1`; no PR, release, or tag

## Summary

Reviewed the production delta from `beaf05c3` through the newly pulled `7efdc01d`, fixed two deployment blockers, passed the complete repository and remote CI contracts, and deployed exact revision `d379cccc6e909129d02e726c04919e7c7ec250e1` to production GCE from GHCR digest `sha256:adc956533d59e46cd50a44ff6380019aa5ece64de5013a2a6eb95646bbd1ca05`.

## What Changed

- Increased the preturn outer deadline from 12 to 15 seconds and locked the invariant that it must exceed the sequential 6-second identity and 8-second evidence/fundamentals budgets. The old combination could discard already completed evidence while the last branch timed out.
- Public chat bootstrap now fills the route cache. Logout clears user and community cache synchronously before its network request, unauthorized bootstrap clears it, and an account-ID change drops the prior account's cached community page.
- Staged the exact immutable runtime using a temporary registry configuration, atomically switched `/opt/hone/current`, and restarted only `hone-web.service`. Feishu and Discord remained inactive as before.

## Verification

- Local: changed-file rustfmt/diff, focused review regressions, full workspace check/test, Web typecheck/full tests/public build, Edge Worker typecheck/45 tests, and `tests/regression/run_ci.sh` all passed.
- Remote: GitHub CI `31241183462`, Runtime Image `31241183454`, Secret Scan, Release Cache Warm, and exact-commit Cloudflare Pages checks passed.
- GCE: the staging script and candidate bundle hashes/revision matched; runtime-env validation passed; two idle reads before cutover and repeated reads after cutover were zero; disk retained about 3.8 GiB free.
- Runtime: `/api/meta` reports exact `d379cccc`, `source=ghcr_linux_oci`, cloud mode, healthy PostgreSQL/R2, cloud storage authority and zero local durable dependencies. The running executable resolves into the new immutable release; `hone-web.service` is active/running with `NRestarts=0`, and the warning-level journal count since rollout was zero.
- Public: `/`, `/chat`, `/roadmap` return `200`; unauthenticated auth/events return application `401`; HSTS, CSP `frame-ancestors 'none'`, `X-Frame-Options: DENY`, `nosniff`, and strict referrer policy are present. Cloudflare Pages serves entry `index-CH0a9V6L.js`, lazy chat `chat-hWm9xgdp.js`, and shared recovery `public-chat-DRmjVRIM.js`; together they contain `active_run`, `started_at_ms`, `run_progress`, and `interrupted_run`, without the legacy quota-derived recovery branch.
- Credential hygiene: registry authentication existed only in a temporary `0700` Docker configuration and was removed immediately after staging; no credential remains on GCE.

## Risks / Follow-ups

- Immediate rollback is `/opt/hone/releases/beaf05c360a7397ce6335ce177fdb74380756662-ghcr-runtime`; the new and prior releases were both retained.
- Direct `origin.hone-claw.com/api/public/auth/me` still returns the pre-existing stale tunnel `307 / Tunnel not found`. The user-facing Worker route returns the correct application `401`; origin alias reconciliation remains separate work.
- The immutable runtime does not currently serve the configured loopback static Pages fallback, so `/` and `/chat` on port `8088` remain `404`; Cloudflare Pages is the verified public frontend authority.
- The event-engine transcript review structure and saved real-source baseline are present, but automatic legal full-transcript acquisition is not. Do not describe that source lane as automatic until a reviewed source is wired into runtime ingestion.

## Next Entry Point

For rollback, confirm two zero active-chat reads, atomically restore `/opt/hone/current` to the retained `beaf05c3` release, restart `hone-web.service`, and repeat exact `/api/meta`, cloud authority and public API probes. For follow-up hardening, reconcile the stale origin alias and decide whether immutable GHCR bundles should carry a verified static public fallback.
