# Bug: Feishu 直达定时任务生成完成后仍在发送阶段落成 `HTTP 400 Bad Request`

- **发现时间**: 2026-04-16 22:08 CST
- **Bug Type**: System Error
- **严重等级**: P1
- **状态**: Fixing
- **证据来源**:
  - 最近一小时调度落库：`data/sessions.sqlite3` -> `cron_job_runs`
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
    - `2026-04-16T21:04:05.652096+08:00` assistant 已写入 `OWALERT_PreMarket` 最终播报
    - `2026-04-16T21:33:06.067389+08:00` assistant 已写入 `Oil_Price_Monitor_Premarket` 最终播报
    - `2026-04-17T04:01:50.132692+08:00` assistant 已写入 `Oil_Price_Monitor_Closing` 最终播报
    - `2026-04-17T04:31:32.813844+08:00` assistant 已写入 `OWALERT_PostMarket` 最终播报
    - 说明调度触发、模型执行、会话持久化都已成功，但用户侧仍未送达
  - 最近一小时运行日志：
    - `data/runtime/logs/hone-feishu.release-restart.log`
      - `2026-04-16T13:04:06.270953Z` `[Feishu] 定时任务投递失败: job=OWALERT_PreMarket target=+8617326027390 err=集成错误: Feishu send message failed: HTTP 400 Bad Request`
      - `2026-04-16T13:33:06.728472Z` `[Feishu] 定时任务投递失败: job=Oil_Price_Monitor_Premarket target=+8617326027390 err=集成错误: Feishu send message failed: HTTP 400 Bad Request`
    - `data/runtime/logs/web.log`
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

- `OWALERT_PreMarket`、`Oil_Price_Monitor_Premarket`、`Oil_Price_Monitor_Closing` 与 `OWALERT_PostMarket` 在最近几个窗口连续四次失败。
- 四次失败都发生在相同用户、相同手机号目标、相同 scheduler 送达链路。
- 与前一日的 `target_resolution_failed` 不同，这一轮 `receive_id` 已解析为正确 actor，但发送接口仍直接返回 400。
- 当前日志只保留了通用 `HTTP 400 Bad Request`，没有记录响应体、请求类型或被拒字段，因此只能确认是发送阶段故障，无法从现有日志进一步判定是 Markdown/卡片格式、消息体长度，还是 `receive_id_type` 等请求参数问题。
- 最近一小时新增样本说明故障已经从“盘前提醒”扩展到“收盘监控 / 收盘后提醒”，属于同一发送链路持续失败，而不是某一个 job 文案偶发异常。
- `2026-04-17 08:34` 的 `Hone_AI_Morning_Briefing` 新样本说明故障仍在最新小时窗活跃，并且已从盘前 / 收盘 / 盘后扩散到“日常早报”任务；受影响对象仍是同一 `receive_id` 与同一目标手机号。

## 用户影响

- 这是功能性缺陷，不是回答质量问题。任务正文已经生成，但用户完全收不到本该送达的定时播报。
- 受影响的是用户高频依赖的盘前提醒链路，且同一目标在本小时连续两次失败，因此定级为 `P1`。
- 之所以不是 `P0`，是因为当前表现为“消息丢失”而不是“误投到错误对象”或更大范围全局不可用。

## 根因判断

- 初步判断：旧的 direct target 解析问题已基本收敛，因为 `detail_json.receive_id` 已回到绑定 actor；当前新故障位于 Feishu 发送请求构造或请求内容校验阶段。
- 现有证据不足以确认具体子根因。两次失败均缺少 Feishu 侧响应 body，日志可观测性不足是当前定位阻塞点。
- 由于同一目标在 `2026-04-16 20:03` 仍有一条 `run_id=1976` 成功送达，而 `2026-04-17 04:01` 与 `04:31` 的新样本继续失败，说明并非该用户或该目标整体不可用，更像是 scheduler 当前某类 payload 形态在发送阶段稳定触发了 400。
- 新增失败样本覆盖盘前、收盘、收盘后三种 job 名称，但都指向同一 `receive_id` 与同一 actor，会更支持“Feishu 发送请求构造/消息体校验”这一公共链路根因，而不是单个任务 prompt 内容问题。
- `08:34` 的早报任务失败进一步排除了“只在某一类油价/盘后模板文案触发 400”的可能性；更像是面向同一 Feishu 直达目标的 scheduler 发送链路在多种长文本 payload 上都可能稳定触发 400。

## 下一步建议

- 在 Feishu scheduler 发送失败分支补记录请求元信息与响应体摘要，至少包含 `receive_id_type`、消息类型、正文长度、是否走 markdown/card 分支。
- 对 `+8617326027390` 最近成功与失败 run 的发送 payload 做差异比对，优先比较 `run_id=1976` 与 `run_id=1998/2005`。
- 若确认只是同一发送链路的新阶段回归，应在修复后回写 `docs/bugs/README.md` 与本文件状态；修复前不要恢复为 `Fixed`。

## 当前修复进展（2026-04-17 10:40 CST）

- 本轮先按“多段 direct scheduler 发送链路不稳定”收口：
  - `bins/hone-feishu/src/outbound.rs` 现在对 `receive_id_type=open_id` 且没有 placeholder 的多段消息，不再默认把后续分段走 `reply_message`；会直接逐段 standalone send。
  - 如果 `update_message` 或 `reply_message` 仍返回 `HTTP 400`，同一分段会自动回退到 standalone send，而不是整轮直接 `send_failed`。
- `bins/hone-feishu/src/client.rs` 也已补发信失败时的响应体日志，后续再出现 400 时不再只剩裸 `Bad Request`，而会带上 Feishu body 摘要，便于继续定位 payload 差异。
- 自动化验证已通过：
  - `cargo test -p hone-feishu`
  - `cargo test -p hone-channels`
- 由于当前还缺少下一轮真实 scheduler 窗口的送达样本，本单状态先更新为 `Fixing`。
