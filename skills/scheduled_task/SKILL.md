---
name: Scheduled Task Management
description: Register, update, cancel, and enrich user scheduled push tasks, including event-driven reminders for portfolio holdings
when_to_use: Use when the user wants recurring or one-off reminders, scheduled briefings, or event-linked follow-up tasks
user-invocable: true
context: inline
allowed-tools:
  - cron_job
  - portfolio
  - data_fetch
  - web_search
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

### Event-Driven Reminder Linkage

If the current context involves the user's holdings or a portfolio-focused scheduled task, proactively check for major events instead of delegating to another skill:

1. Call `portfolio(action="get")` to inspect holdings when the user asks for portfolio-linked reminders or briefings
2. Use `data_fetch(data_type="earnings_calendar")` for near-term earnings, and use `web_search` for other major catalysts such as product launches, FDA decisions, or management events when relevant
3. If a major event is found within the next few days, automatically add a one-time reminder with `cron_job(action="add")`
4. In the user-facing reply, explicitly say that the reminder task has already been scheduled and why
