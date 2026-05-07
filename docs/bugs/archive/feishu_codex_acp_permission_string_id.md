# Bug: Feishu 直聊遇到 Codex ACP 字符串权限请求 id 后整轮失败

- **发现时间**: 2026-04-26 12:25 CST
- **Bug Type**: Compatibility / System Error
- **严重等级**: P1
- **状态**: Fixed
- **证据来源**:
  - `data/runtime/logs/web.log.2026-04-26`:
    - `session=Actor_feishu__direct__ou_5f39103ac18cf70a98afc6cfc7529120e5`
    - `error="codex acp permission request missing id"`
  - `data/runtime/logs/acp-events.log`:
    - 同一 session 收到 `method="session/request_permission"`
    - payload 顶层存在 `id="0e68dcc2-6a6b-4d1e-a49d-cb6dd179beb7"`
    - `params.toolCall.title="Approve MCP tool call"`
  - 运行环境:
    - `codex-cli 0.125.0`
    - `@zed-industries/codex-acp@0.12.0`

## 端到端链路

1. Feishu 用户请求查询定时任务。
2. Codex ACP 调用 `hone/skill_tool` 前发出 `session/request_permission`。
3. 新版 ACP 使用字符串 UUID 作为 JSON-RPC request id。
4. Hone ACP 权限处理代码只用 `as_u64()` 读取 id，导致把字符串 id 误判为缺失。
5. runner 返回 `codex acp permission request missing id`，Feishu 消息流失败并发送通用失败提示。

## 根因判断

JSON-RPC request id 可以是字符串或数字。当前实现只兼容数字 id；Codex ACP 0.12.0 发出的权限请求 id 已变为字符串 UUID，因此触发兼容性故障。

## 修复

- `crates/hone-channels/src/runners/acp_common/protocol.rs`：权限请求响应不再强转数字 id，而是原样 echo 回 ACP 进程。
- `crates/hone-channels/src/runners/opencode_acp.rs`：同步修复旧 opencode 专用权限请求路径，避免同类兼容问题。
- `crates/hone-channels/src/runners/acp_common/tests.rs`：新增字符串 JSON-RPC id 回归测试。

## 验证

- `cargo test -p hone-channels acp_permission_request --all-targets`
- `cargo fmt --check --all`

