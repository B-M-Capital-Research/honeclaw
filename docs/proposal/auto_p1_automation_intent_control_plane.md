# Proposal: Automation Intent Control Plane

status: proposed
priority: P1
created_at: 2026-05-06 17:02:46 CST
owner: automation

## related_files

- `README.md`
- `AGENTS.md`
- `docs/repo-map.md`
- `docs/invariants.md`
- `docs/decisions.md`
- `docs/current-plan.md`
- `docs/proposal/auto_p0_investment_output_safety_gate.md`
- `docs/proposal/auto_p1_delivery_decision_loop.md`
- `docs/proposal/auto_p1_evidence_review_queue.md`
- `docs/proposal/auto_p1_investment_context_intake.md`
- `docs/proposal/auto_p1_investment_document_inbox.md`
- `docs/proposal/auto_p1_linked-user-workspace.md`
- `docs/proposal/auto_p1_research_artifact_library.md`
- `docs/proposal/auto_p1_run_trace_workbench.md`
- `docs/proposal/auto_p1_usage_entitlement_ledger.md`
- `docs/proposals/desktop-bundled-runtime-startup-ux.md`
- `docs/proposals/skill-runtime-multi-agent-alignment.md`
- `memory/src/cron_job/{types,storage,schedule,history}.rs`
- `crates/hone-tools/src/cron_job_tool.rs`
- `crates/hone-tools/src/schedule_view.rs`
- `crates/hone-web-api/src/routes/{cron,schedule,task_runs}.rs`
- `crates/hone-channels/src/scheduler.rs`
- `packages/app/src/context/tasks.tsx`
- `packages/app/src/pages/{tasks,schedule,task-health}.tsx`
- `skills/scheduled_task/SKILL.md`

## 背景与现状

Hone 已经有一条相当完整的自动化执行链路：

- `CronJobStorage` 按 `ActorIdentity` 保存定时任务定义，支持 `daily`、`weekly`、`once`、`workday`、`trading_day`、`holiday` 和 `heartbeat`，并用 `MAX_ENABLED_JOBS_PER_ACTOR = 12` 限制每个 actor 的启用任务数量。
- `cron_job_tool` 允许 agent 在对话中 list/add/update/remove 任务；删除已经要求先返回候选任务，再显式传入 `confirm="yes"` 才真正执行。
- `memory/src/cron_job/history.rs` 把实际执行记录写入 SQLite，记录 execution status、message send status、delivery detail、response preview 和 error message。
- `crates/hone-tools/src/schedule_view.rs` 已把 digest slots、自定义 cron、quiet hours 和即时推送配置聚合成“我的推送日程”，并复用于 NL 工具与管理端 `/api/admin/schedule`。
- 管理端已有 `/tasks` 任务编辑、`/schedule` 推送日程概览和 `/task-health` 周期任务健康页。
- `scheduled_task` skill 能引导模型创建日常简报、一次性提醒、portfolio 相关事件提醒和 heartbeat 任务。

这些能力说明 Hone 已经可以“让 agent 创建自动化并按时执行”。但从产品架构看，自动化仍是底层 cron job 的直接暴露，而不是一个用户可理解、可预演、可审批、可追责的意图控制面。用户说“每天盘前提醒我跟踪 NVDA 风险”，系统会尽快落成一个 `CronJob`；但用户和管理员很难在保存前看到它会在哪些渠道、哪些时间、遵守哪些 quiet hours、是否会触发 heartbeat 轮询、会消耗哪些额度、失败后如何处理，以及未来如何撤销到旧版本。

当前实现里还有两个直接信号：

- `CronJobData` 已经有 `pending_updates: Vec<PendingUpdate>`，但工具和 Web API 主路径没有把它发展成“待确认变更”产品链路。
- `skills/scheduled_task/SKILL.md` 写着“每个用户最多 5 个 scheduled tasks”，而存储层常量是 12。这不是单纯文案问题，而是自动化能力、用户承诺和执行约束没有统一契约的表现。

## 问题或机会

### 问题

1. 自动化变更缺少一等 intent 对象。
   现在 add/update 直接写入 cron JSON，remove 只有工具层的二次确认。对用户来说，任务创建前没有结构化草稿；对管理员来说，也没有“谁在什么时候通过什么入口把自动化从 A 改成 B”的审计视图。

