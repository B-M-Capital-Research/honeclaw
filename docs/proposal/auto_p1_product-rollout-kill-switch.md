# Proposal: Product Rollout and Kill-Switch Registry

status: proposed
priority: P1
created_at: 2026-05-19 14:03:40 +0800
owner: automation

## related_files

- `README.md`
- `AGENTS.md`
- `docs/repo-map.md`
- `docs/invariants.md`
- `docs/decisions.md`
- `docs/current-plan.md`
- `docs/proposal/auto_p0_investment_output_safety_gate.md`
- `docs/proposal/auto_p1_model-route-evaluation-lab.md`
- `docs/proposal/auto_p1_runtime_readiness_matrix.md`
- `docs/proposal/auto_p1_update-compatibility-center.md`
- `docs/proposal/auto_p1_usage_entitlement_ledger.md`
- `docs/proposal/auto_p1_agent-permission-broker.md`
- `docs/proposal/auto_p1_skill-trust-marketplace.md`
- `config.example.yaml`
- `crates/hone-core/src/config/{mod.rs,materialize.rs,mutation.rs,event_engine.rs,server.rs}`
- `crates/hone-web-api/src/routes/{meta.rs,channel_settings.rs,skills.rs,public.rs,web_users.rs}`
- `crates/hone-tools/src/skill_registry.rs`
- `crates/hone-channels/src/{execution.rs,agent_session/mod.rs,turn_builder.rs,scheduler.rs}`
- `crates/hone-event-engine/src/{engine.rs,router,global_digest,unified_digest}`
- `memory/src/{session.rs,session_sqlite.rs,web_auth.rs,quota.rs,cron_job}`
- `bins/hone-cli/src/{onboard.rs,start.rs,reports.rs}`
- `bins/hone-desktop/src/sidecar/{settings.rs,processes.rs,runtime_env.rs}`
- `packages/app/src/app.tsx`
- `packages/app/src/pages/{dashboard.tsx,settings.tsx,skills.tsx,task-health.tsx,notifications.tsx,public-home.tsx,chat.tsx}`
- `packages/app/src/lib/{api.ts,backend.ts,public-chat.ts}`

verification: see `## 验证方式`
risks: see `## 风险与取舍`

## 背景与现状

Honeclaw 已经进入一个典型的多形态 agent 产品阶段：同一套仓库同时服务本地 CLI、Tauri desktop bundled runtime、远端 backend、public Web、admin Web、Feishu/Telegram/Discord/iMessage 多渠道、scheduled tasks、event engine、skills、Hone Cloud runner 和 actor-scoped long-term memory。

当前系统里已经存在大量“开关”，但它们分散在不同层级：

- `config.example.yaml` 里有 `event_engine.enabled`、各类 event source/enrichment 开关、`storage.session_runtime_backend`、`storage.session_sqlite_shadow_write_enabled`、`web.auth_token`、`security.tool_guard`、`agent.runner`、channel `enabled` 和多种 runner/model 配置。
- `crates/hone-core/src/config/materialize.rs` 已经有长期 rollout 修正逻辑，例如启动时规范化 session SQLite shadow write，说明仓库已经遇到过“旧配置把新 rollout 关闭”的问题。
- `crates/hone-tools/src/skill_registry.rs` 有全局 skill enabled/disabled registry，但它只表达 skill 可见性，不表达产品功能灰度、actor cohort、部署模式或紧急停用。
- `/api/meta` 返回静态 capability 列表，例如 `skills`、`cron_jobs`、`company_profiles`、`llm_audit`、`web_invites`，但这些 capability 只说明后端暴露了接口，不说明某个功能是否正在灰度、是否被 kill switch 关闭、是否只对 public/desktop/admin/某些 actor 开放。
- Desktop bundled 模式、channel settings、public SMS login、Hone Cloud API key、event-engine direct push、session SQLite backend、company profile transfer、chart visualization skill 等能力，都可能需要按 deployment、surface、actor、channel 或 risk level 分阶段打开。
- 现有 proposal 已经覆盖 readiness、update compatibility、model route evaluation、safety gate、usage entitlement、operator audit、permission broker 和 skill trust marketplace，但它们各自解决的是“能不能跑”“版本是否兼容”“模型是否值得切换”“输出是否安全”“谁有权益”“谁操作了什么”“agent action 是否可批准”“skill 是否可信”。还没有一个统一层回答：**某个产品功能应该对哪些人、哪些入口、哪些部署模式开放；出现事故时如何一键关闭；关闭后用户和运维如何看懂原因。**

