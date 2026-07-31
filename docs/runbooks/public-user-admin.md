# Public User Administrator Runbook

## Purpose

Safely inspect, grant, or revoke the PostgreSQL-backed administrator role used by the public `/me → 管理` surface. This role is separate from admin-console bearer tokens and channel administrator configuration.

## Preconditions

- Run from the repository root with the production-equivalent canonical config.
- PostgreSQL environment values are available through the normal ignored `.env` / secret-manager path.
- The target phone already belongs to exactly one active domestic Web invite user.
- Never paste database credentials, API keys, session cookies, or complete query output into tickets or committed files.

## Ensure Schema

```bash
cargo run -p hone-cli -- cloud doctor --ensure-schema
```

Confirm PostgreSQL is healthy before any role change.

## Dry Run

Grant preview:

```bash
cargo run -p hone-cli -- cloud web-admin \
  --phone 13800138000 \
  --action grant
```

Revoke preview:

```bash
cargo run -p hone-cli -- cloud web-admin \
  --phone 13800138000 \
  --action revoke
```

The command masks the phone in output and reports the unique `user_id`, active state, current role, requested role, and whether an apply would change state. A missing, disabled, malformed, or non-unique grant target fails closed.

## Apply And Verify

After checking the dry run:

```bash
cargo run -p hone-cli -- cloud web-admin \
  --phone 13800138000 \
  --action grant \
  --apply
```

The command performs a uniqueness-checked transaction and then reads the role back. Success requires `verified_is_admin=true`. Revocation uses the same command with `--action revoke`; role revocation does not revoke the user's ordinary membership.

## Application Verification

After deploying a build that contains the public-admin API/UI:

1. Sign in as the target domestic user.
2. Open `/me`.
3. Confirm “管理” appears and the list excludes international email-only users.
4. Confirm an ordinary user receives `403` from `/api/public/admin/invites` and sees no management module.
5. Add only a controlled test phone. Confirm the remaining count decreases by one.
6. Repeat the same phone and confirm it returns conflict without consuming another slot.
7. Disable the controlled test user and confirm its existing session becomes unauthorized.
8. Do not production-test the sixth successful creation unless five real additions are already intended that day.

## Rollback

- Revoke the role with the same dry-run-first CLI command.
- The role change does not delete users or alter membership.
- If the feature build must be rolled back, the additive `is_admin` column and `cloud_web_admin_actions` rows may remain; older binaries ignore them and existing admin markings are preserved by ordinary invite upserts.
