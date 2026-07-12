# Bug: Web direct 连续用户消息会跳过上一轮未答请求

- 发现时间：2026-07-12 03:02 CST
- Bug Type：Business Error
- 严重等级：P2
- 状态：Fixed
- GitHub Issue：无，非 P1

## 证据来源

- `data/sessions.sqlite3` -> `session_messages`
  - 巡检窗口：2026-07-11 23:02-2026-07-12 03:02 CST。
  - `session_id=Actor_web__direct__web-user-e05f5e5f74a3`。
  - `ordinal=577` / `2026-07-11T23:34:19.372717+08:00`，user 输入 `我的持仓有哪些`。
  - `ordinal=578` / `2026-07-11T23:42:48.215991+08:00`，同一 session 又收到 user 输入 `KLAC是干嘛的`。
  - 两条 user turn 中间没有任何 assistant turn、失败提示或产品化收口。
  - `ordinal=579` / `2026-07-11T23:43:26.081776+08:00`，assistant 只回答 `KLAC` 业务问题，没有补答上一轮持仓查询。
  - `ordinal=580-581` / `2026-07-12T00:45:18-00:45:46+08:00`，同一 Web direct 会话继续能正常回答 `KLAC值不值得买`，说明不是 Web direct 全局停摆。
- 聚合扫描：
  - 同窗 `session_messages` 新增 3 个 user turn / 2 条 assistant final；按“每个 user turn 到下一条 user turn 前必须有 assistant”规则统计，只有 1 个 user turn 未被任何 assistant 收口，即上述持仓查询。
  - assistant final 污染扫描未命中空回复、`<think>`、`reasoning_content`、本机路径、provider 原始错误、panic、quota、`data_fetch`、`quote_short`、`company_profiles/` 或原始工具 JSON。
- `data/runtime/logs/web.log.2026-07-11`
  - 本窗日志继续推进到 2026-07-12 03:01 CST，Web runtime 未全局停止。
  - 同窗 heartbeat 仍有既有结构化退化、时间口径漂移和异常行情信号；这些另归入既有 heartbeat / 行情文档，不作为本缺陷的新根因。

## 端到端链路

1. Web direct 用户发送持仓查询。
2. 该 user turn 被写入会话历史。
3. 在系统给出 assistant final、失败提示或任何可见收口前，用户又发送下一条 `KLAC` 查询。
4. 系统随后只对最新 `KLAC` 查询生成 assistant final。
5. 第一条持仓查询没有被补答，也没有留下用户可见失败提示。

## 期望效果

- 每个 Web direct user turn 都应有可审计终态：成功回答、产品化失败提示、或明确说明已被新请求取消 / 合并处理。
- 如果用户在上一轮仍执行中时发送新消息，系统应串行排队、取消并告知，或在下一轮回复中显式覆盖两个问题，不能静默丢弃上一轮请求。
- 会话历史中不应出现同一 direct session 连续 user turn 之间没有 assistant 终态的情况，除非有明确的取消 / supersede 标记。

## 当前实现效果

- SQLite 会话历史显示同一 Web direct session 出现连续两个 user turn。
- 后续 assistant 只回答第二个 `KLAC` 问题，未回答第一轮 `我的持仓有哪些`。
- 用户从产品体验上会看到持仓查询没有回复，需要自己重试或追问。

## 用户影响

- 这是功能性缺陷，不是单纯回答质量问题：用户明确提出的持仓查询没有完成。
- 影响范围目前限于一个 Web direct 会话的一轮请求；同窗后续 Web direct 仍能正常回答，未见全渠道不可用、错投、数据破坏或敏感信息外泄。
- 因此定级为 `P2`：单轮用户任务被漏答，影响 Web direct 主链路可靠性；但证据不足以证明 P1 级大面积中断。

## 根因判断

- `AgentSession::run()` 虽然已经用 per-session lock 串行化执行，但失败出口存在“只发瞬时事件、不落库 assistant 终态”的缺口。
- 当本轮 user turn 已经 Fast Persist，而运行在 early guard / prepare / runner failure 分支提前失败时，Web 前端能短暂看到 SSE 错误态，但历史恢复仍只会读到上一条 user turn。
- 用户随后继续发送下一条消息时，`session_messages` 会表现成连续两个 user turn，中间没有 assistant terminal turn，看起来像“上一轮被跳过”。
- 该问题不同于 `web_scheduler_acp_stream_disconnect_no_final.md`：本轮是 Web direct，不是 scheduler 到点任务。
- 该问题也不同于 Feishu 直聊 idle timeout 历史缺陷：本轮没有 Feishu placeholder、timeout 失败文案或 runner state DB 证据；用户可见症状是 Web direct 某个 user turn 被后续 user turn 静默跳过。

## 修复情况

- 2026-07-13 03:05 CST 代码级修复：`crates/hone-channels/src/agent_session/core.rs` 新增 `persist_failed_assistant_turn_if_needed(...)`，在 `fail_run(...)` 和 runner 失败收口分支统一补落一条用户可见 assistant 失败终态；仅当当前 session 最新一条仍是 user 时才写入，避免与已有 quota / fallback assistant 重复。
- 新增回归：
  - `run_persists_failed_assistant_turn_for_early_guard_failure`
  - `run_persists_failed_assistant_turn_for_runner_failure`
- 验证通过：
  - `cargo test -p hone-channels run_persists_failed_assistant_turn_for_early_guard_failure --lib -- --nocapture`
  - `cargo test -p hone-channels run_persists_failed_assistant_turn_for_runner_failure --lib -- --nocapture`
  - `cargo test -p hone-channels run_rejects_over_daily_limit_with_user_turn_and_friendly_error --lib -- --nocapture`
  - `cargo check -p hone-channels --tests`
- 本轮未重启 Web 服务，也没有重新制造 live Web direct 运行态样本，因此先按代码级 `Fixed` 记录，待后续巡检复核是否彻底收敛。

## 下一步建议

- 后续巡检继续关注 Web direct 是否还会出现“相邻 user turn 中间没有 assistant terminal turn”的样本；若仍有复发，再排查是否存在多标签页并发发送、手动 stop、或 SSE 断流后前端恢复策略不一致的次级根因。
- 如需进一步提升体验，可在 Web direct 明确补一层“上一轮仍执行中时禁止再次发送 / 显式 superseded 提示”的前后端一致性策略，但这不影响本轮针对“历史里静默悬空 user turn”的修复闭环。
