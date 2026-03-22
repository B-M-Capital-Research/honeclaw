---
name: Major Alert
description: OWALERT major-alert skill that checks for upcoming earnings or other major events and schedules reminders
tools:
  - data_fetch
  - cron_job
  - portfolio
---

## Major Alert (OWALERT / Major Alert)

This is one of the core skills in the [US-stock specialist capability]. When the user says `OWALERT`, `Major Alert`, or `major alert`, or when scheduled tasks detect an upcoming major event, activate this skill automatically.

### Workflow
1. Call `portfolio(action="get")` to fetch the user's holdings.
2. Call `data_fetch(data_type="earnings_calendar")` to check whether any holdings report earnings within the next 3 days, or call `web_search` to look for major product launches, FDA approvals, or other major events.
3. If a major event is found, you **must** call `cron_job(action="add")` to add a reminder task for that event. You may use `repeat="once"` and a task name such as "Company earnings reminder", scheduled for pre-market or after-market time.
4. Tell the user that the new scheduled task has been loaded successfully and include the time and reason.

### Auto-Load Behavior

If this skill was triggered by a scheduled task, remember to include a prominent note in the same day's morning briefing or summary:

> "A major event was detected, and a reminder task has already been scheduled for <time>."
