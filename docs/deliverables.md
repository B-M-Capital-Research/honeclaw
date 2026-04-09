# Deliverables Retention

## Purpose

Define which non-code deliverables should be retained so future agents can understand what changed, how it was verified, and what follow-up constraints remain.

## Keep by default

- Migration notes, rollout notes, and rollback constraints
- Breaking change summaries
- Verification commands and the key result or conclusion
- Regression scripts plus the bug or incident they guard against
- Performance baselines or capacity conclusions when they informed the change
- Important API input/output examples
- Release notes, operational prerequisites, and known risks

## Retention rules

- Prefer links over duplicating large outputs, but keep enough context for a future agent to know why the artifact matters.
- If an artifact is too large or binary, keep a stable path plus a short explanation of how to regenerate or validate it.
- If a task produced no durable deliverable beyond code, say that explicitly in the handoff or delivery note.
