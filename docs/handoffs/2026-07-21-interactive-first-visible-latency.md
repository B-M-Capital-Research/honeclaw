# Interactive First-Visible Latency Repair

- title: Interactive first-visible latency repair and production redeployment
- status: in_progress
- created_at: 2026-07-21
- updated_at: 2026-07-22
- owner: Codex
- related_files: `agents/function_calling/src/lib.rs`, `crates/hone-core/src/{agent.rs,tool_effect.rs}`, `crates/hone-channels/src/agent_session/{core.rs,emitter.rs,helpers.rs,restore.rs,tests.rs,types.rs}`, `crates/hone-channels/src/{investment_response_guard.rs,prompt.rs,turn_builder.rs}`, `crates/hone-channels/src/runners/{types.rs,tool_reasoning.rs}`, `crates/hone-web-api/src/routes/{chat.rs,public.rs}`, `tests/regression/ci/test_finance_automation_contracts.sh`
- related_docs: `docs/adr/0004-agent-owned-research-loop.md`, `docs/decisions.md#d-2026-07-21-01-commit-only-delivered-interactive-prefixes-and-remove-synchronous-ttft-work`, `docs/current-plans/ticker-resolution-architecture.md`, `docs/runbooks/backend-deployment.md`
- related_prs: N/A
- verification: complete local gates passed on 2026-07-22; exact build, production health, and exact-query canary pending
- risks: speculative preamble leakage, phantom committed prefixes on cancellation, post-commit side effects, lost follow-up references or invoked-skill prompts across compact boundaries, serial tool execution if the exact canary still exceeds the target, and a deployment that does not use the exact verified revision

## Incident

The production Web turn `大A有没有类似CRWV、Nebius这样的数据中心的标的` entered the backend at `2026-07-21 19:50:19.973` and completed at `19:51:59.427`, roughly `99.45s` later. The user experienced this as almost two minutes before the first visible word, followed by the answer arriving progressively.

Trace attribution separated the delay:

- synchronous pre-run SessionCompress: `14.377s`;
- five model calls: about `76.5s` total;
- ten business-tool calls: about `8.45s` total;
- final model stream: about `39.45s`, buffered by the runner before publication;
- Web SSE and client rendering: immediate after backend emission, so not the bottleneck.

This was not fixed by restarting the existing binary. The running immutable deployment was `100f5608`, and the later `main` revision initially had no TTFT runtime diff relative to that build. Redeployment was deliberately paused until the repair was restored, tested, committed, and built from an exact revision.

## Repair Boundary

- Only a self-contained strict Interactive request whose current input contains a deterministic explicit security seed uses the fast first attempt. Referential turns such as `第二个再详细点` or `继续分析它` keep the full history/compaction path. Actual context overflow still forces compaction/retry; manual `/compact` and noninteractive behavior stay unchanged.
- Fast-path active context and up to four earlier user-only reference utterances come from one durable Session snapshot. Selection uses final durable user-row order, stops before the current row, and filters automation, failed groups, slash commands, summaries, assistants, and tool payloads without a same-text exemption. Previously invoked skill prompts are restored separately from Session metadata, remain subject to current activation, and do not consume the four-user-reference budget.
- Independent candidate work is requested in one model tool-call batch: per-entity DataFetch search and ticker-independent Web/news/filing/industry discovery may be selected together. The executor still awaits tools serially; this change reduces model round trips rather than claiming parallel tool execution. Symbol-dependent quote/profile/financial/ticker-news calls wait for route-bound resolution and cannot guess a ticker.
- Tool-capable drafts remain hidden. On Web only, after the evidence floor, one complete safe canonical data-time/quote-basis header from the same natural final may commit early. Security-clean complete body lines then continue from that final; the current incomplete line stays buffered until completion, and suspicious content falls back to the finalized tail without reset or replay.
- Every header/body delta extends `committed_visible_prefix` only after the unique downstream publication sink accepts it. Cancellation, a closed SSE receiver, ambiguous sinks, or backpressure cannot create bytes that persistence later claims were delivered.
- Any tool call after visible commitment is a protocol error. Persistent/write tools are rejected before observer notification or registry execution. A later failure preserves only the prefix actually delivered and records a partial terminal.
- Separate telemetry records first provider delta, first runner commitment, first successful Web delta send, and final completion.

## Verification

Required before production acceptance:

- focused Agent, channel, and Web unit tests for post-commit side-effect rejection, canonical header/body streaming, unsafe-line fallback, cancellation/rejected/closed-receiver phantom-prefix prevention, one-snapshot restore, invoked-skill restoration, self-contained fast-path gating, cross-boundary reference selection, compact-summary load elision, and partial persistence;
- finance static contract suite plus complete workspace Rust/Web/Worker/CI-safe gates;
- formatting and diff checks;
- exact commit pushed to `main`, immutable `target/deploy-<sha>` build, binary/skill/soul/config/public-asset manifest verification;
- zero-active-chat SIGINT restart from the repository root with cloud authority, PostgreSQL, object storage, ports, process cwd/executable, and anonymous auth boundaries healthy;
- fresh actor replay of the exact incident query, recording first `assistant_delta`, `run_finished`, event counts, first visible line, exact visible/persisted bytes, two-row history, and active chats returning to zero.

## Rollback

Keep `target/deploy-100f5608` intact. If the new process fails health, correctness, one-output/history, or side-effect checks, drain active chats, SIGINT the new CLI supervisor, and restart the previous immutable build with the same reviewed repository-root environment. Whole-final buffering is an acceptable temporary rollback; phantom prefix recording, post-commit tool execution, synchronous first-attempt compaction, or a second synthesis/publication gate is not.

## Handoff State

Implementation and complete local verification are complete:

- `cargo check --workspace --all-targets --exclude hone-desktop --exclude hone-user-app` passed;
- `cargo test --workspace --all-targets --exclude hone-desktop --exclude hone-user-app` passed, including Agent `109/109`, Channels `662/662`, and Web API `128/128` plus two credentialed tests ignored by contract;
- Web tests passed `280/280`; Public Community Edge typecheck and tests passed `45/45`;
- finance static contracts passed `39/39`; complete `tests/regression/run_ci.sh`, formatting, shell syntax, and `git diff --check` passed;
- independent integration review found no remaining deployment blocker after the byte-stable prefix, ACK rejection/cancellation, late-tool, prompt-marker, invoked-skill, and referential-fast-path fixes.

Exact commit/build, restart, health, and canary evidence will be appended after production acceptance. The umbrella ticker plan remains `in_progress` after this latency phase because the separately tracked scheduler `800G` / `NAND` / `AST` / `SEC` entity-guard P2 is still open; do not archive the plan when this deployment completes.
