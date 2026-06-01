# Proposal: Maintenance Window and Write Freeze Control Plane

status: proposed
priority: P1
created_at: 2026-06-01 14:06:40 +0800
owner: automation
verification: see `## 验证方式`
risks: see `## 风险与取舍`

## related_files

- `README.md`
- `AGENTS.md`
- `docs/repo-map.md`
- `docs/invariants.md`
- `docs/decisions.md`
- `docs/current-plan.md`
- `docs/proposal/auto_p1_product-rollout-kill-switch.md`
- `docs/proposal/auto_p1_runtime-dependency-circuit-breaker.md`
- `docs/proposal/auto_p1_storage-schema-migration-registry.md`
- `docs/proposal/auto_p1_agent-mutation-ledger.md`
- `docs/proposal/auto_p1_config-apply-evidence.md`
- `crates/hone-web-api/src/routes/mod.rs`
- `crates/hone-web-api/src/routes/public.rs`
- `crates/hone-web-api/src/routes/cron.rs`
- `crates/hone-web-api/src/routes/company_profiles.rs`
- `crates/hone-web-api/src/routes/portfolio.rs`
- `crates/hone-web-api/src/routes/notification_prefs.rs`
- `crates/hone-web-api/src/routes/event_engine_admin.rs`
- `crates/hone-web-api/src/routes/research.rs`
- `crates/hone-web-api/src/routes/skills.rs`
- `crates/hone-web-api/src/lib.rs`
- `crates/hone-web-api/src/state.rs`
- `crates/hone-channels/src/agent_session/mod.rs`
- `crates/hone-channels/src/scheduler.rs`
- `crates/hone-event-engine/src/engine.rs`
- `memory/src/session.rs`
- `memory/src/session_sqlite.rs`
- `memory/src/cron_job/mod.rs`
- `memory/src/portfolio.rs`
- `memory/src/company_profile/storage.rs`
- `memory/src/web_auth.rs`
- `bins/hone-cli/src/cloud.rs`
- `bins/hone-desktop/src/sidecar/processes.rs`
- `packages/app/src/pages/settings.tsx`
- `packages/app/src/pages/task-health.tsx`
- `packages/app/src/pages/public-chat.tsx`
- `packages/app/src/pages/public-portfolio.tsx`
- `packages/app/src/lib/api.ts`

## 背景与现状

Honeclaw 已经从本地投研助手变成一个多入口、多写路径的 agent 工作台。当前代码结构里有几类事实值得一起看：

- `crates/hone-web-api/src/routes/mod.rs` 暴露了大量会改状态的 HTTP 路由：public SMS login、public chat、OpenAI-compatible chat、public upload、digest context refresh、admin chat、cron job create/update/delete/toggle、portfolio holdings create/update/delete、notification prefs、global digest config、RSS feeds、mainline distill、company profile import/delete、research start/generate PDF、skill state/reset、web invite 和 API key 管理。
- `crates/hone-web-api/src/lib.rs` 在 `RuntimeRole::runs_worker_tasks()` 下启动 event-engine、mainline distill cron 和 scheduler；这意味着同一后端进程既可能服务交互请求，也可能主动写 task history、delivery log、notification prefs mainline distill、session 或 channel delivery 状态。
- `memory/` 已经有多类 actor-scoped durable state：session JSON/SQLite、cron jobs、portfolio、company profile files、web auth、quota、LLM audit。local mode 与 cloud mode 的真相源不同，但上层路由仍会在同一产品动作里触达多个 store。
- 桌面 bundled runtime 会管理 backend 和渠道 sidecar，remote mode 又连接远端 backend。升级、迁移或事故处理时，用户可能仍从桌面、public Web、admin Web、IM 渠道和 API key 同时进入系统。
- 现有 proposal 已经覆盖了功能灰度/kill switch、依赖熔断、schema migration registry、agent mutation ledger、config apply evidence 等关键局部能力；但还没有一个面向“维护窗口”的总控层，明确在升级、迁移、回滚、事故调查期间哪些写入应被暂停、哪些只读能力仍可用、哪些后台任务要 drain、用户会看到什么。

这不是单纯的 feature flag，也不是单个依赖故障。维护窗口经常是人为、有计划、跨 store、跨入口、跨 worker 的状态：例如本地 JSON -> SQLite/PG 切换、company profile import 修复、public auth 数据清理、event-engine 噪音事故回放、release 后回滚评估、桌面 bundled 数据目录升级。没有统一维护状态时，最危险的不是“某个接口报错”，而是系统一半入口继续写旧结构，另一半入口正在迁移或排障。

