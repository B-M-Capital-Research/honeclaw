# Proposal: Agent Mutation Ledger for Confirmable and Reversible State Changes

status: proposed
priority: P1
created_at: 2026-05-09 17:03:53 +0800
owner: automation

## related_files

- `README.md`
- `AGENTS.md`
- `docs/repo-map.md`
- `docs/invariants.md`
- `docs/decisions.md`
- `docs/current-plan.md`
- `docs/proposal/auto_p1_automation_intent_control_plane.md`
- `docs/proposal/auto_p1_trade_discipline_journal.md`
- `docs/proposal/auto_p1_user-data-trust-center.md`
- `docs/proposals/skill-runtime-multi-agent-alignment.md`
- `crates/hone-tools/src/portfolio_tool.rs`
- `crates/hone-tools/src/cron_job_tool.rs`
- `crates/hone-tools/src/notification_prefs_tool.rs`
- `crates/hone-tools/src/skill_tool.rs`
- `crates/hone-tools/src/local_files.rs`
- `crates/hone-channels/src/agent_session/core.rs`
- `crates/hone-channels/src/execution.rs`
- `crates/hone-channels/src/mcp_bridge.rs`
- `crates/hone-web-api/src/routes/portfolio.rs`
- `crates/hone-web-api/src/routes/company_profiles.rs`
- `crates/hone-web-api/src/routes/notification_prefs.rs`
- `crates/hone-web-api/src/routes/cron.rs`
- `memory/src/portfolio.rs`
- `memory/src/cron_job/storage.rs`
- `memory/src/company_profile/storage.rs`
- `skills/portfolio_management/SKILL.md`
- `skills/notification_preferences/SKILL.md`
- `skills/scheduled_task/SKILL.md`
- `skills/company_portrait/SKILL.md`
- `packages/app/src/components/portfolio-detail.tsx`
- `packages/app/src/pages/public-portfolio.tsx`

## 背景与现状

Hone 已经从问答助手演进成能代用户维护投资工作台状态的 agent：

- `portfolio` tool 可以 `add/update/remove/watch/unwatch` 持仓和关注列表，直接写入 `memory/src/portfolio.rs` 管理的 actor-scoped JSON。
- `cron_job` tool 可以新增、修改和删除定时任务；删除已有 `confirm="yes"` 保护，但 `add/update` 仍会直接落盘。
- `notification_prefs` tool 可以启停推送、改 quiet hours、改 digest slots、调整 kind 白名单和最低严重度，写入后下一条事件即时生效。
- `company_portrait` skill 明确要求 agent 直接维护 actor sandbox 下的 `company_profiles/<profile_id>/profile.md` 与 `events/*.md`，这是长期投资记忆的真相源。
- 管理端 `portfolio`、`cron`、`company_profiles`、`notification_prefs` API 已经可以直接修改用户状态；前端 `PortfolioDetail` 的删除只靠浏览器 `confirm()`，没有统一 server-side mutation record。
- ACP/MCP runner 已能让底层 agent 调用 Hone 内置工具；skill runtime 也支持 script entrypoint 和本地 artifact。也就是说，Hone 的核心价值越来越依赖“agent 替我改东西”，而不只是“agent 回答问题”。

这些能力是产品护城河，但当前缺一个横跨工具、API、渠道和桌面端的 mutation 治理层。每个工具自己判断是否需要确认，成功后返回 JSON；系统没有统一记录“谁在什么上下文下提议了什么改动、改动前后是什么、用户是否确认、如何撤销、是否由 agent 还是 UI 发起”。

## 问题或机会

这是 P1 级问题，因为它直接影响用户信任、可恢复性、客服排障、商业化和长期记忆质量。Hone 面向投资场景，用户状态不是普通偏好：持仓、关注列表、推送规则、定时任务和公司画像都会影响未来回答、主动提醒和投资纪律。

主要问题：

1. **确认语义不一致。**
   `cron_job remove` 有二阶段确认，`portfolio remove` 在 tool 层没有确认，Web 删除依赖浏览器确认，`notification_prefs reset/disable` 会直接生效，company profile 写入则由 runner 文件操作完成。用户很难形成稳定预期：哪些动作会先预览，哪些动作会直接改。

