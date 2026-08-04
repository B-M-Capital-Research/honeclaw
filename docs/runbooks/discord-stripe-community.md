# Runbook: Discord Stripe-only Community Operations

- status: `production_current`
- last_updated: `2026-08-04`
- owner: `Codex + owner`

## Purpose

Operate `1391380994182877205` (`巴芒投研美股社群`) after the Stripe-only
billing cutover. Discord is a community surface, not a payment authority.
Stripe is the only external subscription authority and HONE Billing remains
the application-access truth source.

## Current Resources

- Management bot application: `1483674289721839736` (`Hone-TEST`)
- Guild nickname: `HONE 社区助手`
- Managed Discord role: `VIP 付费用户` (`1419294087508398192`)
- Membership channel: `1419303479712677948` (`💎｜会员权益`)
- Current pinned Stripe-only message: `1534163594952966174`
- Restricted historical payment channel:
  `1423211147674517537` (`📋｜历史支付日志`)
- Stripe activation: `https://hone-claw.com/activate`
- HONE account status: `https://hone-claw.com/me`

The local token lives only in Git-ignored `config.yaml`, which must remain mode
`0600`. Never copy the token, emails, verification codes, payment evidence, or
historical log bodies into source, screenshots, chat, tickets, shell arguments,
or command history.

## Permission Boundary

The owner explicitly approved Discord `Administrator` for the dedicated
management bot. The initial narrower role had Manage Guild/Channels/Roles/
Messages plus channel read/write, but Discord channel-level denies still
blocked posting in `💎｜会员权益`, and a managed bot role cannot edit its own
overwrites. Administrator is therefore the current cross-category management
exception.

Treat the token as production-admin material. Keep the local Discord listener
disabled unless a separate task reviews `chat_scope`, allowlists, response
triggers, scheduler output, and audit requirements. Administrator does not
authorize unsolicited chat replies, moderation, member deletion, bans, bulk
role changes, or access grants.

If a later owner wants to remove Administrator, first create owner-managed
explicit role/channel overwrites for every required public, VIP, and backend
category. Prove create/edit/pin/delete and readback in each required channel,
then downgrade and rerun the same canary. Never downgrade first and leave an
unmaintainable community surface.

## Stripe-only Membership Copy

The current membership pin must:

1. link only to HONE `/activate` for purchase and `/me` for status;
2. say Checkout redirects or screenshots never grant access;
3. say the server must confirm Stripe payment before HONE access appears;
4. avoid claiming automatic Discord role delivery;
5. direct Discord VIP requests to the operations team for private verification;
6. prohibit posting email, payment screenshots, card data, or codes publicly.

HONE does not currently synchronize Stripe webhooks into Discord roles. An
operator may grant or restore `VIP 付费用户` only after confirming the account's
active Stripe-backed HONE entitlement in the protected administrator surface.
Discord role state must never grant HONE access or override the Billing ledger.

## Historical Whop Evidence

The former `📋｜whop` channel is now `📋｜历史支付日志` with a topic explaining
that old records are archival and are not current entitlement evidence. Its old
provider messages may contain personal information, so keep the channel
restricted, do not screenshot/export its contents, and do not bulk-delete it
without a retention decision.

The former public Whop purchase pin was replaced transactionally: create and
pin the correct Stripe message, verify its exact links/content, then delete the
old message. The deleted message ID is retained only as an audit locator; its
body is not persisted in repository documents.

## Acceptance Checklist

- Bot identity and target guild return `200` from Discord API.
- Bot member/roles/channels/integrations/webhooks return `200`.
- Bot permissions report `administrator=true`.
- `💎｜会员权益` has exactly one pin with Stripe, `/activate`, `/me`, and no
  Whop purchase URL.
- Deleted legacy public message returns `404`.
- Historical channel name/topic identify archival status and current Stripe
  authority; channel stays restricted.
- Guild integrations and webhooks contain no active Whop resource.
- Browser page shows `HONE 社区助手`, the current pin, correct links, and zero
  visible Whop text.
- No unrelated message, member, role, permission, or historical log was changed.

## Safe Update / Rollback

For future copy changes, publish and pin the replacement first. Validate the
new message through API and browser before deleting the prior current pin. If
the new copy is wrong, delete only the new message and keep the last verified
pin; never restore a Whop checkout or automatic-role claim.

If the bot token is exposed, rotate it in Discord Developer Portal, replace the
ignored config value through a secret-safe input path, validate identity/guild,
and revoke the old token before another management action.
