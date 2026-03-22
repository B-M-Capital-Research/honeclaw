# Handoff: Tool reasoning 事件跨渠道接入

日期：2026-03-15  
状态：已完成

## 本次目标

让 `AgentSession` 在工具调用开始时输出统一的 reasoning 事件，并让 Web console、Feishu、Discord、Telegram 在可编辑消息链路中消费该事件，用“正在...”文案实时更新占位态。

## 已完成

- `agents/gemini_cli/src/lib.rs`
  - `<tool_call>` 协议扩展为支持 `reasoning`
  - `parse_tool_call` 返回 `(name, arguments, reasoning)`
  - 补充 `reasoning` 存在/缺失的解析测试
- `agents/codex_cli/src/lib.rs`
  - `<tool_call>` 协议补齐 `reasoning`
  - `parse_tool_call` 返回 `(name, arguments, reasoning)`
  - 工具执行前后通过 observer 发出统一事件
- `agents/function_calling/src/lib.rs`
  - 原生 function calling 在工具执行前后通过 observer 发出统一事件
  - 无模型原始说明时由渠道层统一生成兜底 reasoning
- `crates/hone-channels/src/agent_session.rs`
  - `AgentSessionEvent::ToolStatus` 新增 `reasoning: Option<String>`
  - Gemini streaming 在 tool start 时透传 LLM reasoning；缺失时回退工程文案
  - tool done 事件继续保留，但不携带 reasoning
- `crates/hone-channels/src/runners/tool_reasoning.rs`
  - 新增 `codex_cli` / `function_calling` 专用 runner 包装
  - 通过 `RunnerToolObserver` 复用统一的 `ToolStatus` 发射逻辑
- `crates/hone-channels/src/runners/acp_common.rs`
  - ACP `tool_call` update 优先提取 message/detail/text 等字段作为 reasoning
  - 无可用文本时回退 `resolve_tool_reasoning`
- `crates/hone-channels/src/runtime.rs`
  - 新增 `resolve_tool_reasoning`，已知工具走“正在{展示名}...”，未知工具走“正在调用 {tool}...”
- `crates/hone-web-api/src/routes/chat.rs`
  - Web chat 在 `gemini_cli` 下切到 `run_gemini_streaming`
  - `tool_status` SSE 事件携带 `reasoning`
  - `StreamDelta` 改为直接驱动分段输出，Done 时 flush 剩余 buffer
- `packages/app`
  - `ChatStreamEvent` 新增 `reasoning`
  - session 状态新增 `thinkingText`
  - thinking 气泡改为优先展示 tool reasoning
- `bins/hone-feishu/src/main.rs`
  - tool start 时将 reasoning 推到 CardKit 占位内容
- `bins/hone-discord/src/handlers.rs`、`bins/hone-discord/src/group_reply.rs`、`bins/hone-discord/src/utils.rs`
  - 私聊和群聊聚合回复在 `gemini_cli` 下切到 `run_gemini_streaming`
  - 新增共享 `DiscordReasoningListener`，tool start 时编辑占位消息
- `bins/hone-telegram/src/main.rs`
  - 在 `gemini_cli` 下切到 `run_gemini_streaming`
  - 新增 `TelegramReasoningListener`，tool start 时编辑占位消息

## 验证

- `cargo check -p hone-core -p hone-agent-codex-cli -p hone-agent -p hone-channels` ✅
- `cargo test -p hone-agent-codex-cli parse_tool_call -- --nocapture` ✅
- `cargo test -p hone-agent run_notifies_tool_observer_on_execution -- --nocapture` ✅
- `cargo check -p hone-web-api -p hone-feishu -p hone-discord -p hone-telegram` ✅
- `cargo check -p hone-discord -p hone-telegram -p hone-feishu -p hone-web-api -p hone-channels -p hone-agent-gemini-cli` ✅
- `cargo test -p hone-agent-gemini-cli parse_tool_call -- --nocapture` ✅
- `bun run typecheck:web` ✅

## 影响范围

- `gemini_cli`、`codex_cli`、`function_calling` 在工具开始时都会发出 reasoning 事件
- `gemini_acp` / `codex_acp` / `opencode_acp` 会发出 reasoning 事件，但多数情况下来自 ACP update 文本或工程兜底，而非模型原始 tool call 字段
- Web console 的“思考中”气泡不再把 `tool_status` 作为单独 system message 插入时间线，而是显示为实时思考文案

## 剩余风险

- `function_calling` 与 ACP runners 通常拿不到模型原始“为什么要调这个工具”的原话，当前更多是工程侧展示文案
- Discord/Telegram 当前只在已有占位消息存在时编辑 reasoning；若占位消息发送失败，不会额外补发新的 reasoning 消息
- Web console 流式改为依赖 `StreamDelta` 分段；若后续更换 provider，需要再次确认 SSE 语义是否一致
