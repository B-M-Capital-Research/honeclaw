# Bug: Feishu 定时任务目标校验长期失败，任务生成内容后仍无法送达

- **发现时间**: 2026-04-15 22:02 CST
- **Bug Type**: System Error
- **严重等级**: P1
- **状态**: New
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
- 从日志历史看，这个目标解析不一致问题已经持续多周，当前仍没有恢复。

## 用户影响

- 这是功能性缺陷，不是单纯回答质量问题。用户配置的定时任务无法稳定送达，自动提醒链路实际失效。
- 之所以定级为 `P1`，是因为问题影响的是用户主依赖的定时推送能力，而且同一用户的多个任务在当前小时仍持续失败。
- 该问题不是 `P0`，因为现有校验至少阻止了跨用户误投；损害主要表现为“任务无法送达”，而不是“发错对象”。

## 根因判断

- 任务配置中的 `channel_target` 与绑定 actor 的 canonical `open_id` 长期不一致，当前解析逻辑会把手机号再次解析到另一个用户的 `open_id`。
- 发送前新增的一致性校验阻止了旧的跨用户误投，但没有配套的迁移或修复机制来纠正历史错误 target，因此任务会持续卡在拒发状态。
- `cron_job_runs` 已多次显示相同 `channel_target -> receive_id` 映射错误，说明问题更接近“定时任务目标存量数据或解析策略不一致”，而不是本轮模型输出异常。

## 下一步建议

- 先排查这批 direct 定时任务的存量 `channel_target` 与 `actor_user_id` 是否来自历史旧数据；若是，补一轮迁移或重绑定。
- 对 Feishu scheduler 的目标解析链路增加更明确的可观测性，区分“历史脏数据”“手机号反查漂移”“用户目录变更”三类原因。
- 在导航页和后续修复单里把该缺陷与 `feishu_message_misrouting.md` 明确区分：前者是当前被校验拦住的送达失败，后者是历史上的真实跨用户误投。
- 对已经连续失败的任务增加主动告警或在 UI/台账上暴露，让用户和维护者知道这些任务当前不会送达。