2. **缺少统一 before/after diff。**
   当用户说“为什么我今天没收到推送”“为什么 NVDA 变成真实持仓了”“为什么画像里改了长期判断”，运维只能分别查 portfolio JSON、prefs JSON、cron history、company profile mtime、chat transcript 和日志，无法从一个 mutation timeline 找到状态改变的根因。

3. **撤销能力分散且不可靠。**
   cron 可以更新回旧值，但没有统一旧快照；portfolio 和 notification prefs 没有 server-side undo；company profile 文件修改可能只留下最终 Markdown。agent 一旦误解用户意图，恢复成本高于创建成本。

4. **多渠道和群聊放大误改风险。**
   Feishu / Telegram / Discord / iMessage 都可能触发同一套工具。`ActorIdentity` 和 `SessionIdentity` 已经分离，但 mutation 层还没有把确认人、确认渠道、会话、群聊触发者、工具调用结果绑定成可审计对象。

5. **现有提案需要一个共同底座。**
   Automation Intent Control Plane 只治理 cron 任务；Trade Discipline Journal 只记录投资操作决策；User Data Trust Center 关注导出/删除。它们都会受益于一个通用 mutation ledger，但不应各自发明一套确认、diff 和撤销模型。

机会是：不重写工具、不改变 actor 隔离，就可以先加一个薄的 `MutationLedger`。第一版把高影响 mutation 统一记录和可选确认，再逐步把工具迁到 propose/apply 模式。它能让 Hone 从“agent 能改状态”升级为“agent 的每次状态改变都可解释、可确认、可撤销”。

## 方案概述

新增 actor-scoped `AgentMutationLedger`，作为所有用户状态变更的共同治理层：

- `MutationDraft`：一次待应用的变更提案，包含 target、operation、before snapshot、proposed patch、impact preview、risk level、expires_at。
- `MutationRecord`：一次已应用、拒绝、过期或撤销的最终记录。
- `MutationTarget`：标准化目标类型，例如 `portfolio_holding`、`portfolio_watchlist`、`notification_prefs`、`cron_job`、`company_profile`、`skill_registry`、`desktop_settings`、`web_user_api_key`。
- `MutationSource`：`agent_tool`、`admin_web`、`public_web`、`desktop`、`channel_command`、`skill_script`、`system_migration`。
- `MutationRisk`：`low`、`medium`、`high`、`destructive`。风险决定是否必须确认、是否保留完整 snapshot、是否允许自动撤销。
- `apply_mode`：`direct`、`confirm_required`、`admin_only`、`dry_run_only`。
- `revert_plan`：可自动撤销的反向 patch，或明确标注 `manual_only` / `not_reversible`。

第一版不要求所有工具立即二阶段化。可以先做到：

1. 对现有直接写入工具包装 `record_before` / `record_after`。
2. 对高风险动作返回 `needs_confirmation`，沿用当前对话确认体验。
3. 对 Web/API 写入也落同一 ledger。
4. 管理端增加 mutation timeline 和单条 diff。
5. 为 portfolio、notification prefs、cron update/remove 提供一键撤销；company profile 第一版只提供 diff 和恢复建议，不自动合并 Markdown。

## 用户体验变化

### 用户端

- 当用户要求“删除持仓”“全部恢复默认推送”“把 NVDA 从关注改成持仓”“改公司画像长期结论”时，Hone 先给出简洁确认卡：将修改什么、影响哪些未来推送或回答、是否可撤销。
- 普通低风险动作仍可直接执行，例如查看状态、添加 watchlist、设置非破坏性的 notes；执行后回答里带一个短 reference：“已记录变更，可在最近变更中撤销”。
- Public `/portfolio` 或未来 `/me` 中增加“最近变更”列表：显示 agent/UI 改过哪些投资上下文，用户可以撤销最近的安全变更。

### 管理端

