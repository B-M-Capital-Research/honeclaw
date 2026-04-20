# Bug: Feishu 直聊处理中遭遇 runtime 重启风暴后只留下 placeholder，最终回复被整轮吞掉

- **发现时间**: 2026-04-20 18:10 CST
- **Bug Type**: System Error
- **严重等级**: P1
- **状态**: Fixed
- **证据来源**:
  - 最近一小时真实会话：`data/sessions.sqlite3` -> `session_messages`
    - `session_id=Actor_feishu__direct__ou_5fa7fc023b9aa2a550a3568c8ffc4d7cdc`
    - `2026-04-20T17:35:33.240478+08:00` 用户发送：`分析一下SNDK`
    - 截至本轮巡检时，该 session 在这条 user turn 之后没有新的 assistant 落库；最新 assistant 仍停留在上一轮 `2026-04-20T16:55:48.792364+08:00` 的 DELL/HPE/CRWV 对比回答
  - 最近一小时会话索引：`data/sessions.sqlite3` -> `sessions`
    - `session_id=Actor_feishu__direct__ou_5fa7fc023b9aa2a550a3568c8ffc4d7cdc`
    - `updated_at=2026-04-20T17:35:33.240479+08:00`
    - `last_message_at=2026-04-20T17:35:33.240478+08:00`
    - `last_message_role=user`
    - `last_message_preview=分析一下SNDK`
    - 说明最近一小时这轮请求最终没有任何 assistant 消息落库，也没有失败文案改写最后一条消息
  - 最近一小时运行日志：`data/runtime/logs/web.log`
    - `2026-04-20 17:35:31.906` `step=message.accepted ... text_chars=8`
    - `2026-04-20 17:35:33.229` `step=reply.placeholder ... detail=sent`
    - `2026-04-20 17:35:33.243` `recv ... input.preview="分析一下SNDK"`
    - `2026-04-20 17:35:33.382` `step=agent.run ... detail=start`
    - `2026-04-20 17:36:04.807` 记录一次本地 `rg --files company_profiles | rg 'sndk|sandisk|storage|wdc'` 工具失败，但随后又继续执行 `hone/data_fetch` 与多轮 `Searching the Web`
    - 到 `2026-04-20 17:36:21.293` 为止仍可见 search 工具活动，但之后再也没有对应的 `session.persist_assistant`、`done user=... success=...`、`handler.session_run ... completed`、`reply.send` 或任何用户态失败提示
    - 紧接着日志从 `2026-04-20 17:37:15` 开始反复刷出 `⏰ 定时任务调度器启动` 与 `调度事件处理器已启动（渠道: imessage）`
    - `2026-04-20 17:39:43.882` 到 `17:39:46.160` 又出现完整的新启动序列：`Hone Web API 管理端已启动，端口 8077`、`🚀 Hone Feishu 渠道 启动`、`🚀 Hone Telegram Bot 启动`、各渠道 scheduler 重新启动
  - 最近一小时运行日志：`data/runtime/logs/backend_release_restart.log`
    - `2026-04-20 17:37:42.930` 起连续记录 `bundled runtime previous managed children stopped`
    - 随后在 `17:37:42-17:38:55` 间高频重复出现 `preflight locks passed -> previous managed children stopped -> starting embedded web server -> calling hone_web_api::start_server`
    - 同一窗口明确存在 bundled runtime restart storm，而不是单次正常重启
  - 相关历史缺陷：
    - [`feishu_direct_answer_idle_timeout_no_reply.md`](./feishu_direct_answer_idle_timeout_no_reply.md) 已记录旧的 answer idle timeout 无回复，并已标记 `Fixed`
    - [`feishu_direct_placeholder_without_agent_run.md`](./feishu_direct_placeholder_without_agent_run.md) 已记录“placeholder 发出但未真正进入主链路”的旧缺陷，并已标记 `Fixed`
    - [`desktop_release_runtime_supervision_gap.md`](./desktop_release_runtime_supervision_gap.md) 已记录更早的 release runtime 托管不稳问题，并已标记 `Fixed`