这类缺口在 AI agent 产品里会越来越重要。Hone 的新能力通常不是纯前端开关，而是跨 prompt、runner、tool、storage、channel outbound 和自动化执行的链路。例如一个新的 event-engine enrichment 可能消耗 LLM、改变通知、写入 execution history，并通过多渠道主动送达。一个新的 public Web 能力可能同时影响 SMS 登录用户、API-key 用户、Hone Cloud runner 和 entitlement。只靠分散 config bool，后续会越来越难做到可控灰度和快速止血。

## 问题或机会

这是 P1 级产品/架构问题：它不一定像输出安全门禁那样直接阻止危险答案，但会显著影响核心体验稳定性、事故恢复速度、发布节奏和运维信任。

### 问题

1. **功能开关散落在 config、代码常量、前端构建变量和局部 registry 中。**  
   当前 `event_engine.sources.*`、`storage.session_runtime_backend`、skill registry、channel enabled、public surface build、desktop mode、security tool guard 都有不同语义。维护者很难从一个地方看出“哪些高风险功能正在开启，面向谁开启，何时开启，由谁开启”。

2. **缺少 actor/surface/deployment 维度的灰度语义。**  
   很多能力不应该全量同时打开：public chat、desktop bundled、本地 admin、IM direct、group chat、scheduled proactive delivery、Hone Cloud API 的风险面不同。当前开关多为全局配置，难以表达“只给 owner actor / demo actor / 低风险 public cohort / desktop bundled / 非主动推送路径开放”。

3. **事故止血依赖改配置、重启或回滚代码。**  
   如果某个新 digest source 产生噪音、某个 skill script 输出异常、某个 public API 字段造成客户端误用、某个主动推送 enrichment 成本飙升，当前通常要改配置、禁用整个 event_engine/channel/skill，或发新版本。缺少稳定 reason code、TTL、审计和跨进程可见的 kill switch。

4. **前端、CLI、desktop 和 channel 对开关状态没有统一解释。**  
   一个功能被临时关闭后，public 用户、admin、desktop、IM 用户和 `hone-cli doctor/status` 可能看到不同错误。比如 capability 仍在 `/api/meta`，但运行路径被配置或内部降级关闭，用户会以为是 bug 而不是受控停用。

5. **灰度结果难以与已有观测资产连接。**  
   Hone 已经有 LLM audit、cron history、notification logs、task health、channel heartbeat、future run trace / safety / entitlement proposals。但缺少 feature flag id / rollout id，导致一次异常难以归因到“最近打开了哪个产品功能”。

6. **开源和 hosted 两种使用方式需要不同节奏。**  
   本地开源用户通常希望稳定、可解释和可手动控制；hosted/public 用户更适合快速灰度和运营分层。没有 rollout registry 时，两类模式只能共享同一个 config 结构，产品节奏会互相牵制。

### 机会

新增一个轻量但一等的 **Product Rollout and Kill-Switch Registry**，把“功能是否开放”从零散 bool 升级为可审计、可查询、可灰度、可紧急关闭的产品控制面。

这个提案的价值不是引入复杂在线实验平台，而是先提供四个能力：

