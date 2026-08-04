# Discord Stripe-only Community Migration Handoff

- title: Discord Stripe-only community migration
- status: `done`
- created_at: `2026-08-04`
- updated_at: `2026-08-04`
- owner: `Codex + owner`
- related_files:
  - `config.yaml`（Git 忽略；token 未记录）
  - `docs/runbooks/discord-stripe-community.md`
  - `docs/runbooks/whop-discord-fulfillment.md`
- related_docs:
  - `docs/archive/plans/discord-stripe-community-migration.md`
  - `docs/decisions.md#d-2026-08-04-04-make-discord-community-operations-stripe-only`
  - `docs/handoffs/2026-08-04-stripe-only-production-cutover.md`
- related_prs: direct `main` documentation commit; no PR, release, deployment, or tag
- verification: Discord identity/guild/member/roles/channels/integrations/webhooks API readback; transactional pin replacement; deleted-message `404`; Administrator permission proof; real external Chrome DOM and screenshot QA
- risks: management bot has Administrator; token protection is critical; Discord VIP remains a manual operator action and is not Stripe/HONE entitlement authority; historical provider logs contain personal information and must remain restricted

## Summary

The live Discord community now presents Stripe-only membership instructions.
The protected local bot is installed in the correct 629-member guild, has the
owner-approved management authority needed to cross private channel denies,
and appears as `HONE 社区助手`. The public membership channel no longer exposes
Whop purchase or automatic-role instructions.

## What Changed

- Added bot application `1483674289721839736` to guild
  `1391380994182877205` and set its guild nickname to `HONE 社区助手`.
- The first minimal permission grant could read but not post in the membership
  channel because of an `@everyone` send deny. Discord also rejected the bot
  modifying its own managed-role overwrite. The owner then explicitly approved
  Administrator; final role permissions are `8` and API reports
  `administrator=true`.
- Posted and pinned message `1534163594952966174` with HONE/Stripe activation,
  `/me` status, webhook-authoritative access, manual Discord VIP verification,
  and PII safety instructions.
- Deleted the single stale public Whop purchase pin
  `1419304118509375624` only after the replacement passed API verification.
- Renamed the restricted `📋｜whop` channel to `📋｜历史支付日志` and added an
  archival-only Stripe-authority topic. Historical message bodies were retained.
- Confirmed no active Whop Discord integration or webhook remains.

## Verification

- Token stayed inside mode-`0600`, Git-ignored `config.yaml` and was never
  printed. Discord identity returned verified bot `Hone-TEST`.
- Guild, bot member, roles, channels, integrations, and webhooks all returned
  `200`; target guild name and ID matched.
- Final role readback: Administrator, Manage Guild/Roles/Channels/Messages,
  View/Send/Read History all effective.
- Replacement message create `200`, pin `204`, pin readback `200`; it was the
  only final pin and contained Stripe plus HONE links without Whop.
- Old message delete `204`, subsequent readback `404`.
- Channel rename/topic and bot nickname PATCH each returned `200` with exact
  readback.
- Integrations were `Hone`, `Hone-TEST`, and `ad-account-detector`; guild
  webhook list was empty.
- External Chrome showed the full Stripe-only membership pin, `HONE 社区助手`,
  correct links, and zero visible Whop text. Redacted evidence:
  `/Users/bytedance/.codex/visualizations/2026/08/04/discord-stripe-community/01-membership-stripe-only.jpg`.
- A proposed historical-log screenshot was immediately deleted after visual
  inspection found old member personal information; it is not an artifact.

## Risks / Follow-ups

- Administrator includes kick/ban and bypasses channel denies even though this
  migration did not use those abilities. Downgrade only after owner-managed
  category/channel overwrites are created and proven.
- The local listener remains disabled to prevent unsolicited bot participation.
  Enabling it requires a separate channel-runtime review.
- Discord VIP role grants are manual. A future Stripe-to-Discord automation
  needs its own identity binding, webhook/inbox authority, reconciliation,
  revocation, audit, and recovery design; do not infer access from email or a
  payment screenshot.
- Historical provider logs include PII. Keep them restricted and define a
  retention/deletion policy before exporting or purging them.

## Next Entry Point

Use `docs/runbooks/discord-stripe-community.md` for copy, role, or permission
maintenance. For a new VIP request, verify active Stripe-backed HONE entitlement
in the protected administrator surface before changing the Discord role.
