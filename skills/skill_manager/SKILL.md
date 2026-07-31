---
name: Skill Management
description: Design, inspect, create, update, and validate Hone skills, including Codex-native discovery through workspace .agents/skills and compatibility with legacy Hone runners.
when_to_use: Use when the user wants to create a skill, inspect or update an existing skill, migrate old skill metadata, or understand how Hone exposes skills to Codex and other runners.
allowed-tools:
  - discover_skills
  - skill_tool
user-invocable: true
context: inline
---

## Skill Management (skill_manager)

Manage the source skill, not a generated projection of it.

## Runtime Contract

1. Built-in sources live in `skills/<id>/SKILL.md`; custom sources live in `data/custom_skills/<id>/SKILL.md`; legacy project-local sources may live in a closer `.hone/skills/<id>/SKILL.md`.
2. Trusted persistent Codex ACP workspaces expose enabled Hone skills as individual symlinks under `<actor workspace>/.agents/skills/`.
3. Codex performs native progressive disclosure: it starts with each skill's name, description, and path, then reads the complete `SKILL.md` only when the task activates that skill.
4. Do not copy skill bodies into each Codex user turn and do not call Hone MCP skill-loading tools merely to activate a skill.
5. Non-Codex and strict legacy runners retain Hone's `discover_skills` / `skill_tool` bridge. `load_skill` remains a compatibility shim only.

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
script: optional default script path like scripts/run.sh
shell: optional shell hint
---
```

Notes:

- `allowed-tools` replaces legacy `tools` as the main runtime field.
- `context` should usually be `inline`; use `fork` only when the skill should run in an isolated child runner.
- `paths` hides the skill from the default listing until the active task touches matching files.
- `script` declares the default executable entrypoint inside the skill directory. Native Codex may run a trusted bundled script directly; legacy Hone runners may use `skill_tool(..., execute_script=true)`.
- Keep the Markdown body task-oriented and ready to inject as prompt text.

## How To Inspect Skills

On native Codex:

1. Use the native skill list already supplied by Codex.
2. Match from `name` and `description`.
3. Read the selected skill's `SKILL.md` from its disclosed path.
4. Read bundled scripts or references only when the task needs them.

On a legacy Hone runner without native skill discovery:

1. Use `discover_skills(query="...")` to select the skill.
2. Use `skill_tool(skill_name="<skill id>")` to load its full prompt.

## How To Create Or Update Skills

1. Collect the intended skill id, description, trigger conditions, and whether users should be able to invoke it directly with `/<skill-name>`.
2. Write or update the source `SKILL.md`; never edit the actor-workspace symlink as if it were an independent copy.
3. Keep the body concrete: trigger rules, required steps, tool usage expectations, and refusal/verification constraints.
4. Validate native discovery with Codex `skills/list` or a real Codex ACP turn. Validate the legacy bridge separately only when that runner remains in scope.

## Strict Rules

- On native Codex, do not call `discover_skills`, `load_skill`, or `skill_tool` just to discover or load a skill.
- Preserve actor-owned entries in `.agents/skills`; Hone only owns its `hone__*` links.
- Do not teach the deprecated `skill_tool(action="add" | "update" | "remove")` CRUD workflow.
- Do not rely on `load_skill` as the main user-facing path.
- If a skill is path-gated, mention that it may stay hidden until matching files are involved.
- If runtime enforcement is not implemented for a field such as `hooks` or strict tool scoping, say so plainly instead of pretending it is active.