## 问题或机会

这是 P1 级问题：它不一定每天发生，但一旦发生，影响的是数据一致性、升级可信度、主动任务可信度和用户信任。

主要问题：

1. **写路径没有统一冻结判定。**  
   创建 cron、改 portfolio、刷新 mainline、导入公司画像、public upload、start research、修改 notification prefs、更新 skill state、web invite reset、public chat session write 都由不同 route 或 worker 自行处理。维护窗口期间需要逐个入口打补丁才可能停住写入。

2. **后台 worker 与用户请求没有共同的 drain 语义。**  
   `start_server` 会启动 scheduler、event-engine 和 mainline distill cron。即使 admin UI 暂停了某个按钮，后台任务仍可能继续执行并写 cron history、delivery log、prefs 或 session 相关状态。

3. **本地/云端/桌面升级缺少用户可理解的维护状态。**  
   local mode、cloud mode、desktop bundled、desktop remote 的停机动作不同。用户看到的可能是 timeout、按钮失败、channel silent skip 或启动失败，而不是“当前处于维护窗口，只读可用，写入将在 15 分钟后恢复”。

4. **迁移与事故调查期间容易出现新旧状态混写。**  
   schema migration registry 可以管理表版本，但不能单独阻止仍在运行的 public chat、cron worker 或 IM listener 继续写入旧 store。feature kill switch 可以关某功能，但不表达“全站写冻结、按 actor 冻结、按 store 冻结、只允许 admin break-glass 写入”。

5. **运维与商业体验没有稳定承诺。**  
   Hosted/public 用户、API-key 用户和桌面用户都需要知道服务是否只读、自动化是否暂停、已排队任务是否会补跑。否则维护会被感知为随机故障，直接影响付费信任。

机会是新增一个轻量的 **Maintenance Window and Write Freeze Control Plane**：把维护状态变成一等运行时对象，所有写路径和 worker 在执行前都做同一类 decision，前端和多渠道显示同一套用户态解释。

## 方案概述

新增一个 runtime-scoped 的 `MaintenanceController`，用于声明、查询和执行维护窗口：

- `MaintenanceWindow`：维护窗口记录，包含 scope、mode、reason、starts_at、ends_at、created_by、affected_surfaces、affected_stores、operator_note。
- `WriteClass`：把写入分级，例如 `session_message`、`user_upload`、`portfolio_mutation`、`cron_mutation`、`company_profile_mutation`、`notification_pref_mutation`、`config_mutation`、`skill_registry_mutation`、`research_task_start`、`web_auth_mutation`、`background_delivery`、`audit_append`。
- `FreezeMode`：`read_only`、`user_write_frozen`、`automation_draining`、`store_migration`、`full_maintenance`、`break_glass_admin_only`。
- `MaintenanceDecision`：调用前判定结果，包含 `allow`、`queue`、`reject`、`drain`、`reason_code`、`retry_after`、`user_message_key`、`operator_message`、`window_id`。
- `BypassToken`：只给高权限 operator 使用的短 TTL break-glass token，必须写审计，默认不开放给 agent 或普通 admin UI。

第一版目标：

1. 先覆盖 Web API 和 worker 的最高风险写路径，不改底层 store 语义。
2. 允许按 deployment、surface、actor、store、write class 设置维护窗口。
3. 让 public/admin/desktop/IM/API-key 用户看到一致的维护文案和 retry-after。
4. 让 scheduler/event-engine 能 drain 或 skip 可重试写入，而不是在维护期继续制造部分写。
5. 为后续 storage migration、release upgrade、incident response 提供稳定操作入口。

## 用户体验变化

### 用户端

- Public `/chat` 在 `user_write_frozen` 时不再进入长时间运行；立即返回“系统正在维护，暂时不能发送新消息”，并展示预计恢复时间。
- `/portfolio`、`/me`、company profile 只读内容仍可打开；刷新 mainline、上传附件、启动研究、创建任务等写动作显示 disabled 状态和维护原因。
- OpenAI-compatible `/api/public/v1/chat/completions` 返回稳定 error code，例如 `maintenance_write_frozen`，包含 `retry_after`，便于外部客户端退避。
- 如果维护模式允许排队，用户可以看到“已进入待恢复队列”；第一版建议只对低风险后台任务排队，不对 chat 自动排队，避免恢复后生成过期回答。

### 管理端

