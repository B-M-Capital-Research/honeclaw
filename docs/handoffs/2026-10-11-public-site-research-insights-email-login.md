# Public site, Research Workspace, Insights and allowlisted email login

- title: 官网重设计 / Research Workspace 重构 / Insights 同步大V速报 / 白名单邮箱登录
- status: in_progress (implementation and local verification complete; not yet committed or deployed)
- created_at: 2026-10-11
- updated_at: 2026-10-11
- owner: Claude (session with contact@honeclaw.app)
- related_files:
  - Auth: `memory/src/web_auth.rs`, `crates/hone-core/src/cloud_runtime.rs`, `crates/hone-web-api/src/{email_verification.rs,lib.rs,types.rs,routes/public.rs,routes/public_admin.rs,routes/mod.rs}`
  - Frontend auth: `packages/app/src/components/{public-login-form.tsx,public-admin-whitelist-panel.tsx}`, `packages/app/src/lib/{api.ts,types.ts}`
  - Insights: `packages/app/src/components/{community-influencer-feed.tsx,influencer-digest-dashboard.tsx}`, `packages/app/src/pages/public-community.{tsx,css}`, `packages/app/src/lib/public-community-timeline.ts`
  - Research desk: `packages/app/src/pages/public-research.{tsx,css}`, `packages/app/src/pages/public-workspace.css`
  - Home: `packages/app/src/pages/public-home.{tsx,css}`, `packages/app/src/lib/public-content.ts` (`landing`), `packages/app/src/components/public-nav.tsx`, `packages/app/public/home-*.jpg`, `packages/app/index.html`, `packages/app/src/entry.tsx`
- related_docs: `docs/current-plans/public-site-research-insights-overhaul-2026-10-11.md`, `docs/decisions.md` (D-2026-10-11-01), `docs/invariants.md`, `docs/repo-map.md`
- related_prs: none yet

## Summary

All four requests are implemented and verified locally: the home page is rebuilt around the current product, the login form offers phone/SMS or allowlisted email/code, admins can attach emails to allowlist entries, the research desk is decluttered, and Insights has a 大V速报 tab fed by the same influencer snapshot as the desk.

## What Changed

- **Email allowlist login.**
  - An allowlist entry can now carry a phone number, an email, or both. The email is stored in `cloud_web_user_external_state.email_address` (unique index) with identity `domestic_invite`.
  - Admin create (`POST /api/public/admin/invites`) accepts `phone_number` and/or `email_address`.
  - New `POST /api/public/admin/invites/{user_id}/email` attaches, replaces or clears an entry's email. It is audited as `update_email` (the CHECK constraint is migrated idempotently in `ensure_schema`) and does not count against the five-per-day create limit.
  - Admin email edits reject duplicates, refuse to rewrite Stripe purchase identities, and refuse to clear the last login method.
- **Non-disclosing email send.** For sign-in, `/auth/email/send` validates the address shape, rate-limits, then returns a uniform acceptance and issues + sends the code in the background (same pattern as SMS). Revoked entries get no code. The Stripe-intent path keeps its synchronous behaviour. `ensure_international_email_user` no longer flips an allowlisted identity to `international_email` (this was reachable by an unauthenticated Checkout request once domestic entries carry emails).
- **Sender.** `email_verification_sender_from_config` uses the configured `email` block (`account_id`, `from_address` = `no-reply@hone-claw.com`, `from_name`, `api_base`, token from `HONE_CLOUDFLARE_EMAIL_API_TOKEN`) first, and falls back to the legacy `HONE_CLOUDFLARE_ACCOUNT_ID` / `HONE_EMAIL_FROM` env trio. The verification email has a proper HTML layout plus an English line.
- **Frontend auth.** `PublicLoginForm` has a 手机号 / 邮箱 switch, remembered per device. The SMS captcha stays on the phone path only. The admin allowlist panel has optional phone and email fields, an email column, and inline per-row email editing.
- **Insights.** A third tab, 大V速报 (`?view=influencer`), renders `CommunityInfluencerFeed`, which is mounted only when the tab is opened. The influencer dashboard was refactored into a shared `createInfluencerDigestView` and `InfluencerTimeline`, so the research-desk sheet and Insights render one timeline. The official feed's cursor, unread state and edge path are untouched.
- **Research desk.**
  - Sections replace filter tabs: 今日研究 / 产业研究 / admin-only 内测板块. Every product uses one card grammar.
  - The per-card 未发布 badges are gone.
  - Management (usage + allowlist) moved to a separate `?group=admin` view.
  - The guide is folded at the bottom as a per-section reference.
  - The 大V速报 card links to Insights.
  - Shared-shell fix: on mobile, the workspace content no longer reserves the header height a second time. This removes an empty band on every workspace page.
