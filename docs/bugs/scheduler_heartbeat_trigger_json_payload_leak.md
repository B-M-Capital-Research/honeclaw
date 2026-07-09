# Bug: Heartbeat 已触发提醒偶发向用户投递原始 JSON 载荷

- **发现时间**: 2026-04-18 11:06 CST
- **Bug Type**: Business Error
- **严重等级**: P3
- **状态**: New

## 最新进展

- `2026-07-10 03:02 CST` 真实运行态复发，状态从 `Fixed` 回退为 `New`：
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=47777`
    - `job_name=DRAM 心跳监控`
    - `executed_at=2026-07-10T03:01:15.498268+08:00`
    - `execution_status=completed`
    - `message_send_status=sent`
    - `delivered=1`
    - `detail_json.scheduler.parse_kind=JsonTriggered`
    - `response_preview` 前半段已经是自然语言提醒：`DRAM现价$65.25，已较昨收$62.04上涨+5.17%，突破$60触发位...`
    - 但自然语言正文后继续拼入结构化字段残片：`","facts":[...]`、`"actions_needed":[...]`、`{"level":"catalyst"...`
    - `detail_json.scheduler.deliver_preview` 同步保留 `","facts":[...]` 字段尾巴，说明不是单纯台账展示截断，而是准备投递的用户可见正文已经被结构化字段污染。
  - 查重结论：
    - 该样本与本文档既有 `JsonTriggered` 成功送达分支的“自然语言 + JSON 字段尾巴”同根；不是新的独立根因，因此不新建重复文档。
    - 最新污染字段扩展到 `facts`、`actions_needed` 和 catalyst 对象，说明 2026-06-22 的字段尾巴裁剪没有覆盖当前 JSON 形态。
  - 用户影响：
    - heartbeat 触发提醒已执行、已投递，也没有错投、漏投或全链路不可用证据。
    - 但用户会收到混有结构化协议字段的提醒正文，阅读体验和产品可信度下降，并暴露内部输出协议形态；这不影响主功能链路，因此仍按质量性 `P3 / New`，非 P1，不创建 GitHub Issue。

## 修复记录（2026-06-22 03:28 CST）

- 本轮在 `sanitize_scheduler_delivery_text(...)` 增加 heartbeat / scheduler 正文尾随结构化字段残片裁剪：
  - 当用户可见正文已经形成自然语言提醒，但尾部继续拼入 `","data":{...}`、`"direction":...`、`"ticker":...`、`"exchange":...`、`"threshold":...` 等结构化字段时，现在会在第一段可疑 JSON 字段标记前截断。
  - 清理同时兼容未转义和 `\"...\"` 转义残片，避免 `deliver_preview` / 最终投递正文继续暴露协议字段尾巴。
  - 不会影响正常引号文本；新增回归专门覆盖“正常中文引号说明”不被误裁剪。
- 验证：
  - `cargo test -p hone-channels scheduler_delivery_text_ --lib -- --nocapture`
  - `cargo check -p hone-channels --tests`
- 当前按代码与回归验证更新为 `Fixed`；若后续在最新代码运行态仍看到 heartbeat final 拼入新的结构化字段尾巴，再用新样本重新打开。

## 修复记录（2026-06-22 03:08 CST）

- heartbeat 畸形 `triggered` JSON 恢复逻辑已把 `data`、`direction`、`beat_threshold`、`threshold` 识别为 `message` 后续结构化字段，遇到自然语言提醒后拼入这些字段尾巴时会在出站前截断，避免 `","data":...` 或阈值字段残片进入用户可见提醒。
- 验证：
  - `cargo test -p hone-channels heartbeat_malformed_triggered_message_strips --lib -- --nocapture`
- 无关联 GitHub Issue；本轮按代码级修复关闭，不依赖生产日志、线上渠道状态或 live 重启。
- **证据来源**:
  - `2026-06-16 03:03 CST` 巡检补充复发证据：
    - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=43281`
    - `job_id=j_9ee85d42`
    - `job_name=Cerebras IPO与业务进展心跳监控`
    - `executed_at=2026-06-16T00:31:07.317015+08:00`
    - `execution_status=completed`
    - `message_send_status=sent`
    - `delivered=1`
    - `detail_json.scheduler.parse_kind=JsonTriggered`
    - `response_preview` 前半段已经是自然语言提醒，但尾部仍拼入 JSON 字段残片：`","data":{"ticker":"CBRS","exchange":"NASDAQ Global Market`
    - `detail_json.scheduler.deliver_preview` 同步保留该残片，说明不是单纯台账截断，而是准备投递的用户可见正文已经被结构化字段污染
    - 同窗另一条 heartbeat `TSLA 正负触发条件心跳监控` `run_id=43290` 正常触发并送达，无 JSON 残片；其余 heartbeat 失败主要是结构化 JSON / context window 既有形态，说明该问题仍是 `JsonTriggered` 成功送达分支的格式化抖动，而不是整批 scheduler 不可用
  - `2026-06-13 03:01 CST` 巡检补充复发证据：
    - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=41301`
    - `job_id=j_4756be4d`
    - `job_name=伦敦金跌破4500提醒`
    - `executed_at=2026-06-13T01:30:14.803841+08:00`
    - `execution_status=completed`
    - `message_send_status=sent`
    - `delivered=1`
    - `detail_json.scheduler.parse_kind=JsonTriggered`
    - `response_preview` 前半段已经是自然语言提醒，但尾部仍拼入 JSON 字段残片：`"direction":"below_threshold","beat_threshold":"281.83`
    - `detail_json.scheduler.deliver_preview` 同步保留该残片，说明不是单纯台账截断，而是准备投递的用户可见正文已经被结构化字段污染
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

