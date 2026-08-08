# Production GCE Rollout — 2026-08-08

- title: Production GCE rollout for post-`b88069bb` changes
- status: in_progress
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
  - `docs/invariants.md`
  - `docs/decisions.md`

## Goal

Review the changes from the currently deployed production revision through `7efdc01d9c8d8d247c9d00d163585743274b2b2e`, fix any deployment blockers found by that review, validate the affected backend and public Web behavior, deploy the exact reviewed successor revision to the production GCE runtime, and record production acceptance and rollback evidence.

## Scope

- Resolve and record the current production revision and public asset identity.
- Review backend preturn evidence timing, scheduled entity handling, active-run progress recovery, and public frontend navigation/session/theme changes.
- Run focused tests plus the repository CI contract appropriate to the touched Rust, Web, and regression files.
- Confirm the immutable GHCR runtime artifact for the exact revision, stage it on the managed backend host, drain active chats, cut over through the production service manager, and verify revision/cloud authority/storage/channel/auth health.
- Verify the Cloudflare Pages public artifact and protocol markers for active-run recovery.

## Validation

- Review `git diff` and commit history from the live production revision to `7efdc01d`.
- Run changed-file formatting, Rust workspace checks/tests, Web tests/build, Edge Worker checks, and CI-safe regressions unless an earlier blocking failure makes later gates invalid.
- Require immutable runtime digest/revision agreement, sufficient GCE disk space, zero active chats before cutover, exact `/api/meta` revision after restart, and retained rollback release.
- Require public routes/security headers and deployed asset/protocol markers to match the reviewed public build.

## Documentation Sync

- Keep this plan and `docs/current-plan.md` current during execution.
- On success, move this plan to `docs/archive/plans/`, add a production handoff under `docs/handoffs/`, and append `docs/archive/index.md`.
- No `docs/repo-map.md`, `docs/invariants.md`, or decision update is planned unless review or deployment changes module boundaries, long-term constraints, or architecture.

## Risks / Open Questions

- The production revision may predate more than the five newly pulled commits; review scope must be derived from live metadata rather than assumed.
- Backend and frontend changes are protocol-coupled, so a partial backend-only or Pages-only rollout is not acceptable evidence of completion.
- Active user runs, a missing/failed GHCR image, insufficient disk, cloud-authority mismatch, or failed production canary is a stop/rollback condition.
