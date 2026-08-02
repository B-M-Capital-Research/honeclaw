# ADR 0002: ACP-Aligned Agent Runtime Refactor

Date: 2026-03-17
Updated: 2026-08-02
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
- Represent that ownership as `AgentConversationStrategy`: `NativePersistent`, `StructuredReplay`, or `EphemeralCompiledPrompt`. `RunnerConversationInput` is prepared only after the concrete runner is selected, so a native runner cannot accidentally receive the replay fields used by another runner.
- Keep adapter-specific workspace preparation orthogonal to conversation ownership and stream shape. `NativeSkillProjection::CodexWorkspace` is an explicit runner capability; a future native-persistent runner must not inherit Codex skill links or MCP filtering merely because it retains history.
- Take one Beijing clock reading per turn and reuse it across automatic attempts. For a trusted persistent Codex ACP Interactive turn, the current-turn payload is only that time plus normalized current user/attachment content; the native thread owns retained history, tool/MCP lifecycle, and compaction.
- Provision the complete Hone instructions as Codex `developer_instructions` through the adapter-supported `CODEX_CONFIG` process boundary. Every Codex ACP `session/prompt`, including the first prompt and the first prompt after native compaction, contains only the canonical current user turn. Compaction is telemetry and never requests a user-message seed or reseed.
- Bind each native Codex generation to `codex_acp_session_mode=native_turn_v2` and a SHA-256 instruction fingerprint. Legacy or instruction-mismatched metadata deliberately creates a new native session; an exact v2 match resumes and still fails closed if resume fails.
- Before starting a trusted persistent Codex ACP turn, expose each enabled Hone system/custom skill as an individual symlink under the actor workspace's `.agents/skills/`. Codex owns skill discovery and progressive `SKILL.md` loading; Hone MCP remains for live data/action tools, not Codex skill loading.
- Session storage now writes the normalized version-4 user/assistant `content[] + status` model. Hone-managed replay runners may consume it, but Codex native prompts never serialize that local transcript, including during migration or compaction.
- Choose `opencode acp` over stdio / JSON-RPC as the production integration path for `opencode`, instead of CLI text parsing or a `serve` compatibility layer.
- Treat ACP streaming as versioned adapter dialects rather than one byte-identical renderer contract. The current observed profiles are codex-acp `1.1.7` and OpenCode `1.18.11`; each adapter preserves the visible, thought, tool, progress, usage, and final detail that its protocol actually exposes.
- Select the adapter profile from the real connection's matching `initialize.agentInfo.name` and `initialize.agentInfo.version`, not a runner-name constant or a second probe process. A missing or mismatched identity fails before a session. An exact fixture version is `validated`; a newer minor/patch in the same major uses the nearest validated dialect as `compatible_newer` with a warning; an older version, missing/unparseable version, or unknown major fails before `session/new` or `session/prompt`. The Codex CLI companion follows the same exact/same-major/unknown-major boundary around `0.146.0` even though the live adapter initialize remains the stream-dialect authority.
- Persist only sanitized runtime provenance for the last observed Codex/OpenCode profile under the runtime directory. `/api/meta` exposes the Web binary Git SHA, timestamp, profile, bounded build-source kind, binary hash, and observed adapter profiles; an empty profile list means that adapter has not completed initialize in this runtime, not that a version was guessed. Startup logs emit the same bounded build provenance, and adapter selection logs include the detected adapter version, selected dialect/status, and Codex CLI companion version/status.
- Keep stream detail typed through the session boundary. Answer bytes, reasoning deltas, tool status, progress, usage, reset, and terminal state are not interchangeable. Admin Web and full-reasoning direct channels may show sanitized reasoning progress; compact group modes show only a generic analysis signal; OpenAI-compatible output and channels without a safe live-update surface omit it. Final answer semantics do not depend on that presentation choice.
- This refactor is an intentional breaking change and does not preserve the old config keys, old SSE event semantics, or old session write format as a long-term compatibility surface.

## Consequences

- Existing callers, frontend streaming consumers, config files, and session file formats all need to migrate together
- Prompt-prefix cache hits should become more stable, but large static instructions must stay in the native developer layer and mutable content such as summaries must not be pushed into it or into a later user turn
- Persistent Codex Interactive input must not duplicate Harness-owned history/tool/compaction semantics or stable answer/tool-loop contracts; OpenCode and Hone-managed execution paths retain their separately validated context behavior
- A new same-major adapter version is operationally compatible, not validated. Its profile and logs must retain the detected version, baseline dialect, and compatibility status until a real captured fixture promotes it to a new validated dialect.
- Runtime profile files and `/api/meta` must not contain prompt text, credentials, absolute tool paths, user data, or raw protocol payloads.
- Native Codex skill projection must preserve actor-owned `.agents/skills` entries, remove only Hone-managed stale `hone__*` symlinks, and follow the shared skill registry's enabled state
- `opencode_acp` must fail fast before the Rust runner is wired up; it must not silently fall back to another runner, or the incomplete runtime integration will be hidden
- Remaining follow-up work:
  - Continue runner contract coverage and end-to-end ACP behavior alignment
- Revalidate native resume/compaction and stream-event shapes whenever either ACP adapter floor changes

### Validated compatibility matrix

| Runner | Companion floor | Adapter fixture | Selection policy |
| --- | --- | --- | --- |
| `codex_acp` | Codex CLI `0.146.0` with the same exact/same-major/unknown-major policy | codex-acp `1.1.7` | initialize identity must be `codex-acp`; exact validated; same-major newer conservative; older/unknown-major fail closed |
| `opencode_acp` | local provider/model config | OpenCode `1.18.11` | initialize identity must be `opencode`; exact validated; same-major newer conservative; older/unknown-major fail closed |

The fixture version is an external wire sample, not a claim that the two adapters expose identical events.

## Verification / Adoption

- The active follow-up for this ADR is tracked in `docs/current-plans/acp-runtime-refactor.md`
- Versioned captures live in `tests/fixtures/acp/`; runner tests assert capture metadata as well as safely mapped events.
- `docs/decisions.md` records the runtime convergence and dynamic plan policy decisions that frame the remaining work
