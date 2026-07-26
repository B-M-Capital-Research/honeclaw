- title: Whop Purchase To Discord VIP Fulfillment
- status: archived
- created_at: 2026-07-26
- updated_at: 2026-07-26
- owner: Codex
- related_files:
  - docs/current-plan.md
  - docs/runbooks/whop-discord-fulfillment.md
  - docs/handoffs/2026-07-26-whop-discord-fulfillment.md
  - docs/archive/index.md
- related_docs:
  - docs/archive/plans/whop-product-alignment.md
  - docs/handoffs/2026-07-26-whop-product-alignment.md
  - docs/proposal/auto_p2_self-serve-billing-checkout.md

## Goal

Give a paid Whop member a complete Discord fulfillment path: purchase the
canonical annual membership, link a Discord account, claim access, join the
configured Discord server, receive the configured VIP role, and lose that
managed access when the Whop membership no longer grants it.

## Scope

- Prefer Whop's native Discord app for Discord server and role lifecycle.
- Attach the Discord app to canonical product `prod_9jQsUKaifh6ZA`.
- Use the existing Discord server and a dedicated VIP role selected in the
  authenticated dashboards.
- Keep the Whop Bot role above the managed VIP role and grant only the minimum
  Discord permissions required by Whop.
- Do not add a HONE billing webhook merely to manage Discord roles.
- Treat HONE product entitlement sync as a separate follow-up because the
  repository does not yet implement the proposed billing/entitlement ledger.

## Validation

- Confirm the Discord app is included in the canonical Whop product.
- Confirm the configured Discord server and VIP role are visible in the app
  settings.
- Confirm the customer view exposes the Discord claim flow.
- Verify with a non-owner test membership when safely available:
  Discord linking, server join, VIP role grant, duplicate claim idempotency,
  membership termination/revocation, and reactivation.
- Record any verification blocked by the lack of a safe test buyer or by
  Discord/Whop role permissions.

## Documentation Sync

- Add or update `docs/runbooks/whop-discord-fulfillment.md` with setup,
  verification, incident recovery, and rollback instructions.
- On completion or pause, add a handoff with external resource identifiers and
  verified/unverified boundaries.
- Remove this task from `docs/current-plan.md`, archive this plan under
  `docs/archive/plans/`, and update `docs/archive/index.md` when the workflow is
  complete.

## Risks / Open Questions

- A creator account cannot fully prove the buyer claim experience; a separate
  test Whop/Discord identity may be required for end-to-end verification.
- Whop Bot cannot assign a role above its own Discord role.
- Existing users on archived products should not inherit the new Discord access
  unless they also receive the canonical product or are migrated intentionally.
- Refund, dispute, cancellation, and natural expiration can have different
  effective access dates; verification must use Whop membership state rather
  than payment UI alone.

## Completion

- Whop's Discord app is included in the canonical product.
- The app is connected to guild `1391380994182877205`.
- Role `VIP 付费用户`, event channel `whop`, and cancellation action
  `Remove Role` are configured.
- Member preview exposes the linked-account selector and `Claim Access`.
- Operational instructions are recorded in
  `docs/runbooks/whop-discord-fulfillment.md`.
- A real grant/revoke test remains explicitly deferred until a safe non-owner
  Whop membership and separate Discord account are available.
