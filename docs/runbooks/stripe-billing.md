# Runbook: Stripe Billing

- status: `in_progress`
- last_updated: `2026-08-06`
- owner: `Codex`

## Purpose

Operate HONE's international membership as two server-owned Stripe offers:
USD 199.99/year auto-renewing subscription and USD 229.99/12 months one-time
annual pass. The one-time pass can use card, Alipay, or WeChat Pay and never
auto-renews.
`billing_entitlements` is the application-access truth source, while Stripe is
the only external payment authority. A redirect, query parameter, email
login, or frontend state never grants paid access.

## Runtime Shape

- Activation and Checkout: `https://hone-claw.com/activate`
- Account and Portal entry: `https://hone-claw.com/me`
- Checkout API: `POST /api/public/billing/checkout/stripe`
- Portal API: `POST /api/public/billing/portal/stripe`
- Billing status: `GET /api/public/billing/status`
- Billing entitlements: `GET /api/public/billing/entitlements`
- Stripe webhook: `POST /api/public/integrations/stripe/webhook`

The public client chooses only `subscription` or `fixed_term`; it never submits
an amount, currency, duration, Product ID, Price ID, or payment object ID. The
server creates subscription Checkout with `mode=subscription` and one-time
Checkout with `mode=payment`. Checkout
success redirects to `/me`, but access changes only after a verified paid
webhook is projected into the ledger. HONE iOS remains restore-only and does
not display an external purchase action.

## Configuration

Use [`.env.example`](../../.env.example) as the variable checklist. Store real
credentials only in the ignored local environment or the deployment secret
store. Never paste secret keys, webhook secrets, customer emails, verification
codes, or raw webhook bodies into source, chat, screenshots, tickets, shell
history, or logs.

```text
HONE_STRIPE_CHECKOUT_ENABLED=true
HONE_STRIPE_MODE=live
HONE_STRIPE_SECRET_KEY=<rk_live_... or sk_live_...>
HONE_STRIPE_WEBHOOK_SECRET=<whsec_... from the registered live destination>
HONE_STRIPE_PRODUCT_ID=prod_V0FIIUS22IGljn
HONE_STRIPE_SUBSCRIPTION_PRICE_ID=price_1U0Eo6EK7h1dD4JHDrhlnPw8
HONE_STRIPE_FIXED_TERM_PRICE_ID=price_1U1M0rEK7h1dD4JHbKBpIkZ2
HONE_STRIPE_PUBLIC_BASE_URL=https://hone-claw.com/
HONE_BILLING_GRACE_DAYS=7
```

Mode and key prefixes must agree. A test runtime rejects live keys, and a live
runtime rejects test keys. Both Price IDs are configured server-side and must
be distinct. The subscription Price is recurring yearly at USD 199.99; the
fixed-term Price is one-time at USD 229.99. The browser must never select or
override catalog values.

### Environment boundaries

| Runtime | Destination | Signing secret | Mode |
|---|---|---|---|
| Local development | `stripe listen` forwarding to loopback | Temporary listener secret | `test` |
| Deployed sandbox | Registered sandbox HTTPS destination | That sandbox destination's secret | `test` |
| Production | Registered `https://hone-claw.com/api/public/integrations/stripe/webhook` destination | Live destination secret | `live` |

Never reuse a listener secret for a registered destination or a test secret in
production. `HONE_STRIPE_PUBLIC_BASE_URL` controls Checkout and Portal return
URLs only; it does not configure webhook delivery.

### Verification email runtime

`/activate` verifies the HONE account email before creating Checkout. Production
therefore also requires the complete Cloudflare Email Sending runtime set:

```text
HONE_CLOUDFLARE_ACCOUNT_ID=<account id>
HONE_CLOUDFLARE_EMAIL_API_TOKEN=<account token with Email Sending Write only>
HONE_EMAIL_FROM=verify@hone-claw.com
```

