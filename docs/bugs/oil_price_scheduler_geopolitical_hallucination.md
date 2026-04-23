# Bug: 原油定时播报把未核验地缘叙述当作油价事实送达用户

- **发现时间**: 2026-04-22 07:00 CST
- **Bug Type**: Business Error
- **严重等级**: P2
- **状态**: New
- **证据来源**:
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=5108`
    - `job_id=j_38745baf`
    - `job_name=全天原油价格3小时播报`
    - `executed_at=2026-04-23T18:00:50.267119+08:00`
    - `execution_status=completed`
    - `message_send_status=sent`
    - `should_deliver=1`
    - `delivered=1`
    - `response_preview` 再次向用户发送：`WTI原油：$94.43/桶`、`布伦特原油：$103.12/桶`，并把 `霍尔木兹海峡自2月底以来持续受阻（美伊冲突升级）`、`主要产油国出口均受影响`、`摩根大通警告若海峡受阻持续至5月中旬油价可能突破$150`、`特朗普给伊朗设定开放海峡最后期限` 写成近期变动主因。
    - `detail_json.scheduler.raw_preview` 仍以 `<think>` 开头，先承认自己在整理“最新原油价格和近期变动原因”，随后直接把上述地缘叙述组织成 `JsonTriggered` 出站正文；同一轮没有提供可追溯新闻来源、发布时间、来源可信度或交叉验证结果。
  - `data/runtime/logs/sidecar.log`
    - `2026-04-23 18:00:47.665-18:00:47.666` 记录同一任务 `parse_kind=JsonTriggered`、`starts_with_json=false`，`deliver_preview` 与 sqlite 中的送达正文一致。
    - `2026-04-23 18:00:48.344` 与 `18:00:48.601` 同批再次记录 `tavily request failed ... usage limit`，说明外部检索仍带额度降级噪声，但最终仍生成并投递了确定性地缘结论。
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=4975`
    - `job_id=j_38745baf`
    - `job_name=全天原油价格3小时播报`
    - `executed_at=2026-04-23T12:00:26.652887+08:00`
    - `execution_status=completed`
    - `message_send_status=sent`
    - `should_deliver=1`
    - `delivered=1`
    - `response_preview` 继续向用户发送：`WTI原油：约$99-101/桶（4月22日收盘区间）`、`布伦特原油：$101.91/桶（4月22日收盘，较前日上涨超3%，连续第四日上涨）`，并把 `美国燃料库存意外下降`、`全球航运路线地缘政治风险升温`、`美国与伊朗外交谈判进展不明` 和 `分析师预计布伦特Q2峰值或达$115/桶` 写成近期上涨原因。
    - `detail_json.scheduler.raw_preview` 仍以 `<think>` 开头，承认 `没有返回精确的实时价格`，但随后仍把搜索片段拼成 `JsonTriggered` 并送达；同一轮没有提供可追溯来源、发布时间、来源可信度或交叉验证结果。
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=4904`
    - `job_id=j_38745baf`
    - `job_name=全天原油价格3小时播报`
    - `executed_at=2026-04-23T09:01:33.486116+08:00`
    - `execution_status=completed`
    - `message_send_status=sent`
    - `should_deliver=1`
    - `delivered=1`
    - `response_preview` 再次向用户发送：`WTI原油：$93.20/桶，日内涨幅 +$3.53 (+3.94%)`、`布伦特原油：$102.27/桶`，并把 `伊朗停火协议扩展不确定性升温`、`市场担忧中东地缘政治风险及供应扰动` 写成近期变动原因。
    - `detail_json.scheduler.raw_preview` 仍以 `<think>` 开头，内部只把价格数据与“伊朗停火协议扩展不确定性”拼接后直接构造 `JsonTriggered`；同一轮没有提供可追溯来源、发布时间、来源可信度或交叉验证结果。
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=4828`
    - `job_id=j_38745baf`
    - `job_name=全天原油价格3小时播报`
    - `executed_at=2026-04-23T06:00:47.640940+08:00`
    - `execution_status=completed`
    - `message_send_status=sent`
    - `should_deliver=1`
    - `delivered=1`
    - `response_preview` 继续向用户发送：`WTI原油：约$92.60/桶`、`布伦特原油：约$99-101/桶区间`，并把 `4月13日美国宣布封锁伊朗港口后油价飙涨`、`4月17日伊朗外长宣布霍尔木兹海峡恢复商业航运后单日暴跌逾10%`、`市场正关注美伊局势后续走向及谈判进展` 写成近期波动主因。
    - `detail_json.scheduler.raw_preview` 仍以 `<think>` 开头，内部列出 `美伊战争导致霍尔木兹海峡关闭/动荡` 等原因后解析为 `JsonTriggered`；同一轮仍没有给出可追溯来源、发布时间、来源可信度或交叉验证结果。
  - `data/runtime/logs/sidecar.log`
    - `2026-04-23 06:00:45.147` 记录同一任务 `parse_kind=JsonTriggered`、`starts_with_json=false`，`deliver_preview` 与 sqlite 中的送达正文一致；同批 `06:00:10-06:00:25` 多次记录 `tavily request failed ... usage limit`，说明外部检索仍有额度降级迹象。
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=4759`
    - `job_id=j_38745baf`
    - `job_name=全天原油价格3小时播报`
    - `executed_at=2026-04-23T03:00:39.918824+08:00`
    - `execution_status=completed`
    - `message_send_status=sent`
    - `should_deliver=1`
    - `delivered=1`
    - `response_preview` 再次向用户发送：`WTI原油：$92.99 | 日涨幅 +$3.32 (+3.70%)`、`布伦特原油：$102.02 | 日涨幅 +$3.54 (+3.59%)`，并把 `美伊冲突持续升级`、`霍尔木兹海峡实际关闭`、`特朗普表示美伊冲突可能持续数周`、`WTI期货4月初曾单日飙升超12%`、`市场库存缓冲正在被消耗` 写成近期价格大涨主因。
    - `detail_json.scheduler.raw_preview` 仍以 `<think>` 开头，只列出价格与宏观叙述后直接组织 `JsonTriggered` 待投递正文；同一轮未提供可追溯新闻来源、发布时间、来源可信度或交叉验证结果。
  - `data/runtime/logs/web.log`
    - `2026-04-23 03:00:34-03:00:36` 同批多次记录 `tavily request failed ... usage limit`，说明外部搜索链路仍有额度降级；`2026-04-23 03:00:37.236` 随后记录同一原油任务 `parse_kind=JsonTriggered`、`starts_with_json=false`，并把上述地缘/库存叙述纳入 `deliver_preview`。
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=4691`
    - `job_id=j_38745baf`
    - `job_name=全天原油价格3小时播报`
    - `executed_at=2026-04-23T00:00:36.565619+08:00`
    - `execution_status=completed`
    - `message_send_status=sent`
    - `should_deliver=1`
    - `delivered=1`
    - `response_preview` 再次向用户发送：`WTI原油：$92.13/桶（4月22日收盘）`、`布伦特原油：$100.58/桶（4月22日收盘）`，并把 `霍尔木兹海峡局势骤然升温`、`至少三艘集装箱船遭到枪击`、`美国扣押伊朗船只`、`伊朗称其为"战争行为"`、`美伊脆弱的两周停火协议即将于4月23日到期` 写成近期大涨原因。
    - `detail_json.scheduler.raw_preview` 仍以 `<think>` 开头，并在没有可追溯新闻来源、发布时间、来源可信度或交叉验证结果的情况下，将上述地缘叙述直接组织为 `JsonTriggered` 待投递正文。
  - `data/runtime/logs/sidecar.log`
    - `2026-04-23 00:00:09-00:00:31` 同批多次记录 `tavily request failed ... usage limit`，说明外部搜索链路仍有额度降级；`2026-04-23 00:00:34.429` 随后记录同一原油任务 `parse_kind=JsonTriggered`、`starts_with_json=false`，并把上述地缘事件链纳入 `raw_preview`。
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=4622`
    - `job_id=j_38745baf`
    - `job_name=全天原油价格3小时播报`
    - `executed_at=2026-04-22T21:00:41.235036+08:00`
    - `execution_status=completed`
    - `message_send_status=sent`
    - `should_deliver=1`
    - `delivered=1`
    - `response_preview` 再次向用户发送：`WTI原油：$88.82/桶（日内约+1.25%）`、`布伦特原油：$93.59/桶（今日约+0.94%）`，并把 `OPEC+持续减产预期`、`市场预期 OPEC 及盟友将把创纪录减产延长至7月`、`区域性冲突持续扰动供应链` 和 `IEA 警告2026年可能出现每日400万桶以上的供应过剩` 写成近期价格变动原因。
    - 这一轮仍没有给出可追溯新闻来源、发布时间或置信度说明；正文虽附 `价格可能存在延迟`，但原因归因仍是确定性宏观叙事。
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=4562`
    - `job_id=j_38745baf`
    - `job_name=全天原油价格3小时播报`
    - `executed_at=2026-04-22T18:01:40.568312+08:00`
    - `execution_status=completed`
    - `message_send_status=sent`
    - `should_deliver=1`
    - `delivered=1`
    - `response_preview` 再次向用户发送：`WTI原油：$88.73/桶，较前一交易日下跌1.05%，月涨0.68%，年涨42.49%。布伦特原油（6月期货）：$98.57/桶`，并把 `全球原油产量下降`、`地缘政治紧张局势持续`、`伊朗停火协议未能有效安抚市场` 写成近期主要驱动因素。
    - `detail_json.scheduler.raw_preview` 仍以 `<think>` 开头，内部只列出价格、全球加工量预期和地缘叙述，随后直接生成带宏观归因的 `JsonTriggered` 消息；这一轮仍没有给出可追溯新闻来源、时间口径或置信度说明。
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=4498`
    - `job_id=j_38745baf`
    - `job_name=全天原油价格3小时播报`
    - `executed_at=2026-04-22T15:01:28.307896+08:00`
    - `execution_status=completed`
    - `message_send_status=sent`
    - `should_deliver=1`
    - `delivered=1`
    - `response_preview` 再次向用户发送：`WTI原油：$89.26/桶，日跌幅-0.45%；布伦特原油：$98.35/桶，日跌幅-0.14%`，并把 `近期价格波动主要受地缘政治紧张局势及市场需求预期变化影响` 作为确定性归因。
    - `detail_json.scheduler.raw_preview` 仍以 `<think>` 开头，内部只列出两条价格数据，随后直接生成带宏观归因的 `JsonTriggered` 消息；这一轮没有附带可追溯新闻来源或置信度说明。
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=4440`
    - `job_id=j_38745baf`
    - `job_name=全天原油价格3小时播报`
    - `executed_at=2026-04-22T12:00:29.317390+08:00`
    - `execution_status=completed`
    - `message_send_status=sent`
    - `should_deliver=1`
    - `delivered=1`
    - `response_preview` 再次向用户发送：`Trump于4月2日就伊朗问题发表讲话后布伦特一度飙升近8%至$109新高`、`伊朗停火谈判进展令供应忧虑缓解`、`OPEC+减产预期与地缘风险持续支撑价格底部`。
    - `detail_json.scheduler.raw_preview` 仍以 `<think>` 开头，把 WTI/Brent 区间价和地缘叙述拼成待投递正文，再被解析为 `JsonTriggered`；说明 09:01 之后该任务仍在同一根因下继续投递未经充分核验的地缘叙述。
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

