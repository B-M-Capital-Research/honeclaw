- title: Hone Cloud Runner + Web User API Key
- status: archived
- created_at: 2026-05-04
- updated_at: 2026-05-04
- owner: Codex
- related_files:
  - packages/app/src/pages/settings.tsx
  - bins/hone-desktop/src/sidecar.rs
  - crates/hone-core/src/config/agent.rs
  - crates/hone-channels/src/runners.rs
  - crates/hone-web-api/src/routes/public.rs
  - memory/src/web_auth.rs
- related_docs:
  - docs/handoffs/2026-05-04-hone-cloud-runner-api-key.md
  - docs/archive/index.md

## Goal

Add a visible Hone Cloud runner backed by the public hone-claw.com user service, hide legacy multi-agent / codex CLI choices from the client UI, and issue per-web-user API keys that can authenticate an OpenAI-compatible public chat endpoint.

## Scope

- Add `hone_cloud` config, desktop settings fields, runner creation, and user-facing UI.
- Add Web invite API key generation, reset, one-time plaintext return, and hash-based lookup.
- Add `/api/public/v1/chat/completions` with Bearer API key auth and minimal OpenAI-compatible request/response support.
- Keep legacy `multi-agent` and `codex_cli` runtime compatibility, but remove them from visible settings/dashboard runner options.

## Validation

- Rust targeted tests for config, memory web auth API keys, public API key auth endpoint, and runner request parsing.
- Frontend Bun tests for settings model/content/runner visibility.
- Manual smoke command documented in handoff for curl-based `/api/public/v1/chat/completions`.

## Documentation Sync

- Update `docs/current-plan.md` while active.
- On completion, move this plan to `docs/archive/plans/`, add a handoff, and update `docs/archive/index.md`.
- Update long-lived docs only if the implementation changes public runner/config contracts beyond this feature description.

## Risks / Open Questions

- Streaming OpenAI-compatible SSE should preserve useful deltas without exposing internal tool status as invalid OpenAI chunks.
- API keys must never be persisted in plaintext; only the one-time generate/reset response may include the secret.
