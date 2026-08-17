# Bug: Web heartbeat deliver 外露富文本 CSS 片段

## 发现时间

2026-08-18 06:01 CST

## Bug Type

Business Error

## 严重等级

P3

## 状态

New

## GitHub Issue

无，非 P1。

## 证据来源

- `data/logs/hone-console-page-source.log`
  - 巡检窗口：2026-08-18 02:03-06:01 CST（UTC 2026-08-17 18:03-22:01）。
  - `2026-08-17T18:30:45Z` / 02:30 CST，`job_id=j_b95a8df6`、`job=持仓重大事件心跳提醒`、`target=web-user-d415e2c11ced`。
  - 同轮 `HeartbeatDiag deliver` 记录 `parse_kind=PlainTextTriggered`，`deliver_preview` 开头出现 `` `drift` 0px `weight` 400; font-size: 13px; line-height: 1.6;">``，随后才进入“数据时间：北京时间 2026-08-18 02:30；行情口径...”正文。
  - 同窗 Web heartbeat 仍有其它正常 run / deliver；没有错投对象、全渠道不可用、敏感凭据泄露、panic 或本机绝对路径外泄证据。

## 端到端链路

1. Web heartbeat 定时任务到点触发 `持仓重大事件心跳提醒`。
2. 模型输出或上游渲染中产生富文本 / inline style 残片。
3. Scheduler 将结果归类为 `PlainTextTriggered` 并进入 deliver 候选。
4. 出站清理层没有剥离该 CSS 残片，导致用户可见正文开头被样式片段污染。

## 期望效果

- Heartbeat deliver 正文只应包含用户可读的监控结论、时间口径、行情口径和触发事实。
- CSS、HTML style、渲染属性、组件残片和富文本实现细节不应出现在用户可见消息中。
- 若上游生成了无法安全清理的富文本残片，应在送达前剥离或改写成纯文本。

## 当前实现效果

- `deliver_preview` 开头直接保留了 CSS / 富文本片段，破坏消息首屏可读性。
- 该样本仍完成 heartbeat 执行与送达候选生成，问题集中在用户可见输出净化层。

## 用户影响

- 这是质量性 bug，不是功能性 bug。
- 用户可能在 heartbeat 提醒开头看到渲染实现细节，降低消息可信度与可读性。
- 为何不影响功能链路，因此定级为 P3：本轮没有证据显示调度未触发、投递失败、错对象发送、数据破坏、敏感信息泄露或全渠道不可用；主问题是单条 Web heartbeat 用户可见格式污染。

## 根因判断

- 直接根因是 heartbeat deliver 出站净化没有覆盖富文本 style / CSS 残片。
- 与 `web_heartbeat_raw_tool_call_tag_leak.md`、`scheduler_heartbeat_trigger_json_payload_leak.md` 同属 Web heartbeat 出站结构污染族，但本缺陷的污染形态是 CSS / 富文本残片，不是 raw tool-call tag 或 fenced JSON 协议载荷，因此单独登记。

## 下一步建议

1. 在共享用户可见净化层增加 HTML style / CSS fragment 剥离规则，覆盖 `font-size`、`line-height`、`weight`、`drift` 等残片形态。
2. 为 scheduler delivery text 增加回归：正文前缀含 CSS / inline style 时，最终用户可见文本必须从业务正文开始。
3. 复核 Web heartbeat 渲染链路，确认是模型生成、Markdown/HTML 转换还是卡片降级过程中产生该残片。
