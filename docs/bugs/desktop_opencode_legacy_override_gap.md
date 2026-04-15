# Bug: Desktop legacy runtime 会整块覆盖 canonical `agent.opencode` 配置，破坏本机 OpenCode 继承语义

- **发现时间**: 2026-04-14
- **Bug Type**: System Error
- **严重等级**: P1
- **状态**: New
- **证据来源**:
  - 最近提交: `dfd8a01 fix: restore desktop canonical agent config migration`
  - 最近提交: `e802582 fix: migrate desktop legacy runtime user settings`
  - 2026-04-15 当前源码复核: `crates/hone-core/src/config.rs:680-685` 仍以 `agent.opencode.api_key` 是否为空为门槛，并在命中时整块回填 legacy `agent.opencode`
  - 代码证据:
    - `crates/hone-core/src/config.rs:680-685`
    - `docs/invariants.md:90-92`
    - `docs/current-plans/canonical-config-runtime-apply.md:31-34`

## 端到端链路

1. 桌面端升级用户已经进入 canonical config 时代，并在新的设置链路里配置了 `agent.opencode.model`、`variant` 或 `api_base_url`。
2. 这类用户经常会故意把 `agent.opencode.api_key` 留空，因为当前产品约定是：当 `agent.opencode.*` 留空时，`opencode_acp` 应默认继承用户本机 OpenCode 配置，而不是强制改写成本地 Hone overlay。
3. 桌面启动时，`ensure_runtime_paths(...)` 会调用 `promote_legacy_runtime_agent_settings(...)`，尝试把旧版 `data/runtime/config_runtime.yaml` 中还没进 canonical 的设置补迁到 `config.yaml`。
4. 当前迁移逻辑只用 `agent.opencode.api_key` 是否为空作为门槛，但一旦命中，就会把 legacy 的整个 `agent.opencode` 对象整块写回 canonical。
5. 结果是：canonical 中用户刚配置好的 `model`、`variant`、`api_base_url`，甚至“留空以继承本机 OpenCode 配置”的语义，都会被旧版 runtime 的整块 `agent.opencode` 覆盖。

## 期望效果

- Desktop 升级迁移应只补齐 canonical 中真正缺失的 `agent.opencode` 字段，而不是因为 `api_key` 为空就整体回填 legacy 对象。
- 如果用户在 canonical config 中主动把 `agent.opencode.api_key` 留空，系统应继续遵守“继承本机 OpenCode 配置”的长期约束，而不是被旧 runtime 静默写回一个过期 key 或旧模型。
- 用户在新设置页里选定的 `model` / `variant` / `api_base_url` 应在升级后稳定保留，不应被 legacy runtime 值回滚。

## 当前实现效果（问题发现时）

- `promote_legacy_runtime_agent_settings(...)` 对 `agent.opencode` 的迁移条件只有 `string_path_is_blank(&canonical, "agent.opencode.api_key")`。
- 一旦这个条件成立，代码会直接执行 `set_value_at_path(&mut canonical, "agent.opencode", legacy_opencode.clone())`，把整个 `agent.opencode` 节点替换掉。
- `docs/invariants.md` 已明确要求：当 `agent.opencode.model` / `api_base_url` / `api_key` 为空时，Hone 应继承用户本机 OpenCode 配置；当前整块覆盖行为和这一约束相冲突。
- 现有测试只覆盖了“canonical 已配置完整 key 时不要覆盖”的情况，没有覆盖“canonical 已自定义 model/variant，但故意留空 api_key”的升级场景。

## 用户影响

- 用户可能在新版设置页里已经完成模型切换，但下一次 desktop 启动后又被旧 runtime 配置悄悄改回去，表现为模型、路由或鉴权行为异常回退。
- 对依赖本机 OpenCode 默认登录态的用户来说，legacy 里的旧 `api_key` 被重新写回后，`opencode_acp` 可能不再继承本机配置，而是改走过期或错误的 Hone 覆盖配置，直接导致回答失败或跑错模型。
- 这是典型的“升级后表面能启动、但默认对话链路行为被静默改坏”的问题，用户很难从 UI 直接看出根因。

## 当前实现效果（2026-04-15 复核）

- 当前 HEAD 仍在 `string_path_is_blank(&canonical, "agent.opencode.api_key")` 命中后直接执行 `set_value_at_path(&mut canonical, "agent.opencode", legacy_opencode.clone())`。
- 也就是说，只要 canonical 侧故意把 `api_key` 留空以继承本机 OpenCode，迁移代码仍会把 legacy 整个 `agent.opencode` 节点写回，问题尚未被最近提交覆盖。
- 本轮巡检未发现对应修复提交，因此该缺陷继续保持 `New`。

## 根因判断

- legacy 补迁逻辑把 `agent.opencode` 当成“只要 key 为空就整体未配置”，但当前产品契约已经不是这样：`api_key` 为空本身就是一种有效配置，表示继承本机 OpenCode。
- 迁移代码没有按字段粒度处理 `agent.opencode.model`、`variant`、`api_base_url`、`api_key`，而是用了整块对象覆盖，导致 canonical 中的有效新配置被 legacy 值回滚。
- 自动化测试缺少“canonical 保留空 key 继承语义”的回归样例，因此这个缺口没有被最近两次迁移修复捕获。

## 修复线索

- 把 `agent.opencode` 的 legacy 迁移改成字段级 merge，而不是整块覆盖；至少要区分“字段缺失”与“显式留空表示继承本机 OpenCode”。
- 增加一条回归测试：canonical 预设 `agent.opencode.model` / `variant`，`api_key` 为空，legacy 含旧 `agent.opencode` 时，迁移后 canonical 不应被整块覆盖。
- 当前 bug 台账先以 `New` 登记，等待人工确认并转入 `Approved` / `Fixing` / `Fixed` / `Closed`。
