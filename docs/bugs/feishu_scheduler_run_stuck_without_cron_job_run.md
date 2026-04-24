# Bug: Feishu 定时任务进入执行后可长期卡住，既无最终回复也不写 `cron_job_runs`

- **发现时间**: 2026-04-24 09:03 CST
- **Bug Type**: System Error
- **严重等级**: P1
- **状态**: New
- **证据来源**:
  - 2026-04-24 08:30-09:02 最新真实会话与消息落库：`data/sessions.sqlite3` -> `sessions` / `session_messages`
    - `session_id=Actor_feishu__direct__ou_5fe40dc70caa78ad6cb0185c21b53c4732`
      - `2026-04-24T08:30:59.933045+08:00` 导入 `[定时任务触发] 任务名称：每日有色化工标的新闻追踪`
      - 到 `2026-04-24 09:02:49 CST` 复查时，`sessions.last_message_role` 仍是 `user`，`last_message_at=2026-04-24T08:30:59`
    - `session_id=Actor_feishu__direct__ou_5f0a88f4c2105e8388aa2a63ae847f7f28`
      - `2026-04-24T08:30:59.933622+08:00` 导入 `[定时任务触发] 任务名称：创新药持仓每日动态推送`
      - 到 `2026-04-24 09:02:49 CST` 复查时仍只有该条 user turn，没有 assistant 落库
    - `session_id=Actor_feishu__direct__ou_5f3f69c84593eccd71142ed767a885f595`
      - `2026-04-24T08:30:59.956666+08:00` 导入 `[定时任务触发] 任务名称：Hone_AI_Morning_Briefing`
      - 到 `2026-04-24 09:02:49 CST` 复查时仍只有该条 user turn，没有 assistant 落库
  - 2026-04-24 08:30-08:31 最新运行日志：`data/runtime/logs/sidecar.log`
    - `每日有色化工标的新闻追踪`
      - `2026-04-24 08:30:59.947` `step=agent.prepare ... detail=restore_context + build_prompt + create_runner`
      - `2026-04-24 08:31:00.136` `step=agent.run ... detail=start`
      - `2026-04-24 08:31:25.814` 起连续进入 `Tool: hone/data_fetch`，随后继续触发 `Tool: hone/web_search`
      - 到本轮巡检结束，没有出现同 session 的 `step=session.persist_assistant`、`done user=`、`failed user=`、`step=reply.send`
    - `Hone_AI_Morning_Briefing`
      - `2026-04-24 08:30:59.959` `step=agent.prepare ... detail=restore_context + build_prompt + create_runner`
      - `2026-04-24 08:31:00.139` `step=agent.run ... detail=start`
      - `2026-04-24 08:31:35.113` 起进入 `Tool: hone/skill_tool`、`Tool: hone/web_search`、`Tool: hone/data_fetch`
      - 到本轮巡检结束，同样没有看到 `session.persist_assistant`、`done user=`、`failed user=`、`reply.send`
    - `创新药持仓每日动态推送`
      - 同一时间窗已有 `recv` 入站日志，但到 `2026-04-24 09:02` 没有对应 `cron_job_runs` 新记录，也没有 assistant 落库
  - 2026-04-24 09:02 最新调度台账：`data/sessions.sqlite3` -> `cron_job_runs`
    - 对三个受影响 actor_user_id 查询最近记录时，最新 run 仍停留在 2026-04-23：
      - `ou_e40dc70caa78ad6cb0185c21b53c4732` 的 `每日有色化工标的新闻追踪` 最新仍是 `run_id=4890`、`executed_at=2026-04-23T08:32:36`
      - `ou_0a88f4c2105e8388aa2a63ae847f7f28` 的 `创新药持仓每日动态推送` 最新仍是 `run_id=4891`、`executed_at=2026-04-23T08:32:42`
      - `ou_3f69c84593eccd71142ed767a885f595` 的 `Hone_AI_Morning_Briefing` 最新仍是 `run_id=4892`、`executed_at=2026-04-23T08:32:46`
    - 说明这不是“已失败但写错状态”，而是本轮 2026-04-24 08:30 触发压根没有写入新的 `cron_job_runs`
  - 同窗口对照样本：`data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=5455` `美股AI产业链盘后报告` 在 `2026-04-24T08:32:15` 已 `completed + sent`
    - `run_id=5456` `A股盘前高景气产业链推演` 在 `2026-04-24T08:46:02` 已 `completed + sent`
    - `run_id=5467-5469` 的 09:00-09:01 直达定时任务也都正常 `completed + sent`
    - 说明 08:30-09:01 整体调度器、Feishu 出站和会话落库并未全局停摆，故障集中在部分 scheduler run 卡死且未落账
  - 已检索的相关缺陷文档：
    - `feishu_scheduler_send_failed_http_400_after_generation.md`
    - `feishu_scheduler_tenant_access_token_request_failure.md`
    - `feishu_send_message_request_transport_failure.md`
    - `feishu_scheduler_empty_reply_false_success.md`
    - 它们都要求至少存在 `cron_job_runs` 或 assistant 正文；本轮坏态是“执行已开始，但最终回复与调度台账都缺失”，属于新的独立链路异常

