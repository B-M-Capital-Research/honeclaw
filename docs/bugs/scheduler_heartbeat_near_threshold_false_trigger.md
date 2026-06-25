# Bug: 单标的 heartbeat near-threshold guard 会误判触发状态并导致误发或漏发

- **发现时间**: 2026-04-29 10:03 CST
- **Bug Type**: Business Error
- **严重等级**: P2
- **状态**: New

## 最新进展（2026-06-25 19:01 CST）

- 本轮 2026-06-25 15:01-19:01 CST 继续确认同根复发，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-06-25`
    - 17:30 CST `小米30港元破位预警` `job_id=j_654aef9b` 返回 `PlainTextSuppressed + execution_failed`，raw preview 明确写出 `22.30 HKD` 低于 `30 HKD`，需要发送强提醒，最终 Feishu 记录本轮不发送。
    - 18:00 CST 同 job 返回 `JsonTriggered`，raw preview 写出 `22.30 HKD (<= 30 HKD -> alert triggered)`，随后仍记录 `心跳任务未命中，本轮不发送`。
    - 18:30 CST 同 job 再次 `JsonTriggered`，raw preview 写出 `22.3 HKD` 远低于 30 HKD 且条件触发，随后仍记录 `心跳任务未命中，本轮不发送`。
    - 19:00 CST 同 job 生成 `JsonTriggered + deliver_preview`，正文明确当前价格 22.30 港元已触及 30 港元心理止损 / 观察线，说明同一条件在失败、未命中和送达之间继续漂移。
  - 判断：
    - 本窗坏态继续表现为同一条件在 `PlainTextSuppressed`、`JsonTriggered`、未命中分支与偶发送达之间漂移，triggered 结果到投递分支之间仍不稳定。
    - 这是功能性 heartbeat 漏发 / 状态消费问题；影响集中在单个 heartbeat job，没有错对象投递、数据安全或全渠道不可用证据，严重等级维持 `P2`，非 P1，不创建 GitHub Issue。

## 最新进展（2026-06-25 11:01 CST）

- 本轮 2026-06-25 07:04-11:01 CST 继续确认同根复发，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-06-25`
    - 08:00 CST `小米30港元破位预警` `job_id=j_654aef9b` 返回 `PlainTextSuppressed + execution_failed`，raw preview 明确写出 `22.96 HKD` 低于 `30 HKD`，需要发送强提醒，最终 Feishu 记录本轮不发送。
    - 08:30 / 09:00 CST 同 job 继续 `PlainTextSuppressed + execution_failed`；raw preview 分别写出 `22.96 HKD` 低于 30 HKD、条件已触发，并生成自然语言提醒正文，最终仍未发送。
    - 09:30 CST 同 job 返回 `JsonTriggered`，raw preview 写出 `22.7 HKD` 低于 30 HKD，但随后仍记录 `心跳任务未命中，本轮不发送`。
    - 10:00 CST 同 job 再次 `JsonTriggered`，raw preview 写出 `22.16 HKD` 低于 30 HKD，随后仍记录 `心跳任务未命中，本轮不发送`。
    - 10:30 / 11:00 CST 同 job 回到 `PlainTextSuppressed + execution_failed`；raw preview 明确 `22.00 HKD` 低于 30 HKD 并生成强提醒正文，最终未发送。
  - 判断：
    - 本窗坏态继续表现为同一条件在 `PlainTextSuppressed`、`JsonTriggered` 与未命中分支之间漂移，且 triggered 结果到投递分支之间仍可能被未命中分支压制。
    - 这是功能性 heartbeat 漏发 / 状态消费问题；影响集中在单个 heartbeat job，没有错对象投递、数据安全或全渠道不可用证据，严重等级维持 `P2`，非 P1，不创建 GitHub Issue。

## 最新进展（2026-06-25 03:04 CST）

- 本轮 2026-06-24 23:02-2026-06-25 03:04 CST 继续确认同根复发，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-06-24`
    - 23:30 CST `小米30港元破位预警` `job_id=j_654aef9b` 返回 `PlainTextSuppressed + execution_failed`，raw preview 明确写出 `22.96 港元` 等于或低于 `30 港元` 心理止损线，需要立即发送强烈提醒，最终 Feishu 记录本轮不发送。
    - 00:00 CST 同 job 返回 `JsonTriggered` 并生成 `deliver_preview`，正文明确 `22.96 港元` 已跌破 30 港元阈值。
    - 00:30 / 01:00 / 02:00 / 03:00 CST 同 job 多次退化为 `PlainTextSuppressed`，raw preview 仍明确 `22.96 HKD <= 30 HKD` 或需要强提醒。
    - 01:30 CST 同 job 再次 `JsonTriggered + deliver_preview`；02:30 CST 又返回 `JsonNoop`，理由是当前价格与上一条提醒相同、没有新增新闻。
  - 判断：
    - 本窗坏态不再表现为全窗完全无送达预览；但同一触发条件仍在非结构化失败、`JsonTriggered`、`JsonNoop` 与送达预览之间漂移，说明 triggered 结果消费、重复抑制和结构化收口仍不稳定。
    - 这是功能性 heartbeat 漏发 / 状态消费问题；影响集中在单个 heartbeat job，没有错对象投递、数据安全或全渠道不可用证据，严重等级维持 `P2`，非 P1，不创建 GitHub Issue。

## 最新进展（2026-06-24 23:02 CST）

- 本轮 2026-06-24 19:00-23:02 CST 继续确认同根复发，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-06-24`
    - 19:30 CST `小米30港元破位预警` `job_id=j_654aef9b` 返回 `PlainTextSuppressed + execution_failed`，raw preview 明确写出 `22.96 HKD` 低于 `30 HKD` 且应该触发强提醒，最终 Feishu 记录本轮不发送。
    - 20:00 CST 同 job 再次 `PlainTextSuppressed + execution_failed`，raw preview 写出 `22.96 <= 30`、满足触发条件并生成提醒正文，最终未发送。
    - 20:30 CST 同 job 返回 `JsonTriggered`，raw preview 明确 `22.96 港元 < 30 港元`，随后仍记录 `心跳任务未命中，本轮不发送`。
    - 21:30 CST 同 job 退化为 `JsonMalformed + execution_failed`，raw preview 仍包含 `status:"triggered"` 与 `current_price:22.96`。
    - 22:00 CST 同 job 再次 `JsonTriggered`，raw preview 写出 `22.96 HKD` 低于 30 HKD，但本轮仍未稳定送达。
  - 判断：
    - 本窗坏态继续表现为同一条件在 `PlainTextSuppressed`、`JsonTriggered`、`JsonMalformed` 与 `JsonNoop` 间漂移，且 triggered 结果到投递分支之间仍可能被未命中分支压制。
    - 这是功能性 heartbeat 漏发 / 状态消费问题；影响集中在单个 heartbeat job，没有错对象投递、数据安全或全渠道不可用证据，严重等级维持 `P2`，非 P1，不创建 GitHub Issue。

## 最新进展（2026-06-24 19:01 CST）

