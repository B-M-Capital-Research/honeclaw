# Invariants

Last updated: 2026-04-15

## Source of Truth and Document Priority

- Code and tests take priority over explanatory documents
- `README.md`, workspace manifests, and `config.example.yaml` are the primary implementation-level docs
- `docs/repo-map.md`, `docs/decisions.md`, and `docs/adr/*.md` hold long-lived context
- `docs/current-plan.md` is the dynamic index, and `docs/current-plans/*.md` carries the state of individual tasks that still need tracking; neither file should carry long-term rules

## Definition of Done

- Relevant verification must be completed before a task is closed
- Any affected context documents must be updated before a task is closed
- Cross-module long-lived behavior changes or architecture decisions must be recorded in `docs/decisions.md`, with ADRs added when needed
- At the end of a medium-or-greater task, if the task needs handoff, pause-and-resume support, or explicit retention of follow-up risk, a handoff must be left behind

## Planning and Handoff Constraints

- Parallel tasks must not share the same detailed plan; each task should use its own `docs/current-plans/*.md`
- `docs/current-plan.md` only tracks active task links, status, and links
- `docs/current-plan.md` and `docs/current-plans/*.md` are opt-in: they only record cross-turn, cross-module, behavior / structure / workflow changes, or tasks that need parallel collaboration, handoff, or blocker management
- Low-value one-off tasks such as a single commit / sync / rebase, small script or config fixes, no-behavior-change patches, and pure copy or formatting changes should not be mechanically written into the dynamic plan docs
- For the same topic, prefer reusing the original handoff instead of adding fragmented files on the same day or in the same phase
- A handoff is not a logbook; keep only the goal, result, verification, risk, and unfinished items needed by the next person
- Small pure-execution tasks do not require a new handoff unless the user asks for one, the task needs asynchronous follow-up, or the task changes the workflow / structure / risk surface

## Testing and Script Constraints

- Rust unit tests should live next to the implementation in `#[cfg(test)] mod tests`
- Rust integration tests should live under `tests/integration/`
- CI-safe regression scripts should live under `tests/regression/ci/`
- Regression scripts that depend on an external account, an external CLI, or local machine state should live under `tests/regression/manual/`
- One-off troubleshooting scripts should live under `scripts/tmp/` and must not enter CI
- Open-source collaboration uses layered test coverage: prioritize core Rust logic, key cross-module paths, frontend state / data logic, and keep external integration verification split between local contract tests and manual regression
- Coverage numbers are secondary to behavioral proof; do not introduce a repo-wide `90%+` hard gate or optimize for static UI line coverage
- Default CI proof must cover Rust tests, frontend unit tests, and CI-safe regression scripts
- High-risk logic changes must keep success-path, failure-path, and boundary-condition verification in automated tests whenever the behavior is CI-safe
- Company portraits are document-first assets: `profile.md` plus `events/*.md` is the source of truth, while any parsed metadata or API projections are derived views and must not silently diverge from the Markdown files
- Company portrait docs must live inside the current actor sandbox under `company_profiles/`; do not reintroduce a shared/public portrait directory outside actor-scoped user space
- Default PR / push CI excludes `hone-desktop` from workspace-wide `cargo check` and `cargo test`; desktop sidecar resources and packaging checks belong to dedicated desktop build or release flows instead of the generic logic gate
- Local IDE / dev Rust checks may set `HONE_SKIP_BUNDLED_RESOURCE_CHECK=1` to bypass Tauri bundled sidecar validation while still type-checking `hone-desktop`; this flag is only for development syntax checking and must not replace real desktop packaging validation
- Rust CI / release builds use layered GitHub Actions caching:
  - `Swatinem/rust-cache` is the dependency and `target/` cache layer
  - `sccache` is the compiler object cache layer
  - `main` branch prewarms release-target caches, and tag releases should primarily restore from that warmed cache rather than create fresh tag-scoped caches

## Security and Environment Constraints

