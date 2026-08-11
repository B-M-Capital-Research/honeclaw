# HONE Daily Production Deployment — 2026-08-12

- title: HONE Daily Production Deployment — 2026-08-12
- status: archived
- created_at: 2026-08-12 02:02 CST
- updated_at: 2026-08-12 02:09 CST
- owner: Codex
- related_files:
  - `.github/workflows/runtime-image.yml`
  - `scripts/stage_ghcr_runtime.sh`
  - `scripts/check_backend_runtime_env.sh`
  - `packages/app/src/lib/public-subscription-model.test.ts`
- related_docs:
  - `docs/runbooks/backend-deployment.md`
  - `docs/handoffs/2026-08-12-daily-production-deployment-blocked.md`
  - `/Users/fengming2/.codex/automations/hone-daily-production-deploy/memory.md`

## Goal

Freeze exact remote revision `08547811ebeaf6e7a1c078b7516535f4d8d9477d`, validate it in an isolated worktree, and deploy only its immutable runtime digest after every pre-cutover gate passes.

## Scope

- Preserved the user workspace and froze the exact remote target without following later `main` movement.
- Recorded the production revision, current and rollback releases, executable/working directory, disk, runtime env, Web/channel/dependency states, cloud authority, public boundaries, and repeated active-chat counts.
- Reviewed the complete online-to-target delta and classified it as two docs-only commits with no frontend, Worker, runtime skill, share asset, runtime-image workflow, or executable source changes.
- Diagnosed the frozen target's failed GitHub CI as an order-dependent Web test that assumed an English global locale. Fixed it in remote branch `automation/fix-public-subscription-locale-20260812` without retargeting this deployment.
- Stopped before runtime image, staging, cutover, browser smoke, or real canary because the direct origin hard gate failed.

## Validation

- Production `/api/meta`: exact `0c6d0328a04c31eb07839bf6a3b91baa65fb821f`, `ghcr_linux_oci`, `cloud_mode=cloud`, PostgreSQL/R2 healthy, cloud-authoritative, zero local durable dependencies.
- Active-chat reads before and after all work: all `0`.
- `hone-web.service`: active/running, `NRestarts=0`, executable inside the unchanged current release; Caddy and PostgreSQL active; runtime env validator, Chromium 151, and Noto CJK checks passed.
- Public `/` and `/chat`: `200`; auth/events: application JSON `401`; required security headers present; entry asset `/assets/index-DAePXR8U.js`.
- Blocking origin probe: `origin.hone-claw.com/api/public/auth/me` returned Sunny-Ngrok `307` and `Tunnel not found`, not application JSON `401`.
- Frozen target GitHub: Rust job, Secret Scan, CodeQL, and Code Quality passed; frontend job failed two locale-sensitive expectations, so Worker steps did not run and no runtime image existed.
- Minimal test-isolation repair `6b82b519`: focused Web `4/4`, full Web `408/408`, and `git diff --check` passed locally; pushed only to its remote repair branch because `main` moved after freeze.

## Documentation Sync

- Archived this plan, added the blocked-deployment handoff and archive index entry, and appended the automation memory without secrets or private user data.
- No repository architecture, runtime behavior, or deployment workflow changed, so no decision/ADR/runbook update was required.

## Risks / Open Questions

- The stale direct origin route has blocked six consecutive daily runs and must be repaired and independently accepted before another cutover.
- The only configured channel instance, `hone-channel@feishu.service`, is enabled but has been inactive since a graceful SIGTERM at 2026-08-12 00:11 CST; it was not changed because the pre-cutover stop contract forbids production mutation after a failed gate.
- The frozen target remains failed on `main`; the proven repair is on remote branch `automation/fix-public-subscription-locale-20260812` and must be reviewed/merged separately.
- Production disk has about 2.7 GiB free, above but close to the 2 GiB staging floor.

## Outcome

未部署。No release was staged, no service was restarted, no current symlink was changed, no browser/canary was run, and no rollback was needed.
