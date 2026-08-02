# Public Admin Usage Production Rollout

- title: Public Admin Usage Production Rollout
- status: in_progress
- created_at: 2026-08-02
- updated_at: 2026-08-02
- owner: Codex
- related_files: `crates/hone-web-api/src/routes/public_admin.rs`, `packages/app/src/components/public-admin-usage-panel.tsx`, `packages/app/src/components/public-admin-whitelist-panel.tsx`, `scripts/deploy_source_runtime.sh`
- related_docs: `docs/handoffs/2026-08-02-public-admin-usage-analytics.md`, `docs/runbooks/backend-deployment.md`, `docs/runbooks/public-user-admin.md`

## Goal

Push the completed administrator usage analytics feature to `main`, deploy the exact pushed backend revision through the managed drain/rollback state machine, wait for the public frontend cutover, and grant verified cloud administrator access to the three explicitly requested phone numbers.

## Scope

- Re-run the repository CI contract and the public production build before push.
- Run cloud schema health and per-phone administrator dry-runs before any role mutation.
- Commit and push the reviewed feature changes to `main`; do not create a release or tag.
- Deploy the exact pushed backend revision with active-chat drain, immutable build provenance, readiness checks, and automatic rollback on failure.
- Verify local origin and public frontend/auth boundaries, then apply and re-read all three administrator grants.
- Preserve unrelated active plans and user work.

## Validation

- `bash scripts/ci/check_fmt_changed.sh`
- `cargo check --workspace --all-targets --exclude hone-desktop --exclude hone-user-app`
- `cargo test --workspace --all-targets --exclude hone-desktop --exclude hone-user-app`
- `bun run test:web`
- `cd workers/public-community-edge && bun run typecheck && bun run test`
- `bash tests/regression/run_ci.sh`
- `bun run build:web:public`
- `cargo run -p hone-cli -- cloud doctor --ensure-schema`
- Per-phone `hone-cli cloud web-admin` dry-run, apply, and `verified_is_admin=true`
- Exact runtime revision, cloud authority, PostgreSQL/R2, auth boundary, active-chat, frontend asset, and page smoke probes

## Documentation Sync

- Append final rollout evidence and rollback notes to `docs/handoffs/2026-08-02-public-admin-usage-analytics.md`.
- Remove this task from `docs/current-plan.md`, move this plan to `docs/archive/plans/`, and add the completed rollout to `docs/archive/index.md`.

## Risks / Open Questions

- The current foreground local preview owns the backend port and must be drained/stopped before the managed production job can claim it; perform the final immutable build first so the cutover window is limited to process handoff and readiness.
- A phone is only granted when the authoritative PostgreSQL row is unique and active; missing, disabled, or duplicate identities remain fail-closed and must be reported.
- Cloudflare Pages deploys asynchronously after push, so public asset verification must wait for the expected bundle rather than assuming push completion equals cutover.