2. 保存前缺少影响预演。
   `schedule_view` 能在事后展示推送日程，但创建或修改时不能先回答：下一次运行时间是什么、会不会被 quiet hours 吞掉、是否绕过勿扰、目标 channel 是否启用、当前 actor 还有多少任务名额、heartbeat 会以什么频率轮询。

3. Agent 自动写入的信任成本偏高。
   `scheduled_task` skill 要求模型必须实际调用 `cron_job` 工具，且 update 是 single-step。这个执行效率高，但当用户表达含糊、模型理解错时间或任务 prompt 与 schedule 不一致时，用户只能事后发现。

4. 用户端、管理端、桌面端和 IM 端的自动化心智不一致。
   Web 管理端能编辑任务；IM 端通过自然语言创建任务；桌面端负责 bundled runtime 与 channel 进程；schedule 页面展示聚合日程；task-health 展示后台任务健康。这些视图没有围绕同一个“自动化意图和版本”串起来。

5. 自动化很适合商业化和留存，但缺少可解释边界。
   Hone 的高价值不只是单次聊天，而是“持续替我看”。如果用户不能清楚知道系统会在什么时候主动做什么，就会影响付费信任、留存和误推后的恢复体验。

### 机会

新增一个轻量的 Automation Intent Control Plane，不替换现有 cron job 和 scheduler，而是在创建、修改、删除、启停前增加可审计的 intent 草稿、预演和确认层。这样可以把“自动化是 agent 直接改配置”升级为“agent 提出自动化方案，用户或管理员确认后应用”，同时保留已有 cron 执行链路。

## 方案概述

引入 actor-scoped 的 `AutomationIntent`：

- `intent_id`：稳定 ID。
- `actor`：沿用 `ActorIdentity`，不改变权限隔离。
- `source`：`chat`、`public_web`、`admin_web`、`desktop`、`channel_command`、`system_migration`。
- `operation`：`create`、`update`、`delete`、`enable`、`disable`、`clone`。
- `target_job_id`：更新/删除时关联已有 job。
- `proposed_job`：标准化后的 `CronJob` 草稿或局部 patch。
- `before_snapshot`：旧任务快照，用于 diff、回滚和审计。
- `impact_preview`：下一次运行、未来 7 天运行窗口、quiet hours 影响、channel 状态、任务数量限制、可能成本、与 digest/immediate 推送的重叠。
- `status`：`draft`、`needs_confirmation`、`approved`、`applied`、`rejected`、`expired`、`superseded`、`failed`。
- `approval`：确认人、确认入口、确认时间、确认文本或 token。
- `audit`：创建时间、更新时间、应用结果、失败原因。

第一版不需要复杂工作流引擎。它可以先作为 `memory/src/automation_intent.rs` 或 `memory/src/cron_job/intent.rs` 的本地 JSON/SQLite 存储，应用时仍调用现有 `CronJobStorage::{add_job, update_job, remove_job}`。

## 用户体验变化

### 用户端

- 用户在 chat 里要求创建或修改任务时，Hone 先返回一张简洁确认卡：任务名称、时间、频率、渠道、是否受 quiet hours 影响、下一次执行时间、任务内容摘要。
- 用户可以回复“确认”“改成 8:45”“先别开”来 approve、revise 或 reject，而不是让模型一次性直接写入。
- Public `/portfolio` 或未来用户工作台可以展示“自动化草稿”和“最近变更”，让 IM 创建的任务回到 Web 端可见。

### 管理端

- `/tasks` 在当前任务编辑之外增加 intent drawer：显示待确认变更、已应用变更和 before/after diff。
- `/schedule` 不只展示当前日程，也能对一个 draft intent 运行 preview，显示改动后日程会怎么变化。
- `/task-health` 可以从失败执行跳回对应 job，再查看最近一次 intent 变更，判断失败是否由变更引入。

### 桌面端

- Desktop bundled runtime 负责显示本机自动化是否可运行：channel listener 是否在线、backend 是否健康、当前 mode 是 bundled 还是 remote。
- 当用户在桌面 UI 修改任务时，复用同一套 preview/approve，而不是直接绕过 agent 或 cron 工具。