- 本轮 2026-06-24 15:01-19:01 CST 继续确认同根复发，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-06-24`
    - 16:00 CST `小米30港元破位预警` `job_id=j_654aef9b` 返回 `parse_kind=JsonTriggered`，raw preview 明确写出 Xiaomi `22.96 HKD` 低于 `30 HKD`，用户条件应立即强提醒；随后 Feishu 仍记录 `心跳任务未命中，本轮不发送`。
    - 16:30 CST 同 job 退化为 `PlainTextSuppressed + execution_failed`，raw preview 仍明确 `22.96 HKD` 低于 30 港元阈值并需要发送强提醒。
    - 17:30 CST 同 job 返回 `JsonNoop`，但 raw preview 错写 `22.96 港元，高于 30 港元心理止损/观察线`，与数值关系相反，最终仍记录未发送。
    - 18:00 CST 同 job 生成 `JsonTriggered + deliver_preview`，正文明确 `22.96 港元` 已跌破 30 港元；18:30 CST 又回到 `JsonTriggered` 后未发送；19:00 CST 再次 `JsonTriggered + deliver_preview`，随后被 `duplicate_suppressed` 并记录未发送。
  - 判断：
    - 本窗坏态继续表现为同一条件在 `JsonTriggered`、`PlainTextSuppressed` 与 `JsonNoop` 间漂移，且 triggered 结果到投递分支之间仍可能被未命中 / duplicate 分支压制。
    - 这是功能性 heartbeat 漏发 / 状态消费问题；影响集中在单个 heartbeat job，没有错对象投递、数据安全或全渠道不可用证据，严重等级维持 `P2`，非 P1，不创建 GitHub Issue。

## 最新进展（2026-06-24 07:02 CST）

- 本轮 2026-06-24 03:02-07:02 CST 继续确认同根复发，状态维持 `New`：
  - `data/runtime/logs/hone_cli_screen.log`
    - 04:30 CST `小米30港元破位预警` `job_id=j_654aef9b` 返回 `parse_kind=JsonTriggered`，raw preview 明确写出小米集团当前价格 `22.62 港元`，低于 `30 港元`阈值，需要触发强烈提醒；随后仍记录 `心跳任务未命中，本轮不发送`。
    - 05:30 CST 同 job 再次 `JsonTriggered`，raw preview 继续明确 `22.62 HKD < 30 HKD`，但最终仍未发送。
    - 06:30 CST 同 job `JsonTriggered` 并生成 `deliver_preview`，正文写出 `现价22.62港元，已触及30港元心理止损/观察线`；随后仍记录 `心跳任务未命中，本轮不发送`。
  - 判断：
    - 本窗坏态继续表现为 triggered 结果到投递分支之间被未命中分支压制，而不是数据未取到或模型未识别阈值。
    - 这是功能性 heartbeat 漏发 / 状态消费问题；影响集中在单个 heartbeat job，没有错对象投递、数据安全或全渠道不可用证据，严重等级维持 `P2`，非 P1，不创建 GitHub Issue。

## 最新进展（2026-06-24 03:03 CST）

- 本轮 2026-06-23 23:02-2026-06-24 03:00 CST 继续确认同根复发，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-06-23`
    - 00:00 CST `小米30港元破位预警` `job_id=j_654aef9b` 返回 `parse_kind=JsonTriggered` 并生成 `deliver_preview`，正文明确 `现价：22.62 港元`、`已触及 30 港元心理止损/观察线`；随后记录 `duplicate_suppressed` 与 `心跳任务未命中，本轮不发送`。
    - 02:30 CST 同 job 再次 `JsonTriggered + deliver_preview`，正文明确 `现价22.62港元，已触及30港元心理止损/观察线`；随后仍被 `duplicate_suppressed` 压成不发送。
    - 03:00 CST 同 job 继续 `JsonTriggered + deliver_preview`，正文明确 `现价22.62港元，已显著低于30港元心理止损/观察线`，随后再次 `duplicate_suppressed` 并记录未发送。
  - 判断：
    - 本窗坏态继续表现为 triggered 结果到投递分支之间被 duplicate / 未命中压制，而不是数据未取到或模型未识别阈值。
    - 这是功能性 heartbeat 漏发 / 状态消费问题；影响集中在单个 heartbeat job，没有错对象投递、数据安全或全渠道不可用证据，严重等级维持 `P2`，非 P1，不创建 GitHub Issue。

## 最新进展（2026-06-23 23:02 CST）

- 本轮 2026-06-23 19:02-23:02 CST 继续确认同根复发，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-06-23`
    - 19:30 CST `小米30港元破位预警` `job_id=j_654aef9b` 返回 `parse_kind=JsonTriggered`，raw preview 明确写出当前价格 `22.62港元`、`≤30港元`、`已触及心理止损/观察线`，随后仍记录 `心跳任务未命中，本轮不发送`。
    - 20:00 / 21:00 / 21:30 / 22:00 / 23:00 CST 同 job 多次 `JsonTriggered` 并生成或接近生成 `deliver_preview`，正文明确 `22.62 HKD < 30 HKD`、`已大幅跌破 30 港元心理止损/观察线`；随后被 `duplicate_suppressed` 或未命中分支压成不发送。
    - 20:30 / 22:30 CST 同 job 退化为 `JsonNoop`，raw preview 仍先判断当前价格低于 30 港元，但又以近期已提醒为由不发送。
  - 判断：
    - 本窗坏态继续表现为 triggered 结果到投递分支之间被 duplicate / 未命中压制，且同一条件在半小时窗口中在 `JsonTriggered` 与 `JsonNoop` 之间漂移。
    - 这是功能性 heartbeat 漏发 / 状态消费问题；影响集中在单个 heartbeat job，没有错对象投递、数据安全或全渠道不可用证据，严重等级维持 `P2`，非 P1，不创建 GitHub Issue。

## 最新进展（2026-06-23 19:03 CST）

- 本轮 2026-06-23 15:02-19:02 CST 继续确认同根复发，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-06-23`
    - 16:00 CST `小米30港元破位预警` `job_id=j_654aef9b` 返回 `parse_kind=JsonTriggered`，raw preview 明确写出当前价格 `22.66 HKD` 低于 `30 HKD`，并生成 `deliver_preview`：`现价 22.66 港元，已大幅跌破 30 港元心理止损/观察线`；随后记录 `duplicate_suppressed` 与 `心跳任务未命中，本轮不发送`。
    - 17:00 CST 同 job 再次 `JsonTriggered + deliver_preview`，正文明确 `现价：22.62 港元` 且大幅低于 30 港元；随后仍被 `duplicate_suppressed` 并记录未发送。
    - 16:30 / 18:00 / 19:00 CST 同 job raw preview 继续明确 `22.62/23.72 HKD < 30 HKD` 并给出触发提醒正文，但因 `PlainTextSuppressed` / 非结构化输出落成 `execution_failed`，最终不发送。
  - 判断：
    - 本窗坏态一部分表现为 duplicate / 送达前未命中分支压制 `JsonTriggered`，另一部分表现为 heartbeat 结构化退化；两者共同造成同一用户条件提醒未稳定送达。
    - 这是功能性 heartbeat 漏发 / 状态消费问题；影响集中在单个 heartbeat job，没有错对象投递、数据安全或全渠道不可用证据，严重等级维持 `P2`，非 P1，不创建 GitHub Issue。

## 最新进展（2026-06-23 15:02 CST）

- 本轮 2026-06-23 11:02-15:02 CST 继续确认同根复发，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-06-23`
    - 14:30 CST `小米30港元破位预警` `job_id=j_654aef9b` 返回 `parse_kind=PlainTextSuppressed`，raw preview 明确写出 `小米集团（1810.HK）现价 22.66 港元，低于 30 港元心理止损/观察线`，但最终仍记录 `心跳任务未命中，本轮不发送`。
    - 15:00 CST 同 job 返回 `parse_kind=JsonTriggered`，生成 `deliver_preview`，正文明确 `现价22.62港元，已跌破30港元心理止损/观察线`，随后 Feishu 仍记录 `心跳任务未命中，本轮不发送`。
  - 同窗 heartbeat 仍有多条 `JsonTriggered` 后未送达样本（AAOI、ORCL、ASTS 等），但小米 30 港元破位仍是最稳定复现锚点。
- 用户影响：
  - 用户设定的小米 30 港元破位条件继续被模型明确判定触发并生成送达正文，但最终投递分支仍可能落成未命中。
  - 这是功能性 heartbeat 漏发 / 状态消费问题；没有错对象投递、数据安全或全渠道不可用证据，严重等级维持 `P2`，非 P1，不创建 GitHub Issue。

## 最新进展（2026-06-23 11:02 CST）

- 本轮 2026-06-23 07:02-11:02 CST 继续确认同根复发，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-06-23`
    - 08:00 CST `小米30港元破位预警` `job_id=j_654aef9b` 返回 `parse_kind=JsonTriggered`，raw preview 明确写出 `price is 23.72 HKD, which is below 30 HKD`，随后 Feishu 仍记录 `心跳任务未命中，本轮不发送`。
    - 08:30 CST 同 job 再次 `JsonTriggered`，raw preview 写出 `现价23.72港元 **远低于** 30港元的触发线（23.72 < 30）` 与 `条件已触发`，最终仍未发送。
    - 11:00 CST 同 job 继续 `JsonTriggered`，raw preview 写出 `小米（1810.HK）最新价格：23港元`、`现价23港元远低于30港元的触发条件，因此应该触发提醒`，随后仍记录 `心跳任务未命中，本轮不发送`。
  - 同窗 `data/runtime/logs/acp-events.log` JSON 事件统计可见 33 次 `session/prompt`、21 个 session、33 次 `stopReason=end_turn`、0 个 response error；`agent_message_chunk` 用户可见流未见空回复、错投、原始工具 JSON、本机绝对路径、provider 原始错误或思维痕迹。
