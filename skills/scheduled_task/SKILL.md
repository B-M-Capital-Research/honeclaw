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

- **真实工具为唯一权威真相源**：用户要求查看、核对、更新、列出、创建、取消或删除“我的持仓 / 关注列表 / 定时任务 / 心跳任务”时，必须优先调用真实 `portfolio` / `cron_job` 工具；禁止通过沙盒里的 `data/portfolio`、`data/cron_jobs`、`holdings.json`、文件列表、当前工作目录或会话历史自行推断“为空 / 不存在 / 没创建”。
- **取消全部任务**：用户明确说“取消/删除所有定时任务或心跳任务”时，直接调用 `cron_job(action="remove_all")`；这句明确请求本身就是授权，不要再逐条确认、逐条循环删除或只关闭其中一项。
- **自动推送边界与 `automatic_push`**：自动推送不只有定时任务——事件即时推送与每日摘要推送由 `notification_prefs` 管理，删完全部 cron 也不会停。`cron_job` 的 `list` / `remove_all` 结果带有 `automatic_push`：只有 `all_automatic_push_stopped=true` 才可以说“已经没有任何自动提醒”；否则必须逐项说明 `remaining_sources` 里仍会按计划触发的来源，并提示用户可以要求关闭全部自动提醒。禁止在仍有来源时输出“已全部关闭”“不会再有任何推送”“当前没有任何自动任务”这类结论。
- **全量关闭自动提醒**：用户明确说“取消所有自动提醒 / 关闭所有自动推送 / 以后不要自动通知”，或反复表示“关不掉、还在推、明明关了还收到”时，先按 `automatic_push` 核对仍有触发的来源，直接调用 `notification_prefs(action="disable_all")` 一次性收口（同时关闭事件即时/摘要推送并删除全部定时/心跳任务）；禁止只调用 `disable` 后就声称所有自动提醒都已取消。
- **Web 渠道送达边界**：Web 渠道的定时任务结果只保证写入当前 Hone 会话，并在网页在线且 SSE 连接存在时实时追加到页面；当前没有 Web Push / 手机系统通知能力，不要承诺会出现在手机通知中心，也不要引导用户排查手机通知权限；若用户明确需要手机系统级提醒，应说明当前 Web 渠道不支持并建议改用已配置的外部通知渠道。
- **工具不可用时的用户态表达**：如果本轮真实 `cron_job` 工具不可用或调用失败，只能用用户态语言说明“定时任务管理暂时不可用，请稍后再试”；禁止向用户输出 `工具未暴露`、`接口未暴露`、`cron_job / scheduled_task`、`data/cron_jobs`、`cloud_sessions`、`cloud_cron_jobs`、`cloud_cron_job_runs` 或“当前沙盒”等实现细节。
- Any add, update, or cancel action **must actually call the `cron_job` tool**; never reply "updated" or "added" without calling the tool
- Updates are **single-step**: call `update` directly, and the tool saves immediately without extra confirmation
- **You must check the `success` field** in the tool response. If it is `false`, do not say the task succeeded; explain the error and retry.
- Prefer `job_id` for exact matching. If `job_id` is unknown, you may pass a name keyword and let the tool find the unique match
- **Resolve every ticker that goes into `task_prompt`** with `data_fetch(data_type="search")` first, and write the confirmed symbol into the prompt verbatim. A wrong or substituted symbol here is burned into every future run: never replace a user-named security with a parent, a brand owner, or a similar-looking code (SanDisk `SNDK` is not Western Digital `WDC`). If resolution is ambiguous, ask before creating the task.
- **Feasibility check before `add`**: run the task's own data path once in this turn (the same `data_fetch` / `web_search` calls the task will make). If the core metric cannot be retrieved now, say so and either narrow the task to what is retrievable or ask the user before creating it — a task that cannot produce its core number will push an empty briefing every day.
- **Never echo raw parameters back to the user.** `repeat`, `heartbeat`, `enabled=true`, `enabled=false`, `bypass_quiet_hours=true`, `job_id` and `task_prompt` are call-shape details: report them as natural language ("每 30 分钟检查一次", "已启用 / 已停用", "工作日 09:30", "遵守勿扰 / 豁免勿扰") instead.
- When a task is created or changed, restate back in one line **what will be checked, at what local time, how often, and where it will be delivered**, so the user can catch a wrong ticker or a wrong hour immediately.

### Event-Driven Reminder Linkage

If the current context involves the user's holdings or a portfolio-focused scheduled task, proactively check for major events instead of delegating to another skill:

1. Call `portfolio(action="view")` to inspect holdings when the user asks for portfolio-linked reminders or briefings (`view` is the only read action the tool accepts; `get` is rejected)
2. Use `data_fetch(data_type="earnings_calendar")` for near-term earnings, and use `web_search` for other major catalysts such as product launches, FDA decisions, or management events when relevant
3. If a major event is found within the next few days, automatically add a one-time reminder with `cron_job(action="add")`
4. In the user-facing reply, explicitly say that the reminder task has already been scheduled and why
