# ADR 0001: Repository Context Contract for LLM-Assisted Development

Date: 2026-03-07
Status: Accepted

## Context

- The repository already has `AGENTS.md`, but its content mostly covers test organization and CI/CD contracts
- The repo still contains historical documentation, and a new session or a new model can easily mistake outdated explanations for the current implementation
- The project has both a Rust workspace and a Bun / SolidJS frontend with multiple channel entrypoints, so chat history alone is not enough for reliable handoff

## Decision

- Maintain a clearly partitioned set of context files in the repository:
  - `AGENTS.md`: stable collaboration rules and definition of done
  - `docs/repo-map.md`: structure map and key entrypoints
  - `docs/invariants.md`: source-of-truth priority and hard-to-break constraints
  - `docs/current-plan.md`: active task index
  - `docs/current-plans/*.md`: per-task plan state
  - `docs/decisions.md` and `docs/adr/*.md`: long-lived decision records
  - `docs/handoffs/*.md`: one-off task handoffs
- Include context updates in the definition of done instead of relying on the model to remember them
- Make it explicit that historical docs are background only and not the implementation source of truth

## Consequences

- New sessions have a shorter ramp-up path, but these documents need ongoing maintenance
- If module boundaries change without updating the repo map, the docs will drift again
- Cross-module or long-lived changes require the extra work of recording a decision
