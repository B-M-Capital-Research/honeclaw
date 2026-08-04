# Runbook: Stripe-only Billing

- status: `live_promotion_in_progress`
- last_updated: `2026-08-04`
- owner: `Codex`

## Purpose

Operate HONE's international membership as one Stripe-backed billing path.
`billing_entitlements` is the application-access truth source, while Stripe is
the only external subscription authority. A redirect, query parameter, email
login, or frontend state never grants paid access.

## Runtime Shape

- Activation and Checkout: `https://hone-claw.com/activate`
- Account and Portal entry: `https://hone-claw.com/me`
- Checkout API: `POST /api/public/billing/checkout/stripe`
- Portal API: `POST /api/public/billing/portal/stripe`
- Billing status: `GET /api/public/billing/status`
- Billing entitlements: `GET /api/public/billing/entitlements`
- Stripe webhook: `POST /api/public/integrations/stripe/webhook`

The public client obtains Product and Price only from the server. Checkout
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
HONE_STRIPE_PRICE_ID=price_1U0Eo6EK7h1dD4JHDrhlnPw8
HONE_STRIPE_PUBLIC_BASE_URL=https://hone-claw.com/
HONE_BILLING_GRACE_DAYS=7
```

Mode and key prefixes must agree. A test runtime rejects live keys, and a live
runtime rejects test keys. Product and Price IDs are configured server-side;
the browser must never select or override them.

### Environment boundaries

| Runtime | Destination | Signing secret | Mode |
|---|---|---|---|
| Local development | `stripe listen` forwarding to loopback | Temporary listener secret | `test` |
| Deployed sandbox | Registered sandbox HTTPS destination | That sandbox destination's secret | `test` |
| Production | Registered `https://hone-claw.com/api/public/integrations/stripe/webhook` destination | Live destination secret | `live` |

Never reuse a listener secret for a registered destination or a test secret in
production. `HONE_STRIPE_PUBLIC_BASE_URL` controls Checkout and Portal return
URLs only; it does not configure webhook delivery.

## Live Stripe Authority

- Account: `acct_1U0D6UEK7h1dD4JH`
- Product: `prod_V0FIIUS22IGljn`
- Annual Price: `price_1U0Eo6EK7h1dD4JHDrhlnPw8`
- Price: USD 199.99 yearly, quantity one, no trial
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
- `invoice.paid`
- `invoice.payment_failed`
- `customer.subscription.created`
- `customer.subscription.updated`
- `customer.subscription.deleted`

The handler verifies `Stripe-Signature` over the untouched body with a
five-minute tolerance, requires the configured mode and exact catalog, stores
a payload digest plus minimal normalized fields, and queues idempotent
projection. A subscription-status event alone cannot grant first access;
first access requires a paid signal. Failed renewal grants only a bounded
grace period to an account that previously paid.

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
separate domestic-invite entitlement identity. The forward migration:

1. deletes entitlement and webhook rows belonging to retired providers;
2. replaces provider constraints with the Stripe-only contract;
3. preserves every existing Stripe entitlement and inbox row;
4. is idempotent and guarded by the normal startup migration lock.

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
through the durable inbox and proves pending/paid/failure/grace/recovery/
cancel/delete/repurchase transitions, replay safety, catalog filtering, and
paid-route `402` behavior without an external account.

The account-dependent lifecycle remains opt-in:

```bash
HONE_RUN_STRIPE_LIFECYCLE=1 \
bash tests/regression/manual/test_stripe_billing_lifecycle.sh
```

It accepts only a protected test key, creates disposable test objects, drives
the real Checkout/Portal/Test Clock lifecycle, and deletes or archives every
disposable object after acceptance.

## Production Deployment

1. Confirm the intended Git revision and build an immutable runtime package.
2. Check active chat runs twice with a quiet interval; restart only after both
   checks report zero.
3. Back up the owner-only runtime environment file without printing it.
4. Install the live mode, restricted API key, registered webhook secret, live
   Product/Price IDs, public base URL, and seven-day grace period as one change.
5. Remove retired provider and provider-selection variables from the runtime.
6. Keep the environment file `root:root 0600`; validate names/prefixes only,
   never values.
7. Deploy the exact package, restart `hone-web.service`, and verify the reported
   revision, health dependencies, ports, and database migration.
8. Verify an invalid public webhook signature returns `401` with zero database
   mutation.
9. Verify `/plan`, `/activate`, and `/me`; create a live Checkout Session but do
   not submit a real payment solely for technical smoke testing.
10. Retain redacted screenshots and event/status evidence outside Git.

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
- Define and automate refund/dispute handling after explicit owner approval.
- Re-evaluate Stripe Tax before broadening sales jurisdictions.
