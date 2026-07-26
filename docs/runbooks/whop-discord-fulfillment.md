# Runbook: Whop To Discord VIP Fulfillment

Last updated: 2026-07-26

## Purpose

Use Whop's native Discord app to give an active Whop member access to the
巴芒投研 Discord server and the paid-member role. Whop owns Discord account
linking, role grant, and role removal for this workflow; HONE does not mirror
these actions through a billing webhook.

## Canonical Resources

- Whop business: `biz_h0UKqlfUJI55Am` (`巴芒投研`)
- Whop product: `prod_9jQsUKaifh6ZA`
  (`B&M Research Membership / 巴芒投研会员`)
- Whop plan: `plan_ZXfsAisr4UOaw` (`USD 199.99/year`, no trial)
- Public product route:
  `https://whop.com/edda1183-b297-4502-811f-339ae5e773be/bm-research-membership/`
- Whop Discord experience: `discord-rnZu3TEyv0EM79`
- Discord guild: `1391380994182877205` (`巴芒投研美股社群`)
- Managed role: `VIP 付费用户`
- Event log channel: `1423211147674517537` (`whop`)
- Cancellation action: `Remove Role`

The Discord role identifier is not exposed by the Whop settings UI used during
setup. Treat the exact role name plus the guild identifier as the operator
lookup key; confirm the role in both Whop and Discord before changing it.

## Expected Member Flow

1. The customer buys the canonical Whop product.
2. In Whop, the customer opens Connected Accounts and links one Discord
   account.
3. The customer opens the Discord app included in the purchased whop.
4. The customer selects the linked Discord account and chooses
   `Claim Access`.
5. Whop redirects the customer to Discord, joins or opens the configured
   server, and grants `VIP 付费用户`.
6. When the membership no longer grants access, Whop applies the configured
   `Remove Role` cancellation action.

Whop's member instructions are the source of truth for the linking and claim
steps:

- <https://docs.whop.com/memberships-and-access/access-discord-server/access-a-discord-server>
- <https://docs.whop.com/memberships-and-access/accessing-your-purchase/transfer-a-membership>

One membership should be bound to one Discord account. If a member changes
accounts, resolve the existing connection or membership transfer in Whop
instead of manually leaving two Discord accounts with the paid role.

## Configuration Check

In the canonical Whop product editor:

1. Open the product's `Apps` tab.
2. Confirm `Discord` is included and save the product.
3. Open the Discord app settings.
4. Under `Roles`, confirm `VIP 付费用户` is under
   `Roles included in 'Discord'`.
5. Under `Settings`, confirm:
   - Discord server is `巴芒投研美股社群`.
   - Event log channel is `whop`.
   - Cancellation action is `Remove Role`.
   - The past-due role is unset unless the business intentionally adds a
     restricted grace-period role.

In Discord:

1. Confirm the Whop integration is still installed in guild
   `1391380994182877205`.
2. Confirm the Whop Bot/integration role is above `VIP 付费用户`; Discord does
   not allow a bot to manage a role above its own highest role.
3. Confirm `VIP 付费用户` has access to the intended restricted channels and no
   administrative permissions.
4. Confirm the `whop` event-log channel is restricted to operators.

Do not copy a bot token into Whop, repository files, shell history, screenshots,
or tickets. The HONE Discord bot is a separate message/agent integration and is
not part of this fulfillment path.

## Acceptance Test

Use a non-owner Whop test membership and a separate Discord test account when
one is safely available:

1. Purchase or grant the canonical product without charging an unrelated real
   customer.
2. Link the test Discord account.
3. Claim Discord access.
4. Confirm the account joins guild `1391380994182877205`.
5. Confirm `VIP 付费用户` is granted and the VIP channels are readable.
6. Claim access again and confirm the operation is idempotent.
7. End the test membership using the intended cancellation/expiration path.
8. Confirm `VIP 付费用户` is removed at Whop's effective access-end time.
9. Reactivate the membership and confirm the role can be granted again.
10. Review the `whop` event log for every grant/removal and any error.

The creator's `Preview as member` mode proves the linked-account selector and
`Claim Access` UI, but intentionally does not grant a real Discord role. Do not
record that preview as proof of the grant/revoke lifecycle.

## Incident Recovery

### Member cannot see the Discord app

- Confirm the membership is for `prod_9jQsUKaifh6ZA`, not an archived product.
- Confirm Discord remains included in the canonical product's Apps tab.
- Confirm the membership is active and has not reached its effective access-end
  time.

### Member cannot claim access

- Ask the member to confirm the intended Discord account under Whop Connected
  Accounts.
- Check whether the membership is already bound to another Discord account.
- Check the Whop event log before making any manual role change.
- Confirm the Whop integration remains connected to the intended guild.

### Member joins but does not receive VIP

- Confirm `VIP 付费用户` is still included in the Whop Discord experience.
- Move the Whop integration role above the managed role if hierarchy is wrong.
- Confirm the integration has permission to manage roles.
- Retry `Claim Access` after correcting the integration; avoid leaving a manual
  grant as the permanent source of truth.

### Former member still has VIP

- Confirm the membership's effective access-end time; cancellation, refund,
  dispute, and natural expiration can end access at different times.
- Check the Whop event log for a removal failure.
- Correct the Whop role hierarchy or connection and let Whop reconcile first.
- If immediate containment is required, remove the role manually and record the
  membership plus incident. Keep Whop as the long-term owner so reactivation
  remains consistent.

## Rollback

If Discord fulfillment itself must be stopped:

1. Remove the Discord app from the canonical product or disconnect the Discord
   server in the Whop Discord settings.
2. Verify the public/member product view no longer offers Discord claim access.
3. Decide explicitly whether existing managed roles should remain or be
   removed; do not bulk-delete members or historical Whop products.
4. Preserve the `whop` event-log evidence for diagnosis.

Rollback of this integration does not alter HONE product entitlements. The
repository currently has no Whop billing webhook or implemented entitlement
ledger; that remains a separate product/backend project.
