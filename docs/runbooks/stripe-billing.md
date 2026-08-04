# Runbook: Unified Stripe + Whop Billing

- status: `test_endpoint_online_verified_checkout_disabled`
- last_updated: `2026-08-04`
- owner: `Codex`

## Purpose

Operate HONE's provider-neutral billing path. Stripe and Whop are payment
adapters; `billing_entitlements` is the only application-access truth source.
The public frontend never grants access from a redirect, query parameter,
email login, or provider-specific field.

## Runtime Shape

- Unified activation: `https://hone-claw.com/activate`
- Existing Whop buyer recovery: `https://hone-claw.com/activate?provider=whop`
- Stripe Checkout: `POST /api/public/billing/checkout/stripe`
- Stripe Portal: `POST /api/public/billing/portal/stripe`
- Billing status: `GET /api/public/billing/status`
- Billing entitlements: `GET /api/public/billing/entitlements`
- Stripe webhook: `POST /api/public/integrations/stripe/webhook`
- Whop webhook: `POST /api/public/integrations/whop/webhook`

The Stripe and Whop webhook endpoints remain enabled when new Checkout is
disabled. This keeps already-paid customers synchronized during a rollout
pause or rollback.

The target business state is new users defaulting to Stripe while Whop remains
available for existing buyers and as a secondary channel. A deployment may
temporarily keep Whop primary with Stripe Checkout disabled while test webhook
delivery is verified; do not mistake that safe stage for the final channel
strategy.

## Configuration

Use [`.env.example`](../../.env.example) as the variable checklist. Store real
values only in the ignored local environment or deployment secret manager.
Never paste secret keys, webhook secrets, buyer emails, verification codes, or
raw webhook bodies into source, chat, screenshots, tickets, or logs.

Start in Stripe test mode:

```text
HONE_BILLING_PRIMARY_PROVIDER=whop
HONE_WHOP_NEW_PURCHASES_ENABLED=true
HONE_STRIPE_CHECKOUT_ENABLED=false
HONE_STRIPE_MODE=test
HONE_STRIPE_SECRET_KEY=<sk_test_...>
HONE_STRIPE_WEBHOOK_SECRET=<whsec_... from the test endpoint/listener>
HONE_STRIPE_PRODUCT_ID=<test prod_...>
HONE_STRIPE_PRICE_ID=<test price_...>
HONE_STRIPE_PUBLIC_BASE_URL=https://hone-claw.com/
HONE_BILLING_GRACE_DAYS=7
```

Mode and secret-key prefixes must agree. A test runtime rejects `sk_live_...`;
a live runtime rejects `sk_test_...`. Product and Price IDs are supplied by
the server, never by the browser.

### Environment-specific webhook configuration

The API path may stay `/api/public/integrations/stripe/webhook`, but the
destination and signing secret are scoped to each environment:

| Runtime | Stripe destination | `HONE_STRIPE_WEBHOOK_SECRET` source | Mode |
|---|---|---|---|
| Local development | `http://127.0.0.1:<port>/api/public/integrations/stripe/webhook` through `stripe listen` | Temporary secret printed by that listener | `test` |
| Deployed sandbox/test | Public HTTPS URL for the deployed test runtime | Signing secret of the registered test endpoint | `test` |
| Production | Public HTTPS URL for the production runtime | Signing secret of the registered live endpoint | `live` |

Never reuse a local listener secret for a registered endpoint, or reuse a test
endpoint secret in live mode. One HONE process deliberately accepts only one
mode and one endpoint secret; serve test and live concurrently from separate
deployments/processes. `HONE_STRIPE_PUBLIC_BASE_URL` controls Checkout
success/cancel and Portal return URLs only—it does not register or configure a
webhook destination.

## Test Catalog

Create a Stripe **test-mode** catalog separate from the existing live product:

- Product name: `B&M Research Membership — Full Access`
- Price: `US$199.99`
- Interval: yearly
- Trial: none
- Quantity: one
- Stripe Tax: off until the owner approves the tax posture

Do not use the live product or live Price ID for development. Record the test
Product and Price IDs in the protected test environment, not this runbook.

## Customer Portal

In Stripe test mode, configure the Customer Portal before testing the Portal
endpoint. Allow customers to update payment methods and cancel subscriptions;
do not enable product switching until a product-migration policy exists. HONE
creates a short-lived Portal Session on demand and never persists its URL.

## Webhook Endpoint

