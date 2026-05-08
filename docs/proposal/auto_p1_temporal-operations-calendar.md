# Proposal: Temporal Operations Calendar for Automation and Push Reliability

status: proposed
priority: P1
created_at: 2026-05-08 23:03:55 CST
owner: automation

## related_files

- `README.md`
- `docs/repo-map.md`
- `docs/invariants.md`
- `docs/current-plan.md`
- `config.example.yaml`
- `crates/hone-tools/src/schedule_view.rs`
- `crates/hone-web-api/src/routes/schedule.rs`
- `crates/hone-web-api/src/routes/task_runs.rs`
- `crates/hone-web-api/src/routes/notifications.rs`
- `memory/src/cron_job/types.rs`
- `packages/app/src/pages/schedule.tsx`
- `packages/app/src/pages/task-health.tsx`
- `packages/app/src/pages/notifications.tsx`
- `packages/app/src/pages/users.tsx`
- `bins/hone-cli/src/onboard.rs`
- `bins/hone-cli/src/start.rs`

## 背景与现状

Honeclaw 已经从单次聊天扩展成多入口、多渠道、可主动运行的投资助理：

- `README.md` 把 scheduled tasks、position monitoring、company portraits、多渠道接入列为核心卖点。
- `memory/src/cron_job/types.rs` 保存每个 actor 的自定义 cron 任务，并记录 cron 执行历史。
- `config.example.yaml` 的 `event_engine.digest.default_slots`、prefetch offset、quiet hours、price/news poller 等配置决定主动推送节奏。
- `crates/hone-tools/src/schedule_view.rs` 已经把 digest slots、自定义 cron jobs、即时推阈值和 quiet hours 拍平成 per-actor "我的推送日程"。
- `/api/admin/schedule` 和 `packages/app/src/pages/schedule.tsx` 让管理端按 actor 查看静态日程表。
- `/api/admin/task-runs` + `task-health.tsx` 负责运行后的任务健康，`/api/admin/notifications` + `notifications.tsx` 负责投递后的审计。

这说明后端已经有不少事实源，但当前产品形态仍是分散的三块：计划表、运行健康、投递日志。用户和运营者很难在一个地方回答：

- 接下来 24 小时 / 7 天 Hone 会为某个用户运行哪些任务？
- 哪些任务会撞在同一个窗口，哪些会被 quiet hours hold，哪些只会进入 digest？
- event-engine 的 prefetch、digest slot、自定义 cron、heartbeat 和手工 research task 在时间线上是否互相打架？
- 桌面端或 channel listener 当前不健康时，哪些未来任务会受影响？

业界 AI agent 产品正在从"对话框"转向"可解释的执行工作台"：用户不只需要知道 agent 说了什么，还需要知道它未来会代表自己做什么、何时做、失败后在哪里恢复。Hone 已具备主动任务和多渠道投递能力，但还缺少一个面向未来的 temporal control surface。

## 问题或机会

### 问题

1. **未来行为不可见**
   - 现有 `/schedule` 只显示每日时刻和来源，不展开未来 occurrence。
   - 用户看不到一次性任务、weekly 任务、heartbeat、digest prefetch、event-engine poller 与 quiet hours 的具体交互。

2. **排障链路后置**
   - `/task-health` 和 `/notifications` 都偏"发生之后"。
   - 如果未来两个任务会在同一时间挤压同一 channel，或者某任务会持续被 quiet hours hold，系统要等用户抱怨"没收到"后才排查。

3. **用户体验割裂**
   - 用户端只知道聊天和账号额度，管理端才有 schedule / task-health / notifications。
   - 桌面端启动后能看到 channel 数量，但不容易知道"今晚哪些自动化需要这些 channel 在线"。
   - 多渠道用户无法从 channel 内快速问出一张"未来推送日历"并得到与管理端一致的答案。

4. **商业化和留存信号没有被产品化**
   - Hone 的核心差异是"持续守纪律的投资助理"。如果用户看不见未来守护计划，就难以形成信任和付费预期。
   - 自动化越多，越需要一个低焦虑的日历视图来降低"agent 在后台乱做事"的感觉。

### 机会

把现有静态 schedule、运行健康和投递日志升级为一个只读派生的 **Temporal Operations Calendar**：以 actor / workspace / channel 为维度，预测未来窗口内的任务、摘要、主动推送、预取和依赖状态，并在执行后把实际结果回填到同一条时间线。

