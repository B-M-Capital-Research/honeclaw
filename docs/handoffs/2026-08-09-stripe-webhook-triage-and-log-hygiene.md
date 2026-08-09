# Handoff: Stripe webhook alert triage, origin access logging, FMP calendar coverage

- title: Stripe webhook alert triage, origin access logging, FMP calendar coverage
- status: done
- created_at: 2026-08-09
- updated_at: 2026-08-09
- owner: Chet Zhang
- related_files:
  - `crates/hone-web-api/src/routes/public_finance_calendar.rs`
  - `agents/function_calling/src/lib.rs`
  - `/etc/caddy/Caddyfile` (managed host, not in repo)
- related_docs:
  - `docs/runbooks/backend-deployment.md`
  - `docs/runbooks/stripe-billing.md`
- related_prs:

## Summary

A Stripe "webhook endpoint failing" notice for
`https://hone-claw.com/api/public/integrations/stripe/webhook` turned out not to
be a backend fault. Triage produced three unrelated findings that were worth
fixing: the origin had no per-request log to settle delivery disputes, the
finance calendar reported FMP subscription rejections as JSON parse failures,
and the function-calling agent logged tool-budget probes as rejections.

## What Changed

### Managed host (no repo change)

- `/etc/caddy/Caddyfile` now writes a per-request JSON access log to
  `/var/log/caddy/origin-access.log` (20MiB x 5 roll, 14-day retention).
  Previously Caddy logged only warnings and errors, so nothing recorded whether
  a given upstream request ever reached the origin.
- Both the default logger and the access log strip `X-Hone-Origin-Token`,
  `Cookie`, `Authorization` and `Stripe-Signature`. The origin token had been
  written verbatim into every 5xx entry in the journal.
- Backup of the previous file: `/etc/caddy/Caddyfile.bak-20260809`.
- Pruned superseded artifacts under `/opt/hone/releases` and `/opt/hone/builds`
  following the retention rule in `docs/runbooks/backend-deployment.md`; kept
  current (`d379cccc`), previous (`beaf05c3`) and one rollback (`c99babc1`).
  Root filesystem went from 88% to 78% used (3.6G -> 6.3G free).

### `crates/hone-web-api/src/routes/public_finance_calendar.rs`

- `fetch_fmp_json_once` checks the HTTP status before parsing the body. FMP
  answers plan rejections with plain-text HTTP 402, so every one of them was
  surfacing as `FMP JSON 解析失败: expected value at line 1 column 1`.
- `normalize_calendar_symbol` rewrites a US share-class dot to FMP's dash form
  (`BRK.B` -> `BRK-B`) while leaving exchange suffixes (`0700.HK`, `688167.SH`)
  and single-letter exchange suffixes (`SHEL.L`) untouched.
- Symbols rejected as out-of-plan are remembered for 6 hours and skipped, logged
  at debug instead of warn, and reported to the user as one consolidated
  "not covered by the current FMP subscription" line rather than one error per
  symbol per calendar build.

### `agents/function_calling/src/lib.rs`

- `tool_budget_error` no longer logs. Two of its three call sites use it as a
  feasibility probe (`.is_none()` / `.is_some()`), so it was emitting "tool call
  rejected" warnings for calls that were never rejected. The site that actually
  rejects logs once, at debug, because reaching a configured budget is the
  designed stop rather than a fault.
- `MAX_MARKET_MOVE_FINAL_CORRECTIONS` raised from 1 to 2. Production drafts kept
  failing the same "cite target date and source URL in the cause paragraph" rule
  on the single correction round and degraded to the deterministic gap answer.

## Verification

- `cargo check --workspace --all-targets --exclude hone-desktop --exclude hone-user-app`
- `cargo test --workspace --all-targets --exclude hone-desktop --exclude hone-user-app`
  — 2456 passed, 0 failed
- `bash tests/regression/run_ci.sh` — exit 0
- Live FMP probe from the managed host confirmed the split cause:
  `BRK-B` -> HTTP 200 with earnings data, `BRK.B` -> HTTP 402,
  `0700.HK` and `00700.HK` -> HTTP 402, ADR `TCEHY` -> HTTP 200.
- Webhook path proven healthy end to end through Cloudflare: a stale timestamp
  returns `401 {"error":"Stripe webhook timestamp 已过期"}` and a fresh timestamp
  with a bad signature returns `401 {"error":"Stripe webhook 签名无效"}`, both
  carrying `via: 1.1 Caddy` and `x-hone-origin-region: gce-us-central1`.
- Caddy reloaded through the admin API with an unchanged MainPID; no restart and
  no dropped connections.

## Risks / Follow-ups

- **Stripe test-mode destination still registered.** The failing endpoint belongs
  to a *test-mode* destination pointing at the production URL, while the host
  runs `HONE_STRIPE_MODE=live`. `billing_webhook_events` and
  `billing_entitlements` are both empty, so nothing was lost. Delete or repoint
  that destination in the Stripe test dashboard; Stripe stops retrying it on
  2026-08-15. The live destination `we_1U0c0XEK7h1dD4JHrvQ9CRaH` is unaffected.
- **Origin token rotation.** Historical journal entries still contain the
  plaintext `X-Hone-Origin-Token`. The filter only protects entries written from
  now on.
- **HK/SH earnings coverage needs a decision**, not a code fix: the FMP plan does
  not cover `.HK` / `.SH` listings on `/stable/earnings`. Options are upgrading
  the plan, mapping those holdings to their US ADR, or accepting the gap (which
  the calendar now states explicitly).
- **Disk pressure is structural.** Each GHCR runtime release is ~1.3G against a
  30G root disk, and `journald` has no `SystemMaxUse` cap (currently 1.1G,
  default ceiling is 10% of the filesystem). A full disk previously produced a
  4-minute crash loop on 2026-08-05 01:31 UTC, when `hone-cli` could not write
  its effective config and systemd restarted it every 5s. Consider capping the
  journal and adding a free-space alert.
- **The market-move retry bump is a tuning change**, not a guarantee. Do not
  "fix" the remaining failures by mechanically stapling an eligible URL onto a
  cause paragraph: that would manufacture exactly the evidence link the check
  exists to verify. The deterministic gap answer is the correct degradation.

## Next Entry Point

`/var/log/caddy/origin-access.log` now carries `Cf-Ray` per request, so a future
Cloudflare or Stripe delivery dispute can be reconciled by ray ID instead of
inferred from process uptime.
