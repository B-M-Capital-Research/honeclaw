# Public Chat Native Runner Streaming

- title: Public Chat Native Runner Streaming
- status: done
- created_at: 2026-07-12
- updated_at: 2026-07-12
- owner: Codex
- related_files:
  - `crates/hone-llm/src/provider.rs`
  - `crates/hone-llm/src/openai_compatible.rs`
  - `crates/hone-llm/src/openrouter.rs`
  - `agents/function_calling/src/lib.rs`
  - `crates/hone-channels/src/run_event.rs`
  - `crates/hone-channels/src/runners/tool_reasoning.rs`
  - `crates/hone-web-api/src/routes/chat.rs`
  - `packages/app/src/pages/chat.tsx`
  - `packages/app/src/lib/public-chat.ts`
- related_docs:
  - `docs/decisions.md#d-2026-07-12-03-stream-public-replies-from-the-active-safe-runner`
  - `docs/handoffs/2026-07-12-public-chat-startup-experience.md`
- verification:
  - `cargo test -p hone-llm`: 13 passed
  - `cargo test -p hone-agent`: 7 passed
  - `cargo test -p hone-channels`: 500 passed
  - `cargo test -p hone-web-api --lib`: 101 passed, 2 credentialed tests ignored
  - `bun run test:web`: 218 passed
  - `bun run typecheck:web`: passed
  - `bun run build:web:public`: passed
  - `cargo check --workspace --all-targets --exclude hone-desktop --exclude hone-user-app`: passed
  - Cloudflare Pages check succeeded; production public chat chunk contains `assistant_reset`
  - local/public-origin/Worker auth probes returned expected JSON `401`; `/chat` returned `200`
- risks:
  - A stream transport failure after visible output is not retried, by design, to prevent duplicate text or duplicate tool execution.
  - Chrome had no authenticated HONE cookie, so production verification did not fabricate a session or submit an SMS login; the next normal signed-in turn remains the hands-on timing check.

## Goal

Make public chat replies progressively visible from the actual model stream while preserving actor isolation, tool-loop correctness, final transcript persistence, and Codex ACP compatibility.

## Completed Scope

- [x] Confirmed the live public runner and located the non-streaming function-calling boundary.
- [x] Added a provider-neutral structured stream contract for content, reasoning, tool calls, and usage.
- [x] Added native tool-call SSE for generic OpenAI-compatible and OpenRouter providers with pre-stream key fallback.
- [x] Added sanitized function-calling deltas, fragmented parallel tool assembly, and transient reset semantics.
- [x] Kept final persistence as one normalized assistant turn and retained non-native provider compatibility fallback.
- [x] Batched frontend deltas by animation frame and kept reset/error states in the same assistant card.
- [x] Updated decisions, repository map, handoff, archive index, and deployment evidence.

## Risks / Open Questions

No implementation blocker remains. Authenticated browser timing should be observed on the next normal user turn because production credentials were deliberately not extracted or synthesized during automated verification.
