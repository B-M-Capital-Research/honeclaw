# Bug: 原油定时播报把未核验地缘叙述当作油价事实送达用户

- **发现时间**: 2026-04-22 07:00 CST
- **Bug Type**: Business Error
- **严重等级**: P2
- **状态**: New
- **证据来源**:
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=4397`
    - `job_id=j_38745baf`
    - `job_name=全天原油价格3小时播报`
    - `executed_at=2026-04-22T09:01:42.878090+08:00`
    - `execution_status=completed`
    - `message_send_status=sent`
    - `should_deliver=1`
    - `delivered=1`
    - `response_preview` 已再次向用户发送：`Trump于4月2日就伊朗问题发表讲话后布伦特飙升近8%至$109新高`、`OPEC+减产预期与地缘风险持续支撑价格`、`摩根大通预测2026年布伦特均价约$60（偏空）`。
    - `detail_json.scheduler.raw_preview` 显示模型在内部把 WTI/Brent 价格和地缘叙述混在同一段 `<think>` 自由文本中，再解析为 `JsonTriggered` 并送达；与 06:02 样本同根因，说明市场播报仍缺少来源质量与置信度门禁。
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=4326`
    - `job_id=j_38745baf`
    - `job_name=全天原油价格3小时播报`
    - `executed_at=2026-04-22T06:02:01.143394+08:00`
    - `execution_status=completed`
    - `message_send_status=sent`
    - `should_deliver=1`
    - `delivered=1`
    - `response_preview` 已向用户发送：`美伊战争于2月28日爆发`、`4月17日伊朗宣布霍尔木兹海峡重新开放，油价暴跌逾11%`、`4月19日美伊海上对峙升级`、`美伊停火协议即将于4月22日到期`
    - `detail_json.scheduler.raw_preview` 显示模型在内部将这些高风险叙述当作“关键事件”和“背景”组织，随后解析为 `JsonTriggered` 并送达。
  - `data/runtime/logs/web.log`
    - `2026-04-22 06:01:58.639` 记录同一任务 `parse_kind=JsonTriggered`、`starts_with_json=false`，`raw_preview` 以 `<think>` 开头并把上述地缘叙述纳入触发正文。
    - 同一小时 `06:01:14-06:01:44` 多次记录 `tavily request failed ... usage limit`，说明本轮外部检索质量本身存在降级迹象，但最终仍生成并投递了确定性地缘结论。
  - 对照同一任务前序输出：
    - `run_id=4263`（`2026-04-22T03:00:52.079347+08:00`）也曾围绕“美伊核谈判进展”“地缘紧张局势”解释原油价格，但 06:02 样本进一步升级为更具体的战争、海峡开放、停火到期等确定性事件串，并已投递。

## 端到端链路

Feishu heartbeat scheduler 触发 `全天原油价格3小时播报` -> function-calling runner 查询原油价格与近期原因 -> web search 多个 key 触发额度失败告警 -> 模型仍把未核验的地缘冲突叙述组织成原油波动原因 -> heartbeat 解析为 `JsonTriggered` -> Feishu scheduler 以 `completed + sent` 送达用户。

## 期望效果

原油播报应只发送已核验的价格、时间口径与可追溯的近期驱动因素。外部检索降级或来源不足时，应明确降低置信度、避免编造成确定性地缘事件链，必要时只播报价格并说明“原因需等待更多可验证来源”。

## 当前实现效果

系统在外部搜索出现额度失败告警的同一轮，仍将高风险地缘叙述当作事实发送给用户，并把它们作为油价变化的主因。09:01 复核显示，即使输出不再复述 06:02 的完整战争/停火叙事，也仍继续把单一政治讲话、布伦特冲高、OPEC+ 与地缘风险包装成确定性驱动并送达。该任务并非“美伊战争专题”，而是原油价格播报；当前输出把未经足够验证的宏观事件链混入投资相关定时提醒。

## 用户影响

用户会收到已经送达的市场播报，并可能据此判断原油、通胀、成长股和组合风险。由于这是投资相关定时提醒的事实正确性问题，影响的是用户对市场变量的判断，而不仅是表达风格或格式质量，因此按业务正确性缺陷定级为 P2。

## 根因判断

初步判断是 heartbeat 内容生成链路缺少事实置信度和来源质量门禁：外部检索出现配额失败时，模型仍可沿用历史上下文、旧摘要或低质量搜索片段生成确定性事件叙述；调度器只按 `JsonTriggered` 发送，不校验事实来源、时间口径和任务主题是否足够匹配。

## 下一步建议

- 为市场播报类 heartbeat 增加来源质量检查：搜索配额失败或来源不足时，不允许输出确定性宏观事件链。
- 原油价格任务应把“价格数据”和“原因解释”拆成结构化字段，并为原因附来源时间；缺少来源时降级为低置信度说明。
- 检查历史 compact summary 是否把“美伊战争 / 霍尔木兹”叙述长期带入市场类 prompt，避免旧上下文污染后续定时播报。