- Do not hardcode secrets in docs, scripts, or tests
- Do not add flows that depend on external account credentials to the default CI gate
- Diagnostic scripts should avoid mutating business data
- Treat iMessage features as local privileged capabilities; do not assume they can run in generic environments
- Tool calls must go through a security guard that blocks risky command fragments by default
- 用户可见的长期研究记忆目前只保留 company portraits；不要重新暴露 KB 页面、KB API 或 `kb_search` 类记忆入口
- Company portraits are long-term research assets, not trade execution artifacts: they may store thesis, moat, management, financial quality, capital allocation, valuation frame, risks, and dated event deltas, but they must not evolve into implicit day-trading or automatic recommendation logs
- Company portrait mutations must stay agent-mediated: the Web console may render portrait documents and event timelines, but create / update / append flows should go through the agent's native file operations inside the actor sandbox; the only direct UI mutation allowed is full portrait deletion
- Company portraits should preserve not only the current conclusion but also enough rationale to make future review possible: `profile.md` should keep the current thesis, key operating metrics, valuation frame, risk ledger, and disconfirming conditions, while event docs should retain why the event mattered, supporting evidence / refs, and a compact research trail when no standalone research-notes layer exists yet
- Non-local Web console deployments must enable a Bearer token
- `ChatMode` describes only the message shape (`direct` / `group`) and does not determine session ownership; shared group context must explicitly go through `SessionIdentity`
- `ActorIdentity` and `SessionIdentity` must stay separate: the former is for permissions, quota, sandbox, and private-data isolation, while the latter is for context recovery and session persistence
- Global finance-domain constraints are injected at runtime by `crates/hone-channels/src/prompt.rs`: no stock-picking recommendations, reject non-finance questions, warn users not to blindly follow buy or sell advice, keep greetings short, and require macro / market narrative analysis to distinguish noise from thesis-changing evidence. Do not flip between conflicting narratives on a few days of price action or a single headline unless the prior hypothesis has been explicitly falsified, and do not override these core rules only in a single channel or in a local config.
- Runtime prompt time anchoring is a core behavior contract: Hone must keep the session-provided current time as the source of truth for macro / news / event-driven analysis, must state the current time first on clearly time-sensitive macro answers, and must rewrite relative-time macro searches into absolute-date queries before calling search tools.
- `config.yaml` is the only long-lived user-writable config source
- `data/runtime/effective-config.yaml` is the generated runtime input for child processes, and deleting `data/runtime/` must be a safe runtime reset that does not remove user config
- No steady-state runtime path should read or write legacy `data/runtime/config_runtime.yaml` or sibling `.overrides.yaml` files anymore; the only allowed exception is one-way startup migration that promotes still-missing user settings into canonical `config.yaml`
- `storage.session_runtime_backend` decides the production session read path:
  - `json`: `data/sessions/*.json` is the source of truth
  - `sqlite`: `storage.session_sqlite_db_path` is the source of truth
- During the current rollout, even when `session_runtime_backend=sqlite`, JSON should continue to dual-write as a rollback mirror until SQLite stability is proven over time

## Agent Runtime Constraints

- Use `agent.runner` as the single source for runner selection; channels and the Web UI should not branch `gemini_cli`, `function_calling`, or `codex_cli` execution paths on their own
- `AgentSession` exposes `run()` as the only public entry point; its responsibilities should stay limited to session orchestration, persistence, and listener dispatch. When adding a new execution path, prefer extending the unified runner contract instead of adding a new `run_xxx` branch.
- Shared execution preparation belongs in `crates/hone-channels/src/execution.rs`; session flows and transient task flows should reuse it instead of each path rebuilding tool registry / runner / sandbox wiring on its own
- Runner selection and provider / CLI differences belong in `HoneBotCore::create_runner()` and `crates/hone-channels/src/runners/`; do not reintroduce provider-specific checks in channel entrypoints or `AgentSession`
- Heartbeat and other transient scheduler tasks may bypass transcript persistence, but they must still reuse the same execution-preparation path instead of directly instantiating concrete runner types inside `scheduler.rs`
- A channel actor's local file visibility must stay inside a repo-external actor sandbox; the default root lives under `hone-agent-sandboxes/` in the system temp directory and can be overridden with `HONE_AGENT_SANDBOX_DIR`
- Channel attachments must be written to `uploads/<session_id>/` inside the actor sandbox; do not point the underlying runner `cwd` back at the repo root or at a shared upload directory inside the repo
- 用户可见的运行进度允许保留执行细节，但如果文案中包含 actor sandbox 内的绝对路径，必须改写为相对 sandbox 根目录的路径；sandbox 外绝对路径不得原样透出
- Runner timeout config must stay converged at `agent.step_timeout_seconds` and `agent.overall_timeout_seconds`; do not reintroduce runner-specific timeout knobs in channel/runtime config.
- `gemini_acp` currently uses `gemini --experimental-acp` over stdio / JSON-RPC; startup must verify `gemini >= 0.30.0`. Authentication should prefer the local `gemini-cli` login state; if an environment variable such as `GEMINI_API_KEY` is present, prefer the explicit API key.
- `gemini_cli` in channel runtime must default to sandboxed execution and `approval-mode=plan`; it must no longer default to `yolo`
- `gemini_acp` in channel runtime must also default to `approval-mode=plan`, but it must not force `--sandbox` right now: local Gemini CLI 0.33.1 exits before `initialize` when run with `--experimental-acp --sandbox`
- `codex_acp` currently uses `codex-acp` over stdio / JSON-RPC; startup must verify the local runtime version first. The minimum validated combination is `codex >= 0.115.0` and `codex-acp >= 0.9.5`; otherwise fail fast with a clear upgrade command.
- `codex_acp` and `codex_cli` workspace-write mode may still read repo files outside the sandbox. The repo explicitly allows that for production channels today, so if they are used as the default runner, treat that out-of-bounds read risk as accepted and avoid mixing sensitive files with the channel runtime environment.
- `opencode_acp` currently uses `opencode acp` over stdio / JSON-RPC; the ACP session id must be written back into Hone session metadata so a new ACP session is not created on every turn
- When `agent.opencode.model` / `api_base_url` / `api_key` are empty, Hone must inherit the user's local OpenCode config instead of shadowing `~/.config/opencode/opencode.json` via a separate config home
- If `agent.opencode.model` is non-empty, Hone must call ACP `session/set_model` before `session/prompt`; `agent.opencode.variant` should be appended to `modelId` through the same call (for example `openrouter/openai/gpt-5.4/medium`) instead of relying on temporary selection state in the local opencode UI
- The auxiliary heartbeat / session-compression path must stay separate from the main dialogue model. Prefer `llm.auxiliary` as the source of truth for that OpenAI-compatible background route; `llm.openrouter.sub_model` remains only as a legacy fallback and must not silently replace either the local OpenCode default model or the Hone-selected `agent.opencode.model`
- Before Hone has its own ACP permission negotiation layer, `opencode_acp` must deny one `session/request_permission` request by default and must not silently allow file writes or terminal execution; the channel runtime may inject a minimal custom `OPENCODE_CONFIG`, but it must be a narrow permission overlay rather than a full replacement for the user's local OpenCode config
- The system prompt must stay layered:
  - Static system instructions live in the prefix
  - Session-fixed context is concatenated separately
  - Mutable content such as summaries must not be written back into the static system prefix
