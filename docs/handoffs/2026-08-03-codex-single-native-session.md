# Codex Persistent Session Identity And Pre-Prompt Checkpoint

- status: `done`
- created_at: `2026-08-03`
- updated_at: `2026-08-03`
- owner: `Codex`
- related_files:
  - `crates/hone-channels/src/runners/codex_acp.rs`
  - `crates/hone-channels/src/runners/types.rs`
  - `crates/hone-channels/src/execution.rs`
  - `crates/hone-channels/src/agent_session/core.rs`
  - `crates/hone-channels/src/runners/tests.rs`
  - `tests/regression/manual/codex_probe_home.sh`
  - `docs/adr/0002-agent-runtime-acp-refactor.md`
  - `docs/decisions.md`
- related_prs: `none`
- related_docs:
  - `docs/current-plans/acp-runtime-refactor.md`
  - `docs/invariants.md`
  - `docs/repo-map.md`
- verification: focused Codex ACP `1.1.7`, execution checkpoint, native no-auto-retry, and real isolated Codex probes passed; workspace all-target check/test, Web `347`, Edge `45`, CI-safe regressions, shell syntax, changed-file format, and diff checks all passed
- risks: the ACP protocol does not expose an atomic transaction spanning remote `session/new` and local persistence, so a machine failure in the narrow interval after the adapter creates the native session but before the immediate checkpoint can still leave an unreachable orphan. No prompt is sent before persistence, and every recoverable failure after checkpoint reuses the same ID.

## Outcome

The persistent native identity is now the stored `codex_acp_session_id` itself. Instruction fingerprints and mode markers no longer rotate tasks. The execution layer supplies a narrow write-only checkpoint without giving runners read access to `SessionStorage`; Codex invokes it before the first prompt. Existing-ID resume remains fail closed, and native prompt failures are never automatically resent.

## Acceptance Evidence

- The versioned Codex CLI `0.146.0` / codex-acp `1.1.7` stdio boundary observes exactly `session/new -> session/resume -> session/resume` across compact and changed developer instructions, with one native ID and current-turn-only prompts.
- A rejected metadata checkpoint produces no `session/prompt`.
- A simulated adapter crash after prompt dispatch leaves the checkpointed ID available; the next process calls `session/resume`, not `session/new`.
- A forced `session/resume` protocol failure produces neither `session/new` nor `session/prompt`.
- AgentSession keeps bounded automatic retry for replay runners but suppresses it for any runner that retains native history.
- Repository Codex manual probes use a temporary isolated `CODEX_HOME` with only an authentication-file symlink, preventing acceptance sessions from entering the primary desktop task index.

## Follow-up / Rollback

- If the stored native task was explicitly deleted or corrupted, repair/reset the Hone session metadata deliberately; runtime code will not fork automatically.
- If Codex CLI or codex-acp versions change, record a new version-labelled external capture before updating the dialect contract.
- Rollback must revert the checkpoint interface, stable-ID selection, no-auto-retry rule, tests, and documentation together; reverting only the runner would restore the duplicate-task race.
