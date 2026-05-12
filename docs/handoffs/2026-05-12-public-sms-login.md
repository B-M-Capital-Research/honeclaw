# Public SMS Verification Login

- title: Public SMS Verification Login
- status: done
- created_at: 2026-05-12
- updated_at: 2026-05-12
- owner: Codex
- related_files:
  - `crates/hone-web-api/src/aliyun_sms.rs`
  - `crates/hone-web-api/src/routes/public.rs`
  - `crates/hone-web-api/src/routes/mod.rs`
  - `memory/src/web_auth.rs`
  - `packages/app/src/components/public-login-form.tsx`
  - `packages/app/e2e/public-sms-login.spec.ts`
  - `config.example.yaml`
- related_docs:
  - `docs/archive/plans/public-sms-login.md`
  - `docs/repo-map.md`
  - `docs/invariants.md`
- related_prs: N/A

## Summary

Public user login now uses phone number + Aliyun SMS verification as the default path. Existing admin-created web invite users are treated as the whitelist: only active, non-revoked phone numbers can request or verify SMS codes.

## What Changed

- Added Aliyun PNVS `SendSmsVerifyCode` / `CheckSmsVerifyCode` integration with signed RPC requests and local response/signing tests.
- Added `/api/public/auth/sms/send` and `/api/public/auth/sms/login`; old invite/password public auth handlers are no longer routed.
- SMS login records ToS acceptance and creates the existing HttpOnly `hone_web_session` server-side session.
- Replaced the public login UI with one SMS-code form and invitation-only copy pointing to `bm@hone-claw.com`.
- Removed the forced first-login password guard from `/chat` and `/me`; admin invite wording now describes the list as a whitelist while retaining compatibility invite codes.

## Verification

- `cargo test -p hone-memory web_auth::tests::active_invite_user_by_phone_is_sms_login_whitelist`
- `cargo test -p hone-memory web_auth::tests::record_tos_acceptance_updates_public_login_terms`
- `cargo test -p hone-web-api aliyun_sms::tests`
- `cargo check -p hone-web-api`
- `bun run --cwd packages/app typecheck`
- `bun run --cwd packages/app test:e2e -- --project=public public-sms-login.spec.ts`
- Optional live smoke, sends a real SMS when credentials are set: `HONE_ALIYUN_SMS_LIVE_PHONE=13871396421 cargo test -p hone-web-api aliyun_sms::tests::live_send_verify_code_smoke -- --ignored --nocapture`

## Risks / Follow-ups

- Live SMS send/check is intentionally opt-in because it requires real Aliyun credentials and SMS spend. Production must set `ALIBABA_CLOUD_ACCESS_KEY_ID` and `ALIBABA_CLOUD_ACCESS_KEY_SECRET`; optional overrides are documented in `config.example.yaml`.
- Admin APIs and type names still use `invite` internally for compatibility. A later cleanup can rename the surface more deeply once migration risk is lower.
- The test phone `13871396421` must exist as an active web whitelist user before live SMS testing.

## Next Entry Point

Start with `crates/hone-web-api/src/aliyun_sms.rs` for provider behavior, `crates/hone-web-api/src/routes/public.rs` for auth flow, and `packages/app/src/components/public-login-form.tsx` for the UI.
