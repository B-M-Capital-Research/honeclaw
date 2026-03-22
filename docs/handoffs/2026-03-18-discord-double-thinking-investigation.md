# 2026-03-18 Discord 重复“正在思考中”排查

## 结论

- 这次 Discord `正在思考中...` 重复，不是单次 `AgentSession` / 单次 `opencode_acp` run 内部把同一条 thinking 文案发了两次。
- 证据是 direct session 文件里，同一条用户消息在 705ms 内被持久化了两次：
  - `data/sessions/<discord-dm-session>.json`
  - 两条 `user` timestamp 分别是 `2026-03-18T15:11:54.363688+08:00` 与 `2026-03-18T15:11:55.068810+08:00`
- 当前 Discord 即时私聊链路在单进程内有 `SessionLockRegistry`；若只有一个 `hone-discord` 进程，同一 session 的第二次 run 不应在第一轮 assistant 完成前就把第二条 user message 落盘。
- 因此更符合“同一条 Discord 输入被两个独立 consumer / 进程各自处理了一次”的形态。两个进程各自拥有独立的 dedup 和 session lock，会各发一条 placeholder，于是用户侧看到两个 `正在思考中...`。

## 证据链

- Discord DM 当前链路：
  - `bins/hone-discord/src/handlers.rs`
  - `handle_immediate_message()` -> `run_session_with_outbound()` -> 先发一次 placeholder
- 通用出站链路：
  - `crates/hone-channels/src/outbound.rs`
  - `run_session_with_outbound()` 只会先调用一次 `adapter.send_placeholder()`
- Discord placeholder / reasoning 更新：
  - `bins/hone-discord/src/utils.rs`
  - reasoning 事件只会编辑已有 placeholder；不会在 placeholder 正常存在时再平白补发第二条相同消息
- `opencode_acp` 单次工具调用验证：
  - 手工直连 `opencode acp` + Hone MCP 后统计到 `tool_call_count=1`
  - `tool_call_update_completed_count=1`
  - 没有发现单次 prompt 自行产出两条 `tool_call/start`
- 历史 session 扫描：
  - 仅发现这一个 Discord session 存在“2 秒内连续两条完全相同 user message”的异常
  - 没发现更广泛的批量性重复写入

## 复现情况

- 未能复现“单个 `opencode_acp` run 自己双发 thinking”。
- 已复现并验证单次 ACP prompt 只有一条 `tool_call/start`，因此 runner 侧不是这次双 thinking 的主因。
- 若要复现当前现象，需要制造“同一条 Discord 输入被两个独立 consumer 同时消费”的条件，例如：
  - 同时跑两个 `hone-discord` 进程
  - 桌面 sidecar 与手工 `cargo run -p hone-discord` 并存
  - 某个旧进程未被 runtime pid 管理正确回收

## 现状与风险

- 当前机器上排查时只看到一个活着的 `hone-discord` 进程：`target/debug/hone-discord`
- 但 `data/runtime/discord.pid` 已不存在，说明 Discord 进程管理状态和运行现状并不完全一致；这会放大“旧进程残留、重复消费”的嫌疑。
- 本次没有改代码；如果后续还出现同类现象，优先检查：
  - `pgrep -lf hone-discord`
  - 是否同时存在桌面 sidecar 与手工启动实例
  - 启动/停止 Discord sidecar 时 pid 文件是否同步创建和清理

## 本次验证

- `sed -n '1,220p' data/sessions/<discord-dm-session>.json`
- `pgrep -lf hone-discord`
- `bash tests/regression/manual/test_opencode_acp_hone_mcp.sh`
- 直接驱动 `opencode acp`，统计：
  - `tool_call_count=1`
  - `tool_call_update_completed_count=1`
