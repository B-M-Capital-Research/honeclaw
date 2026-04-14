# Bug: Desktop legacy runtime 迁移遗漏 OpenRouter key 池，升级后默认对话链路可能直接失效

- **发现时间**: 2026-04-14
- **Bug Type**: System Error
- **严重等级**: P1
- **状态**: New
- **证据来源**:
  - 最近提交: `dfd8a01 fix: restore desktop canonical agent config migration`
  - 最近提交: `e802582 fix: migrate desktop legacy runtime user settings`
  - 代码证据:
    - `bins/hone-desktop/src/sidecar.rs:1274-1313`
    - `crates/hone-core/src/config.rs:695-703`
    - `crates/hone-channels/src/core.rs:991-1019`
    - `crates/hone-llm/src/openrouter.rs:46-53`
    - `bins/hone-cli/src/main.rs:2128-2140`

## 端到端链路

1. 桌面端用户在旧版设置页或 `hone-cli configure` 中配置 OpenRouter 密钥时，当前产品写入的是 `llm.openrouter.api_keys`，并顺手把旧的单值字段 `llm.openrouter.api_key` 清空。
2. 用户升级到最近的 canonical config 迁移版本后，desktop startup 会在 `ensure_runtime_paths(...)` 中把 legacy `data/runtime/config_runtime.yaml` 的用户设置补迁到新的 canonical `config.yaml`。
3. 当前补迁逻辑只会在 `llm.openrouter.api_key` 为空时复制 legacy 单值字段 `llm.openrouter.api_key`，但不会处理 legacy 已经在用的 `llm.openrouter.api_keys` 数组。
4. 随后 runtime 重新生成 `effective-config.yaml`，并只从 canonical config 读取 OpenRouter key 池。
5. 对于依赖 OpenRouter key 池而不是 `agent.opencode.api_key` 直填的桌面会话，runner 最终拿到的是空 key 池，默认对话链路会在启动或首轮请求时退化为“未配置 OpenRouter API Key”。

## 期望效果

- 升级后，桌面端应完整保留用户已经配置的 OpenRouter key 池，不论它存的是单值 `api_key` 还是数组 `api_keys`。
- canonical config 迁移应与当前设置写入语义一致，至少保证 desktop settings、`hone-cli configure`、runtime apply 这三条链路不会因为字段形态不同而丢失密钥。
- 升级后的默认聊天、辅助模型、以及需要注入 `OPENROUTER_API_KEY` 的 opencode / multi-agent answer 路径应继续可用，而不是要求用户重新录入密钥。

## 当前实现效果（问题发现时）

- Desktop / CLI 当前保存 OpenRouter 密钥时，都会把值写到 `llm.openrouter.api_keys`，并把 `llm.openrouter.api_key` 清空。
- 最近新增的 legacy 补迁逻辑只处理 `llm.openrouter.api_key`，没有迁移 `llm.openrouter.api_keys`。
- `opencode_acp` 和 `multi-agent` answer 路径在 Hone 接管 OpenRouter 路由、且 `agent.opencode.api_key` 为空时，会尝试从 `llm.openrouter.effective_key_pool()` 注入首个 OpenRouter key；该 key 池在这条迁移缺口下会变空。
- function-calling / OpenRouter provider 直接依赖同一个 key 池，空池会返回“LLM API key 未配置（环境变量或 config.yaml）”。

## 用户影响

- 已在旧版桌面端配置过 OpenRouter 多 Key 的升级用户，升级后可能看到配置页仍能正常打开，但聊天、辅助任务或 answer agent 首轮就失败。
- 默认 `opencode_acp` 桌面链路在未显式填写 `agent.opencode.api_key`、而是依赖 OpenRouter key 池兜底时，最容易直接受影响。
- 这是典型的“升级后配置静默丢失”问题，用户端表现为产品突然不可用，且根因不容易自查。

## 根因判断

- 最近的配置迁移修复把 FMP、Tavily、多数 agent/channel 设置都补迁到了 canonical config，但 OpenRouter 仍按旧的单值字段思路处理，只迁 `llm.openrouter.api_key`。
- 迁移逻辑没有和当前 desktop / CLI 的实际写入契约对齐，而这两个入口早已把 OpenRouter 密钥池收敛到 `llm.openrouter.api_keys`。
- 相关测试只覆盖了 `llm.openrouter.api_key` 的迁移断言，没有覆盖 legacy `api_keys` 数组场景，因此这个回归缺口没有被自动化捕获。

## 修复线索

- 在 legacy 补迁逻辑中补齐 `llm.openrouter.api_keys` 的迁移，并明确与单值 `api_key` 的优先级和去重语义。
- 新增最少一条自动化回归：legacy `config_runtime.yaml` 仅含 `llm.openrouter.api_keys` 时，desktop 启动后 canonical / effective config 仍应保留完整 key 池。
- 当前 bug 台账先以 `New` 登记，等待人工确认并转入 `Apprived` / `Fixing` / `Fixed` / `Closed`。