These values belong in the production secret manager and the owner-only
`/etc/hone/runtime.env`, not only in a developer's ignored `.env`. All three
missing intentionally leaves the endpoint fail-closed with `503`; a partial or
malformed set is a startup error. After a deployment or host migration, require
the non-secret startup message `Cloudflare 邮箱验证码服务已装配`, request one real
code, receive it in the intended inbox, and use that same challenge to create
Checkout. Never print the token, code, or recipient address during diagnosis.

## Live Stripe Authority

- Account: `acct_1U0D6UEK7h1dD4JH`
- Product: `prod_V0FIIUS22IGljn`
- Subscription Price: `price_1U0Eo6EK7h1dD4JHDrhlnPw8`
- Subscription: USD 199.99 yearly, quantity one, no trial, auto-renewing
- Fixed-term Price: `price_1U1M0rEK7h1dD4JHbKBpIkZ2`
- Fixed-term pass: USD 229.99 once, 12 calendar months, no renewal
- Live wallet configuration: Alipay and WeChat Pay both have
  `display_preference=on`; Stripe currently reports `available=false` and the
  Dashboard shows `pending approval`, so live Checkout must not be described as
  wallet-ready until Stripe changes both methods to available
- Webhook destination: `we_1U0c0XEK7h1dD4JHrvQ9CRaH`
- Webhook name: `HONE production billing`
- API version: `2026-07-29.dahlia`

Production promotion requires `charges_enabled=true`,
`payouts_enabled=true`, and empty current/past-due requirements. The Customer
Portal must allow payment-method updates and period-end cancellation. Product
switching, quantity changes, and customer-information edits stay disabled
until HONE has an explicit migration policy.

The production API key should be a permanent restricted live key named
`HONE production billing` with only these write permissions:

- Checkout Sessions (v1)
- Customer Portal

Do not use the expiring Stripe CLI key as a production runtime credential.

## Webhook Contract

Subscribe the live destination to exactly:

- `checkout.session.completed`
- `checkout.session.async_payment_succeeded`
- `checkout.session.async_payment_failed`
- `checkout.session.expired`
- `invoice.paid`
- `invoice.payment_failed`
- `customer.subscription.created`
- `customer.subscription.updated`
- `customer.subscription.deleted`
- `charge.refunded`

The handler verifies `Stripe-Signature` over the untouched body with a
five-minute tolerance, requires the configured mode and exact catalog, stores
a payload digest plus minimal normalized fields, and queues idempotent
projection. A subscription-status event alone cannot grant first access;
first access requires a paid signal. Failed renewal grants only a bounded
grace period to an account that previously paid.

For `fixed_term` Checkout, a paid completion or async success creates exactly
one entitlement keyed by the PaymentIntent and ending 12 calendar months after
the provider event time. A completed-but-unpaid, failed, or expired Session
never grants access. A full `charge.refunded` event revokes only the fixed-term
entitlement for that PaymentIntent; a partial refund does not mutate access.
Recurring subscriptions continue to use Invoice and Subscription authority and
are never revoked by a fixed-term refund.

`checkout.session.completed` is provisional and orders from the Checkout
Session creation time. Stripe can emit authoritative invoice/subscription
events immediately before the completion envelope. The inbox retains the
actual webhook-envelope timestamp for audit, while authoritative paid,
failure, status, and inactive transitions use their own provider event times.

## Inbox Processing And Recovery

The endpoint acknowledges only after the normalized event is durably inserted
into `billing_webhook_events`. Projection is asynchronous. Events are claimed
with a five-minute lease and may be retried at most ten durable attempts. The
runtime scans recoverable `received`, `failed`, and expired-lease events every
30 seconds; the request-triggered worker also performs three short attempts.

Completion is fenced by the claimed `attempt_count`, so an expired worker
cannot overwrite a newer claim. Entitlement upserts reject older provider
timestamps. Retry, replay, and out-of-order delivery therefore cannot create a
second access truth or roll a newer subscription state backward.

## Stripe-only Data Migration

Fresh SQLite and PostgreSQL schemas accept only Stripe billing rows, plus the
separate domestic-invite entitlement identity. Entitlements carry an explicit
`entitlement_kind` and a generic `provider_reference_id`: Stripe subscription
IDs identify recurring rows, and PaymentIntent IDs identify fixed-term rows.
The forward migration:

