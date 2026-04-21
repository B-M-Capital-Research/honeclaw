# Bug: Feishu 定时汇总旧会话在自动 compact 后仍无法完成日报，最终退化为“当前会话上下文过长”失败提示

- **发现时间**: 2026-04-19 12:22 CST
- **Bug Type**: System Error
- **严重等级**: P2
- **状态**: Fixed
- **证据来源**:
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=2923`
    - `job_id=j_7c688485`
    - `job_name=每日公司资讯与分析总结`
    - `executed_at=2026-04-19T12:02:17.355973+08:00`
    - `execution_status=execution_failed`
    - `message_send_status=sent`
    - `delivered=1`
    - `response_preview=当前会话上下文过长。我已经自动尝试压缩历史，但这次仍无法继续。请直接继续提问重点、发送 /compact，或开启一个新会话后再试。`
    - 同一任务前一天 `2026-04-18 12:00` 还表现为 [`feishu_scheduler_daily_company_digest_skips_realtime_research.md`](./feishu_scheduler_daily_company_digest_skips_realtime_research.md) 记录的“零检索伪完成”；说明这条链路在最新小时窗已从质量性静默降级漂移成自动 compact 后的功能性失败
  - `data/runtime/logs/hone-feishu.release-restart.log`
    - `2026-04-19T04:01:18.436Z` 记录 `context overflow detected, compacting and retrying`
    - `2026-04-19T04:01:38.973Z` 记录 `context overflow recovery compacted=true`
    - 紧接着第二轮 search 又继续执行 `data_fetch quote TEM/CAI/NBIS/CRWV/NVDA/GOOGL/TSM` 与 `data_fetch news TEM/CAI/NBIS/CRWV/NVDA/GOOGL/TSM`，最后还执行 `data_fetch earnings_calendar`
    - `2026-04-19T04:02:15.602Z` 记录 `stage=search.done success=false iterations=2 tool_calls=15 live_search_tool=true`
    - `2026-04-19T04:02:15.699Z` 最终失败收口为产品化文案：`当前会话上下文过长。我已经自动尝试压缩历史，但这次仍无法继续。...`
    - 这说明本轮不是“search 根本没跑”，而是在一次 overflow recovery 之后仍未能完成 answer
  - `data/sessions.sqlite3` -> `session_messages`
    - `session_id=Actor_feishu__direct__ou_5f39103ac18cf70a98afc6cfc7529120e5`
    - `2026-04-19T12:00:00.416514+08:00` 新一轮 `每日公司资讯与分析总结` 触发
    - `2026-04-19T12:01:38.970370+08:00` 会话写入 `Conversation compacted`，并再次插入一条 `role=user` 的 `【Compact Summary】...` 持仓表
    - 截至本轮巡检时，该会话最后一条可见消息仍是 `role=user` 的任务触发文本，没有看到本轮正常 assistant 正文落库
    - 这说明用户侧虽然收到失败提示，但会话库没有留下与调度台账一致的本轮正常答复，链路在“送达失败文案 / 落库最终结果”上仍存在不一致
  - 相关已有缺陷：
    - [`context_overflow_recovery_gap.md`](./context_overflow_recovery_gap.md) 已修复的是“超窗时直接外泄 provider 原始报错”，而不是“auto compact 后仍无法完成任务”
    - [`feishu_direct_compact_retry_still_cannot_answer_new_topic.md`](./feishu_direct_compact_retry_still_cannot_answer_new_topic.md) 记录的是直聊问答链路；本单是同类根因在 scheduler 日报链路上的独立受影响面

## 端到端链路

1. Feishu 直达定时任务在旧会话里触发 `每日公司资讯与分析总结`，要求汇总 7 家公司的最新资讯、分析师总结与财报日期。
2. search 阶段先执行多轮 `data_fetch`，随后命中 `context overflow detected`。
3. `AgentSession` 触发 `context_overflow_recovery`，会话被 compact 一次后自动重试。
4. 但重试后的 search 继续执行 15 次工具调用，仍未进入可完成 answer 的状态。
5. 最终系统向用户返回统一的“当前会话上下文过长...仍无法继续”失败提示，而不是本轮日报内容。

## 期望效果

- 对这类固定格式的定时汇总，auto compact 后应能稳定完成至少一轮日报生成，而不是把旧会话压力直接转化为失败提示。
- 即使 compact 后仍无法完成，也应保证调度台账、会话落库和用户可见结果三者口径一致。
- scheduler 链路不应因为沿用旧会话上下文而反复退化成“请开启新会话”的 direct-style fallback。

## 当前实现效果

- 最新样本里，系统已经不再外泄 provider 原始超窗报错，而是返回产品化失败文案。
- 但 auto compact 并没有恢复主功能链路；本轮日报在执行 15 次 `data_fetch` 后仍然失败，用户没有拿到任务要求的资讯汇总。
- `cron_job_runs` 把本轮记为 `execution_failed + sent + delivered=1`，而当前 `session_messages` 里没有对应的本轮正常 assistant 正文，说明 scheduler 台账与消息落库仍未完全对齐。
- 这不是单纯质量下降，而是定时任务在真实生产窗口里无法完成交付。

## 用户影响

- 这是功能性缺陷。用户订阅的是自动日报，但最新 12:00 窗口只收到“当前会话上下文过长”的失败提示，没有拿到日报内容。
- 问题发生在 scheduler 主链路，用户在任务触发时没有机会像直聊那样主动切换新会话或重写问题，因此影响高于普通 P3 质量退化。
- 之所以定级为 `P2` 而不是 `P1`，是因为当前证据集中在单一任务链路，且系统至少给出了可理解的失败提示，没有出现误投递、数据损坏或跨用户扩散。

## 根因判断

- 根因更接近“旧会话瘦身策略不足 + scheduler 重试后 search 再次膨胀”，而不是 `context_overflow_recovery_gap.md` 已修复的“超窗错误未产品化”问题。
- 同一任务在 compact 后仍继续执行大量 `quote/news/earnings_calendar` 查询，说明 retry 后没有真正把旧上下文负担压到可答复区间。
- 会话里再次出现 `role=user` 的 `【Compact Summary】...`，说明 compact 产物仍在消息库层面污染 scheduler transcript，这很可能继续放大重试阶段的上下文预算与状态不一致问题。
- 当前坏态已不同于 `2026-04-18` 的“零检索伪完成”；这次是执行了大量工具后仍无法完成 answer，属于新的独立失败形态。

## 下一步建议

- 为 scheduler 长会话补专门的 compact/retry 策略，避免沿用 direct session 的统一 fallback 文案直接结束日报任务。
- 增加回归：同一 `每日公司资讯与分析总结` 会话在旧历史下触发时，compact 后仍应至少产出一条结构化日报或受控部分完成态。
- 补齐台账一致性检查，确保 `cron_job_runs.response_preview`、会话落库和用户实际收到的文案在失败分支上保持一致。
