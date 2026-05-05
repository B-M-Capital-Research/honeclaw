# Bug: Feishu 定时任务内部失败仍会外发通用失败提示，且 direct session 不回写失败记录

- **发现时间**: 2026-04-27 21:02 CST
- **Bug Type**: System Error
- **严重等级**: P1
- **状态**: New
- **GitHub Issue**: [#22](https://github.com/B-M-Capital-Research/honeclaw/issues/22)
- **证据来源**:
  - 最近一小时真实窗口：`data/sessions.sqlite3` -> `cron_job_runs`
    - `2026-05-05 05:21` 窗口：
      - `run_id=15653` / `job_name=Oil_Price_Monitor_Closing`
      - `execution_status=execution_failed`
      - `message_send_status=sent`
      - `delivered=1`
      - `response_preview=抱歉，处理超时了。请稍后再试。`
      - `error_message=抱歉，处理超时了。请稍后再试。`
      - `detail_json.scheduler.failure_kind=internal_error_suppressed`
    - `2026-05-05 06:22` 窗口：
      - `run_id=15654` / `job_name=OWALERT_PostMarket`
      - `execution_status=execution_failed`
      - `message_send_status=sent`
      - `delivered=1`
      - `response_preview=抱歉，处理超时了。请稍后再试。`
      - `error_message=抱歉，处理超时了。请稍后再试。`
      - `detail_json.scheduler.failure_kind=internal_error_suppressed`
    - 两条最新 run 都说明 scheduler 仍把内部失败压成通用超时文案并登记为 `sent + delivered=1`，修复结论已被真实窗口推翻。
  - 最近一小时真实会话源文件：`data/sessions/Actor_feishu__direct__ou_5f3f69c84593eccd71142ed767a885f595.json`
    - 文件 `updated_at=2026-05-05T05:21:58+08:00`
    - 最新 JSON 会话尾部只有两条 scheduler user turn：
      - `2026-05-05T04:09:17+08:00` `[定时任务触发] Oil_Price_Monitor_Closing`
      - `2026-05-05T05:21:42+08:00` `[定时任务触发] OWALERT_PostMarket`
    - 对应窗口没有新增 assistant 失败提示，也没有补偿写回的 transcript marker。
  - 最近一小时会话镜像：`data/sessions.sqlite3` -> `session_messages`
    - `session_id=Actor_feishu__direct__ou_5f3f69c84593eccd71142ed767a885f595`
    - 最新落库消息仍停在：
      - `ordinal=33`
      - `role=assistant`
      - `timestamp=2026-04-27T08:32:45.019580+08:00`
    - 说明不仅 `2026-05-05` 的失败 assistant 没有回写，连对应 user turn 也未进入 sqlite transcript。
  - 最近一小时真实窗口：`data/sessions.sqlite3` -> `cron_job_runs`
    - `2026-04-27 20:30` 窗口：
      - `run_id=7963` / `job_id=j_a9eee6cd` / `job_name=每日仓位复盘`
      - `run_id=7964` / `job_id=j_286d90cf` / `job_name=美股盘前宏观与财报日历梳理`
    - `2026-04-27 21:00` 窗口：
      - `run_id=7991` / `job_id=j_93e6f575` / `job_name=晚9点盘前推演(XME及加密ETF)`
      - `run_id=7992` / `job_id=j_52a67256` / `job_name=美股盘前分析与个股推荐`
      - `run_id=7993` / `job_id=j_f02dfce5` / `job_name=OWALERT_PreMarket`
      - `run_id=7994` / `job_id=j_917c1c2e` / `job_name=持仓与关注股交易日晚间合并研判`
    - 上述 6 条 run 全部落成：
      - `execution_status=execution_failed`
      - `message_send_status=sent`
      - `delivered=1`
      - `response_preview=抱歉，这次处理失败了。请稍后再试。`
      - `error_message=抱歉，这次处理失败了。请稍后再试。`
    - 说明最近一小时已有多名 Feishu 用户的常规定时任务统一退化为通用失败提示，而不是个别任务偶发波动。
  - 最近一小时真实会话落库：`data/sessions.sqlite3` -> `sessions` / `session_messages`
    - `Actor_feishu__direct__ou_5f39103ac18cf70a98afc6cfc7529120e5`
      - 最新会话仍停在 `2026-04-27T16:54:20+08:00`
      - 本轮 `20:30` 的 `每日仓位复盘` 没有新增 scheduler user turn，也没有新增 assistant 失败消息
    - `Actor_feishu__direct__ou_5f62439dbed2b381c0023e70a381dbd768`
      - 最新会话仍停在 `2026-04-27T16:33:29+08:00`
      - 本轮 `21:00` 的 `美股盘前分析与个股推荐` 没有新增 scheduler user turn，也没有新增 assistant 失败消息
    - `Actor_feishu__direct__ou_5f3f69c84593eccd71142ed767a885f595`
      - 最新会话仍停在 `2026-04-27T08:32:45+08:00`
      - 本轮 `21:00` 的 `OWALERT_PreMarket` 没有新增 scheduler user turn，也没有新增 assistant 失败消息
    - `Actor_feishu__direct__ou_5fe09f5f16b20c06ee5962d1b6ca7a4cda`
      - 最新会话仍停在 `2026-04-27T09:02:42+08:00`
      - 本轮 `21:00` 的 `晚9点盘前推演(XME及加密ETF)` 没有新增 scheduler user turn，也没有新增 assistant 失败消息
    - `Actor_feishu__direct__ou_5f44eaaa05cec98860b5336c3bddcc22d1`
      - 最新会话仍停在 `2026-04-25T14:58:40+08:00`
      - 本轮 `21:00` 的 `持仓与关注股交易日晚间合并研判` 没有新增 scheduler user turn，也没有新增 assistant 失败消息
    - 说明 Feishu scheduler 虽然把失败提示记成“已发送”，但真实 transcript 仍完全没有本轮痕迹。
  - 最近一小时运行日志：`data/runtime/logs/sidecar.log`
    - `2026-04-27 20:31:54.658`：`Actor_feishu__direct__ou_5f39103ac18cf70a98afc6cfc7529120e5` 落成
      - `error="codex acp prompt ended before tool completion: Searching the Web, ..."`
    - `2026-04-27 21:01:55.467`：`Actor_feishu__direct__ou_5f62439dbed2b381c0023e70a381dbd768` 落成
      - `error="codex acp prompt ended before tool completion: Searching the Web"`
    - `2026-04-27 21:02:21.570`：`Actor_feishu__direct__ou_5f3f69c84593eccd71142ed767a885f595` 落成
      - `error="codex acp prompt ended before tool completion: Searching the Web, Searching the Web, Searching the Web"`
    - `2026-04-27 21:02:31.783`：`Actor_feishu__direct__ou_5f44eaaa05cec98860b5336c3bddcc22d1` 落成
      - `error="codex acp prompt ended before tool completion: Searching the Web, Searching the Web, Searching the Web"`
    - 同批日志只看到 `recv`、`agent.run start`、大量 `runner.tool ... Searching the Web` / `hone/data_fetch`，但没有对应的 `session.persist_assistant` 成功落库。
  - 相关已有缺陷对照：
    - [`web_scheduler_codex_acp_unfinished_tool_send_failed.md`](./web_scheduler_codex_acp_unfinished_tool_send_failed.md) 关注 Web scheduler 在同型 unfinished-tool 失败时既未落库也未送达。
    - [`feishu_scheduler_codex_acp_stream_closed_false_sent.md`](./feishu_scheduler_codex_acp_stream_closed_false_sent.md) 关注的是更早的 `stream closed before response` 断流路径。
    - 本单是新的 Feishu scheduler 受影响链路：根因同样属于 Codex ACP unfinished-tool 失败，但当前表象是“通用失败提示被记成 sent/delivered=1，用户侧可能收到一句抱歉，然而会话 transcript 完全不回写”。

## 端到端链路

1. Feishu 常规定时任务到点触发，scheduler 向对应 direct session 注入 `[定时任务触发]` user turn。
2. Codex ACP 进入 `agent.run`，开始执行多轮 `Searching the Web` / `hone/web_search` / `hone/data_fetch`。
3. 在搜索工具仍未完全收敛时，runner 以 `codex acp prompt ended before tool completion` 失败退出。
4. scheduler 外层把底层错误净化成用户态通用文案“抱歉，这次处理失败了。请稍后再试。”，并把 run 记为 `sent + delivered=1`。
5. 但真实 `sessions` / `session_messages` 没有新增本轮 scheduler user turn，也没有新增 assistant 失败消息，导致会话侧无法回溯这轮执行。

## 期望效果

- Feishu scheduler 遇到 unfinished-tool 失败时，至少要满足两件事之一：
  - 把失败提示稳定写入对应 direct 会话，便于用户和巡检后续追溯
  - 或明确把 run 记成“仅通道送达、未写会话”的独立状态，而不是让 transcript 看起来像从未触发
- 当底层错误是 `codex acp prompt ended before tool completion` 时，应保留可审计失败分类，而不是全部压成无法区分的通用失败提示。
- 多个用户、多条常规定时任务不应在同一小时窗里集中退化为相同失败形态。

## 当前实现效果

- `2026-05-05 05:21` 与 `06:22` 的真实定时任务再次落成 `execution_failed + sent + delivered=1`，且失败文案已从旧的“抱歉，这次处理失败了”漂移成“抱歉，处理超时了。请稍后再试。”，说明问题并不限于单一 unfinished-tool 文案分支。
- 对应 direct session JSON 与 sqlite transcript 依旧没有新增 assistant 失败消息，表明 `2026-04-30` 标记为已补的 transcript 补偿在 live 窗口没有实际生效。
- 最近一小时至少 6 条 Feishu 常规定时任务集中落成 `execution_failed + sent + delivered=1`，覆盖多个用户和两个调度窗口（20:30、21:00）。
- `sidecar.log` 明确显示这些 run 的底层错误都属于 `codex acp prompt ended before tool completion`，并非用户 prompt 内容各自独立失败。
- 但 `sessions` / `session_messages` 侧没有相应的 scheduler 注入或 assistant 失败消息，说明当前“已发送”的唯一证据只剩 scheduler 台账。
- 这不是单条任务质量波动，而是 Feishu scheduler 主链路在最近一小时出现成批退化。

## 2026-04-27 止血修复

- 已先按用户侧安全优先完成止血：内部失败不再转换成“抱歉，这次处理失败了。请稍后再试。”外发给用户。
- `crates/hone-channels/src/runtime.rs` 新增 `user_visible_error_message_or_none(...)`：`codex acp`、timeout、provider/protocol 等内部错误返回 `None`，只允许足够具体、可直接面向用户的错误文本通过。
- `crates/hone-channels/src/scheduler.rs` 在非 heartbeat scheduler 失败分支改为：当错误不可外发时 `should_deliver=false`，并记录 `suppressed generic failure fallback` 日志，不再把空正文 + 通用错误记成可投递消息。
- `crates/hone-channels/src/outbound.rs` 的共享 outbound 失败分支同样只在存在具体用户态错误时调用 `send_error`，否则静默记录。
- `bins/hone-feishu/src/handler.rs` 直聊失败分支改为：若已有真实 partial stream 正文则发送正文并标注可能不完整；若只有内部错误、panic、空回复或重启恢复中断，则只写日志，不再补发“请稍后再试”类兜底。
- 已验证：
  - `cargo test -p hone-channels user_visible_error_message -- --nocapture`
  - `cargo test -p hone-feishu failed_reply_text_ -- --nocapture`
  - `cargo check -p hone-channels -p hone-feishu -p hone-discord -p hone-telegram`
- 待真实窗口继续验证：底层 `codex acp prompt ended before tool completion` 仍是上游根因，本轮只保证这类失败不再污染用户侧消息；后续还需要在 ACP runner 层继续修复 pending tool 收口质量。

## 用户影响

- 这是功能性缺陷。用户订阅的常规定时播报在最近一小时集中失败，只收到通用抱歉文案，拿不到任务应产出的正文。
- 即使用户在后续打开会话，也看不到这轮任务发生过什么，无法区分“任务未触发”“任务失败”“任务被吞掉”。
- `2026-05-05` 的两条复发 run 说明该缺陷当前仍是活跃问题，而不是单纯历史分析项。
- 之所以定级为 `P1`，是因为它在最近一小时同时影响多名 Feishu 用户、多个常规定时任务和两个连续调度窗口，已经构成活跃的核心调度能力退化，而不是单任务局部问题。

## 根因判断

- 上游根因与 Web 新单相同，都是 Codex ACP 在搜索工具尚未完成时提前结束 prompt，触发 `unfinished tool` 类失败。
- Feishu 当前还叠加了第二层缺口：scheduler 台账会把净化后的通用失败文案记成 `sent + delivered=1`，但并未把对应失败回写到 direct session transcript。
- 从最新 `detail_json.scheduler.failure_kind=internal_error_suppressed` 与用户态“抱歉，处理超时了”来看，live 链路可能已从最初的 unfinished-tool 文案收口漂移到更宽泛的内部错误抑制/超时 fallback，但“false sent + transcript 无痕迹”这个对用户最关键的故障形态仍未消失。
- 这与 `stream closed before response` 不是同一路径；本轮日志能看到大量工具调用和较长执行时间，说明故障发生在工具收口阶段，而不是 runner 刚启动即断流。

## 下一步建议

- 为 Feishu scheduler 的 `unfinished tool` 失败补专门收口：
  - 保留失败分类
  - 明确是否真正写会话
  - 明确是否真正完成通道送达
- 增加回归：覆盖 Feishu scheduler 在 `Searching the Web` 未完成时的失败场景，验证不会再出现“台账 sent/delivered=1，但 transcript 无痕迹”。
- 将本单与 Web 对应缺陷并行跟踪，确认共享 runner 修复后 Feishu 和 Web 都能一致落库失败消息。

## 修复进展（2026-04-28）

- `crates/hone-channels/src/runtime.rs` 新增 `user_visible_error_message_or_none(...)`：`codex acp prompt ended before tool completion`、协议错误、provider 细节等内部错误返回 `None`，timeout 仍保留用户可理解的超时文案。
- `crates/hone-channels/src/scheduler.rs` 在非 heartbeat scheduler 失败分支使用该函数；内部错误不再外发通用“抱歉，这次处理失败了”，而是落成 `should_deliver=false`、`skipped_error`，并在 metadata 记录 `failure_kind=internal_error_suppressed`。
- 验证：`cargo test -p hone-channels user_visible_error_message_or_none --lib`。
- 上游 ACP pending-tool 根因仍由 Web / ACP 共享缺陷继续跟踪；本单从 Feishu “通用失败外发 + transcript 无痕迹”活跃队列移入 `Later`，若真实窗口继续出现同形态再改回 `New`。

## 修复进展（2026-04-30）

- `crates/hone-channels/src/scheduler.rs` 在非 heartbeat scheduler 的内部失败抑制分支新增会话落库补偿：当 `codex acp prompt ended before tool completion` 等内部错误被判定不可外发时，仍会向对应 direct session 追加一条脱敏 assistant 记录：`本轮定时任务未能完成，系统已记录失败并将在下一次触发时重试。`
- 该补偿只写 transcript，不恢复 Feishu 通道外发；调度台账仍保持 `should_deliver=false` / `skipped_error` / `failure_kind=internal_error_suppressed`，避免把内部 runner 错误或通用抱歉重新推给用户。
- 新增回归 `suppressed_scheduler_failure_persists_single_transcript_marker`，覆盖失败记录可追溯且重复调用不会连续刷同一失败 marker。
- 验证：
  - `cargo test -p hone-channels suppressed_scheduler_failure_persists_single_transcript_marker --lib -- --nocapture`
  - `cargo test -p hone-channels user_visible_error_message_or_none --lib -- --nocapture`
  - `cargo test -p hone-channels scheduler::tests --lib -- --nocapture`
  - `cargo check -p hone-channels`
- 当前结论：`2026-04-30` 的补偿方案曾尝试收口该问题，但 `2026-05-05 05:21/06:22` 的真实窗口表明“内部失败仍记 sent 且 transcript 无痕迹”已复发，因此本单状态改回 `New`，继续沿用 Issue [#22](https://github.com/B-M-Capital-Research/honeclaw/issues/22) 跟踪。
