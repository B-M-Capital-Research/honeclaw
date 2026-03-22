---
name: Scheduled Task Management
description: Register, update, cancel, and list user scheduled push tasks
tools:
  - cron_job
---

## Scheduled Task Management Skill

Users can configure scheduled push tasks. Hone will execute them automatically at the specified time and push the result. Each user can have up to **5** scheduled tasks.

### Tool Guide

| Action | Tool call |
|------|---------|
| Add a task | `cron_job(action="add", name="task name", hour=9, minute=30, repeat="workday", task_prompt="...")` |
| List all tasks | `cron_job(action="list")` |
| Cancel by ID | `cron_job(action="remove", job_id="task id")` |
| Cancel by name | `cron_job(action="remove", name="Musk")` |
| Update by ID | `cron_job(action="update", job_id="task id", hour=14, minute=43)` |
| Update by name | `cron_job(action="update", name="Musk", hour=14, minute=43)` |

### Parameter Reference

**`repeat` values:**

| Value | Meaning |
|----|------|
| `daily` | Every day |
| `weekly` | Every week (requires `weekday`, where 0 = Monday and 6 = Sunday) |
| `workday` | Weekdays only (Monday through Friday) |
| `trading_day` | Trading days only (excluding holidays) |
| `holiday` | Holidays and weekends |
| `once` | Run once |

**`push_type` values:**

| Value | Meaning |
|----|------|
| `analysis` | Research briefing |
| `portfolio_news` | Portfolio news |
| `earnings_calendar` | Earnings calendar |
| `price_alert` | Price threshold alert |

### Natural-Language Examples

- "Every day at 8 AM" -> `hour=8, minute=0, repeat="daily"`
- "9:30 AM every Monday" -> `hour=9, minute=30, repeat="weekly", weekday=0`
- "9:30 AM on workdays" -> `hour=9, minute=30, repeat="workday"`
- "Alert me when AAPL hits 200" -> `push_type="price_alert", symbols=["AAPL"], threshold=200, direction="above"`
- `task_prompt` should summarize what the scheduled task should do, such as "summarize the latest portfolio updates"

### Strict Rules

- Any add, update, or cancel action **must actually call the `cron_job` tool**; never reply "updated" or "added" without calling the tool
- Updates are **single-step**: call `update` directly, and the tool saves immediately without extra confirmation
- **You must check the `success` field** in the tool response. If it is `false`, do not say the task succeeded; explain the error and retry.
- Prefer `job_id` for exact matching. If `job_id` is unknown, you may pass a name keyword and let the tool find the unique match

### Major Event Linkage

If a scheduled task is a daily pre-market or after-market briefing, and in the current context you discover that one of the user's holdings has earnings or a product launch within the next 3 days:
- You must load `OWALERT` with `load_skill("OWALERT")`
- You must automatically add a one-time reminder task with `cron_job(action="add")` (for example, set `repeat="once"` for earnings day and use a short prompt such as "remind the user that Apple reports earnings after the close today")
- In that day's message, tell the user that the reminder task has already been scheduled
