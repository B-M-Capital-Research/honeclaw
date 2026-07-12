# Public Chat Native Runner Streaming

- title: Public Chat Native Runner Streaming
- status: in_progress
- created_at: 2026-07-12
- updated_at: 2026-07-12
- owner: Codex
- related_files:
  - `crates/hone-llm/src/provider.rs`
  - `crates/hone-llm/src/openai_compatible.rs`
  - `crates/hone-llm/src/openrouter.rs`
  - `agents/function_calling/src/lib.rs`
  - `crates/hone-channels/src/runners/tool_reasoning.rs`
  - `crates/hone-web-api/src/routes/chat.rs`
  - `packages/app/src/pages/chat.tsx`
  - `docs/decisions.md`
  - `docs/repo-map.md`
- verification:
  - LLM stream event parsing and tool-call assembly tests
  - Function-calling final-answer stream and fallback tests
  - Web API SSE listener tests
  - Frontend stream batching tests, full frontend and affected Rust suites
  - Authenticated production public-chat stream timing probe
- risks:
  - Tool-call delta arguments arrive fragmented and must be assembled by index without corrupting parallel calls.
  - Internal reasoning blocks must never enter user-visible stream output.
  - Streaming failure before the first visible delta may fall back safely; failure after visible output must not duplicate the final answer.

## Goal

Make public chat replies progressively visible from the actual model stream while preserving actor isolation, tool-loop correctness, final transcript persistence, and Codex ACP compatibility.

## Todo

- [x] Confirm the live runner and locate the non-streaming boundary.
- [x] Add a provider-neutral structured stream contract for content, reasoning, tool calls, and usage.
- [x] Implement native tool-call streaming for configured OpenAI-compatible providers with non-stream fallback.
- [x] Emit sanitized final-answer deltas from the function-calling runner and preserve one final persisted message.
- [x] Batch frontend delta rendering without replacing the in-thread assistant card.
- [ ] Add regressions, update architecture/decision/handoff docs, archive the plan, deploy, and verify production.
