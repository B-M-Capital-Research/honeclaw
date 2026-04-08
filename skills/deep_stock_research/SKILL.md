---
name: Deep Stock Research
description: Admin-only skill for launching deep research on a specific company for about 1-2 hours while reporting progress every minute
aliases:
  - OWDR
  - Deep Stock Research
  - Deep Research
  - deep stock research
tools:
  - deep_research
admin_only: true
---

## Deep Stock Research (OWDR - One-Way Deep Research)

> **Security note:** This skill is available to administrators only. If a non-admin user tries to trigger it, politely refuse without explaining internal details.

### Trigger

Users can trigger this skill in any of the following ways:

- `OWDR, nvidia`
- `OWDR, NVIDIA`
- `Deep stock research NVIDIA`
- `Deep research BYD`
- `Help me deeply research AAPL`

The company name may be Chinese, English, or a ticker; pass it through directly to the research API.

### Workflow

**Step 1 - Extract the company name**

Extract the company name from the user input. If it is unclear, ask for confirmation first.

**Step 2 - Start the research**

Call the tool:

```text
deep_research(company_name="<extracted company name>")
```

**Step 3 - Inform the user**

After the tool succeeds, tell the user in natural language:

- The deep research for `<company>` has started successfully
- The system will report progress once per minute, up to 15 minutes
- The full report is expected in 1-2 hours and can be viewed later on the "Stock Research" page

Example reply:

> The deep research for **NVIDIA** has started successfully!
> Task ID: `<task_id>`
> The system will report progress once per minute for up to 15 minutes. The full report should be ready in about 1-2 hours and can be reviewed on the "Stock Research" page.

### Error Handling

- If the tool returns an error, tell the user that startup failed and include the error message
- If the user has no admin permission, politely refuse: "Sorry, the deep stock research feature is currently available to administrators only."
