# Public Login Production Hotfix

- title: Public Login Production Hotfix
- status: done
- created_at: 2026-05-13
- updated_at: 2026-05-13
- owner: Codex
- related_files:
  - `packages/app/src/lib/messages.ts`
  - `packages/app/src/lib/public-chat.ts`
  - `packages/app/src/pages/chat.test.ts`
  - `crates/hone-web-api/src/routes/public.rs`
  - `data/runtime/logs/desktop_release_app.log`
- related_docs:
  - `docs/runbooks/desktop-release-app-runtime.md`
- related_prs: N/A

## Summary

Production was rebuilt and switched to the 0.11.2 release app. During the switch, public chat was also patched to tolerate legacy or malformed history rows whose `content` or `attachments` fields are missing, which prevented the user-facing page from crashing with `undefined is not an object (evaluating 'e.split')`.

A follow-up SMS login issue was traced to phone-number normalization: users could enter a `+86...` number while most whitelist rows are stored as local mainland China numbers. Before the fix, those requests were rejected by the whitelist before reaching Aliyun, so no verification SMS was sent.

## What Changed

- Normalized non-string history message content before generating timeline rows.
- Treated missing history attachments as an empty list.
- Filtered malformed public attachment records before rendering.
- Added regression coverage for missing content / attachment history rows.
- Made public SMS login accept both `+86...` and local China phone numbers for whitelist lookup.
- Sent Aliyun SMS verification and verification checks using the local number form expected by the configured China country code.
- Restarted the release app from the rebuilt `.app` bundle with SMS provider credentials supplied through a transient launch env file. The env file was removed after sourcing; do not write credentials into checked-in docs or command examples.

## Verification

- `bun --filter @hone-financial/app test -- chat.test.ts`
- `bun --filter @hone-financial/app typecheck`
- `bun run build:web:public`
- `cargo test -p hone-web-api routes::public::tests::sms_phone_candidates_accept_plus_86_and_local_numbers`
- `cargo test -p hone-web-api aliyun_sms::tests`
- `bash scripts/build_desktop.sh` with `CARGO_TARGET_DIR=/Users/fengming2/Library/Caches/honeclaw/target`
- `curl http://127.0.0.1:8077/api/meta` returned `version=0.11.2`
- `curl http://127.0.0.1:8088/api/public/auth/me` returned expected unauthenticated `401`
- `POST http://127.0.0.1:8088/api/public/auth/sms/send` with a whitelisted `+86...` test number returned `200 {"ok":true}` after the backend normalization fix
- Chrome headless loaded `http://127.0.0.1:8088/chat` without TypeError/pageerror; the only console error was the expected unauthenticated `401`
- `/api/channels` reported `web`, `discord`, and `feishu` running; `imessage` and `telegram` were disabled by config

## Risks / Follow-ups

- SMS send was live-smoked through the production endpoint and Aliyun accepted the request. Full login was not completed because that requires reading the delivered verification code from the target handset.
- Event-engine logs showed OpenRouter credit-related `402` warnings for earnings quality review after restart. They were degraded background enrichment failures, not blockers for web / Feishu startup.

## Next Entry Point

- Runtime status: `curl http://127.0.0.1:8077/api/channels`
- Public user page: `http://127.0.0.1:8088/chat`
- Logs: `data/runtime/logs/desktop_release_app.log`
