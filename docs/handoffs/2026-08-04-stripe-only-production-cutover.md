# Stripe-only Production Cutover Handoff

- title: Stripe-only production cutover
- status: `done`
- created_at: `2026-08-04`
- updated_at: `2026-08-04`
- owner: `Codex + owner`
- related_files:
  - `memory/src/{billing,web_auth}.rs`
  - `crates/hone-web-api/src/routes/{billing,stripe,public}.rs`
  - `packages/app/src/pages/{public-activate,public-me,public-plan}.tsx`
  - `.github/workflows/runtime-image.yml`
  - `deploy/runtime/Dockerfile`
  - `scripts/{package,verify,stage}_runtime_bundle.sh`
- related_docs:
  - `docs/archive/plans/stripe-whop-parallel-billing.md`
  - `docs/archive/plans/whop-email-delivery.md`
  - `docs/decisions.md#d-2026-08-04-01-make-stripe-the-only-external-billing-provider`
  - `docs/decisions.md#d-2026-08-04-03-build-managed-linux-runtimes-in-actions-and-deliver-them-through-ghcr`
  - `docs/runbooks/stripe-billing.md`
  - `docs/runbooks/backend-deployment.md`
- related_prs: direct `main` commits `9961652f`, `91e93b51`, and `edddfc5b890d124d76d8c6eddc9aa85f2e94b807`; no PR, release, or tag
- verification: complete repository gates; GitHub Runtime Image run `30893733765`; exact GHCR digest/revision staging; production health, database, auth, email, live Checkout, unpaid entitlement, and Whop retirement browser acceptance
- risks: no live payment was submitted; refund/dispute/Tax/reconciliation remain follow-ups; one open unpaid Checkout Session expires normally; the old Whop company API key was intentionally retained because it is outside the Billing runtime

## Summary

HONE production is Stripe-only. The deployed runtime contains no Whop adapter,
route, provider configuration, UI branch, or data authority. Stripe is the sole
external subscription authority; the HONE Billing ledger remains the only
application-access truth source, and paid access still requires a verified paid
webhook rather than a redirect, email login, or frontend state.

Production runs source revision
`edddfc5b890d124d76d8c6eddc9aa85f2e94b807` from OCI manifest digest
`sha256:0dcd14a825a124344908b34f6cab19f83eca1f614a40eb2bdf08df2f093f0eee`.
The two identities are recorded separately because a Git revision and an OCI
digest must never be treated as interchangeable.

## What Changed

- Stripe-only implementation commit `9961652f` removed Whop runtime code and
  compatibility, tightened SQLite/PostgreSQL provider constraints, and updated
  the public activation/account/plan flow.
- GHCR architecture commits `91e93b51` and
  `edddfc5b890d124d76d8c6eddc9aa85f2e94b807` moved managed Linux compilation
  into digest-pinned Debian `linux/amd64` GitHub Actions builds with BuildKit
  cache and strict release metadata/SHA verification.
- Runtime Image run `30893733765` passed in about 5 minutes 23 seconds and
  published digest
  `sha256:0dcd14a825a124344908b34f6cab19f83eca1f614a40eb2bdf08df2f093f0eee`.
  The private package was exported daemonlessly with a temporary minimally
  scoped `read:packages` credential; no registry credential remains on GCE.
- `/opt/hone/current` points to
  `/opt/hone/releases/edddfc5b890d124d76d8c6eddc9aa85f2e94b807-ghcr-runtime`.
  The protected runtime environment is `root:root 0600`; its pre-live backup is
  `/etc/hone/runtime.env.pre-stripe-live-20260804T084657Z`.
- A post-cutover `503` on verification email exposed that production had relied
  on a developer-local ignored `.env`. The three Cloudflare Email Sending
  values are now installed in the formal runtime environment. Startup confirms
  sender assembly without logging credentials.
- In the external Chrome `bamang_research` profile, Whop product
  `prod_9jQsUKaifh6ZA` and plan `plan_ZXfsAisr4UOaw` were hidden; the public
  product page has no price or purchase CTA, and the HONE webhook was deleted.

## Verification

- Full `bash tests/regression/run_ci.sh` passed on the final code, including
  workspace Rust checks/tests, Web tests/build/typecheck, Edge Worker tests, and
  CI-safe billing/runtime-image contracts.
- `/api/meta` reports exact source SHA
  `edddfc5b890d124d76d8c6eddc9aa85f2e94b807`, source
  `ghcr_linux_oci`, healthy authoritative PostgreSQL/S3, and zero local durable
  dependency. `hone-web.service` is active with `NRestarts=0`, and loopback
  ports `8077/8088` are listening.
- Production Billing tables contained zero entitlement and webhook rows before
  and after cutover; Stripe-only database constraints are installed. An invalid
  public Stripe webhook returned `401` without mutation, and unauthenticated
  Checkout returned `401`.
- The production environment is live mode with Checkout enabled, exact Product
  `prod_V0FIIUS22IGljn`, annual Price
  `price_1U0Eo6EK7h1dD4JHDrhlnPw8`, registered eight-event webhook, restricted
  key, seven-day grace, and no Whop provider variables.
- A real verification email was received, the same challenge code was accepted,
  and HONE created an official Stripe live Checkout Session. Stripe API and the
  official page showed `livemode=true`, subscription mode, USD 199.99/year,
  `status=open`, and `payment_status=unpaid`. No payment details were entered
  and no charge occurred.
- Canceling the live Checkout returned to HONE; `/me` still reported no paid
  entitlement. This proves the live success/cancel surface cannot grant access
  before a paid webhook.
- Whop readback showed the product hidden, the plan hidden, active users `0`,
  lifetime revenue `US$0.00`, no public purchase CTA, and no HONE webhook.
- Redacted visual evidence is in
  `/Users/bytedance/.codex/visualizations/2026/08/03/019fc5c7-d3a5-7df1-83fc-5f0826ad4519/stripe-billing-acceptance/`:
  `20-plan-live.png`, `21-activate-stripe-only.png`,
  `22-live-checkout-summary.png`, `23-me-unpaid-entitlement.png`, and
  `24-whop-product-plan-hidden.png`.

## Risks / Follow-ups

- Live acceptance intentionally stopped before real payment. The complete
  payment/webhook/entitlement lifecycle is proven in Stripe test mode, including
  a 13-event Test Clock run, but live `invoice.paid → active entitlement` still
  requires an explicitly authorized real US$199.99 transaction.
- Add provider API reconciliation for webhook events missed beyond retries.
- Define refund/dispute automation, statement descriptor/support operations,
  and Stripe Tax policy before materially expanding sales jurisdictions.
- The existing Whop company API key was not deleted because it is not used by
  Billing and may support unrelated company management. Remove it only after a
  separate usage audit.
- Never expose or persist Stripe/Cloudflare secrets in Git, screenshots, shell
  arguments/history, logs, tickets, or broad registry configuration.

## Next Entry Point

For normal operation, start with `docs/runbooks/stripe-billing.md`. For the
optional final live-money proof, obtain explicit owner authorization first,
then use a controlled purchaser, verify the exact live webhook event IDs and
ledger projection, inspect Portal, and refund/cancel according to the approved
business policy.
