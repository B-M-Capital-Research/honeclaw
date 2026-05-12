# Public SMS Verification Login

- title: Public SMS Verification Login
- status: archived
- created_at: 2026-05-12
- updated_at: 2026-05-12
- owner: Codex
- related_files:
  - `crates/hone-web-api/src/aliyun_sms.rs`
  - `crates/hone-web-api/src/routes/public.rs`
  - `crates/hone-web-api/src/routes/mod.rs`
  - `crates/hone-web-api/src/types.rs`
  - `memory/src/web_auth.rs`
  - `packages/app/src/components/public-login-form.tsx`
  - `packages/app/src/pages/chat.tsx`
  - `packages/app/src/pages/public-me.tsx`
  - `packages/app/src/lib/api.ts`
  - `packages/app/src/lib/public-content.ts`
  - `packages/app/e2e/public-sms-login.spec.ts`
  - `config.example.yaml`
  - `docs/repo-map.md`
  - `docs/invariants.md`
- related_docs:
  - `docs/handoffs/2026-05-12-public-sms-login.md`

## Goal

Replace public user login with phone number + Aliyun SMS verification code as the primary login path. Existing admin-created invite users remain the whitelist source: a non-revoked invite user phone number is allowed to receive and verify SMS login codes.

## Scope

- Add public API endpoints for sending and checking SMS verification codes.
- Integrate Aliyun PNVS `SendSmsVerifyCode` and `CheckSmsVerifyCode` on the server side.
- Keep current invite-user storage as the whitelist and session owner source.
- Update public login UI from password/invite tabs to one SMS-code flow with invitation-only copy and `bm@hone-claw.com` contact.
- Remove the forced first-login password guard from public chat and account pages.
- Keep existing admin invite management usable as whitelist management.

## Validation

- `cargo test -p hone-memory web_auth::tests::active_invite_user_by_phone_is_sms_login_whitelist`
- `cargo test -p hone-memory web_auth::tests::record_tos_acceptance_updates_public_login_terms`
- `cargo test -p hone-web-api aliyun_sms::tests`
- `cargo check -p hone-web-api`
- `bun run --cwd packages/app typecheck`
- `bun run --cwd packages/app test:e2e -- --project=public public-sms-login.spec.ts`
- Optional live smoke, sends a real SMS when credentials are set: `HONE_ALIYUN_SMS_LIVE_PHONE=13871396421 cargo test -p hone-web-api aliyun_sms::tests::live_send_verify_code_smoke -- --ignored --nocapture`

## Documentation Sync

- Updated `docs/repo-map.md`, `docs/invariants.md`, and `config.example.yaml`.
- Added `docs/handoffs/2026-05-12-public-sms-login.md`.
- Added archive index entry in `docs/archive/index.md`.

## Risks / Open Questions

- Aliyun credentials are environment/configuration concerns and must not be committed.
- The live Aliyun API is opt-in because it requires real credentials and SMS spend.
- Live smoke testing should use `13871396421` only after that phone is present as an active admin-created whitelist user.