1. deletes entitlement and webhook rows belonging to retired providers;
2. renames the old subscription-shaped reference to
   `provider_reference_id` and backfills existing Stripe rows as
   `recurring_subscription`;
3. replaces provider/kind constraints with the typed Stripe-only contract;
4. preserves every existing Stripe entitlement and inbox row;
5. is idempotent and guarded by the normal startup migration lock.

The 2026-08-04 production inventory found zero retired-provider entitlement
rows and zero retired-provider webhook rows, so the live migration has no paid
member data to translate.

## Automated Verification

```bash
bash scripts/ci/check_fmt_changed.sh
cargo check --workspace --all-targets --exclude hone-desktop --exclude hone-user-app
cargo test --workspace --all-targets --exclude hone-desktop --exclude hone-user-app
bun run typecheck:web
bun run test:web
cd workers/public-community-edge && bun run typecheck && bun run test
bash tests/regression/run_ci.sh
```

For focused billing checks:

```bash
cargo test -p hone-memory billing::tests
cargo test -p hone-web-api routes::stripe::tests
bash tests/regression/ci/test_billing_contract.sh
bash tests/regression/ci/test_billing_http_e2e.sh
```

`test_billing_http_e2e.sh` starts an isolated real backend with temporary
SQLite and obviously fake test credentials. It sends signed raw Stripe events
through the durable inbox and proves subscription failure/grace/recovery/
cancel/delete/repurchase plus fixed-term pending/paid/replay/refund/isolation,
catalog filtering, and paid-route `402` behavior without an external account.

The account-dependent lifecycle remains opt-in:

```bash
HONE_RUN_STRIPE_LIFECYCLE=1 \
bash tests/regression/manual/test_stripe_billing_lifecycle.sh
```

It accepts only a protected test key, creates disposable test objects, creates
both Checkout modes, drives the recurring Portal/Test Clock lifecycle, and
deletes or archives every disposable object after acceptance. Alipay and WeChat
Pay require separate official test Checkout browser acceptance because their
hosted confirmation surfaces cannot be proven by a fabricated webhook.

## Production Deployment

1. Confirm the intended Git revision, wait for its fixed Debian `linux/amd64`
   Runtime Image workflow, and record the immutable GHCR manifest digest. Stage
   that exact digest with `scripts/stage_ghcr_runtime.sh`; do not compile on GCE
   or deploy from the mutable `main` image tag.
2. Check active chat runs twice with a quiet interval; restart only after both
   checks report zero.
3. Back up the owner-only runtime environment file without printing it.
4. Install the live mode, restricted API key, registered webhook secret, live
   Product plus both Price IDs, public base URL, and seven-day grace period as
   one change.
5. Remove retired provider and provider-selection variables from the runtime.
6. Keep the environment file `root:root 0600`; validate names/prefixes only,
   never values.
7. Deploy the exact package, restart `hone-web.service`, and verify the reported
   revision, health dependencies, ports, and database migration.
8. Verify an invalid public webhook signature returns `401` with zero database
   mutation.
9. Verify `/plan`, `/activate`, and `/me`; create one live Checkout Session for
   each offer and confirm USD 199.99 recurring versus USD 229.99 one-time plus
   the enabled wallet methods. Do not submit a real payment solely for
   technical smoke testing.
10. Retain redacted screenshots and event/status evidence outside Git.

## Accepted Production State (2026-08-04)

- Source revision:
  `edddfc5b890d124d76d8c6eddc9aa85f2e94b807`
- GHCR digest:
  `sha256:0dcd14a825a124344908b34f6cab19f83eca1f614a40eb2bdf08df2f093f0eee`
- Release:
  `/opt/hone/releases/edddfc5b890d124d76d8c6eddc9aa85f2e94b807-ghcr-runtime`
- Runtime Image workflow run: `30893733765`
- `/api/meta`: exact revision, `ghcr_linux_oci`, healthy authoritative
  PostgreSQL/S3, zero local durable dependency
