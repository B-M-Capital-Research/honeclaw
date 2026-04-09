# ADR 0002: ACP-Aligned Agent Runtime Refactor

Date: 2026-03-17
Updated: 2026-04-09
Status: Accepted
Owner: shared
Related docs: `docs/decisions.md`, `docs/current-plans/acp-runtime-refactor.md`, `docs/archive/index.md`
Supersedes: N/A
Superseded by: N/A

## Context

- The existing `AgentSession` unified part of the session lifecycle, but the execution path was still split: `run_blocking` used the generic agent path, and `run_gemini_streaming` used a Gemini-specific streaming branch.
- That structure caused provider-specific branching to spread into the channel layer, Web SSE, prompt assembly, and session compression, and adding `opencode` would have turned `AgentSession` into a new branching hub.
- The existing system prompt re-injected dynamic time, year, `session_id`, and summary on every turn, which hurt large-prefix cache reuse.
- The existing session history encoded the summary as a special `system` message, which made it harder to migrate to a clearer session / message / part model later.
- External references showed that a unified runtime path is feasible:
  - `AionUi` already routes multiple execution backends through a single ACP backend layer
  - `opencode`'s ACP entrypoint and session / tool event model are already built around `opencode acp` and session / message / part persistence

## Decision

- Collapse `AgentSession` into a single `run()` entrypoint; channels, schedulers, and the Web UI all call that entrypoint and no longer branch execution paths by provider at the edge.
- Rename executor configuration from `agent.provider` to `agent.runner` and treat the runner as a first-class runtime concept.
- Converge the internal runtime on ACP semantics with the goal that every runner eventually emits the same session event classes; the Web SSE layer should upgrade to the new runtime event protocol directly.
- Rework prompt assembly into three layers:
  - static system prompt
  - session-fixed context
  - dynamic session context
- Freeze Beijing time once when the session is created and store it in session runtime metadata; later turns reuse that frozen time instead of regenerating the current time.
- Upgrade session storage to versioned JSON v2 and explicitly store:
  - `version`
  - `runtime.prompt.frozen_time_beijing`
  - `summary { content, updated_at }`
- Stop encoding the summary as a fake `system` message; the compressed summary belongs in the explicit summary field.
- Choose `opencode acp` over stdio / JSON-RPC as the production integration path for `opencode`, instead of CLI text parsing or a `serve` compatibility layer.
- This refactor is an intentional breaking change and does not preserve the old config keys, old SSE event semantics, or old session write format as a long-term compatibility surface.

## Consequences

- Existing callers, frontend streaming consumers, config files, and session file formats all need to migrate together
- Prompt-prefix cache hits should become more stable, but large static instructions must stay at the front and mutable content such as summaries must not be pushed back into the static layer
- `opencode_acp` must fail fast before the Rust runner is wired up; it must not silently fall back to another runner, or the incomplete runtime integration will be hidden
- Remaining follow-up work:
  - Real stdio / JSON-RPC implementation for `OpencodeAcpRunner`
  - Runner contract tests
  - An explicit session v1 to v2 migration script and verification

## Verification / Adoption

- The active follow-up for this ADR is tracked in `docs/current-plans/acp-runtime-refactor.md`
- `docs/decisions.md` records the runtime convergence and dynamic plan policy decisions that frame the remaining work
