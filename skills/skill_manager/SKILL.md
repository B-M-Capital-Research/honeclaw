---
name: Skill Management
description: Guide users through creating custom skills step by step, and list, edit, or delete custom skills; built-in system skills cannot be changed through chat
tools:
  - skill_tool
---

## Skill Management (skill_manager)

Load this skill when the user says things like "add a skill", "create a skill", "help me write a skill", "manage skills", "list skills", "edit xxx skill", or "delete xxx skill".

---

### Skill Type Reference

| Type | Storage location | Can be modified through chat |
|------|-----------------|------------------------------|
| `system` | `skills/` directory | No |
| `custom` | `data/custom_skills/` directory | Yes |

**System-built skills (`type=system`) can only be maintained by directly editing `skills/<name>/SKILL.md`. Shell scripts also refuse to change system skills, and any chat request to edit or delete a system skill should be politely refused.**

---

### Operation 1: Create a Skill

When the user says "I want to create a skill" or "help me add a skill", start a multi-step guided flow:

#### Guidance Steps (ask one question at a time; do not list all fields at once)

**Step 1 - English ID (`name`)**
- Ask: "What is the English identifier for this skill? It is used for internal recognition and may contain only letters, numbers, and underscores, for example `MY_SKILL` or `news_digest`."
- After the user answers, validate the format with the regex `^[a-zA-Z][a-zA-Z0-9_]*$`. If it fails, ask again.

**Step 2 - Display Name (`display_name`)**
- Ask: "What is the display name of this skill? It can be Chinese if you want, for example: 'Daily News Digest'."

**Step 3 - Aliases (optional)**
- Ask: "Do you have any aliases or keywords that usually trigger this skill? You can provide multiple items separated by commas, for example `news, daily updates`. If not, just skip it."
- If the user says "none" or leaves it blank, set `aliases` to an empty array.

**Step 4 - One-Sentence Description (`description`)**
- Ask: "Please describe the skill in one sentence, for example: 'Automatically fetches financial news every day and generates a summary.'"

**Step 5 - Required Tools (`tools`, optional)**
- Ask: "Which tools does this skill need? Choose from the list below; you can pick multiple or skip if you are not sure."
- Tool list: `web_search`, `data_fetch`, `portfolio_tool`, `cron_job`, `image_gen`, `x_draft`, `x_publish`, `skill_tool`

**Step 6 - Detailed Instructions (`prompt` / `guide`)**
- Ask: "Finally, describe the execution logic for this skill. When it is triggered, what steps should the AI follow? Please be as clear as possible."
- This is the most important step and helps the user think through the workflow.
- If the user gives a short answer, help fill in the missing details and show a draft for confirmation.

#### After Collection

1. Show the full skill preview in `SKILL.md` format and ask the user to confirm it
2. After confirmation, call:
   ```text
   skill_tool(action="add", name="...", display_name="...", aliases=["..."], description="...", tools=["..."], prompt="...")
   ```
3. If the result returns `success=true`, tell the user the skill has been added and explain how to trigger it
4. If the result returns `success=false`, explain the `error` field and retry

---

### Operation 2: List All Skills

When the user says "list skills" or "what skills do I have":
1. Call `skill_tool(action="list")`
2. Show the result as a clean list with separate sections for system skills and custom skills, for example:

```text
System skills (not editable)
- Stock Research (stock_research)
- Portfolio Management (portfolio_management)
- ...

My custom skills
- Daily News Digest (news_digest) - aliases: news, daily updates
- (none)
```

---

### Operation 3: View Skill Details

When the user says "show me xxx skill":
1. First call `skill_tool(action="list")` to confirm that the skill exists
2. Then call `load_skill(skill_name="xxx")` to load the details
3. Show: name, aliases, description, required tools, and a summary of the execution logic

---

### Operation 4: Edit a Custom Skill

When the user says "modify xxx skill" or "edit the description / prompt of xxx":
1. First use `skill_tool(action="list")` to confirm that the skill is `type=custom`. If it is `type=system`, refuse politely and explain why
2. Ask which fields the user wants to change (`display_name`, `aliases`, `description`, `tools`, or `prompt`)
3. After collecting the new values, call:
   ```text
   skill_tool(action="update", name="xxx", <only the fields that need to change>)
   ```
4. Confirm that the update succeeded

---

### Operation 5: Delete a Custom Skill

When the user says "delete xxx skill":
1. First use `skill_tool(action="list")` to confirm that the skill is `type=custom`. If it is `type=system`, refuse politely and explain why
2. Ask for a second confirmation: "Are you sure you want to delete the `xxx` skill? This cannot be undone."
3. After the user confirms, call:
   ```text
   skill_tool(action="remove", name="xxx")
   ```
4. Confirm that the deletion succeeded

---

### Strict Rules

- **For any add, edit, or delete action, you must actually call `skill_tool`. Do not say the task is complete without calling the tool.**
- **System skills (`type=system`) cannot be modified or deleted through `skill_tool`.** Any such attempt must be clearly refused.
- When adding a skill, `name` must be unique. If the same name already exists, ask the user whether to rename it or convert the request into an update
- You must check the `success` field in the tool response; if it is `false`, never tell the user the operation succeeded