- `/users` 或 actor detail 增加 Mutation tab，按时间线展示 portfolio、prefs、cron、company profile、API key 等变更。
- `/portfolio`、`/tasks`、`/notifications` 详情页显示最近相关 mutation，能从状态异常跳到对应变更原因。
- 管理员可以对 reversible mutation 执行 revert；对 destructive 或 manual-only mutation 查看 before snapshot 和操作建议。
- 客服排障时不需要先猜数据在哪个文件，而是从 actor mutation timeline 开始。

### 桌面端

- Desktop bundled 模式在本地设置、channel enable/disable、backend mode 切换、持仓编辑等动作上复用同一 mutation record。
- 因为本地用户可能没有云端客服，桌面端应提供“导出最近变更诊断包”，包含 ledger 摘要、相关 config/prefs 快照 hash 和日志 reference，不上传到外部服务。

### 多渠道

- IM 端高风险 mutation 使用短确认码或 reply-to-bot 确认；确认 token 绑定 actor、session、target 和过期时间。
- 群聊中确认人必须是触发该 mutation 的用户；其他成员不能确认或撤销。
- 弱交互渠道可以先只支持“确认/取消”文本，不要求卡片。

## 技术方案

### 1. Ledger 存储

在 `memory` 增加 `mutation_ledger` 模块，优先使用 SQLite，因为需要按 actor、target、status、时间范围查询：

```sql
CREATE TABLE mutation_records (
  id TEXT PRIMARY KEY,
  actor_channel TEXT NOT NULL,
  actor_scope TEXT NOT NULL,
  actor_user_id TEXT NOT NULL,
  session_id TEXT,
  source TEXT NOT NULL,
  target_type TEXT NOT NULL,
  target_id TEXT NOT NULL,
  operation TEXT NOT NULL,
  risk TEXT NOT NULL,
  status TEXT NOT NULL,
  before_json TEXT,
  after_json TEXT,
  proposed_patch_json TEXT,
  revert_patch_json TEXT,
  confirmation_token_hash TEXT,
  confirmed_by TEXT,
  confirmed_at TEXT,
  created_at TEXT NOT NULL,
  applied_at TEXT,
  expires_at TEXT,
  error TEXT
);
```

保留 JSON export helper，方便 User Data Trust Center 未来打包用户数据。Ledger 只记录 actor-scoped 用户状态，不记录完整 prompt；需要关联 prompt 时只保存 `session_id`、`trace_id` 或 `llm_audit_id`。

### 2. Mutation API 与服务层

新增 `MutationLedgerService`：

- `propose(actor, source, target, operation, before, patch, risk) -> MutationDraft`
- `apply(draft_id, confirmation) -> MutationRecord`
- `record_direct(actor, source, target, operation, before, after, risk) -> MutationRecord`
- `revert(record_id) -> MutationRecord`
- `list(actor, filter) -> Vec<MutationRecord>`

第一版可以不引入复杂 JSON Patch。对结构化 JSON 状态使用完整 before/after snapshot；对 Markdown 文件使用 content hash、excerpt 和可选 unified diff。

### 3. Tool 包装策略

按风险分层迁移现有工具：

- `portfolio`
  - `view/watch` 默认 low，可 direct record。
  - `add/update/remove` 如果是真实持仓、期权、批量修改、从 watchlist promote，默认 medium/high，需要在非 admin agent 入口先 propose。
  - Web UI 手工编辑可 direct apply，但仍记录 before/after，并提供 undo。
- `notification_prefs`
  - `disable/reset/clear_allow/clear_block/set_digest_slots/set_quiet_hours` 默认 medium；会吞掉 digest 或关闭所有推送的变更必须确认。
  - `get/get_overview` 不入 ledger。
- `cron_job`
  - 复用 Automation Intent Control Plane 的 cron intent；ledger 记录最终 applied mutation，intent 记录 preview workflow。
  - 在 cron intent 未落地前，`add/update/remove` 先写 ledger。
- `company_portrait`
  - 文件写入仍由 runner 完成，但 local file tools 或 runner event 可以在写入前后计算 hash/diff，生成 `company_profile` mutation。
  - 第一版不自动 revert Markdown，只把 before snapshot 存为恢复材料。
- `skill_registry` / settings / API key
  - 管理端 enable/disable skill、reset API key、修改 runner/channel 配置都写 ledger，避免未来排障只能查日志。