### 多渠道

- Feishu / Telegram / Discord 创建任务时返回渠道原生确认消息；确认 token 只在当前 actor 和短时间内有效。
- 群聊场景下，确认必须绑定触发用户和 `SessionIdentity`，避免其他群成员误确认或取消。
- iMessage 等弱交互渠道可以退化为“回复确认码”，不要求复杂卡片。

## 技术方案

### 1. Intent 存储与数据边界

在 `memory` 中新增 automation intent 存储，优先按 actor 隔离：

- JSON 定义可以放在 `data/automation_intents/automation_intents_<actor_key>.json`，便于本地开发和回滚。
- 如果需要跨 actor 查询和管理端分页，再镜像到 SQLite。
- `CronJobData.pending_updates` 可以迁移为兼容读取：老字段不再扩展新语义，新 intent 存储成为真相源。

所有应用动作仍走现有 `CronJobStorage`，避免第一版同时重写 scheduler。

### 2. Preview 服务

新增 `AutomationPreviewService`，复用现有纯逻辑：

- 调用 cron schedule 判定逻辑计算 next run 和 7 日运行窗口。
- 调用 `schedule_view::build_overview` 生成应用前后的 schedule diff。
- 读取 notification prefs 判断 quiet hours、digest slots、immediate push overlap。
- 读取 channel config / heartbeat registry 给出目标渠道是否可能投递。
- 读取 `MAX_ENABLED_JOBS_PER_ACTOR` 和当前启用数，提前返回 quota 风险。

Preview 结果只做事实解释，不调用模型。

### 3. Tool 与 skill 契约

把 `cron_job_tool` 扩展为两阶段写入：

- `action="propose"`：返回 intent 与 preview，不改 cron job。
- `action="approve"`：应用 intent。
- `action="reject"`：关闭 intent。
- 保留现有 `add/update/remove` 作为兼容路径，但在非 admin 交互中优先由 `scheduled_task` skill 使用 `propose`。

同时更新 `skills/scheduled_task/SKILL.md`：

- 任务数量上限必须从代码或工具响应引用，不再硬编码 5。
- 高影响变更默认先 propose。
- 用户明确说“直接帮我开”时可以在同一轮 propose + approve，但仍要把 preview 和应用结果说清楚。

### 4. API 与前端

新增 API：

- `GET /api/automation-intents?actor=...&status=...`
- `POST /api/automation-intents/preview`
- `POST /api/automation-intents/{id}/approve`
- `POST /api/automation-intents/{id}/reject`
- `POST /api/automation-intents/{id}/apply-dry-run`

前端落点：

- `packages/app/src/context/tasks.tsx` 读取当前 job 的 recent intents。
- `packages/app/src/pages/tasks.tsx` / `TaskDetail` 增加 preview/diff 区域。
- `packages/app/src/pages/schedule.tsx` 支持查看 draft schedule overlay。
- `packages/app/src/pages/task-health.tsx` 从失败 run 链接到 job 与最近 intent。

### 5. 执行历史关联

`CronJobExecutionRecord.detail` 中补充 `last_intent_id` 或 `job_version`：

- 任务执行失败时可以定位它是在某次变更后开始失败。
- 回滚时可以从 `before_snapshot` 恢复。
- 管理端可以展示“应用后首次成功/失败”的结果。

## 实施步骤

### Phase 1: Intent 与 preview 只读骨架

- 新增 `AutomationIntent` 类型和 actor-scoped 存储。
- 实现 `preview_create/update/delete`，不写入 cron job。
- 给现有 `/api/cron-jobs` 创建/更新路径旁路生成 preview，但不改变现有行为。
- 增加单元测试覆盖 next run、quiet hours、limit、heartbeat preview。

### Phase 2: Agent 两阶段提案

- 扩展 `cron_job_tool` 的 `propose/approve/reject`。
- 更新 `scheduled_task` skill，让自然语言创建/修改优先输出确认卡。
- IM 渠道实现最小确认 token。
- 保留 admin bypass 直接写入能力，但记录 synthetic intent。

### Phase 3: 管理端控制面