## 端到端链路

1. Feishu scheduler 到点触发，把任务正文以 `[定时任务触发]` user turn 注入直达会话。
2. 会话层已经进入 `agent.prepare`、`agent.run`，并继续调用 `data_fetch` / `web_search` / `skill_tool`。
3. 运行在工具阶段后长时间停滞，没有进入用户可见的完成或失败收口。
4. `session_messages` 没有 assistant 最终消息，`cron_job_runs` 也没有对应的新 run 记录。
5. 用户实际收不到日报，运维台账也看不到“本轮发生了什么”。

## 期望效果

- Feishu 定时任务一旦进入 `agent.run`，最终应收口为二选一：
  - 正常生成 assistant 正文，并写入 `cron_job_runs completed/sent`
  - 明确失败并写入 `cron_job_runs execution_failed/...`，同时给会话留下可见失败结果或至少失败痕迹
- 调度台账不能缺失本轮 run；否则人工巡检无法区分“未触发”“执行中”“卡死”“已失败”。

## 当前实现效果

- 本轮 08:30 的三个 Feishu 定时任务都已成功注入会话，并至少部分进入工具执行。
- 到触发后 30 分钟以上，三条会话仍停在 user turn，没有 assistant 最终回复。
- 同时 `cron_job_runs` 完全没有这三条任务的 2026-04-24 新 run 记录。
- 这意味着当前链路存在“运行已开始但未完成、未失败、未落账”的悬挂态。

## 用户影响

- 这是功能性缺陷，不是回答质量问题。用户预期收到晨报/持仓日报/行业新闻追踪，但本轮根本没有收到结果。
- 之所以定级为 `P1`，是因为问题直接影响 Feishu scheduler 的核心交付链路，而且台账缺失会同时阻断运维定位与补发判断。
- 这不是 `P3`，因为损害不只是“内容浅或格式差”，而是任务未完成且无失败结论。

## 根因判断

- 从现象看，问题不在“调度未触发”，而在 trigger 注入会话后 run 卡在中途。
- 从现象看，问题也不同于已有的 Feishu 发送失败或 token 请求失败，因为当前没有 assistant 正文，也没有 `cron_job_runs`。
- 更接近的根因方向是：
  - scheduler run 在 agent 执行阶段悬挂，没有走到统一的完成/失败落账逻辑
  - 或落账写入发生在收尾阶段，但当前收尾没有超时/崩溃兜底，导致 run 消失在台账外

## 下一步建议

- 先排查 scheduler run 的“创建台账时机”是否过晚；应尽量在触发后立即写一条进行中的 run，避免执行中断时完全无账可查。
- 为 scheduler 运行中的长时间无进展状态补 watchdog 或超时收口，至少写入 `execution_failed` 与错误摘要。
- 后续巡检继续重点关注：
  - 同类会话是否仍表现为 `last_message_role=user`
  - `sidecar.log` 是否只有 `agent.run` / 工具调用而没有 `done` / `failed`
  - `cron_job_runs` 是否继续缺失对应 run 记录
