# Bug: MiniMax 搜索阶段 HTTP 发送失败后缺少自动重试与降级，用户仅收到通用失败提示

- **发现时间**: 2026-04-16 13:08 CST
- **Bug Type**: System Error
- **严重等级**: P2
- **状态**: Fixed
- **证据来源**:
  - 最近真实会话：`data/sessions.sqlite3` -> `sessions` / `session_messages`
    - `session_id=Actor_feishu__direct__ou_5f39103ac18cf70a98afc6cfc7529120e5`
    - `2026-04-16T13:05:06.310005+08:00` 用户提问：`rklb要不要加`
    - `2026-04-16T13:05:33.912242+08:00` 本轮先触发一次 auto compact
    - `2026-04-16T13:08:39.185` 对应日志记录本轮搜索阶段失败，用户侧收到统一失败提示
    - `2026-04-16T13:09:31.197201+08:00` 用户再次发送同一句 `rklb要不要加`
    - `2026-04-16T13:10:29.132843+08:00` 第二次尝试成功返回完整 assistant 答复，说明同一问题在短时间内可恢复
  - 最近运行日志：`data/runtime/logs/web.log`
    - `2026-04-16 13:05:34.047` `runner.stage=multi_agent.search.start`
    - `2026-04-16 13:05:45.267` 成功执行 `local_search_files query="RKLB Rocket Lab" path="company_profiles"`
    - `2026-04-16 13:05:48.600` 成功执行 `data_fetch snapshot RKLB`
    - `2026-04-16 13:05:57.558` 成功执行 `data_fetch earnings_calendar`
    - 随后长时间没有新的工具完成事件，直到 `2026-04-16 13:08:39.158` 记录 `stage=search.done success=false iterations=2 tool_calls=2 live_search_tool=true elapsed_ms=185113`
    - 紧接着 `2026-04-16 13:08:39.185` 记录 `error="LLM 错误: http error: error sending request for url (https://api.minimaxi.com/v1/chat/completions)"`
    - `2026-04-16 13:09:31.333` 同一会话再次进入搜索阶段
    - `2026-04-16 13:10:00.995` 第二次搜索成功，`2026-04-16 13:10:29.134` assistant 成功落库，说明失败不是由用户输入不可处理导致
  - 历史同类日志：
    - `data/runtime/logs/web.log`
      - `2026-04-06 15:25:39.754` Feishu 直聊搜索阶段同样记录 `http error: error sending request for url (https://api.minimaxi.com/v1/chat/completions)`
      - `2026-04-13 11:23:26.335` Feishu 直聊搜索阶段再次出现相同错误
      - `2026-04-15 09:06:44.283` Feishu 会话再次出现相同错误
    - `data/runtime/logs/hone-feishu.release-restart.log`
      - `2026-04-15T08:05:48.658410Z` session compactor 也出现相同 `chat/completions` 发送失败，说明该问题不只影响单一用户会话
  - 代码线索：
    - 搜索阶段 provider 仍指向 `https://api.minimaxi.com/v1`
    - 通用用户态文案收口位于 `crates/hone-channels/src/runtime.rs`
  - 2026-04-16 配置与连通性复核：
    - `crates/hone-tools/src/web_search.rs` 已确认 `web_search` 工具走 Tavily，不走 MiniMax
    - 当前 desktop 生效配置中 Tavily key 池存在 4 个 key，其中抽查结果为 1 个 `HTTP 432` 配额拒绝、3 个可正常返回 `200`
    - 因此，本缺陷中的 `https://api.minimaxi.com/v1/chat/completions` 发送失败属于 multi-agent search provider 传输问题，不是 Tavily 缺 key 或 Tavily 全局不可用所致
  - 相关缺陷：
    - `docs/bugs/channel_raw_llm_error_exposure.md`
    - `docs/bugs/feishu_direct_cron_job_iteration_exhaustion_no_reply.md`

## 端到端链路