- 用户影响：
  - 用户设置的小米 30 港元破位条件仍被模型明确判定为触发，但最终没有稳定投递。
  - 这是功能性 heartbeat 漏发 / 状态消费问题；影响集中在 heartbeat triggered 结果到 Feishu 发送之间，没有错对象投递、数据安全或全渠道不可用证据，严重等级维持 `P2`，非 P1，不创建 GitHub Issue。

## 最新进展（2026-06-23 07:06 CST）

- 本轮 2026-06-23 03:04-07:02 CST 继续确认同根复发，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-06-22`
    - 03:30 CST `小米30港元破位预警` `job_id=j_654aef9b` 返回 `parse_kind=JsonTriggered`，raw preview 明确写出 `现价：23.72 港元`、`23.72 < 30` 这类触发语义，随后 Feishu 仍记录 `心跳任务未命中，本轮不发送`。
    - 04:00 / 04:30 / 05:00 / 06:00 CST 同 job 多次退化为 `PlainTextSuppressed + execution_failed`；raw preview 仍明确写出当前价格 `23.72 港元` 低于 `30 港元`，并生成自然语言触发提醒正文。
    - 05:30 / 06:30 CST 同 job 再次 `JsonTriggered`，raw preview 继续写出当前价格低于 30 港元阈值；06:30 最终仍记录 `心跳任务未命中，本轮不发送`。
    - 07:00 CST 同 job `JsonTriggered` 且生成 `deliver_preview`，正文明确 `当前价格：23.72港元`、`已触及30港元心理止损/观察线`；随后被 `duplicate_suppressed` 命中并继续记录 `心跳任务未命中，本轮不发送`。
  - 同窗 `data/runtime/logs/acp-events.log` 文本扫描可见 8 次 `session/prompt`、8 次 `stopReason=end_turn`、0 个 response error；Feishu / Web direct final 正常收口，未见 P1 级错投、全链路不可用或敏感原始错误外泄。
- 用户影响：
  - 用户设置的小米 30 港元破位条件仍在多个半小时窗口被判定为触发，甚至生成送达正文，但最终未稳定投递。
  - 这是功能性 heartbeat 漏发 / 状态消费问题；影响集中在 heartbeat triggered 结果到 Feishu 发送之间，没有错对象投递、数据安全或全渠道不可用证据，严重等级维持 `P2`，非 P1，不创建 GitHub Issue。

## 最新进展（2026-06-22 23:03 CST）

- 本轮 2026-06-22 23:03-2026-06-23 03:02 CST 继续确认同根复发，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-06-22`
    - 23:30 CST `小米30港元破位预警` `job_id=j_654aef9b` 返回 `parse_kind=JsonTriggered`，raw preview 明确写出 `现价：23.72 港元`、`现价 23.72 港元 <= 30 港元？YES，23.72 < 30，条件触发`，随后 Feishu 仍记录 `心跳任务未命中，本轮不发送`。
    - 00:00 CST 同 job 再次 `JsonTriggered`，raw preview 写出 `触发事实：小米（1810.HK）现价 23.72 港元，触及 <=30 港元心理止损/观察线`，最终仍未发送。
    - 00:30 CST 同 job 生成 `deliver_preview`，正文明确 `现价：23.72 港元（-3.50%，日低 23.52 港元创年内新低）`，但随后仍记录 `心跳任务未命中，本轮不发送`。
    - 01:30 CST 同 job 退化为 `JsonMalformed + execution_failed`；raw preview 仍写出 `小米当前价格为23.72港元，明显低于30港元阈值` 与 `status:"triggered"`。
    - 02:30 / 03:00 CST 同 job 继续 `JsonTriggered` 且 raw preview 明确 `当前价格 23.72港元`、`低于30港元`、`应该触发提醒`，最终仍记录未发送。
  - 同窗还观察到 AAOI / ORCL heartbeat triggered 后未发送样本，但与本单同属 heartbeat triggered 结果到投递分支之间的漏发 / 抑制问题，不新建重复缺陷。
- 用户影响：
  - 用户设定的小米 30 港元破位条件在多个半小时窗口被模型明确判定为触发，甚至生成送达正文，但最终没有投递。
  - 这仍是功能性 heartbeat 漏发，影响监控提醒正确性；没有错对象投递、数据安全或全渠道不可用证据，严重等级维持 `P2`，非 P1，不创建 GitHub Issue。

- 本轮 19:00-23:03 CST 继续确认同根复发，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-06-22`
    - 19:00 CST `小米30港元破位预警` `job_id=j_654aef9b` 返回 `parse_kind=JsonTriggered`，raw preview 明确写出 `当前价格：23.72 港元`、`触发阈值：≤ 30 港元`、`状态：已触发（23.72 < 30）`，随后 Feishu 仍记录 `心跳任务未命中，本轮不发送`。
    - 19:30 CST 同 job 退化为 `parse_kind=JsonMalformed + execution_failed`；raw preview 仍包含 `status:"triggered"`、`price:23.72`、`threshold:30.00`、`direction:"below_threshold"`，但最终记录 `heartbeat 输出不是合法 JSON，任务已标记失败`。
    - 20:00 / 20:30 CST 同 job 生成 `deliver_preview`，正文明确 `现价：23.72 港元` 并写出当前价格已低于 30 港元观察线；20:30 后仍记录 `心跳任务未命中，本轮不发送`。
    - 21:00 CST 同 job 再次返回 `JsonTriggered`，raw preview 写出 `价格23.72港元，低于30港元阈值，需要发送提醒`，最终仍记录 `心跳任务未命中，本轮不发送`。
  - 同窗统计：`JsonTriggered` 22 条、`heartbeat 输出不是结构化 JSON` 75 条、`JsonMalformed` 8 条、`心跳任务未命中` 146 条；ACP 同窗 47 次 `stopReason=end_turn`、0 个 response error，未见 Feishu 400、panic、transport disconnect 或 P1 级错投 / 全链路不可用证据。
- 用户影响：
  - 这仍是功能性 heartbeat 漏发 / 状态消费问题：模型多次判断低于阈值且可生成送达正文，但最终投递分支仍可能落成未命中或失败。
  - 影响集中在单任务 heartbeat triggered 结果到 Feishu 发送之间的判定链路；严重等级维持 `P2`，非 P1，不创建 GitHub Issue。

## 最新进展（2026-06-22 15:04 CST）

- 本轮 15:04-19:00 CST 同根缺陷继续复发，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-06-22`
    - 16:00 CST `小米30港元破位预警` `job_id=j_654aef9b` 返回 `parse_kind=JsonTriggered`，raw preview 明确写出 `现价：23.72 港元`、`用户设定条件：现价 ≤ 30港元`、`当前价格 23.72 港元 < 30 港元，已触发提醒条件`，随后 Feishu 仍记录 `心跳任务未命中，本轮不发送`。
    - 17:00 / 17:30 / 18:00 / 18:30 / 19:00 CST 同 job 继续多次 `JsonTriggered`，raw preview 分别写出 `23.72 港元` 低于 `30 港元`、`条件已触发`、`23.72 < 30` 等触发语义，但最终均落入未发送分支。
    - 15:30 / 16:30 CST 同 job 还出现 `PlainTextSuppressed`，raw preview 同样可见 23.72 港元低于 30 港元的判断；这些样本归入 heartbeat 结构化退化缺陷，不单独新建文档。
  - 同窗 `data/runtime/logs/acp-events.log` 可重构 5 次 `session/prompt`、5 次 `stopReason=end_turn`、0 个 response error；用户可见 final 未见空回复、错投、投递失败、原始工具 JSON、本机绝对路径、transport trace 或思维痕迹。
