# Bug: Web direct 旧会话上下文耗尽后简单请求也无法回复

- 发现时间：2026-06-23 15:02 CST
- Bug Type：System Error
- 严重等级：P2
- 状态：New

## 证据来源

- `data/runtime/logs/web.log.2026-06-23`
  - 巡检窗口：2026-06-23 11:02-15:02 CST。
  - `2026-06-23 11:14:39 CST`，Web direct 会话 `Actor_web__direct__web-user-f40ae1caa720` 收到用户短请求“取消所有定时任务”，随后 `11:14:57 CST` 记录 `runner.error kind=AgentFailed` 与 `处理失败`，底层错误为 `context_window_exceeded`。
  - `2026-06-23 15:01:25 CST`，同一 Web direct 会话收到用户短请求“你能为我干什么”，`15:02:13 CST` 再次记录 `runner.stage=acp.usage ... used=258400`，随后 `runner.error kind=AgentFailed` 与 `处理失败`，底层仍为 `context_window_exceeded`。
  - 同窗 `web.log.2026-06-23` 仍有 12 条 heartbeat `context_window` 相关失败，但这些归入既有 heartbeat context/结构化缺陷；本单只记录 Web direct 直聊链路。
- `data/runtime/logs/acp-events.log`
  - 2026-06-23 11:02-15:02 CST 可重构 9 次 `session/prompt`、7 个 session、2 个 ACP response error。
  - 两个 ACP response error 均来自 `Actor_web__direct__web-user-f40ae1caa720`，payload 为 `codex_error_info=context_window_exceeded`。

## 端到端链路

1. Web 用户在已有 direct 会话中发送一个短请求。
2. Web runtime 持久化 user turn 并为同一会话创建新的 Codex ACP session。
3. runner 恢复旧会话上下文并进入 ACP 请求。
4. ACP 返回 `context_window_exceeded`，Web runtime 将本轮记为 `AgentFailed` / `处理失败`。
5. 用户请求没有得到可用答复；同一会话后续简单问题继续失败。

## 期望效果

- 旧会话上下文过长时，Web direct 应自动 compact、截断可恢复历史、提示用户开启新会话，或至少返回产品化的用户态失败说明。
- 对“取消所有定时任务”这类有明确工具副作用的短请求，系统不应仅因历史上下文过长而直接丢失本轮处理机会。
- 日志和台账应能区分“模型无法回答”和“会话上下文预算耗尽”，方便后续自动恢复或用户引导。

## 当前实现效果

- runner 在恢复上下文后直接把 prompt 推到上下文上限，`usage_update` 显示 `used=258400`。
- ACP 返回 `context_window_exceeded` 后，本轮 Web direct 只落为 `AgentFailed` / `处理失败`，未见自动 compact/retry 成功，也未见同会话内的 assistant final 收口。
- 同一会话在 11:14 CST 和 15:02 CST 两次简单短请求均复现，说明这不是单次网络或模型波动。

## 用户影响

- 这是功能性 bug。用户在 Web direct 中的正常短请求无法完成，尤其是“取消所有定时任务”这类操作型请求会被会话历史体积阻断。
- 影响目前集中在单个过长 Web direct 会话，未观察到全渠道不可用、跨用户错投、数据安全问题或大面积 direct 会话失败，因此定级为 `P2`，不是 `P1`。

## 根因判断

- 高概率是 Web direct 的上下文恢复 / prompt 组装缺少预算上限，或未在 Codex ACP `context_window_exceeded` 后触发与普通会话一致的 compact / retry / 用户态降级。
- 既有 `scheduler_heartbeat_context_window_limit_no_recovery.md` 覆盖 heartbeat 监控任务超窗，本单覆盖 Web direct 直聊会话；两者影响链路不同，因此单独建档。
- 既有 `context_overflow_recovery_gap.md` 为历史归档修复项，但本轮真实运行态显示当前 Web direct 旧会话仍会在短请求上复现，因此按新活跃缺陷记录。

## 下一步建议

- 检查 Web direct `restore_context + build_prompt + create_runner` 路径是否在进入 ACP 前估算上下文预算。
- 对 `context_window_exceeded` 增加 Web direct 专用恢复：自动压缩历史、丢弃低价值旧工具 transcript，或明确返回“当前会话过长，请开启新会话 / 已尝试压缩失败”的用户态提示。
- 为同一会话连续超窗增加健康标记，避免后续短请求继续无效重试。
- 增加回归覆盖：旧会话含超长历史时，一个短 direct 请求仍能通过 compact/retry 收口，或至少返回产品化失败说明。
