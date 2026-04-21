# Bug: Heartbeat 已触发提醒偶发向用户投递原始 JSON 载荷

- **发现时间**: 2026-04-18 11:06 CST
- **Bug Type**: Business Error
- **严重等级**: P3
- **状态**: Fixed
- **证据来源**:
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=2398`
    - `job_id=j_818f0150`
    - `job_name=TEM大事件心跳监控`
    - `executed_at=2026-04-18T10:31:30.506141+08:00`
    - `execution_status=completed`
    - `message_send_status=sent`
    - `delivered=1`
    - `response_preview` 直接等于原始 JSON 对象字符串：
      - `{"trigger":"标的: TEM (Tempus AI)\n触发条件: 利好类事件 - 重要学术会议重磅数据发布\n当前价格: $55.87 ..."}`
    - `detail_json.scheduler.parse_kind=JsonTriggered`
    - `detail_json.scheduler.deliver_preview` 同样记录为原始 JSON 对象字符串，而不是自然语言提醒
  - 最近运行日志：
    - `data/runtime/logs/web.log`
      - `2026-04-18 10:31:26.888` `job_id=j_818f0150` 记录 `parse_kind=JsonTriggered`
      - 同一行 `deliver_preview="{"trigger":"标的: TEM (Tempus AI)\n触发条件: 利好类事件 - 重要学术会议重磅数据发布 ..."}"`
    - `data/runtime/logs/hone-feishu.release-restart.log`
      - `2026-04-18T02:31:26.888655Z` 同一任务同样记录 `deliver_preview="{"trigger":"标的: TEM (Tempus AI)\n触发条件: 利好类事件 - 重要学术会议重磅数据发布 ..."}"`
  - 同任务前后对照样本：
    - `run_id=2366`，`executed_at=2026-04-18T09:01:32.710632+08:00`，同一 `TEM大事件心跳监控` 已能投递自然语言提醒
    - `run_id=2408`，`executed_at=2026-04-18T11:01:27.592766+08:00`，同一任务再次恢复为自然语言提醒
    - 说明问题不是用户配置或任务语义变化，而是同一 heartbeat 触发链路在相邻窗口间出现“有时正常格式化、有时直接投递 JSON”的不稳定行为

## 端到端链路

1. Feishu heartbeat 任务 `TEM大事件心跳监控` 在 `2026-04-18 10:31` 命中触发条件，scheduler 进入已触发投递分支。
2. 模型原始输出依旧带有 `<think>` 分析段，但解析器成功识别出 `JsonTriggered`。
3. 当前投递链路没有把这次解析结果稳定格式化成自然语言提醒，而是直接把提取出的 JSON 对象字符串作为最终投递正文。
4. 调度台账把本轮记为 `completed + sent + delivered=1`，但用户实际拿到的是结构化对象文本，而不是面向人类阅读的提醒文案。

## 期望效果

- heartbeat 在命中 `JsonTriggered` 后，应始终输出稳定、可直接阅读的自然语言提醒。
- 无论模型内部返回中文、英文，或不同字段顺序的 JSON，scheduler 最终投递都不应把原始对象字符串直接发给用户。
- `cron_job_runs.response_preview` 应反映用户最终看到的提醒文案，而不是格式化前的结构化对象。

## 当前实现效果

- `2026-04-18 10:31` 的 `TEM大事件心跳监控` 已经成功命中触发并送达，但送达内容退化为原始 JSON 对象字符串。
- 这一轮不是简单的“记录脏了但用户侧正常”：`detail_json.scheduler.deliver_preview` 已直接等于 JSON 字符串，说明调度器准备发送的正文本身就是未格式化对象。
- 同一个任务在 `09:01` 和 `11:01` 又都恢复为自然语言提醒，进一步说明这是格式化链路的不稳定抖动。
- 同时间窗里其它 heartbeat 任务仍持续保留 `<think>` 污染的 `raw_preview`，说明当前 `JsonTriggered` 的投递格式化也仍建立在脆弱的协议解析之上。

## 用户影响

- 这是质量类缺陷。任务已执行、已投递，也没有发生错投、漏投或系统级失败。
- 但用户收到的是原始结构化对象，而不是产品化提醒文案，阅读体验和可信度明显下降，也会暴露内部协议形态。
- 之所以定级为 `P3`，是因为它没有阻断 heartbeat 主功能链路，用户仍能从 JSON 中读到大部分关键信息；当前伤害主要是格式与质量退化，而不是功能不可用。

## 根因判断

- heartbeat `JsonTriggered` 分支的结果规范化不稳定；同一任务有时会把提取出的对象渲染成自然语言，有时却直接把 JSON 字符串作为最终正文。
- 结合最近一小时其它 heartbeat 仍保留 `<think>` 污染输出，可以推断当前格式化逻辑仍依赖脆弱的“先解析结构，再拼装文案”路径，不同轮次对对象形态或字段内容的兼容不一致。
- 这与 [`scheduler_heartbeat_unknown_status_silent_skip.md`](./scheduler_heartbeat_unknown_status_silent_skip.md) 共享同一协议脆弱背景，但这里的直接症状已从“失败跳过”变成“成功送达但格式退化”。

## 下一步建议

- 检查 heartbeat `JsonTriggered` 结果的统一格式化入口，确认对象型结果何时会被直接 `to_string` 或原样透传。
- 为 `triggered` 分支补回归测试，至少覆盖：
  - 对象型 `{"trigger":"..."}` 返回
  - 中英文字段内容
  - 同时含 `<think>` 污染原文但已成功解析出触发态的情况
- 在台账里继续观察是否还有其它 heartbeat 任务把 `response_preview` / `deliver_preview` 记成原始 JSON；若扩散到多条任务，可考虑提升优先级。