- `2026-06-16 00:31` 的 `Cerebras IPO与业务进展心跳监控` 已成功触发并送达，正文主体是自然语言提醒，但后面继续拼入 `data.ticker` / `data.exchange` 字段残片。该样本与 `2026-06-13` 的金价样本同属“自然语言 + 结构化字段尾巴”混合输出形态，说明尾随 JSON 字段清理仍未覆盖非金价 heartbeat 任务。
- `2026-06-13 01:30` 的 `伦敦金跌破4500提醒` 已经成功触发并送达，正文主体是自然语言提醒，但末尾仍外露 JSON 字段残片 `direction` / `beat_threshold`。这晚于 2026-04-20 `unwrap_nested_json_message` 修复记录，说明修复只覆盖了完整 `{"trigger": ...}` 对象直出，未覆盖“自然语言 + 结构化字段尾巴”的混合输出形态。
- `2026-04-18 10:31` 的 `TEM大事件心跳监控` 已经成功命中触发并送达，但送达内容退化为原始 JSON 对象字符串。
- 这一轮不是简单的“记录脏了但用户侧正常”：`detail_json.scheduler.deliver_preview` 已直接等于 JSON 字符串，说明调度器准备发送的正文本身就是未格式化对象。
- 同一个任务在 `09:01` 和 `11:01` 又都恢复为自然语言提醒，进一步说明这是格式化链路的不稳定抖动。
- 同时间窗里其它 heartbeat 任务仍持续保留 `<think>` 污染的 `raw_preview`，说明当前 `JsonTriggered` 的投递格式化也仍建立在脆弱的协议解析之上。

## 用户影响

- 这是质量类缺陷。任务已执行、已投递，也没有发生错投、漏投或系统级失败。
- 但用户收到的是原始结构化对象，而不是产品化提醒文案，阅读体验和可信度明显下降，也会暴露内部协议形态。
- 之所以定级为 `P3`，是因为它没有阻断 heartbeat 主功能链路，用户仍收到触发提醒和核心价格信息；当前伤害主要是格式与质量退化，而不是功能不可用。

## 根因判断

- heartbeat `JsonTriggered` 分支的结果规范化不稳定；同一任务有时会把提取出的对象渲染成自然语言，有时却直接把 JSON 字符串作为最终正文。
- `2026-06-16` 复发样本显示污染字段已扩展到通用 `data` 对象字段（如 `ticker` / `exchange`），不是金价阈值任务的专属字段清理遗漏。
- `2026-06-13` 复发样本显示，格式化入口还可能只剥离对象开头或主体字段，却没有完整截断尾随结构化字段，导致自然语言正文后拼接 `direction` / `beat_threshold`。
- 结合最近一小时其它 heartbeat 仍保留 `<think>` 污染输出，可以推断当前格式化逻辑仍依赖脆弱的“先解析结构，再拼装文案”路径，不同轮次对对象形态或字段内容的兼容不一致。
- 这与 [`scheduler_heartbeat_unknown_status_silent_skip.md`](./scheduler_heartbeat_unknown_status_silent_skip.md) 共享同一协议脆弱背景，但这里的直接症状已从“失败跳过”变成“成功送达但格式退化”。

## 下一步建议

- 检查 heartbeat `JsonTriggered` 结果的统一格式化入口，确认对象型结果何时会被直接 `to_string` 或原样透传。
- 为 `triggered` 分支补回归测试，至少覆盖：
  - 对象型 `{"trigger":"..."}` 返回
  - 中英文字段内容
  - 同时含 `<think>` 污染原文但已成功解析出触发态的情况
- 在台账里继续观察是否还有其它 heartbeat 任务把 `response_preview` / `deliver_preview` 记成原始 JSON；若扩散到多条任务，可考虑提升优先级。