- 用户影响：
  - 该问题仍是功能性 heartbeat 漏发：模型已明确判定价格阈值触发，但最终未送达提醒。
  - 影响范围集中在单个 heartbeat job 的 triggered 结果到 Feishu 发送之间；没有错对象投递、数据安全或全渠道不可用证据，严重等级维持 `P2`，非 P1，不创建 GitHub Issue。

- 本轮 11:02-15:04 CST 确认同根缺陷在当前 web runtime 进程中继续复发，状态从 `Fixed` 回退为 `New`：
  - `data/runtime/logs/web.log.2026-06-22`
    - 10:30 CST 后日志出现 schema migration / cloud table 初始化信息，说明 web runtime 已重新加载当前服务进程；后续样本不再按 11:03 巡检中的“未确认部署运行态”处理。
    - 12:00 CST `小米30港元破位预警` `job_id=j_654aef9b` 返回 `parse_kind=JsonTriggered`，raw preview 明确写出 `当前价格：23.76 港元`、`触发条件：≤ 30 港元`、`23.76 < 30`，随后 Feishu 仍记录 `心跳任务未命中，本轮不发送`。
    - 12:30 / 13:00 / 13:30 / 14:00 / 14:30 / 15:00 CST 同 job 继续多次 `JsonTriggered`，raw preview 分别写出 `23.68 HKD`、`23.58港元`、`23.6 港元`、`23.9 港元`、`23.72港元` 等低于 30 港元阈值的触发条件，但最终均落入未发送分支。
  - 同窗 ACP 可重构 3 次 `session/prompt`、3 次 `stopReason=end_turn`、0 个 response error；用户可见 final 未见空回复、错投、原始工具 JSON、本机绝对路径、transport trace 或思维痕迹。
- 用户影响：
  - 这是功能性 heartbeat 漏发：模型已明确判定用户设置的价格阈值触发，但最终未送达提醒。
  - 影响集中在单任务 heartbeat triggered 结果到 Feishu 发送之间的判定链路；没有错对象投递、数据安全或全渠道不可用证据，严重等级维持 `P2`，非 P1，不创建 GitHub Issue。

## 修复记录（2026-06-21 23:07 CST）

- 本轮修复 2026-06-21 19:03 回退样本中“真实下穿阈值仍被送达前 guard 压成未命中”的代码边界：
  - `heartbeat_near_threshold_without_crossing(...)` 新增明确下穿识别：当用户可见 triggered message 中有可解析的当前价与阈值，且正文写明 `当前价 <= 阈值`、`低于 / 跌破 / below / under` 且数值上 `current <= threshold` 时，不再进入 near-threshold 抑制。
  - 继续保留既有反向保护：`当前价 > 触发价` 却声称“已低于触发价”、`接近但未达`、`未触发 / 未超过 / 未触及` 等文案仍会被抑制。
  - 新增回归 `heartbeat_explicit_lower_price_crossing_is_not_near_threshold_suppressed`，覆盖 `小米30港元破位预警` 的 `24.58 <= 30` 中文样本和 `latest price is HKD 24.58 ... below 30 HKD` 英文样本。
- 验证：
  - `cargo test -p hone-channels heartbeat_ --lib -- --nocapture`
- 无关联 GitHub Issue；当前按本地代码和回归验证更新为 `Fixed`，未依赖当前机器生产日志、线上渠道状态或 live 服务重启复核。

## 修复结论复核（2026-06-22 07:08 CST）

- 本轮不再用当前机器旧运行态 / 未确认部署进程作为重新打开依据，只按当前仓库代码和本地回归验证复核。
- 当前代码已覆盖 2026-06-22 03:02 补证里的小米 30 港元破位样本：
  - `heartbeat_explicit_lower_price_crossing_is_not_near_threshold_suppressed` 覆盖中文 `24.58 <= 30` 与英文 `HKD 24.58 ... below 30 HKD` triggered 文案。
  - 回归确认当前不会写入 `near_threshold_suppressed=true`，`should_deliver` 保持 `true`。
- 验证：
  - `cargo test -p hone-channels heartbeat_explicit_lower_price_crossing_is_not_near_threshold_suppressed --lib -- --nocapture`
- 结论：状态维持代码级 `Fixed`；后续只有在确认运行当前代码后，仍有本地可复现或代码路径证据证明 triggered alert 被送达前 guard 抑制时，才重新打开。

## 运行态观察（2026-06-22 11:03 CST）

- 本轮 07:02-11:02 CST 当前机器 live 日志仍可观察到旧形态，但未确认进程已加载 2026-06-22 07:08 CST 的当前代码修复；因此只补证为旧/未确认部署运行态观察，不把状态从 `Fixed` 回退：
  - `data/runtime/logs/web.log.2026-06-22`
    - 08:00 CST `小米30港元破位预警` `job_id=j_654aef9b` 返回 `parse_kind=JsonTriggered`，raw preview 明确写出 current price `~24.58 HKD` below `30 HKD` / condition triggered，随后 Feishu 仍记录 `心跳任务未命中，本轮不发送`。
    - 08:30 CST 同 job 再次 `parse_kind=JsonTriggered`，raw preview 写出 `24.58港元` 明显低于 `30港元`、触发条件已满足，随后仍记录未发送。
    - 10:30 CST 同 job 生成触发正文，raw preview 写出 `现价：23.62 港元`、`当前价格 23.62 港元 <= 30 港元，条件触发`，但因输出不是结构化 JSON 被标记 `execution_failed`，本轮不发送。
    - 11:00 CST 同 job 再次 `parse_kind=JsonTriggered`，raw preview 写出 current price `23.68 HKD` significantly below `30 HKD`，Feishu 仍记录 `心跳任务未命中，本轮不发送`。
  - 同窗统计：`JsonTriggered` 13 条、`heartbeat 输出不是结构化 JSON` 54 条、`JsonNoop` 86 条、`JsonUnknownStatus` 12 条、`JsonMalformed` 2 条；未见 Feishu 400、panic、transport disconnect、错对象投递或全渠道不可用证据。
- 处理结论：
  - 当前代码已有定向回归覆盖本轮小米破位语义，状态继续保持 `Fixed`。
  - 若确认已部署 / 重启到当前代码后仍有同样 triggered alert 被送达前 guard 抑制，再重新打开；本轮非 P1，不创建 GitHub Issue。

## 最新进展（2026-06-22 03:02 CST）

