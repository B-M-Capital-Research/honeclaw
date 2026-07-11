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
  - `packages/app/src/components/finance-calendar-message.tsx`
  - `packages/app/src/lib/api.ts`
  - `packages/app/src/lib/public-chat.ts`
  - `crates/hone-web-api/src/routes/public.rs`
  - `crates/hone-web-api/src/routes/history.rs`
  - `docs/repo-map.md`
- verification:
  - 214 frontend tests and typecheck passed
  - 96 Web API tests passed, 2 ignored in the pagination phase
  - Public production build passed
  - 390 x 844 public-entry browser inspection passed without console warnings
- risks:
  - Authenticated production visual QA still requires a real user's HttpOnly session.

## Goal

Make public chat startup, history restoration, media loading, and assistant reply generation behave as one stable IM timeline without duplicate recovery screens, top jumps, layout growth, or a detached composer status strip.

## Completed

- [x] Made route loading and session recovery render the same full-page shell.
- [x] Added stable server pagination over projected public history.
- [x] Replaced client-only slicing with upward cursor loading and viewport anchoring.
- [x] Guaranteed initial bottom positioning after the message viewport mounts.
- [x] Reserved calendar preview and action-row height before the image loads.
- [x] Unified thinking, streaming, completion, abort, error, and recovered background work into one in-thread assistant card.
- [x] Preserved optimistic/recovered message identity when final history arrives.
- [x] Removed the composer status and completion strips.
- [x] Added regression tests and ran frontend/backend verification.
- [x] Updated the handoff, repository map, archive plan, and archive index.