- Service: `hone-web.service` active, `NRestarts=0`, ports `8077/8088`
- Database: Stripe-only constraints installed; entitlement and webhook tables
  both empty at cutover
- Security: invalid public webhook `401` with no mutation; unauthenticated
  Checkout `401`; protected runtime environment `root:root 0600`
- Email: real inbox receipt and same-challenge verification passed after the
  Cloudflare runtime set was moved into the formal production environment
- Checkout: official Stripe live Session and page showed USD 199.99/year,
  `open` and `unpaid`; no payment was submitted and `/me` remained inactive
- Retired channel: Whop product/plan hidden, public purchase CTA absent, HONE
  webhook deleted
- Redacted evidence: `20-plan-live.png` through
  `24-whop-product-plan-hidden.png` in the plan's acceptance directory

## Accepted Fixed-term Deployment (2026-08-06)

- Implementation revision:
  `c99babc1e1ea3c54db41256331eb65dcefa7bd1d`
- Runtime Image workflow run: `31082512757`, job `92554342413`
- Immutable GHCR digest:
  `sha256:dadf8fcf340cf8fa4971605c3f085f7e097efc7cc2c9a8e1ff4a61d757ca90cb`
- Release:
  `/opt/hone/releases/c99babc1e1ea3c54db41256331eb65dcefa7bd1d-ghcr-runtime`
- Binary SHA-256:
  `14b4aaa79484f598223aabf8ebd144afcae819c02670af49ea7437b26b4068f2`
- Protected runtime backup:
  `/etc/hone/runtime.env.pre-stripe-fixed-term-20260806T160000Z`
- `/api/meta`: exact source revision, build timestamp
  `2026-08-06T07:51:28Z`, `ghcr_linux_oci`, healthy authoritative
  PostgreSQL/S3, and zero local durable dependency
- Service: `hone-web.service` active, `NRestarts=0`, ports `8077/8088`;
  protected runtime environment remains `root:root 0600`
- Public catalog: recurring USD 199.99/12 months/auto-renewing and fixed USD
  229.99/12 months/non-renewing both report `enabled=true`
- Security boundary: invalid webhook signature returned `401`; unauthenticated
  fixed-term Checkout creation returned `401`
- Browser acceptance: production `/activate` showed both offers; authenticated
  hosted live Checkout showed USD 229.99 with no recurring marker. The payment
  form exposed card only, and no live payment was submitted.
- Wallet gate: the card-only live form matches Stripe's current authority.
  Alipay and WeChat Pay remain `display_preference=on`, `available=false`, and
  `pending approval`. Test mode already completed real hosted test-payment
  lifecycles for both methods; production must not be called wallet-ready until
  the live API reports both available and hosted Checkout renders them.
- Redacted evidence outside Git:
  `25-test-checkout-alipay-wechat.png`, `27-live-dual-offer-activate.png`,
  `28-live-fixed-checkout-summary.png`, and
  `29-live-fixed-checkout-payment-methods.png` in the Stripe acceptance
  directory. The live screenshots exclude email, verification codes, payment
  credentials, Checkout identifiers, and secrets.

## Rollback

1. Set `HONE_STRIPE_CHECKOUT_ENABLED=false` and restart the exact prior package
   if code rollback is necessary.
2. Keep the live webhook destination, secret, and ingestion path active so
   existing subscriptions continue to synchronize.
3. Keep `/me` and Customer Portal available to existing Stripe customers.
4. Never delete Stripe products, prices, customers, subscriptions, invoices,
   webhook history, or Stripe ledger rows as a rollback shortcut.
5. Diagnose by event ID and `billing_webhook_events`; never grant access by
   editing frontend state or trusting a success URL.

## Known Follow-ups

- Add provider API reconciliation for events missed beyond webhook retries.
- Define dispute handling and any partial-refund access policy after explicit
  owner approval. Full fixed-term refunds already revoke the matching pass.
- Re-evaluate Stripe Tax before broadening sales jurisdictions.