1. 管理员知道当前哪些功能处于 `off`、`observe`、`canary`、`enabled`、`killed`。
2. 后端和前端能用同一个 decision API 判断某个 actor/surface/deployment 是否可用某功能。
3. 出事故时能用 TTL kill switch 关闭明确功能，而不是关闭整个 channel、event engine 或回滚 release。
4. 所有拒绝/降级都带稳定 reason code，能进入 logs、task health、notifications、future traces 和 support bundle。

## 方案概述

新增 `ProductRolloutRegistry`，作为 config 之上的运行时覆盖层。它不替代 canonical `config.yaml`，也不改变已有长期真相源，而是为产品功能提供一层可审计的 decision：

```text
canonical config + build capabilities + deployment mode + actor/surface context
  -> ProductRolloutRegistry
  -> FeatureDecision { enabled, mode, reason_code, expires_at, evidence }
```

核心对象：

- `FeatureKey`：稳定功能 ID，例如 `public_chat.attachments`, `event_engine.earnings_quality_review`, `session.sqlite_runtime_read`, `skills.chart_visualization.script`, `company_profiles.import_apply`, `hone_cloud.api_completions`, `multichannel.local_image_delivery`。
- `FeatureDefinition`：功能元数据，包含 owner module、risk level、default mode、dependencies、related config paths、affected surfaces、recommended kill behavior。
- `RolloutRule`：按 deployment、surface、actor、channel、cohort、percentage、time window、version window 决定 mode。
- `KillSwitch`：紧急覆盖，支持 reason、severity、scope、TTL、created_by、created_at、expires_at、rollback note。
- `FeatureDecision`：运行时判定结果，包含 `mode=off|observe|canary|enabled|killed`、`allowed`、`reason_code`、`user_message_key`、`operator_message`、`expires_at`、`matched_rule_id`。
- `RolloutAuditEvent`：创建/修改/删除 rule 与 kill switch 的审计记录，不存密钥和用户正文。

第一版建议覆盖 8 到 12 个高价值功能，而不是全仓所有 bool：

- `session.sqlite_runtime_read`
- `event_engine.direct_push`
- `event_engine.earnings_quality_review`
- `event_engine.sec_filing_enrichment`
- `global_digest.mainline_distill`
- `public_chat.api_completions`
- `public_chat.attachments`
- `company_profiles.import_apply`
- `skills.script_execution`
- `skills.chart_visualization`
- `multichannel.local_image_delivery`
- `desktop.bundled_channel_sidecars`

这些功能共同特点是：跨模块、可能影响用户可见体验、可能产生成本或主动推送、事故时需要快速关闭。

## 用户体验变化

### 用户端

- Public `/chat` 或 OpenAI-compatible API 如果某功能被 kill switch 关闭，返回稳定、简洁的用户态说明：
  - `feature_temporarily_disabled`
  - `attachment_input_paused`
  - `api_streaming_canary_unavailable`
  - `scheduled_delivery_paused`
- 用户不看到内部 config 路径或规则详情，只看到可执行下一步，例如“稍后重试”“使用文本输入”“联系管理员”。
- 已登录 public 用户的 `/me` 可以显示“服务状态”摘要：聊天可用、附件暂停、API 可用、自动任务暂停等，不暴露其它 actor 或全局内部细节。

### 管理端

- Dashboard 增加 `Rollouts and Kill Switches` 区块：
  - 当前 active kill switches。
  - 正在 canary/observe 的功能。
  - 最近 24h 因 feature decision 被拒绝或降级的运行数。
- Settings 或独立页面展示 feature 列表、默认状态、依赖、关联 config path、风险等级、当前 effective decision。
- 管理员可以创建 TTL kill switch，例如“暂停 event_engine direct push 2 小时，仅保留 digest buffer”，并填写 reason。
- Task Health、Notifications、LLM Audit 和未来 Run Trace detail 可以显示 `feature_decision_id`，帮助定位“不是 runner 失败，而是功能被受控关闭”。

