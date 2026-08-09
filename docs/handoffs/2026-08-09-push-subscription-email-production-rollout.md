# Push Subscription / Email Production Rollout Handoff — 2026-08-09

- title: Production-ready push management, login-free unsubscribe, and Cloudflare email configuration
- status: done
- created_at: 2026-08-09
- updated_at: 2026-08-09
- owner: Codex
- related_files: `crates/hone-core/src/email.rs`, `crates/hone-core/src/unsubscribe_token.rs`, `crates/hone-web-api/src/routes/mod.rs`, `packages/app/src/lib/public-subscription-model.ts`, `packages/app/src/components/public-subscription-manager.tsx`
- related_docs: `docs/archive/plans/push-subscription-email-production-rollout.md`, `docs/runbooks/backend-deployment.md`, `docs/archive/index.md`
- related_prs: direct `main` implementation commit `9eff909aba898dfd12b268a75f71bc269f1e7c4d`; no PR, release, or tag

## Summary

Reviewed the subscription-management, signed unsubscribe, and Cloudflare Email Sending changes; fixed the production blockers; passed the complete local and GitHub CI contracts; and deployed exact revision `9eff909aba898dfd12b268a75f71bc269f1e7c4d` to production GCE from immutable GHCR digest `sha256:1335325fb6075c98a85ee585bbfc12a3e1073a2b90d3ad860e3fcafec13ba758`. The Cloudflare token and a newly generated `hone_unsubscribe_v1_…` signing secret are present only in the root-owned mode-`0600` production runtime environment.

## What Changed

- Moved login-free unsubscribe from the authenticated administrator router to `/api/public/unsubscribe/{token}`, matching the public Pages/Worker proxy and generated links.
- Added `PUT` to public CORS so in-place subscription schedule edits work in browsers.
- Corrected the public schedule model to scheduler-native Monday-first weekday numbering and distinct `workday`, `trading_day`, `holiday`, and `heartbeat` summaries.
- Hardened Cloudflare Email Sending responses: delivery requires an accepted/queued provider result, permanent bounces fail, and errors no longer log response bodies that can contain recipient PII.
- Standardized the provider environment name as `HONE_CLOUDFLARE_EMAIL_API_TOKEN`, documented `HONE_UNSUBSCRIBE_SECRET`, and installed both production values without adding them to Git or logs.
- Staged and verified the immutable runtime, performed two zero-active-chat reads, atomically switched `/opt/hone/current`, and restarted only `hone-web.service`. The previous `d379cccc` release remains available for rollback.

## Verification

- Focused tests passed: core email 9, core unsubscribe 7, scheduler unsubscribe 4, Web API unsubscribe 4, and public subscription model/autosend 9.
- Full local gates passed: changed-file formatting, workspace check/test, Web tests and public build, Public Community Edge Worker typecheck/45 tests, CI-safe regression suite, diff check, and pre-push gitleaks.
- GitHub Actions passed for the implementation revision: CI, Runtime Image, Secret Scan, and Code Quality. The runtime workflow verified the immutable manifest before deployment.
- Production `/api/meta` reports exact Git SHA `9eff909a`, `source=ghcr_linux_oci`, healthy PostgreSQL/S3, `cloud_storage_authoritative=true`, and `local_durable_dependency_count=0`. `hone-web.service` is active with `NRestarts=0`; active chats are zero.
- Public `/`, `/chat`, and `/pushes` return `200`; unauthenticated `/api/public/auth/me` and `/api/public/subscriptions` return JSON `401`; invalid GET/POST unsubscribe requests reach the new public HTML handler and return `404`; public `PUT` CORS preflight returns `200`.
- Cloudflare Pages serves entry `index-DMkBoGCs.js` and push chunk `public-pushes-C6E0BVeY.js`; the deployed code contains the new `workday`/`trading_day` schedule paths and Monday-first content.
- The supplied Cloudflare token passed the provider's authenticated token-verification endpoint before installation. No live message was sent because no designated test recipient was provided.

## Risks / Follow-ups

- Scheduled push email delivery is **not** complete: `hone_core::email::EmailSender` is not called by the scheduler, and there is no reviewed mapping from a scheduled actor to a verified recipient email. Provider configuration and existing verification-email assembly are live, but scheduler-to-email delivery needs a separate implementation and production canary before it can be advertised.
- Rotate the Cloudflare token because it was pasted into chat. Update only `HONE_CLOUDFLARE_EMAIL_API_TOKEN` through the protected runtime environment, then perform a zero-chat restart and repeat provider-assembly/public acceptance checks.
- Rotating `HONE_UNSUBSCRIBE_SECRET` invalidates all previously issued unsubscribe links. Preserve it across ordinary deployments and rotate only with an explicit migration/expiry plan.
- `origin.hone-claw.com` still redirects to the pre-existing stale tunnel error endpoint. The user-facing `hone-claw.com/api/public/*` Worker path is healthy; origin alias repair remains separate infrastructure work.

## Next Entry Point

To complete scheduled email pushes, start from `hone_core::email::EmailSender`, define a verified recipient-resolution contract for authenticated Web actors, invoke it from the scheduler delivery boundary with idempotency/observability, and add a no-PII provider fixture plus one designated-recipient production canary. For immediate backend rollback, confirm two zero-active-chat reads, atomically restore `/opt/hone/current` to `/opt/hone/releases/d379cccc6e909129d02e726c04919e7c7ec250e1-ghcr-runtime`, restart `hone-web.service`, and repeat exact `/api/meta`, cloud-authority, and public unsubscribe probes.
