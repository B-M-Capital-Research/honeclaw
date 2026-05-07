# Bug: Desktop Agent 设置页缺少 `codex_acp` runner 入口，实际已切到 Codex ACP 时仍无法一致展示

- **发现时间**: 2026-04-16 15:02 CST
- **Bug Type**: System Error
- **严重等级**: P2
- **状态**: Fixed
- **证据来源**:
  - 2026-04-16 用户在当前 desktop 实例中反馈“显示还是 multiagent，并不是 codex acp”
  - 2026-04-16 代码与运行态复核：
    - `data/runtime/desktop-config/config.yaml`
    - `data/runtime/effective-config.yaml`
    - `packages/app/src/pages/settings.tsx`
    - `packages/app/src/pages/start.tsx`
    - `bins/hone-desktop/src/sidecar.rs`

## 端到端链路

1. 用户通过 desktop 设置、CLI 或运行态配置把 `agent.runner` 切到 `codex_acp`。
2. live 配置文件 `data/runtime/desktop-config/config.yaml` 与 `data/runtime/effective-config.yaml` 都已经落成 `agent.runner: codex_acp`。
3. Desktop sidecar 的 `get_agent_settings_impl(...)` 也会直接把 `config.agent.runner` 返回给前端。
4. 但设置页 `packages/app/src/pages/settings.tsx` 只有 `multi-agent`、`opencode_acp`、`gemini_cli`、`codex_cli` 四张卡片，没有 `codex_acp`。
5. 启动页 `packages/app/src/pages/start.tsx` 的 runner 卡片列表同样缺少 `codex_acp`。
6. 结果是：即使底层配置和 live listener 已经切到 `codex_acp`，desktop UI 仍缺少对应展示入口，用户只能看到不完整或误导性的 runner 选择状态。

## 期望效果

- Desktop 设置页与启动页应把 `codex_acp` 作为一级 runner 正常展示。
- 当实际配置已经是 `codex_acp` 时，UI 应明确标识当前 runner，而不是让用户继续怀疑仍在跑 `multi-agent` 或其它旧 runner。
- runner 的可见选项必须和 `AgentProvider` / sidecar 返回值保持一致，避免再次出现“底层支持，UI 缺席”的分叉。

## 当前实现效果（问题发现时）

- `data/runtime/desktop-config/config.yaml` 与 `data/runtime/effective-config.yaml` 均已显示 `agent.runner: codex_acp`。
- `bins/hone-desktop/src/sidecar.rs:get_agent_settings_impl(...)` 会直接返回 `config.agent.runner.clone()`，说明后端并未把 `codex_acp` 强制改写为 `multi-agent`。
- 但 `packages/app/src/pages/settings.tsx` 中不存在 `selectRunner("codex_acp")` 的卡片，也没有对应的检测状态与说明区。
- `packages/app/src/pages/start.tsx` 的 `CHANNELS` 数组同样缺少 `codex_acp`，导致首页 runner 提示无法覆盖当前真实配置。

## 用户影响

- 用户无法通过 desktop UI 确认当前是否真的跑在 `codex_acp`，会直接感知为“设置没生效”或“系统信息互相打架”。
- 这会放大 runner 排障成本，尤其在同时存在 canonical config、effective config、release listener 与 desktop UI 多个观察面时最明显。
- 由于功能链路本身仍可继续运行，主要受损的是 desktop 配置与观测体验，因此定级为 `P2` 而不是更高等级。

## 根因判断

- Desktop 前端的 runner 可见列表没有跟随后端已经支持的 `codex_acp` 一起扩展。
- 这不是运行时实际 runner 回退，而是 UI 契约落后于真实配置契约。
- 同一系统里“配置接口支持 `codex_acp`，前端卡片列表却不支持”造成了信息不一致。

## 修复情况

- `packages/app/src/pages/settings.tsx` 已新增 `Codex ACP` runner 卡片、当前态标识、CLI 检测入口与说明文案。
- `packages/app/src/pages/start.tsx` 已把 `codex_acp` 补进首页 runner 卡片列表。
- 修复后，desktop UI 可以直接展示当前 `codex_acp` 配置，不再依赖用户从日志或 yaml 文件反推实际 runner。

## 下一步建议

- 后续新增 runner 时，把“sidecar 返回值、设置页卡片、启动页卡片、联通检测入口”作为同一 checklist 一次性收口。
- 可以补一层前端回归测试，显式锁住 `codex_acp` 出现在 desktop runner 列表中，避免未来再发生 UI 漏挂。
