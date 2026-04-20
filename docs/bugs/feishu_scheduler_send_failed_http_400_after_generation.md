# Bug: Feishu 直达定时任务生成完成后仍在发送阶段落成 `HTTP 400 Bad Request`

- **发现时间**: 2026-04-16 22:08 CST
- **Bug Type**: System Error
- **严重等级**: P1
- **状态**: Fixing
- **证据来源**:
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
- `bins/hone-feishu/src/client.rs` 也已补发信失败时的响应体日志，后续再出现 400 时不再只剩裸 `Bad Request`，而会带上 Feishu body 摘要，便于继续定位 payload 差异。
- 自动化验证已通过：
  - `cargo test -p hone-feishu`
  - `cargo test -p hone-channels`
- 但 `2026-04-17 21:32` 的下一轮真实 scheduler 窗口已经再次复现 `Oil_Price_Monitor_Premarket -> completed + send_failed + HTTP 400`；说明当前修复还没有收口到生产链路，本单继续保持 `Fixing`。
