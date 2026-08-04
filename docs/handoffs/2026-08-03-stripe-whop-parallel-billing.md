# Stripe + Whop Provider-Neutral Billing Handoff

- title: Stripe + Whop provider-neutral Billing handoff
- status: `in_progress`
- created_at: `2026-08-03`
- updated_at: `2026-08-04`
- owner: `Codex + owner`
- related_files:
  - `memory/src/billing.rs`
  - `memory/src/web_auth.rs`
  - `crates/hone-core/src/cloud_runtime.rs`
  - `crates/hone-web-api/src/routes/{billing,stripe,whop,public}.rs`
  - `packages/app/src/pages/{public-activate,public-me,public-plan}.tsx`
  - `tests/regression/{ci,manual}/test_*billing*.sh`
- related_docs:
  - `docs/current-plans/stripe-whop-parallel-billing.md`
  - `docs/runbooks/stripe-billing.md`
  - `docs/runbooks/whop-hone-activation.md`
- related_prs: no PR; Billing landed on `main` through `92e87a94`, `5028870d`, and Whop route fix `e31c29ac`; no release or tag was created
- verification: local automated/browser acceptance, real Stripe test Checkout, CLI delivery, active entitlement, Portal, exact production backend revision, rotated test endpoint secret installation, online `200 catalog_mismatch` delivery with zero database mutation, and reactive Whop route regression passed
- risks: non-owner Whop buyer and real failure/cancel/recovery/repurchase cases remain pending; GitHub CI still has the known missing-`rg` and two legacy reference-asset findings; live catalog/configuration remains untouched

## Summary

HONE now has a provider-neutral Billing domain with Stripe and Whop adapters,
one canonical entitlement ledger, and no runtime compatibility layer. The
implementation, repository-local validation, recovery semantics, legal copy,
and desktop/iOS browser acceptance are complete. The owner confirmed external
test-mode setup: the exact annual catalog and Customer Portal now exist.
The owner explicitly resumed external sandbox work and completed the Stripe CLI
authenticator challenge, pairing approval, and callback. The active CLI profile
is `HoneClaw`, the real-account catalog check passed, and the ignored local
`.env` now contains the test key. With explicit authorization, Codex submitted
Stripe's public test card for a real test-mode Checkout. The CLI delivered the
three resulting signed events and the Customer Portal displayed the paid
subscription. That run exposed a real provider-ordering edge; the fix and
replay now produce one active entitlement and paid API `200`. The remaining
Whop production acceptance still needs a real non-owner buyer to complete their
own email challenge. Production now runs exact backend `5028870d`; the
registered Stripe test endpoint is online with its rotated secret, while
Checkout remains disabled and Whop remains the temporary primary channel.
Commit `e31c29ac` also fixes the Whop recovery link so changing only the query
on `/activate` updates the page immediately instead of requiring a reload.

## 2026-08-04 Deployment And Online Acceptance

- Production `/api/meta` reports exact `5028870dcb341476e17b57fdfa84d72624b04200`, cloud Web role, authoritative PostgreSQL/S3 storage, and no local durable dependency.
- The old test endpoint signing secret was rotated immediately after accidental exposure. Only the replacement value was installed; it is absent from source, chat, screenshots, and this handoff.
- `/etc/hone/runtime.env` remains `root:root 0600`; the pre-change backup is `/etc/hone/runtime.env.pre-webhook-rotation-20260804T034309Z`. Two consecutive active-chat checks were zero before restarting only `hone-web.service`.
- Runtime policy is intentionally `primary_provider=whop`, `stripe_checkout_enabled=false`, `whop_new_purchases_enabled=true`. The final business direction remains new users on Stripe with Whop as the legacy/secondary channel; current values are a safe verification stage.
- Public invalid signatures return `401`. Stripe test event `evt_1U0ZKeEK7h1dD4JHB59OFEjY` returned `200` with `catalog_mismatch`; production `billing_entitlements` and `billing_webhook_events` were both zero before and after delivery.
- Evidence `13`–`17` covers Workbench `200`, safe-stage `/plan`, `/activate`, unauthenticated `/me`, and the production Whop same-route query transition. Directory: `/Users/bytedance/.codex/visualizations/2026/08/03/019fc5c7-d3a5-7df1-83fc-5f0826ad4519/stripe-billing-acceptance/`.

## What Changed

- Added SQLite/PostgreSQL `billing_entitlements` and
  `billing_webhook_events`; startup performs a one-time forward migration of
  legacy Whop projections and removes provider-specific runtime fields.
- Added authenticated same-origin Stripe Checkout/Portal, exact server-owned
  catalog and mode checks, raw signed Stripe/Whop webhooks, paid-signal-first
  access, bounded grace, duplicate-provider detection, and OR access policy.
- Added durable inbox recovery with five-minute leases, ten-attempt ceiling,
  30-second scans, idempotent/out-of-order-safe projection, and attempt-fenced
  completion.
- Made Checkout completion explicitly provisional: it orders from Session
  creation, so Stripe invoice/subscription events that have slightly earlier
  envelope timestamps still become authoritative. Added unit and signed HTTP
  regression coverage for the exact real-event topology.
