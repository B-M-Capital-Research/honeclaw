# 2026-03-18 Runner 切换到 Gemini 3.1 Pro

## 结果

- 最终将项目 runner 切换为 `gemini_acp`，并使用 `gemini-3.1-pro-preview`。
- 已同时更新 `data/runtime/config_runtime.yaml` 与项目根 `config.yaml`，避免 Desktop 运行时与种子配置分叉。
- 本机 `gemini` CLI 版本为 `0.33.1`，满足 `gemini_acp` 的最低版本要求。
- 先前同日做过的 `opencode_acp + Gemini via OpenRouter` 配置已被这次切换覆盖，不再是当前默认 runner。

## 为什么这次要改 runtime + 项目配置

- `data/runtime/config_runtime.yaml` 现已存在，Desktop 实际生效配置应优先以它为准。
- `config.yaml` 仍是种子配置；如果后续重建 runtime 文件，会从这里恢复默认值。
- 因此这次切换不能只改一处，否则 Desktop 和 CLI / 首次启动默认值会重新分叉。

## 验证

- `gemini --version` → `0.33.1`
- `bash tests/regression/manual/test_gemini_streaming.sh`
  - 返回 `gemini stream-json output detected`
- `printf 'Reply with exactly: HONE_HONECLI_GEMINI_ACP_OK\nquit\n' | cargo run -q -p hone-cli`
  - 启动日志显示 `dialog.engine=gemini_acp`
  - 日志显示 `model=gemini-3.1-pro-preview`
  - 返回 `HONE_HONECLI_GEMINI_ACP_OK`

## 风险与后续

- 当前 `hone-cli` 仍固定读取项目根 `config.yaml`；Desktop 运行时读取 `data/runtime/config_runtime.yaml`。后续若继续调整 runner，两处仍需保持同步，除非统一入口的读取策略被重构。
- `gemini_acp` 当前依赖本机 `gemini-cli` 登录态；如果后续登录态失效，需要重新登录或显式提供 `GEMINI_API_KEY`。