### 桌面端

- Desktop bundled 模式展示本地 effective features：哪些能力由本地 backend 支持，哪些被远端/backend kill switch 暂停。
- Remote mode 只消费远端 `/api/rollouts/effective`，不在本地伪造远端开关。
- 当 bundled channel sidecars 被 kill switch 暂停时，desktop 显示“渠道监听已临时暂停”，而不是简单显示 stopped 或 failed。
- 本地开发者可以通过 CLI 创建本机-only kill switch，用于调试某个新功能而不改 `config.yaml`。

### 多渠道

- Feishu/Telegram/Discord/iMessage 在功能被暂停时使用统一短文案，不泄漏内部规则：
  - 主动推送暂停：默认不向用户刷屏，只在 task history / admin 记录原因。
  - direct chat 功能不可用：回复一条短消息说明功能维护中。
  - group chat 中高风险功能被关闭：保持现有 `ActorIdentity` / `SessionIdentity` 边界，不允许群成员绕过。
- Channel startup 可以读取 feature decisions，避免某些功能被 kill 后仍启动无效 sidecar 或注册误导性 heartbeat。

## 技术方案

### 1. Feature definition registry

在 `hone-core` 新增只含数据结构和 deterministic evaluator 的模块，例如：

```text
crates/hone-core/src/rollout.rs
```

第一版定义内置 feature manifest，后续可迁移到 `docs` 或 `resources`：

```rust
pub struct FeatureDefinition {
    pub key: String,
    pub owner: String,
    pub risk: FeatureRisk,
    pub default_mode: RolloutMode,
    pub surfaces: Vec<String>,
    pub dependencies: Vec<String>,
    pub related_config_paths: Vec<String>,
}
```

不要把 feature registry 做成第二套 config schema。它只描述产品功能，并在 evaluator 中引用现有 config。

### 2. Runtime overlay 存储

新增 runtime 覆盖文件，建议路径：

```text
data/runtime/product_rollouts.json
```

结构示例：

```json
{
  "version": 1,
  "updated_at": "2026-05-19T06:03:40Z",
  "rules": [
    {
      "id": "rule_public_attachments_owner_canary",
      "feature": "public_chat.attachments",
      "mode": "canary",
      "scope": { "surfaces": ["public_web"], "actors": ["web:owner"] },
      "starts_at": "2026-05-19T00:00:00Z",
      "expires_at": "2026-05-26T00:00:00Z",
      "reason": "owner canary before public rollout"
    }
  ],
  "kill_switches": [
    {
      "id": "kill_event_direct_push_20260519",
      "feature": "event_engine.direct_push",
      "mode": "killed",
      "scope": { "surfaces": ["channel_delivery"] },
      "reason_code": "event_noise_incident",
      "created_by": "admin",
      "created_at": "2026-05-19T06:03:40Z",
      "expires_at": "2026-05-19T08:03:40Z"
    }
  ]
}
```

这是 runtime state，不是 canonical config。删除 `data/runtime/` 后应回到默认功能状态；长期默认仍来自 `config.yaml` 和代码。

### 3. Decision API

Evaluator 输入：

- `HoneConfig`
- deployment mode: `local` / `desktop_bundled` / `desktop_remote` / `public_service`
- surface: `admin_web` / `public_web` / `openai_api` / `desktop` / `cli` / `channel_direct` / `channel_group` / `scheduler` / `event_engine`
- actor identity, optional
- channel id, optional
- build/runtime capabilities

输出：

```rust
pub struct FeatureDecision {
    pub feature: String,
    pub mode: RolloutMode,
    pub allowed: bool,
    pub reason_code: Option<String>,
    pub user_message_key: Option<String>,
    pub operator_message: Option<String>,
    pub matched_rule_id: Option<String>,
    pub expires_at: Option<String>,
}
```

判定顺序：