## 端到端链路

1. Feishu 用户在直聊里发送 `分析一下SNDK`。
2. 系统正常接受消息、发送 placeholder，并实际进入 `agent.run`。
3. search 阶段已经执行 `skill_tool`、`data_fetch`、`web_search`，说明这不是“入口未启动”或“消息没进主链路”。
4. 但在 answer 或后续收口出现前，bundled runtime 从 `17:37` 起进入连续重启风暴，日志开始反复刷新的 scheduler / web server / channel startup 序列。
5. in-flight 的 Feishu 直聊因此既没有最终 assistant，也没有受控失败文案；用户侧只能看到 placeholder 后一直等不到结果。

## 期望效果

- Feishu 直聊在已经发送 placeholder 且进入 `agent.run` 后，即使 runtime 被替换、重启或 supervisor 发生抖动，也应向用户补发明确失败提示。
- runtime 重启不应无声吞掉飞行中的 direct session；至少应留下 `handler.session_run completed success=false`、`reply.send` 或 assistant 失败文案中的一种可观测收口。
- bundled runtime 若出现异常重启风暴，应被快速熔断或限制，不能持续中断正在处理中的用户请求。

## 当前实现效果

- 真实会话已经证明：本轮不是“placeholder 后从未进入主链路”，因为 `agent.run` 与后续工具执行日志都存在。
- 也不是已知的 answer idle timeout 旧形态，因为本轮没有看到 `session/prompt idle timeout`、`failed ... timeout` 或统一超时兜底文案。
- 当前最近一小时的真实坏态是：placeholder 已送达、工具已开始跑、随后 runtime 进入 restart storm，最终这轮用户问题既没有 assistant 成功答复，也没有失败收口。
- 这说明用户主功能链路被中途截断，而现有 direct failure fallback 并没有覆盖“进程级重启 / runtime 被替换”这类中断面。

## 用户影响

- 这是功能性缺陷，不是回答质量问题。用户明确发起的 `SNDK` 分析任务没有完成，也没有收到失败提示。
- 之所以定级为 `P1`，是因为问题发生在 Feishu 直聊主链路，而且会给用户制造“系统正在思考但永远不回”的直接故障感知。
- 之所以不是 `P0`，是因为当前证据仍集中在最近一小时的单条真实会话，尚不能证明所有直聊都会被同一重启风暴同时吞掉。

## 根因判断

- 当前更接近新的独立根因，而不是旧缺陷回潮：
  - 不是 `feishu_direct_placeholder_without_agent_run`，因为本轮已经进入 `agent.run` 并执行了多次工具调用。
  - 不是 `feishu_direct_answer_idle_timeout_no_reply`，因为没有 timeout 错误，也没有统一超时文案。
  - 更像是 bundled runtime / release runtime 在 `17:37-17:39` 进入连续重启风暴，直接切断了正在处理中的 Feishu direct session。
- `backend_release_restart.log` 里连续的 `previous managed children stopped` 与 `start_server` 循环，说明根因更靠近 runtime supervision / restart orchestration，而不是单次工具失败本身。
- 本轮 `rg company_profiles` 的 tool failure 只是前置噪声；因为它之后链路仍继续执行 `data_fetch` 与 `Searching the Web`，不能解释“为何整轮既无成功也无失败收口”。

## 下一步建议

- 优先排查 bundled runtime 在 `17:37-17:39` 触发连续 restart storm 的上游原因，确认是谁在反复调用 restart / connect backend。
- 为 direct session 增加“进程级中断”兜底：若 runtime 在会话处理中被替换，应补发统一失败文案并落库 assistant 失败消息。
- 把 `handler.session_run` 与 runtime 重启事件关联起来，确保重启前后能定位哪些 in-flight sessions 被中断、是否已补发失败文案。
- 下一轮巡检继续重点复核 Feishu direct 最近一小时会话，确认这是否已扩散到其它用户或其它新问题，而不只是 `SNDK` 单样本。