- 本轮 23:02-03:01 CST 真实运行态继续确认同根复发，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-06-21`
    - 23:30 CST `小米30港元破位预警` `job_id=j_654aef9b` 返回 `parse_kind=JsonTriggered`，raw preview 明确写出 latest price `24.58 HKD` below `30 HKD`，并生成 `status=triggered`。
    - 同一窗口紧接着出现 deliver preview：`【强烈提醒】小米（1810.HK）现价 24.58 港元... 当前价格已远低于30港元心理观察线...`，说明触发提醒正文已生成。
    - Feishu 随后仍记录 `心跳任务未命中，本轮不发送: job=小米30港元破位预警`，用户未收到该 triggered alert。
  - 同窗统计：`parse_kind=JsonTriggered` 18 条、`heartbeat 输出不是结构化 JSON` 64 条、`JsonNoop` 99 条、`JsonUnknownStatus` 16 条、`JsonMalformed` 6 条；未见 Feishu 400、panic、transport disconnect 或 P1 级错投 / 全链路不可用证据。
- 用户影响：
  - 这仍是功能性 heartbeat 漏发：模型和 deliver preview 均已确认低于阈值，但最终投递分支仍被压成未命中。
  - 影响集中在单任务 heartbeat triggered 结果到 Feishu 发送之间的判定链路；严重等级维持 `P2`，非 P1，不创建 GitHub Issue。

## 最新进展（2026-06-21 23:03 CST）

- 本轮 19:02-23:01 CST 真实运行态继续确认同根复发，状态维持 `New`：
  - `data/runtime/logs/web.log.2026-06-21`
    - 19:30 / 20:00 / 20:30 / 21:00 / 21:30 / 22:00 / 22:30 / 23:00 CST `小米30港元破位预警` 多次返回 `parse_kind=JsonTriggered`，raw preview 或 deliver preview 明确写出 `24.58 < 30`、`24.58 <= 30`、`condition=现价≤30港元`、`status=triggered` 等触发条件；调度随后仍记录 `心跳任务未命中，本轮不发送`。
    - 21:01 / 23:00 CST `光迅科技关键事件心跳提醒` 返回 `JsonTriggered`，raw preview 写出价格 `266.2`、`up 10%`、`limit-up/涨停` 等触发语义；同一 target 后续仍落到跳过发送分支。
    - 23:00 CST `闪迪关键事件心跳提醒` 返回 `JsonTriggered`，raw preview 写出 `Current price: $2,184.75`、`up 11.54%`、`new all-time high` 等触发语义；同一窗口仍未形成用户可见送达。
  - 同窗统计：`parse_kind=JsonTriggered` 26 条、`心跳任务未命中` 128 条、`heartbeat 输出不是结构化 JSON` 71 条、ACP response error 0；未见 Feishu 400、panic、transport disconnect 或资源耗尽。
- 用户影响：
  - 这仍是功能性 heartbeat 漏发 / 抑制问题，不是文案质量问题；用户设置的价格或重大事件条件已由模型判为 triggered，但最终没有送达。
  - 影响集中在 heartbeat 判定到投递之间的链路；没有错对象投递、数据安全或全渠道不可用证据，严重等级维持 `P2`，非 P1，不创建 GitHub Issue。

## 最新进展（2026-06-21 19:03 CST）

- 本轮最近四小时真实运行态确认同根复发，状态从 `Fixed` 回退为 `New`：
  - `data/runtime/logs/web.log.2026-06-21`
    - 16:30 CST `小米30港元破位预警` `job_id=j_654aef9b` 返回 `parse_kind=JsonTriggered`，raw preview 明确写出 `当前价格：24.58 港元`、`现价 24.58 港元 <= 30 港元 -> 触发`，随后仍记录 `心跳任务未命中，本轮不发送`。
    - 17:30 CST 同 job 再次返回 `parse_kind=JsonTriggered`，raw preview 写出 `latest price is HKD 24.58, which is below 30 HKD` 和 `status: triggered`，随后仍记录未发送。
    - 18:00 / 18:30 / 19:00 CST 同 job 继续多次返回 `JsonTriggered`，raw preview 均明确低于 30 港元阈值，最终仍被压成“未命中，本轮不发送”。
  - 该样本与 2026-05-12 DRAM 创历史新高被送达前 guard 压成 `noop + skipped_noop` 的分支同根：模型已给出 `JsonTriggered` 且触发条件明确满足，但 scheduler 最终没有投递。
  - 代码对照显示 heartbeat 出站前仍会在 `JsonTriggered` 之后执行 `heartbeat_near_threshold_without_crossing(...)` 等送达前 guard；当前文案包含 `心理止损/观察线`、`52周新低`、`24.58 <= 30` 等混合表达，疑似被 guard 或后续送达判定误抑制。
- 用户影响：
  - 这是功能性告警漏发：用户设置的小米 30 港元破位条件已经满足，但连续多个半小时窗口没有收到提醒。
  - 它不涉及 P1 的错对象投递、数据安全或批量全链路不可用；当前按单任务 heartbeat 漏发定为 `P2 / New`，不创建 GitHub Issue。

## 修复结论复核（2026-05-12 11:16 CST）

- 本轮按当前自动化约束复核：当前机器旧运行态 / 未重启进程的 live 数据不再作为重新打开本单的依据。
- 当前仓库代码已覆盖 `DRAM 心跳监控` 创历史新高被 near-threshold guard 误抑制的关键条件：
  - `heartbeat_near_threshold_without_crossing(...)` 只拦截“接近 / 未达 / 未触及 / 未触发 / 未超过阈值”等否认越线语义，`盘中创历史新高（满足条件2）` 不属于 near-threshold 否认文本。
  - `heartbeat_execution_from_content(...)` 对 `DRAM 盘中创历史新高（满足条件2）` 保持 `should_deliver=true`，不会写入 `near_threshold_suppressed=true`。
- 本轮新增回归 `heartbeat_record_high_trigger_is_not_near_threshold_suppressed`，锁住 2026-05-12 11:00 CST 复发形态。
- 验证：
  - `cargo test -p hone-channels heartbeat_record_high_trigger_is_not_near_threshold_suppressed --lib -- --nocapture`
  - `cargo test -p hone-channels heartbeat_duplicate_preview_match_allows_dram_record_high_after_cerebras_ipo --lib -- --nocapture`
- 结论：本单维持 `Fixed`；后续只有在部署/重启到当前代码后，仍能用本地可复现测试或新代码路径证明真实创新高触发被 near-threshold guard 抑制时，才应重新打开。

## 证据来源

- `data/runtime/logs/web.log.2026-06-21`
  - 巡检窗口：2026-06-21 15:03-19:03 CST。
  - 16:30:30 CST `job_id=j_654aef9b job=小米30港元破位预警` 收口为 `parse_kind=JsonTriggered`，raw preview 明确写出 `当前价格：24.58 港元`、`现价 24.58 港元 <= 30 港元 -> 触发`；紧接着 Feishu 记录 `心跳任务未命中，本轮不发送`。
  - 17:30:18 CST 同 job 再次 `parse_kind=JsonTriggered`，raw preview 写出 `latest price is HKD 24.58, which is below 30 HKD` 与 `status: triggered`；随后仍未发送。
  - 18:00:31 / 18:30:27 / 19:00:31 CST 同 job 继续多次 `JsonTriggered`，raw preview 均围绕 `24.58` 低于 `30` 的阈值触发条件，最终仍记录未命中不发送。
  - 同窗普通 Feishu / Web direct 会话仍有 `end_turn` 收口；该问题集中在 heartbeat triggered 结果到最终送达的判定链路。
- `2026-05-12 11:02 CST` 本轮巡检把本单从 `Fixed` 回退为 `New`：最近四小时真实 heartbeat 窗口出现相反方向的坏态，模型已返回 `JsonTriggered` 且正文明确说 `DRAM 盘中创历史新高（满足条件2）`，但送达前 near-threshold guard 把它压成 `noop + skipped_noop`：
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=19216`
    - `job_name=DRAM 心跳监控`
    - `executed_at=2026-05-12T11:00:57.141816+08:00`
    - `execution_status=noop`
    - `message_send_status=skipped_noop`
    - `delivered=0`
    - `detail_json.parse_kind=JsonTriggered`
    - `detail_json.near_threshold_suppressed=true`
    - `detail_json.deliver_preview` 写明：`触发条件：DRAM 盘中创历史新高（满足条件2）`，并列出 `盘中最高 $56.38 = 上市以来历史最高价`。
  - `data/runtime/logs/sidecar.log`
    - `2026-05-12 11:00:57 CST` 记录 `DRAM 心跳监控` 先完成 `parse_kind=JsonTriggered` 与 `deliver_preview`，随后 Feishu scheduler 仍以“心跳任务未命中，本轮不发送”收口。
  - 对照同一任务 `09:01 / 09:30 / 10:01 / 10:31 CST` 窗口，DRAM 创新高触发还被跨 job duplicate suppression 漏发；`11:00 CST` 不再命中 duplicate，但又被 near-threshold guard 抑制，说明当前单标的送达前保险闸仍会在真实触发条件成立时造成漏发。
  - 结论：这仍属于单标的 heartbeat 的送达前阈值语义判断缺陷，只是从“接近阈值误发”扩展为“真实触发误抑制”；损害点是用户漏收本应送达的 DRAM 创历史新高提醒，因此维持功能性 `P2 / New`。