Subscribe the test endpoint to:

- `checkout.session.completed`
- `checkout.session.async_payment_succeeded`
- `checkout.session.async_payment_failed`
- `invoice.paid`
- `invoice.payment_failed`
- `customer.subscription.created`
- `customer.subscription.updated`
- `customer.subscription.deleted`

The handler verifies `Stripe-Signature` over the untouched body with a
five-minute tolerance, requires the configured test/live mode and exact
Product/Price, stores a payload digest plus minimal normalized fields, and
queues idempotent processing. First access requires a paid signal; an `active`
subscription-status event alone cannot grant a never-paid account. Failed
renewals grant only a bounded grace period to an account that already paid.

`checkout.session.completed` is a provisional pending marker. Stripe can
create `invoice.paid` and `customer.subscription.created` immediately before
the later Checkout-completed event. HONE therefore orders only this
provisional marker from the Checkout Session's `created` time, while the inbox
retains the actual webhook-envelope time for audit. Authoritative paid,
failure, status, and inactive transitions continue to use their own provider
event times.

### Deployed test endpoint acceptance

The deployed test endpoint is considered online only after all of the
following hold without printing or persisting its signing secret:

1. The secret belongs to the registered test endpoint, not `stripe listen`.
2. The runtime environment remains owner-only (`root:root 0600` in the current
   GCE deployment) and mode/key/catalog prefixes agree.
3. An invalid signature returns `401` from the public HTTPS route.
4. A signed wrong-catalog test event returns `2xx` with an ignored reason and
   creates neither an inbox row nor an entitlement.
5. The Workbench delivery response and redacted database counts are retained
   as evidence.

If any signing secret is exposed, rotate it immediately, expire the old value,
install only the replacement, and repeat this acceptance. Never copy the value
into a runbook, handoff, screenshot, shell history, or ticket.

## Inbox Processing And Recovery

Both providers acknowledge only after the verified normalized event is durably
inserted into `billing_webhook_events`; entitlement projection is asynchronous.
An event is claimed with a five-minute processing lease and may be retried at
most ten durable attempts. The runtime scans recoverable `received`, `failed`,
and expired-lease events every 30 seconds, while the request-triggered worker
also performs three short bounded attempts.

Completion is fenced by the claimed `attempt_count`. A worker whose lease has
expired cannot overwrite the inbox state after a newer worker reclaims the
event. Entitlement upserts remain idempotent and reject older provider event
timestamps, so retry and out-of-order delivery do not create a second access
truth.

For local forwarding:

```bash
brew install stripe/stripe-cli/stripe
stripe login

stripe listen \
  --events checkout.session.completed,checkout.session.async_payment_succeeded,checkout.session.async_payment_failed,invoice.paid,invoice.payment_failed,customer.subscription.created,customer.subscription.updated,customer.subscription.deleted \
  --forward-to http://127.0.0.1:8088/api/public/integrations/stripe/webhook
```