系统在外部搜索出现额度失败告警的同一轮，仍将高风险地缘叙述当作事实发送给用户，并把它们作为油价变化的主因。09:01 和 12:00 复核显示，即使输出不再复述 06:02 的完整战争/停火叙事，也仍继续把单一政治讲话、布伦特冲高、OPEC+ 与地缘风险包装成确定性驱动并送达。15:01 复核进一步显示，即便正文缩短到价格播报，模型仍会在没有可追溯来源的情况下把“地缘政治紧张局势及市场需求预期变化”写成确定性原因。18:01 复核又把“全球产量下降”“伊朗停火协议未能安抚市场”等宏观叙事写成近期驱动因素并送达。21:00 复核继续把 OPEC+ 延长创纪录减产、区域冲突扰动供应链和 IEA 2026 供给过剩预测作为确定性原因送达。2026-04-23 00:00 复核又把霍尔木兹枪击、美国扣押伊朗船只和美伊停火到期写成确定性近期大涨原因；03:00 复核继续把美伊冲突升级、霍尔木兹实际关闭、WTI 单日飙升超 12% 等未核验叙述写成价格大涨主因；06:00 复核再次把美国封锁伊朗港口、霍尔木兹恢复商业航运和美伊局势当作价格波动主因送达；09:01 复核又把“伊朗停火协议扩展不确定性”和“中东供应扰动”写成涨价原因；12:00 复核在承认没有精确实时价格的情况下，仍把 WTI 约 99-101 美元、布伦特 101.91 美元和航运/伊朗谈判风险作为确定性播报送达；18:00 最新复核又把“霍尔木兹海峡持续受阻”“主要产油国出口受影响”“油价可能突破150美元”“特朗普设定海峡开放最后期限”写成近期主因并成功送达，说明缺陷没有自然收口。该任务并非“美伊战争专题”，而是原油价格播报；当前输出把未经足够验证的宏观事件链混入投资相关定时提醒。

## 用户影响

用户会收到已经送达的市场播报，并可能据此判断原油、通胀、成长股和组合风险。由于这是投资相关定时提醒的事实正确性问题，影响的是用户对市场变量的判断，而不仅是表达风格或格式质量，因此按业务正确性缺陷定级为 P2。

## 根因判断

初步判断是 heartbeat 内容生成链路缺少事实置信度和来源质量门禁：外部检索出现配额失败时，模型仍可沿用历史上下文、旧摘要或低质量搜索片段生成确定性事件叙述；调度器只按 `JsonTriggered` 发送，不校验事实来源、时间口径和任务主题是否足够匹配。

## 下一步建议

- 为市场播报类 heartbeat 增加来源质量检查：搜索配额失败或来源不足时，不允许输出确定性宏观事件链。
- 原油价格任务应把“价格数据”和“原因解释”拆成结构化字段，并为原因附来源时间；缺少来源时降级为低置信度说明。
- 检查历史 compact summary 是否把“美伊战争 / 霍尔木兹”叙述长期带入市场类 prompt，避免旧上下文污染后续定时播报。
