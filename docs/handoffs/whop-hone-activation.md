# Whop → HONE International Activation Handoff

- title: Whop → HONE 国际邮箱激活与付费权益
- status: done
- created_at: 2026-07-26
- updated_at: 2026-07-28
- owner: Codex
- related_files:
  - `memory/src/web_auth.rs`
  - `crates/hone-web-api/src/email_verification.rs`
  - `crates/hone-web-api/src/routes/public.rs`
  - `crates/hone-web-api/src/routes/whop.rs`
  - `packages/app/src/pages/public-whop-activate.tsx`
  - `packages/app/src/pages/public-me.tsx`
- related_docs:
  - `docs/archive/plans/whop-hone-activation.md`
  - `docs/decisions.md`
  - `docs/invariants.md`
  - `docs/repo-map.md`
  - `docs/runbooks/whop-hone-activation.md`
- related_prs: implementation `4632dfa9`; current production runtime `f5663107`

## Summary

HONE now separates payment membership, registration policy, and login identity. Mainland
users keep the existing invited phone/SMS path. International Whop buyers receive a HONE
account from a verified membership webhook and use the purchase email plus a HONE-owned
email challenge; they do not need a phone number or a Whop login.

The email sender is intentionally an unconfigured interface in this phase. Its API fails
closed with `503` until a transactional email implementation is injected. The code is now
included in production, but no production Whop webhook secret or email provider is configured,
so international entitlement activation remains unavailable.

## What Changed

- Added local and cloud-compatible external identity state alongside existing Web users:
  registration policy, normalized email, email verification state, exact Whop membership,
  renewal state, and last processed event.
- Added signed Standard Webhooks ingestion for `membership.activated`,
  `membership.deactivated`, and `membership.cancel_at_period_end_changed`.
- Enforced raw-body HMAC verification, five-minute timestamp tolerance, event ID agreement,
  Whop API `v1`, and exact business/product/plan matching.
- Added event idempotency, stale-event rejection, and repurchase protection so an old
  membership deactivation cannot revoke a newer purchase.
- Added HONE email challenge send/login endpoints, rate limits, challenge expiry and attempt
  limits, digest-only storage, terms acceptance, and ordinary HttpOnly session creation.
- Kept `/auth/me` available to an authenticated but inactive buyer while paid APIs fail with
  `402`; API-key access follows the same membership check.
- Added `/activate/whop`, linked it from login and pricing, and changed `/me` to render the
  server-owned membership state, masked purchase email, renewal period, and Whop management
  URL.
- Updated terms/privacy to `v2.2` for channel-specific identity, Whop membership data, and
  email/SMS providers. Discord role fulfillment remains owned by Whop's native Discord app.

## Verification

- `cargo test -p hone-memory web_auth`: 26 passed.
- `cargo test -p hone-web-api whop`: 2 passed.
- `cargo test -p hone-web-api email_verification`: 1 passed.
- `cargo check --workspace --all-targets --exclude hone-desktop --exclude hone-user-app`:
  passed; one pre-existing unused-function warning remains.
- `cargo test --workspace --all-targets --exclude hone-desktop --exclude hone-user-app`:
  passed with no failures.
- `bun run test:web`: 292 passed.
- `bun run typecheck:web`: passed.
- `bun run build:web:public`: passed with the existing chunk-size warning.
- `bash tests/regression/run_ci.sh`: passed.
- `workers/public-community-edge`: frozen dependency install, typecheck, and 45 tests passed.
- Explicit `rustfmt --check` on every changed Rust file and `git diff --check`: passed.
- Isolated runtime acceptance used the current binary on `18077`/`18088` and public Vite on
  `13001`, leaving the existing local stack untouched.
- A correctly signed activation webhook returned `created`; replay returned `duplicate`;
  a modified raw body with the original signature returned `401`; a signed deactivation
  returned `updated`.
- Browser acceptance passed on the default desktop viewport and `390x844` mobile viewport:
  `/activate/whop`, the unconfigured sender error state, domestic/Whop cross-links,
  `/plan`, `/terms`, `/privacy`, logged-out `/me`, active Whop `/me`, and canceled Whop
  `/me`. Active state showed masked email, renewal period, management URL, and paid actions;
  canceled state retained the account and exposed only renewal/logout actions.
- With the canceled session, `/api/public/auth/me` returned `200` while the paid
  `/api/public/history` route returned `402`. Browser console warning/error count was zero.
- Real email delivery was not tested because the requested sender implementation is
  deliberately absent.

## 2026-07-28 Production Package Inclusion

- Implementation commit `4632dfa9` is included in exact production runtime `f5663107`.
- The final immutable package is `target/deploy-f5663107`; all `502` payload hashes match
  manifest SHA-256 `b908de852668a47ea350e8f00dfb8ef09c47e7dcfa494a68a24c4994d32428bd`.
- Local, origin, and public `/activate/whop` return `200`; anonymous auth remains JSON `401`.
- `HONE_WHOP_WEBHOOK_SECRET` and a transactional email sender are absent. Unsigned local,
  origin, and public webhook probes return intentional JSON `503`, so the deployed code cannot
  create production entitlement until configuration and live acceptance are completed.
- The surrounding runtime is healthy: version `0.15.3`, PostgreSQL/R2 authoritative, zero
  local durable dependencies, ports `8077/8088`, established Feishu connectivity, and zero
  active chats.

## Risks / Follow-ups

- Implement and inject a transactional `EmailVerificationSender`, then validate real delivery,
  spam placement, retry behavior, and the full purchase-email login.
- Configure the production endpoint and `HONE_WHOP_WEBHOOK_SECRET` using a Whop/company
  credential with webhook-management permission; never commit the secret.
- Run a non-owner live purchase → webhook → email login → Discord connection → VIP role
  acceptance, followed by cancel, expiry, repurchase, and role-removal checks.
- Add refund/dispute coverage and periodic reconciliation before relying on webhook delivery as
  the only long-term membership repair mechanism.
- The current cloud-compatible implementation preserves external state inside the existing
  invite record. At higher write volume, move it to a dedicated Postgres table with explicit
  concurrency control.

## Next Entry Point

Start with `docs/runbooks/whop-hone-activation.md`. Implement a concrete sender behind
`crates/hone-web-api/src/email_verification.rs`, inject it where `AppState` is constructed,
then execute the runbook's production configuration and live buyer acceptance matrix.
