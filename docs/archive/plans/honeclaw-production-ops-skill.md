# Honeclaw Production Ops Repository Skill

- title: Honeclaw Production Ops Repository Skill
- status: archived
- created_at: 2026-08-01
- updated_at: 2026-08-01
- owner: Codex
- related_files: `.agents/skills/honeclaw-production-ops/`, `docs/runbooks/gcp-backend-access.md`, `docs/runbooks/backend-deployment.md`, `docs/repo-map.md`
- related_docs: `docs/handoffs/2026-08-01-honeclaw-production-ops-skill.md`, `docs/archive/index.md`

## Goal

Move the private personal production-operations skill into the repository as a safe, reusable Codex skill without publishing account identity, credentials, or private GCP resource identifiers.

## Scope

- Added the repository-owned `.agents/skills/honeclaw-production-ops` skill and UI metadata.
- Added a focused GCP backend access runbook that discovers live project and instance coordinates instead of committing them.
- Updated the general backend runbook and repository map to reflect the Cloudflare plus GCP deployment boundary and co-located PostgreSQL topology.
- Removed the duplicate personal skill after the repository copy was validated.
- Prepared the scoped documentation/skill change for a normal branch push without a release or tag.

## Validation

- Parsed the skill frontmatter and `agents/openai.yaml` and verified all referenced runbooks exist.
- Ran the documented live `gcloud compute instances list` discovery command with output suppressed; the command completed successfully.
- Scanned the changed files for the known account email, private project/instance identifiers, OAuth URLs, tokens, and key material; no matches remained.
- Ran `git diff --check`; no whitespace errors were reported.
- No Rust or frontend tests were required because this task changes only Codex skill and Markdown operational documentation.

## Documentation Sync

- Updated `docs/repo-map.md` for the repository-owned operator skill and focused runbook.
- Updated `docs/runbooks/backend-deployment.md` to replace the obsolete current macOS/Sunny-ngrok assumption with live GCP/Cloudflare discovery.
- Added `docs/handoffs/2026-08-01-honeclaw-production-ops-skill.md` and the corresponding archive index entry.
- Removed the task from `docs/current-plan.md` and archived this plan after completion.

## Risks / Open Questions

- The public repository deliberately lacks private GCP coordinates; operators need an already authorized local `gcloud` context and must stop if instance selection is ambiguous.
- Cloudflare product, route, origin mapping, service names, ports, and process supervisor remain live-discovery facts.
- No production configuration, process, database, Cloudflare route, or traffic was changed by this documentation task.
