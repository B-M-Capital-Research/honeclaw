# Public site, Research Workspace, Insights and allowlisted email login

- title: 官网重设计 / Research Workspace 重构 / Insights 同步大V速报 / 白名单邮箱登录
- status: done (deployed 2026-10-12; real email delivery remains unverified)
- created_at: 2026-10-11
- updated_at: 2026-10-12
- owner: Claude (implementation); Codex (readiness fixes and production rollout)
- related_files:
  - Auth: `memory/src/web_auth.rs`, `crates/hone-core/src/cloud_runtime.rs`, `crates/hone-web-api/src/{email_verification.rs,lib.rs,types.rs,routes/public.rs,routes/public_admin.rs,routes/mod.rs}`
  - Frontend auth: `packages/app/src/components/{public-login-form.tsx,public-admin-whitelist-panel.tsx}`, `packages/app/src/lib/{api.ts,types.ts}`
  - Insights: `packages/app/src/components/{community-influencer-feed.tsx,influencer-digest-dashboard.tsx}`, `packages/app/src/pages/public-community.{tsx,css}`, `packages/app/src/lib/public-community-timeline.ts`
  - Research desk: `packages/app/src/pages/public-research.{tsx,css}`, `packages/app/src/pages/public-workspace.css`
  - Home: `packages/app/src/pages/public-home.{tsx,css}`, `packages/app/src/lib/public-content.ts` (`landing`), `packages/app/src/components/public-nav.tsx`, `packages/app/public/home-*.jpg`, `packages/app/index.html`, `packages/app/src/entry.tsx`
- related_docs: `docs/archive/plans/public-site-research-insights-overhaul-2026-10-11.md`, `docs/decisions.md` (D-2026-10-11-01), `docs/invariants.md`, `docs/repo-map.md`
- related_prs: none; application commit `1ca6197c5c9efb1fa4affe6b7a458fd25cacb8fd` pushed to main

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

## Production rollout accepted — 2026-10-12

- status: done / deployed. This section supersedes the earlier local-only status and readiness follow-up; the October 11 review remains historical evidence.
- Application revision: `1ca6197c5c9efb1fa4affe6b7a458fd25cacb8fd`. Both October 11 change sets are committed and on `main`; no release tag was created. Unrelated untracked `hone-quant/` and `output/` were excluded.
- Preserved the live `42ed74f5` dedicated synchronous-query connection repair by merging its published branch. The withdrawn `3e26eb4f` state was not deployed by itself.
- Additional fixes: email challenge issuance/consumption uses transactional compare-and-exchange; the email audit migration resolves its exact table constraint OID before inspecting the definition, avoiding planner evaluation against concurrently dropped test schemas. Added concurrent old-constraint migration/teardown coverage. Renderer salvage uses private temporary directories; company streaming retains a bounded chunk; secret-environment tests run in a child process and payload cleanup asserts the exact file. README and SMS browser fixture baseline issues were corrected.

### Verification and evidence

