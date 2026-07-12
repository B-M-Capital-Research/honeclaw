# Public Chat Startup Experience

- title: Public Chat Startup Experience
- status: done
- created_at: 2026-07-12
- updated_at: 2026-07-12
- owner: Codex
- related_files:
  - `packages/app/src/components/public-chat-startup.tsx`
  - `packages/app/src/pages/chat.tsx`
  - `crates/hone-web-api/src/routes/public.rs`
- related_docs:
  - `docs/archive/plans/public-chat-startup-experience.md`
  - `docs/runbooks/backend-deployment.md`
- related_prs:
  - this change set

## Summary

Public `/chat` no longer exposes route loading, authenticated empty chat, restored history, and image arrival as separate visual jumps. One eager HONE shell spans route loading and session bootstrap; auth/quota and history return together and are committed atomically; restored images reserve space and fade in after asynchronous decoding.

## What Changed

- Added a lightweight responsive startup shell to the main entry chunk and removed the generic `Loading…` fallback for `/chat`.
- Added `/api/public/bootstrap`, replacing serial `/auth/me` then `/history` startup requests with one actor-authenticated response.
- Kept the chat shell hidden until both user and history state are ready, preventing the empty-message flash.
- Added stable progressive placeholders and lazy asynchronous decoding for restored inline and attachment images.
- Kept the static HTML HONE marker only until Solid mounts, then removes it before the first application render.

## Verification

- `bun run test:web`: 212 passed.
- `bun --filter @hone-financial/app typecheck`: passed.
- `cargo test -p hone-web-api`: 95 passed, 2 credentialed tests ignored.
- `bun run build:web:public`: passed; chat remains a separate 39 KB gzip route chunk.
- Local public health returned 200; unauthenticated `/api/public/bootstrap` returned expected 401 JSON.
- Browser QA at 390 x 844 and 1280 x 800 confirmed one stable startup shell, clean login landing, and no browser console warnings/errors.

## Risks / Follow-ups

- Authenticated browser QA depends on a real user's HttpOnly session and was proven through the atomic state path and endpoint tests rather than copying production credentials into automation.
- Historical image dimensions are not persisted, so generic media uses a stable 16:10 or 4:3 preview frame rather than its exact natural ratio.

## Follow-up Reopened

Production feedback found that route loading and in-page recovery still differed, the first restored viewport could remain at the oldest loaded item, history pagination was client-only, and the calendar action row appeared only after image load. The task is reopened to move pagination to the public API, enforce a post-mount bottom anchor, and reserve the complete calendar card height.

## Follow-up Completed

- Route Suspense and session recovery now render the same full-page shell with identical localized copy; the normal navigation does not mount during recovery.
- Bootstrap returns the newest 20 projected public messages plus an absolute cursor. Upward scrolling requests older 20-item pages, prepends them, and preserves the viewport anchor.
- Stable IDs include the server's absolute projected-history offset, so page refresh, prepend, and live-tail reconciliation do not reorder rows.
- A post-mount two-frame anchor pins first restore to the newest message instead of the first loaded row.
- Finance-calendar controls remain in layout while the image loads, so successful media decoding changes opacity and availability rather than card height.
- Verification: 214 frontend tests, 96 Web API tests, frontend typecheck, public production build, local 390 x 844 recovery-shell browser QA, and healthy restarted admin/public/channel runtime.

## Next Entry Point

Start with `PublicChatStartup`, `restoreSession()` in `chat.tsx`, and `handle_bootstrap()` in the public Web API. Cloudflare Pages deployment follows `docs/runbooks/backend-deployment.md`.

## Thinking Card Follow-up

Production feedback found that the composer-side `HONE 思考中` strip reads like a system status instead of an assistant reply. This follow-up moves the pending lifecycle into the timeline and preserves one assistant card/message identity until the final answer replaces it in place.

## Thinking Card Completed

- Sending now inserts an empty in-thread assistant card immediately. Its `data-phase` advances through thinking, streaming, and done/error without switching components or replacing the DOM node.
- The detached composer status and transient completion strip were removed. Elapsed time and stop stay inside the active assistant card, while completed turns expose the normal copy/share actions.
- Refreshing during a server-side run appends a `_background` timeline placeholder. The final persisted assistant row adopts that temporary ID before Solid reconciliation, so the same card is edited in place.
- Abort errors now use localized user-facing copy instead of exposing browser exception text.
- Verification: 214 frontend tests, frontend typecheck, targeted lifecycle tests, public production build, diff check, and 390 x 844 public-entry browser QA with no console warnings. Authenticated production sending still requires the user's HttpOnly session for final hands-on acceptance.

## Mobile Gesture And Share Follow-up

- Exported user queries now use one tested layout contract with horizontal, vertical, and text centering inside the dark bubble, matching the modal preview and rasterization source.
- `/chat` applies a runtime viewport lock plus a non-passive multi-touch guard so accidental Safari page pinch cannot leave the conversation enlarged. The finance-calendar lightbox is explicitly allowlisted and continues to use its bounded custom pinch/pan implementation.
- Verification: 216 frontend tests, frontend typecheck, public production build, 390 x 844 share-card visual QA, runtime viewport `scale=1` with `maximum-scale=1` and `user-scalable=no`, and no browser console warnings.

## Native Runner Streaming Follow-up

- Production logs confirmed ordinary Web users were correctly routed away from configured `codex_acp` to the strict actor-bound function-calling runner, but that runner always returned `streamed_output=false`; a representative turn waited about 47 seconds across three model rounds and four tools before sending 1240 characters at once.
- `hone-llm` now exposes structured tool-capable stream events. Generic OpenAI-compatible/Minimax and OpenRouter providers parse actual upstream SSE, assemble fragmented parallel tool calls by index, and only rotate API keys before a stream starts.
- Function calling now streams sanitized final-answer content through the canonical runner/session event path. Cross-chunk `<think>` and tool protocol blocks stay hidden; a preamble followed by a tool call emits `StreamReset`, so the public client clears that temporary text and continues editing the same card. Final session persistence remains one normalized assistant message.
- Public chat handles `assistant_reset`, batches deltas once per animation frame, and keeps failed runs in the card's error phase instead of marking them done.
- Verification: 13 LLM tests, 7 function-calling tests, 500 channel tests, 101 Web API tests with two credentialed tests ignored, full frontend tests, frontend typecheck, public production build, and changed-file format/diff checks.
