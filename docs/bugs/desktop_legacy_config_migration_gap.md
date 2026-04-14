# Bug: Desktop 启动时未完整迁移 legacy runtime 用户配置

- **发现时间**: 2026-04-14
- **Bug Type**: Business Error
- **严重等级**: P1
- **状态**: New
- **证据来源**:
  - 最近修复提交: `dfd8a01 fix: restore desktop canonical agent config migration`
  - 最近修复提交: `e802582 fix: migrate desktop legacy runtime user settings`
  - 关联计划: `docs/current-plans/canonical-config-runtime-apply.md`

## 端到端链路

1. 老用户在 desktop 环境里已经通过 legacy `data/runtime/config_runtime.yaml` 持有 agent、channel、search、FMP 等实际可用配置。
2. Desktop 启动后切到 canonical `config.yaml` 作为真相源，并生成新的 effective config 给 sidecar / channels。
3. 旧实现只迁移了部分 agent 字段，遗漏了 channel 开关、bot token、search/FMP key、chat scope 等用户设置。
4. 结果是 desktop 看起来“成功启动”，但运行时实际拿到的是不完整的 canonical 配置。

## 期望效果

- 升级后的 desktop 首次启动应自动把 legacy runtime 中仍有效的用户配置单向提升到 canonical `config.yaml`。
- 迁移后生成的 effective config 应继续保持原先可工作的 agent、channel 与 search 行为，不应让用户重新配置。

## 当前实现效果（问题发现时）

- 旧实现会让 canonical config 回退到种子态或半空状态，遗漏一部分真实用户设置。
- 用户可能看到 runner、multi-agent、Feishu / Telegram / Discord、Tavily、FMP 等链路“看似已配置，实际不可用”。
- 这不是单个字段显示错误，而是 desktop 启动后整条用户工作流被静默破坏。

## 用户影响

- 升级或切换 desktop runtime 后，用户可能突然失去既有 agent / 渠道 / 搜索能力。
- 问题表现为“设置页面有值但运行不生效”或“以前能跑的链路现在不工作”，排障成本很高。
- 对依赖 desktop 持续运行的用户来说，这是明显的端到端回归。

## 根因判断

- canonical config 切换过程中，legacy runtime 到 canonical 的迁移范围不完整。
- desktop 启动链路优先相信 canonical config，但 canonical 当时并未完整承接历史用户配置。

## 修复线索

- `dfd8a01` 先恢复 agent 配置迁移；`e802582` 再补齐 channel、search、FMP 与 chat scope 等缺口。
- 当前 bug 台账先以 `New` 登记，等待人工确认是否按现网状态转 `Fixed` / `Closed`。