- Settings 或 Task Health 增加 `Maintenance` 区块：
  - 当前 active window、scope、剩余时间、影响写类型、创建人、原因。
  - 一键进入 `read_only`、`automation_draining`、`full_maintenance` 的安全预设。
  - 可查看最近被拒绝/排队/drain 的请求计数。
- 维护期内，cron/portfolio/company profile/import/skill/web invite/global digest config 等按钮显示同一套 blocked reason，而不是每页自行报错。
- Task Health 可以把维护导致的 skipped/drained 与真实失败分开，避免维护窗口污染故障率。
- break-glass 写入需要明确确认、短 TTL、reason，并写入未来 operator audit / mutation ledger。

### 桌面端

- Bundled mode 启动时先读取本地维护状态：如果 store migration 正在进行，桌面只启动只读 backend，不启动会写状态的 channel sidecar。
- Remote mode 只展示远端 `/api/maintenance/status`，不在本地伪造远端状态。
- 桌面 dashboard 显示维护 banner 和下一个恢复时间；tray 可以提供“打开维护状态”入口。
- 如果用户在桌面离线期间升级，首次启动可进入本地 `store_migration` 窗口，完成低风险迁移后自动关闭。

### 多渠道

- Feishu/Telegram/Discord/iMessage direct chat 在维护窗口内返回短提示，不进入 runner。
- 群聊中不广播详细维护 scope，只在 explicit trigger 时返回一句用户态解释。
- 主动推送在 `automation_draining` 下停止 direct delivery，可选择保留 digest buffer 或记录 `maintenance_drained`，恢复后按策略补跑。
- iMessage 这类本地 privileged worker 必须遵守同一维护状态，不能绕过 Web API 的冻结。

## 技术方案

### 1. 维护状态存储

local mode 使用 runtime 文件或 SQLite 小表，cloud mode 使用 PG 表；第一版可以先做 local file + cloud adapter trait：

```text
data/runtime/maintenance_windows.json
```

建议结构：

```json
{
  "version": 1,
  "windows": [
    {
      "id": "mw_20260601_storage_migration",
      "mode": "store_migration",
      "scope": {
        "surfaces": ["public_web", "admin_web", "desktop", "channel", "worker"],
        "actors": [],
        "stores": ["session_runtime", "company_profiles"]
      },
      "write_classes": ["session_message", "company_profile_mutation", "background_delivery"],
      "reason_code": "session_store_cutover",
      "starts_at": "2026-06-01T06:00:00Z",
      "ends_at": "2026-06-01T06:30:00Z",
      "created_by": "operator",
      "operator_note": "Freeze writes during session runtime backend validation."
    }
  ]
}
```

不要把它做成第二套长期 config。它是 runtime 操作状态，删除 `data/runtime/` 在 local mode 仍应是安全 reset；若需要长期 hosted 操作历史，再由 PG/audit 保留。

### 2. Decision API 与写入分类

在 `hone-core` 或 `hone-web-api` 定义纯类型：

```rust
pub enum WriteClass {
    SessionMessage,
    UserUpload,
    PortfolioMutation,
    CronMutation,
    CompanyProfileMutation,
    NotificationPrefMutation,
    ConfigMutation,
    SkillRegistryMutation,
    ResearchTaskStart,
    WebAuthMutation,
    BackgroundDelivery,
    AuditAppend,
}

pub struct MaintenanceContext {
    pub surface: MaintenanceSurface,
    pub actor: Option<ActorIdentity>,
    pub write_class: WriteClass,
    pub store_hint: Option<String>,
    pub request_id: String,
}
```

所有写路径调用：

```text
maintenance.before_write(context) -> MaintenanceDecision
```

`AuditAppend`、logs、read-only health API 默认允许，即使 full maintenance 下也应保留最小证据写入；如果证据写入也失败，应降级到 stderr/file log。

### 3. Web API 接入点

优先覆盖这些 route：

- Public：`/api/public/chat`、`/api/public/v1/chat/completions`、`/api/public/upload`、`/api/public/digest-context/refresh`、SMS login/send 是否冻结要按 mode 区分；普通 read-only auth/me/history 不冻结。
- Admin：`/api/chat`、cron job create/update/delete/toggle、portfolio holding create/update/delete、notification prefs put、global digest put、RSS feeds create/update/delete、mainline distill now、company profile import apply/delete、research start/generate PDF、skill state/reset、web invite create/reset/API key mutation、channel settings put。
- Events/SSE/history/read-only profile/list routes 不冻结，只在响应中可附带 maintenance status。

实现上可以先写一层 helper：

```rust
fn reject_if_maintenance(decision: MaintenanceDecision) -> Option<Response>
```

