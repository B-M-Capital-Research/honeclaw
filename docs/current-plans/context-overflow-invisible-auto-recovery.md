# Invisible Context-Overflow Auto-Recovery

- title: Eliminate user-visible context-window failures and recover automatically
- status: in_progress
- created_at: 2026-07-26
- updated_at: 2026-07-26
- owner: Codex
- related_files: `agents/function_calling/src/lib.rs`, `crates/hone-core/src/`, `crates/hone-channels/src/agent_session/`, `crates/hone-channels/src/runtime.rs`, `tests/regression/ci/test_finance_automation_contracts.sh`
- related_docs: `docs/bugs/feishu_direct_compact_retry_still_cannot_answer_new_topic.md`, `docs/decisions.md`, `docs/invariants.md`, `docs/repo-map.md`, `docs/adr/0004-agent-owned-research-loop.md`
- verification: pending focused Agent/Channel regressions, complete repository gates, exact immutable deployment, and fresh long-session production replay
- risks: recovery must not replay mutations, drop the current user intent, fabricate omitted tool facts, change the established finance answer format, or create a second visible answer

## Goal

Interactive users must never receive implementation-facing context-window, automatic-compaction, `/compact`, absolute-path, token-limit, or “open a new session” instructions. A read-only turn that outgrows the model request must automatically reduce current-turn tool evidence and historical context, then let the same Agent complete one normal answer. Preserve the existing finance first line and answer-section contract.

## Scope

- Reproduce the exact 2026-07-26 Web incident for `最近AI概念股票疯狂回调，未来一个月结合宏观，及重要事件，重要财报等分析未来一个月可能的走势。`.
- Separate current-turn growth from durable-session growth. The incident executed nine successful quote/calendar/Web calls before the first overflow; forced Session compact then re-executed another nine tools and overflowed again, proving history-only compaction was the wrong recovery boundary.
- On a recoverable read-only Agent turn, replace an oversized raw current-turn tool transcript with one deterministic bounded evidence copy and continue with `tools=[]`. Keep full tool results in audit/response state; explicitly mark omitted fields unavailable to the model.
- Bound that copy in fixed linear passes. A result far beyond its per-call budget goes directly to a valid-JSON preview; recovery must never remove one array item and reserialize the complete payload repeatedly. The first production replay reached this path with a large earnings-calendar result and proved that the former tail-pop loop could monopolize one Web worker at 99% CPU for more than four minutes.
- If a runner overflows before usable current-turn tool evidence exists, retain forced Session compaction as the first fallback, then retry from current-turn-only context with no compact summary, restored historical tool protocol, or invoked-skill snapshots.
- Remove the hard-coded user-visible overflow terminal. Exhausted infrastructure failure may remain a sanitized technical failure, but no context/compact/path/session-instruction language may cross a channel or enter assistant history.
- Preserve execute-once behavior: no automatic rerun after an uncertain persistent/write-capable call, and no compact recovery may claim an unconfirmed mutation succeeded.

## Validation

- Agent regressions:
  - oversized read-only tool results trigger one bounded same-Agent tools-disabled continuation;
  - the compact evidence payload is valid JSON, bounded, source-labelled, and does not contain the full oversized sentinel;
  - a 12,000-row earnings-calendar payload is compacted within a five-second test ceiling without iterative array-tail deletion;
  - the answer keeps the exact configured finance prefix and emits once;
  - a second provider failure remains unsuccessful without re-executing tools or exposing context-window language.
- Channel regressions:
  - first overflow force-compacts and retries;
  - a second overflow uses current-turn-only context and succeeds;
  - current-turn-only recovery excludes compact summary, old assistant/tool protocol, and invoked-skill snapshots;
  - persistent/uncertain operations are not retried;
  - failed history and channel error events contain no context/compact/internal path instructions.
- Static CI rejects reintroduction of the production fallback symbol, requires the shared overflow classifier/public sanitizer regressions, and locks both automatic recovery layers. Runtime tests feed the exact legacy sentence and `<absolute-path>/compact` variant through every public response/event/history boundary.
- Run changed-file formatting, workspace check/test excluding Apple clients, Web tests, Edge Worker checks, finance contracts, and complete CI-safe regressions.
- Build the exact pushed commit into an immutable manifest, drain active chats, restart healthy roles, verify cloud/storage/auth/ports, then replay the exact incident wording on a deliberately long canary session. Acceptance requires one successful terminal, no reset/error/internal limit copy, unchanged finance format, byte-identical visible/history content, and active chats returning to zero.

## Documentation Sync

- Update D-2026-07-26-07 in `docs/decisions.md`, plus the runtime flow in `docs/invariants.md` and `docs/repo-map.md`.
- Update the existing compact-retry bug and `docs/bugs/README.md`; do not create a duplicate defect.
- Keep this task in `docs/current-plan.md` while implementation/deployment evidence remains incomplete.
- On completion, write one handoff, archive this plan, update `docs/archive/index.md`, and remove the active-plan entry.

## Risks / Open Questions

- A mechanically bounded tool-result copy must preserve source identity and useful scalar fields without pretending omitted arrays or long text were fully inspected.
- Bounded recovery runs on the request path. Its time complexity and serialized output are both safety properties: no per-element full-payload serialization, unbounded traversal retry, or event-loop monopolization is allowed.
- Current input can itself contain a very large attachment. The Session-level minimal retry must remove old material but cannot silently discard the user's current attachment; attachment-specific truncation remains governed by the existing bounded ingest contract.
- A model/provider whose configured window cannot fit the static system contract plus one ordinary current question is an infrastructure/configuration fault. It must remain sanitized and observable internally, not converted into a business answer or exposed as context-window guidance.
