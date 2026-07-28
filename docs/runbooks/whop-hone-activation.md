# Runbook: Whop To HONE Email Activation

Last updated: 2026-07-28

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

The single source of truth for this integration's runtime variable names,
required/optional status, and canonical non-secret IDs is
[`.env.example`](../../.env.example). Copy it to the repository-root `.env`
and fill the required secret values. The backend loads `KEY=value` entries
from that file when its working directory is the repository root; do not add
the shell-only `export` prefix.

The canonical Whop IDs are compiled as safe defaults, but keeping them explicit
in `.env` makes the deployed integration auditable from one file. Override them
only for an isolated staging business.

Never put the webhook secret, Whop company API key, buyer email, email code, or
raw webhook body in committed config, screenshots, logs, or this runbook.

## Cloudflare Email Sending

The backend depends on
`hone_web_api::email_verification::EmailVerificationSender`. Production uses
Cloudflare Email Sending through its REST API. The default
`UnconfiguredEmailVerificationSender` still returns `503` when all email
environment variables are absent. A partial configuration is a startup error
rather than a silent fallback.

Cloudflare setup:

1. Confirm the zone uses Cloudflare DNS and the account has Email Sending
   entitlement. The beta may require the Workers Paid plan; do not upgrade a
   plan without an explicit owner decision.
2. In Cloudflare Dashboard → Compute → Email Service → Email Sending, select
   **Onboard Domain** and choose `hone-claw.com`.
3. Review and allow Cloudflare to add the `cf-bounce` MX, SPF TXT, and DKIM TXT
   records. Wait until the sending domain shows enabled/healthy.
4. Create an account-scoped API token with only `Email Sending: Edit` for the
   account that owns the zone. Do not use the Global API Key.
5. Fill the Cloudflare Email Sending section of the repository-root ignored
   `.env`, using [`.env.example`](../../.env.example) as the complete variable
   checklist.

These values are runtime secrets/configuration and do not travel with a Git
push. The deployment owner must inject them through the production secret
manager or supervisor environment. If the token exists only in a developer's
ignored `.env`, transfer it only through an approved secret channel or create
a separate production token with the same minimal `Email Sending: Edit`
scope. Never paste the token into a deployment command, chat, ticket, or
repository file.

The supervisor must start the backend from the repository root so the CLI can
load the reviewed ignored `.env`, or inject the complete set represented by
[`.env.example`](../../.env.example) through the supervisor environment.
When all three Cloudflare variables are missing, the email endpoint remains
fail-closed with `503`; a partial configuration makes startup fail rather than
silently disabling delivery. A configured startup logs
`Cloudflare 邮箱验证码服务已装配` without exposing credential values.

The configured sender address must belong to the onboarded domain. The
application calls:

```text
POST https://api.cloudflare.com/client/v4/accounts/{account_id}/email/sending/send
```

The implementation sends one recipient, a plain-text body, and an HTML body.
It accepts only a successful Cloudflare response that reports an immediate
delivery, queued delivery, or non-empty provider message ID, and rejects
permanent bounces. The message-ID fallback covers the live beta response that
accepted and delivered a message while omitting the delivery arrays. Provider
bodies, buyer emails, codes, and tokens are not copied into application errors
or logs.

Official references:

- <https://developers.cloudflare.com/email-service/get-started/send-emails/>
- <https://developers.cloudflare.com/email-service/api/send-emails/rest-api/>
- <https://developers.cloudflare.com/api/resources/email_sending/methods/send/>

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
cargo check -p hone-web-api --all-targets
bun run test:web
bun run typecheck:web
```

Before production release, use a non-owner buyer and a distinct test email:

1. Complete the canonical Whop purchase without an existing HONE session.
2. Confirm one `membership.activated` delivery returns `2xx`; resend the same
   event and confirm it is idempotent.
3. Open `/activate/whop`, request the code, and confirm Cloudflare Email
   Sending activity reports `delivered` or `queued`. Verify the email in the
   real inbox, enter the code, and land on `/me`.
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
- Email send `503`: all Cloudflare email environment variables are absent.
- Backend startup failure naming email environment variables: the Cloudflare
  configuration is partial or structurally invalid.
- Email send `502` with Cloudflare HTTP/error code: inspect Email Sending
  activity, domain onboarding status, token scope, account ownership, quota,
  and recipient bounce state. Never paste the token into logs or tickets.
- HONE paid route `402`: the stored Whop status does not grant access.

Inspect Whop's webhook event log and the HONE membership projection before any
manual account edit. Do not use a checkout redirect, query parameter, Discord
role, or manually supplied email as an entitlement override.

## Follow-ups

- Add `refund.created`, `dispute.created` / updated handling.
- Add periodic Whop membership reconciliation for missed events.
- Move cloud external identities to a dedicated indexed PG table if measured
  user volume or concurrent-write evidence makes JSON-record scans unsafe.