- **Home page.**
  - New `CONTENT.landing` (zh/en), replacing the stale `hero` / `trust` / `cases` copy and the dead `video` / `capabilities` / `community` / `repo` trees.
  - Sections: hero with a production chat capture, proof strip, four product tiles, assistant, research + Insights (generated desk capture plus the production mobile capture), hone-quant (backtest and disclaimer kept), B&M community, principles, final CTA.
  - Nav adds 研究 and renames 社区 to 洞察. The dead footer `#` community links now point to Discord, 知识星球, `/plan` and Bilibili, and the broken README install anchor now points to `docs/wiki.md`.
  - 15 retired images (5.9 MB) were removed.
- **Page titles.** `entry.tsx` drops the static `<title>` before mount. Before this, the static `index.html` title shadowed every page's `<Title>` site-wide. `index.html` meta now uses the new positioning.

## Verification

- `cargo test -p hone-memory web_auth -- --include-ignored`: 33/33 against local PostgreSQL, including 3 new email-allowlist tests.
- `cargo test --no-fail-fast -p hone-web-api -p hone-memory -p hone-core`:
  - `hone-web-api`: 392 passed.
  - `hone-memory`: 67 passed.
  - `hone-core`: 206 passed, 1 failed. The failure is pre-existing and unrelated: `config_example_avoids_stale_config_knobs` expects `README_EN.md` to mention `llm.providers.openrouter.api_key/api_keys`, which the README rewrite in `83f0929a` removed.
- The CHECK-constraint migration was checked manually against a legacy table in a scratch schema.
- `cargo check --workspace --all-targets --exclude hone-desktop --exclude hone-user-app`: passes. rustfmt is clean on all changed Rust files.
- `bun run test:web`: 665 pass. `bun run typecheck` and `bun run build:web:public` both pass.
- Regressions: `test_navigation_responsiveness_contract`, `test_research_curation_contract`, `test_design_system_contract`, `test_billing_contract` and `test_community_forum_research_boundary` all pass.
- Browser verification with a local mock API: desk (admin and reader views, management view, allowlist panel email editing), Insights influencer tab via deep link, login switch, and home page. Each was checked on desktop/mobile, light/dark, and zh/en for the home page.
- Not verified: a real Cloudflare email send from production config, and the Playwright e2e suite (`e2e/public-sms-login.spec.ts` already expects outdated hint text).

## Risks / Follow-ups

- Production needs `email.account_id`, `email.from_address` and `HONE_CLOUDFLARE_EMAIL_API_TOKEN` resolvable on the web-api host. Otherwise email login returns 503; phone login is unaffected. After deploy, send one test code to an allowlisted email.
- Verification codes share Cloudflare's daily sending quota with push email.
- `/roadmap` still shows v0.12.4-era content (17 skills, 7 channels) and was not redesigned in this pass.
- `README_EN.md` should regain the OpenRouter key-pool line so the hone-core config test passes again.
- Fix `e2e/public-sms-login.spec.ts`: its hint text was already out of sync.

## Next Entry Point

Review the working tree diff, commit, and deploy web-api and the public bundle together, since the frontend calls the new admin email endpoint. After deploy, sign in with an allowlisted email end to end.

## Rollout readiness follow-up — 2026-10-12

- status: in_progress; user authorized production deployment after review.
- Fixed stale email issuance/verification writes restoring an administrator-changed address. Verification now uses a locked compare-and-exchange, including revocation and single-consumption behavior. PostgreSQL auth suite: 34/34.
- Restored README_EN OpenRouter configuration documentation and incorporated the existing `943dc2bd` streaming test fixture correction. SMS browser fixtures now use the current ToS version and billing response shape.
- Production inspection confirmed `42ed74f5` is live. Its published branch has been integrated so this rollout preserves the query pool and synchronous bridge repair.
- Web 669/669, typecheck/public build and Worker typecheck/45 tests passed before integration. The finance static contract has the identical ten failures on unchanged `83f0929a`; it is a documented existing baseline, not new application behavior. Integrated verification and deployment evidence follow below.