后续再升级为 axum middleware + route-local write class annotation。第一版不要试图自动推断所有 POST 都是同一种写入，因为 `/auth/sms/send`、`/events`、`/runtime/heartbeat`、`/logs/stream` 的语义不同。

### 4. Worker drain 语义

`crates/hone-web-api/src/lib.rs` 启动 worker 前读取维护状态：

- `full_maintenance`：不启动 scheduler/event-engine/mainline distill cron，只启动只读 API。
- `automation_draining`：worker 启动但 before-run 判定为 drain，写 task run `maintenance_drained`，不调用 LLM 或 channel sink。
- `store_migration`：只冻结命中 store 的 write class，例如 session/company profile/portfolio；不影响只读通知日志。

`crates/hone-channels/src/scheduler.rs` 和 event-engine dispatch 前检查 `BackgroundDelivery` / `SessionMessage` / `NotificationPrefMutation` 等写 class。恢复后是否补跑应按任务类型决定：

- heartbeat：不补跑。
- digest：下一 slot 正常生成，避免重复补发旧摘要。
- explicit cron：可记录 missed window，允许用户或 admin 手动 rerun。
- event-engine direct push：默认不补发 direct，进入 digest buffer 或记录 skipped。

### 5. 前端状态传播

新增：

- `GET /api/maintenance/status`
- `POST /api/maintenance/windows`
- `DELETE /api/maintenance/windows/{id}`
- `GET /api/public/maintenance/status`

`/api/meta` 可加入简要 `maintenance_active` 和 `maintenance_mode`，让前端启动时立刻渲染 banner。public 端只拿用户态字段，不暴露 store path、operator note、actor list。

前端接入：

- admin layout 顶部 banner。
- settings/task-health 维护控制台。
- public chat 发送按钮、upload、refresh mainline 的 disabled reason。
- desktop backend disconnected 与 maintenance 区分显示。

### 6. 审计、兼容与边界

- 创建/关闭维护窗口必须写 audit event；在 operator access audit proposal 落地前，先写结构化 runtime log 和可导出的 JSONL。
- `maintenance_windows.json` 损坏时 fail closed 还是 fail open 要按 deployment 决定：local dev fail open 并告警；hosted/cloud fail closed 到 read-only 更安全。
- 维护窗口不能作为长期功能授权系统；功能灰度仍属于 Product Rollout Registry。
- 维护窗口不能替代 dependency circuit breaker；外部 provider 故障仍由 circuit breaker 自动处理。
- 维护窗口不能替代 schema migration registry；它只负责冻结/排队/提示，具体迁移版本由 schema registry 管。

## 实施步骤

### Phase 1: Status and Read-Only Surfacing

1. 定义 `MaintenanceWindow`、`WriteClass`、`MaintenanceDecision` 类型和 local runtime store。
2. 新增 status API 和 `/api/meta` 简要状态。
3. Admin/public/desktop 前端显示维护 banner，不阻断请求。
4. 写 store 单元测试：active window 匹配、过期过滤、scope 匹配、损坏文件处理。

### Phase 2: Web API Write Freeze

1. 给最高风险 Web API route 接入 explicit write class helper。
2. 返回统一 error code、retry-after 和用户态文案。
3. 前端禁用 chat/upload/refresh/import/delete/create/update 等写按钮。
4. 增加 route-level 单元或集成测试，证明 read-only route 仍可用、写 route 被拒绝。

### Phase 3: Worker Drain and Desktop Startup

1. scheduler/event-engine/mainline distill cron 启动和 run 前检查维护 decision。
2. Task run / notification log 记录 `maintenance_drained` 或 `maintenance_skipped`。
3. Desktop bundled startup 在 store migration/full maintenance 下避免启动会写状态的 sidecar。
4. 增加 CI-safe regression：创建维护窗口后启动最小 Web API，确认 worker 不执行写路径。

### Phase 4: Operator Controls and Cloud Adapter

1. Admin UI 支持创建 TTL 窗口、关闭窗口、查看受影响写 class。
2. CLI 增加 `hone-cli maintenance status/start/end`，用于 release 和 migration runbook。
3. Cloud mode 使用 PG-backed maintenance store，并支持多 worker 共享维护状态。
4. 与 operator audit / mutation ledger / schema migration registry 对接，形成升级前后证据链。

## 验证方式

- 单元测试：
  - `MaintenanceDecision` 对 surface、actor、store、write class、时间窗口的匹配。
  - 过期 window 自动忽略，重叠 window 选择更严格 mode。
  - 损坏维护文件按 deployment 策略 fail open/fail closed。