1. canonical config 是否已经关闭底层能力，例如 `event_engine.enabled=false`。
2. build/deployment 是否支持，例如 desktop remote 不能代表本地 sidecar。
3. dependency 是否满足，例如 skill script execution 需要 skill enabled、tool guard policy 和 runner stage 支持。
4. active kill switch 是否命中。
5. rollout rules 是否命中。
6. feature default mode。

### 4. 接入点

第一阶段只接只读展示和低风险高价值路径：

- `/api/meta` 保持原 capability 列表，但可新增 `rollout_features` 摘要或新增独立 `/api/rollouts/effective`。
- `crates/hone-web-api/src/routes/public.rs` 在 public API/chat 入口检查 `public_chat.api_completions`、未来附件检查 `public_chat.attachments`。
- `crates/hone-tools/src/skill_registry.rs` 仍然是 skill enabled 真相源；`turn_builder` 和 `skill_tool` 在 script/高副作用 skill 上额外检查 `skills.script_execution` 或具体 feature。
- `crates/hone-event-engine` 在 direct push sink 前检查 `event_engine.direct_push`；kill 时仍允许入库和 digest buffer，避免数据丢失。
- `crates/hone-channels/src/scheduler.rs` 在主动发送前读取 scheduler/event-related decisions，把被 kill 的运行记录为 `skipped_feature_disabled`。
- Desktop channel sidecar startup 在启动前检查 `desktop.bundled_channel_sidecars`。

### 5. API、CLI 与前端

新增 admin-only API：

- `GET /api/rollouts/features`
- `GET /api/rollouts/effective?surface=&actor=&channel=`
- `POST /api/rollouts/rules`
- `POST /api/rollouts/kill-switches`
- `POST /api/rollouts/kill-switches/:id/expire`
- `GET /api/rollouts/audit?feature=&from=&to=`

Public API 只暴露当前用户可见摘要：

- `GET /api/public/service-status`

CLI：

- `hone-cli status --json` 增加 `rollouts` 摘要。
- `hone-cli rollouts list`
- `hone-cli rollouts kill <feature> --ttl 2h --reason <code>`
- `hone-cli rollouts expire <id>`

前端：

- `packages/app/src/lib/rollouts.ts`
- `packages/app/src/context/rollouts.tsx`
- Dashboard rollouts panel
- Settings/Operations 页面中的 feature list + kill switch drawer
- Task Health / Notifications 行内展示 feature-disabled reason

### 6. 审计与观测

第一版可以把 rollout audit 写入 SQLite 或 JSONL。建议放在 `memory` 或 `hone-core` 外的 runtime store，避免过早污染业务存储：

```text
data/runtime/rollout_audit.jsonl
```

字段：

- `event_id`
- `created_at`
- `operator_actor` or `source`
- `action`
- `feature`
- `rule_id` / `kill_switch_id`
- `before_hash`
- `after_hash`
- `reason_code`

运行时拒绝事件不需要全部落完整日志，避免噪声过大；可以按 feature + reason 做 counter，并在 task/run detail 上保留相关 decision id。

### 7. 兼容与迁移

- 不删除现有 config bool。Feature decision 只能进一步限制或解释功能，不能偷偷启用 canonical config 已关闭的能力。
- 不替代 skill registry。Skill registry 仍回答“这个 skill 是否启用”；rollout registry 回答“这个产品功能/执行方式是否对当前上下文开放”。
- 不替代 permission broker。Permission broker 管理 agent runtime action 的批准；rollout registry 管理产品功能是否进入该路径。
- 不替代 entitlement。Entitlement 管理用户是否有权益消耗某能力；rollout registry 管理平台是否开放该能力。
- 不替代 readiness。Readiness 管理当前配置是否能跑；rollout registry 管理当前产品策略是否允许跑。
- 不替代 update compatibility。Compatibility 管理版本组合是否支持；rollout registry 管理支持窗口内是否启用。

## 实施步骤