- `/tasks` 增加 intent timeline 和 before/after diff。
- `/schedule` 增加 draft overlay。
- `/task-health` 关联最近 intent 和 job version。
- 修复 skill 文档与存储上限漂移，增加前端或测试断言防止再次硬编码。

### Phase 4: 回滚与指标

- 支持从 intent `before_snapshot` 一键生成 rollback intent。
- 增加指标：intent approval rate、preview conflict rate、automation edit-to-success rate、post-change failure rate。
- 把高风险自动化变更接入未来的 usage entitlement 或 safety gate，但不在第一版阻塞。

## 验证方式

- 单元测试：
  - `AutomationPreviewService` 对 daily/weekly/once/trading_day/heartbeat 的 next run 计算。
  - quiet hours 场景下 preview 标记 held / bypass。
  - 已有 12 个启用任务时 propose create 返回 limit risk。
  - update/delete intent 保留 `before_snapshot`。
- 工具测试：
  - `cron_job(action="propose")` 不写 cron JSON。
  - approve 后才出现新 job。
  - reject 后不能再次 approve。
  - 兼容旧 add/update/remove 流程。
- API 测试：
  - actor A 不能 approve actor B 的 intent。
  - expired token 不能应用。
  - admin 可查看跨 actor intents，但应用仍保留 actor 审计。
- 前端测试：
  - draft overlay 不改变当前 schedule。
  - before/after diff 正确展示时间、频率、enabled、quiet hours 变化。
- 手工验收：
  - 在 Web chat、Telegram/Discord/Feishu 至少各创建一个任务草稿，确认后能在 `/tasks` 和 `/schedule` 看到一致状态。
  - 修改一个会被 quiet hours 吞掉的任务，确认卡必须明确提示。
  - 任务失败后，`/task-health` 能跳回对应 job 和最近 intent。

## 风险与取舍

- 增加一步确认会降低部分 power user 的速度。取舍是允许 admin 或明确 direct 模式继续快速应用，但默认用户路径以可信度优先。
- Intent 存储可能与 cron job 状态漂移。第一版应用时必须重新校验 `before_snapshot` 或 `job_version`，发现目标已变化则要求重新 preview。
- 群聊确认 token 容易设计过重。第一版只绑定触发用户、actor、session 和短 TTL，不做复杂权限模型。
- 不在第一版引入通用 workflow engine，也不把 digest/event-engine delivery 决策迁进 intent。它只管理用户可创建和修改的 automation job。
- 不直接解决通知质量问题；通知是否该发仍属于 Delivery Decision Loop，本提案只解决“这条自动化为什么存在、改动前会产生什么影响、谁确认了它”。

## 与已有提案的差异

- 与 `auto_p1_delivery_decision_loop.md` 不同：该提案关注一次事件为什么发/不发，以及如何从 delivery log 调偏好；本提案关注 cron/heartbeat 自动化定义变更前后的 intent、preview、确认和回滚。
- 与 `auto_p1_run_trace_workbench.md` 不同：run trace 聚合一次 agent run 的日志、prompt audit、LLM audit 和 session 事件；本提案聚合的是自动化 job 的变更意图和版本，不替代运行追踪。
- 与 `auto_p1_usage_entitlement_ledger.md` 不同：entitlement ledger 解决权益和成本控制；本提案只在 preview 中引用任务名额和潜在成本，不定义商业 plan。
- 与 `auto_p1_investment_context_intake.md` 不同：intake 解决持仓、画像和偏好缺口；本提案解决用户把持续监控交给 Hone 之前的自动化确认与治理。
- 与 `auto_p0_investment_output_safety_gate.md` 不同：safety gate 约束投资输出内容；本提案约束自动化配置变更，不判断投资结论本身。
- 与 `docs/proposals/desktop-bundled-runtime-startup-ux.md` 不同：desktop startup UX 关注本机进程和锁恢复；本提案只把 desktop 作为自动化运行能力和渠道状态的一个 preview 输入。
- 与 `docs/proposals/skill-runtime-multi-agent-alignment.md` 不同：skill runtime alignment 关注 skill 可见性、权限和 runner 语义；本提案只要求 `scheduled_task` skill 复用新的 intent/preview 工具契约。
