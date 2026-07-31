# ADR 0002: ACP-Aligned Agent Runtime Refactor

Date: 2026-03-17
Updated: 2026-07-31
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
- Rework prompt assembly around explicit ownership boundaries:
  - static Hone system instructions
  - Hone-managed history/session context for runners that do not own a native thread
  - current-turn dynamic facts
- Take one Beijing clock reading per turn and reuse it across automatic attempts. For a trusted persistent Codex ACP Interactive turn, the current-turn payload is only that time plus normalized current user/attachment content; the native thread owns retained history, tool/MCP lifecycle, and compaction.
- Send the complete static Hone system prompt at the native Codex seed boundary (first prompt and first successful prompt after native compaction), not on ordinary `session/resume` turns.
- Before starting a trusted persistent Codex ACP turn, expose each enabled Hone system/custom skill as an individual symlink under the actor workspace's `.agents/skills/`. Codex owns skill discovery and progressive `SKILL.md` loading; Hone MCP remains for live data/action tools, not Codex skill loading.
- Session storage now writes the normalized version-4 user/assistant `content[] + status` model. Codex can use the local transcript for one-time native-session initialization/migration, while ordinary resumes do not replay it.
- Choose `opencode acp` over stdio / JSON-RPC as the production integration path for `opencode`, instead of CLI text parsing or a `serve` compatibility layer.
- This refactor is an intentional breaking change and does not preserve the old config keys, old SSE event semantics, or old session write format as a long-term compatibility surface.

## Consequences

- Existing callers, frontend streaming consumers, config files, and session file formats all need to migrate together
- Prompt-prefix cache hits should become more stable, but large static instructions must stay at the native seed boundary and mutable content such as summaries must not be pushed back into that static layer
- Persistent Codex Interactive input must not duplicate Harness-owned history/tool/compaction semantics or stable answer/tool-loop contracts; OpenCode and Hone-managed execution paths retain their separately validated context behavior
- Native Codex skill projection must preserve actor-owned `.agents/skills` entries, remove only Hone-managed stale `hone__*` symlinks, and follow the shared skill registry's enabled state
- `opencode_acp` must fail fast before the Rust runner is wired up; it must not silently fall back to another runner, or the incomplete runtime integration will be hidden
- Remaining follow-up work:
  - Continue runner contract coverage and end-to-end ACP behavior alignment
  - Revalidate native resume/compaction event shapes whenever the Codex adapter floor changes

## Verification / Adoption

- The active follow-up for this ADR is tracked in `docs/current-plans/acp-runtime-refactor.md`
- `docs/decisions.md` records the runtime convergence and dynamic plan policy decisions that frame the remaining work
