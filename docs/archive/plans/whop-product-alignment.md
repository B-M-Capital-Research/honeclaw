- title: Whop Product Alignment With Knowledge Planet Membership
- status: done
- created_at: 2026-07-26
- updated_at: 2026-07-26
- owner: Codex
- related_files:
  - packages/app/src/pages/public-plan.tsx
  - packages/app/src/lib/public-content.ts
  - packages/app/src/pages/public-me.tsx
  - docs/current-plan.md
  - docs/handoffs/2026-07-26-whop-product-alignment.md
  - docs/archive/index.md
- related_docs:
  - docs/deliverables.md
  - docs/proposal/auto_p2_self-serve-billing-checkout.md

## Goal

Create one canonical Whop membership product that is equivalent to the current
Knowledge Planet membership benefits while retaining the repository-declared
international price of USD 199.99 per year. Remove the stale 30-day free trial
and Discord-specific fulfillment copy, verify the public checkout, update the
repository purchase URL, and safely retire obsolete archived Whop products.

## Outcome

- Created canonical Whop product `prod_9jQsUKaifh6ZA` at
  `https://whop.com/edda1183-b297-4502-811f-339ae5e773be/bm-research-membership/`.
- Created annual renewal plan `plan_ZXfsAisr4UOaw` at USD 199.99/year with no
  free trial, unlimited stock, tax collected exclusively, and no affiliate
  enrollment.
- Kept all four historical products archived. Whop returned
  `deletable: false` for every attached historical plan, including the two
  zero-member test plans, so no destructive cleanup was attempted.
- Updated the repository English purchase path to the new public product and
  added regression coverage for localized pricing and the canonical Whop URL.

## Scope

- Treat `packages/app/src/lib/public-content.ts` as the product-copy and
  localized-pricing source of truth:
  - Knowledge Planet: CNY 1,299/year, current CNY 100 newcomer discount, and an
    additional CNY 100 referral promotion.
  - Whop: USD 199.99/year.
- Create one Whop product/plan with the same four membership benefits shown on
  the public plan and account pages.
- Do not add a free trial, Discord-only fulfillment claim, or unsupported HONE
  entitlement automation.
- Update `WHOP_URL` only after the new public checkout is verified.
- Delete obsolete archived products only when Whop reports deletion is allowed
  and the product has no membership/history dependency. Otherwise leave them
  archived and record why.

## Validation

- Read back the created Whop product and plan through the Whop CLI.
- Verify price, annual billing period, no trial, visibility, tax behavior,
  benefits copy, public purchase URL, and product route.
- Open the public Whop product page and confirm the checkout surface renders.
- Run the focused Web tests/typecheck that cover public pricing content and the
  purchase page.
- Confirm `git diff --check` and inspect the final worktree.

## Documentation Sync

- Keep this plan current while the external configuration and repository link
  are changing.
- On completion, add a handoff with Whop resource IDs, verification evidence,
  cleanup results, and rollback notes.
- Remove this task from `docs/current-plan.md`, archive this plan under
  `docs/archive/plans/`, and add the completed task to `docs/archive/index.md`.

## Risks / Open Questions

- Whop product deletion may be blocked or harmful when historical/active
  memberships exist; archive state is safer than forced deletion.
- The current Whop OAuth profile cannot list webhooks through the CLI, but the
  dashboard currently shows no webhook rows.
- A Whop purchase still does not grant HONE entitlement automatically because
  the repository has no billing webhook implementation.
- Knowledge Planet promotions remain platform-specific; the new Whop product
  uses the repository-declared USD 199.99/year international price.
