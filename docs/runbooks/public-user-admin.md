# Public User Administrator Runbook

## Purpose

Safely inspect, grant, or revoke the PostgreSQL-backed administrator role used by the public `/me → 管理` surface. This role is separate from admin-console bearer tokens and channel administrator configuration.

## Preconditions

- Run from the repository root with the production-equivalent canonical config.
- PostgreSQL environment values are available through the normal ignored `.env` / secret-manager path.
- The target phone already belongs to exactly one active domestic Web invite user.
- Never paste database credentials, API keys, session cookies, or complete query output into tickets or committed files.

## Production Authority Check

For the managed GCE deployment, run the role command on the GCE host with the same
config and environment file as `hone-web.service`. Do not treat a local `.env`, a
forwarded `127.0.0.1` PostgreSQL port, or a successful local read-back as proof that
the production role changed; those may point at a different PostgreSQL instance.

Confirm the active service inputs without printing secret values:

```bash
sudo systemctl show hone-web \
  -p WorkingDirectory -p ExecStart -p EnvironmentFiles --no-pager
sudo awk -F= \
  '/^(HONE_CLOUD_MODE|DATABASE_URL|HONE_POSTGRES_[A-Z0-9_]+)=/ {print $1"=SET"}' \
  /etc/hone/runtime.env | sort
```

Then run dry-run/apply from that host through the installed release binary:

```bash
sudo bash -lc '
  set -a
  source /etc/hone/runtime.env
  set +a
  cd /var/lib/hone
  /opt/hone/current/bin/hone-cli \
    --config /srv/honeclaw/config.yaml \
    cloud web-admin --phone 13800138000 --action grant
'
```

Add `--apply` only after the GCE-hosted dry-run identifies exactly one active target.
After apply, require `verified_is_admin=true`, then refresh the target's authenticated
production `/me` page and confirm both administrator sections render.

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
3. Confirm “HONE 使用统计” and “会员白名单” appear, while the whitelist excludes international email-only users. The usage section defaults open; the whitelist defaults closed; both must expand and collapse.
4. Confirm `/api/public/admin/usage` shows the latest 14 Beijing dates, real Web questions only, scheduled run/success/failure counts, and a generated timestamp. Verify known scheduler triggers are not counted as user questions and every `user_id` beginning with `codex` (case-insensitive) is absent from both question and execution rows.
5. Confirm the usage data stays in one vertically bounded, internally scrollable table with a sticky header; mobile keeps the same table with horizontal scrolling instead of expanding every row into a page-length card list.
6. Confirm two trend charts appear above the table: “每日使用用户数” and “每日提问量”. Each must expose the same 14 consecutive Beijing dates, including zero-filled dates; the first counts distinct users with at least one real question, while the second sums real questions. Push-only users must not enter the user chart. On mobile the two cards stay in one horizontally scrollable row.
7. Confirm “统计日期” lists every one of the 14 report dates newest-first, including dates with no rows. Switch between “最近 14 天”, a date with known activity, and a zero-activity date. The top sentence must change with the selection: the 14-day view compares the latest seven days with the prior seven, while a single date compares with the same date one week earlier. A zero-activity date must remain selected after refresh and show 0 users/questions/pushes plus the empty-table state; a date whose comparison falls outside the returned window must say that comparison data is unavailable.
8. Confirm an ordinary user receives `403` from both `/api/public/admin/usage` and `/api/public/admin/invites`, and sees no management module.
9. Add only a controlled test phone. Confirm the remaining count decreases by one.
10. Repeat the same phone and confirm it returns conflict without consuming another slot.
11. Disable the controlled test user and confirm its existing session becomes unauthorized.
12. Do not production-test the sixth successful creation unless five real additions are already intended that day.

## Rollback

- Revoke the role with the same dry-run-first CLI command.
- The role change does not delete users or alter membership.
- If the feature build must be rolled back, the additive `is_admin` column and `cloud_web_admin_actions` rows may remain; older binaries ignore them and existing admin markings are preserved by ordinary invite upserts.
