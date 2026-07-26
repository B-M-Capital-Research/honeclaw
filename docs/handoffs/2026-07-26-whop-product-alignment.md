- title: Whop Product Alignment With Knowledge Planet Membership
- status: done
- created_at: 2026-07-26
- updated_at: 2026-07-26
- owner: Codex
- related_files:
  - packages/app/src/pages/public-plan.tsx
  - packages/app/src/pages/public-plan-purchase-contract.test.ts
  - packages/app/src/lib/public-content.test.ts
  - docs/archive/plans/whop-product-alignment.md
  - docs/archive/index.md
- related_docs:
  - packages/app/src/lib/public-content.ts
  - packages/app/src/pages/public-me.tsx
  - docs/proposal/auto_p2_self-serve-billing-checkout.md
- related_prs: N/A

## Summary

The international Whop membership now mirrors the current Knowledge Planet
membership benefits while retaining the repository-declared localized price.
Knowledge Planet remains CNY 1,299/year with the current CNY 100 newcomer
discount and additional CNY 100 referral promotion. Whop uses USD 199.99/year.

## What Changed

- Created Whop product `prod_9jQsUKaifh6ZA`:
  - title: `B&M Research Membership / 巴芒投研会员`
  - route: `bm-research-membership`
  - visibility: `visible`
  - public URL:
    `https://whop.com/edda1183-b297-4502-811f-339ae5e773be/bm-research-membership/`
  - global and member affiliate enrollment disabled
  - bilingual copy covers the same four benefits as
    `packages/app/src/lib/public-content.ts`
- Created Whop plan `plan_ZXfsAisr4UOaw`:
  - annual renewal, USD 199.99/year
  - no free trial
  - unlimited stock
  - exclusive tax collection
  - webhook-facing metadata identifies
    `service=bm_research_membership`, `entitlement=full_access`, and
    `billing_period=annual`
- Removed shipping-address collection in the Whop dashboard.
- Replaced the archived `vip-copy-18` URL in
  `packages/app/src/pages/public-plan.tsx`.
- Added regression assertions for the localized pricing strategy, four-benefit
  contract, canonical Whop URL, and removal of the old route.
- Historical products remain archived. Their attached plans all report
  `deletable: false`; the two products with existing members were never
  candidates for deletion, and Whop also locks the two zero-member plans.

## Verification

- Whop CLI product readback:
  - visible route `bm-research-membership`
  - zero members at creation
  - no Discord text
  - affiliate status disabled
- Whop CLI plan readback:
  - `$199.99 / year`
  - `trial_period_days: null`
  - `billing_period: 365`
  - `tax_type: exclusive`
  - `collect_tax: true`
  - `unlimited_stock: true`
- Browser verification:
  - dashboard shows paid annual access and the free-trial switch off
  - public product page renders the bilingual benefits and USD 199.99/year
  - no stale Discord `Claim Access` fulfillment text appears
- Public URL returned HTTP 200.
- `bun test --preload ./happydom.ts ./src/lib/public-content.test.ts ./src/pages/public-plan-purchase-contract.test.ts`:
  5 passed, 0 failed.
- `bun run typecheck` in `packages/app`: passed.
- `bun run test:web`: 283 passed, 0 failed.

## Risks / Follow-ups

- Whop payments still do not automatically grant HONE access. The repository
  has no implemented Whop billing webhook or entitlement consumer, and the
  Whop dashboard currently has no webhook configured.
- Existing active memberships remain attached to archived historical products.
  Do not delete or repoint them without a migration and renewal-impact review.
- The Whop CLI currently treats boolean create flags as presence-only; dashboard
  verification was used for shipping, trial, and local-currency settings.
- Rollback: hide `prod_9jQsUKaifh6ZA` and `plan_ZXfsAisr4UOaw`, then restore
  `WHOP_URL` to a verified replacement. Do not restore the old archived product
  without first reviewing its 30-day trial and Discord-specific copy.

## Next Entry Point

For self-serve entitlement automation, begin with
`docs/proposal/auto_p2_self-serve-billing-checkout.md`, add a verified Whop
webhook flow, and map `plan_ZXfsAisr4UOaw` metadata to the HONE full-access
entitlement.
