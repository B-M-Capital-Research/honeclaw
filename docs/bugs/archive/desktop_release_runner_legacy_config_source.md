# Bug: Release app / 渠道进程仍可被 legacy `config_runtime.yaml` 驱动，导致 runner 改完后 live 服务不立即生效

- **发现时间**: 2026-04-16 14:35 CST
- **Bug Type**: System Error
- **严重等级**: P1
- **状态**: Fixed
- **证据来源**:
  - 2026-04-16 14:08 CST 当前 release Feishu 启动日志：`data/runtime/logs/hone-feishu.release-restart.log`
  - 2026-04-16 14:43 CST 进程环境复核：`ps eww -p 20089`
  - 2026-04-16 当前源码与运行约束：
    - `bins/hone-desktop/src/sidecar/runtime_env.rs`
    - `launch.sh`
    - `docs/runbooks/desktop-release-app-runtime.md`

## 端到端链路

1. 用户在 Desktop 设置页或其它 canonical config 入口修改 `agent.runner`，期望当前 release app / live listener 立即切到新 runner。
2. Desktop sidecar 的正式实现会把 `HONE_USER_CONFIG_PATH` 指向 canonical `config.yaml`，并把 `HONE_CONFIG_PATH` 指向生成后的 `effective-config.yaml`。
3. 但本轮真实运行态里，release `hone-feishu` 进程环境仍是：
   - `HONE_CONFIG_PATH=/Users/ecohnoch/Desktop/honeclaw/data/runtime/config_runtime.yaml`
   - `HONE_USER_CONFIG_PATH=/Users/ecohnoch/Desktop/honeclaw/data/runtime/config_runtime.yaml`
4. 同一时间窗的 `hone-feishu.release-restart.log` 启动 banner 继续打印 `dialog.engine=multi-agent`，与桌面当前期望切到 `codex_acp` 的操作不一致。
5. 由于 live 服务实际上继续读取 legacy runtime config，用户会感知为“runner 改了，但当前正在跑的服务还是旧 runner”，从而把配置切换误判成未生效或随机失效。

## 期望效果

- release desktop host 与其拉起的 bundled backend/channel sidecar 应只把 legacy `data/runtime/config_runtime.yaml` 当成一次性迁移来源，而不是 steady-state 运行真相源。
- 即使外部启动命令或旧 runbook 仍把 `HONE_CONFIG_PATH` / `HONE_USER_CONFIG_PATH` 指到 legacy 文件，desktop 也应优先回到 canonical `config.yaml`，并重新生成 `effective-config.yaml`。
- 用户修改 runner 后，当前 live 进程应在一次明确的重启后切到同一 runner，不再出现“UI/配置已变，live listener 仍跑旧值”。

## 当前实现效果（问题发现时）

- `ps eww -p 20089` 显示当前 `hone-feishu` 进程直接继承了 `HONE_CONFIG_PATH` 与 `HONE_USER_CONFIG_PATH` 都指向 `data/runtime/config_runtime.yaml` 的旧环境。
- `data/runtime/logs/hone-feishu.release-restart.log` 在 `2026-04-16T06:08:54Z` 的最新启动序列中，明确打印：
  - `config.path=/Users/ecohnoch/Desktop/honeclaw/data/runtime/config_runtime.yaml`
  - `dialog.engine=multi-agent`
- `docs/runbooks/desktop-release-app-runtime.md` 仍把 `config_runtime.yaml` 作为 release app、backend lane 和 detached restart 的推荐环境变量写法，容易让人工运维持续把 live 服务绑回 legacy 文件。

## 用户影响

- runner 切换会表现成伪成功：配置面显示已改，但当前 live 服务并未同步切换。
- 这会直接影响用户对 runner 特性的判断，例如明明要验证 `codex_acp`，实际运行的却仍是 `multi-agent`。
- 对消息渠道而言，这属于主链路问题：用户会在同一台服务上持续命中错误 runner，无法稳定验证或使用目标链路。

## 根因判断

- steady-state config 真相源已经迁到 canonical `config.yaml` + generated `effective-config.yaml`，但 release 运维路径仍保留了把 `config_runtime.yaml` 当作主配置文件的旧习惯。
- desktop 侧 `desktop_canonical_config_path(...)` 之前会无条件接受 `HONE_USER_CONFIG_PATH` / `HONE_CONFIG_PATH` override，因此只要外部启动环境把它们指向 legacy 文件，desktop 就会继续围绕 legacy 文件运行。
- 这导致 code path 虽然已经支持 canonical/effective config，但 live release 运行态仍可能被旧环境变量拖回 legacy 配置源。

## 修复情况

- `bins/hone-desktop/src/sidecar/runtime_env.rs` 已补自保护：
  - 当 `HONE_USER_CONFIG_PATH` 或 `HONE_CONFIG_PATH` 指向 legacy `config_runtime.yaml` 时，desktop 不再把它当成 canonical steady-state 配置源
  - release app 会回退到 canonical `config.yaml`，并继续把 legacy 文件只作为单向补迁来源
- 新增桌面侧回归测试：
  - `desktop_canonical_config_path_ignores_legacy_runtime_override`
  - `desktop_canonical_config_path_prefers_non_legacy_user_override`
- `docs/runbooks/desktop-release-app-runtime.md` 已同步改成 canonical/effective config 的推荐启动方式，避免人工重启再次把 live 服务绑回 legacy 文件。

## 下一步建议

- 所有 release app / backend / channel 的手工启动命令统一使用：
  - `HONE_USER_CONFIG_PATH=/Users/ecohnoch/Desktop/honeclaw/config.yaml`
  - `HONE_CONFIG_PATH=/Users/ecohnoch/Desktop/honeclaw/data/runtime/effective-config.yaml`
- 若日志再次出现 `config.path=.../config_runtime.yaml`，应直接判定为旧 supervisor 或旧 runbook 仍在生效，而不是继续从业务代码层排查 runner 切换。
- 后续可考虑在启动日志中额外显式打印 canonical 与 effective config 的双路径，降低排障时的判断成本。
