# Actor Sandbox Hardening

最后更新：2026-03-18
状态：已完成

## 目标

- 收紧渠道 runner 的本地文件可见范围，避免底层 CLI 因 repo `cwd` 或向上规则发现而读到仓库其他文件。
- 让附件落盘、CLI cwd、ACP cwd 与系统 prompt 注入方式都对齐到 actor 级隔离模型。

## 结果

- 新增 `crates/hone-channels/src/sandbox.rs`，渠道 actor 默认使用 repo 外 sandbox 根目录：
  - 默认位置：系统临时目录下的 `hone-agent-sandboxes/`
  - 可选覆盖：环境变量 `HONE_AGENT_SANDBOX_DIR`
- `AgentSession` 不再把 runner `working_directory` 指向仓库根目录，而是为每个 actor 选择独立 sandbox。
- 附件 ingest 改为落盘到 actor sandbox 下的 `uploads/<session_id>/`。
- `codex_acp` 默认强制 `workspace-write + never`，并忽略危险 bypass。
- `gemini_cli` 默认启用 `--sandbox --approval-mode plan`；`gemini_acp` 仅固定 `approval-mode=plan`，不再强制 `--sandbox`。
- `opencode_acp` 会在 actor sandbox 下生成最小权限 `opencode.jsonc`，拒绝 `external_directory` 与高风险内建工具。
- ACP `session/prompt` 现在显式拼接 Hone 生成的 system prompt，不再依赖 repo `cwd` 下被底层 CLI 自动发现的规则文件。
- `codex_cli` 也改为通过 `--cd` 和 sandbox flags 对齐 actor sandbox。
- 追加 strict guard：`codex_cli` / `codex_acp` 在 actor sandbox 路径上会直接报错拒绝，因为实测 `codex workspace-write` 仍允许读取 sandbox 外 repo 文件。

## 验证

- `cargo check -p hone-core -p hone-channels -p hone-discord -p hone-feishu -p hone-imessage -p hone-agent-codex-cli`
- `cargo test -p hone-core actor::tests -- --nocapture`
- `bash tests/regression/manual/test_actor_sandbox_isolation.sh`

## 未完成 / 风险

- 仓库根 `Cargo.toml` 目前引用了缺失的 `bins/hone-mcp/Cargo.toml`，导致根目录级别的 `cargo fmt --all` / `cargo test` 失败；本次只能使用针对性 `cargo check` 验证。
- `opencode_acp` 当前的最小权限配置基于已确认的权限键；若 opencode 升级后新增默认工具，需要补一轮权限矩阵回归。
- `gemini_acp` 的 ACP 启动依赖本机 Gemini CLI 行为；当前已确认 `--experimental-acp --sandbox` 会导致 `initialize timeout`，后续升级 Gemini CLI 时需要复跑手工回归。
- `tests/regression/manual/test_actor_sandbox_isolation.sh` 目前是手工回归，不进 CI；它依赖本机已安装并登录 `gemini` / `opencode` / `codex`。
