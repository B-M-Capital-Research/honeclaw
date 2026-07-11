# Public Chat Startup Experience

- title: Public Chat Startup Experience
- status: done
- created_at: 2026-07-12
- updated_at: 2026-07-12
- owner: Codex
- related_files:
  - `packages/app/src/app.tsx`
  - `packages/app/src/pages/chat.tsx`
  - `packages/app/src/pages/public-chat.css`
  - `packages/app/src/lib/api.ts`
  - `crates/hone-web-api/src/routes/public.rs`
  - `docs/repo-map.md`
- verification:
  - 212 frontend tests and typecheck passed
  - 95 Web API tests passed, 2 ignored
  - Public production build passed
  - Desktop and mobile browser startup inspection passed
- risks:
  - Authenticated visual QA requires a real HttpOnly browser session.

## Goal

Make `/chat` enter through one stable HONE shell, fetch bootstrap state once, commit authenticated history atomically, and reserve image layout while media decodes progressively.

## Completed

- [x] Measured and mapped route, authentication, history, push, and image startup phases.
- [x] Added a route-level public chat shell and unified restore visuals.
- [x] Combined authentication and history in one bootstrap endpoint and published ready state atomically.
- [x] Added stable progressive image placeholders for restored message media.
- [x] Added regression coverage for the bootstrap API contract.
- [x] Ran unit tests, typecheck, production build, and desktop/mobile browser verification.
- [x] Updated repository context, handoff, plan archive, and archive index.

