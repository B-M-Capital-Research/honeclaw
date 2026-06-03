# Bug: Feishu direct actor 读取 Cron 与持仓作用域为空，导致任务和投资上下文丢失

- **发现时间**: 2026-06-03 23:02 CST
- **Bug Type**: System Error
- **严重等级**: P1
- **状态**: New
- **GitHub Issue**: [#49](https://github.com/B-M-Capital-Research/honeclaw/issues/49)

## 证据来源

- `data/sessions.sqlite3`
  - 巡检时间窗：2026-06-03 19:02-23:02 CST。
  - `session_messages` 共有 21 个 user turn 与 22 个 assistant 记录，Feishu direct 最新会话均有 assistant 收口；多出的 assistant 是 daily-limit final/text 双记录，不构成重复回复缺陷。
  - assistant final 污染扫描未命中空回复、`hone-mcp binary not found`、本机绝对路径、`data/agent-sandboxes`、raw tool 字段、`reasoning_content`、`<think>`、provider 原始错误、`HTTP 400`、`Param Incorrect`、`Resource temporarily unavailable`、`quota exhausted`、panic 或 `index out of bounds`。
  - `session_id=Actor_feishu__direct__ou_5f85509d35510291f93cd79a3b1c9eebf3` 在 2026-06-03 21:08 CST 收到用户追问盘前 Cron 为何未执行。
  - 2026-06-03 21:10 CST assistant final 明确反馈当前定时任务列表返回 `jobs=[]`，并指出原有 `20:00` / `07:00` 任务在当前调度器视角下都不在任务表里。
  - 2026-06-03 21:13 CST assistant final 声称已重建两条 workday 常规任务，但同轮又反馈当前持仓工具返回 `holdings=[]`、`watchlist=[]`，即同一 actor 的持仓和关注列表也读成空。
  - 2026-06-03 21:21 CST assistant final 显示用户被迫重新提交并落库 13 条持仓；关注列表仍为空。
  - 2026-06-03 21:28 CST assistant final 声称已补建一次性补跑任务 `j_a9e14511`，计划 21:29 执行；2026-06-03 21:31 CST assistant final 又反馈该 once 任务到点后仍没有生成成功运行记录，`last_run_at=null`。
- `data/sessions.sqlite3` -> `cron_job_runs`
  - 2026-06-03 19:02-23:02 CST 没有新增 `cron_job_runs`。
  - 全库最近 `executed_at` 仍停在 `2026-06-01T00:26:00.908925+08:00`，最新三条普通 scheduler row 仍是 `AAOI / RKLB / TEM 每日动态监控` 的 `running + pending`。
- `data/runtime/logs/acp-events.log`
  - 同窗可见 Feishu direct runner tool / chunk / `stopReason=end_turn`，说明直聊生成链路仍可收口。
  - 未见新的用户可见 `hone-mcp binary not found` 或 provider 原始失败；异常集中在 actor 业务数据读取与 scheduler due-run 链路。
- 最近四小时非文档代码提交：无。

## 端到端链路

1. Feishu 用户已有常规定时任务和持仓/关注投资上下文。
2. 用户在 2026-06-03 21:08 CST 反馈盘前 Cron 又未执行。
3. direct assistant 查询后发现当前 actor 视角下 Cron 任务列表为空，而不是单条任务执行失败。
4. 用户同意重建任务后，assistant 又发现同一 actor 的持仓与关注列表也为空。
5. 用户被迫手动重建持仓；随后补建的 once 任务仍未进入 `cron_job_runs` 执行台账。

## 期望效果

- Feishu direct 与 scheduler 应使用同一稳定 actor 身份和持久化作用域读取 Cron、portfolio 与 watchlist。
- 已存在的用户定时任务和持仓上下文不应在同一用户会话中读成空。
- 当任务或持仓真实不存在时，应能区分“从未创建”和“当前作用域/后端读取异常”，并给出可审计错误，而不是让用户重建数据后仍无法补跑。

## 当前实现效果

- 直聊主回复可以正常生成和投递，但同一 actor 的任务表、持仓表、关注表在工具读取阶段表现为空。
- 用户的定时任务交付被阻断：20:00 常规任务未执行，21:29 once 补跑也没有生成 run 台账。
- 用户投资上下文被迫重建：原持仓/关注数据在当前工具链下不可见，后续定时简报若使用空上下文会失真。

## 用户影响

- 这是功能性缺陷，不是输出质量问题。
- 用户已配置的 Cron 与投资上下文不可见，会导致定时投研任务漏执行、补跑失败、组合简报缺少真实持仓，且用户需要手动重建关键投资数据。
- 定级为 `P1`：影响持久化业务数据正确性和 scheduler 核心交付链路；但本窗直聊仍能正常收口，未见跨用户错投或数据破坏扩散证据，因此不是 `P0`。

## 根因判断

- 初步判断是 Feishu direct / scheduler / portfolio 工具之间的 actor identity、channel target、cloud/local 存储后端或作用域解析不一致，导致同一用户的 Cron 与 portfolio 读取落到空作用域。
- 该问题不同于 `feishu_scheduler_no_runs_after_midnight.md`：旧 P1 主要是任务仍存在但 scheduler loop 不再产生 run；本轮新证据显示任务列表本身读成空，且 portfolio/watchlist 也同时为空。
- 该问题也不同于 `sessions_sqlite_mirror_stalled_after_successful_direct_replies.md`：本轮 `session_messages` 仍在更新，异常不在会话镜像停滞，而在业务数据读取与 scheduler due-run。

## 下一步建议

- 优先核对 Feishu direct actor 的 canonical identity、Cron job owner、portfolio owner 与 cloud/local 后端选择是否一致。
- 增加端到端回归：同一 Feishu actor 创建 Cron 与 portfolio 后，后续 direct 查询、scheduler due scan 与 once 补跑都必须读到同一份数据。
- 在 Cron / portfolio 工具返回空列表时增加可观测性：记录 actor id、channel scope、backend、storage source 与查询条件，避免把作用域错读伪装成“用户没有数据”。