- `data/sessions.sqlite3` -> `cron_job_runs`
  - `run_id=12511`
  - `job_id=j_db12f27f`
  - `job_name=持仓重大事件心跳检测`
  - `executed_at=2026-05-01T15:02:33.037767+08:00`
  - `execution_status=completed`
  - `message_send_status=sent`
  - `delivered=1`
  - `response_preview` 明确写出：`RKLB当前$82.51，较前收盘价上涨+7.13%，突破5%阈值`
  - 同条正文继续把 `4月29日宣布获得1.9亿美元国防合同` 包装成“重大增量”，说明同一旧合同叙述已从单标的 `RKLB异动监控` 扩散到组合级 `持仓重大事件心跳检测`，并被错误升级成正式提醒。
- `data/runtime/logs/web.log.2026-05-01`
  - `2026-05-01 15:02:27.644` 记录同一 `job_id=j_db12f27f` 收口为 `parse_kind=JsonTriggered` 并执行 `deliver`。
  - 同窗 `2026-05-01 15:00:37.400` 的 `RKLB异动监控` 刚回落成 `parse_kind=Empty`，说明不是价格条件突然跨越了更高单标的阈值，而是旧事件背景被组合级 heartbeat 重新包装成“重大增量”。

- `data/sessions.sqlite3` -> `cron_job_runs`
  - `run_id=12420`
  - `job_id=j_1241aad0`
  - `job_name=RKLB异动监控`
  - `executed_at=2026-05-01T13:01:08.230303+08:00`
  - `execution_status=completed`
  - `message_send_status=sent`
  - `delivered=1`
  - `response_preview` 明确写出：`当前价 $82.51，单日涨跌幅 +7.13%（已接近但未达8%阈值）`
  - 同条正文继续把 `4月29日确认获得1.9亿美元国防合同` 包装成“重大订单利好”，说明同一根因在 `09:30` 之后并未消失，而是在 `13:00` 窗口再次把“未达阈值 + 旧消息背景”送成正式提醒。
- `data/runtime/logs/sidecar.log`
  - `2026-05-01 13:00:01.838` 记录同一 `job_id=j_1241aad0` heartbeat 启动。
  - `2026-05-01 13:00:47.850-13:01:08.230` 对应窗口最终落成 `completed + sent`，与 sqlite 中 `run_id=12420` 的正式送达一致。

- `data/sessions.sqlite3` -> `cron_job_runs`
  - `run_id=12263`
  - `job_id=j_1241aad0`
  - `job_name=RKLB异动监控`
  - `executed_at=2026-05-01T09:30:27.966052+08:00`
  - `execution_status=completed`
  - `message_send_status=sent`
  - `delivered=1`
  - `response_preview` 明确写出：`当前现货报价$82.51，相对昨收$77.02上涨7.13%，单日涨跌幅接近但未达8%阈值`
  - 同条正文还继续把 `4月29日披露Rocket Lab赢得1.9亿美元国防合同` 拼成“重大订单利好，但非当日新发消息”，说明最新窗口仍把“未达价格阈值 + 旧事件背景”包装成正式 `triggered` 提醒。
- `data/runtime/logs/web.log.2026-05-01`
  - `2026-05-01 09:30:26.415` 记录同一 `job_id=j_1241aad0` 收口为 `parse_kind=JsonTriggered` 并执行 `deliver`，`raw_preview` / `deliver_preview` 都直接写出 `单日涨跌幅接近但未达8%阈值`。
  - 同一 job 在同日更早窗口 `08:00:45.430` 与 `09:00:17.846` 又都正常落成 `parse_kind=JsonNoop` 并记录 `心跳任务未命中，本轮不发送`，说明 `09:30` 这轮不是持续越线后的稳定提醒，而是同一价格条件在相邻窗口间再次漂成正式误触发。

- `data/sessions.sqlite3` -> `cron_job_runs`
  - `run_id=11216`
  - `job_id=j_1241aad0`
  - `job_name=RKLB异动监控`
  - `executed_at=2026-04-30T13:00:31.251295+08:00`
  - `execution_status=completed`
  - `message_send_status=sent`
  - `delivered=1`
  - `response_preview` 明确写出：`RKLB触发重大订单提醒... 当前股价$77.02... 涨跌幅未超过8%阈值`
  - 同一用户直聊会话 `Actor_feishu__direct__ou_5f64ee7ca7af22d44a83a31054e6fb92a3` 在 `12:17:02-12:29:11 CST` 刚连续反馈“这合同是什么时候的”与“所以这些老新闻不要重复发了 你昨天也发了好多次给我”，说明最近一小时用户侧已明确把这类文案识别为旧新闻噪音；但 heartbeat 仍在 `13:00` 把同一旧合同和未命中阈值的价格状态包装成正式 `triggered`。
- `data/runtime/logs/sidecar.log`
  - `2026-04-30 13:00:28.749-13:00:28.750` 记录同一 `job_id=j_1241aad0` 收口为 `parse_kind=JsonTriggered` 并执行 `deliver`，`raw_preview` / `deliver_preview` 都直接写出 `涨跌幅未超过8%阈值`。
  - 这说明 `2026-04-30 08:01` 的 RKLB 误触发并非单次波动；在用户刚明确要求“不要重复发旧新闻”后，同一 job 仍会把“旧合同 + 未命中阈值”继续升级成正式提醒。
- `data/sessions.sqlite3` -> `cron_job_runs`
  - `run_id=10943`
  - `job_id=j_1241aad0`
  - `job_name=RKLB异动监控`
  - `executed_at=2026-04-30T08:01:18.374082+08:00`
  - `execution_status=completed`
  - `message_send_status=sent`
  - `delivered=1`
  - `response_preview` 明确写出：`RKLB异动提醒... 最新价$77.02，较前收$78.59下跌-2.00%，未触发涨跌幅8%阈值`
  - 这说明最新复发已经不再局限于 ASTS / ORCL，而是扩展到 `RKLB异动监控`：正文明确承认“未触发 8% 阈值”，链路仍以正式 `triggered` 提醒送达用户。
- `data/runtime/logs/sidecar.log`
  - `2026-04-30 08:01:16.470-08:01:16.473` 记录同一 `job_id=j_1241aad0` 收口为 `parse_kind=JsonTriggered` 并执行 `deliver`，`raw_preview` / `deliver_preview` 都直接写出 `未触发涨跌幅8%阈值`。
  - 这说明当前线上坏态不是单个 ASTS 模板特例，而是“正文否认命中阈值，结构化状态仍给 triggered”这条单标的 heartbeat 误报链路继续扩散。

- `data/sessions.sqlite3` -> `cron_job_runs`
  - `run_id=10643`
  - `job_id=j_fc7749ca`
  - `job_name=ASTS 重大异动心跳监控`
  - `executed_at=2026-04-30T02:00:58.810628+08:00`
  - `execution_status=completed`
  - `message_send_status=sent`
  - `delivered=1`
  - `response_preview` 明确写出：`ASTS 基本面积事件触发... 当前股价 $69.61（昨收 $71.88），日内跌幅 -3.16%，未触及 8% 涨跌幅阈值`
  - 这说明最新复发已经跨日延续到 `2026-04-30 02:00` 窗口，且坏态仍是 `status=triggered` 与正文结论直接自相矛盾。
- `data/runtime/logs/sidecar.log`
  - `2026-04-30 02:00:55.564-02:00:55.565` 记录同一 `job_id=j_fc7749ca` 收口为 `parse_kind=JsonTriggered` 并执行 `deliver`，`raw_preview` / `deliver_preview` 都直接写出 `未触及 8% 涨跌幅阈值`。
  - 这说明 `2026-04-29 19:04` 补上的近阈值保险闸没有稳定覆盖最新 ASTS 变体；线上仍会把“未命中阈值”的文案作为正式提醒送达。

- `data/sessions.sqlite3` -> `cron_job_runs`
  - `run_id=10183`
  - `job_id=j_fc7749ca`
  - `job_name=ASTS 重大异动心跳监控`
  - `executed_at=2026-04-29T17:01:39.662237+08:00`
  - `execution_status=completed`
  - `message_send_status=sent`
  - `delivered=1`
  - `response_preview` 明确写出：`触发条件：单日涨跌幅超过 8%`，随后正文又承认 `当前跌幅未达到 8% 阈值，日内振幅未触及 8% 门槛`，但本轮仍以正式触发提醒送达。
