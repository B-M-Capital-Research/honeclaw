- title: Whop Purchase To Discord VIP Fulfillment
- status: done
- created_at: 2026-07-26
- updated_at: 2026-07-26
- owner: Codex
- related_files:
  - docs/runbooks/whop-discord-fulfillment.md
  - docs/archive/plans/whop-discord-fulfillment.md
  - docs/current-plan.md
  - docs/archive/index.md
  - docs/decisions.md
- related_docs:
  - docs/archive/plans/whop-product-alignment.md
  - docs/handoffs/2026-07-26-whop-product-alignment.md
  - docs/proposal/auto_p2_self-serve-billing-checkout.md
- related_prs: []

## Summary

The canonical Whop annual membership now includes Whop's native Discord app.
It is connected to the existing 巴芒投研美股社群 server, grants the existing
`VIP 付费用户` role, writes integration events to `#whop`, and removes the role
when Whop applies the configured cancellation action.

This workflow does not require a custom Whop webhook or the HONE Discord bot.
Whop remains the source of truth for Discord account binding and Discord role
lifecycle. HONE product entitlements remain a separate, not-yet-implemented
billing project.

## What Changed

- Enabled the Discord app for Whop product `prod_9jQsUKaifh6ZA`.
- Confirmed Whop Discord experience `discord-rnZu3TEyv0EM79`.
- Confirmed guild `1391380994182877205` (`巴芒投研美股社群`).
- Confirmed `VIP 付费用户` is included in the experience.
- Confirmed event log channel `1423211147674517537` (`whop`).
- Confirmed cancellation action `Remove Role`.
- Added `docs/runbooks/whop-discord-fulfillment.md`.
- Recorded the native-integration decision in `docs/decisions.md`.
- Did not add a webhook, backend endpoint, Discord role-management code, or
  secret to the repository.

The provided HONE bot credential was handled only for a secret-safe membership
check. That bot is installed in another guild and is not installed in the
target guild, so it was not used or invited. No token value was printed,
persisted, or copied into the repository.

## Verification

- Authenticated Whop product editor: Discord included and product saved.
- Authenticated Whop Discord settings:
  - correct server visible;
  - `VIP 付费用户` listed under included roles;
  - `whop` selected as event log channel;
  - `Remove Role` selected for cancellation.
- Whop member preview:
  - linked Discord account selector visible;
  - `Add Account` visible;
  - `Claim Access` visible;
  - preview mode correctly refused to perform a real role grant and identified
    itself as a simulation.
- Repository audit:
  - no current Whop webhook implementation;
  - `hone-discord` handles messaging/agent traffic, not Discord role lifecycle;
  - self-serve billing/entitlements remain proposal-only.
- Documentation checks: `git diff --check`.

## Risks / Follow-ups

- A separate non-owner Whop membership and Discord account are still required
  to prove a real join, grant, duplicate claim, expiration/removal, and
  reactivation sequence. Admin preview cannot prove those mutations.
- Before the first real customer claim after this re-enable, confirm in Discord
  that the Whop integration role is above `VIP 付费用户`.
- Archived Whop products remain outside this entitlement. Do not migrate their
  members implicitly.
- If HONE itself must unlock paid application features, implement the billing
  entitlement ledger and webhook as a separate scoped project.

## Next Entry Point

Use `docs/runbooks/whop-discord-fulfillment.md` for operations and the non-owner
acceptance checklist. Use
`docs/proposal/auto_p2_self-serve-billing-checkout.md` only for the separate
HONE entitlement project.