- Final local workspace Rust tests: **2,909 passed, 0 failed, 121 ignored**. Explicit memory PostgreSQL integration suite: **97 passed** (including email authentication regressions). Final changed-file formatting and workspace compilation passed. Development PostgreSQL was the existing local instance on port 5432 with an explicit non-empty test password, as documented in the report handoff.
- Web **669 passed**, typecheck and clean exact-revision public build passed; community Worker typecheck and **45 tests passed**. Browser fixtures: SMS/ToS/email **3 passed**, PDF/Markdown delivery **6 passed**.
- Linux CI [38204502572](https://github.com/B-M-Capital-Research/honeclaw/actions/runs/38204502572): formatting, compile, Rust tests, explicit PostgreSQL tests and frontend/Worker checks passed. CI overall remains red solely on the **same ten existing finance static assertions** reproduced on clean `83f0929a`; all 25 local regression scripts were executed individually, with 24 passing and that one unchanged baseline failure. This is not an all-green CI claim; no gate was disabled and no content rejection logic was added to make it pass.
- [Runtime build](https://github.com/B-M-Capital-Research/honeclaw/actions/runs/38204510638) and [verified artifact export](https://github.com/B-M-Capital-Research/honeclaw/actions/runs/38205717801) succeeded. Image digest: `sha256:ad3941bd9508851aca2bebdf8adb4da658258cb1481ec0fa89e664256d52f730`; exported archive SHA-256: `6b5a75c61e795620fd6e83c3337e3064fd8be4323d659276469e8d669c4ce63e`.
- Runtime, matching fallback frontend and earnings skill were verified before cutover. Two zero-active-work samples preceded the managed stop/start; existing unit signal/timeout behavior was preserved. These samples are not a lossless quiesce protocol. The independent deployment job completed with acceptance, without rollback.
- The processes owning both HTTP listeners and the CLI supervisor resolve to the candidate bundle. Live HTTP binary SHA-256: `39fcff1fc58da73724bb40d470a51048fda2aba638d5d567bdcf73b2816f2bed`. Exact revision, PostgreSQL/object-storage health, authoritative cloud storage, role `all` and zero local durable dependencies were checked again after the metadata cache interval. Eight concurrent skill list/detail reads passed; earnings-research is enabled and loaded from the system root.
- Live auth probes: malformed email send **400**, unauthenticated admin invites **401**, unauthenticated company-analysis tasks **401**. The sender fallback is configured as `no-reply@hone-claw.com` and the existing credential is present. No real verification email was sent; Cloudflare's send credential did not expose a zone for a read-only domain check, so mail delivery is not claimed verified.
- Cloudflare Pages production deployment `8265365b-2ec6-499c-a12b-4e75a1637d89` succeeded from the same application commit. Chrome verified the new homepage, populated Research sections and the selected Insights 大V速报 tab with live entries. The existing ten-message chat retained the same IDs, order and text through backend and frontend updates.
- Origin fallback entry: `/assets/index-CCgP6Vm2.js`; public Pages entry: `/assets/index-uz4t8qSX.js` (SHA-256 `89a045c87bec4106069bdda345ecc1e5b11d567a47a6a00b48685739ffeb77dc`), replacing public `/assets/index-CqGkMAe9.js`. Pages and local entry JavaScript are identical after normalizing referenced asset hashes. Public chat `/assets/chat-DXr9e1Hs.js` (SHA-256 `c8498df7549a174c95553a3bca83789548b52ea3f49f7ba134090ded7a299764`) likewise matches local chat code except asset references, with `markdown_ready` present. Public security headers retain HSTS, CSP frame-ancestors none, DENY, nosniff and strict-origin-when-cross-origin.
- Operator-retained evidence directory: sibling `honeclaw-artifacts/claude-rollout-20261012/` (outside the repository). It contains final Rust/PG logs, per-script regression results and baseline comparison, exact asset manifests, immutable runtime export, model/renderer smoke evidence, live acceptance JSON and served-asset comparison. Managed-host deployment evidence and restricted environment backup are under `/var/lib/hone/deployments/<application-revision>/`; never publish the environment backup. Runtime artifacts can be regenerated through the documented exact-revision export workflow after the Actions artifact expires.

### Rollback and remaining limits

Restore the verified runtime `42ed74f5` plus the previous frontend/earnings trees `9a2f27c7`, and the restricted pre-deploy environment backup; use the same idle-checked supervised procedure. The old targets and checksum manifests remain intact. The additive `update_email` audit constraint does not require deleting rows for rollback. Roll back Pages to its prior production deployment if reverting the complete user-facing version.

No further code or deployment work remains for the two October 11 batches. Real allowlisted email receipt/login and three complete model report canaries were not run; model connectivity and 32k configuration, renderer/transport, persistence contracts and mocked download flows were verified instead. The known finance static baseline and real-content evaluation remain explicit follow-ups, not claimed successes. See the report handoff for the quality/cost/latency canary scope.