`stripe login` uses browser device authorization; the account owner must verify
the displayed pairing and complete any security-key/authenticator challenge.
Do not use the interactive API-key paste flow when browser authorization is
available. Use the listener's temporary `whsec_...` only for that local run,
and do not copy it into a registered endpoint, committed file, or screenshot.
Stripe likewise documents that local verification uses the secret printed by
`stripe listen`, while registered destinations use the unique endpoint secret
from Workbench: [Stripe webhook documentation](https://docs.stripe.com/webhooks).

## Automated Verification

```bash
bash tests/regression/ci/test_billing_contract.sh
bash tests/regression/ci/test_billing_http_e2e.sh
cargo check -p hone-web-api
bun run typecheck:web
bun run test:web
```

`test_billing_http_e2e.sh` starts the actual public backend in a temporary
working directory with isolated SQLite and obviously fake test credentials. It
does not load the repository `.env` or call Stripe/Whop. It sends signed raw
HTTP events through both adapters and proves inbox persistence, async
projection, lifecycle transitions, duplicate-provider policy, and paid-route
`402` behavior.

The account-dependent catalog check is deliberately manual:

```bash
HONE_RUN_STRIPE_SANDBOX=1 \
HONE_STRIPE_PRODUCT_ID=prod_... \
HONE_STRIPE_PRICE_ID=price_... \
HONE_STRIPE_WEBHOOK_URL=http://127.0.0.1:8088/api/public/integrations/stripe/webhook \
bash tests/regression/manual/test_stripe_billing_sandbox.sh
```

Local visual evidence is stored outside the repository so it cannot capture
secrets in Git. The current acceptance set is under
`/Users/bytedance/.codex/visualizations/2026/08/03/019fc5c7-d3a5-7df1-83fc-5f0826ad4519/stripe-billing-acceptance/`
and covers desktop `/plan`, Stripe `/activate`, duplicate Stripe+Whop rows on
`/me`, the HONE-iOS no-purchase/restore-only policy, the paid test Customer
Portal (`11-stripe-test-portal-paid-subscription.png`), and the active HONE
account (`12-hone-account-stripe-active.png`). Production evidence `13`–`17`
covers the registered endpoint `200`, safe-stage public pages, and the Whop
same-route query transition without reload.

The real test payment produced `checkout.session.completed`, `invoice.paid`,
and `customer.subscription.created` through the CLI listener. It exposed the
provisional-ordering edge above; after the fix, the same provider events passed
through a fresh signed endpoint exactly once, produced one active entitlement,
and changed the paid API from `402` to `200`. The registered deployed test
endpoint was subsequently accepted with a signed wrong-catalog event returning
`200 catalog_mismatch` and zero inbox/entitlement mutation.

The unified activation page must read `provider` reactively from the router.
Test both direct load and same-route navigation from `/activate` to
`/activate?provider=whop`; the Whop badge, heading, email label, steps, and
submit action must change without a browser reload.

## Sandbox Acceptance Matrix

Use a non-owner buyer and a distinct test email. Preserve event IDs and safe
status screenshots, but redact email and all secrets.

1. With Checkout disabled, verify `/billing/config` reports it disabled while
   both provider webhook endpoints remain available.
2. Enable test Checkout and set Stripe as primary. Verify a browser displays
   Stripe as primary and Whop only as the optional secondary path.
3. Verify a `HONE-iOS` client displays neither price nor an external purchase
   call to action; login and restore remain available.
4. Verify email control, accept Terms `2.3`, create Checkout, and confirm its
   URL is under `checkout.stripe.com`.
5. Cancel Checkout before payment. Confirm no active entitlement exists and
   paid APIs still return `402`.
6. Pay with Stripe's successful test card. Confirm the success page says it is
   waiting, then `invoice.paid` activates exactly one entitlement.
7. Replay the same event. Confirm no second record or state change.
8. Send an older event after a newer one. Confirm latest state remains.
9. Send a signed event for another Product/Price. Confirm it is ignored and
   grants no access.
10. Open Customer Portal from `/me`, update the test payment method, and return.
11. Exercise a failed renewal. Confirm a paid account enters `grace` with a
    finite deadline; a never-paid account remains `pending`.
12. Restore payment. Confirm a later paid event returns the row to `active`.
13. Cancel at period end. Confirm access remains until Stripe reports the
    subscription ended; deletion inactivates only that Stripe entitlement.
14. Add a valid Whop entitlement. Confirm canceling Stripe does not revoke
    HONE, a duplicate warning appears, and only both channels becoming inactive
    makes paid APIs return `402`.
15. Repurchase. Confirm a new subscription activates and a late event for the
    old subscription cannot revoke the new one.

## Live Promotion

Do not switch to live mode until the sandbox matrix passes and the owner has
approved tax, refund, statement-descriptor, support, and dispute operations.
Promotion requires a new live webhook endpoint/secret and separately reviewed
live Product/Price IDs. Change mode, key, webhook secret, and catalog as one
reviewed deployment; never mix test and live identifiers.

Begin with Stripe Checkout enabled but Whop still primary. After a controlled
live purchase and cancellation pass, set `HONE_BILLING_PRIMARY_PROVIDER=stripe`.
Keep Whop new purchases enabled only for the intended secondary channel.

## Rollback

1. Set `HONE_STRIPE_CHECKOUT_ENABLED=false`.
2. Leave the Stripe webhook secret and endpoint active.
3. Do not delete products, customers, subscriptions, invoices, or ledger rows.
4. Keep `/me` and Portal available to existing Stripe customers.
5. If desired, set the primary provider back to `whop`.
6. Diagnose from the event ID and `billing_webhook_events`; never grant access
   by editing frontend state or trusting a success URL.

## Known Follow-ups

- Add provider API reconciliation for events missed beyond webhook retries.
- Define and automate refund/dispute policy after owner approval.
- Re-evaluate Stripe Tax before broadening sales jurisdictions.
