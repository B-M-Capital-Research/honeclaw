# Public Admin Usage Production Rollout

- title: Public Admin Usage Production Rollout
- status: done
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

## Result

- Feature commit `39ce9ce54f5cbfea26e664459cb70edf3fd97292` was pushed to `main`; GitHub CI, secret scan, CodeQL, and Linux cache warm all completed successfully.
- Cloudflare Pages serves `index-vHyTHbU6.js`. Public `/` and `/me` return `200`; `/api/public/admin/usage` now reaches the new backend and correctly returns `401` without a session instead of the previous `404`.
- GCE atomically switched from `/opt/hone/releases/d48c1f50-feishu-heartbeat-20260801` to `/opt/hone/releases/39ce9ce54f5cbfea26e664459cb70edf3fd97292-admin-usage-20260802` after two zero-active-chat checks. Web and Feishu services are active, the embedded Git SHA matches, and PostgreSQL/R2 health is green.
- The initial local read-back was later found to target a different forwarded PostgreSQL connection and did not change the GCE production roles. The same-day correction ran the installed CLI on GCE with `hone-web.service`'s config/runtime env; all three targets changed from non-admin to admin with `verified_is_admin=true`, and the authenticated production Chrome page rendered both administrator sections.
- Build swap and staging artifacts were removed after cutover; OS Login 2FA was restored to `TRUE`, and the temporary gcloud configuration was deleted.

## Documentation Sync

- Append final rollout evidence and rollback notes to `docs/handoffs/2026-08-02-public-admin-usage-analytics.md`.
- Remove this task from `docs/current-plan.md`, move this plan to `docs/archive/plans/`, and add the completed rollout to `docs/archive/index.md`.

## Risks / Open Questions

- The local source-runtime cold-start preflight currently requires a compatibility shim because the installed `codex-acp` package does not implement the deployment script's `--version` probe. The cloud GCE release does not depend on that shim, but the local deploy contract should be reconciled in the ACP umbrella plan.
- GCE `/api/meta` proves the exact embedded Git SHA and binary hash, but reports build `source=unknown`; future immutable build tooling should preserve an explicit source label as well as the revision.
- The full local workspace Rust test command encountered ten pre-existing FMP stub failures in unchanged `hone-channels` tests. Focused Web API tests, the full required cargo check, all frontend/worker/regression gates, and GitHub CI were green.
- A local `.env` or loopback PostgreSQL forward can point at a different instance while still returning structurally valid users. Production role mutations and their read-after-write verification must run on GCE with the effective `hone-web.service` config/env; the runbook now records this constraint.
