# Runbook: GCP Backend Access And Diagnosis

Last updated: 2026-08-01

## When to use

- Accessing the Honeclaw production backend through GCP Identity-Aware Proxy (IAP)
- Diagnosing VM, application, or PostgreSQL health
- Preparing a controlled production configuration, code, migration, restart, or deployment change
- Separating a Cloudflare/public-route failure from a backend-origin failure

## Private topology boundary

Cloudflare serves the public delivery layers. The backend and PostgreSQL are currently co-located on one GCP Compute Engine VM; PostgreSQL is not a separately purchased managed database. Treat that topology as a starting point and verify it live before every operation.

This public repository intentionally omits the Google account identity, GCP project ID, instance name, zone, IP address, OAuth state, SSH key, database URL, and two-step verification data. Resolve private coordinates from the authorized operator's local `gcloud` state. Do not copy values discovered at runtime into commits, issues, PRs, logs, or handoffs.

## Establish authenticated GCP context

Prefer the installed CLI and inspect state before opening a browser:

```bash
command -v gcloud
gcloud auth list --filter=status:ACTIVE --format='value(account)'
gcloud config get-value project
```

If there is no suitable active account or project, stop and let the user run or approve the required `gcloud auth login` and project selection. The user must personally complete passwords, passkeys, phone prompts, hardware-key checks, and authenticator codes. Never retain or repeat the account email, OAuth URL, authorization code, OTP, access token, refresh token, or SSH private key.

Capture the active project only in the current shell, then list the live Compute Engine inventory:

```bash
hone_project="$(gcloud config get-value project 2>/dev/null)"
test -n "$hone_project" && test "$hone_project" != "(unset)"
gcloud compute instances list \
  --project="$hone_project" \
  --format='table(name,zone.basename(),status,labels.list())'
```

Select the backend instance and zone from current evidence. If more than one candidate is plausible, stop and ask the user rather than guessing:

```bash
hone_instance='<reviewed-instance-name>'
hone_zone='<reviewed-zone>'
gcloud compute instances describe "$hone_instance" \
  --project="$hone_project" \
  --zone="$hone_zone" \
  --format='yaml(name,zone,status,labels,machineType,networkInterfaces[].networkIP)'
```

Require the expected instance name, zone, `RUNNING` status, and reviewed labels before connecting.

## Connect through IAP

Use IAP rather than assuming the VM has or should expose a public SSH address:

```bash
gcloud compute ssh "$hone_instance" \
  --project="$hone_project" \
  --zone="$hone_zone" \
  --tunnel-through-iap
```

The remote login may require Google server-side two-step verification. Send a phone prompt only once, wait for the user to approve it, and only then continue. Never request an authenticator code in chat.

For a quick authority check, keep the remote command as one quoted argument:

```bash
gcloud compute ssh "$hone_instance" \
  --project="$hone_project" \
  --zone="$hone_zone" \
  --tunnel-through-iap \
  --command='id; sudo -n true && echo sudo-ok'
```

## Establish a read-only VM baseline

Successful SSH proves access only. Run a bounded baseline before diagnosing or changing the service:

```bash
id
sudo -n true && echo sudo-ok
hostname
uptime
df -h
free -h 2>/dev/null || true
sudo ss -ltnp 2>/dev/null || sudo lsof -nP -iTCP -sTCP:LISTEN
sudo systemctl --failed --no-pager 2>/dev/null || true
sudo systemctl list-units --type=service --all --no-pager 2>/dev/null | rg -i 'hone|postgres|docker|podman' || true
docker ps --format 'table {{.Names}}\t{{.Image}}\t{{.Status}}\t{{.Ports}}' 2>/dev/null || true
```

Discover the application supervisor, working directory, deployed Git revision or image, local health port, and recent logs. Do not assume service names, container names, paths, or ports from a previous incident. Avoid commands such as an unfiltered `docker inspect` or `systemctl show` that can print secret-bearing environment values.

## Verify co-located PostgreSQL

Determine whether PostgreSQL runs through systemd, Docker, or another reviewed supervisor before acting:

```bash
sudo systemctl status postgresql --no-pager 2>/dev/null || true
docker ps --format '{{.Names}}\t{{.Image}}\t{{.Status}}' 2>/dev/null | rg -i 'postgres|timescale' || true
pg_isready 2>/dev/null || true
```

Do not infer database health from the application process alone. Check readiness, disk space, recent database logs, and application connection errors. Inspect secret presence, ownership, permissions, or redacted structure instead of printing a database URL or password.

Before a schema migration or data repair:

1. Identify the actual database process, data volume, database name, and migration tool without exposing credentials.
2. Take an appropriate recoverable backup and verify that the backup can be read.
3. Record the current application revision and rollback command.
4. Drain or stop writes when the migration contract requires it.
5. Apply the smallest reviewed change, then verify PostgreSQL readiness, application health, and the public path.

## Diagnose by boundary

Work from the outside inward and record the first failing boundary:

1. **Cloudflare/public edge:** verify the exact public URL, response status, relevant headers, DNS/route, and current Pages, Worker, or connector deployment state.
2. **GCP control plane:** verify the active project, selected instance, zone, power state, and IAP reachability.
3. **VM:** check resource pressure, disk, listeners, failed services, containers, and recent system events.
4. **Application:** verify supervisor state, deployed revision/image, config source, local health endpoint, and logs.
5. **PostgreSQL:** verify its actual supervisor, readiness, storage, and application connection path.
6. **End to end:** compare local application health from the VM with the public Cloudflare path. A healthy local origin plus a failed public request points outward; an unhealthy local origin points inward.

Use [the backend deployment runbook](backend-deployment.md) for the public frontend, Worker route, cloud-authority, drain, deployment, and rollback checks. Do not mark production healthy until the relevant local and public checks both pass.

## Production mutation contract

- Read the deployed configuration, process definition, current revision, worktree state, and database state before editing.
- Preserve unrelated dirty changes, runtime data, and existing rollback artifacts.
- Record the exact target, current version or image, expected behavior, validation commands, and rollback path before restart or deploy.
- Change one layer at a time. Do not combine code, database, Cloudflare, DNS, credential, and supervisor mutations into an unreviewable action.
- Require explicit user authority for deletion, irreversible migration, credential rotation, traffic switching, DNS changes, or a production release not already requested.
- After a change, verify the process/container, local health, PostgreSQL connectivity, public Cloudflare path, and relevant logs.

## Completion evidence

Report the active project only as verified local context unless the user explicitly requests its value. Record the selected instance and zone in the private operator transcript, not in repository files. Also report:

- remote Unix identity and non-interactive sudo result;
- application supervisor, deployed version, and local health result;
- how PostgreSQL is actually running and its readiness result;
- public Cloudflare result and the first failing boundary;
- each production mutation, validation result, rollback state, and remaining uncertainty.
