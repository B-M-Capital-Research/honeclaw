# Bug: Feishu 定时任务目标校验长期失败，任务生成内容后仍无法送达

- **发现时间**: 2026-04-15 22:02 CST
- **Bug Type**: System Error
- **严重等级**: P1
- **状态**: New
- **GitHub Issue**: [#32](https://github.com/B-M-Capital-Research/honeclaw/issues/32)
- **证据来源**:
  - 最近一小时真实任务台账：`data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=1795`，`job_id=j_f02dfce5`，`job_name=OWALERT_PreMarket`，`executed_at=2026-04-15T21:04:28.730839+08:00`
    - `execution_status=execution_failed`，`message_send_status=target_resolution_failed`，`delivered=0`
    - `error_message=集成错误: resolved receive_id ou_e31244b1208749f16773dce0c822801a does not match actor ou_3f69c84593eccd71142ed767a885f595 for direct task target +8617326027390`
    - 同一 run 的 `response_preview` 已留下 `opencode acp session/prompt idle timeout (180s)`，说明任务侧已经进入执行链路，但最终仍因为目标校验失败被拦截
    - `run_id=1803`，`job_id=j_dac3b571`，`job_name=Oil_Price_Monitor_Premarket`，`executed_at=2026-04-15T21:35:01.517036+08:00`
    - `execution_status=completed`，`message_send_status=target_resolution_failed`，`delivered=0`
    - 同样的 `error_message` 再次出现，说明不是单个 job 配置损坏
  - 最近一小时运行日志：
    - `data/runtime/logs/web.log`
      - `2026-04-15 21:04:28.729` `[Feishu] 定时任务目标校验失败: job=OWALERT_PreMarket target=+8617326027390 receive_id=ou_e31244b1208749f16773dce0c822801a`
      - `2026-04-15 21:35:01.515` `[Feishu] 定时任务目标校验失败: job=Oil_Price_Monitor_Premarket target=+8617326027390 receive_id=ou_e31244b1208749f16773dce0c822801a`
    - `data/runtime/logs/hone-feishu.release-restart.log`
      - `2026-04-15T13:04:28.729124Z` 同样记录 `target_resolution_failed`
      - `2026-04-15T13:35:01.515179Z` 同样记录 `target_resolution_failed`
  - 2026-04-16 08:31 最新复核：
    - `run_id=1838`，`job_id=j_b3bc4b42`，`job_name=每日宏观与AI早报`，`executed_at=2026-04-16T08:31:58.676736+08:00`
    - `execution_status=completed`，`message_send_status=target_resolution_failed`，`delivered=0`
    - `response_preview` 已生成约 1.3k 字完整晨报，但 `error_message=集成错误: No user found for mobile ou_e31244b1208749f16773dce0c822801a`
    - 说明当前失败形态已从“receive_id 与 actor 不匹配”扩展到“解析结果本身无法在用户目录中反查”，但仍然落在同一条 direct scheduler 目标校验链路上
  - 更早历史复核：
    - 同一 `actor_user_id=ou_3f69c84593eccd71142ed767a885f595`、同一 `channel_target=+8617326027390` 的失败记录可追溯到 `2026-03-25`
    - 说明这不是“本小时偶发回归”，而是长期存在且当前仍在持续影响用户的活跃缺陷
  - 相关历史缺陷：
    - `docs/bugs/feishu_message_misrouting.md`
    - 区别在于该缺陷当前表现为“校验拦截后无法送达”，而不是把消息误投给错误用户

## 端到端链路

1. 用户此前创建了 Feishu 直达型定时任务，任务配置里保存的 `channel_target` 为手机号 `+8617326027390`。
2. 调度器按计划触发任务，模型执行阶段已经产生了内容或至少进入了正式运行。
3. 发送前的目标解析把 `channel_target` 解析成了另一个 `receive_id=ou_e31244b1208749f16773dce0c822801a`。
4. 系统随后把解析结果与任务绑定的 `actor_user_id=ou_3f69c84593eccd71142ed767a885f595` 做一致性校验。
5. 由于两者不匹配，发送链路被标记为 `target_resolution_failed` 并终止，最终 `delivered=0`，用户拿不到任何推送结果。

## 期望效果

- Feishu 定时任务在 direct/p2p 场景下应稳定解析回任务创建者本人的 `open_id`，并把结果送达到正确对象。
- 如果解析结果与绑定 actor 不一致，系统可以拒绝发送，但不能让同一批任务长期重复失败而无人修复。
- 对用户而言，应要么成功收到推送，要么看到可操作的明确失败信号，而不是后台长期静默积累 `target_resolution_failed`。

## 当前实现效果

- 当前同一用户的多个 Feishu 定时任务在最近一小时内连续命中 `target_resolution_failed`。
- 一部分 run 已经生成了较长 `response_preview`，说明任务主体并非没跑，而是“产出内容后在投递前被拦住”。
- 另一部分 run 会同时叠加执行期错误，但即便执行阶段成功，最终也仍然会在目标校验处失败。
- `08:31` 的 `每日宏观与AI早报` 还表明，即使本轮内容完整生成，目标解析也可能进一步退化成 `No user found for mobile ou_e31244...`，说明 direct scheduler 的目标字段已经不是稳定的 canonical mobile/open_id 表达。
- 从日志历史看，这个目标解析不一致问题已经持续多周，当前仍没有恢复。

## 用户影响

- 这是功能性缺陷，不是单纯回答质量问题。用户配置的定时任务无法稳定送达，自动提醒链路实际失效。
- 之所以定级为 `P1`，是因为问题影响的是用户主依赖的定时推送能力，而且同一用户的多个任务在当前小时仍持续失败。
- 该问题不是 `P0`，因为现有校验至少阻止了跨用户误投；损害主要表现为“任务无法送达”，而不是“发错对象”。

## 根因判断

- 任务配置中的 `channel_target` 与绑定 actor 的 canonical 标识长期不一致，当前解析逻辑既可能把手机号解析到另一个用户的 `open_id`，也可能把已经像 `open_id` 的值再次当成 mobile 反查。
- 发送前新增的一致性校验阻止了旧的跨用户误投，但没有配套的迁移或修复机制来纠正历史错误 target，因此任务会持续卡在拒发状态。
- `cron_job_runs` 已多次显示相同 direct scheduler 在 `receive_id 不匹配` 与 `No user found` 两种校验错误间切换，说明问题更接近“定时任务目标存量数据或解析策略不一致”，而不是本轮模型输出异常。

## 修复情况（2026-04-16）

- 已在 `bins/hone-feishu/src/handler.rs` 收紧 direct target 解析规则：
  - `looks_like_mobile(...)` 现在只会把由数字/`+`/常见电话号码分隔符构成的目标识别为手机号，不再把带很多数字的 `open_id` 误判成 mobile
  - 新增 `scheduler_receive_id_for_target(...)`，对 direct Feishu scheduler 的 email/mobile 历史 target 直接回收为绑定 actor 的 `open_id`
- 已在 `bins/hone-feishu/src/scheduler.rs` 接入该 direct scheduler 优先规则：
  - direct 任务不再依赖历史 `channel_target` 二次解析后再做一致性校验
  - 发送时优先使用 `event.actor.user_id` 这一已绑定且稳定的 `open_id`
- 这样既保留了历史上的防误投校验，又避免 direct 定时任务因为旧手机号/email target 漂移而长期卡在 `target_resolution_failed`。

## 状态更新（2026-05-05 07:40 CST）

- 本轮巡检确认该缺陷不能继续维持 `Fixed`：
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=15655`
    - `job_id=j_240ef7aa`
    - `job_name=科技成长赛道大盘极值与情绪监控`
    - `actor_user_id=ou_895bed1573d53053e89bfc382b523a44`
    - `channel_target=+8613067903569`
    - `executed_at=2026-05-05T05:23:45.720309+08:00`
    - `execution_status=completed`
    - `message_send_status=target_resolution_failed`
    - `delivered=0`
    - `response_preview` 已是约 700+ 字完整监控结论开头，说明任务主体已跑完
    - `error_message=集成错误: Feishu resolve mobile request failed: error sending request for url (https://open.feishu.cn/open-apis/contact/v3/users/batch_get_id?user_id_type=open_id)`
  - `data/runtime/logs/web.log.2026-05-04`
    - `2026-05-05 05:23:45.716` 明确记录：`[Feishu] 定时任务目标解析失败: job=科技成长赛道大盘极值与情绪监控 target=+8613067903569 err=集成错误: Feishu resolve mobile request failed ... batch_get_id?user_id_type=open_id`
  - 历史台账复核：
    - `run_id=626`（`2026-04-08T08:33:21.037181+08:00`，`创新药持仓每日动态推送`）也曾出现同一错误文案：`Feishu resolve mobile request failed ... batch_get_id?user_id_type=open_id`
    - 说明 contact lookup 传输失败不是单次孤立抖动，而是 direct scheduler 目标解析链路的已复现变体
- 这次坏态与 2026-04-15/04-16 的旧样本不同：
  - 旧样本主要是 `resolved receive_id ... does not match actor ...` 或 `No user found for mobile ...`
  - 最新样本则是在 Feishu 联系人查询 `batch_get_id` 请求阶段直接传输失败
  - 但三者的共同结果没有变：内容已经生成，最终仍在 target resolution 链路被拦截，用户收不到定时任务
- 因此本单状态回退为 `New`，严重等级维持 `P1`。

## GitHub Issue

- [#32](https://github.com/B-M-Capital-Research/honeclaw/issues/32) `[P1][hone-scanner] Feishu direct scheduler target resolution blocks delivery`
- 旧的 [#34](https://github.com/B-M-Capital-Research/honeclaw/issues/34) 已关闭；当前活跃跟踪统一回挂到仍为 `OPEN` 的 #32，避免重复建单。

## 回归验证

- `cargo test -p hone-feishu scheduler_delivery_ -- --nocapture`
- `cargo test -p hone-feishu looks_like_mobile_does_not_treat_open_id_as_mobile -- --nocapture`
- `cargo test -p hone-feishu direct_scheduler_prefers_actor_open_id_for_contact_targets -- --nocapture`
- `cargo check -p hone-feishu`
- `rustfmt --edition 2024 --check bins/hone-feishu/src/handler.rs bins/hone-feishu/src/scheduler.rs`
- `git diff --check`