## 方案概述

新增一个只读的 operations calendar 投影层，不替换 cron、notification prefs、event-engine 或 task run 存储。

核心能力：

1. **未来 occurrence 展开**
   - 输入 actor、时间窗口、timezone。
   - 展开 digest slots、自定义 cron jobs、heartbeat jobs、一次性任务、weekly/workday/trading_day 任务。
   - 标注 quiet hours、bypass、digest-only、immediate、prefetch window、channel target。

2. **冲突和风险预判**
   - 同一 actor / channel / target 在短窗口内任务过密时标注 `collision`.
   - 任务命中 quiet hours 且不 bypass 时标注 `held_by_quiet_hours`.
   - channel 未启用、未运行、最近有失败或 runner readiness 不足时标注 `dependency_at_risk`.
   - digest slot 过近导致 `event_engine.digest.min_gap_minutes` 可能吞掉后续摘要时标注 `digest_gap_suppression`.

3. **执行结果回填**
   - 对已经发生的 occurrence，关联 cron execution record、delivery_log、task_runs。
   - 展示 planned / skipped / running / delivered / failed / held / cooled_down 等状态。

4. **多入口复用**
   - 管理端新增日历/时间线视图。
   - 用户端 `/me` 或 `/portfolio` 提供轻量版"未来守护计划"。
   - 桌面 dashboard/tray 显示下一个关键任务和风险数量。
   - 多渠道通过 `notification_prefs.get_overview` 的后续增强输出同一份 forecast 摘要。

## 用户体验变化

### 用户端

- `/me` 展示未来 24 小时的简版守护计划：
  - 下一次摘要时间
  - 下一次自定义任务
  - 今天是否会被 quiet hours hold
  - 当前剩余额度/权益是否影响自动化
- `/portfolio` 可在持仓旁展示"下一次检查"和"该标的是否有主动提醒覆盖"。
- 文案重点不是技术任务名，而是用户能理解的承诺：`今晚 09:00 汇总持仓事件`、`盘前 08:30 发送摘要`、`TSLA 财报前提醒`。

### 管理端

- 新增 `/operations-calendar` 或扩展 `/schedule` 为 tab：
  - `Overview`: 当前 actor 的未来 7 天时间线
  - `Risks`: 被 quiet hours、channel offline、digest gap、任务过密影响的 occurrence
  - `History Overlay`: 最近 24 小时 planned vs actual
- 每条 occurrence 提供 deep link：
  - cron job -> `/tasks/<id>` 或现有 task detail
  - digest prefs -> notification prefs edit surface
  - channel risk -> `/settings` / channel status
  - failed run -> `/task-health` or `/notifications`

### 桌面端

- Dashboard 顶部除 backend/channel live count 外，增加 `Next automation`：
  - 下一个未来任务
  - 所需 channel
  - 预计是否会被静音或跳过
- Tray 菜单可展示最近 3 个未来任务和一键打开 calendar。
- bundled runtime 启动后，如果未来 12 小时有任务依赖未运行 channel，提示用户修复，而不是等任务失败。

### 多渠道

- 用户在 Telegram/Discord/Feishu/iMessage 中问"今天 Hone 会提醒我什么"时，返回与管理端一致的 compact forecast。
- 群聊场景只展示该 group session 的共享 schedule，不泄露 actor 私有 direct 任务。
- 回复格式复用 `schedule_view::render_overview` 的跨渠道渲染经验，避免 markdown 表格在 Feishu/iMessage 中不可读。

## 技术方案

### 1. 新增派生 forecast 模块

建议新增：

- `crates/hone-tools/src/operations_calendar.rs`
- 或先在 `schedule_view.rs` 下拆出 `forecast` 子模块，等类型稳定后独立。

核心类型示意：

```rust
pub struct OperationsCalendarQuery {
    pub actor: ActorIdentity,
    pub since: DateTime<Utc>,
    pub until: DateTime<Utc>,
    pub timezone: String,
    pub include_history_overlay: bool,
}

pub struct OperationsOccurrence {
    pub occurrence_id: String,
    pub source: OperationsSource,
    pub actor: ActorIdentity,
    pub planned_at: DateTime<Utc>,
    pub local_time: String,
    pub title: String,
    pub channel: String,
    pub channel_target: String,
    pub status: OperationsStatus,
    pub risk_flags: Vec<OperationsRiskFlag>,
    pub related_job_id: Option<String>,
    pub related_event_id: Option<String>,
    pub related_run_id: Option<i64>,
    pub edit_hint: String,
}
```

