# Proposal: Config Apply Evidence Center for Canonical, Effective, and Running State

status: proposed
priority: P1
created_at: 2026-05-31 20:05:40 +0800
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
- `docs/proposal/auto_p1_agent-mutation-ledger.md`
- `docs/proposal/auto_p1_runtime_readiness_matrix.md`
- `docs/proposal/auto_p1_update-compatibility-center.md`
- `docs/proposal/auto_p1_channel-activation-proof.md`
- `docs/proposal/auto_p2_signal-source-lab.md`
- `crates/hone-core/src/config/mutation.rs`
- `crates/hone-core/src/config/materialize.rs`
- `crates/hone-core/src/config/yaml.rs`
- `crates/hone-web-api/src/routes/channel_settings.rs`
- `crates/hone-web-api/src/routes/meta.rs`
- `crates/hone-web-api/src/routes/event_engine_admin.rs`
- `bins/hone-cli/src/yaml_io.rs`
- `bins/hone-cli/src/main.rs`
- `bins/hone-cli/src/mutations.rs`
- `bins/hone-desktop/src/sidecar.rs`
- `bins/hone-desktop/src/sidecar/settings.rs`
- `bins/hone-desktop/src/sidecar/runtime_env.rs`
- `packages/app/src/pages/settings.tsx`
- `packages/app/src/pages/settings-model.ts`
- `packages/app/src/lib/types.ts`

## 背景与现状

Hone 已经把配置源从历史 runtime snapshot 收口到 canonical `config.yaml`，并在启动或保存后生成 `data/runtime/effective-config.yaml` 供子进程读取。当前代码里已经有清晰基础：

- `crates/hone-core/src/config/materialize.rs` 明确区分 canonical、legacy runtime config 和 effective config，并通过 `generate_effective_config` 输出只读快照。
- `promote_legacy_runtime_agent_settings` 和 `normalize_runtime_storage_rollout_settings` 会在启动/生成 effective 时保守迁移或修正常见历史字段，保护用户旧配置。
- `crates/hone-core/src/config/mutation.rs` 提供 `apply_config_mutations`、`apply_overlay_mutations`、`classify_config_paths`、敏感字段识别和脱敏。
- CLI 的 `bins/hone-cli/src/yaml_io.rs` 已经把 mutation 写入 canonical 后立即重新生成 effective，并返回 `ConfigApplyPlan`：热生效、需重启组件或需全量重启。
- Web API 的 `channel_settings.rs` 直接写 canonical 并生成 effective，但在普通 Web/CLI runtime 下只能提示“需要重启后拉起或停止渠道监听器”。
- Desktop 的 `sidecar.rs` / `sidecar/settings.rs` 会在 bundled 模式保存 agent、FMP、Tavily、channel 设置后标记 bundled runtime dirty 并重启，使部分设置立即生效。
- `event_engine_admin.rs` 使用 `apply_overlay_mutations` 写入 `<config>.overrides.yaml`，以避免覆盖用户手写 YAML 注释；但它也说明部分 source/scheduler 变更需要重启后才进入运行中配置。

这些实现已经解决了“配置应该写哪里”和“哪些路径大致需要重启”的工程问题，但还没有一个产品级的 **Config Apply Evidence**：用户或管理员保存设置后，很难确认 canonical、overlay、effective snapshot、当前 backend 内存配置、channel sidecar、desktop bundled runtime 和 remote backend 是否已经一致。

这在 Hone 当前形态下会越来越关键。一个配置项可能从 CLI、Desktop Tauri command、Web Settings、event-engine admin API、onboard 流程或历史迁移写入；生效路径又可能是 hot apply、effective snapshot regeneration、channel process restart、bundled backend restart、remote backend next-start only、full process restart。缺少证据层时，用户只看到“已保存”，但真正的失败会在第一条聊天、第一条渠道消息、第一轮 digest 或第一次 model route 调用时暴露。

## 问题或机会

这是 P1。它不直接替代 P0 安全边界，但会显著提升首次配置成功率、桌面可信度、远端部署支持效率和多渠道稳定性。