### Phase 1: Feature definitions and read-only effective decisions

- 在 `hone-core` 定义 feature/rollout/decision 类型和 deterministic evaluator。
- 内置 8 到 12 个高价值 `FeatureDefinition`。
- 读取可选 `data/runtime/product_rollouts.json`；不存在时返回默认 decisions。
- 新增 `/api/rollouts/features` 和 `/api/rollouts/effective` 只读 API。
- `hone-cli status --json` 增加 rollouts 摘要，但不改变现有启动行为。

### Phase 2: Admin-visible kill switches

- 新增 kill switch 写入/过期 API 与本地 CLI 命令。
- Dashboard / Settings 增加 active kill switches 和 feature list。
- 写入 `rollout_audit.jsonl`。
- 支持 TTL 到期自动忽略，不要求后台常驻清理。

### Phase 3: Low-risk enforcement paths

- Public API/chat 对明显关闭的 `public_chat.api_completions` 返回 stable error。
- Event-engine direct push sink 支持 `killed -> buffer/digest/log only`，不丢事件。
- Scheduler 发送前支持 `skipped_feature_disabled` execution record。
- Skill script execution 在现有 skill enabled 基础上检查 `skills.script_execution`。

### Phase 4: Cross-surface product integration

- Desktop bundled/remote 展示有效 decisions。
- Task Health、Notifications、Logs、future Run Trace 引用 feature decision id。
- Public `/me` 或 `/api/public/service-status` 展示用户可见服务状态。
- 把 canary/observe decision 与 LLM audit、usage event、notification outcomes 关联，支持后续 rollout 成功率判断。

### Phase 5: Controlled canary rules

- 支持 actor allowlist、surface allowlist、deployment mode 和 percentage cohort。
- Percentage cohort 必须基于稳定 actor hash，不能每次请求随机变化。
- Canary 默认禁止主动高风险推送，除非 rule 明确允许 `scheduler` / `event_engine` surface。
- Promote 只生成建议或 config mutation 草稿，不自动改 canonical config。

## 验证方式

- 单元测试：
  - canonical config 关闭时，rollout rule 不能把功能重新打开。
  - active kill switch 优先级高于 canary/enabled rule。
  - expired kill switch 不再生效。
  - actor allowlist 和 stable percentage cohort 判定稳定。
  - deployment/surface 条件正确区分 public Web、OpenAI API、desktop bundled、desktop remote、scheduler 和 channel direct。
- Web API 测试：
  - 非 admin 不能创建/过期 kill switch。
  - `/api/rollouts/effective` 不返回密钥或敏感路径。
  - public service-status 只能返回当前用户可见摘要。
- Runtime contract 测试：
  - `event_engine.direct_push=killed` 时事件仍入库，direct sink 不发送，execution/log 记录 `skipped_feature_disabled`。
  - `public_chat.api_completions=killed` 时 OpenAI-compatible route 返回稳定 error code 和 request id。
  - `skills.script_execution=killed` 时普通 skill listing 可继续展示，但脚本执行路径硬拒绝并说明原因。
- 前端测试：
  - feature decision model 能把 `killed`、`canary`、`observe`、`enabled` 显示为正确状态。
  - active kill switch drawer 对 TTL、reason、scope 做校验。
  - 旧后端没有 rollouts API 时 Dashboard graceful degrade。
- 手工验收：
  - 创建一个 30 分钟 kill switch，确认 admin UI、CLI status、public route、scheduler/event path 看到一致状态。
  - TTL 到期后无需手工改文件，effective decision 自动恢复默认。
  - 删除 `data/runtime/` 后 canonical config 不丢失，rollout overlay 回到默认。

## 风险与取舍

- 风险：再加一层开关会让系统更复杂。  
  取舍：第一版只覆盖少数高风险跨模块功能；不把所有 config bool 都迁进 rollout registry。

