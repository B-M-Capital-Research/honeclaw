# Runbook: Whop To HONE Email Activation

Last updated: 2026-07-26

Use this runbook to configure and verify the international Whop purchase →
HONE purchase-email verification → application entitlement path. Discord role
fulfillment remains separate in `docs/runbooks/whop-discord-fulfillment.md`.

## Canonical Scope

- Company: `biz_h0UKqlfUJI55Am`
- Product: `prod_9jQsUKaifh6ZA`
- Plan: `plan_ZXfsAisr4UOaw`
- Public webhook: `POST https://hone-claw.com/api/public/integrations/whop/webhook`
- Buyer activation: `https://hone-claw.com/activate/whop`
- Required Whop webhook API version: `v1`
- Events:
  - `membership.activated`
  - `membership.deactivated`
  - `membership.cancel_at_period_end_changed`
- Whop permissions must include `member:basic:read`,
  `member:email:read`, and `webhook_receive:memberships`.

Whop documents the payload and test-event workflow at
<https://docs.whop.com/developer/guides/webhooks> and
<https://docs.whop.com/api-reference/memberships/membership-activated>.

## Runtime Configuration

Set the webhook secret only in the backend runtime environment:

```bash
export HONE_WHOP_WEBHOOK_SECRET='whsec_...'
```

The canonical IDs are compiled as safe defaults. They may be overridden for an
isolated staging business:

```bash
export HONE_WHOP_COMPANY_ID='biz_...'
export HONE_WHOP_PRODUCT_ID='prod_...'
export HONE_WHOP_PLAN_ID='plan_...'
```

Never put the webhook secret, Whop company API key, buyer email, email code, or
raw webhook body in committed config, screenshots, logs, or this runbook.

## Email Sender Boundary

The backend depends on
`hone_web_api::email_verification::EmailVerificationSender`. The default
`UnconfiguredEmailVerificationSender` returns `503` before creating or sending
a challenge. Production is not ready for buyer activation until a
transactional provider implementation is injected into `AppState`.

The provider implementation must:

1. Send only the supplied recipient, eight-digit code, and expiration.
2. Avoid logging message bodies or codes.
3. Return delivery acceptance errors to the caller.
4. Keep provider credentials outside repository files.
5. Preserve the endpoint's generic response for unknown emails.

HONE stores only a SHA-256 challenge digest, request/expiry times, and bounded
attempt count; it does not store plaintext codes.

## Create The Whop Webhook

In Whop Developer → Webhooks:

1. Create a company webhook at the exact public endpoint above.
2. Select API version `v1`.
3. Select the three membership events listed above.
4. Copy the returned `whsec_...` value into the backend secret environment.
5. Restart the backend and send a Whop test event.

The endpoint verifies the Standard Webhooks signature over the untouched raw
body, checks a five-minute delivery window, requires matching header/body event
IDs, and then checks company, product, and plan before writing anything.

## Acceptance Matrix

Automated local proof:

```bash
cargo test -p hone-memory web_auth
cargo test -p hone-web-api whop
cargo test -p hone-web-api email_verification
bun run test:web
bun run typecheck:web
```

Before production release, use a non-owner buyer and a distinct test email:

1. Complete the canonical Whop purchase without an existing HONE session.
2. Confirm one `membership.activated` delivery returns `2xx`; resend the same
   event and confirm it is idempotent.
3. Open `/activate/whop`, request the code, verify the email, and land on `/me`.
4. Confirm `/me` shows the masked purchase email, Whop status, renewal end, and
   Whop manage link.
5. Confirm `/chat`, `/portfolio`, and `/community` work while active.
6. Link Discord through Whop and confirm the native app grants the VIP role.
7. Cancel at period end and confirm HONE keeps current-period access while
   showing the non-renewing state.
8. Let access end or send the controlled deactivation event. Confirm `/me`
   remains visible, paid APIs return `402`, and Whop removes the Discord role.
9. Repurchase. Confirm the newer membership restores HONE access and the old
   membership's late deactivation cannot revoke it.

## Failure Diagnosis

- `401`: invalid/missing signature headers, bad secret, or expired delivery.
- `422`: wrong API version, company, product, plan, or membership payload.
- `409`: an email/membership is already bound to a conflicting HONE or Whop
  identity; do not overwrite it manually before investigation.
- Email send `503`: no email provider is injected.
- HONE paid route `402`: the stored Whop status does not grant access.

Inspect Whop's webhook event log and the HONE membership projection before any
manual account edit. Do not use a checkout redirect, query parameter, Discord
role, or manually supplied email as an entitlement override.

## Follow-ups

- Add `refund.created`, `dispute.created` / updated handling.
- Add periodic Whop membership reconciliation for missed events.
- Move cloud external identities to a dedicated indexed PG table if measured
  user volume or concurrent-write evidence makes JSON-record scans unsafe.
