---
name: honeclaw-production-ops
description: Safely inspect, diagnose, modify, restart, or deploy the Honeclaw production environment spanning Cloudflare and a GCP Compute Engine backend. Use when Codex is asked to investigate an online incident, inspect production health or logs, access the GCP backend through IAP, check the co-located PostgreSQL service, change production configuration or code, run a migration, restart a service, or verify a production deployment.
---

# Honeclaw Production Ops

Treat production operations as a live-evidence workflow. Start read-only, identify the first failing boundary, and preserve a rollback path before changing state.

Read [the GCP backend access runbook](../../../docs/runbooks/gcp-backend-access.md) before accessing the production VM. For public frontend, Worker, origin, cloud-authority, drain, or deployment checks, also read the relevant section of [the backend deployment runbook](../../../docs/runbooks/backend-deployment.md).

## Stable topology boundary

- Cloudflare owns public delivery surfaces; discover the active Pages, Worker, DNS, route, and origin state live.
- The backend and PostgreSQL are currently co-located on one GCP Compute Engine VM reached through IAP. Verify that this remains true before every incident or change.
- The public repository intentionally stores no Google account identity, project ID, instance name, zone, IP address, OAuth state, token, SSH private key, database URL, or two-step verification data.
- Resolve private coordinates from the operator's authenticated local `gcloud` configuration and current instance inventory. Never guess them from an old transcript.

## Required workflow

1. Check the worktree and read the applicable plan, handoff, invariant, and runbook context before changing repository code.
2. Verify the active `gcloud` account and project without printing credentials. List current instances and make the target selection explicit.
3. Connect through IAP and run `id` plus `sudo -n true && echo sudo-ok` before relying on remote authority.
4. Establish a read-only VM baseline: resource pressure, disk, listeners, failed services, containers, deployed revision, local health, and PostgreSQL runtime/readiness.
5. Compare the local origin result with the public Cloudflare path. Successful SSH proves access only; it does not prove application, database, or public-route health.
6. Before any mutation, record the exact target, current version/configuration, backup or rollback path, and validation plan. Change one layer at a time.
7. After a mutation, verify the process/container, local application health, PostgreSQL connectivity, public Cloudflare path, and relevant logs.

## Safety rules

- Let the user personally complete passwords, passkeys, phone prompts, hardware-key checks, and authenticator codes. Never ask them to paste an OTP into chat.
- Do not print environment files, database connection strings, tokens, cookies, SSH keys, or unredacted process configuration.
- Preserve unrelated dirty changes and runtime data. Do not run broad Git, Docker, database, or filesystem cleanup commands.
- Require explicit user authority for deletion, irreversible migration, credential rotation, traffic switching, DNS changes, or a production release not already requested.
- Report verified-current facts separately from stable topology and assumptions. Include the failing layer, mutations, validation evidence, rollback status, and remaining uncertainty.
