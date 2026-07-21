# Bug: Feishu 直聊取消关注 CBRS 后仍未关联并取消活跃心跳任务

- 发现时间：2026-07-21 19:02 CST
- Bug Type：Business Error
- 严重等级：P2
- 状态：New
- GitHub Issue：无，非 P1

## 证据来源

- `data/sessions.sqlite3`
  - 巡检窗口：2026-07-21 15:02-19:02 CST。
  - 最近四小时新增 13 条 user、9 条 assistant、4 条 system compact，覆盖 4 个更新 session；最近会话均以 assistant 收口，未见空回复、错投、原始 provider 错误、`<think>` 或本机路径进入 assistant final。
  - session_id：`Actor_feishu__direct__ou_5f64ee7ca7af22d44a83a31054e6fb92a3`
  - `2026-07-21T15:42:15.042153+08:00` 用户要求取消 `poet 和cbrs` 心跳任务。
  - `2026-07-21T15:43:01.057945+08:00` assistant 先调用 `cron_job action="list"`，并向用户确认两条任务仍已启用：`SIVE POET/Nokia/1.6T DFB 心跳检测`（`j_06c962f6`）与 `Cerebras IPO与业务进展心跳监控`（`j_9ee85d42`），且需要用户确认删除。
  - `2026-07-21T17:57:44.922564+08:00` 用户继续要求：`取消对 cbrs 的关注`。
  - `2026-07-21T17:57:54.233293+08:00` assistant final 回复：`CBRS（美光/CBRE）目前不在关注列表中，无需重复取消。若你指的是其他标的（比如 Cerebras 代码 CBRS），请确认具体代码，我来操作。`
- `data/runtime/logs/web.log.2026-07-21`
  - `2026-07-21 17:57:49` 直聊 runner 实际执行的是 `portfolio unwatch CBRS`，不是 `cron_job remove` 或 heartbeat 取消。
  - 同一日志显示 `j_9ee85d42` / `Cerebras IPO与业务进展心跳监控` 在用户 17:57 取消后仍继续运行：
    - `18:00:01` run_start，`18:00:15` run_finish，随后 `PlainTextNoop`，本轮不发送。
    - `18:30:01` run_start，`18:30:20` run_finish，生成 `CBRS 心跳检查 | 本轮结果：noop` 后被 duplicate suppression 压掉。
    - `19:00:01` run_start，`19:00:21` run_finish，继续生成 `CBRS 心跳检查 | 本轮结果：noop` 后被 duplicate suppression 压掉。
- 现有文档去重：
  - `feishu_direct_cron_management_tool_unavailable_internal_state_exposed.md` 覆盖 direct 任务管理工具不可用或未暴露；本轮 `cron_job list` 与 `portfolio unwatch` 均有真实工具调用，问题不是工具未暴露。
  - `feishu_actor_scope_cron_portfolio_empty.md` 覆盖权威 Cron / portfolio 数据被读成空；本轮 15:43 曾正确列出 `j_9ee85d42`，后续是取消意图与 portfolio / heartbeat 作用域未关联。
  - `feishu_direct_nonstandard_ticker_guess_for_trade_advice.md` 覆盖非标准 ticker 下的交易建议实体确认；本轮不是交易建议，而是用户关注 / 心跳状态变更未完成。

## 端到端链路

1. Feishu direct 用户围绕 `CBRS` 要求取消画像、心跳任务和关注。
2. assistant 在 15:43 已通过 `cron_job list` 确认 `Cerebras IPO与业务进展心跳监控` 仍启用，且任务 ID 为 `j_9ee85d42`。
3. 用户 17:57 再次要求取消 `cbrs` 关注。
4. runner 只调用 `portfolio unwatch CBRS`，没有把该意图关联到仍启用的 CBRS heartbeat 任务。
5. assistant final 反而告诉用户 CBRS 不在关注列表中，并要求用户确认是否指 Cerebras。
6. 同一 heartbeat job 在 18:00、18:30、19:00 继续执行。

## 期望效果

- 当用户在同一会话里刚讨论过某 ticker / 公司对应的心跳任务，后续说“取消关注”“别关注了”“取消这只”时，应结合最近已确认的 `cron_job` / portfolio / watchlist 真相源。
- 若存在同 ticker 的 enabled heartbeat 任务，应明确提示“已找到对应心跳任务，是否删除/停用”，或在用户已经明确确认后直接执行安全删除流程。
- 最终回复不能在同一会话已确认任务存在后，又说该 ticker 不在关注列表或要求重新确认同一实体。

## 当前实现效果

- 直聊链路正常收口，没有空回复或原始错误外泄。
- 但业务动作没有完成：CBRS 相关 heartbeat 任务仍继续运行。
- final 给出的状态判断与同一会话 15:43 的 `cron_job list` 结果冲突，用户会误以为 CBRS 已无可取消关注项。

## 用户影响

- 这是功能性 bug：用户明确要求取消关注 / 心跳相关对象，但系统没有完成对应的任务治理动作。
- 影响范围目前证据覆盖单个 Feishu direct actor 和一个已确认 enabled heartbeat job；未见跨用户错投、数据破坏或全局 scheduler 停摆，因此定级为 `P2` 而不是 `P1`。
- 该问题会导致用户继续收到或继续消耗同一标的的监控运行，即使用户认为已经取消。

## 根因判断

- 初步判断取消类意图没有统一合并 portfolio watchlist、company profile 与 cron heartbeat 三个状态源。
- `portfolio unwatch CBRS` 的工具结果被直接当成最终事实，但 answer 阶段没有检查当前会话刚确认过的 `cron_job` enabled 任务，也没有把 `CBRS` 与 `Cerebras IPO与业务进展心跳监控` 做稳定别名关联。
- 这属于关注 / 监控治理链路的状态源消费问题，不是模型单纯语气或格式质量问题。

## 下一步建议

- 为 direct 取消类意图增加统一解析：`取消关注 <ticker>` 应同时查询 portfolio watchlist、company profile 和同 ticker / alias 的 enabled cron / heartbeat。
- answer 阶段应优先消费同轮和近期 `cron_job list` 的已验证任务状态；若同一实体存在 enabled heartbeat，不得仅凭 portfolio unwatch 结果说“无需取消”。
- 增加回归样本：用户先查询并看到 `CBRS` heartbeat enabled，再说“取消对 cbrs 的关注”，最终回复必须返回任务删除确认或删除确认请求，且不得声称 `CBRS` 不存在。