- `data/runtime/logs/sidecar.log`
  - `2026-04-29 17:01:34.563-17:01:34.564` 记录同一 `job_id=j_fc7749ca` 收口为 `parse_kind=JsonTriggered` 并执行 `deliver`，`raw_preview` / `deliver_preview` 都直接写出 `当前跌幅未达到 8% 阈值`。
  - 这说明最新复发已不只是“接近 8% 警戒阈值”的措辞漂移，而是 `status=triggered` 与正文结论正面自相矛盾。
- `data/sessions.sqlite3` -> `cron_job_runs`
  - `run_id=9912`
  - `job_id=j_39a96b7a`
  - `job_name=ORCL 大事件监控`
  - `executed_at=2026-04-29T11:30:36.068108+08:00`
  - `execution_status=completed`
  - `message_send_status=sent`
  - `delivered=1`
  - `response_preview` 明确写出：`当前价格 $165.92，跌幅 4.07%（相对昨收 $172.96）... 该触发接近 5% 阈值，建议关注`
  - 同一 job 在下一窗口 `run_id=9941`（`2026-04-29T12:01:32.811230+08:00`）又恢复 `noop + skipped_noop`，说明这不是持续越线后的正常提醒，而是“接近 5%”被直接包装成正式触发。
- `data/runtime/logs/sidecar.log`
  - `2026-04-29 11:30:32.238-11:30:32.239` 记录同一 `job_id=j_39a96b7a` 收口为 `parse_kind=JsonTriggered`，`raw_preview` 与 `deliver_preview` 都明确承认只有 `跌幅 4.07%`，但仍落成正式投递。
- `data/sessions.sqlite3` -> `cron_job_runs`
  - `run_id=9818`
  - `job_id=j_fc7749ca`
  - `job_name=ASTS 重大异动心跳监控`
  - `executed_at=2026-04-29T09:31:25.539312+08:00`
  - `execution_status=noop`
  - `message_send_status=skipped_noop`
  - `detail_json.scheduler.raw_preview={"status":"noop"}`
- `data/sessions.sqlite3` -> `cron_job_runs`
  - `run_id=9844`
  - `job_id=j_fc7749ca`
  - `job_name=ASTS 重大异动心跳监控`
  - `executed_at=2026-04-29T10:01:20.670987+08:00`
  - `execution_status=completed`
  - `message_send_status=sent`
  - `delivered=1`
  - `response_preview` 明确写出：`最新价格 $71.88，相对昨收 $77.20 跌幅 -6.89% ... 触发原因：单日涨跌幅（跌）接近 8% 警戒阈值，且距离 8% 仅差约 1.1 个百分点`
  - 同一条消息还把 `盘中低点 $71.00`、`日内振幅 7.81%` 与 FCC / BlueBird 7 旧事件一并拼入触发文案，但正文没有给出任何真实越过 `8%` 的新证据。
- `data/runtime/logs/sidecar.log`
  - `2026-04-29 09:31:25.539` 记录同一 `job_id=j_fc7749ca` 收口为 `parse_kind=JsonNoop`，并写出 `心跳任务未命中，本轮不发送`。
  - `2026-04-29 10:01:17.536` 同一 job 又记录 `parse_kind=JsonTriggered`，`raw_preview` 与 `deliver_preview` 都把 `跌幅 -6.89%` 包装成“接近 8% 警戒阈值”，随后实际投递。
- 相关缺陷对照：
  - [`scheduler_heartbeat_orcl_intraday_range_false_trigger.md`](./archive/scheduler_heartbeat_orcl_intraday_range_false_trigger.md) 已修复的是“把日内高低点/振幅误当成涨跌幅阈值”。
  - [`scheduler_watchlist_near_threshold_false_trigger.md`](./archive/scheduler_watchlist_near_threshold_false_trigger.md) 是多标的 watchlist 把“接近阈值”包装成触发。
  - 本次样本已覆盖两个单标的 heartbeat：`ASTS 重大异动心跳监控` 把 `-6.89%` 包装成“接近 8%”，`ORCL 大事件监控` 把 `-4.07%` 包装成“接近 5%”；两者都属于同一条“接近阈值 => triggered”链路。

## 端到端链路

1. Feishu heartbeat scheduler 触发单标的价格监控任务（如 `ASTS 重大异动心跳监控`、`ORCL 大事件监控`）。
2. runner 查询最新价格与相关背景信息。
3. 某些窗口会正常返回 `noop`。
4. 一旦自然语言里出现“接近 5% / 8% 阈值，建议关注”之类表述，链路会把未越线的观察性提示收口成 `{"status":"triggered"}`。
5. scheduler 消费该结果后按 `completed + sent + delivered=1` 正式向用户发送告警；下一窗口又可能回到 `noop`。

## 期望效果

- 当 heartbeat 条件写的是“单日涨跌幅（跌）达到 5% / 8%”时，只有真实越过阈值才允许返回 `triggered`。
- 若仅接近阈值，最多只能作为风险观察或上下文说明，不应进入最终发送链路。
- 同一 job 不应在前一窗口 `noop`、后一窗口没有新增越线证据的情况下，把“接近阈值”直接升级成正式提醒。

## 当前实现效果

- `2026-05-01 15:02` 的 `持仓重大事件心跳检测` 再次把 `{"status":"triggered"}` 正式送达给用户，正文把 `RKLB当前$82.51，较前收盘价上涨+7.13%` 与 `4月29日` 旧合同共同包装成“重大增量”。
- 这说明最新坏态已经不只停留在单标的阈值误报；同一旧合同/近阈值叙事还会被组合级 heartbeat 吸收并放大成正式提醒，本单继续保持活跃 `New` 状态。
- `2026-05-01 13:00` 的 `RKLB异动监控` 再次把 `{"status":"triggered"}` 正式送达给用户，但正文明确承认 `单日涨跌幅 +7.13%（已接近但未达8%阈值）`，同时继续把 `4月29日` 的旧合同包装成本轮触发理由。
- 这说明 `09:30` 样本并非孤立波动；同一 job 在同一个交易日午后窗口仍会把未越线价格和旧新闻背景升级成正式告警，本单继续保持活跃 `New` 状态。
- `2026-05-01 09:30` 的 `RKLB异动监控` 再次把 `{"status":"triggered"}` 正式送达给用户，但正文明确承认 `单日涨跌幅接近但未达8%阈值`，还补充 `4月29日` 旧合同只是“非当日新发消息”。
- 这说明 `2026-04-30` 标记为 `Fixed` 的修复结论没有在线上稳定生效；最新窗口里，同一 job 在 `08:00/09:00` 还能正常 `noop`，到 `09:30` 又把未达阈值的观察性提示升级成正式告警。
- `2026-04-30 13:00` 的 `RKLB异动监控` 再次把 `{"status":"triggered"}` 送达给用户，但正文明确承认 `涨跌幅未超过8%阈值`，且主题仍是用户在同一小时刚追问过时间点的旧合同。
- 这说明线上最新坏态不只是“接近阈值也算触发”，还会把已被用户指出是旧新闻的事件继续叠加到误触发告警里，形成“错误触发 + 重复旧闻”双重噪音。
- `2026-04-30 08:01` 的 `RKLB异动监控` 把 `{"status":"triggered"}` 送达给用户，但正文明确承认 `较前收下跌 -2.00%，未触发涨跌幅8%阈值`。
- 这说明线上最新坏态已从 ASTS / ORCL 继续扩展到 RKLB：只要存在重大事件叙述，模型仍会把“事件成立但价格条件未命中”的观察性提示升级成正式触发提醒。
- `2026-04-30 02:00` 的 `ASTS 重大异动心跳监控` 再次把 `{"status":"triggered"}` 送达给用户，但正文明确承认 `日内跌幅 -3.16%，未触及 8% 涨跌幅阈值`。
- 这说明线上最新坏态没有收口到单日样本，而是跨日后继续把“事件存在但价格条件未命中”的观察性提示升级成正式触发提醒。
- `2026-04-29 17:01` 的 `ASTS 重大异动心跳监控` 再次把 `{"status":"triggered"}` 送达给用户，但正文明确承认 `当前跌幅未达到 8% 阈值，日内振幅未触及 8% 门槛`。
- 这说明线上最新坏态已经不只是“接近阈值也算触发”，而是触发状态与结论文本直接自相矛盾，用户会收到一条自称“已触发”但正文说“未触发”的告警。
- `2026-04-29 10:01` 的 `ASTS 重大异动心跳监控` 把 `跌幅 -6.89%` 解释成“接近 8% 警戒阈值”，并成功送达。
- `2026-04-29 11:30` 的 `ORCL 大事件监控` 又把 `跌幅 4.07%` 解释成“接近 5% 阈值”，同样成功送达；`12:01` 下一窗口立即恢复 `noop`。
- 两条文案都没有声称价格真的越过阈值，而是明确承认“接近阈值”，却仍返回 `JsonTriggered`，说明当前链路会把观察性提示直接升级成用户可见触发告警。

