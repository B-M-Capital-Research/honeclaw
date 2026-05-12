# Bug: Feishu 直聊命中 Codex usage limit 后只返回通用失败，用户请求连续无法完成

- 发现时间：2026-05-12 07:03 CST
- Bug Type：System Error
- 严重等级：P1
- 状态：Fixed
- GitHub Issue：[#40](https://github.com/B-M-Capital-Research/honeclaw/issues/40)

## 证据来源

- 最近四小时运行日志：
  - `data/runtime/logs/sidecar.log`
  - `2026-05-12 06:39:47 CST`：Feishu 直聊 session 收到一条 ASTS 财报分析与建仓判断请求，随后进入 `agent.prepare` / `agent.run`。
  - `2026-05-12 06:39:59 CST`：同一 message_id 记录 `runner.error kind=AgentFailed`，底层 Codex ACP 返回 `usage_limit_exceeded`，并提示可稍后恢复；handler 记录 `completed success=false reply_chars=0`。
  - `2026-05-12 06:40:00 CST`：Feishu 只发送 `reply.send detail=failure_fallback segments.sent=1`。
  - `2026-05-12 06:41:27 CST`：同一 session 再次收到相同主题请求。
  - `2026-05-12 06:41:33 CST`：再次命中 `usage_limit_exceeded`，同样以 `failure_fallback segments.sent=1` 收口。
- ACP 事件日志：
  - `data/runtime/logs/acp-events.log`
  - `2026-05-11T22:41:33Z` 对应 Codex ACP JSON-RPC response 包含 `codex_error_info="usage_limit_exceeded"` 和恢复时间提示；该信息没有被映射成用户可理解的额度/运行能力提示。
- 当前机器不是生产机器，本轮不依赖线上运行态判定修复完成；代码修复以本地可复现错误映射和回归测试为准。

## 端到端链路

1. 用户通过 Feishu 直聊发起 ASTS 财报分析和是否建仓判断请求。
2. Feishu handler 发送处理中 placeholder，并把 user turn 写入当前运行态 session。
3. AgentSession 创建 Codex ACP runner 并发起 `session/prompt`。
4. Codex ACP 返回 `usage_limit_exceeded`，包含“额度已达上限 / 稍后恢复”的可解释失败原因。
5. `user_visible_error_message()` 将包含 `codex acp` 的内部错误统一压成通用失败文案。
6. Feishu handler 发送通用 `failure_fallback`，用户没有得到“服务端 Codex 额度暂时耗尽、可稍后重试”的明确解释。
7. 用户随后重复同一请求，第二轮仍以相同方式失败，实际任务没有完成。

## 期望效果

- 当 runner 返回明确的 usage / quota / entitlement 类错误时，系统应把它映射成脱敏但具体的用户态提示，例如“当前执行额度已用尽，请稍后再试”。
- 运行日志和会话记录应继续保留结构化失败原因，便于运营判断是 runner 额度、用户业务额度、provider quota，还是普通系统错误。
- 若产品要求 Feishu 直聊在 Codex 额度耗尽时仍可用，应切换到可用 runner 或明确降级。

## 根因判断

- 直接触发原因是 Codex ACP 返回 `usage_limit_exceeded`。
- 代码侧 `crates/hone-channels/src/runtime.rs` 的 `looks_internal_error_detail()` 会把包含 `codex acp` 的错误统一视作内部错误；`user_visible_error_message()` 因此返回通用失败文案。
- 现有额度友好文案只覆盖 Hone 自身的“已达到今日对话上限”，没有覆盖 runner / upstream usage limit 这类可解释的资源耗尽错误。
- Feishu handler 的失败收口能发送 fallback，说明当前不是出站投递失败；缺口在错误分类和用户态文案映射。

## 修复记录

- 2026-05-12 11:04 CST：共享错误净化层新增 runner usage-limit 识别，对 Codex / runner / ACP 相关的 `usage limit`、`rate limit`、`quota exceeded`、`quota exhausted`、`insufficient quota`、`try again later` 返回统一用户可见文案：`当前执行额度已用尽，暂时无法继续处理。请稍后再试。`
- Feishu direct 失败回复优先展示上述 usage-limit 错误，不再被 placeholder 或 partial stream 遮蔽。
- 本修复只做通用错误边界加固，不为单次外部额度耗尽写重试或硬编码绕过。

## 验证

- `cargo test -p hone-channels user_visible_error_message --lib -- --nocapture`
- `cargo test -p hone-feishu failed_reply_text_keeps_codex_usage_limit_over_partial_stream -- --nocapture`
- `cargo check -p hone-channels -p hone-feishu --tests`

## 后续建议

- 若部署当前代码后仍有 Feishu direct 在 Codex usage-limit 窗口返回通用失败，应记录脱敏错误字符串并检查是否属于新的 runner 文案变体，再扩展通用识别规则。
- 若该部署需要高可用直聊，评估在 runner usage limit 时切换备用 runner / 模型，或在配置健康检查中提前标记 Codex runner 不可用。
