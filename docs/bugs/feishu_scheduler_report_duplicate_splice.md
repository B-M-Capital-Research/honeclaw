# Bug: Feishu scheduler 报告中段拼入重复正文导致结构退化

## 发现时间

2026-06-16 15:04 CST

## Bug Type

Business Error

## 严重等级

P3

## 状态

New

## GitHub Issue

无，非 P1

## 证据来源

- `data/sessions.sqlite3` -> `session_messages`
  - 巡检窗口：2026-06-16 11:01-15:04 CST。
  - `session_id=Actor_feishu__direct__ou_5f39103ac18cf70a98afc6cfc7529120e5`
  - user `ordinal=247`
  - `timestamp=2026-06-16T12:00:02+08:00`
  - 用户任务为 Feishu scheduler 触发的 `每日公司资讯与分析总结`，要求汇总 TEM、Caris、NBIS、CRWV、NVDA、GOOGL、TSM 的最新资讯、分析师总结和下次财报日期。
  - assistant `ordinal=248`
  - `timestamp=2026-06-16T12:03:30+08:00`
  - assistant final 总长度约 4587 字符；正文在 NVIDIA 段落中途出现拼接断点 `Wolfe 等继续偏北京时间 2026年6月16日 12:00`，随后从开头结论和 TEM / CAI / NBIS / CRWV 段落重新输出一遍。
  - `instr(content, '北京时间 2026年6月16日 12:00。结论：今天没有看到')=1658`，说明重复报告正文从第 1658 字符附近重新开始。
- `data/sessions.sqlite3` -> `cron_job_runs`
  - `run_id=43608`
  - `job_name=每日公司资讯与分析总结`
  - `actor_channel=feishu`
  - `executed_at=2026-06-16T12:03:37+08:00`
  - `execution_status=completed`
  - `message_send_status=sent`
  - `delivered=1`
  - `should_deliver=1`
  - `detail_json={"delivery_key":"j_7c688485:2026-06-16:12:00","receive_id":"...","scheduler":null}`
- 同窗摘要：
  - 最近四小时按真实 `timestamp` 共有 4 个 user turn 与 4 个 assistant final，均以 assistant 收口，没有 user-only 残留。
  - 普通 scheduler 1 条为 `completed + sent + delivered=1`，即本条 `每日公司资讯与分析总结`。
  - assistant final 污染扫描未命中空回复、本机绝对路径、`data/agent-sandboxes`、raw tool 字段、思维痕迹、provider 原始错误、`open_id / chat_id`、SQLite、`cron_job` 或 `data_fetch` 外露。
  - heartbeat 同窗新增 79 条 `noop + skipped_noop + delivered=0` 与 25 条 `execution_failed + skipped_error + delivered=0`，失败分布为 24 条 `heartbeat 输出不是结构化 JSON`、1 条未知状态；均未进入用户可见 assistant final，仍落在既有 heartbeat 结构化退化缺陷范围。
  - 最近四小时无非文档代码提交。

## 端到端链路

1. Feishu scheduler 按 12:00 CST 触发 `每日公司资讯与分析总结`。
2. runner 生成覆盖 7 个公司的一次性长报告。
3. assistant final 正常落库并进入 `cron_job_runs`，发送状态记为 `completed + sent + delivered=1`。
4. 用户可见正文在 NVIDIA 段落中途拼入另一份从开头开始的报告正文，导致报告结构退化、部分段落重复。

## 期望效果

- 同一 scheduler 报告应按用户要求输出每家公司一次，段落顺序稳定。
- 长报告中途不应把前文结论和公司段落重新拼入，也不应出现 `...继续偏北京时间...` 这类明显拼接断点。
- 若模型或流式聚合阶段发生重复片段，应在最终持久化 / 出站前做重复块检测或失败收口，而不是按成功态投递。

## 当前实现效果

- 报告成功生成、落库和投递，用户可以读到大部分公司资讯、分析师口径和财报日期。
- 但 NVIDIA 段落未自然结束，直接拼入第二个开头结论，之后 TEM、CAI、NBIS、CRWV 等段落重复出现。
- 该问题没有表现为未回复、投递失败、错投、空回复或内部实现细节泄露。

## 用户影响

- 这是质量性 bug，不是功能性 bug。
- 用户仍收到完整可读报告，主调度、生成、落库和 Feishu 投递链路均完成。
- 受影响的是报告结构、阅读效率和可信度：重复正文会让用户怀疑报告是否被截断、拼接或覆盖。
- 因此它不影响主功能链路，按规则定级为 `P3`，而不是 `P1/P2`。

## 根因判断

- 初步判断是长报告生成或 ACP 流式聚合阶段出现重复片段拼接，最终持久化层没有识别“报告开头再次出现”的大块重复。
- 该样本不是 Feishu 发送层重复投递：`cron_job_runs` 只有一条 `completed + sent + delivered=1`，问题发生在单条 assistant final 文本内部。
- 该问题也不同于 `feishu_direct_daily_limit_duplicate_assistant_transcript.md`：本轮不是 daily-limit 短路提示重复落库，而是 scheduler 长文 final 内部正文重复。
- 当前只有一条真实样本；若后续同类长报告继续复现，应检查 ACP chunk 聚合、final 文本选择、以及出站前的重复段落/重复标题检测。

## 下一步建议

- 为 scheduler final 增加出站前大块重复检测：当同一报告开头、标题序列或公司段落在单条 final 中重复出现时，至少标记为质量告警。
- 检查 ACP stream 聚合和最终 `final` 抽取逻辑，确认是否可能把旧 partial 与最终答案拼接到同一可见文本。
- 针对 Feishu scheduler 长报告增加回归样本，覆盖“中段重新出现开头结论 / 段落序列”的结构退化。