1. Feishu 用户在直聊里提问 `rklb要不要加`。
2. 会话先完成 auto compact，随后 Multi-Agent 搜索阶段启动，前两次工具调用成功返回。
3. 搜索阶段继续向 MiniMax `chat/completions` 发起后续请求时，HTTP 层直接报 `error sending request for url (...)`。
4. 当前链路没有针对这类传输失败做自动重试、provider fallback 或“保留已有证据先给部分结论”的降级处理。
5. 用户最终只看到统一兜底文案“抱歉，这次处理失败了。请稍后再试。”；手动重发同一句后，本轮很快成功。

## 期望效果

- 当搜索阶段遇到 `http error: error sending request for url (...)` 这类传输层失败时，系统应至少自动重试一次，而不是立即整轮失败。
- 若重试后仍失败，应考虑保留已完成的工具结果做降级答复，或切换到明确的“上游搜索服务暂时不可用”提示，而不是让用户自行重复提问碰运气。
- 对同一 provider 的持续性传输失败，应有更明确的可观测性与聚合策略，避免它只以分散的通用失败提示出现。

## 当前实现效果

- 这次真实会话里，搜索阶段已经完成了本地资料检索和两次 `data_fetch`，说明并非“任务还没开始”或“用户输入非法”。
- 真正导致失败的是继续调用 MiniMax `chat/completions` 时发生 HTTP 发送失败；该错误发生在搜索阶段内部，而非 answer 阶段。
- 用户侧现在拿到的是统一安全文案“抱歉，这次处理失败了。请稍后再试。”，不会再看到原始 provider 报错；这说明错误净化是生效的。
- 但从用户任务完成角度看，当前行为仍然退化为“只要上游发送失败一次，这一轮就直接失败”，缺少自动恢复。
- 同一句问题在 52 秒后重发即成功，进一步说明这更像是瞬时传输抖动，而系统当前没有把这种抖动吸收掉。
- 与最近其它失败样本横向对比后可确认：这条 HTTP 发送失败是真实活跃问题，但不是当前通用失败提示的主导根因；主导根因另见 `openai_compatible_tool_call_protocol_mismatch_invalid_params.md`。

## 用户影响

- 这是功能性缺陷，不是单纯文案问题。用户这轮问题没有被完成，只能手动再问一次。
- 之所以定级为 `P2`，是因为当前证据更像是上游/网络瞬时抖动触发的恢复性失败，而不是稳定必现或跨用户持续不可用；手动重试通常可恢复。
- 之所以不是 `P3`，是因为损害不在于“回答质量一般”，而在于主链路直接失败。

## 根因判断

- 直接触发点是 MiniMax 搜索阶段向 `https://api.minimaxi.com/v1/chat/completions` 发送请求时发生 HTTP 传输失败。
- 现有系统虽然已经把原始报错净化成统一用户态文案，但没有在搜索阶段对这类传输错误做自动重试或 fallback。
- 历史日志里从 4 月 6 日到 4 月 16 日多次出现同一错误，说明这不是单次会话特例，而是上游不稳定与本地缺少吸震策略共同形成的活跃缺陷。
- 这条缺陷与 `channel_raw_llm_error_exposure` 不同：后者关注“报错是否暴露给用户”，本条关注“对已知传输抖动是否具备自动恢复能力”。

## 修复进展

- 2026-04-18 当前工作区里已经出现面向 `crates/hone-llm/src/openai_compatible.rs` 的 provider 级重试补丁草案，目标是在 `chat` / `chat_with_tools` 命中 `error sending request for url (...)`、连接被提前关闭或同类瞬时超时信号时补一次自动重试。
- 同一工作区还出现了配套的本地 HTTP 假服务测试草案，方向与本缺陷根因一致。
- 但截至本轮巡检结束，上述补丁仍是未提交的本地改动，不属于仓库主线事实，也没有最近一小时真实会话样本可证明已收口，因此当前状态只能更新为 `Fixing`，不能记为 `Fixed`。

## 后续观察点

- 待补丁提交并进入仓库主线后，重新用真实会话或线上日志复核是否还出现 `error sending request for url (https://api.minimaxi.com/v1/chat/completions)`。
- 若失败发生前已经积累了足够工具结果，后续仍可评估 answer 阶段是否允许基于现有证据生成降级答复，避免整轮清零。
- 继续聚合 `error sending request for url (https://api.minimaxi.com/v1/chat/completions)` 的生产样本，区分“单次抖动”与“持续 provider 故障”。