1. **保存成功不等于运行中生效。**  
   `ConfigApplyPlan` 能分类路径，但当前多处调用只把结果翻译成简短文案。运行中的 backend 是否加载了同一个 revision、channel sidecar 是否拿到新 effective、desktop 是否完成 restart、event-engine source 是否仍是旧 snapshot，都没有统一证据。

2. **同一设置在不同 surface 的 apply 语义不一致。**  
   CLI `config set` 会重生成 effective；Web channel settings 保存后不重启普通 runtime；Desktop bundled 会 dirty + restart；event-engine source 写 overlay 并返回 restart hint；language 写 overlay 但 meta 中运行时语言可能仍来自当前 `state.core.config`。这些差异对用户都是“设置页保存”，但实际生效窗口不同。

3. **canonical / overlay / effective / running 缺少可视 diff。**  
   `apply_overlay_mutations` 保留 base YAML 注释很合理，但用户和管理员需要知道：某个字段来自 base 还是 overlay，effective 是哪个 revision，运行进程当前读的是哪个 revision，是否有 pending restart。

4. **支持和回滚缺少最小现场证据。**  
   当前可以从日志、status、doctor、settings、meta、channel status 里拼接问题，但没有一条标准记录回答“谁/哪个 surface 改了哪些 path，旧值/新值摘要是什么，写入了哪个文件，生成了哪个 effective revision，哪些组件重启成功/失败，如何回滚”。

5. **后续 proposal 会反复需要配置生效证明。**  
   Runtime Readiness、Channel Activation Proof、Signal Source Lab、Update Compatibility、Secrets Vault、Operator Access Audit 都会触及配置写入、重启、密钥、版本窗口或 proof。没有横向 apply evidence，后续每个功能都会重复实现局部 pending diff 和 restart 状态。

机会是：在不改变 canonical config source of truth 的前提下，新增一个轻量 `ConfigApplyEvidenceCenter`，把已有 mutation result、effective revision、process heartbeat、desktop restart result 和 overlay diff 汇总成可查询、可回滚、可解释的配置生效证据。

## 方案概述

新增 **Config Apply Evidence Center**：围绕每一次配置写入生成 `ConfigApplyRecord`，并派生出当前 `ConfigRuntimeDrift` 视图。

核心对象：

- `ConfigApplyRecord`
  - `apply_id`
  - `source_surface`: `cli`、`web_admin`、`desktop`、`event_engine_admin`、`onboard`、`migration`
  - `operator_ref` / `actor_ref`：第一版可为空或 legacy bearer；未来接 Operator Access Audit
  - `config_path`
  - `overlay_path`
  - `effective_config_path`
  - `before_revision`
  - `after_canonical_revision`
  - `after_effective_revision`
  - `changed_paths`
  - `redacted_before_after`
  - `apply_plan`
  - `component_results`
  - `status`: `saved`、`effective_written`、`hot_applied`、`restart_pending`、`restart_succeeded`、`restart_failed`、`rolled_back`

- `ConfigRuntimeDrift`
  - 当前 canonical / overlay / effective revision
  - backend loaded revision
  - 各 channel heartbeat 携带的 config revision
  - pending restart paths 和 components
  - base vs overlay source map
  - 最近一次成功/失败 apply 记录

- `ConfigRollbackPlan`
  - 针对某条 apply 记录生成反向 mutation 或 restore snapshot
  - 默认只恢复本次 touched paths
  - 对 secret path 只允许恢复 redacted-safe snapshot 引用，不在 UI 展示明文

第一版目标不是新建一个复杂配置管理系统，而是把已有 mutation 管道升级成“保存后有证据、未生效可见、回滚有入口”。

## 用户体验变化

### 用户端

- Public 用户通常不看到底层配置证据；当服务配置正在重启或未生效时，只看到稳定的短状态，例如“服务设置正在更新，请稍后重试”。
- 如果未来开放用户自助通知设置或 API key 设置，用户能看到“已保存，下一次推送起生效”或“需要管理员完成渠道验证”这类清晰状态。

### 管理端

- Settings 页增加 `Apply evidence` 抽屉：
  - 最近配置变更时间、来源、changed paths、脱敏 diff。
  - 是否写入 effective config。
  - 哪些组件已热生效、哪些组件需要重启、哪些重启失败。
  - 当前 backend / channel sidecar 读取的 config revision。