- 风险：runtime overlay 和 canonical config 可能让用户困惑。  
  取舍：明确规则是“canonical config 决定能力是否存在，rollout overlay 只能进一步限制或灰度”。UI 必须展示相关 config path 和当前 decision 来源。

- 风险：kill switch 被滥用会掩盖真正 bug。  
  取舍：所有 kill switch 必须有 reason、TTL 和 audit；长期关闭应转成 config/代码修复，不允许无期限静默关闭高价值功能。

- 风险：canary 对主动推送路径可能影响真实用户。  
  取舍：第一版不做 percentage canary enforcement；后续 canary 默认只允许 direct/admin/desktop 低风险 surface，高风险 scheduler/event_engine 需要显式 allow。

- 风险：percentage cohort 引入隐私或不稳定行为。  
  取舍：只使用 actor key 的稳定 hash，不存额外画像；public UI 只展示用户可见结果，不展示 cohort 细节。

- 风险：与 entitlement、permission broker、skill trust marketplace 产生边界重叠。  
  取舍：本提案只回答“平台/部署是否开放该功能”。用户是否有额度由 entitlement 决定，agent action 是否可批准由 permission broker 决定，第三方 skill 是否可信由 skill trust marketplace 决定。

## 与已有提案的差异

本轮查重范围包括 `docs/proposal/` 和 `docs/proposals/` 下全部现有提案。相邻但不重复的主题如下：

- `auto_p1_runtime_readiness_matrix.md` 回答“当前配置、路径、凭证和进程能否支撑某个能力运行”；本提案回答“即便能运行，产品策略是否允许当前 actor/surface/deployment 使用它，以及事故时如何暂停”。
- `auto_p1_update-compatibility-center.md` 回答“安装版本、Web bundle、sidecar、API window 是否兼容”；本提案工作在兼容版本之内，控制功能灰度和 kill switch。
- `auto_p1_model-route-evaluation-lab.md` 专门评估模型/provider/profile 的离线质量、canary 和 promote；本提案覆盖所有产品功能，不决定模型质量，只可消费其结果作为某个 feature/rule 的依据。
- `auto_p0_investment_output_safety_gate.md` 是运行时输出安全门禁；本提案可以临时关闭某个输出路径或让 safety gate 的 observe/enforce rollout 可见，但不替代安全判定本身。
- `auto_p1_usage_entitlement_ledger.md` 管理用户权益、用量和成本；本提案不计费，只控制平台是否开放某功能。
- `auto_p1_agent-permission-broker.md` 管理 agent 文件/命令/工具动作的批准；本提案在更外层决定某产品功能是否进入该动作路径。
- `auto_p1_skill-trust-marketplace.md` 管理 skill 安装、可信度、兼容和回滚；本提案只为 skill script execution、chart visualization 等产品功能提供灰度/kill switch，不做 skill package 治理。
- `docs/proposals/desktop-bundled-runtime-startup-ux.md` 解决 desktop startup ownership 和 process recovery；本提案只让 desktop 能显示和消费功能决策，不改变进程锁接管策略。

查重结论：现有 proposal 已覆盖运行 readiness、版本兼容、模型评估、安全输出、权益、权限、skill 可信和桌面启动，但尚未覆盖“跨产品功能的统一灰度、actor/surface/deployment scoped rollout、TTL kill switch、feature decision reason code 和跨观测资产归因”。因此本主题是新的、可落地的 P1 产品/架构提案。

## 文档同步说明

本轮只创建 proposal，不开始实施，不修改业务代码、测试代码、运行配置或 `docs/current-plan.md`。若后续实际落地该提案，应按动态计划准入标准新增或复用 `docs/current-plans/product-rollout-kill-switch.md`，并在新增 rollout 类型、runtime overlay、API、CLI 命令、前端页面或执行路径 enforcement 时同步更新 `docs/repo-map.md`、`docs/invariants.md`、必要的 decision/ADR 和相关 runbook。
