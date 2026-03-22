# 2026-03-18 ACP Runner Hone MCP Bridge

## 结果

- `gemini_acp`、`codex_acp`、`opencode_acp` 现在都会在 `session/new` / `session/load` 中附带 Hone 本地 MCP server，而不是继续传空 `mcpServers`。
- 新增 `bins/hone-mcp` 作为 stdio MCP server；启动后会基于 `HONE_CONFIG_PATH`、actor、`channel_target`、`allow_cron` 重建 `ToolRegistry`，把 Hone 内置工具暴露成 MCP tools。
- `AgentRunnerRequest` 已补齐 `actor`、`channel_target`、`allow_cron`、`config_path`，用于 runner 到 MCP 子进程的上下文透传。
- `skill_tool` 的数组参数 schema 已修复为合法 JSON Schema，避免 ACP runner 将 Hone tools 暴露给 provider 后触发 `invalid_function_parameters`。

## 关键实现

- `crates/hone-channels/src/mcp_bridge.rs`
  - 负责生成 ACP `mcpServers` 配置
  - 负责 `hone-mcp` 的 JSON-RPC stdio loop
  - 将 `ToolRegistry` schema 转成 MCP `tools/list`
  - 将 `ToolRegistry::execute_tool()` 包成 MCP `tools/call`
- `crates/hone-channels/src/runners/acp_common.rs`
  - `create_acp_session()` 现在接收 `mcp_servers`
- `crates/hone-channels/src/runners/gemini_acp.rs`
- `crates/hone-channels/src/runners/codex_acp.rs`
- `crates/hone-channels/src/runners/opencode_acp.rs`
  - 都会在新建/恢复 ACP session 时注入 Hone MCP server
- `crates/hone-tools/src/skill_tool.rs`
  - `aliases` / `tools` 的 `items` 从非法 `"string"` 修正为 `{ "type": "string" }`

## 验证

- `cargo check -p hone-channels -p hone-tools -p hone-mcp -p hone-cli`
- `cargo build -p hone-mcp -p hone-cli`
- 直连 `target/debug/hone-mcp`
  - `initialize` 成功
  - `tools/list` 返回 8 个 Hone tools
  - `tools/call name=web_search` 成功返回 Tavily 搜索结果
- 手工驱动 `opencode acp --print-logs`
  - `session/new` 接受 Hone MCP server，日志显示 `mcp key=hone toolCount=8 create() successfully created client`
  - `session/prompt` 时 provider 真实拿到了 `hone_kb_search` / `hone_web_search` 等工具 schema
  - 实测返回内容包含 `I searched the knowledge base for Rocket Lab with hone_kb_search` 和 `HONE_ACP_MCP_OK`
- `bash tests/regression/manual/test_opencode_acp_hone_mcp.sh`
  - 直接驱动 `opencode acp` 的 stdio JSON-RPC
  - 断言 Hone MCP 注册成功、`hone_kb_search` 真实被调用、最终流式文本中包含 `HONE_OPENCODE_ACP_MCP_OK`
- `bash tests/regression/manual/test_gemini_acp_initialize.sh`
  - 直接驱动 `gemini --experimental-acp`
  - 断言在不强制 `--sandbox` 时，`initialize` 与带 Hone MCP 的 `session/new` 都能成功返回

## 风险与后续

- `gemini_acp` 的 `initialize timeout` 已定位到 Gemini CLI 0.33.1 与 `--experimental-acp --sandbox` 的组合：本机直跑时会在返回任何 JSON-RPC 之前直接退出。Hone 已改为仅强制 `approval-mode=plan`，并新增 `tests/regression/manual/test_gemini_acp_initialize.sh` 覆盖 `initialize/session/new`。
- `hone-cli` 仍硬编码读取项目根 `config.yaml`，无法仅通过 `HONE_CONFIG_PATH` 临时切 runner；后续若要做 runner 级回归，优先直接驱动 runner/ACP 或先统一 CLI 配置入口。
- `hone-mcp` 当前只暴露 `tools/list` / `tools/call`，`resources/list` / `prompts/list` 仍为空实现；如果后续要给 ACP agent 提供更多上下文材料，再扩展这两类能力。
