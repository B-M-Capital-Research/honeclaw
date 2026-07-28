# Whop 购买邮箱真实投递 Handoff

- title: Whop 购买邮箱真实投递
- status: in_progress
- created_at: 2026-07-28
- updated_at: 2026-07-28
- owner: Codex
- related_files:
  - `.env.example`
  - `crates/hone-web-api/src/email_verification.rs`
  - `crates/hone-web-api/src/lib.rs`
  - `crates/hone-web-api/src/routes/whop.rs`
- related_docs:
  - `docs/current-plans/whop-email-delivery.md`
  - `docs/runbooks/whop-hone-activation.md`
  - `docs/decisions.md`
- related_prs: none; source is published directly to `main`

## Summary

HONE now has a concrete Cloudflare Email Sending REST implementation for Whop
purchase-email verification. It remains inactive when all three runtime
variables are absent and refuses partial/invalid configuration at startup.
The owner approved Workers Paid, the sending domain and scoped token are
configured, and controlled delivery/browser acceptance passed. The working
tree is published directly to `main` without a release tag. Exact commit
`482c34d5` is now deployed in production with the current `ws_...` signing
contract and complete Cloudflare/Whop runtime configuration. The only remaining
acceptance item is a real non-owner Whop buyer completing the same production
challenge from purchase through inbox code entry.

## What Changed

- Added a Cloudflare sender that posts one-recipient text/HTML verification
  messages with an account-scoped bearer token.
- Required delivered, queued, or non-empty provider message-ID acceptance,
  rejected permanent bounces, and kept provider bodies, codes, emails, and
  tokens out of errors. The message-ID fallback covers the observed Beta
  response that delivered successfully while omitting delivery arrays.
- Added all-or-nothing runtime configuration through
  `HONE_CLOUDFLARE_ACCOUNT_ID`, `HONE_CLOUDFLARE_EMAIL_API_TOKEN`, and
  `HONE_EMAIL_FROM`.
- Updated Whop Standard Webhooks verification to use the complete current
  `ws_...` secret as the HMAC key and intentionally reject legacy
  `whsec_...` credentials.
- Injected the sender during Web API startup while preserving the existing
  fail-closed unconfigured path.
- Updated the repository map, activation runbook, and long-term provider
  decision.
- User-approved Workers Paid purchase succeeded and Billing reports the plan
  active. The Email Sending domain reports `Enabled`, DNS `Configured`, and
  reputation `Healthy`.
- Cloudflare created the `cf-bounce` MX/SPF/DKIM records plus DMARC; independent
  public DNS queries returned the expected records.
- Created an account-scoped token with only `Email Sending: Edit`. The real
  value exists only in the Git-ignored local `.env` with mode `0600`.
- Cloudflare Activity Log shows two controlled verification messages as
  `Delivered`.
- The user designated a real inbox in the private task conversation for final
  acceptance. Do not copy that address into committed files, logs, screenshots,
  or unrelated messages.

## Verification

- `cargo test -p hone-memory web_auth`: 27 passed.
- `cargo test -p hone-web-api email_verification`: 6 passed.
- `cargo test -p hone-web-api whop`: 2 passed.
- `cargo check -p hone-web-api --all-targets`: passed with one pre-existing
  unused-function warning.
- `bun run test:web`: 309 passed.
- `bun run typecheck:web`: passed.
- `workers/public-community-edge`: typecheck passed; 45 tests passed.
- `bash tests/regression/run_ci.sh`: all CI-safe regression suites passed.
- `git diff --check`: passed.
- `https://hone-claw.com/activate/whop`: `200`.
- `https://origin.hone-claw.com/api/public/auth/me`: expected unauthenticated
  `401`.
- Cloudflare dashboard: domain `Enabled`, DNS `Configured`, reputation
  `Healthy`.
- Public DNS: expected MX, SPF, DKIM, and DMARC present.
- Cloudflare Activity Log: two controlled sends reported `Delivered`.
- The owner later returned a received verification code, confirming real inbox
  receipt. The code value was not persisted or copied into repository
  artifacts.
- Browser isolated acceptance:
  `/activate/whop` → send success → `/me?checkout=success`; membership state,
  renewal period, masked email, and Whop management link rendered correctly.
- Gmail connector was disconnected and Chrome was logged into a different
  mailbox, so the designated inbox was not read. The login half used an
  equivalent known challenge in the isolated SQLite acceptance database; it
  did not mutate production membership. The owner-provided real code arrived
  after the isolated runtime had been cleaned up, so it was not replayed.
- `cargo test -p hone-web-api whop`: 2 passed on exact `482c34d5`; valid current
  signing, legacy-secret rejection, and raw-body tamper rejection are covered.
- `cargo test -p hone-web-api email_verification`: 6 passed, and
  `cargo check -p hone-web-api --all-targets` passed with one pre-existing
  unused-function warning.
- Immutable production package `target/deploy-482c34d5` contains five runtime
  binaries and 498 runtime payloads. All manifest entries verified; manifest
  SHA-256 is
  `e09f7716a0a07f5c2e9fbe4195cbdc0de1474afb62a6da77d37e3b5aee91a518`.
- Production was switched after two consecutive zero-active-chat checks.
  Startup assembled the Cloudflare sender; PostgreSQL and R2 are authoritative
  and healthy, local durable dependencies are zero, and ports `8077/8088`
  remain available.
- Both local and public no-side-effect signed probes returned
  `200 {"ignored":true,"ok":true}`. Reusing the signature with a modified body
  and omitting signature headers returned `401`; no membership was written.
- Public `/`, `/chat`, `/roadmap`, and `/activate/whop` returned `200`; anonymous
  auth returned JSON `401`; the unknown-email response remained the uniform
  `200`; security headers remained intact.
- Cloudflare token verification returned HTTP `200` with `success=true`.
- Three post-cutover probes found PostgreSQL/R2 healthy, zero active chats, and
  exactly one live Feishu process. The Web process remained alive beyond the
  prior supervisor self-exit window.

## Risks / Follow-ups

- This is a direct `main` source publication, not a formal release, and must
  not create a tag.
- Final production acceptance still needs a real non-owner Whop purchase and
  same-challenge code entry; local real inbox receipt itself is confirmed.
- The ignored local `.env` does not travel with Git. Host migration, secret
  rotation, or supervisor working-directory changes must re-inject the complete
  `.env.example` contract through approved secret management.
- Because the webhook secret appeared in the private task conversation, rotate
  it when operationally convenient and update the ignored runtime environment
  before another controlled restart.
- Never paste the token into chat, logs, screenshots, committed files, shell
  history, or a supervisor command line.
- The working tree also contains the pre-existing untracked `.idea/` directory;
  it is unrelated and must remain untouched.

## Next Entry Point

Use a real non-owner Whop buyer for `/activate/whop` → purchase-email challenge
→ same inbox code → `/me` production acceptance. Then cover cancel, expiry,
repurchase, and Discord role lifecycle. If the secret is rotated first, follow
`docs/runbooks/backend-deployment.md` for the same zero-active-chat controlled
restart and repeat the signed no-side-effect probe.
