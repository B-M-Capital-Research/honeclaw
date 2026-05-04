- title: Hone Cloud Runner + Web User API Key
- status: done
- created_at: 2026-05-04
- updated_at: 2026-05-04
- owner: Codex
- related_files:
  - crates/hone-channels/src/runners/hone_cloud.rs
  - crates/hone-web-api/src/routes/public.rs
  - memory/src/web_auth.rs
  - packages/app/src/pages/settings.tsx
- related_docs:
  - docs/archive/plans/hone-cloud-runner-api-key.md
  - docs/repo-map.md
  - docs/archive/index.md
- related_prs: N/A

## Summary

Added the visible `Hone Cloud` runner and per-web-user API key support. Desktop settings now show Hone Cloud first, hide legacy `multi-agent`, remove the standalone `codex_cli` card, and keep `Codex ACP` as the Codex option. Existing legacy runner values remain readable by runtime/config code.

## What Changed

- Added `agent.hone_cloud.base_url/api_key/model`, desktop settings read/write support, and `HoneCloudRunner`, which calls the configured service through OpenAI-compatible `chat/completions`.
- Added `/api/public/v1/chat/completions` with `Authorization: Bearer <api_key>` auth, minimal `model/messages/stream` request handling, non-stream OpenAI JSON responses, and OpenAI-style SSE chunks.
- Extended `web_invite_users` with API key hash/prefix/timestamps. Invite creation generates a one-time plaintext key; existing users can generate once or reset from the admin invite table.
- Updated Settings and Dashboard UI labels/content and `docs/repo-map.md`.

## Verification

- `rustfmt --edition 2024 --check` on touched Rust files.
- `cargo test -p hone-memory web_auth -- --nocapture`
- `cargo test -p hone-core agent_runner_kind_keeps_wire_values_and_probe_mapping -- --nocapture`
- `cargo test -p hone-channels resolves_hone_cloud_chat_url_without_duplicate_path -- --nocapture`
- `cargo check -p hone-web-api`
- `cargo check -p hone-desktop`
- `cargo test -p hone-web-api openai -- --nocapture` (compiled test target; no matching tests)
- `./packages/app/node_modules/.bin/tsc -p packages/app/tsconfig.json --noEmit`

## Risks / Follow-ups

- `bun` is not installed in the current shell, so Bun unit tests were not run. TypeScript checked cleanly via local `tsc`.
- Full live smoke still needs a real generated API key and a production/non-looping backend runner config:
  `curl -H "Authorization: Bearer hck_..." -H "Content-Type: application/json" -d '{"model":"hone-cloud","messages":[{"role":"user","content":"hi"}]}' https://hone-claw.com/api/public/v1/chat/completions`

## Next Entry Point

Use `docs/archive/plans/hone-cloud-runner-api-key.md` for implementation intent and the files listed above for follow-up fixes.
