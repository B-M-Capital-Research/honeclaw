# Bug: Feishu 用户触发日对话额度上限后仍无最终额度提示，且最新 user turn 不落库

- 发现时间：2026-04-16 15:52 CST
- Bug Type：Business Error
- 严重等级：P1
- 状态：New
- GitHub Issue：[#26](https://github.com/B-M-Capital-Research/honeclaw/issues/26)

## 证据来源

- 2026-05-01 19:02 最近一小时新增复现：
  - 运行日志：
    - `data/runtime/logs/sidecar.log`
    - `2026-05-01 19:02:15.383` 记录 `step=message.accepted user=+8616620121491 text_chars=6`
    - `2026-05-01 19:02:17.205` 紧接着发送 `step=reply.placeholder`
    - `2026-05-01 19:02:17.212` 同一 session 再次直接落成 `step=handler.session_run ... completed success=false reply_chars=0`
    - 同一秒再次记录 `suppressed generic failure fallback: ... 已达到今日对话上限（12/12，北京时间 2026-05-01），请明天再试`
    - 同窗仍没有对应的 `session.persist_user`、`session.persist_assistant` 或 `reply.send`
  - 会话快照：
    - `data/sessions/Actor_feishu__direct__ou_5f680322a6dcbc688a7db633545beae42c.json`
    - 文件最新消息仍停在 `2026-05-01T06:06:31.075388+08:00` 的 assistant 回复；`19:02` 这轮新 user turn 和任何 quota 提示都没有写入
  - 这说明该链路在 `09:03`、`12:59` 之后，到 `19:02` 仍持续以相同形态复现；先前 `Fixed` 结论已失效，应恢复为活跃 `P1`

- 2026-05-01 12:59 最近一小时新增复现：
  - 运行日志：
    - `data/runtime/logs/sidecar.log`
    - `2026-05-01 12:59:13.198` 记录 `step=message.accepted user=+8616620121491 text_chars=8`
    - `2026-05-01 12:59:14.502` 紧接着发送 `step=reply.placeholder`
    - `2026-05-01 12:59:14.508` 同一 session 直接落成 `step=handler.session_run ... completed success=false reply_chars=0`
    - `2026-05-01 12:59:14.509` 同一秒再次记录 `suppressed generic failure fallback: ... 已达到今日对话上限（12/12，北京时间 2026-05-01），请明天再试`
    - 同窗没有对应的 `session.persist_user`、`session.persist_assistant` 或 `reply.send`
  - 会话快照：
    - `data/sessions/Actor_feishu__direct__ou_5f680322a6dcbc688a7db633545beae42c.json`
    - 文件最后两条消息仍是 `2026-05-01T06:04:50.527230+08:00` 的 user turn `DJT适合加仓吗` 与 `2026-05-01T06:06:31.075388+08:00` 的 assistant 回复；`12:59` 这轮新 user turn 和任何 quota 提示都没有写入
  - 这说明旧缺陷不只是 `2026-04-30 22:36` 的一次性回归，而是在 `2026-05-01 12:59` 继续以同一链路稳定复现：quota 触顶后仍是“placeholder 后无最终回复 + 无 user turn 落库”

- 2026-04-30 22:36 最近一小时新增复现：
  - 运行日志：
    - `data/runtime/logs/sidecar.log`
    - `2026-04-30 22:36:35.124` 记录 `step=message.accepted user=+8613811525279 text_chars=41`
    - `2026-04-30 22:36:36.114` 紧接着发送 `step=reply.placeholder`
    - `2026-04-30 22:36:36.119` 同一 session 直接落成 `step=handler.session_run ... completed success=false reply_chars=0`
    - 同一秒还记录 `suppressed generic failure fallback: ... 已达到今日对话上限（12/12，北京时间 2026-04-30），请明天再试`
    - 同窗没有对应的 `session.persist_user`、`session.persist_assistant` 或 `reply.send`
  - 会话快照：
    - `data/sessions/Actor_feishu__direct__ou_5f0e001c305cfc075babe830a9b2c6079c.json`
    - 文件最新消息仍停在 `2026-04-30T22:35:49.433594+08:00` assistant 对 `aur今天为什么大涨` 的答复；`22:36` 的新 user turn 和任何 quota 提示都没有写入
  - 这说明旧缺陷并未保持修复：用户触顶 quota 后，当前坏态已经从“收到通用稍后再试”回退成“placeholder 后完全没有最终回复”，并且最新 user turn 再次丢失
- 会话库：
  - `data/sessions.sqlite3`
  - session_id：`Actor_feishu__direct__ou_5f5ffb1004abf2c344917ee093ffb14c15`
- 会话快照：
  - `data/sessions/Actor_feishu__direct__ou_5f5ffb1004abf2c344917ee093ffb14c15.json`
- 额度快照：
  - `data/conversation_quota/feishu__direct__ou_5f5ffb1004abf2c344917ee093ffb14c15/2026-04-16.json`
- 运行日志：
  - `data/runtime/logs/web.log`
  - `data/runtime/logs/hone-feishu.release-restart.log`
  - 最近一小时新增复现：
    - `2026-04-16 22:22:06.869` `web.log` 记录 `step=message.accepted user=+8613121812525 text_chars=16`
    - `2026-04-16 22:22:07.978` 同一 session 紧接着只出现 `step=reply.placeholder`
    - 会话库对应新增的最新消息仍只有 `2026-04-16T22:22:07.994063+08:00` assistant 失败兜底文案，新的 user turn 没有入库
    - `data/conversation_quota/feishu__direct__ou_5f5ffb1004abf2c344917ee093ffb14c15/2026-04-16.json` 仍显示 `success_count = 12`、`in_flight = 0`

## 端到端链路

1. Feishu 直聊用户 `+8613121812525` 在 2026-04-16 15:44 左右连续发送了短文本消息，用户侧观察到“hi”“你好啊”后直接收到“抱歉，这次处理失败了。请稍后再试。”。
2. `web.log` 记录该会话最新一条消息仅到：
   - `2026-04-16 15:44:17` `step=message.accepted`
   - `2026-04-16 15:44:18` `step=reply.placeholder`
3. 同一时间窗里没有出现该 message_id 对应的：
   - `session.persist_user`
   - `agent.prepare`
   - `agent.run`
4. 会话库最终只新增了一条 assistant 失败兜底消息，时间为 `2026-04-16T15:44:18.359632+08:00`，正文为“抱歉，这次处理失败了。请稍后再试。”。
5. 同一 actor 的额度文件显示：
   - `quota_date = 2026-04-16`
   - `success_count = 12`
   - `in_flight = 0`
6. `crates/hone-channels/src/agent_session.rs` 中 `reserve_conversation_quota()` 在超限时会直接返回：
   - `已达到今日对话上限（...），请明天再试`
   且该分支发生在 `session.persist_user` 之前。
7. 最近一小时同一 actor 再次复现相同形态：
   - `2026-04-16 22:22:06` 新 text message 已被 handler 接收
   - 但之后仍只看到 placeholder 与通用失败兜底，既没有新 user turn 落库，也没有进入 `agent.prepare / agent.run`
   - 说明这不是 15:44 的一次性事件，而是 quota 触顶后直到夜间仍会稳定复现

## 期望效果

- 当用户触发当日对话额度上限时，渠道应明确返回“已达到今日对话上限，请明天再试”或等价的明确说明，而不是误导性的“稍后再试”。
- 即使因为 quota 被拒绝，本轮用户输入也应至少保留可审计痕迹，避免支持侧无法从 session 里还原用户到底发了什么。
- 前端/渠道文案应能区分：
  - 真正的临时系统失败
  - 业务规则拒绝（如 quota、权限、白名单）

## 当前实现效果

- 最新坏态已经比 `2026-04-16` 更差，而且到 `2026-05-01 19:02` 仍未恢复：用户命中 quota 后只看到 placeholder，随后既没有“已达到今日对话上限”的友好提示，也没有通用失败兜底。
- 同窗日志明确显示 handler 知道真实错误是 `已达到今日对话上限（12/12）`，但又因为 `suppressed generic failure fallback` 把最终用户态回复整轮吞掉。
- `data/sessions/Actor_feishu__direct__ou_5f0e001c305cfc075babe830a9b2c6079c.json` 与 `data/sessions/Actor_feishu__direct__ou_5f680322a6dcbc688a7db633545beae42c.json` 都证明最新 user turn 没有进入会话历史；后者在 `2026-05-01 19:02` 复现时仍停在 `06:06:31+08:00` 的上一轮 assistant 成功答复。
- 因此当前不是单纯“文案不友好”，而是 quota 拒绝在 Feishu 直聊里再次退化成“无最终回复 + 无 user turn 落库”的功能性故障。

## 用户影响

- 受影响用户会被稳定阻断，而且现在连“明天再试”的明确提示都看不到，只会感知为机器人卡住或吞消息。
- `2026-04-30 22:36`、`2026-05-01 09:03`、`12:59`、`19:02` 四个真实样本里，placeholder 发出后都整轮无最终回复，属于 Feishu 直聊主链路的明显功能失败，因此维持 `P1`。
- 由于最新 user turn 未落库，排障与客服支持会丢失关键信息，容易把 quota 问题误归因为系统故障。

## 根因判断

- `2026-04-30 22:36` 与 `2026-05-01 12:59` 的日志都说明，底层 quota 判定本身仍然存在，而且 handler 也拿到了业务拒绝文案；但 Feishu 失败收口又把这类 quota 错误误判成“generic failure fallback 应 suppress”，导致最终用户态文案被整轮吞掉。
- 根因一：`AgentSession::run()` 在 `reserve_conversation_quota()` 被拒绝时直接 `fail_run()`，该路径发生在 `session.persist_user` 之前，因此用户消息不会写入会话。
- 根因二：Feishu handler 当前对 `response.success == false` 统一使用通用失败文案收口，没有把业务拒绝类错误映射成用户可理解的专用提示。
- 根因三：当前外层可观测性对“系统失败”和“业务规则拒绝”的区分不够，导致真实原因被掩盖。
- 最近四次真实窗口（`2026-04-30 22:36`、`2026-05-01 09:03`、`12:59`、`19:02`）都说明问题不依赖特定消息内容；只要当日额度已达上限，这条链路当前就会稳定退化成“placeholder 后无最终回复”。

## 修复结论变化（2026-05-01 19:02）

- 虽然 `2026-05-01` 代码侧已经补过 quota 优先级保护和 handler 回归测试，但最近一小时真实 Feishu 流量仍然命中同一坏态。
- 因此本单不能维持 `Fixed`；当前结论应回退为 `New`，继续按活跃 `P1` 跟踪，已有 GitHub Issue `#26` 可继续复用，无需重复建单。

## 修复情况（2026-04-17）

- `crates/hone-channels/src/agent_session.rs` 的 `reserve_conversation_quota()` 现在直接返回用户态额度提示文本，不再经过 `HoneError::Tool(...)` 包装，避免下游把这类业务拒绝误判成内部错误。
- 同一文件的 quota 拒绝分支现在会先补最小 `session.persist_user` 审计落库，再返回失败结果；因此即使本轮被额度拦截，session 历史里也能看到用户真实输入。
- 这条修复不增加 `success_count`，仍会保持 quota 拒绝不计入成功对话数；新增回归测试已覆盖“明确 quota 文案 + user turn 落库 + 不触发 LLM”三件事。
- 代码层曾完成一轮修复并通过 crate 级验证，但 `2026-04-30 22:36` 的真实 Feishu 流量已证明当前运行态出现回归/变体：旧问题不但没有保持 `Fixed`，还退化成“无最终回复”。本单现恢复为 `New`。

## 修复情况（2026-04-30）

- `crates/hone-channels/src/agent_session/core.rs` 将 quota 拒绝从 `HoneError::Tool(...)` 改为用户态 `HoneError::Other(...)`，避免 Feishu 失败收口把“已达到今日对话上限”误判为内部工具错误并 suppress。
- 同一 quota 拒绝分支在返回失败前补写当前 user turn，并记录 `session.persist_user=quota_rejected`，因此即使不进入 LLM，也能在会话历史中追溯用户输入。
- 该路径不预留也不提交 quota，不会增加 `success_count`，仍保持超限请求不计入成功对话数。
- 回归测试 `run_rejects_over_daily_limit_with_user_turn_and_friendly_error` 覆盖：
  - 返回明确 quota 文案且不带 `工具执行错误` 前缀。
  - 不调用 LLM。
  - 会话历史写入 1 条 user turn。
  - quota 计数保持 `success_count=daily_limit / in_flight=0`。
- 验证：
  - `cargo test -p hone-channels run_rejects_over_daily_limit_with_user_turn_and_friendly_error -- --nocapture`

## 修复情况（2026-05-01）

- `bins/hone-feishu/src/handler.rs` 的失败收口新增 quota 业务拒绝优先级保护：当错误为“已达到今日对话上限”时，即使 placeholder 或 stream probe 已留下 partial 文本，也必须优先向用户展示 quota 文案，而不是保留 placeholder / partial 或退回通用失败文案。
- 会话层 `AgentSession::run()` 既有 quota 拒绝 user turn 落库路径保持不变，本轮补 Feishu handler 级回归，覆盖“quota 错误优先于 placeholder partial”这一渠道侧契约。
- 验证：
  - `cargo test -p hone-feishu failed_reply_text_keeps_quota_error_over_placeholder_partial -- --nocapture`
  - `cargo test -p hone-channels run_rejects_over_daily_limit_with_user_turn_and_friendly_error -- --nocapture`
- 关联 GitHub Issue：[#26](https://github.com/B-M-Capital-Research/honeclaw/issues/26)

## 回归验证

- `cargo test -p hone-channels run_rejects_over_daily_limit_with_user_turn_and_friendly_error -- --nocapture`
- `cargo test -p hone-channels`
- `cargo check -p hone-channels`
- `rustfmt --edition 2024 --check crates/hone-channels/src/agent_session.rs`

## 下一步建议

- 优先检查 `bins/hone-feishu/src/handler.rs` 里 `suppressed generic failure fallback` 与 quota 错误的交互，确保 `已达到今日对话上限` 不会再被当作需要 suppress 的 generic failure。
- 继续补结构化日志字段，例如 `failure_kind=quota_rejected`，便于监控和 bug 巡检直接聚类。
- 建议补一条 Feishu handler 级回归，直接锁住“quota 拒绝时必须发送用户态额度提示，而且不能丢 user turn”这一渠道侧契约。
