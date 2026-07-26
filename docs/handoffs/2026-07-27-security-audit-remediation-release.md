# Honeclaw Security Audit Remediation And v0.15.1 Release

- title: Honeclaw Security Audit Remediation And v0.15.1 Release
- status: `done`
- created_at: `2026-07-27`
- updated_at: `2026-07-27`
- owner: Codex
- related_files: `packages/app/public/_worker.js`, `crates/hone-web-api/src/public_auth.rs`, `crates/hone-web-api/src/routes/public.rs`, `crates/hone-channels/src/attachments/ingest.rs`, `crates/hone-tools/src/skill_tool.rs`, `skills/chart_visualization/scripts/render_chart.py`, `crates/hone-core/src/config/`, `crates/hone-core/src/cloud_runtime.rs`, `memory/src/web_auth.rs`, `bins/hone-cli/src/`, `docs/releases/v0.15.1.md`
- related_docs: `docs/archive/plans/security-audit-remediation-release.md`, `docs/invariants.md`, `docs/runbooks/backend-deployment.md`
- related_prs: release commit containing this handoff; annotated tag `v0.15.1`

## Summary

The authorized repository scan and bounded, non-destructive production checks completed with full declared coverage. Twelve findings were validated: six Medium resource/abuse/secret-persistence issues and six Low browser/local-secret/auth-oracle issues. All twelve have implementation fixes and regressions in `v0.15.1`; no Critical or High issue was found.

## What Changed

- Public Pages responses enforce HSTS, anti-framing CSP/XFO, MIME-sniffing protection, and a strict referrer policy.
- Public SMS auth uses bounded limiter cardinality, per-phone and per-IP budgets, successful resend cooldown, generic send/login responses, provider-independent response timing, and code verification before invite lookup.
- ZIP and TAR-family extraction enforce per-file, cumulative-byte, expansion-ratio, and per-actor persistent-storage budgets with partial-output cleanup.
- Chart rendering validates bounded argument bytes and semantic complexity before expensive imports and terminates child processes on timeout/drop.
- Credential YAML writes are atomic and owner-only; existing local canonical/effective config files were repaired to `0600`; administrator-token changes require full restart; Discord token prompts are hidden.
- One-time cloud Web API keys are digest-only after issuance, and cloud schema initialization removes the legacy plaintext JSON field.
- Workspace, desktop, user-app, and iOS version sources are synchronized to `0.15.1`. The architecture SVG did not change because this patch adjusts enforcement, thresholds, headers, and serialization without changing documented topology or module boundaries.

## Verification

- Codex Security scan `c771f7ab-01ee-4f83-9a14-19e6df35834b`: 45 full-file reviews; 12/12 validation and attack-path receipts; complete coverage ledger; sealed Markdown/JSON/SARIF output; three-option structural hardening portfolio.
- `cargo check --workspace --all-targets --exclude hone-desktop --exclude hone-user-app`
- `cargo test --workspace --all-targets --exclude hone-desktop --exclude hone-user-app`
- Exact changed-file `rustfmt --check` and `git diff --check`
- `bun run test:web` and `bun run build:web:public`
- `cd workers/public-community-edge && bun install --frozen-lockfile && bun run typecheck && bun run test`
- `bash tests/regression/run_ci.sh`
- `bash scripts/prepare_release_notes.sh v0.15.1 /tmp/release-notes-v0.15.1.md`
- Built `packages/app/dist-public/_worker.js` contains the reviewed response-header policy.
- The four unrelated event-engine diffs were excluded from staging; their patch digest stayed unchanged across the required rebase onto `origin/main`.
- Final release acceptance requires the release commit and annotated tag on the remote, the tag-triggered Release workflow, and the live `hone-claw.com` header probe.

## Risks / Follow-ups

- Browser cooldowns, captcha, Aliyun quotas, and Cloudflare WAF remain defense-in-depth; the server-side SMS budgets are the authoritative application boundary.
- The tactical resource limits close the validated chart/archive paths. A shared admission-lease API or isolated conversion worker is a future structural option if production load justifies the added operational boundary.
- Backend fixes become active only after the established external process supervisor deploys/restarts the release. Do not start an ad-hoc second backend process.
- Cloud schema cleanup is idempotent, but operators should confirm the migration marker and cloud health during the supervised backend rollout.

## Next Entry Point

Use `docs/releases/v0.15.1.md` for user-facing changes, `docs/runbooks/backend-deployment.md` for controlled rollout and live probes, and the sealed Codex Security scan `c771f7ab-01ee-4f83-9a14-19e6df35834b` for detailed local evidence.
