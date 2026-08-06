# Stripe Wallet Fixed-term Pass Deployment

- title: Stripe Wallet Fixed-term Pass Deployment
- status: `blocked`
- created_at: `2026-08-06`
- updated_at: `2026-08-06`
- owner: `Codex + owner`
- related_files: `memory/src/billing.rs`, `crates/hone-core/src/cloud_runtime.rs`, `crates/hone-web-api/src/routes/{billing,stripe}.rs`, `packages/app/src/pages/{public-activate,public-me}.tsx`, `tests/regression/{ci,manual}/test_stripe_billing_*.sh`
- related_docs: `docs/current-plans/stripe-wallet-one-time-pass.md`, `docs/runbooks/stripe-billing.md`, `docs/decisions.md#d-2026-08-06-02-offer-recurring-and-fixed-term-stripe-memberships-as-separate-products`
- related_prs: direct `main` implementation commit `c99babc1e1ea3c54db41256331eb65dcefa7bd1d`; no PR, release, or tag
- verification: complete repository gates; official Stripe test-mode Alipay and WeChat Pay payments; exact GHCR/GCE deployment; public config and auth probes; external-Chrome production offer and Checkout acceptance without live payment
- risks: live Alipay and WeChat Pay are externally blocked on Stripe approval; no production wallet claim is valid while Stripe reports `available=false`

## Summary

HONE now offers two server-owned Stripe products: the existing USD 199.99/year
auto-renewing card subscription and a USD 229.99/12-month fixed-term pass that
is paid once and does not renew. The code, tests, Stripe catalog/webhook,
immutable GHCR deployment, production dual-offer page, and live fixed-term
Checkout all passed. Production wallet display is the only remaining gate:
Stripe still marks both Alipay and WeChat Pay pending approval, so the current
live Checkout correctly exposes card only.

## What Changed

- Billing storage now uses explicit `recurring_subscription` and
  `fixed_term_purchase` entitlements with a generic provider reference.
- Fixed-term Checkout uses `mode=payment`; only verified paid events grant 12
  calendar months. Pending, failed, expired, replayed, wrong-catalog, and
  forged events fail closed. A matching full refund revokes only that pass.
- `/activate` presents both offers; `/me` shows fixed validity without a
  Customer Portal cancellation action.
- The live fixed-term Price is `price_1U1M0rEK7h1dD4JHbKBpIkZ2`. The live
  webhook destination `we_1U0c0XEK7h1dD4JHrvQ9CRaH` listens to the exact
  ten-event contract in `docs/runbooks/stripe-billing.md`.
- Revision `c99babc1e1ea3c54db41256331eb65dcefa7bd1d` was built by Runtime Image
  run `31082512757` and deployed from immutable digest
  `sha256:dadf8fcf340cf8fa4971605c3f085f7e097efc7cc2c9a8e1ff4a61d757ca90cb`.

## Verification

- Rust format/check/test, Web typecheck and 364 tests, Edge Worker typecheck and
  45 tests, CI-safe regressions, focused Billing/Stripe tests, signed HTTP E2E,
  and three Playwright billing cases passed.
- Official Stripe test Checkout completed both Alipay and WeChat Pay
  test-payment lifecycles at USD 229.99.
- GCE runs the exact revision at
  `/opt/hone/releases/c99babc1e1ea3c54db41256331eb65dcefa7bd1d-ghcr-runtime`;
  `/api/meta` reports `ghcr_linux_oci`, authoritative healthy PostgreSQL/S3,
  and zero local durable dependency. The service is active with `NRestarts=0`.
- Production public Billing config reports both offers enabled with the correct
  amount, duration, and renewal flags. Invalid webhook and unauthenticated
  fixed-term Checkout probes both returned `401`.
- External Chrome showed the production dual-offer page and an authenticated
  USD 229.99 live Checkout. No live payment was submitted. Redacted evidence is
  retained outside Git as `25-test-checkout-alipay-wechat.png`,
  `27-live-dual-offer-activate.png`, `28-live-fixed-checkout-summary.png`, and
  `29-live-fixed-checkout-payment-methods.png`.

## Risks / Follow-ups

- Alipay and WeChat Pay are `display_preference=on` but `available=false` and
  `pending approval`. This is Stripe-controlled and cannot be fixed by another
  code or deployment change.
- Until approval, UI copy states the intended fixed-term wallet support, while
  the authoritative live Checkout displays only card. Do not advertise the
  production wallets as active yet.
- No live-money acceptance is necessary. After approval, a zero-charge browser
  inspection of hosted Checkout is sufficient to close the remaining gate.

## Next Entry Point

1. Read both live methods from Stripe and require `available=true`.
2. Create a fresh authenticated `fixed_term` Checkout from production
   `/activate`; do not submit payment.
3. Confirm USD 229.99, no recurring marker, and visible card, Alipay, and WeChat
   Pay options; save one redacted screenshot.
4. Mark the plan `done`, move it to `docs/archive/plans/`, remove it from
   `docs/current-plan.md`, and update this handoff plus `docs/archive/index.md`.
