# Bug: OpenAI-compatible 搜索阶段出现 tool-call 协议错位，`invalid params` 失败被统一收口成通用失败提示

- **发现时间**: 2026-04-16 13:30 CST
- **Bug Type**: System Error
- **严重等级**: P1
- **状态**: Fixed
- **证据来源**:
  - 最近失败样本横向比对：`data/runtime/logs/web.log`
    - 在最近 5000 行内共识别到 17 条 `MsgFlow/feishu failed`
    - 其中 13 条底层错误完全相同：`LLM 错误: bad_request_error: invalid params, tool call result does not follow tool call (2013)`
    - 对比同一时间窗其它错误仅有：
      - `opencode acp session/prompt idle timeout (180s)` 2 条
      - `已达最大迭代次数 8` 1 条
      - `http error: error sending request for url (https://api.minimaxi.com/v1/chat/completions)` 1 条
    - 说明最近用户感知到的大量“抱歉，这次处理失败了。请稍后再试。”，其主导根因并不是单次网络抖动，而是这类协议错位错误
  - 最近真实会话 1：图片持仓识别链路
    - `session_id=Actor_feishu__direct__ou_5f5ffb1004abf2c344917ee093ffb14c15`
    - `2026-04-16 00:05:55.268`、`00:05:57.918`、`00:06:00.133`、`00:06:48.002`、`00:07:21.540`、`00:07:23.578`、`00:07:25.806`、`00:07:28.410`、`01:10:05.509`、`01:10:08.485`
    - 同一会话在用户补发图片后持续触发相同 `invalid params` 错误，期间没有稳定完成持仓识别任务
  - 最近真实会话 2：普通直聊问答链路
    - `session_id=Actor_feishu__direct__ou_5f39103ac18cf70a98afc6cfc7529120e5`
    - `2026-04-16T13:12:40.613607+08:00` 用户提问：`高质量的软件成长股 估值已经降了不少 但成长性并没有崩的 反而是机会 当然也包括非软件类的 这部分其实也是有配置需求的，这个对吗`
    - `2026-04-16 13:12:44.615` 日志记录 `failed ... error="LLM 错误: bad_request_error: invalid params, tool call result does not follow tool call (2013)"`
    - `sessions` 表中该会话最新消息仍停留在这条 `role=user`，没有新的 assistant 落库，说明用户侧这轮没有拿到最终答复
  - 最近真实会话 3：晨间轻量直聊
    - `session_id=Actor_feishu__direct__ou_5fe31244b1208749f16773dce0c822801a`
    - `2026-04-16 08:32:00.009` 与 `08:32:03.641` 两次记录同一 `invalid params` 错误
    - 该用户后续在 `09:55:54` 再发 `在吗`，`09:56:14` 才恢复拿到正常回复，说明失败后仍需要用户人工再次唤起
  - 相关历史文档：
    - `docs/bugs/channel_raw_llm_error_exposure.md`
    - `docs/bugs/feishu_attachment_internal_transcript_leak.md`
  - 代码线索：
    - Multi-agent 搜索阶段 provider 为 OpenAI-compatible `https://api.minimaxi.com/v1`
    - 用户态错误在 `crates/hone-channels/src/runtime.rs` 被统一收口为通用失败提示
  - 2026-04-16 当前源码修复与验证：
    - `crates/hone-channels/src/runners/multi_agent.rs` 中 `sanitize_search_context(...)` 现已同时移除历史 `tool` 消息、剥离历史 assistant 上残留的 `tool_calls`，并丢弃只剩空壳 tool-call 的 assistant 消息
    - 定向回归：`cargo test -p hone-channels sanitize_search_context -- --nocapture` 通过
    - release 打包验证：`env CARGO_TARGET_DIR=/Users/ecohnoch/Library/Caches/honeclaw/target bunx tauri build --config bins/hone-desktop/tauri.generated.conf.json` 通过

## 端到端链路

1. 用户在 Feishu 直聊中发起正常问题，或补充图片/继续追问。
2. Multi-agent 搜索阶段开始执行工具调用，部分工具结果已经成功返回。
3. 随后 OpenAI-compatible provider 返回 `bad_request_error: invalid params, tool call result does not follow tool call (2013)`。
4. 当前系统会把原始内部错误净化成统一用户态失败提示，因此用户看到的是“抱歉，这次处理失败了。请稍后再试。”
5. 但搜索阶段本身没有自动恢复能力，最终表现为本轮问题失败、用户需要再次重试。

## 期望效果

- 搜索阶段的 tool-call 协议应保持严格闭合，不应再触发 `tool call result does not follow tool call` 这类 provider 协议级错误。
- 即便出现这类协议错位，也应有更明确的恢复或降级策略，而不是让用户只看到通用失败提示然后自行重试。
- 质量巡检与缺陷台账应把这类主导性根因单独跟踪，而不是被“通用失败提示”掩盖。

## 当前实现效果

- 用户侧现在不会再看到原始 `invalid params` 文本，这说明错误净化层本身是生效的。
- 修复前，从最近失败样本统计看，通用失败提示背后最常见的根因正是这条协议错位错误，而不是超时或网络抖动。
- 根因在于搜索阶段为了避免旧工具结果污染新一轮搜索，只删除了历史 `tool` 消息，却保留了对应 assistant `tool_calls`；这会把 transcript 变成“不完整的 tool-call 对”，进而被 OpenAI-compatible provider 以 `tool call result does not follow tool call (2013)` 拒绝。
- 当前源码已把这类历史 assistant `tool_calls` 一并清洗，且会丢弃只剩 tool-call 骨架、没有正文内容的 assistant 消息，避免继续把无效协议片段带入新一轮搜索阶段。

## 用户影响

- 这是功能性缺陷，不是单纯文案问题。用户的主问题无法完成，只能再次发送消息碰运气恢复。
- 之所以定级为 `P1`，是因为它已经成为最近通用失败提示背后的主导根因，且覆盖普通 Feishu 直聊主链路。
- 之所以不是 `P0`，是因为当前证据仍主要集中在单渠道、单 provider 族的搜索阶段，没有证明系统全局不可用。

## 根因判断

- 直接触发点是 OpenAI-compatible 搜索阶段在 tool-call 序列上出现协议错位，provider 因 `tool call result does not follow tool call` 拒绝继续处理。
- 更具体地说，`sanitize_search_context(...)` 旧实现只删除 `role=tool` 的历史消息，没有同步删除或剥离与之配对的 assistant `tool_calls`，从而构造出了 provider 侧非法的消息序列。
- 现有系统之前只解决了“这类内部错误不要直接暴露给用户”，没有解决“这类协议错误为什么频繁发生、发生后如何恢复”；本轮已完成源码级修复和回归验证。

## 下一步建议

- 持续观察 `data/runtime/logs/web.log` 中是否还有新的 `tool call result does not follow tool call (2013)` 样本，以确认线上行为已收敛。
- 若后续仍有零星 provider 协议失败，可继续考虑在搜索阶段为这类已知协议错误增加一次受控重建/重试。