- Pre-compact cache stability is a runtime contract: before the next compaction boundary, Hone must not introduce avoidable cache misses by shrinking its own restore window below the active compaction threshold or by injecting turn-specific related-skill hints into the static system prefix. Turn-specific guidance belongs in the current turn input; a post-compact prefix change is expected and acceptable.
- ACP runners must receive the Hone-assembled system prompt explicitly; they must not rely on the underlying CLI discovering `AGENTS.md` or `GEMINI.md` from the repo `cwd`
- Session summaries and compacted restore materials may stay session-scoped, but the displayed current time in prompt session context must be recalculated from the live current Beijing time on every turn; do not reuse stale session creation time as "当前时间"
- Session summaries must be stored in the explicit `summary` field instead of relying on a fake `system` summary message
- SQLite-backed session persistence must preserve the original `session_id` and source JSON semantics; do not silently normalize historical `Actor_*`, `Session_*`, or `User_*` identities during mirror writes or cutover reads
- Heartbeat tasks are first-class cron jobs: they must stay visible in the normal cron list, carry an explicit heartbeat marker, and poll on 30-minute slots without pretending to be a fixed daily time
- Skill runtime stays two-phase:
  - Turn-0 / discovery prompt text may expose only compact skill summaries
  - Full `SKILL.md` bodies must be injected only when `skill_tool(...)` or a user slash skill actually invokes that skill
- Registered skills and enabled skills are separate truth domains:
  - Registration still comes from `skills/`, `data/custom_skills/`, or `.hone/skills/`
  - Runtime activation comes from the global `data/runtime/skill_registry.json` override layer
  - When a skill is disabled there, it must disappear from discover/list/search surfaces and be hard-rejected by slash invocation and `skill_tool(...)` across all channels and runners
- User slash skills and model `skill_tool(...)` calls must share the same prompt-expansion source of truth; do not let the Web/CLI path hand-maintain a divergent skill prompt template
- Invoked skills that materially change the current turn prompt must be recoverable from session metadata after compaction / resume; do not rely only on persisted tool results for skill restoration
- Session restore must respect current skill activation state. If a previously invoked skill is now disabled, its historical prompt may stay in stored metadata, but it must not be re-injected into the live runtime context on restore/compaction recovery
- Skill frontmatter fields such as `allowed-tools`, `model`, `effort`, `context`, `paths`, `hooks`, `arguments`, `script`, and `shell` should be parsed from `SKILL.md` as the source of truth. If runner infrastructure does not yet enforce one of these fields, document that gap instead of silently dropping or misrepresenting it
- Skill script execution must stay opt-in. Loading or disclosing a skill may expand its prompt, but running a declared `script` must only happen through an explicit execution path such as `skill_tool(..., execute_script=true)`
- Invoked skill context and per-turn user input must not be conflated. Only the stable invoked skill prompt may be persisted for restore/compaction; one-shot user supplements after a slash invocation belong only to the current turn
- Session compaction must preserve an explicit boundary model. Post-compact restore should derive context from the last compact boundary forward, with the compact summary stored as part of the message stream rather than treated only as an out-of-band prompt field
- Manual `/compact` must reuse the same boundary-based compaction path as automatic compression, must not consume daily conversation quota, and must not persist the slash command text as a normal user message
- Compact summaries and compact skill snapshots may stay in the stored message stream as restore materials, but primary chat history views should treat them as transcript-only rather than ordinary user-visible dialogue

## Change Constraints

- When directory responsibilities, entrypoints, or major data flows change, update `docs/repo-map.md`
- When workflows, testing contracts, or collaboration rules change, update `AGENTS.md` or this file
- When release or runtime strategy changes, update the matching docs, installer notes, and formula/tap metadata together
- Canonical config and runtime state must stay split:
  - `config.yaml` is the only long-lived user-writable config source
  - `data/runtime/effective-config.yaml` is generated runtime input for child processes
  - deleting `data/runtime/` must be a safe runtime reset that does not remove user config