- Added a CI-safe HTTP lifecycle regression that boots an isolated real backend
  from a temporary working directory, never loads the repository `.env`, and
  drives signed Stripe and Whop events through the public routes and SQLite
  inbox without contacting either provider.
- Replaced `/activate/whop` with unified `/activate`, moved `/me` and `/plan` to
  server-authoritative Billing data, and made HONE-iOS restore-only with no
  price or external purchase CTA.
- Updated Terms 2.3, privacy/provider copy, architecture documents, runbooks,
  CI-safe contracts, and a manual account-dependent Stripe sandbox script.
- Created test Product `prod_V0J9fIdOhCrS4z`, annual Price
  `price_1U0IXPEK7h1dD4JHHavBWqmr`, and test Portal configuration
  `bpc_1U0IZEEK7h1dD4JHxYx1GhDy`. Portal permits payment-method updates and
  period-end cancellation, forbids plan/quantity changes, and returns to
  `https://hone-claw.com/me`.
- Installed the official Stripe CLI `1.45.0` from Stripe's Homebrew tap and
  completed owner device authorization. Secrets remain only in ignored local
  state; no secret is committed or recorded in acceptance evidence.
- Replaced the activation page's one-time `window.location.search` snapshot
  with a reactive Solid Router search-param source. Contract and Playwright
  regressions prove `/activate` can switch to `?provider=whop` without reload.

## Verification

- Rust: Billing `5/5`, Stripe `7/7`, Whop `2/2`, migration and email-limiter
  tests; full workspace check/test excluding Apple clients passed.
- The fifth Billing regression proves that a late cancellation for an older
  Stripe subscription cannot revoke a newer repurchase, and that access is
  denied only once every entitlement is inactive.
- Web: typecheck, public production build, and `350/350` tests passed.
- Public Community Edge Worker: typecheck and `45/45` tests passed.
- Billing CI contract passed. The final aggregate CI-safe rerun included both
  Billing regressions, reached `43/44`, and failed only the unrelated existing finance automation
  `current-data-capability` prompt contract; this change does not touch that
  subsystem.
- The isolated signed HTTP lifecycle passed. It proves pending checkout stays
  behind `402`, paid activation, exact-catalog rejection, replay idempotency,
  out-of-order safety, bounded grace and recovery, period-end cancellation,
  provider-isolated deletion, duplicate warning, all-inactive `402`, and
  same-provider repurchase behavior for both adapters.
- Real Stripe test mode: Checkout using the public successful test card
  returned to HONE; the CLI listener delivered Checkout, paid Invoice, and
  Subscription events with `202`. After fixing their real timestamp topology,
  a fresh signed replay processed all three once, produced exactly one active
  Stripe entitlement, and changed the paid API from `402` to `200`. Customer
  Portal exposed payment-method update, period-end cancellation, and the paid
  invoice; cancellation was not executed.
- Browser: desktop `/plan`, `/activate`, and `/me` plus 390×844 HONE-iOS
  variants passed visual/DOM checks. An iOS restore-step copy defect was found,
  fixed, and re-captured. A later audit found `/me` still exposed external
  billing management on iOS; server policy now hides both purchase and
  management actions fail-closed. The final 390×844 `/me` screenshot confirms
  both provider states remain visible, management actions are absent, and
  horizontal overflow is zero. The same directory also contains test catalog
  and Portal screenshots. `11-stripe-test-portal-paid-subscription.png` proves
  the paid Portal, and `12-hone-account-stripe-active.png` proves HONE access.
  Evidence directory:
  `/Users/bytedance/.codex/visualizations/2026/08/03/019fc5c7-d3a5-7df1-83fc-5f0826ad4519/stripe-billing-acceptance/`.
- Production registered-endpoint acceptance: the Workbench delivery returned
  `200 catalog_mismatch` for a safe wrong-catalog event, and PostgreSQL inbox
  and entitlement counts remained `0 | 0`. The Whop route fix passed full Web
  tests, typecheck, public production build, and a real Playwright same-route
  transition test before push.

## Risks / Follow-ups

- Local CLI, registered online test, and registered live webhook destinations
  require distinct signing secrets. The registered test endpoint is now
  accepted online; no live endpoint or live secret exists for this rollout.
- Finish the real failure/recovery, cancel/end, and repurchase cases from the
  sandbox matrix. Obtain action-time confirmation before canceling even the
  test subscription.
- Tax, refund, statement descriptor, support, dispute, and live promotion
  policy require explicit owner approval. Keep Stripe Tax off in the sandbox.
- The user-owned untracked `.claude/worktrees/` directory was not modified.

## Next Entry Point

After explicit owner approval, make only the focused GitHub CI fixes already
identified: a `grep` fallback for the Billing contract and the narrowest safe
allowlist for two legacy Alipay reference assets. Then finish the real
failure/recovery/cancel/end/repurchase and non-owner Whop matrices. Obtain
action-time approval before canceling any test subscription and a separate
business approval before live promotion. Keep this task active until those
items pass; only then archive the plan and update `docs/archive/index.md`.
