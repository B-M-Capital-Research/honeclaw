# Bug: 定时任务链路绕过统一输出净化，向用户投递内部思考与未清洗富文本

- **发现时间**: 2026-04-14
- **Bug Type**: Business Error
- **严重等级**: P1
- **状态**: New
- **证据来源**:
  - 最近提交: `ee342b3 feat(channels): harden company memory and rich text delivery`
  - 相关修复提交: `12a5352 fix: sanitize leaked internal agent output`
  - 2026-04-15 当前源码复核:
    - `bins/hone-telegram/src/scheduler.rs:96-100` 仍直接对原始 `response` 调用 `split_html_segments(...)`
    - `bins/hone-discord/src/scheduler.rs:113-114` 仍直接对原始 `response` 调用 `split_into_segments(...)`
    - `bins/hone-feishu/src/scheduler.rs:160-168` 仍直接把原始 `response` 交给 `send_rendered_messages(...)`
  - 代码证据:
    - `bins/hone-telegram/src/listener.rs:40-46`
    - `bins/hone-telegram/src/scheduler.rs:62-102`
    - `bins/hone-discord/src/utils.rs:46-49`
    - `bins/hone-discord/src/scheduler.rs:113-118`
    - `bins/hone-feishu/src/handler.rs:622-689`
    - `bins/hone-feishu/src/scheduler.rs:160-168`

## 端到端链路

1. 用户在 Telegram / Discord / Feishu 创建普通定时任务或 heartbeat 任务，期待到点后收到和正常对话一致的最终答复。
2. 调度器通过 `execute_scheduler_event(...)` 跑出 `response.content`，进入各渠道的 scheduler 发送逻辑。
3. 当前用户对话链路都会先调用 `render_think_blocks(..., ThinkRenderStyle::Hidden)`，Telegram 还会额外经过 `sanitize_telegram_html_public(...)`，再做分段发送。
4. 但 scheduler 链路直接把原始 `response` 传给 `split_html_segments`、`split_into_segments` 或 `send_rendered_messages`，没有复用统一的“隐藏内部思考 / 清洗渠道富文本”步骤。
5. 结果是：一旦模型输出含有 `<think>`、工具协议残渣、Markdown-ish 文本或 Telegram 不支持的 HTML 片段，定时任务就会把这些原样发给用户。

## 期望效果

- 定时任务与普通用户会话应共享同一套最终出站净化规则。
- 用户只能收到最终可见答案，不应看到 `<think>`、工具调用协议、内部控制片段或未清洗的富文本。
- Telegram 定时消息在发送前也应走 HTML 清洗和格式归一化，否则应回退到安全纯文本，而不是直接尝试发送原始模型输出。

## 当前实现效果（问题发现时）

- Telegram 普通会话路径会先隐藏 think 并做 HTML 清洗，但 Telegram scheduler 只做 `split_html_segments(&response, ...)`，没有执行任何净化。
- Discord 普通会话路径会先隐藏 think，再做 Markdown 分段；Discord scheduler 则直接对原始 `response` 分段发送。
- Feishu 普通会话路径在隐藏 think 之后再交给 `send_rendered_messages(...)`；Feishu scheduler 直接把原始 `response` 交给 `send_rendered_messages(...)`。
- 这意味着最近对“多代理内部输出泄漏”和“富文本分段稳定性”的修补，没有完整覆盖调度投递链路。

## 用户影响

- 用户可能在定时提醒里直接看到内部思考、工具协议、半成品富文本，收到的不是产品化后的提醒消息。
- Telegram 场景下更容易出现格式降级或发送失败，因为 scheduler 路径没有先把 Markdown-ish 输出归一化成 Telegram 支持的 HTML。
- 对 heartbeat / 条件提醒这类“自动发出、用户没有上下文纠错机会”的链路来说，异常可感知度高，且会直接损害可信度。

## 当前实现效果（2026-04-15 复核）

- Telegram scheduler 仍然从原始 `response` 直接切分 HTML 分片，没有先执行 `render_think_blocks(..., ThinkRenderStyle::Hidden)` 或 `sanitize_telegram_html_public(...)`。
- Discord scheduler 仍然直接按原始 `response` 分段发送，没有复用普通会话隐藏 think 的出站净化。
- Feishu scheduler 仍然把原始 `response` 直接交给 `send_rendered_messages(...)`，入口层没有补上一致的最终可见文本构造。
- 本轮巡检未发现覆盖这三条 scheduler 出站路径的修复提交，因此该缺陷继续保持 `New`。

## 根因判断

- 输出净化逻辑当前按“入口类型”散落在各渠道对话 listener 中，而不是在 scheduler 与用户会话共享的统一出站层完成。
- `12a5352` 解决了用户对话链路的内部输出泄漏，但 scheduler 发送路径没有同步复用这套规则。
- `ee342b3` 给 Telegram scheduler 补了分段能力，却仍然基于未经净化的原始 `response` 分段，放大了这一缺口。

## 修复线索

- 把 scheduler 出站链路收敛到与普通用户消息相同的“最终可见文本”构造流程，至少统一复用：
  - `render_think_blocks(..., ThinkRenderStyle::Hidden)`
  - 渠道特定的净化函数，如 Telegram 的 `sanitize_telegram_html_public(...)`
  - 统一的 reply prefix / segmenter 语义
- 当前 bug 台账先以 `New` 登记，等待人工确认并转入 `Fixing` / `Fixed` / `Closed`。