- Channel 设置保存后不只显示泛化文案，而是展示“Telegram settings saved, effective revision X written, telegram sidecar restart pending/finished”。
- Event-engine source 保存后可看到 configured vs running diff，而不是只看到 `needs_restart=true`。
- 管理员可以导出 redacted apply evidence 到 support bundle，降低排障成本。

### 桌面端

- Desktop bundled 模式保存 agent / data / channel 设置后，展示一条完整状态：
  - canonical written
  - effective written
  - bundled backend stopped / started
  - `/api/meta` connected with revision
  - channel sidecars restarted or pending
- Remote mode 明确显示“远端 backend 返回的 apply state”，不暗示本地 Tauri 已经重启远端进程。
- 如果 bundled restart 失败，用户看到“配置已保存但未生效，可重试重启或回滚本次设置”，而不是只看到 backend disconnected。

### 多渠道

- 对 channel runtime，heartbeat payload 可带 `config_revision` / `effective_config_mtime`，让 `/api/channels` 区分“进程在线但仍读旧配置”和“已拿到新配置”。
- 当用户在 IM 中问“为什么 Telegram 任务没发”，agent 或管理端可以引用最近 channel config apply record，而不是让用户重复保存设置。

## 技术方案

### 1. 记录 ConfigApplyRecord

建议在 `memory` 新增 `config_apply` 存储，local mode 用 SQLite；cloud mode 后续用 PG。记录只保存必要摘要，不保存完整敏感配置。

建议表：

```text
config_apply_records (
  apply_id TEXT PRIMARY KEY,
  source_surface TEXT NOT NULL,
  source_ref TEXT,
  config_path TEXT NOT NULL,
  overlay_path TEXT,
  effective_config_path TEXT,
  before_revision TEXT,
  after_canonical_revision TEXT,
  after_effective_revision TEXT,
  changed_paths_json TEXT NOT NULL,
  redacted_diff_json TEXT NOT NULL,
  apply_plan_json TEXT NOT NULL,
  component_results_json TEXT NOT NULL,
  status TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
)
```

`redacted_diff_json` 使用现有 `is_sensitive_config_path` / `redact_sensitive_value`，对 `api_key`、`secret`、`token`、`password` 只存 `changed: true`、长度区间、prefix/suffix hash 或 `was_empty -> is_empty`，不存明文。

### 2. 把 mutation 管道升级为 record-aware

保留现有 `apply_config_mutations` / `apply_overlay_mutations` 作为纯写入能力，在调用层新增 wrapper：

- `apply_config_mutations_with_evidence`
- `apply_overlay_mutations_with_evidence`

wrapper 负责：

1. 读取 before revision 和 touched path before values。
2. 调用现有 mutation 函数。
3. 生成 effective config（如该 surface 需要）。
4. 写入初始 `ConfigApplyRecord`。
5. 根据 apply plan 记录 hot/live/restart pending。
6. 如果调用方执行了 desktop/backend/channel restart，再更新 component result。

CLI、Web、Desktop 不需要一次性全部迁移；第一阶段可先让 CLI `config set/unset`、Desktop agent settings、Web channel settings、event-engine source 使用 wrapper。

### 3. 运行中 revision 证明

新增轻量 revision 传播：

- `generate_effective_config` 返回的 revision 已存在；把它写入 runtime file sidecar 或 `/api/meta`。
- `AppState` 启动时记录 `loaded_config_revision`。
- channel bootstrap 读取 effective config 后在 heartbeat 中附带 `loaded_config_revision`。
- Desktop sidecar restart 完成后读取 `/api/meta` 的 revision，更新 `component_results`.

`/api/meta` 可新增：

```json
{
  "configRevision": "...",
  "effectiveConfigRevision": "...",
  "configApply": {
    "lastApplyId": "...",
    "pendingRestart": true
  }
}
```

旧前端忽略新增字段，保持兼容。

### 4. Drift API

新增 admin/desktop API：

- `GET /api/config-apply/records?limit=20&path_prefix=agent`
- `GET /api/config-apply/records/:id`
- `GET /api/config-apply/drift`
- `POST /api/config-apply/records/:id/rollback-preview`
- `POST /api/config-apply/records/:id/rollback`