`occurrence_id` 必须稳定可重算，例如：

- `cron:{actor_key}:{job_id}:{planned_at}`
- `digest:{actor_key}:{slot_id_or_time}:{planned_at}`
- `heartbeat:{actor_key}:{job_id}:{planned_at}`
- `prefetch:{poller}:{slot_time}:{planned_at}`

这个模块只读现有事实源，不新增真相源。

### 2. 复用现有存储和配置

输入事实源：

- `CronJobStorage::list_jobs(actor)` 展开自定义任务。
- `FilePrefsStorage::load(actor)` 读取 timezone、quiet hours、digest slots、immediate policy。
- `config.event_engine.digest.default_slots`、`prefetch_offset_mins`、`min_gap_minutes` 生成默认 digest/prefetch occurrence。
- `task_observer::read_recent_task_runs` 作为历史运行覆盖。
- `cron_job_runs` 和 event-engine `delivery_log` 作为投递覆盖。
- `/api/channels` 的 heartbeat/process scan 结果作为 channel dependency risk 的输入。

非目标：v1 不需要把 forecast 写入 SQLite；未来如果要做 SLA 或通知提前预警，再考虑定期 materialize。

### 3. API 设计

管理端：

- `GET /api/admin/operations-calendar?actor=channel::scope::user_id&since=&until=&history=true`

用户端：

- `GET /api/public/operations-calendar?window=24h`
- public 版本只能读取当前 web session actor，不接受任意 actor query。

多渠道工具：

- 扩展 `notification_prefs.get_overview`，新增 `mode="forecast"` 或单独新增 `schedule_forecast` 工具。
- 工具返回已经按 channel format 渲染好的文本，LLM 只 relay，不重新推理时间线。

### 4. UI 结构

管理端可以先不做复杂月历，采用更适合 agent 运维的纵向 timeline：

- 左侧 actor selector 复用 `ActorSelect`。
- 顶部窗口选择：24h / 7d / 14d。
- 主列表按日期分组，每个 occurrence 展示：
  - planned time
  - source badge
  - title
  - status
  - risk flags
  - deep link
- 右侧 risk panel 汇总可修复项。

这比月历更适合移动宽度和排障，也更容易复用到 desktop。

### 5. 兼容策略

- 不改变 cron、notification prefs、event-engine、delivery_log 的写入语义。
- 不改变 `ActorIdentity` / `SessionIdentity` 边界。
- 不把 public 用户暴露到 admin API 之外；public 只看自己的 actor。
- 所有 forecast 均标注 `generated_at` 和 `timezone`，避免用户误以为是持久承诺。
- 无法展开的 repeat 类型先显示为 `unsupported_schedule` risk，而不是猜测。

## 实施步骤

### Phase 1: Forecast 纯函数与管理端只读 API

- 在 `hone-tools` 增加 occurrence 展开逻辑。
- 覆盖 daily / weekly / once / heartbeat / digest slot。
- 增加 `/api/admin/operations-calendar`，只返回未来 planned occurrence，不做历史覆盖。
- 添加 Rust 单元测试，重点覆盖 timezone、跨日 quiet hours、weekly、once、heartbeat。

### Phase 2: 风险标注

- 增加 quiet hours、bypass、digest min gap、channel disabled/offline、任务过密 risk flags。
- 复用 `/api/channels` 的 process heartbeat 判断 channel dependency。
- 管理端 UI 增加 risk panel 和 deep link。

### Phase 3: History overlay

- 关联 cron execution、event-engine delivery_log、task_runs。
- 最近 24 小时 occurrence 展示 planned vs actual。
- 无法匹配的实际运行单独放入 `unplanned_or_manual` 分组，避免误归因。

### Phase 4: 用户端、桌面端和多渠道入口

- Public `/me` 展示未来 24 小时简版。
- Desktop dashboard/tray 展示 next automation 和风险数量。
- `notification_prefs.get_overview` 增加 forecast 模式，复用同一 formatter。

## 验证方式

- Rust 单元测试：
  - `operations_calendar` occurrence 展开。
  - quiet hours 跨午夜命中。
  - digest slot 使用 actor prefs 覆盖全局默认。
  - once 任务只在指定日期出现。
  - heartbeat 不被误显示为普通每日固定任务。
