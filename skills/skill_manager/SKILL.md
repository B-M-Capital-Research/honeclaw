---
name: Skill Management
description: Design, inspect, create, and update Hone skills using the Claude Code-style runtime contract.
when_to_use: Use when the user wants to create a new skill, inspect an existing skill, migrate old skill metadata, or understand how Hone skills are disclosed and invoked.
allowed-tools:
  - discover_skills
  - skill_tool
user-invocable: true
context: inline
---

## Skill Management (skill_manager)

Use this skill when the user asks to add, create, edit, inspect, migrate, or align a Hone skill.

The runtime contract is now:

1. Skills live in `skills/<name>/SKILL.md`, `data/custom_skills/<name>/SKILL.md`, or a closer `.hone/skills/<name>/SKILL.md`.
2. The model sees a compact listing first.
3. The full skill body is only injected into the current turn when `skill_tool(skill_name="...")` or a user slash command like `/<skill-name>` is invoked.
4. `load_skill` is only a compatibility shim. Do not teach it as the primary workflow.

## Frontmatter Contract

Prefer this frontmatter schema:

```yaml
---
name: Human readable name
description: One-line description
when_to_use: Brief trigger guidance
allowed-tools:
  - skill_tool
user-invocable: true
model: optional model override
effort: optional effort override
context: inline
agent: optional agent hint
paths:
  - src/**/*.rs
hooks: {}
arguments: []
shell: optional shell hint
---
```

Notes:

- `allowed-tools` replaces legacy `tools` as the main runtime field.
- `context` should usually be `inline`; use `fork` only when the skill should run in an isolated child runner.
- `paths` hides the skill from the default listing until the active task touches matching files.
- Keep the Markdown body task-oriented and ready to inject as prompt text.

## How To Inspect Skills

When the user asks what skills exist or which skill fits a task:

1. Call `discover_skills(query="...")` with the user's task or keyword.
2. Summarize the relevant skills from the returned metadata.
3. If one skill should actually be used for the current task, call `skill_tool(skill_name="...")` so the full skill prompt is expanded for this turn.

When the user asks to inspect a specific skill in detail:

1. Use `discover_skills(query="<skill name>")` to confirm the match if needed.
2. Call `skill_tool(skill_name="<skill id>")`.
3. Explain the resolved metadata and the injected prompt body, not a hand-written summary that drifts from the source.

## How To Create Or Update Skills

When the user wants to create or edit a skill:

1. Collect the intended skill id, description, trigger conditions, and whether users should be able to invoke it directly with `/<skill-name>`.
2. Write or update the actual `SKILL.md` file with the new frontmatter schema.
3. Keep the body concrete: trigger rules, required steps, tool usage expectations, and refusal/verification constraints.
4. If you created or changed a skill that should be runnable immediately, validate it by invoking `skill_tool(skill_name="<skill id>")` and checking the rendered prompt.

## Strict Rules

- Do not teach the deprecated `skill_tool(action="add" | "update" | "remove")` CRUD workflow.
- Do not rely on `load_skill` as the main user-facing path.
- If a skill is path-gated, mention that it may stay hidden until matching files are involved.
- If runtime enforcement is not implemented for a field such as `hooks` or strict tool scoping, say so plainly instead of pretending it is active.