### 4. AgentSession 与 MCP 关联

`AgentSession::run()` 和 `execution` 层已经集中处理 actor、session、runner、prompt audit 和 tool registry，适合注入 `MutationContext`：

- actor、session identity、chat mode、channel、trace id。
- 当前 turn 的 user-visible confirmation token registry。
- 工具执行后统一回传 mutation event 到 listener，Web SSE 可显示“pending mutation / applied mutation”。

MCP bridge 暴露工具时不需要让底层 runner 自己实现确认逻辑；确认状态由 Hone tool wrapper 决定。如果 runner 调用高风险 tool 且未带 confirmation，tool 返回 `needs_confirmation` 和 draft id。

### 5. API 与前端

新增 API：

- `GET /api/mutations?channel=&user_id=&channel_scope=&target_type=&status=`
- `GET /api/mutations/{id}`
- `POST /api/mutations/{id}/confirm`
- `POST /api/mutations/{id}/reject`
- `POST /api/mutations/{id}/revert`
- `GET /api/public/mutations`
- `POST /api/public/mutations/{id}/confirm`
- `POST /api/public/mutations/{id}/reject`

前端入口：

- `packages/app/src/pages/users.tsx` 或 actor detail：新增 Mutation tab。
- `packages/app/src/components/portfolio-detail.tsx`：删除和编辑成功后显示最近变更，可撤销。
- `packages/app/src/pages/notifications.tsx`：展示最近 prefs 变更与影响。
- `packages/app/src/pages/chat.tsx`：当 SSE 或 final response 带 pending mutation 时渲染确认按钮；无 JS 卡片能力的渠道继续文本确认。
- `packages/app/src/pages/public-portfolio.tsx`：只读画像旁边显示最近 company profile mutation 摘要，帮助用户理解主线何时被更新。

### 6. 兼容与迁移

- 不迁移历史全部数据；从上线后记录新 mutation。
- 对现有 `cron_job remove` 的 `confirm="yes"` 保持兼容，内部映射为 draft confirm。
- 对旧前端 API 调用不强制改为 propose；先 direct record，避免破坏管理端效率。
- 对高风险 agent tool 调用逐步收紧：observe -> warn -> require confirmation。
- Ledger 不改变 `ActorIdentity` / `SessionIdentity` 语义，只引用它们。

## 实施步骤

### Phase 1: 只读 ledger 与直接写入记录

- 增加 `memory/src/mutation_ledger.rs` 类型、SQLite 存储和单元测试。
- 在 portfolio API/tool、notification prefs tool、cron job tool 成功写入后记录 before/after。
- 增加 admin list/detail API 和最小 Mutation tab。
- 验证不改变现有工具行为。

### Phase 2: 高风险确认草稿

- 为 portfolio true holding remove/update、notification prefs reset/disable、cron remove/update 增加 `needs_confirmation` draft。
- 在 public chat 和主要 IM 渠道支持确认码。
- Web chat 支持 pending mutation 卡片。
- 增加过期清理和拒绝状态。

### Phase 3: Revert 与跨页面排障

- 支持 portfolio、notification prefs、cron job 的自动 revert。
- `/portfolio`、`/tasks`、`/notifications` 详情页链接最近 mutation。
- 生成本地 mutation diagnostic bundle，供桌面和管理员排障。

### Phase 4: Company profile 与 runner 文件写入

- 在 actor sandbox 文件写入前后捕捉 company profile hash/diff。
- 将 `company_portrait` skill 的重要写回纳入 mutation timeline。
- 只提供恢复材料和人工 revert，不在第一版自动合并 Markdown。

## 验证方式

- 单元测试：
  - ledger roundtrip、actor 隔离、target filter、status transition。
  - portfolio add/update/remove 记录 before/after。
  - notification prefs disable/reset 记录完整 snapshot。
  - confirmation token hash 校验、过期拒绝、非同 actor 不能确认。
  - revert 对 portfolio 和 prefs 可恢复到 before snapshot。
- API 测试：
  - admin 只能按显式 actor 查询 mutation。
  - public mutation API 只能读取当前 cookie actor。
  - confirm/reject/revert 对状态机非法转移返回 409 或同等错误。
