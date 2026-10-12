# Public site, Research Workspace, Insights overhaul + email allowlist login

- title: Public site redesign, Research Workspace overhaul, Insights influencer sync, allowlisted email login
- status: in_progress (implementation + local verification complete; awaiting review, commit and deploy)
- created_at: 2026-10-11
- updated_at: 2026-10-12
- owner: Claude (session with contact@honeclaw.app)
- related_files:
  - `packages/app/src/pages/public-home.tsx`, `public-site.css`, `lib/public-content.ts`
  - `packages/app/src/pages/public-research.tsx`, `public-research.css`, `lib/public-workspace-research.ts`
  - `packages/app/src/pages/public-community.tsx`, `components/influencer-digest-dashboard.tsx`
  - `packages/app/src/components/public-login-form.tsx`, `components/public-admin-whitelist-panel.tsx`
  - `crates/hone-web-api/src/{email_verification.rs,lib.rs,routes/public.rs,routes/public_admin.rs,types.rs}`
  - `memory/src/web_auth.rs`, `crates/hone-core/src/cloud_runtime.rs`
- related_docs: `docs/invariants.md` (public login / admin allowlist / Insights streams), `docs/repo-map.md` (packages/app), `docs/decisions.md` (D-2026-10-11-01), `docs/handoffs/2026-10-11-public-site-research-insights-email-login.md`

## Goal

1. Main website: replace outdated content with a concise, professional presentation of the current product and the community, using current images and descriptions.
2. Login: add email + one-time code login next to phone + SMS. Both stay restricted to the admin allowlist; admins can attach an email to an allowlist entry (or create an email-only entry). Codes are sent through Cloudflare Email Sending from the configured `email.from_address` (`no-reply@hone-claw.com`).
3. Research Workspace: complete overhaul to remove clutter.
4. Insights: surface the posts collected by 大V速报 (influencer digest) inside Insights.

## Scope

- Backend auth: allowlist entries carry an optional email (unique, stored in `cloud_web_user_external_state`, identity stays `domestic_invite`); admin create accepts phone and/or email; admin can set/clear an entry's email (audited); email send for login is non-disclosing (background send, uniform response), skips revoked users; verification sender prefers `config.email` and falls back to the legacy env trio; Stripe-intent email flow can no longer flip an allowlisted identity to `international_email`.
- Frontend auth: login form gets a phone/email switch; admin whitelist panel gets an email field + per-entry email edit.
- Home page, research page, Insights page redesign within the existing SolidJS app and design tokens.

## Validation

- `cargo test -p hone-memory web_auth`, `cargo test -p hone-web-api` (email/admin/public auth), `cargo check --workspace --all-targets --exclude hone-desktop --exclude hone-user-app`
- `bun run test:web`, `bun --filter @hone-financial/app typecheck`, public build
- Browser preview of `/`, `/research`, `/community`, login form (desktop + mobile, light + dark)

## Documentation Sync

- `docs/invariants.md`: public login now phone/SMS or allowlisted email/code; non-disclosure + allowlist rules for email.
- `docs/repo-map.md`: packages/app research/community/home ownership; influencer posts in Insights.
- `docs/decisions.md`: allowlist email identity decision.
- Handoff + archive index at completion.

## Risks / Open Questions

- Production must have `email.account_id`, `email.from_address` and `HONE_CLOUDFLARE_EMAIL_API_TOKEN` resolvable or email login returns 503 (phone login unaffected).
- Influencer posts in Insights must not break the official feed's cursor/unread/edge-snapshot contract.

## Progress (2026-10-11)

- [x] Backend allowlist email (storage, admin create/edit API, audit migration, non-disclosing send, config-driven no-reply sender, Checkout identity guard)
- [x] Frontend login method switch + admin allowlist email UI
- [x] Insights 大V速报 tab (shared influencer timeline)
- [x] Research desk overhaul + shared mobile header offset fix
- [x] Home page redesign, nav/footer fixes, retired images removed, page-title fix
- [x] Tests: Rust (memory/web-api/core), web 665, typecheck, public build, targeted CI regressions
- [x] Docs: invariants, repo-map, decisions, handoff
- [ ] Commit / deploy (user decision); production email-send smoke test after deploy

## Production readiness and rollout (2026-10-12)

User authorized reviewing both October 11 Claude changes, completing any gaps, and deploying when ready. This continuation includes the report reliability / shared valuation changes documented in `docs/handoffs/2026-10-11-report-reliability-and-valuation-prompts.md`.

- [ ] Review the auth/storage/UI and report transport/renderer changes, preserve unrelated `hone-quant/` and generated `output/` content, and fix concrete readiness issues.
- [ ] Preserve the currently deployed `42ed74f5` query-pool / synchronous-bridge repair by integrating its published branch before rollout; do not replace it with the older main runtime.
- [ ] Verify changed Rust formatting, workspace check/tests with local PostgreSQL, Web tests/typecheck/public build, Worker checks and CI regressions; verify relevant browser flows.
- [ ] Inspect production revision, managed access, email configuration, active work, and rollback targets before changing production.
- [ ] Commit the reviewed changes, build the exact Linux runtime and matching public/skill bundles, and deploy them together with rollback evidence.
- [ ] Verify exact live revision/assets, cloud health, auth and report paths, and real canaries where production access permits.
- [ ] Update both handoffs and any affected runbooks/invariants; record evidence and remaining risks; archive this plan and remove its active index only after rollout acceptance.

Readiness findings: direct managed-host SSH confirmed production `42ed74f5` (IAP transport failed). Email issuance/verification previously wrote a stale full external-state snapshot; added locked compare-and-exchange with address-change, revocation and one-time-code replay regressions. Restore README_EN provider configuration documentation and reuse the reviewed `943dc2bd` streaming fixture repair. Baseline finance static regression reports 10 failures and needs comparison with current behavior before acceptance.
