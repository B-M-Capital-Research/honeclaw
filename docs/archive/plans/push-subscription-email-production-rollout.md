# Push Subscription / Email Production Rollout

- title: Push subscription and Cloudflare email production rollout
- status: done
- created_at: 2026-08-09
- updated_at: 2026-08-09
- owner: Codex
- related_files:
  - `crates/hone-core/src/email.rs`
  - `crates/hone-core/src/unsubscribe_token.rs`
  - `crates/hone-scheduler/src/lib.rs`
  - `crates/hone-web-api/src/routes/mod.rs`
  - `crates/hone-web-api/src/routes/public_subscriptions.rs`
  - `crates/hone-web-api/src/routes/unsubscribe.rs`
  - `packages/app/src/components/public-subscription-manager.tsx`
  - `packages/app/src/lib/public-subscription-model.ts`
  - `config.example.yaml`
- related_docs:
  - `docs/runbooks/backend-deployment.md`
  - `docs/handoffs/2026-08-09-push-subscription-email-production-rollout.md`

## Goal

Review the new push-subscription, login-free unsubscribe, and Cloudflare Email Sending changes; close production blockers; then deploy an exact reviewed revision to production GCE with the Cloudflare token and a newly generated HONE unsubscribe signing secret installed through secret-safe runtime paths.

## Scope

- Fast-forward local `main` from `origin/main` without overwriting local work.
- Review authentication, actor scoping, unsubscribe capability signing, HTML handling, provider error handling, scheduler integration, and frontend/API contracts.
- Distinguish production-ready subscription/unsubscribe behavior from email-delivery plumbing that is not yet connected to a scheduler delivery path.
- Run focused and repository-required validation, then deploy an immutable GHCR runtime and perform production canaries.
- Never commit, print, or persist the Cloudflare token or generated signing secret outside the approved production secret environment.

## Validation

- Focused Rust tests for core email/unsubscribe, scheduler, and Web API routes passed.
- Web unit tests and the public production build passed.
- The complete repository CI contract passed locally and in GitHub Actions for `9eff909aba898dfd12b268a75f71bc269f1e7c4d`.
- GHCR digest `sha256:1335325fb6075c98a85ee585bbfc12a3e1073a2b90d3ad860e3fcafec13ba758` was staged and verified before cutover.
- Production reports the exact revision, healthy PostgreSQL/S3 authority, zero local durable dependencies, an active service, and zero active chats.
- Public `/`, `/chat`, and `/pushes` return `200`; unauthenticated account/subscription APIs return application `401`; invalid login-free unsubscribe reaches the public HTML handler with `404`; public `PUT` CORS preflight returns `200`.

## Documentation Sync

- Updated `docs/runbooks/backend-deployment.md` with the production environment names and the current scheduler/email integration boundary.
- Wrote `docs/handoffs/2026-08-09-push-subscription-email-production-rollout.md` with deployment identity, validation, rollback, and follow-up risks.
- Removed this task from `docs/current-plan.md`, archived this plan, and added the task to `docs/archive/index.md`.

## Risks / Open Questions

- `hone_core::email::EmailSender` is production-configured provider plumbing, and the existing email-verification service is assembled, but scheduled push delivery still has no call path from the scheduler to `EmailSender` and no reviewed recipient-resolution source. Do not claim scheduled email pushes until that integration and its tests are added.
- Login-free unsubscribe tokens are bearer capabilities. Rotating `HONE_UNSUBSCRIBE_SECRET` invalidates existing links; missing configuration fails closed.
- The Cloudflare token was supplied in chat and should be rotated after the rollout even though it was not committed or printed by deployment commands.
- `origin.hone-claw.com` retains the pre-existing stale tunnel redirect. The user-facing Worker route is healthy and was the production acceptance surface.