`drift` 输出：

- base config revision
- overlay revision
- effective revision
- backend loaded revision
- channel process revisions
- pending restart components
- changed paths since last successful full restart

Public API 不开放完整 drift；只在必要时返回 coarse service state。

### 5. 回滚策略

第一版只支持低风险 preview：

- 对普通 path，反向 mutation 恢复 before value 或 unset。
- 对 secret path，如果 before value 没有安全 snapshot，则只提示“需要重新输入旧密钥”，不从日志恢复明文。
- 对 overlay mutation，回滚应写 overlay，不改 base。
- 对 full restart path，回滚后仍要求重启。

回滚动作本身也写一条新的 `ConfigApplyRecord`，并关联 `rollback_of_apply_id`。

### 6. 前端落点

- `packages/app/src/lib/config-apply.ts`
- `packages/app/src/context/config-apply.tsx`
- Settings 页顶部 `Last apply` status strip。
- Channel / Agent / Data 子页保存后展开本次 apply result。
- Dashboard / Logs 可显示“configuration drift warning”。

第一版不要做复杂 YAML 编辑器；只做 records、diff、drift、rollback preview。

## 实施步骤

### Phase 1: Record and drift skeleton

- 新增 `ConfigApplyRecord` 类型、SQLite store 和 redacted diff helper。
- CLI `config set/unset` 使用 evidence wrapper。
- `/api/config-apply/records` 和 `/api/config-apply/drift` 返回基础记录。
- `hone-cli config set --json` 增加 `apply_id`，文本模式仍保持当前简洁文案。

### Phase 2: Desktop and Web settings integration

- Desktop agent / FMP / Tavily / channel settings 写入 apply record。
- Web channel settings 和 language / event-engine overlay 写入 apply record。
- Desktop bundled restart 成功/失败后更新 component result。
- Settings 页显示本次保存的 apply evidence 和 pending restart。

### Phase 3: Runtime revision propagation

- `/api/meta` 返回 loaded config/effective revision。
- channel heartbeat 带 loaded config revision。
- `/api/channels` 和 drift API 标出旧 revision sidecar。
- Dashboard 增加 drift warning。

### Phase 4: Rollback preview and support bundle

- 增加 rollback preview / rollback API。
- secret path 回滚需要用户重新输入或选择“清空”，不恢复明文。
- Support bundle 导出最近 N 条 redacted apply records。
- 与 Operator Access Audit / Agent Mutation Ledger 建立可选关联字段。

## 验证方式

- Rust 单元测试：
  - `redacted_diff_json` 对 secret path 不含明文，只保留变更摘要。
  - apply record 能保存 changed paths、before/after revision、apply plan。
  - overlay apply 不修改 base config，record 标明 overlay path。
  - rollback preview 对 set/unset、secret path、overlay path 生成正确计划。
  - `classify_config_paths` 的 result 能完整序列化进入 apply record。

- API 测试：
  - `GET /api/config-apply/drift` 在无记录时返回空状态，不 500。
  - admin 可读取 records；public 不可读取。
  - records 不泄露 token、API key、password、secret。
  - backend loaded revision 与 effective revision 不一致时返回 pending restart。

- Desktop / frontend 测试：
  - 保存 agent settings 后 UI 显示 apply id、restart result 和 message。
  - remote mode 不展示本地 restart 成功假象。
  - channel sidecar revision 旧于 effective revision 时显示 pending restart。

- 手工验收：
  - CLI `hone-cli config set agent.runner opencode_acp --json` 返回 apply id 和 revision。
  - Web 保存 Telegram token 后，Settings 能看到 token changed 但无明文 diff。
  - Desktop bundled 保存 model 后，backend 重启成功并在 drift 中显示 loaded revision 已追上 effective。
  - 修改 event-engine RSS source 后，drift 显示 overlay 写入且 scheduler running snapshot 仍 pending restart，重启后 pending 消失。

- 指标：
  - “保存设置但未生效”支持问题下降。
  - pending restart 平均持续时间。
  - config rollback 成功率。
  - settings 保存后首次 chat/channel failure 中 config drift 相关占比。

