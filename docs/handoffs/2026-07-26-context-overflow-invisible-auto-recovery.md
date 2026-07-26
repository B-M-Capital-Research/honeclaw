# Context-Overflow Invisible Auto-Recovery

- title: Context-overflow invisible auto-recovery
- status: done
- created_at: 2026-07-26
- updated_at: 2026-07-26
- owner: Codex
- related_files: `agents/function_calling/src/lib.rs`, `crates/hone-core/src/context_window.rs`, `crates/hone-channels/src/agent_session/`, `crates/hone-channels/src/runtime.rs`, `tests/regression/ci/test_finance_automation_contracts.sh`
- related_docs: `docs/archive/plans/context-overflow-invisible-auto-recovery.md`, `docs/bugs/feishu_direct_compact_retry_still_cannot_answer_new_topic.md`, `docs/decisions.md#d-2026-07-26-07-make-context-window-recovery-invisible-and-current-turn-aware`
- related_prs: commits `268561c4`, `620391d1`

## Summary

The recurring context-overflow refusal is closed in production. The decisive failure was current-turn tool growth, not only durable chat history: the old recovery compacted history, reran the same read-only research, overflowed again, and persisted a hard-coded `/compact` / new-session instruction. Exact runtime `620391d1` now recovers invisibly while preserving the established finance answer format.

## What Changed

- One shared overflow classifier recognizes provider request/window variants. Public channel error and history boundaries strip raw provider, context, compaction, absolute-path, `/compact`, and new-session guidance.
- A read-only FunctionCalling turn with current tool evidence keeps the original audited results but gives the same Agent a valid-JSON bounded copy and `tools=[]`; tools are not executed again.
- Overflow without usable current evidence first force-compacts once. A repeated overflow retries from current-turn-only context without old assistant/tool protocol, compact summary, or restored skill snapshots.
- Persistent, uncertain, write-capable, send, schedule, delete, and trade operations remain execute-once and are never replayed by this recovery.
- Evidence compaction uses fixed linear work. A far-over-budget JSON value becomes one bounded preview; iterative array-tail removal plus full reserialization is statically and dynamically regression-tested.
- No finance prompt format was changed. The original data-time/market-scope first line and downstream answer structure remain the contract.

## Verification

- Agent `138/138`; Channels `682 passed, 1 host OCR ignored`.
- Workspace check/test passed with the documented Apple-client exclusions.
- Web `294/294`; Edge Worker typecheck plus `45/45`.
- Finance contracts `44/44`; complete `tests/regression/run_ci.sh`; rustfmt, diff, shell syntax, pre-push gitleaks.
- Exact immutable runtime `620391d1f137d433d89a34f944b5076463936d62`: 501 manifest payloads, all hashes verified; Web and Feishu run from that package.
- Fresh exact-query actor `codex-canary-620391d1-context-202607262213`: `104.86s`, eight read-only tools, one start/delta/successful terminal, no reset/error/partial/prohibited guidance, two-row history, visible/history SHA-256 `3d665ee8fe43262e72db2c46e06a63180dc85964548f230966e3e254e8ee295d`, active chats zero.
- During the long replay, ten concurrent health probes returned HTTP 200 in about `1–3ms`; console CPU remained about `0–0.3%`. PostgreSQL, object storage, `8077/8088`, origin/public anonymous auth, and Feishu were healthy.

## Risks / Follow-ups

- The configured Discord credential is still rejected by the gateway. Discord remains offline and isolated; Web and Feishu are healthy.
- `/api/llm-audit` list inspection timed out after the canary. It did not affect chat, history, ports, storage, or process health and is outside this defect; investigate separately if audit-list users report slowness.
- Reopen this exact bug if any public surface again emits context/compaction/new-session guidance, if a read-only overflow re-executes tools, or if recovery monopolizes a runtime worker.

## Next Entry Point

Start with `bounded_context_overflow_tool_evidence`, `AgentSession` overflow recovery tests, finance contract case 44, and the archived plan. Keep user-facing answer-format changes out of this recovery path.