- Web/API 测试：
  - public history/profile/auth/me/read-only routes 在 read-only window 下仍返回。
  - public chat/upload/digest refresh 返回 `maintenance_write_frozen`。
  - admin cron/portfolio/company profile import/skill state/web invite mutations 被拒绝。
  - `AuditAppend` 或 runtime log 在维护期仍可写最小证据。
- Worker 回归：
  - `automation_draining` 下 scheduler 不调用 runner，不发送 channel sink，写 `maintenance_drained`。
  - `full_maintenance` 下 runtime role `worker/all` 不启动主动任务。
  - 恢复后 heartbeat 不补跑，digest/direct push 按策略处理 missed window。
- 前端验收：
  - admin/public/desktop 显示一致维护 banner、retry time 和 disabled reason。
  - OpenAI-compatible endpoint 返回可机器处理的 error code 和 retry-after。
- 指标：
  - 维护期间写请求拒绝数、drained worker run 数、维护后恢复成功率。
  - 维护窗口内是否出现未分类写入；目标是第一阶段观测，第二阶段收敛到 0。

## 风险与取舍

- **风险：冻结范围过宽影响用户体验。** 取舍：第一版提供预设 mode，同时允许按 write class/store 缩小 scope；默认 read-only 内容保持可访问。
- **风险：冻结范围过窄仍出现混写。** 取舍：先覆盖最高风险写路径，并增加“未分类 POST/worker 写入观测”报告；不要声称 v1 已覆盖全仓所有副作用。
- **风险：维护状态本身成为第二套配置。** 取舍：只保存短期 runtime window，不承载长期功能开关和权益授权。
- **风险：排队语义复杂。** 取舍：v1 以 reject/drain 为主，只对明确可重试的后台任务保留 queue 预留字段；chat 不自动排队。
- **风险：hosted 多实例状态不一致。** 取舍：local file 只服务单机；cloud mode 必须使用 PG store 和短缓存 TTL，不允许每个 worker 私有维护窗口。
- **风险：break-glass 被滥用。** 取舍：break-glass 不在 v1 默认 UI 暴露；必须 TTL、reason、operator audit，并禁止 agent 自动使用。

## 与已有提案的差异

查重范围包括 `docs/proposal/` 下全部 `auto_p*.md`，以及历史 `docs/proposals/desktop-bundled-runtime-startup-ux.md`、`docs/proposals/skill-runtime-multi-agent-alignment.md`。

- 不重复 `auto_p1_product-rollout-kill-switch.md`：该提案管理单个产品功能的灰度、开启、事故止血；本提案管理有计划或事故期间的跨功能、跨 store、跨入口写入冻结和 worker drain。
- 不重复 `auto_p1_runtime-dependency-circuit-breaker.md`：该提案自动处理外部依赖故障；本提案处理人为维护/迁移/升级/调查窗口，即使所有外部依赖健康也可能启用。
- 不重复 `auto_p1_storage-schema-migration-registry.md`：该提案记录 schema 版本、迁移执行和数据健康；本提案在迁移前后冻结或恢复写路径，避免迁移期间新旧状态混写。
- 不重复 `auto_p1_agent-mutation-ledger.md`：该提案追踪 agent 做了什么可确认/可回滚的变更；本提案决定维护期哪些变更根本不允许开始。
- 不重复 `auto_p1_config-apply-evidence.md`：该提案解释 canonical/effective/running config 是否一致；本提案解释当前系统是否处于维护窗口，以及哪些写动作被拒绝或 drain。
- 不重复 desktop startup UX、runtime readiness、task health、notification policy 等提案：这些关注启动体验、能力就绪、运行结果或通知策略；本提案补的是横跨用户请求和后台 worker 的维护控制面。

差异结论：现有提案已经覆盖“功能是否开放”“依赖是否健康”“schema 是否已迁移”“变更是否可追踪”，但缺少“维护期间全系统哪些写入必须暂停、后台任务如何 drain、用户如何看到一致说明”的产品/架构层。因此本提案是新的、可执行的 P1 主题。

## 文档同步说明

本轮只新增 proposal，不开始执行实现，因此不更新 `docs/current-plan.md`，也不归档任何活跃任务。若后续实际落地本提案，应新增或复用 `docs/current-plans/maintenance-write-freeze.md`，并在引入维护状态、worker drain、CLI maintenance 命令或维护期 API 错误契约时同步更新 `docs/repo-map.md`、`docs/invariants.md`、相关 runbook 与 release/upgrade 操作说明。