## 风险与取舍

- **风险：和 Agent Mutation Ledger 重叠。**  
  取舍：Mutation Ledger 记录所有用户长期状态变更及可撤销材料；本提案只专注配置写入到运行中生效的证据、revision drift、component apply result 和 config-specific rollback。未来两者可通过 `mutation_record_id` / `apply_id` 关联。

- **风险：记录配置 diff 可能泄露密钥。**  
  取舍：沿用敏感 path 检测；secret diff 默认只记录 changed/empty/hash，不记录明文；support bundle 也只用 redacted record。

- **风险：revision 传播增加心智负担。**  
  取舍：普通用户只看“已生效 / 需重启 / 重启失败”；管理员和桌面调试才展开 revision。

- **风险：回滚可能和手工编辑 YAML 冲突。**  
  取舍：rollback 前必须重新读取 current revision；若 touched path 已被后续记录或手工变更改写，则要求重新 preview，不强行覆盖。

- **风险：overlay/source map 实现复杂。**  
  取舍：第一版只标明 record 写 base 还是 overlay；字段级 source map 可在 signal source / event engine 等高价值区域先落地。

- **不做：**
  - 不改变 `config.yaml` 作为长期用户配置源。
  - 不引入中心化配置服务。
  - 不自动重启 remote backend。
  - 不把完整 config 文件写入 SQLite。
  - 不绕过现有 `apply_config_mutations` 校验。

## 与已有提案的差异

查重范围覆盖 `docs/proposal/` 与 `docs/proposals/` 下全部现有提案，并重点核对了 `mutation ledger`、`readiness`、`update compatibility`、`channel activation proof`、`signal source lab`、`secrets vault`、`operator access audit`、`runtime dependency circuit breaker` 等相关主题。

- 不重复 `auto_p1_agent-mutation-ledger.md`：该提案记录 agent/UI 对 portfolio、notification prefs、company profile、settings、cron 等长期状态的 before/after、确认与撤销；本提案专注配置生效链路，回答 canonical/overlay/effective/running 是否一致以及哪些组件仍 pending restart。
- 不重复 `auto_p1_runtime_readiness_matrix.md`：Readiness 判断当前配置和依赖是否足以完成产品动作；本提案记录配置写入后是否已经进入运行进程。Readiness 可以消费 drift 状态，但不替代 apply evidence。
- 不重复 `auto_p1_update-compatibility-center.md`：Update Compatibility 处理安装包、版本、API window、升级和回滚；本提案处理同一版本内用户配置变更的 apply、restart、drift 和 rollback。
- 不重复 `auto_p1_channel-activation-proof.md`：Channel Activation Proof 证明外部渠道 target 可投递；本提案证明渠道配置变更是否已经被 channel sidecar 读取。二者组合后才能说明“配置已生效且目标可达”。
- 不重复 `auto_p2_signal-source-lab.md`：Signal Source Lab 针对 RSS/Telegram source 做 probe、trial、enable、pending restart；本提案抽象出所有配置 path 通用的 evidence/drift/rollback 底座，source lab 可以复用它。
- 不重复 `auto_p0_operator-access-audit.md`：Operator Audit 关注谁有权限、谁做了管理动作；本提案关注动作产生的配置文件和运行时生效证据。未来应把 operator_id 写入 apply record。

查重结论：现有提案覆盖状态变更审计、运行前可用性、版本兼容、渠道投递验证和事件源生命周期，但没有覆盖“配置从 canonical 写入到 effective snapshot、backend/channel/desktop 运行中配置追平”的横向证据层。因此本主题是新的、可执行的 P1 提案。

## 文档同步说明

本轮只新增 proposal，不开始实施方案，不修改业务代码、测试代码、运行配置或 `docs/current-plan.md`。该任务属于自动化单次提案产出，不进入动态计划、不新增 handoff、无需归档计划页。若后续开始实现，应按动态计划准入标准新增或复用 `docs/current-plans/config-apply-evidence.md`，并在新增 store、API、CLI 输出、desktop restart evidence、channel heartbeat revision 或 rollback 行为后同步更新 `docs/repo-map.md`、`docs/invariants.md`、相关 runbook 和必要 decision/ADR。
