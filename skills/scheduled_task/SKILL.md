---
name: Scheduled Task Management
description: Register, update, cancel, and enrich user scheduled push tasks, including event-driven reminders for portfolio holdings
when_to_use: Use when the user wants recurring or one-off reminders, scheduled briefings, or event-linked follow-up tasks。触发词：自动化、推送、提醒、播报
aliases:
  - 定时任务
  - 定时推送
  - 推送任务
  - 自动化任务
  - 心跳检测
  - 取消推送
  - 每天提醒
  - 推送时间
user-invocable: true
context: inline
allowed-tools:
  - cron_job
  - portfolio
  - data_fetch
  - web_search
---

## Scheduled Task Management Skill

Users can configure scheduled push tasks. Hone will execute them automatically at the specified time and push the result. There is no fixed per-user task cap; heartbeat tasks count as tasks too, so when a user already runs many overlapping briefings, propose merging them instead of silently adding one more.

### Tool Guide

| Action | Tool call |
|------|---------|
| Add a task | `cron_job(action="add", name="task name", hour=8, minute=0, repeat="daily", task_prompt="...")` |
| List all tasks (heartbeat included) | `cron_job(action="list")` |
| Cancel one task | `cron_job(action="remove", job_id="task id", confirm="yes")` |
| Cancel every scheduled + heartbeat task | `cron_job(action="remove_all")` |
| Update by ID | `cron_job(action="update", job_id="task id", hour=14, minute=43)` |
| Update by name | `cron_job(action="update", name="Musk", hour=14, minute=43)` |
| Keep one task running inside quiet hours | `cron_job(action="update", job_id="...", bypass_quiet_hours=true)` |

`remove` is destructive and runs in two steps: call it first without `confirm` (with `job_id`, or with a `name` keyword when the id is unknown) to get the candidate list, then repeat the call with the exact `job_id` **plus `confirm="yes"`**. A `remove` without `confirm` never deletes anything — do not report the task as cancelled after that first call.

### Parameter Reference

**`repeat` values:**

| Value | Meaning |
|----|------|
| `daily` | Every day |
| `weekly` | Every week (requires `weekday`, where 0 = Monday and 6 = Sunday) |
| `workday` | Weekdays only (Monday through Friday) |
| `trading_day` | Trading days only (excluding holidays) |
| `holiday` | Holidays and weekends |
| `once` | Run once (needs `date="YYYY-MM-DD"`) |
| `heartbeat` | No fixed clock time — the condition is re-checked every 30 minutes |

Pick `heartbeat` only for "tell me when X happens" requests that carry no clock time (a price threshold, an announcement, a filing), and only after the user agrees to a polling task; tag it with `tags=["heartbeat"]`. A conditional request must never be disguised as a `daily` task at an invented hour.

### 时刻优先于条件轮询

用户本轮说出了任何可解析的钟点——`20:30`、`八点半`、`早上 8 点`、`盘前 9:15`、`每周一 9:30`——这条任务就必须建成带 `hour` / `minute` 的常规任务（`daily` / `weekly` / `workday` / `trading_day` / `holiday` / `once`）。此时不得传 `repeat="heartbeat"`，不得带 `tags=["heartbeat"]`，也不得省略 `hour` / `minute`：heartbeat 不校验 hour/minute，省略时两者落 0，用户拿到的就是一条 00:00 的任务，他给的那个时刻已经没了。

- **时刻和条件同时出现时，时刻赢。**「北京时间 20:30 盘前分析」「每天开盘前看看有没有异动」里的条件是任务要做的事，写进 `task_prompt`；不要因为句子里有「有没有」「如果」「盘前」就把整条任务改成轮询。如果用户确实还想要盘中随时提醒，单独问一句，再单独建第二条 heartbeat 任务。
- **只给时段、没给数字**（「盘前」「收盘后」「开盘就推」）时，先自己换算出一个具体钟点再建任务：说明换算依据（美股夏令时 21:30 开盘，盘前推送取 20:45；冬令时顺延一小时），把钟点和依据一起讲给用户，然后按 `trading_day` + `hour` / `minute` 创建。不得因为用户没报数字就退回 `heartbeat`，也不得随手挑一个整点却不说来由。
- 反方向的判断见上一段：完全没有时刻的纯条件请求才走 `heartbeat`，而且要先征得同意。

### 建完回读真正落库的时刻

`add` / `update` 返回 `success=true` 之后，复述用的时刻**必须取自本次返回体里的 `job.schedule.hour` / `job.schedule.minute` / `job.schedule.repeat`**，不能复述你刚才传进去的参数——传参写着 20:30、落库变成 heartbeat 的 00:00，正是这个缺陷的形态，只有读返回值才看得见。返回体没带回 `job` 时，立刻补一次 `cron_job(action="list")` 读回来。

这一句就是文末 Strict Rules 里那条「一句话复述」，只是里面的数字必须来自返回值，并且说到用户能一眼核对的程度：

> 已经建好：每天 20:30 推送美股盘前分析，结果发到当前会话。

- 回读到 `00:00`、而用户并没有要午夜推送时，判定为时刻丢失：当场调用 `cron_job(action="update", job_id="...", repeat="daily", hour=20, minute=30)` 补正，再回读一次确认，然后才回话。不要把 00:00 原样报给用户，也不要只在旁边解释「心跳任务不显示固定时刻」就算完成。
- 用户来问「我设的 20:30 怎么变成 00:00 了」「时间没保存上」时，先 `cron_job(action="list")` 定位这条任务、`update` 改回他要的时刻、回读确认，再解释原因。只解释不修，这一轮不算完成。
- 用户本来就要午夜推送时，`00:00` 是正常值，照常回读即可。

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
- **Resolve every ticker that goes into `task_prompt`** with `data_fetch(data_type="search")` first, and write the confirmed symbol into the prompt verbatim. A wrong or substituted symbol here is burned into every future run: never replace a user-named security with a parent, a brand owner, or a similar-looking code (SanDisk `SNDK` is not Western Digital `WDC`). If resolution is ambiguous, ask before creating the task.
- **Feasibility check before `add`**: run the task's own data path once in this turn (the same `data_fetch` / `web_search` calls the task will make). If the core metric cannot be retrieved now, say so and either narrow the task to what is retrievable or ask the user before creating it — a task that cannot produce its core number will push an empty briefing every day.
- **Never echo raw parameters back to the user.** `repeat`, `heartbeat`, `enabled=true`, `bypass_quiet_hours`, `job_id` and `task_prompt` are call-shape details: report them as natural language ("每 30 分钟检查一次", "已启用", "工作日 09:30", "已豁免勿扰时段") instead.
- When a task is created or changed, restate back in one line **what will be checked, at what local time, how often, and where it will be delivered**, so the user can catch a wrong ticker or a wrong hour immediately.

### Event-Driven Reminder Linkage

If the current context involves the user's holdings or a portfolio-focused scheduled task, proactively check for major events instead of delegating to another skill:

1. Call `portfolio(action="view")` to inspect holdings when the user asks for portfolio-linked reminders or briefings (`view` is the only read action the tool accepts; `get` is rejected)
2. Use `data_fetch(data_type="earnings_calendar")` for near-term earnings, and use `web_search` for other major catalysts such as product launches, FDA decisions, or management events when relevant
3. If a major event is found within the next few days, automatically add a one-time reminder with `cron_job(action="add")`
4. In the user-facing reply, explicitly say that the reminder task has already been scheduled and why