- API 测试：
  - 缺 actor 返回 400。
  - public API 只能返回当前 session actor。
  - `history=true` 时能匹配 cron execution record。
- 前端测试：
  - timeline 按日期和 planned time 排序。
  - risk flags 文案稳定。
  - 空状态不误导用户以为自动化关闭。
- 手工验收：
  - 创建一个 08:30 digest、一个 08:35 cron、一个 quiet-hours 内任务，确认 UI 标出 collision / held。
  - 关闭 Telegram channel 后，未来 Telegram 任务标出 dependency risk。
  - 执行一次 cron 后，history overlay 从 planned 更新为 delivered / failed。
- 指标：
  - 用户询问"为什么没收到"之前，管理端能预先看到 risk。
  - 自动化相关支持问题减少。
  - Public 用户对"下次提醒/下次摘要"入口的点击率和留存提升。

## 风险与取舍

- **风险：forecast 被误认为强承诺。** UI 必须展示 generated_at、timezone，并注明市场数据、外部 API、channel 状态会影响实际执行。
- **风险：时间规则复杂度上升。** v1 只支持已有 CronSchedule 类型和 digest slots，不引入自然语言日历规则。
- **风险：和 readiness / notifications 页面职责重叠。** Calendar 只回答"未来何时做什么以及风险"，readiness 回答"能力是否可用"，notifications 回答"过去实际送达什么"。
- **风险：trading_day 语义需要市场日历。** 如果当前 CronSchedule 只存 repeat 字符串且没有可靠交易日服务，v1 先按可解释的保守策略标记 `calendar_uncertain`，不假装精确。
- **取舍：先做纵向 timeline，不做复杂拖拽月历。** Hone 的核心场景是排障和信任，不是通用日程软件。
- **取舍：v1 不自动修复。** 只给 deep link 和 edit_hint；修改 cron / prefs 仍走现有工具或 UI，避免新增并发写入面。

## 与已有提案的差异

查重范围：

- `docs/proposal/auto_p0_investment_output_safety_gate.md`
- `docs/proposal/auto_p1_automation_intent_control_plane.md`
- `docs/proposal/auto_p1_cross-company-thesis-map.md`
- `docs/proposal/auto_p1_delivery_decision_loop.md`
- `docs/proposal/auto_p1_evidence_review_queue.md`
- `docs/proposal/auto_p1_investment_context_intake.md`
- `docs/proposal/auto_p1_investment_document_inbox.md`
- `docs/proposal/auto_p1_investment_playbook_launcher.md`
- `docs/proposal/auto_p1_linked-user-workspace.md`
- `docs/proposal/auto_p1_multichannel-render-preview.md`
- `docs/proposal/auto_p1_research_artifact_library.md`
- `docs/proposal/auto_p1_response-feedback-learning-loop.md`
- `docs/proposal/auto_p1_run_trace_workbench.md`
- `docs/proposal/auto_p1_runtime_readiness_matrix.md`
- `docs/proposal/auto_p1_trade_discipline_journal.md`
- `docs/proposal/auto_p1_usage_entitlement_ledger.md`
- `docs/proposals/desktop-bundled-runtime-startup-ux.md`
- `docs/proposals/skill-runtime-multi-agent-alignment.md`

差异结论：

- 不重复 `auto_p1_automation_intent_control_plane.md`：该提案关注用户创建自动化前的 intent preview / confirm / rollback；本提案关注所有已存在自动化的未来时间线和风险预测。
- 不重复 `auto_p1_delivery_decision_loop.md`：该提案解释单条通知为什么发送、降级或过滤；本提案在通知发生前展示未来 occurrence，并在发生后做轻量 overlay。
- 不重复 `auto_p1_runtime_readiness_matrix.md`：readiness 关注 runner、model、channel、capability 是否可用；本提案把可用性投影到具体未来任务，回答"哪些用户计划会受影响"。
- 不重复 `auto_p1_run_trace_workbench.md`：trace workbench 聚合单次 agent run 的执行证据；本提案是跨任务、跨渠道、面向时间的 operations calendar。
- 不重复 `auto_p1_investment_playbook_launcher.md`：playbook launcher 关注标准投资工作流模板的启动；本提案关注启动后的未来计划、冲突和执行结果。
- 不重复 `desktop-bundled-runtime-startup-ux.md`：该历史提案关注桌面 bundled runtime 启动与进程接管；本提案只把未来任务依赖投影到 desktop 状态，不改变 sidecar ownership。
