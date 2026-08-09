# Push Subscription / Email Production Rollout

- title: Push subscription and Cloudflare email production rollout
- status: in_progress
- created_at: 2026-08-09
- updated_at: 2026-08-09
- owner: Codex
- related_files:
  - `crates/hone-core/src/email.rs`
  - `crates/hone-core/src/unsubscribe_token.rs`
  - `crates/hone-scheduler/src/lib.rs`
  - `crates/hone-web-api/src/routes/public_subscriptions.rs`
  - `crates/hone-web-api/src/routes/unsubscribe.rs`
  - `packages/app/src/pages/public-pushes.tsx`
  - `config.example.yaml`
- related_docs:
  - `docs/runbooks/backend-deployment.md`

## Goal

Review the new push-subscription, login-free unsubscribe, and Cloudflare Email Sending changes; close production blockers; then deploy an exact reviewed revision to production GCE with the Cloudflare token and a newly generated HONE unsubscribe signing secret installed through secret-safe runtime paths.

## Scope

- Fast-forward local `main` from `origin/main` without overwriting local work.
- Review authentication, actor scoping, unsubscribe capability signing, HTML handling, provider error handling, scheduler integration, and frontend/API contracts.
- Distinguish production-ready subscription/unsubscribe behavior from email-delivery plumbing that is not yet connected to a scheduler delivery path.
- Run focused and repository-required validation, then create an immutable source-runtime deployment and perform production canaries.
- Never commit, print, or persist the Cloudflare token or generated signing secret outside the approved production secret environment.

## Validation

- Focused Rust tests for core email/unsubscribe, scheduler, and Web API routes.
- Web unit tests and public production build.
- Repository CI contract proportional to the final change set, including formatting, workspace check/test, Edge Worker checks, and CI-safe regressions.
- Exact-revision runtime manifest verification, production health checks, public subscription route checks, login-free unsubscribe negative/confirmation checks, and post-restart active-chat/process checks.

## Documentation Sync

- Keep this plan and `docs/current-plan.md` current while review or rollout is active.
- On completion, write a production handoff with exact revision, verification evidence, configured secret names (never values), rollback entry point, and any email-delivery gap.
- Move this plan to `docs/archive/plans/`, remove it from the active index, and add the completed task to `docs/archive/index.md`.

## Risks / Open Questions

- The new `EmailSender` is provider plumbing only; production email delivery must not be claimed until an actual scheduler call path and recipient source are implemented and verified.
- Login-free unsubscribe tokens are bearer capabilities. Secret rotation invalidates old links; missing secret must fail closed.
- A backend restart can interrupt active chats, so deployment requires a zero-active-chat gate and a verified rollback artifact.
- The token was supplied in chat and should be rotated after this rollout even though it will not be written to the repository or echoed by deployment commands.
