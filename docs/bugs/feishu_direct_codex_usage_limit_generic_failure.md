# Bug: Feishu 直聊命中 Codex usage limit 后只返回通用失败，用户请求连续无法完成

- 发现时间：2026-05-12 07:03 CST
- Bug Type：System Error
- 严重等级：P1
- 状态：New
- GitHub Issue：[#40](https://github.com/B-M-Capital-Research/honeclaw/issues/40)

## 证据来源

- 最近四小时运行日志：
  - `data/runtime/logs/sidecar.log`
  - `2026-05-12 06:39:47 CST`：Feishu 直聊 session `Actor_feishu__direct__ou_5ff0946a82698f7d16d9a5684696c84185` 接收一条 ASTS 财报分析与建仓判断请求，随后进入 `agent.prepare` / `agent.run`。
  - `2026-05-12 06:39:59 CST`：同一 message_id 记录 `runner.error kind=AgentFailed`，底层 Codex ACP 返回 `usage_limit_exceeded`，并提示可稍后恢复；handler 记录 `completed success=false reply_chars=0`。
  - `2026-05-12 06:40:00 CST`：Feishu 只发送 `reply.send detail=failure_fallback segments.sent=1`。
  - `2026-05-12 06:41:27 CST`：同一 session 再次收到相同主题请求。
  - `2026-05-12 06:41:33 CST`：再次命中 `usage_limit_exceeded`，同样以 `failure_fallback segments.sent=1` 收口。
- ACP 事件日志：
  - `data/runtime/logs/acp-events.log`
  - `2026-05-11T22:41:33Z` 对应 Codex ACP JSON-RPC response 包含 `codex_error_info="usage_limit_exceeded"` 和恢复时间提示；该信息没有被映射成用户可理解的额度/运行能力提示。
- 会话镜像限制：
  - `data/sessions.sqlite3` 的 `sessions` / `session_messages` 仍停在 `2026-04-27T16:54:20+08:00`，这是既有 fixed-but-live-old 证据；本轮缺陷证据以 runtime 日志为准。

## 端到端链路

1. 用户通过 Feishu 直聊发起 ASTS 财报分析和是否建仓判断请求。
2. Feishu handler 发送处理中 placeholder，并把 user turn 写入当前运行态 session。
3. AgentSession 创建 Codex ACP runner 并发起 `session/prompt`。
4. Codex ACP 返回 `usage_limit_exceeded`，包含“额度已达上限 / 稍后恢复”的可解释失败原因。
5. `user_visible_error_message()` 将包含 `codex acp` 的内部错误统一压成通用失败文案。
6. Feishu handler 发送通用 `failure_fallback`，用户没有得到“服务端 Codex 额度暂时耗尽、预计恢复时间、可稍后重试或切换运行能力”的明确解释。
7. 用户随后重复同一请求，第二轮仍以相同方式失败，实际任务没有完成。

## 期望效果

- 当 runner 返回明确的 usage / quota / entitlement 类错误时，系统应把它映射成脱敏但具体的用户态提示，例如“当前运行额度暂时耗尽，请稍后再试”。
- 若上游提供可恢复时间，用户态文案应保留概略恢复时间或建议稍后重试，避免误导为普通临时故障。
- 运行日志和会话记录应继续保留结构化失败原因，便于运营判断是 runner 额度、用户业务额度、provider quota，还是普通系统错误。
- 若产品要求 Feishu 直聊在 Codex 额度耗尽时仍可用，应切换到可用 runner 或明确降级。

## 当前实现效果

- 最近四小时内，同一用户同一问题连续两次没有得到任务答案。
- 系统内部已经拿到 `usage_limit_exceeded` 和恢复时间提示，但用户侧只收到通用失败 fallback。
- 这不是 Markdown、格式或表达偏好问题，而是 Feishu 直聊主链路在 runner 额度耗尽时无法完成用户任务；同时失败原因被过度泛化，用户不知道应等待额度恢复还是需要换入口。

## 用户影响

- 用户明确请求财报分析与建仓判断，但连续两次被阻断，只能看到失败提示，无法完成核心投研任务。
- 同类错误在服务端 Codex 账号额度耗尽期间可能影响所有依赖 Codex ACP 的 Feishu 直聊请求。
- 因为失败提示没有区分 runner usage limit 与普通系统失败，用户可能立即重复发送，继续消耗处理链路并制造重复失败。
- 本缺陷影响主功能链路的可用性和错误可解释性，因此定级为 P1。

## 根因判断

- 直接触发原因是 Codex ACP 返回 `usage_limit_exceeded`。
- 代码侧 `crates/hone-channels/src/runtime.rs` 的 `looks_internal_error_detail()` 会把包含 `codex acp` 的错误统一视作内部错误；`user_visible_error_message()` 因此返回通用失败文案。
- 现有额度友好文案只覆盖 Hone 自身的“已达到今日对话上限”，没有覆盖 runner / upstream usage limit 这类可解释的资源耗尽错误。
- Feishu handler 的失败收口能发送 fallback，说明当前不是出站投递失败；缺口在错误分类和用户态文案映射。

## 下一步建议

- 在共享错误映射层新增 runner usage limit 分类，识别 `usage_limit_exceeded`、`You've hit your usage limit`、`try again at ...` 等信号，输出脱敏用户态额度提示。
- 在 Feishu handler 或 AgentSessionResult metadata 中保留结构化 `failure_kind=runner_usage_limit`，便于后续统计和告警。
- 增加回归测试覆盖：Codex ACP usage limit 错误不暴露原始 stderr / JSON-RPC，但会返回明确的“运行额度暂时耗尽”类文案。
- 若该部署需要高可用直聊，评估在 runner usage limit 时切换备用 runner / 模型，或在配置健康检查中提前标记 Codex runner 不可用。
