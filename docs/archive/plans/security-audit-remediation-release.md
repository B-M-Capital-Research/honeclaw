# Honeclaw Security Audit Remediation And Release

- title: Honeclaw Security Audit Remediation And Release
- status: `archived`
- created_at: `2026-07-27`
- updated_at: `2026-07-27`
- owner: Codex
- related_files: `packages/app/public/_worker.js`, `crates/hone-web-api/src/{public_auth.rs,routes/public.rs}`, `crates/hone-tools/src/{skill_tool.rs,skill_runtime.rs}`, `skills/chart_visualization/scripts/render_chart.py`, `crates/hone-core/src/config/`, `bins/hone-cli/src/`, `memory/src/web_auth.rs`, `crates/hone-channels/src/attachments/ingest.rs`, `docs/releases/`, workspace version files
- related_docs: `docs/current-plan.md`, `docs/invariants.md`, `docs/runbooks/backend-deployment.md`, `docs/handoffs/`, `docs/archive/index.md`

## Goal

Close every reportable item from the authorized 2026-07-27 repository and non-destructive production security assessment, prove the security boundaries with focused regressions and repository gates, then publish one formal release from `main`.

## Scope

- Complete the scan's validation, attack-path analysis, coverage ledger, and sealed canonical report.
- Add response-level frame and transport protections for the public site without breaking required application behavior.
- Bound chart renderer parameters and child-process lifetime/resource use.
- Enforce owner-only credential-file permissions, correct runtime apply semantics for administrator-token rotation, and mask Discord credential input.
- Bound anonymous authentication limiter cardinality, successful SMS resend behavior, and invitation-membership response differences.
- Bound attachment-reference count, duplicate expansion, cumulative extracted bytes, and successful extraction lifecycle for ZIP and TAR-family archives.
- Prevent cloud Web API-key plaintext from entering persistent records or later list responses, including compatibility cleanup of existing records.
- Preserve the user's unrelated event-engine working-tree edits and exclude them from the release commit.
- Update version metadata, user-facing bilingual release notes, handoff/archive context, then push `main` and an annotated release tag.

## Validation

- Add at least one focused automated regression for every fixed security boundary, with positive coverage for legitimate behavior.
- Run affected Rust crate tests, Web tests/type checks where applicable, changed-file formatting, and CI-safe regression scripts.
- Run the repository release baseline: workspace `cargo check`, release-note preparation, and applicable full CI gates from `AGENTS.md`.
- Re-run bounded black-box checks against the built/deployed public surface where deployment access is available; never send real SMS, stress storage/CPU, or use destructive payloads.
- Audit the final staged diff and prove the four pre-existing event-engine files are not included.
- Confirm `main` and the annotated tag reach the remote and that the tag-triggered release workflow starts.

## Documentation Sync

- Keep this plan and `docs/current-plan.md` current while the task is active.
- Record durable security invariants or deployment/header requirements in `docs/invariants.md` and/or `docs/runbooks/backend-deployment.md` when the implementation establishes them.
- On completion, create/update one handoff, move this plan to `docs/archive/plans/`, remove it from `docs/current-plan.md`, and append `docs/archive/index.md`.
- Create `docs/releases/vX.Y.Z.md` from the repository template and state whether `resources/architecture.svg` required an update.

## Risks / Open Questions

- Provider-side SMS quotas and Cloudflare WAF rules are not a substitute for application limits and may remain unreadable from the local repository.
- Some local configuration files may already be too permissive; the fix must repair existing files without exposing their contents.
- Cloud records may already contain plaintext API keys; compatibility cleanup must remove the field without invalidating stored hashes.
- A production deployment may require external service coordination after release artifacts are built; if deployment authority or credentials are unavailable, repository release completion and live rollout status must be reported separately.

## Completion

- Codex Security scan `c771f7ab-01ee-4f83-9a14-19e6df35834b` completed with full repository coverage, 12 validated findings (6 Medium, 6 Low), individual validation/attack-path/write-up artifacts, a structural hardening portfolio, and a sealed canonical report.
- Every validated finding has a targeted implementation and automated regression in the release change set.
- The workspace version, desktop Tauri versions, user-app Tauri version, iOS marketing version, bilingual release notes, durable invariants, deployment runbook, handoff, and archive index are synchronized for `v0.15.1`.
- The four pre-existing event-engine working-tree files remain outside the release scope and are preserved byte-for-byte across the remote rebase.
- Local full gates and post-rebase verification are required to pass before the release commit and tag; remote workflow and live Pages headers are the final acceptance gates.