- 回归脚本：
  - 在 `tests/regression/ci/` 增加无外部依赖的 mutation flow：创建持仓 -> 更新 -> list mutations -> revert -> portfolio 恢复。
  - Cron 相关只测试 storage/tool 层，不触发真实外部渠道。
- 手工验收：
  - Public chat 中要求删除真实持仓，应得到确认卡/确认码，未确认前数据不变。
  - Telegram 或 Feishu 中回复确认码后变更生效，并能在 Web 管理端看到同一条 record。
  - 修改 quiet hours 后，Notifications 页面能看到变更来源和 before/after。
  - Company profile 更新后，Mutation tab 至少展示文件 hash、路径和摘要 diff。
- 指标：
  - 高风险 mutation 中确认率、拒绝率、撤销率。
  - 因 mutation revert 解决的 support/debug 次数。
  - agent tool 误改后恢复时间。

## 风险与取舍

- 风险：确认太多会降低 agent 的“能干活”体验。取舍：只对 destructive/high-risk agent mutation 强制确认，低风险和 admin UI 手工动作先 direct record。
- 风险：存 before/after 可能包含敏感投资资料。取舍：ledger 默认 actor-scoped、本地存储、admin-only；public 只显示摘要，完整 snapshot 导出遵守 User Data Trust Center 的权限模型。
- 风险：JSON snapshot 和 Markdown diff 增加磁盘占用。取舍：对大文件存 hash + excerpt + backup path，不把完整大附件塞进 SQLite。
- 风险：工具迁移到 propose/apply 会扩大接口复杂度。取舍：第一阶段只记录 direct mutation；确认草稿按工具逐步开启。
- 风险：自动 revert 可能覆盖用户后续改动。取舍：revert 前校验当前状态 hash 是否等于 original after hash；若已发生后续变更，只允许人工恢复或生成新 draft。
- 风险：与 Automation Intent Control Plane 重叠。取舍：cron intent 负责预演和审批 workflow，mutation ledger 只记录所有领域的状态变更事实和可撤销材料；cron intent applied 后也写 ledger。
- 不做：不实现完整 workflow engine，不做跨用户共享审批，不把 ledger 当作聊天历史替代品，不允许模型绕过 actor 权限直接改其他用户状态，不自动回滚 company profile Markdown。

## 与已有提案的差异

- 不重复 `auto_p1_automation_intent_control_plane.md`：该提案只治理自动化/cron 的创建、修改、删除预演；本提案治理 portfolio、notification prefs、company profile、settings、API key、cron 等所有状态变更的统一审计、确认和撤销底座。
- 不重复 `auto_p1_trade_discipline_journal.md`：journal 记录用户投资操作意图、理由和复盘；mutation ledger 记录系统状态是否被 agent/UI 改写。买卖决策可以产生 journal，更新持仓则产生 mutation record。
- 不重复 `auto_p1_user-data-trust-center.md`：trust center 解决用户数据清单、导出、删除；本提案解决变更过程的 before/after、确认、撤销和排障。未来 trust center 可导出 mutation ledger。
- 不重复 `auto_p1_run_trace_workbench.md`：run trace 关注一次 agent run 的执行链路、日志和错误；mutation ledger 关注跨 run 持续存在的用户状态改变。
- 不重复 `auto_p1_response-feedback-learning-loop.md`：feedback loop 收集回答质量反馈；mutation ledger 处理工具和 UI 对持久状态的变更，不评价自然语言回答质量。
- 不重复 `docs/proposals/skill-runtime-multi-agent-alignment.md`：skill runtime alignment 关注 skill 可见性、runner 语义和工具权限；本提案在技能真正造成持久状态变化时记录和治理 mutation。

查重结论：现有 proposal 覆盖输出安全、自动化 intent、数据隐私、运行追踪、权益、研究资产、证据复盘和交易纪律，但没有覆盖“所有 agent/UI 持久状态变更的统一确认、审计和撤销层”。本主题是新的、可落地的 P1 产品/架构提案，能直接提升用户信任、排障效率和多渠道 agent 的可控性。
