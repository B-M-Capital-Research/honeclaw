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
- related_prs: implementation `4632dfa9`; email/signature follow-ups
  `92cad045`, `c12e95a6`, and `482c34d5`; current production runtime
  `482c34d54aef4f0d9726acea0b753d751a5973be`

## Summary

HONE now separates payment membership, registration policy, and login identity. Mainland
users keep the existing invited phone/SMS path. International Whop buyers receive a HONE
account from a verified membership webhook and use the purchase email plus a HONE-owned
email challenge; they do not need a phone number or a Whop login.

The follow-up Cloudflare Email Sending implementation and current Whop `ws_...`
signature contract are now configured and deployed in production. Missing or partial
runtime configuration still fails closed, while the configured production path accepts
valid raw-body signatures and sends the bounded HONE-owned email challenge. A real
non-owner buyer full-chain production acceptance remains outstanding.

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
- At the original `4632dfa9` implementation checkpoint, real email delivery was
  not tested because the sender was deliberately absent; the follow-up section
  below records the later Cloudflare delivery and production enablement.

## 2026-07-28 Production Email And Signature Enablement

- Current exact production runtime is
  `482c34d54aef4f0d9726acea0b753d751a5973be`.
- Immutable package `target/deploy-482c34d5` has five runtime binaries and 498
  runtime payloads; every manifest entry matches SHA-256 manifest
  `e09f7716a0a07f5c2e9fbe4195cbdc0de1474afb62a6da77d37e3b5aee91a518`.
- Complete Cloudflare email and Whop webhook configuration is loaded from the
  ignored owner-only runtime environment. Startup logs
  `Cloudflare 邮箱验证码服务已装配`.
- Local and public valid-signature no-side-effect probes return `200 ignored`;
  an altered body or missing headers returns `401`. Unknown-email send remains
  a uniform `200`, and anonymous auth remains JSON `401`.
- The surrounding runtime is healthy: version `0.15.3`, PostgreSQL/R2
  authoritative, zero local durable dependencies, ports `8077/8088`, one live
  Feishu process, and repeated zero active chats.

## Risks / Follow-ups

- Run a non-owner live purchase → webhook → email login → Discord connection → VIP role
  acceptance, followed by cancel, expiry, repurchase, and role-removal checks.
- Rotate the webhook secret because it appeared in the private task conversation;
  keep the replacement only in approved secret storage and the ignored runtime
  environment.
- Add refund/dispute coverage and periodic reconciliation before relying on webhook delivery as
  the only long-term membership repair mechanism.
- The current cloud-compatible implementation preserves external state inside the existing
  invite record. At higher write volume, move it to a dedicated Postgres table with explicit
  concurrency control.

## Next Entry Point

Start with `docs/runbooks/whop-hone-activation.md` and execute the real non-owner
buyer acceptance matrix against the already configured production runtime. Rotate
the webhook secret before or immediately after that acceptance when operationally
convenient.
