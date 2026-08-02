# GCE SMS Runtime Environment Persistence

- title: GCE SMS Runtime Environment Persistence
- status: done
- created_at: 2026-08-02
- updated_at: 2026-08-02
- owner: Codex
- related_files: `scripts/check_backend_runtime_env.sh`, `tests/regression/ci/test_backend_runtime_env_contract.sh`, `docs/runbooks/backend-deployment.md`
- related_docs: `docs/handoffs/2026-08-02-gce-sms-runtime-env-persistence.md`

## Goal

Restore production SMS verification by safely persisting the existing local Aliyun SMS credentials in the GCE systemd runtime environment, and prevent future backend restarts or releases from silently starting without the required credential pair.

## Scope

- Confirm the production failure from provider logs without exposing credentials or verification codes.
- Add a reusable, secret-safe runtime environment validator for all supported Aliyun credential aliases.
- Cover missing, empty, quoted, compatibility-alias, and valid credential cases with a CI-safe regression.
- Install the validator as a persistent `hone-web.service` `ExecStartPre` gate and atomically update `/etc/hone/runtime.env` from the ignored local `.env`.
- Drain active chats, restart only the required services, verify the exact running revision and cloud authority, and perform one explicitly scoped SMS canary.
- Restore OS Login 2FA and remove all temporary credential/configuration files.

## Validation

- `bash tests/regression/ci/test_backend_runtime_env_contract.sh`
- `bash tests/regression/run_ci.sh`
- `git diff --check`
- Remote non-secret key presence, ownership, mode, systemd drop-in, and `ExecStartPre` failure/success probes
- Two zero-active-chat checks before restart; Web/Feishu active afterward
- `/api/meta` exact Git SHA plus PostgreSQL/R2 health and zero local durable dependencies
- One designated phone SMS canary with provider acceptance and no new production warning

## Documentation Sync

- Update `docs/runbooks/backend-deployment.md` with the preflight gate, persistent systemd drop-in, and post-restart SMS acceptance requirements.
- Record the incident, production mutation, rollback, and verification in `docs/handoffs/2026-08-02-gce-sms-runtime-env-persistence.md`.
- On completion, remove this entry from `docs/current-plan.md`, archive this plan under `docs/archive/plans/`, and add it to `docs/archive/index.md`.

## Result

- Production journal at `2026-08-02T14:37:40Z` proved the detached provider call failed because the managed Web service lacked `ALIBABA_CLOUD_ACCESS_KEY_ID`; the public generic acceptance response intentionally hid that failure.
- The existing ignored local credential pair was transferred through a `0600` temporary file, atomically merged into `/etc/hone/runtime.env`, and retained as `root:root 0600` without exposing values.
- `/usr/local/sbin/hone-check-web-env` and the persistent `hone-web.service` `ExecStartPre` drop-in now fail closed on missing, empty, or placeholder SMS credentials before any future start.
- After two zero-active-chat checks, Web restarted on the unchanged exact revision `39ce9ce54f5cbfea26e664459cb70edf3fd97292`; Web and Feishu are active, PostgreSQL/R2 are healthy, and local durable dependency count is zero.
- One public canary for the explicitly involved administrator phone returned the generic accepted response and produced no provider-failure warning in the detached-send window.
- Remote and local temporary credential copies were deleted, OS Login 2FA was restored to `TRUE`, and the temporary gcloud configuration was removed.

## Risks / Open Questions

- Secrets must never appear in Git, tool output, process arguments, logs, or handoff text; transfer only a `0600` temporary file and delete it after atomic installation.
- OS Login 2FA may need a tightly bounded temporary disable for IAP SSH; restore it to `TRUE` before deleting the temporary gcloud configuration.
- A live SMS canary creates one provider send and must target only the explicitly involved administrator phone, never a batch.
- A failed env validation must block future Web starts while leaving the currently running process untouched; installation order therefore updates and validates the env before enabling the systemd gate.