## 用户影响

- 用户会收到并不存在的 ASTS / ORCL 触发提醒，以为“单日跌幅达到 8% / 5%”这类监控条件已经满足。
- 用户还会在组合级 heartbeat 里收到“旧合同 + 近阈值涨幅”被包装成新增催化的 RKLB 提醒，进一步放大重复旧闻和误触发噪音。
- 该问题会直接影响监控可信度和用户后续交易/关注决策，属于功能性告警误报，因此定级为 `P2`。

## 根因判断

- 初步判断不是发送链路或通用 JSON 解析失败，而是单标的 heartbeat 模板仍允许模型把“接近阈值”“建议关注风险”这类观察性表达直接收口成 `triggered`。
- `2026-05-01 15:02` 的组合级样本说明，同一根因还包含“旧合同/旧催化缺少增量去重”，使得近阈值价格观察与旧事件背景结合后，更容易被模型升级成 `triggered`。
- 这与已修复的 ORCL/ASTS 高低点口径混算不同；本次样本里正文已经明确承认没有达到 `5% / 8%`，说明缺口更偏向“缺少 triggered 前的数值硬校验”。
- 同时它与 watchlist 的近阈值误报表现相似，提示“接近阈值也算触发”的语义漂移并不只存在于多标的 watchlist。

## 修复记录

- 2026-05-02 03:05: 本轮补强 heartbeat 已送达预览去重，覆盖最新样本中“近阈值价格观察 + 旧合同/旧催化”被组合级或单标的 heartbeat 重新包装成正式触发的路径：`RKLB 4月29日 1.9 亿美元国防合同` 这类中英混写旧事件即使换写法也会命中 `duplicate_suppressed`；既有近阈值硬拦截仍覆盖 `接近但未达 / 未超过 / 未触及` 等明确否认越线文案。回归验证：`cargo test -p hone-channels heartbeat_duplicate_preview_match --lib -- --nocapture`、`cargo test -p hone-channels heartbeat_ --lib -- --nocapture` 通过；无关联 GitHub Issue。
- 2026-05-01 15:18: 最新真实窗口再次确认本单仍活跃：`run_id=12511` 把组合级 `持仓重大事件心跳检测` 中的 `RKLB当前$82.51...上涨+7.13%` 与 `4月29日` 旧合同再次落成 `completed + sent + delivered=1`；说明同一近阈值/旧事件误报已经扩散到组合级 heartbeat。
- 2026-05-01 13:03: 最新真实窗口再次确认本单仍活跃：`run_id=12420` 把 `RKLB异动监控` 的 `已接近但未达8%阈值` 文案再次落成 `completed + sent + delivered=1`；说明同一根因在 `09:30` 之后没有收敛，继续维持 `New`。
- 2026-05-01 10:02: 最新真实窗口再次确认本单回归：`run_id=12263` 把 `RKLB异动监控` 的 `接近但未达8%阈值` 文案又落成 `completed + sent + delivered=1`；此前 `Fixed` 结论失效，本单状态改回 `New` 并重新进入活跃队列。
- 2026-04-30 15:05: 本轮继续补强同一送达前保险闸，新增 `未触发 / 没有触发 / 尚未触发` 以及 `未超过 / 没有超过 / 尚未超过` 等直接否认触发的阈值措辞覆盖；`RKLB异动监控` 中 `未触发涨跌幅8%阈值`、`涨跌幅未超过8%阈值` 这类 `triggered` 输出会被落成 `near_threshold_suppressed`，不再投递。回归验证：`cargo test -p hone-channels heartbeat_ -- --nocapture`。
- 2026-04-30 13:00 最新真实窗口再次确认本单仍在扩散：`run_id=11216` 把 `RKLB异动监控` 的 `涨跌幅未超过8%阈值` 文案再次落成 `completed + sent + delivered=1`，且同一小时用户已直接反馈“老新闻不要重复发”；说明当前保护仍未覆盖“旧事件 + 价格条件明确否认命中”的单标的 heartbeat 变体，本单继续保持 `New`。
- 2026-04-30 08:01 最新真实窗口再次确认本单仍在扩散：`run_id=10943` 把 `RKLB异动监控` 的 `未触发涨跌幅8%阈值` 文案仍落成 `completed + sent + delivered=1`；说明当前保护没有稳定覆盖“事件触发 + 价格阈值明确否认命中”的单标的 heartbeat 新变体，本单继续保持 `New`。
- 2026-04-30 02:00 最新真实窗口再次确认 ASTS 仍复发：`run_id=10643` 在正文已明确写出 `日内跌幅 -3.16%，未触及 8% 涨跌幅阈值` 的前提下，仍落成 `completed + sent + delivered=1`；说明当前保护仍未稳定覆盖“事件触发 + 价格阈值否认命中”的跨日变体，本单继续保持 `New`。
- 2026-04-29: `crates/hone-channels/src/scheduler.rs` 在 heartbeat 送达前增加近阈值保险闸：`跌幅 -6.89% 接近 8% / 仅差约 1.1 个百分点` 这类承认未达到阈值的 `triggered` 文案会被抑制，不再进入用户可见发送链路。
- 回归验证：`cargo test -p hone-channels heartbeat_near_threshold_trigger_is_suppressed -- --nocapture`。
- 2026-04-29 17:01 最新真实窗口再次确认 ASTS 仍复发：`run_id=10183` 在正文已明确写出 `当前跌幅未达到 8% 阈值` 的前提下，仍落成 `completed + sent + delivered=1`；说明当前保护尚未覆盖“触发条件声明 + 正文否认命中”这一新变体。
- 2026-04-29 11:30-12:01 最新真实窗口仍复现回归：`run_id=9912` 把 ORCL `跌幅 4.07%` 写成“接近 5% 阈值”并送达，下一窗口 `run_id=9941` 才恢复 `noop`；说明近阈值保险闸尚未稳定覆盖所有单标的 heartbeat 变体，本单改回 `New`。
- 2026-04-29 19:04: 本轮补强同一保险闸，新增 `门槛 / 未触及 / 未命中 / 未满足 / 未达` 等否认命中措辞覆盖；`触发条件：超过 8%` 但正文写出 `当前跌幅未达到 8% 阈值，日内振幅未触及 8% 门槛` 的 `triggered` 输出会被落成 `near_threshold_suppressed`，不再投递。回归验证：`cargo test -p hone-channels heartbeat_ -- --nocapture`。

## 下一步建议

- 后续仍可把 heartbeat `triggered` 结果升级成机器可校验的数值字段，例如 `metric`, `threshold`, `observed_value`, `comparison_passed`，进一步减少模型自由文本判断空间。
- 在 ASTS / ORCL / watchlist 这类价格阈值模板里明确禁止把“接近阈值”“距离阈值不远”“建议关注波动”解释成 `triggered`。
- 为单标的 heartbeat 增加回归样本：当最新涨跌幅仅 `-6.89%` 对 `-8%`、或仅 `-4.07%` 对 `-5%` 时，必须返回 `noop` 或独立的 `near_threshold`，不得发送正式提醒。
