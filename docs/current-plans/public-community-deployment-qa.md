# Public Community Deployment And QA

- title: Public Community Deployment And QA
- status: in_progress
- created_at: 2026-07-12
- updated_at: 2026-07-12
- owner: Codex
- related_files:
  - crates/hone-core/src/cloud_runtime.rs
  - crates/hone-web-api/src/routes/public_community.rs
  - packages/app/src/pages/public-community.tsx
  - packages/app/src/pages/chat.tsx
- related_docs:
  - docs/archive/plans/public-community-readonly.md
  - docs/runbooks/backend-deployment.md

## Goal

Review the public community experience on desktop and mobile, fix production-readiness issues, deploy the current user service, run production smoke checks, and commit/push the complete scoped change set.

## Scope

- Inspect current production version and deployment boundary.
- Exercise `/chat` community entry, unread dot, timeline pagination, protected-file states, and media preview at desktop/mobile sizes.
- Deploy through the repository runbook without creating a formal release tag.
- Stage only community-related code/docs, commit intentionally, and push the current branch.

## Validation

- Rust checks/tests and Web tests/typecheck/production build.
- Desktop and mobile browser QA against the local candidate and deployed service.
- Production API/asset/runtime smoke checks after upgrade.

## Progress

- Backend runtime rebuilt and upgraded from `0.13.0` to `0.14.1`; local, origin, and Worker community routes now return authenticated `401` instead of `404` for anonymous requests.
- Cloud doctor passed with PostgreSQL schema and R2 healthy.
- Desktop and `390x844` browser QA passed for the timeline, multi-image grid, zoom controls, modal close/focus behavior, mobile quick-action strip, unread dot, and `/chat` -> `/community` navigation.
- Remaining step: push the production branch, wait for Cloudflare Pages to publish the new asset fingerprint, then archive this plan.

## Documentation Sync

- Update this plan, the existing community handoff, and archive index when complete.

## Risks / Open Questions

- The worktree may contain unrelated user changes; never stage them implicitly.
- Source-protected files remain metadata-only and must not become downloadable during QA.
- A formal `v*` release is out of scope unless the user explicitly requests it.
