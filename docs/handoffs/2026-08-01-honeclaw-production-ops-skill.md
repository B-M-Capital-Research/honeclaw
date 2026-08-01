# Honeclaw Production Ops Repository Skill

- title: Honeclaw Production Ops Repository Skill
- status: done
- created_at: 2026-08-01
- updated_at: 2026-08-01
- owner: Codex
- related_files: `.agents/skills/honeclaw-production-ops/`, `docs/runbooks/gcp-backend-access.md`, `docs/runbooks/backend-deployment.md`, `docs/repo-map.md`
- related_docs: `docs/archive/plans/honeclaw-production-ops-skill.md`, `docs/archive/index.md`
- related_prs: branch `codex/honeclaw-production-ops-skill`; no PR, release, or tag requested

## Summary

Honeclaw production operations now have a repository-owned Codex skill and a focused GCP access/diagnosis runbook. Future tasks can trigger the skill from production incident, GCP/IAP, PostgreSQL, restart, migration, or deployment language without referring to the originating chat.

## What Changed

- Added `.agents/skills/honeclaw-production-ops` with native Codex metadata and links to repository runbooks.
- Added `docs/runbooks/gcp-backend-access.md` for authenticated project/instance discovery, IAP SSH, remote authority checks, VM/application/PostgreSQL baselines, boundary diagnosis, mutation safety, and completion evidence.
- Updated `docs/runbooks/backend-deployment.md` to reflect the current Cloudflare plus GCP boundary and remove the obsolete claim that the current backend is a macOS/Sunny-ngrok host.
- Updated `docs/repo-map.md` to distinguish repository-owned Codex operator skills from runtime/end-user `skills/`.
- Removed the duplicate personal skill after the repository copy passed validation.
- Kept account identity, project ID, instance name, zone, IP address, OAuth/2FA data, tokens, keys, and database URLs out of Git; the runbook discovers private coordinates from live authenticated `gcloud` state.

## Verification

- Skill frontmatter and `agents/openai.yaml` parsed successfully; referenced runbooks exist.
- The documented `gcloud compute instances list` discovery command completed successfully against the authorized local context with output suppressed.
- Sensitive-value scan found no known account email, private project/instance identifier, OAuth URL, token, or key material in the changed files.
- `git diff --check` passed.
- No Rust/frontend tests were run because the change is limited to Codex skill and Markdown operational documentation.

## Risks / Follow-ups

- Live Cloudflare routing, GCP instance selection, application supervisor, ports, deployed revision, and PostgreSQL runtime must still be rediscovered during every incident.
- Successful IAP SSH proves access only; it does not prove backend, database, or public-route health.
- No production mutation or end-to-end health declaration was made in this task.

## Next Entry Point

Start with `.agents/skills/honeclaw-production-ops/SKILL.md`, then read `docs/runbooks/gcp-backend-access.md`. Use the relevant sections of `docs/runbooks/backend-deployment.md` for public frontend, Worker, origin, cloud-authority, drain, deployment, and rollback checks.
