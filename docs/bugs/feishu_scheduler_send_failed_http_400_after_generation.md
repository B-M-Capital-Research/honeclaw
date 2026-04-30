# Bug: Feishu 直达定时任务生成完成后仍在发送阶段落成 `HTTP 400 Bad Request`

- **发现时间**: 2026-04-16 22:08 CST
- **Bug Type**: System Error
- **严重等级**: P1
- **状态**: Fixed
- **GitHub Issue**: [#25](https://github.com/B-M-Capital-Research/honeclaw/issues/25)
- **证据来源**:
  - 2026-04-30 22:33 最近一小时最新样本：
    - `data/runtime/logs/acp-events.log`
      - `22:33:03.251`、`22:33:08.142` 连续两次记录 `channel sink failed, falling back to log: feishu send HTTP 400 Bad Request`，返回体明确 `code=99992361`、`msg="open_id cross app"`；紧接着只剩 `[dryrun sink]` 的 `RKLB 跨过 +6% 档` 事件卡片
      - `22:33:11.720`、`22:33:16.559` 同一窗口又连续两次命中同样的 `open_id cross app` 返回体，对应 `TEM 跨过 +6% 档` 事件卡片；四次失败都带新的 Feishu `log_id`
      - 同窗还持续有其它 `sink delivered` 样本，说明不是 Feishu 出站全局不可用，而是同一类事件 sink 目标再次稳定触发 `open_id cross app`
    - 这说明 `2026-04-28` 记录为止血的 current-app open_id fallback 还没有覆盖当前生产事件链路；到 `2026-04-30 22:33` 为止，本单已经从 `Later` 回到活跃复现，应恢复为 `New`
  - 2026-04-28 19:10 最近一小时最新样本：
    - `data/runtime/logs/web.log.2026-04-28`
      - `19:10:02.903` 再次记录 `channel sink failed, falling back to log: feishu send HTTP 400 Bad Request`，返回体仍是 `code=99992361`、`msg="open_id cross app"`，并带新的 `log_id=20260428191002FFEF81CF094F1F3A5E68`
      - `19:10:20.374` 与 `19:10:43.402` 同一窗口又连续两次命中同样的 `open_id cross app` 返回体，对应新 `log_id=202604281910201AA21839C7340E39087F` 与 `20260428191043969C752591093B37588E`
      - 三次失败后都只剩 `[dryrun sink] 今日全球要闻 · 6/7 条 · 2026-04-28`，说明故障点仍是“已生成 digest 标题/正文，但最终 Feishu sink send API 被拒”
      - 同窗 `19:10:02.921`、`19:10:20.391`、`19:10:43.424` 仍分别记录 `global digest sent`，说明上游 global digest 调度本身没有停摆，坏态继续集中在最终 Feishu 发送链路
  - 2026-04-28 08:00 最近一小时最新样本：
    - `data/sessions.sqlite3` -> `cron_job_runs`
      - `run_id=8507`，`job_id=j_6547def8`，`job_name=每日美股收盘与持仓早报`，`executed_at=2026-04-28T08:02:34.000419+08:00`
      - 本轮再次落成 `execution_status=completed`、`message_send_status=sent`、`delivered=1`，说明同一 `08:00` 窗口里正常日报链路仍可送达，故障并非 Feishu 出站全局不可用
    - `data/runtime/logs/sidecar.log`
      - `2026-04-28 08:00:50.373` 继续记录 `channel sink failed, falling back to log: feishu send HTTP 400 Bad Request`，返回体仍是 `code=99992361`、`msg="open_id cross app"`，并带新的 `log_id=202604280800507B74587E8735DEF5DAB5`
      - 紧接着同一日志打印 `[dryrun sink] {"zh_cn":{"title":"【要闻】 $NVDA · 📄 SEC 8-K"...}}`，说明失败点仍是“已生成卡片正文，但最终 Feishu send API 被拒”
      - `2026-04-28 08:00:50.377` 同窗还有其它 `sink delivered` 样本，进一步证明这不是全局 sink/网络中断，而是同一类目标/标识域仍会稳定触发 `open_id cross app`
  - 2026-04-21 21:02 最近一小时最新样本：
    - `data/sessions.sqlite3` -> `cron_job_runs`
      - `run_id=4136`，`job_id=j_f02dfce5`，`job_name=OWALERT_PreMarket`，`executed_at=2026-04-21T21:02:48.696425+08:00`
      - 再次落成 `execution_status=completed`、`message_send_status=send_failed`、`delivered=0`、`should_deliver=1`
      - `error_message` 继续明确返回 `code=99992361`、`msg="open_id cross app"`，说明 Feishu 最终投递阶段仍被同一绑定域错误拒绝
      - `response_preview` 已保留盘前扫描正文开头，说明本轮仍是生成完成后发送失败；这次正文还以 `Context compacted` 开头，叠加了压缩标记外泄问题，但决定性未送达原因仍是 Feishu 400
    - `data/sessions.sqlite3` -> `session_messages`
      - `session_id=Actor_feishu__direct__ou_5f3f69c84593eccd71142ed767a885f595`
      - `2026-04-21T21:02:47.997793+08:00` assistant 已写入本轮最终正文，与 `response_preview` 对齐
      - 说明 scheduler 注入、模型生成、会话持久化都已完成，故障继续停留在最终 Feishu 出站阶段
  - 2026-04-20 21:31 最近一小时最新样本：
    - `data/sessions.sqlite3` -> `cron_job_runs`
      - `run_id=3633`，`job_id=j_dac3b571`，`job_name=Oil_Price_Monitor_Premarket`，`executed_at=2026-04-20T21:31:24.235993+08:00`
      - 再次落成 `execution_status=completed`、`message_send_status=send_failed`、`delivered=0`、`should_deliver=1`
      - `error_message` 继续明确返回 `code=99992361`、`msg="open_id cross app"`，并附上新的 Feishu `log_id=2026042021312424437E9F8568C4DA7107`
      - `response_preview` 已保留完整盘前油价播报开头，说明这次不是 answer 半成品，而是 scheduler 已拿到完整播报后仍在 Feishu 最终投递阶段失败
    - `data/sessions.sqlite3` -> `session_messages`
      - `session_id=Actor_feishu__direct__ou_5f3f69c84593eccd71142ed767a885f595`
      - `2026-04-20T21:31:23.743374+08:00` assistant 已写入本轮 `Oil_Price_Monitor_Premarket` 可见正文，长度与 `response_preview` 对齐
      - 说明 scheduler 注入、会话持久化与“拿到待发送正文”都已完成，真正失败点仍停留在 Feishu 最终投递阶段
    - `data/runtime/logs/web.log`
      - `2026-04-20 21:31:24.234` 记录 `[Feishu] 定时任务投递失败: job=Oil_Price_Monitor_Premarket ... HTTP 400 Bad Request - {"code":99992361,"msg":"open_id cross app",...}`
      - 这说明故障没有停留在 `21:01` 的盘前扫描；同一目标上的油价盘前播报在同一小时窗里继续复现相同返回体
  - 2026-04-20 21:01 最近一小时最新样本：
    - `data/sessions.sqlite3` -> `cron_job_runs`
      - `run_id=3619`，`job_id=j_f02dfce5`，`job_name=OWALERT_PreMarket`，`executed_at=2026-04-20T21:01:26.218056+08:00`
      - 再次落成 `execution_status=completed`、`message_send_status=send_failed`、`delivered=0`、`should_deliver=1`
      - `error_message` 继续明确返回 `code=99992361`、`msg="open_id cross app"`，并附上新的 Feishu `log_id=202604202101260FE9A88A408E4AF1BA56`
      - `response_preview` 已保留盘前扫描正文开头，说明这次不是 answer 空结果，而是 scheduler 已拿到完整播报后仍在 Feishu 最终投递阶段失败
    - `data/sessions.sqlite3` -> `session_messages`
      - `session_id=Actor_feishu__direct__ou_5f3f69c84593eccd71142ed767a885f595`
      - `2026-04-20T21:00:59.985150+08:00` assistant 已写入本轮 `OWALERT_PreMarket` 可见正文，长度与 `response_preview` 对齐
      - 说明 scheduler 注入、会话持久化与“拿到待发送正文”都已完成，真正失败点仍停留在 Feishu 最终投递阶段
    - `data/runtime/logs/web.log`
      - `2026-04-20 21:01:26.216` 记录 `[Feishu] 定时任务投递失败: job=OWALERT_PreMarket ... HTTP 400 Bad Request - {"code":99992361,"msg":"open_id cross app",...}`
      - 这说明故障已从盘前/盘后/早报/财报提醒继续扩散到同一目标上的盘前扫描任务，而不是只停留在某一个提醒模板
  - 2026-04-20 20:00 最近一小时最新样本：
    - `data/sessions.sqlite3` -> `cron_job_runs`
      - `run_id=3590`，`job_id=j_98f3899c`，`job_name=GEV earnings reminder`，`executed_at=2026-04-20T20:00:38.780847+08:00`
      - 再次落成 `execution_status=completed`、`message_send_status=send_failed`、`delivered=0`、`should_deliver=1`
      - `detail_json.receive_id=ou_3f69c84593eccd71142ed767a885f595`，仍与历史失败目标一致
      - `error_message` 再次明确返回 `code=99992361`、`msg="open_id cross app"`，并附上新的 Feishu `log_id=20260420200038769EC0904EBA08FA7907`
      - `response_preview` 仅保留一段 72 字计划句，说明这轮不仅发送失败，连 scheduler 拿到的可见正文都已经退化成半成品
    - `data/sessions.sqlite3` -> `session_messages`
      - `session_id=Actor_feishu__direct__ou_5f3f69c84593eccd71142ed767a885f595`
      - `2026-04-20T20:00:36.997863+08:00` assistant 已写入本轮 `GEV earnings reminder` 的可见回复，正文与 `response_preview` 对齐
      - 说明 scheduler 注入、会话持久化与“拿到待发送正文”都已完成，真正失败点仍停留在 Feishu 最终投递阶段
    - `data/runtime/logs/web.log`
      - `2026-04-20 20:00:38.779` 记录 `[Feishu] 定时任务投递失败: job=GEV earnings reminder ... HTTP 400 Bad Request - {"code":99992361,"msg":"open_id cross app",...}`
      - 这说明故障已经从盘前/盘后/早报任务进一步扩散到财报提醒类任务，而不是局限在某几个固定模板
  - 2026-04-20 08:33 最近一小时最新样本：
    - `data/sessions.sqlite3` -> `cron_job_runs`
      - `run_id=3348`，`job_id=j_248f0f3c`，`job_name=Hone_AI_Morning_Briefing`，`executed_at=2026-04-20T08:33:04.280905+08:00`
      - 再次落成 `execution_status=completed`、`message_send_status=send_failed`、`delivered=0`、`should_deliver=1`
      - `response_preview` 保留了完整早报正文开头，说明这轮仍是模型执行成功、会话持久化成功，但最终 Feishu 投递失败
      - `error_message` 再次明确返回 `code=99992361`、`msg="open_id cross app"`，并给出 Feishu troubleshooting `log_id`
    - `data/runtime/logs/sidecar.log`
      - `2026-04-20 08:33:04.280` 记录 `[Feishu] 定时任务投递失败: job=Hone_AI_Morning_Briefing ... HTTP 400 Bad Request - {"code":99992361,"msg":"open_id cross app",...}`
      - 这说明故障已经不只停留在油价/盘后提醒，而是扩散到同一目标上的日常早报任务
  - 2026-04-20 04:32 最近一小时最新样本：
    - `data/sessions.sqlite3` -> `cron_job_runs`
      - `run_id=3260`，`job_id=j_a6577b6f`，`job_name=OWALERT_PostMarket`，`executed_at=2026-04-20T04:32:37.532710+08:00`
      - 紧接 `04:02 Oil_Price_Monitor_Closing` 后再次落成 `execution_status=completed`、`message_send_status=send_failed`、`delivered=0`、`should_deliver=1`
      - `response_preview` 保留了完整盘后复盘正文开头，说明本轮依旧是模型执行与会话持久化成功、最终 Feishu 投递失败
      - `error_message` 与 `04:02` 样本一致，继续返回 `code=99992361`、`msg="open_id cross app"`
    - `data/sessions.sqlite3` -> `session_messages`
      - `session_id=Actor_feishu__direct__ou_5f3f69c84593eccd71142ed767a885f595`
      - `2026-04-20T04:32:36.633054+08:00` assistant 已写入本轮 `OWALERT_PostMarket` 最终播报，长度与 `response_preview` 对齐
      - 说明本轮 scheduler 注入、LLM 生成、会话落库都成功，真正缺口仍在 Feishu 出站
    - `data/runtime/logs/web.log`
      - `2026-04-20 04:32:37.531` 记录 `[Feishu] 定时任务投递失败: job=OWALERT_PostMarket ... HTTP 400 Bad Request - {"code":99992361,"msg":"open_id cross app",...}`
      - 同一 `actor_user_id / receive_id` 在 30 分钟内连续第二次命中同一返回体，说明这不是单个任务模板偶发异常
  - 2026-04-20 04:02 最近一小时最新样本：
    - `data/sessions.sqlite3` -> `cron_job_runs`
      - `run_id=3249`，`job_id=j_355ba2f1`，`job_name=Oil_Price_Monitor_Closing`，`executed_at=2026-04-20T04:02:07.830452+08:00`
      - 再次落成 `execution_status=completed`、`message_send_status=send_failed`、`delivered=0`、`should_deliver=1`
      - `response_preview` 保留了完整收盘前油价播报开头，说明本轮模型执行与调度收口都成功，故障继续停留在最终 Feishu 投递阶段
      - `error_message` 这次已不再只是裸 `HTTP 400`，而是明确返回 `code=99992361`、`msg="open_id cross app"`
    - `data/sessions.sqlite3` -> `session_messages`
      - `session_id=Actor_feishu__direct__ou_5f3f69c84593eccd71142ed767a885f595`
      - `2026-04-20T04:02:07.350233+08:00` assistant 已写入本轮 `Oil_Price_Monitor_Closing` 最终播报，且正文长度与 `response_preview` 对齐
      - 说明 scheduler 注入、LLM 生成、会话持久化全部成功，但用户侧依然没有收到真正投递
    - `data/runtime/logs/web.log`
      - `2026-04-20 04:02:07.829` 记录 `[Feishu] 定时任务投递失败: job=Oil_Price_Monitor_Closing ... HTTP 400 Bad Request - {"code":99992361,"msg":"open_id cross app",...}`
      - 这是当前已落库样本里第一次直接拿到 Feishu 返回体，根因从“泛化 400”收敛到“当前发送所用 open_id 与 app 绑定关系不一致”
  - 最近一小时调度落库：`data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=2207`，`job_id=j_dac3b571`，`job_name=Oil_Price_Monitor_Premarket`，`executed_at=2026-04-17T21:32:32.066317+08:00`
    - 同样落成 `execution_status=completed`、`message_send_status=send_failed`、`delivered=0`
    - `error_message=集成错误: Feishu send message failed: HTTP 400 Bad Request`
    - `detail_json.receive_id=ou_3f69c84593eccd71142ed767a885f595`，继续与 `actor_user_id` 对齐
    - `response_preview` 保留了完整油价播报开头，说明最近一轮真实 scheduler 窗口里模型执行与会话持久化仍然成功，故障继续停留在发送阶段
    - `run_id=1998`，`job_id=j_f02dfce5`，`job_name=OWALERT_PreMarket`，`executed_at=2026-04-16T21:04:06.271882+08:00`
    - `execution_status=completed`，`message_send_status=send_failed`，`delivered=0`
    - `error_message=集成错误: Feishu send message failed: HTTP 400 Bad Request`
    - `detail_json.receive_id=ou_3f69c84593eccd71142ed767a885f595`，已与 `actor_user_id` 对齐，说明不再是旧的 target resolution mismatch
    - `response_preview` 已保留完整盘前播报开头，说明模型输出已生成，失败发生在发送阶段
    - `run_id=2005`，`job_id=j_dac3b571`，`job_name=Oil_Price_Monitor_Premarket`，`executed_at=2026-04-16T21:33:06.730340+08:00`
    - 同样落成 `execution_status=completed`、`message_send_status=send_failed`、`delivered=0`
    - `detail_json.receive_id=ou_3f69c84593eccd71142ed767a885f595`
    - `run_id=2063`，`job_id=j_355ba2f1`，`job_name=Oil_Price_Monitor_Closing`，`executed_at=2026-04-17T04:01:50.774858+08:00`
    - 同样落成 `execution_status=completed`、`message_send_status=send_failed`、`delivered=0`
    - `error_message=集成错误: Feishu send message failed: HTTP 400 Bad Request`
    - `detail_json.receive_id=ou_3f69c84593eccd71142ed767a885f595`
    - `run_id=2068`，`job_id=j_a6577b6f`，`job_name=OWALERT_PostMarket`，`executed_at=2026-04-17T04:31:33.415283+08:00`
    - 同样落成 `execution_status=completed`、`message_send_status=send_failed`、`delivered=0`
    - `error_message=集成错误: Feishu send message failed: HTTP 400 Bad Request`
    - `detail_json.receive_id=ou_3f69c84593eccd71142ed767a885f595`
  - 最近一小时真实会话：`data/sessions.sqlite3` -> `session_messages`
    - `session_id=Actor_feishu__direct__ou_5f3f69c84593eccd71142ed767a885f595`
    - `2026-04-17T21:32:31.204674+08:00` assistant 已再次写入 `Oil_Price_Monitor_Premarket` 最终播报，但 `cron_job_runs.run_id=2207` 仍落成 `send_failed`
    - `2026-04-16T21:04:05.652096+08:00` assistant 已写入 `OWALERT_PreMarket` 最终播报
    - `2026-04-16T21:33:06.067389+08:00` assistant 已写入 `Oil_Price_Monitor_Premarket` 最终播报
    - `2026-04-17T04:01:50.132692+08:00` assistant 已写入 `Oil_Price_Monitor_Closing` 最终播报
    - `2026-04-17T04:31:32.813844+08:00` assistant 已写入 `OWALERT_PostMarket` 最终播报
    - 说明调度触发、模型执行、会话持久化都已成功，但用户侧仍未送达
  - 最近一小时运行日志：
    - `data/runtime/logs/hone-feishu.release-restart.log`
      - `2026-04-17T13:32:32.064878Z` `[Feishu] 定时任务投递失败: job=Oil_Price_Monitor_Premarket target=+8617326027390 err=集成错误: Feishu send message failed: HTTP 400 Bad Request`
      - `2026-04-16T13:04:06.270953Z` `[Feishu] 定时任务投递失败: job=OWALERT_PreMarket target=+8617326027390 err=集成错误: Feishu send message failed: HTTP 400 Bad Request`
      - `2026-04-16T13:33:06.728472Z` `[Feishu] 定时任务投递失败: job=Oil_Price_Monitor_Premarket target=+8617326027390 err=集成错误: Feishu send message failed: HTTP 400 Bad Request`
    - `data/runtime/logs/web.log`
      - `2026-04-17 21:32:32.064` 同样记录 `Oil_Price_Monitor_Premarket` 发送 400，说明 10:40 补的 direct-scheduler fallback 在最新真实窗口里还未收口
      - `2026-04-16 21:04:06.271` 同样记录 `OWALERT_PreMarket` 发送 400
      - `2026-04-16 21:33:06.728` 同样记录 `Oil_Price_Monitor_Premarket` 发送 400
      - `2026-04-17 04:01:50.773` 同样记录 `Oil_Price_Monitor_Closing` 发送 400
      - `2026-04-17 04:31:33.413` 同样记录 `OWALERT_PostMarket` 发送 400
  - 2026-04-17 08:34 最近一小时新增样本：
    - `run_id=2111`，`job_id=j_a1772833`，`job_name=Hone_AI_Morning_Briefing`，`executed_at=2026-04-17T08:34:22.570953+08:00`
    - 同样落成 `execution_status=completed`、`message_send_status=send_failed`、`delivered=0`
    - `error_message=集成错误: Feishu send message failed: HTTP 400 Bad Request`
    - `detail_json.receive_id=ou_3f69c84593eccd71142ed767a885f595`，仍与 `actor_user_id` 对齐，说明故障继续停留在发送阶段而不是 target resolution
    - `response_preview` 保留了完整早报开头，且 `session_messages` 中 `2026-04-17T08:34:21.422395+08:00` 已写入最终播报，说明会话执行成功但用户侧继续未送达
    - `data/runtime/logs/web.log` 在 `2026-04-17 08:34:22.569` 再次记录 `job=Hone_AI_Morning_Briefing` 向同一目标发送 400
  - 历史对照：
    - 同一 `channel_target=+8617326027390` 的旧问题已登记在 `docs/bugs/feishu_scheduler_target_resolution_failed.md`
    - 但本轮 `detail_json.receive_id` 已等于 `actor_user_id`，失败形态从 `target_resolution_failed` 变为 `send_failed`，属于新的独立故障阶段

## 端到端链路

1. Feishu 直达定时任务到点触发，scheduler 把任务正文注入 `Actor_feishu__direct__ou_5f3f69c84593eccd71142ed767a885f595`。
2. Multi-Agent 正常完成搜索与 Answer 阶段，assistant 最终文本已经持久化进会话。
3. `cron_job_runs` 也记录了非空 `response_preview`，说明调度层已经拿到待发送正文。
4. Feishu scheduler 在真正调用发送接口时返回 `HTTP 400 Bad Request`。
5. 本轮运行最终落成 `execution_status=completed`、`message_send_status=send_failed`、`delivered=0`，用户收不到推送。

## 期望效果

- 当 scheduler 已生成最终可见文本且 `receive_id` 与任务绑定 actor 一致时，Feishu 直达投递应成功送达用户。
- 如果 Feishu API 返回 4xx，系统应至少记录可定位的请求上下文或响应体，便于区分是内容格式、字段构造还是接收者类型错误。
- 同一用户的高价值盘前任务不应在相邻两个时间点连续“生成成功但发送失败”。

## 当前实现效果

- `2026-04-30 22:33` 的最近一小时最新样本说明，这条缺陷已经重新回到活跃态：同一窗口里 `RKLB`、`TEM` 两类事件卡片都已生成，但最终 Feishu sink 连续四次命中 `code=99992361 / open_id cross app`，用户只剩 dryrun log，看不到真实推送。
- `2026-04-28 08:00` 的真实窗口说明，这条缺陷仍在最新一小时活跃：同一时窗里普通 `每日美股收盘与持仓早报` 已成功 `completed + sent + delivered=1`，但事件推送链路仍在 `08:00:50.373` 命中 `HTTP 400 / code=99992361 / open_id cross app`，且失败后只剩 dryrun log，用户侧收不到这条已生成的卡片。
- `2026-04-21 21:02` 的 `OWALERT_PreMarket` 说明，这条缺陷在最新巡检窗口仍活跃：同一目标又一次落成 `completed + send_failed + code=99992361/open_id cross app`，用户仍收不到已经生成并落库的盘前扫描。
- `2026-04-20 21:31` 的 `Oil_Price_Monitor_Premarket` 说明，在 `21:01` 的盘前扫描失败后，同一目标的盘前油价播报又再次落成 `completed + send_failed + code=99992361/open_id cross app`。
- 这次 `response_preview` 已经是完整油价正文开头，不再叠加 `GEV earnings reminder` 那种 72 字计划句；因此最新小时窗进一步收敛出站根因仍独立存在，不能再归因于 answer 侧半成品收口。
- `2026-04-20 21:01` 的 `OWALERT_PreMarket` 再次落成 `completed + send_failed + code=99992361/open_id cross app`，并且这次 `response_preview` 已经是完整盘前扫描正文开头，不再只是 `GEV earnings reminder` 那种 72 字计划句；这说明即使 answer 侧没有再明显截断，Feishu 出站 400 仍会单独导致用户完全收不到提醒。
- `2026-04-20 20:00` 的 `GEV earnings reminder` 继续命中同一 `open_id cross app` 返回体，说明故障已经从盘前/盘后扫描、日常早报进一步扩散到财报提醒任务；当前不能再把它视为少数模板异常。
- 这轮最新样本的 `response_preview` 只有 72 字计划句，说明发送失败链路当前还会叠加 answer 侧的半成品收口；但从台账角度看，真正导致“用户完全收不到提醒”的决定性故障仍是 Feishu 出站 400。
- `2026-04-20 08:33` 的 `Hone_AI_Morning_Briefing` 继续落成 `completed + send_failed`，且错误体与 `04:02/04:32` 两轮完全一致，仍然是 `code=99992361 / open_id cross app`。
- 这说明故障已经从 `Oil_Price_Monitor_Closing`、`OWALERT_PostMarket` 扩散到同一目标上的通用早报任务；问题不是单个 job 模板、单个消息长度或单个盘前/盘后场景偶发异常。
- `2026-04-20 04:02` 的 `Oil_Price_Monitor_Closing` 与 `04:32` 的 `OWALERT_PostMarket` 在同一目标上连续两轮都落成 `completed + send_failed`，并且 Feishu 返回体都明确是 `code=99992361 / open_id cross app`。
- 这说明故障已经不只是“收盘前油价播报”单任务失败，而是同一 Feishu 直达 scheduler 发送链路在盘前之外的盘后复盘任务上也稳定复现。
- `OWALERT_PreMarket`、`Oil_Price_Monitor_Premarket`、`Oil_Price_Monitor_Closing` 与 `OWALERT_PostMarket` 在最近几个窗口连续四次失败。
- 四次失败都发生在相同用户、相同手机号目标、相同 scheduler 送达链路。
- 与前一日的 `target_resolution_failed` 不同，这一轮 `receive_id` 已解析为正确 actor，但发送接口仍直接返回 400。
- `2026-04-20 04:02` 的最新 `Oil_Price_Monitor_Closing` 样本进一步证明，这类失败并不是“消息体太长”或“单个模板 markdown 非法”那么宽泛；Feishu 已明确返回 `code=99992361 / open_id cross app`。
- 也就是说，当前链路即使拿到了正确会话与最终正文，最终投递时使用的 `open_id` 仍可能不属于正在发消息的 app 绑定域，导致整轮在 Feishu API 校验阶段被拒绝。
- 最近一小时新增样本说明故障已经从“盘前提醒”扩展到“收盘监控 / 收盘后提醒”，属于同一发送链路持续失败，而不是某一个 job 文案偶发异常。
- `2026-04-17 08:34` 的 `Hone_AI_Morning_Briefing` 新样本说明故障仍在最新小时窗活跃，并且已从盘前 / 收盘 / 盘后扩散到“日常早报”任务；受影响对象仍是同一 `receive_id` 与同一目标手机号。
- `2026-04-17 21:32` 的最新 `Oil_Price_Monitor_Premarket` 样本说明，哪怕在 10:40 已补 direct scheduler 的 standalone fallback 之后，下一轮真实窗口里同一 `receive_id` 仍会稳定落成 `completed + send_failed + HTTP 400`，所以当前不能把这条链路视为“待验证修复”，而应视为“修复尝试后仍在线复现”。

## 用户影响

- 这是功能性缺陷，不是回答质量问题。任务正文已经生成，但用户完全收不到本该送达的定时播报。
- 受影响的是用户高频依赖的盘前提醒链路，且同一目标在本小时连续两次失败，因此定级为 `P1`。
- 之所以不是 `P0`，是因为当前表现为“消息丢失”而不是“误投到错误对象”或更大范围全局不可用。

## 根因判断

- `2026-04-30 22:33` 的四连发 `open_id cross app` 说明，当前 event-engine / sink 实际发送路径仍有一段没有用到 `2026-04-28` 所说的 current-app open_id fallback，或者 fallback 命中的联系人集与真实发送对象仍不一致。
- `2026-04-28 08:00` 同窗里既有 `run_id=8507` 这种正常 `completed + sent + delivered=1` 的日报，也有 `08:00:50.373` 的 `open_id cross app` 发送失败；这进一步收敛出问题不在 Feishu token、全局网络或全部发送请求，而仍在某一类事件 sink 最终选择的 `receive_id/open_id` 标识域。
- `2026-04-21 21:02` 的 `OWALERT_PreMarket` 新样本进一步说明，问题仍不依赖某一份特定 prompt 或某一天的模板；只要命中同一目标，scheduler 最终发送到 Feishu API 时仍可能收到 `open_id cross app`。
- `2026-04-20 21:31` 的 `Oil_Price_Monitor_Premarket` 样本说明，问题不依赖某一份特定 prompt 或持仓扫描模板；即使是另一条油价播报任务，只要命中同一目标，scheduler 最终发送到 Feishu API 时仍会稳定收到 `open_id cross app`。
- `2026-04-20 21:01` 的 `OWALERT_PreMarket` 与 `20:00` 的 `GEV earnings reminder` 连续两轮都命中相同 `code=99992361 / open_id cross app`，且都指向同一 `actor_user_id`，进一步说明问题核心仍在 scheduler 最终投递时选择/复用的 Feishu 标识域，而不是某一轮 answer 内容刚好异常。
- 初步判断：旧的 direct target 解析问题并没有完全退出生产链路。`detail_json.receive_id` 虽然表面上已回到绑定 actor，但 `2026-04-20 04:02` 的 Feishu 返回体已明确指出当前发送使用的是 `open_id cross app`。
- 因此根因比“泛化的发送阶段 400”更具体：scheduler 最终调用 Feishu 发送 API 时，所选 `receive_id/open_id` 与当前 app 的绑定关系仍不一致，或者仍沿用了跨 app 域的历史标识。
- 由于同一目标在 `2026-04-16 20:03` 仍有一条 `run_id=1976` 成功送达，而 `2026-04-17 04:01` 与 `04:31` 的新样本继续失败，说明并非该用户或该目标整体不可用，更像是 scheduler 当前某类 payload 形态在发送阶段稳定触发了 400。
- 新增失败样本覆盖盘前、收盘、收盘后三种 job 名称，但都指向同一 `receive_id` 与同一 actor，会更支持“Feishu 发送请求构造/消息体校验”这一公共链路根因，而不是单个任务 prompt 内容问题。
- `08:34` 的早报任务失败进一步排除了“只在某一类油价/盘后模板文案触发 400”的可能性；更像是面向同一 Feishu 直达目标的 scheduler 发送链路在多种长文本 payload 上都可能稳定触发 400。
- `21:32` 与 `04:02` 的连续失败都发生在已经补了 `reply/update -> standalone send` 回退之后，说明问题不只是“回复链路选错 API”；当前更像是 scheduler 在直达 Feishu 目标上仍会选到跨 app 域的 `open_id` 或其等价标识。

## 下一步建议

- 优先核对 `+8617326027390` / `ou_3f69c84593eccd71142ed767a885f595` 当前 scheduler 发送时实际落下的 `receive_id_type` 与 `receive_id` 来源，确认是否仍在跨 app 域复用旧 `open_id`。
- 对 `+8617326027390` 最近成功与失败 run 的发送 payload 做差异比对，优先比较 `run_id=1976` 与 `run_id=3249/2207/1998`。
- 即便已有响应体日志，也应继续补发信分支的请求元信息，至少包含 `receive_id_type`、消息类型、正文长度、是否走 markdown/card 分支。
- 若确认只是同一发送链路的新阶段回归，应在修复后回写 `docs/bugs/README.md` 与本文件状态；修复前不要恢复为 `Fixed`。

## 当前修复进展（2026-04-17 10:40 CST）

- 本轮先按“多段 direct scheduler 发送链路不稳定”收口：
  - `bins/hone-feishu/src/outbound.rs` 现在对 `receive_id_type=open_id` 且没有 placeholder 的多段消息，不再默认把后续分段走 `reply_message`；会直接逐段 standalone send。
  - 如果 `update_message` 或 `reply_message` 仍返回 `HTTP 400`，同一分段会自动回退到 standalone send，而不是整轮直接 `send_failed`。

## 修复进展（2026-04-28 / 2026-04-30 复核）

- 已在 `crates/hone-event-engine/src/sinks/feishu.rs` 为 event-engine Feishu sink 增加 current-app open_id 解析缓存：
  - 单用户安装场景下，如果 `feishu.allow_mobiles` 或 `feishu.allow_emails` 只有一个稳定联系人，事件推送会先通过 Feishu `batch_get_id?user_id_type=open_id` 解析当前 app 绑定的 open_id。
  - 只有唯一联系人时才启用该 fallback；配置里有多个 email/mobile 或通配 `*` 时不会猜测映射，避免跨用户误投。
  - 群聊仍继续走 `chat_id`，不受 direct open_id fallback 影响。
- `crates/hone-web-api/src/lib.rs` 组装 event-engine sink 时已把 `feishu.allow_emails` / `feishu.allow_mobiles` 传入该 fallback。
- `2026-04-30 22:33` 最近一小时真实事件窗口已再次连续返回 `code=99992361 / open_id cross app`，因此本单按约定从 `Later` 调回 `New`，继续作为活跃 `P1` 跟踪；已有 GitHub Issue `#25` 继续沿用。
- `bins/hone-feishu/src/client.rs` 也已补发信失败时的响应体日志，后续再出现 400 时不再只剩裸 `Bad Request`，而会带上 Feishu body 摘要，便于继续定位 payload 差异。
- 自动化验证已通过：
  - `cargo test -p hone-feishu`
  - `cargo test -p hone-channels`
- 但 `2026-04-17 21:32` 的下一轮真实 scheduler 窗口已经再次复现 `Oil_Price_Monitor_Premarket -> completed + send_failed + HTTP 400`；说明当前修复还没有收口到生产链路，本单继续保持 `Fixing`。

## 修复进展（2026-04-30 bug-2）

- 本轮根据 GitHub Issue [#25](https://github.com/B-M-Capital-Research/honeclaw/issues/25) 复核后补齐 Feishu scheduler 仍可能遗留的 open_id 直传缺口：
  - `bins/hone-feishu/src/handler.rs` 新增 scheduler 专用解析入口；当历史任务的 `channel_target` 直接保存为 `ou_...` open_id，且当前 Feishu 配置只包含一个稳定 `allow_email` 或一个稳定 `allow_mobile` 时，会先用该联系人通过 Feishu API 重新解析 current-app-scoped open_id。
  - 多联系人、通配 `*`、email 与 mobile 同时存在等无法唯一确认收件人的配置不会猜测映射，仍保持原有目标，避免误投。
  - 已有 email / mobile 目标继续走原本的 Feishu API 解析；群聊与普通非 open_id 目标不受影响。
- 新增回归覆盖：
  - stale `ou_...` 目标在唯一 email / mobile 配置下会改走联系人解析。
  - ambiguous / wildcard / plain target 不会猜测 fallback。
- 验证：
  - `cargo test -p hone-feishu scheduler_resolution_target -- --nocapture`
  - `cargo test -p hone-feishu direct_scheduler_always_falls_through_to_api_resolution -- --nocapture`
- 当前结论：本轮只闭合 Feishu scheduler 历史 `ou_...` direct target 继续直传的本地可修缺口；`2026-04-30 22:33` event-engine 价格异动卡片四连发 `code=99992361 / open_id cross app` 仍是更新鲜证据，因此本单保持 `Fixing`，不恢复为 `Fixed`。

## 修复进展（2026-05-01 bug-2）

- 本轮根据 GitHub Issue [#25](https://github.com/B-M-Capital-Research/honeclaw/issues/25) 与 `2026-04-30 22:33` event-engine 价格异动卡片四连发 `open_id cross app` 复核后，继续收口 event-engine Feishu sink 的 current-app open_id fallback：
  - `crates/hone-event-engine/src/sinks/feishu.rs` 原先只在配置里“唯一 email 或唯一 mobile，且二者不能同时存在”时启用联系人解析；如果单用户安装同时保留同一人的 email 和 mobile，fallback 会被关闭，event-engine 仍会把 `actor.user_id` 里的历史 `ou_...` 直接作为 `open_id` 发送。
  - 现在 fallback 会把所有非空、非通配的稳定 email/mobile 一起提交给 Feishu `batch_get_id?user_id_type=open_id`；只有返回结果去重后恰好是一个 current-app open_id 时才使用，返回 0 个或多个不同用户时视为无法唯一确认，不猜测映射。
  - 群聊仍继续走 `chat_id`；没有稳定联系人或联系人解析不唯一时，保留原有 direct actor 发送逻辑，避免多用户配置误投。
- 新增/调整回归覆盖：
  - 同时配置 email + mobile 时会进入 direct fallback 候选，而不是关闭 fallback。
  - `batch_get_id` 返回同一个 user_id 的重复记录会解析为唯一 open_id。
  - `batch_get_id` 返回多个不同 user_id 时会拒绝 fallback，避免跨用户误投。
- 验证：
  - `cargo test -p hone-event-engine direct_contact --lib -- --nocapture`
  - `cargo test -p hone-event-engine unique_batch_get_open_id --lib -- --nocapture`
  - `cargo test -p hone-event-engine sinks::feishu --lib -- --nocapture`
  - `rustfmt --edition 2024 --check crates/hone-event-engine/src/sinks/feishu.rs`
  - `cargo check -p hone-event-engine -p hone-web-api --tests`
- 当前结论：本轮闭合了 event-engine 价格异动卡片仍绕过 current-app open_id fallback 的本地可修缺口，并保留“解析结果唯一才替换”的误投保护；本缺陷更新为 `Fixed`。当前机器不是生产机器，未用线上健康检查或真实投递作为判定依据。
