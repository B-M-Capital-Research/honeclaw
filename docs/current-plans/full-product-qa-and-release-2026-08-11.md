# Full Product QA And Release 2026-08-11

- title: Full Product QA And Release 2026-08-11
- status: `blocked`
- created_at: `2026-08-11`
- updated_at: `2026-08-11`
- owner: `Codex / stress-test Agent / functional-test Agent`
- related_files:
  - `crates/hone-web-api/src/routes/`
  - `packages/app/src/components/`
  - `packages/app/src/pages/chat.tsx`
  - `skills/hari-invest/`
  - `skills/company-thesis-ratings/`
- related_docs:
  - `docs/decisions.md`
  - `docs/runbooks/backend-deployment.md`
  - `docs/runbooks/source-web-startup.md`

## Goal

Independently validate all HONE product functions added or changed on 2026-08-10 and 2026-08-11, produce one evidence-backed test report, and release only if functional, performance, repository, security and production-readiness gates have no blocking failure.

## Scope

- One stress-test Agent covers bounded local concurrency, latency, errors and process stability across shared dashboards and authenticated product APIs.
- One functional-test Agent covers desktop/mobile UI, daily tools, valuation/rating integration, evidence boundaries, research/community isolation and conversation behavior.
- The primary Agent runs repository-level gates, reconciles both reports, checks release provenance and performs deployment/production canaries only after a clean go decision.
- Simulation data is test-only and must never be promoted as production evidence.

## Validation

- Stress report: request counts, concurrency, success rate, p50/p95/p99, error classification and stability observations.
- Functional report: per-feature pass/fail/block status, API/browser evidence, expected external-key degradations and defect reproduction steps.
- Repository gates: changed-file formatting, workspace check/test, Web tests, Worker gates, CI-safe regressions, public production build and secret scan.
- Release gates: clean committed revision, pushed provenance, approved deployment mechanism, cloud authority health, static asset hash change and authenticated/unauthenticated production canaries.

Current result: code/data-integrity remediation is complete. Web API (298 passed, 2 credentialed tests ignored), Web (465 passed), TypeScript, production build and CI-safe regressions all pass. The original 30,400-request stress run and the 6,000-request authenticated keep-alive retest both had zero failures. Release remains blocked only by absent safe-model/FMP/search provider configuration and by the lack of a clean, current, pushed candidate revision; external credentials remain fail-closed and are not replaced with simulated facts.

## Documentation Sync

- Write `docs/handoffs/2026-08-11-full-product-qa-and-release.md` with the final report and release/no-go conclusion.
- If complete, move this plan to `docs/archive/plans/`, remove this index entry and append `docs/archive/index.md`.
- If blocked, keep the plan active and record exact blockers and the next entry point.

## Risks / Open Questions

- The shared `main` worktree contains a large uncommitted product batch; release must not proceed from a dirty or unpushed revision.
- Several dashboards truthfully degrade without production provider/model credentials. Expected missing-data states are not defects, but a production release still requires the configured real evidence paths to remain fail-closed.
- Backend origin replacement requires the approved managed-host path and cloud-authority checks; a local Vite/backend restart is not production deployment.
- The AI completeness overclaim, test-sample contamination, simulation leakage, localization/design contracts and Web/CI failures are closed. Remaining blockers are the unavailable safe conversation model, absent real provider/model evidence and unreleasable dirty/behind Git provenance. No deployment may proceed until these are closed and the exact candidate is retested.
