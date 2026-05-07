# Bug: Feishu 定时汇总已送达但未执行最新资讯检索，静默退化为非实时摘要

- **发现时间**: 2026-04-18 12:18 CST
- **Bug Type**: Business Error
- **严重等级**: P3
- **状态**: Closed
- **证据来源**:
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=2923`
    - `executed_at=2026-04-19T12:02:17.355973+08:00`
    - 同一任务 `每日公司资讯与分析总结` 在最新小时窗已经不再表现为 `tool_calls=0 + completed + sent` 的伪完成形态
    - 最新一轮调度台账改为 `execution_failed + sent + delivered=1`，`response_preview` 直接变成 `当前会话上下文过长...仍无法继续`
    - 对应 `hone-feishu.release-restart.log` 也显示本轮经历了一次 `context_overflow_recovery`，重试后执行了 15 次 `data_fetch` 才失败；说明 2026-04-18 记录的“零检索直接成功送达”症状在最新样本中已不再复现
    - 当前同链路已转化为新的功能性失败，改由 [`feishu_scheduler_compact_retry_still_cannot_finish_company_digest.md`](./feishu_scheduler_compact_retry_still_cannot_finish_company_digest.md) 继续跟踪
  - `data/sessions.sqlite3` -> `session_messages`
    - `session_id=Actor_feishu__direct__ou_5f39103ac18cf70a98afc6cfc7529120e5`
    - `2026-04-18T12:00:59.095367+08:00` 用户触发 Feishu 定时任务：`每日公司资讯与分析总结`
    - 请求明确要求：汇总 `TEM/CAI/NBIS/CRWV/NVDA/GOOGL/TSM` 的“最新资讯、最新的分析师总结以及各公司的下次财报发布日期”
    - `2026-04-18T12:02:15.710608+08:00` assistant 已成功落库，但正文直接写出：`当前具体新闻动态与分析师目标价细节未完成最新实时接口校验`
    - 同一条回复仍继续给出 7 家公司的宏观式判断与操作建议，说明不是整轮失败，而是在未完成实时检索的前提下直接生成了看似完整的日报
  - `data/runtime/logs/hone-feishu.release-restart.log`
    - `2026-04-18T04:00:59.216532Z` 同会话进入 `runner.stage=multi_agent.search.start`
    - `2026-04-18T04:01:36.197269Z` `stage=search.done success=true iterations=1 tool_calls=0 live_search_tool=false`
    - `2026-04-18T04:02:15.705437Z` `stage=answer.done success=true iterations=1 tool_calls=0`
    - `2026-04-18T04:02:15.726073Z` `[MsgFlow/feishu] done ... success=true ... tools=none reply.chars=1493`
    - 整轮没有任何 `data_fetch`、`web_search`、`earnings_calendar` 或其它最新资讯查询工具调用，却完成了“最新资讯与分析师总结”类请求
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=2430`
    - `job_id=j_7c688485`
    - `job_name=每日公司资讯与分析总结`
    - `executed_at=2026-04-18T12:02:18.065316+08:00`
    - `execution_status=completed`
    - `message_send_status=sent`
    - `delivered=1`
    - `response_preview` 与会话正文一致，同样明确写出 `当前具体新闻动态与分析师目标价细节未完成最新实时接口校验`
  - 历史对照：
    - 同一任务在 `2026-04-17 12:00` 曾落入 [`feishu_direct_cron_job_iteration_exhaustion_no_reply.md`](./feishu_direct_cron_job_iteration_exhaustion_no_reply.md) 记录的“search 阶段耗尽迭代后整轮无回复”
    - `2026-04-18 12:00` 这轮虽然不再无回复，但退化成“零检索成功送达”的新表现，说明问题已从功能性中断转为质量性静默降级

## 端到端链路

1. Feishu 直达定时任务按日触发“每日公司资讯与分析总结”，要求汇总 7 家公司的最新资讯、分析师总结与财报日期。
2. Multi-agent search 阶段正常启动，但仅 1 轮就以 `tool_calls=0` 收口，没有执行任何实时查询。
3. Answer 阶段也未补做工具查询，直接基于已有上下文或历史记忆生成一份长摘要。
4. 最终用户确实收到了消息，但正文明确承认“最新实时接口校验”未完成，任务要求中的“最新资讯/最新分析师总结”并未真正完成。

## 期望效果

- 这类定时汇总任务应至少执行与任务要求对应的最新资讯、评级/研报、财报日历查询，而不是零检索直接出结论。
- 如果最新检索未执行成功，应显式降级为“本轮未完成最新更新”，而不是继续输出带强结论的日报。
- `cron_job_runs` 的成功态不应对应一条自己承认“未完成最新实时校验”的伪完成摘要。

## 当前实现效果

- 本轮链路在系统层面记为 `completed + sent + delivered=1`，表面上主功能链路正常。
- 但 search/answer 全程 `tool_calls=0`，与任务中“最新资讯、最新分析师总结、财报日期”的信息要求明显不匹配。
- 最终正文还主动承认“当前具体新闻动态与分析师目标价细节未完成最新实时接口校验”，说明模型自己也没有拿到足够的新鲜证据。
- 这不是单纯的表述保守，而是任务内容已静默降级成“基于旧上下文的泛化总结”，用户很难从成功态表象识别本轮其实没有完成预期信息采集。
- 但截至 `2026-04-19 12:00` 的最新巡检，这个特定坏态已不再活跃：同一任务不再以 `tool_calls=0` 伪装成成功完成，而是改成 overflow recovery 后的功能性失败。
- 因此本单关闭，不再把“旧会话 auto compact 后失败提示”混入本单；当前活跃问题已由新文档单独跟踪。

## 用户影响

- 这是质量类缺陷。消息已正确送达，没有出现无回复、错投、系统崩溃或数据损坏。
- 但用户订阅的是“最新资讯与分析师总结”类日报，实际收到的却是缺少实时检索支撑的泛化摘要，任务完成度明显不足。
- 之所以定级为 `P3`，是因为主功能链路仍可用，用户仍收到一条可读消息；当前问题主要是内容时效性与完成度退化，而不是链路中断。

## 根因判断

- 当前 scheduler 触发的这类定时汇总请求，存在“search 阶段过早判定已足够回答”的问题，导致在未调用任何实时工具时就直接进入 answer。
- answer 阶段也没有对“任务要求必须包含最新资讯/分析师总结”建立硬约束，因此会在缺证据状态下继续生成高置信度结论。
- 这与“search 耗尽 8 轮后整轮无回复”不是同一症状：前者是收敛失败后无结果，后者是收敛过早导致伪完成。

## 下一步建议

- 检查 scheduler 触发的“公司资讯汇总”类 prompt 是否缺少必须执行实时检索的硬约束，尤其是 search 阶段的停机条件。
- 为“要求 latest/news/analyst summary 的定时任务”增加回归：若 search 阶段零工具调用，则不得直接标记成功完成。
- 若系统决定允许降级，也应输出明确的“本轮未完成实时更新”失败或部分完成态，而不是继续给出强结论与交易建议。
- 对最新 `2026-04-19 12:00` 起的新失败形态，转入 [`feishu_scheduler_compact_retry_still_cannot_finish_company_digest.md`](./feishu_scheduler_compact_retry_still_cannot_finish_company_digest.md) 继续跟踪，不在本单里混合记录不同根因。
