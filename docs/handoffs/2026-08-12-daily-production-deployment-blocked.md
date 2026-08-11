# Daily Production Deployment Blocked — 2026-08-12

- title: Daily Production Deployment Blocked — 2026-08-12
- status: done
- created_at: 2026-08-12 02:02 CST
- updated_at: 2026-08-12 02:09 CST
- owner: Codex
- related_files: `packages/app/src/lib/public-subscription-model.test.ts`
- related_docs: `docs/archive/plans/daily-production-deployment-2026-08-12.md`, `docs/runbooks/backend-deployment.md`, `docs/archive/index.md`
- related_prs: no PR; repair commit `6b82b519` on remote branch `automation/fix-public-subscription-locale-20260812`

## Summary

The daily deployment froze exact remote revision `08547811ebeaf6e7a1c078b7516535f4d8d9477d` and stopped before any production mutation. The direct origin still returned the stale Sunny-Ngrok `307 Tunnel not found` surface rather than the required application JSON `401`. The target also had a real GitHub CI failure caused by a global-locale-dependent Web test. Production remained on exact revision `0c6d0328a04c31eb07839bf6a3b91baa65fb821f`; no runtime image was triggered, no release was staged, no symlink or service was changed, and no rollback was required.

## What Changed

- The online-to-target delta contained only `55d7ab4d` and `08547811`, changing six governance/handoff files; no executable, frontend, Worker, runtime skill/share asset, earnings implementation, or runtime-image workflow changed.
- The failed subscription schedule tests expected English strings while consuming the process-global locale without setting or restoring it. Repair commit `6b82b519` now scopes those assertions to English and restores the prior locale in `finally`.
- The repair passed focused `4/4`, full Web `408/408`, and `git diff --check`, then was pushed to its own remote branch. It was not merged or used as a deployment target because remote `main` advanced after the frozen revision; later commits remain next-run work.
- Repository context only was updated in an isolated tracking worktree. No architecture decision or operational runbook changed.

## Verification

- Production current, actual Web executable, and `/api/meta.build.git_sha` all remained `0c6d0328...`; target release `08547811...-ghcr-runtime` remained absent.
- `/api/meta` remained `cloud_mode=cloud`, cloud-authoritative, PostgreSQL/R2 healthy, and `local_durable_dependency_count=0`. Runtime env validation, Chromium 151, Noto CJK, Caddy, and PostgreSQL checks passed.
- Every active-chat read before and after the aborted deployment returned `0`.
- `hone-web.service` remained active/running with `NRestarts=0`. The only configured channel, `hone-channel@feishu.service`, was enabled but inactive/dead after a graceful SIGTERM at 2026-08-12 00:11 CST; no crash loop or credential failure was observed.
- Public `/` and `/chat` returned `200`; auth and SSE unauthenticated boundaries returned application JSON `401`; HSTS, CSP frame denial, `X-Frame-Options: DENY`, `nosniff`, and strict referrer policy were present; entry asset was `/assets/index-DAePXR8U.js`.
- Direct `origin.hone-claw.com/api/public/auth/me` returned `307`, `x-powered-by: Sunny-Ngrok`, and `Tunnel not found`, which is a mandatory pre-cutover stop.
- Frozen-target GitHub CI `31512369953`: Rust job passed; frontend failed the two locale-sensitive tests (`406 pass / 2 fail`), skipping Worker steps. Secret Scan `31512369831` and both CodeQL/Code Quality runs passed. No runtime image/digest was created for the docs-only target.
- Because cutover never began, browser smoke, real Web canary, and earnings/PDF canary were correctly not run.

## Risks / Follow-ups

- Fix and independently accept the direct origin DNS/tunnel/backend JSON `401` contract before another daily deployment. This is the sixth consecutive blocked run.
- Restore and verify the enabled Feishu service through an explicitly authorized production recovery/deployment action; require `active`, recent `Stream connected`, zero warning/credential/crash-loop errors, and preserve active user work.
- Review/merge remote repair branch `automation/fix-public-subscription-locale-20260812`, then require a green main CI before selecting any future deployment target.
- The root filesystem has about 2.7 GiB free. It is above the 2 GiB export floor but should be watched before staging another runtime.

## Next Entry Point

Begin the next run with a fresh `git fetch --prune`, freeze the then-current remote `main`, require direct origin JSON `401`, green GitHub CI, all enabled channel instances active, and two zero-active-chat reads. Do not reuse `08547811` or the repair branch as a production target without first proving the selected exact 40-character revision exists on remote `main` and has an immutable successful runtime image.
