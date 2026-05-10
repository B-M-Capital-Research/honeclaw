# Bug: Heartbeat 破位预警直接输出无条件止损交易指令

- **发现时间**: 2026-05-10 07:04 CST
- **Bug Type**: Business Error
- **严重等级**: P2
- **状态**: New

## 最新进展（2026-05-10 19:02 CST）

- `2026-05-10 23:10 CST` 本轮继续确认同一缺陷活跃：
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=18222`
    - `job_name=CAI破位预警`
    - `executed_at=2026-05-10T19:30:40.801091+08:00`
    - `execution_status=completed`
    - `message_send_status=sent`
    - `delivered=1`
    - `detail_json.scheduler.parse_kind=JsonTriggered`
    - `response_preview` 与 `detail_json.scheduler.deliver_preview` 继续包含 `建议动作：无条件止损`。
  - 结论：直接交易指令 guard 在 19:30 真实窗口仍未覆盖 live 出站路径，维持 `P2 / New`。

- 本轮缺陷巡检确认该缺陷在最近四小时真实 heartbeat 窗口复发，状态从 `Fixed` 回退为 `New`：
  - `data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=18117`
    - `job_name=CAI破位预警`
    - `executed_at=2026-05-10T15:30:30.441923+08:00`
    - `execution_status=completed`
    - `message_send_status=sent`
    - `delivered=1`
    - `detail_json.scheduler.parse_kind=JsonTriggered`
    - `response_preview` 继续向用户送达 `建议动作：无条件止损`。
  - 同条 `detail_json.scheduler.deliver_preview` 与 `raw_preview` 都保留了相同直接交易指令，说明这是模型结构化触发后进入最终出站的用户可见内容，不是中间草稿。
- 结论：这是同一根因/同一链路复发，不新建重复文档。主发送链路成功，但自动化金融预警仍越过“只报告事实和条件化风险边界”的要求，影响投研输出安全性，因此继续按功能性 `P2 / New` 跟踪。

## 修复进展（2026-05-10 07:05 CST）

- `crates/hone-channels/src/scheduler.rs` 为 heartbeat prompt 增加“交易动作边界”：自动预警只能报告触发事实、价格 / 成交量 / 时间口径和条件化风险管理框架，不得输出 `无条件止损`、`必须卖出`、`立即清仓`、`马上买入` 等直接交易指令。
- 同时在 scheduler 出站前增加通用 guard：命中直接交易指令时，会把正文改写为风险提示，保留价格与触发事实片段，移除无条件买卖 / 止损 / 清仓动作句。
- 新增回归：
  - `heartbeat_prompt_rejects_direct_trade_instructions`
  - `heartbeat_direct_trade_instruction_gets_risk_guard`
- 验证：
  - `cargo test -p hone-channels heartbeat_direct_trade_instruction --lib -- --nocapture`
  - `cargo test -p hone-channels heartbeat_prompt_rejects_direct_trade_instructions --lib -- --nocapture`
  - `cargo check -p hone-channels --tests`

## 证据来源

- `data/sessions.sqlite3` -> `cron_job_runs`
  - `run_id=17796`
  - `job_name=CAI破位预警`
  - `executed_at=2026-05-10T03:30:24.716901+08:00`
  - `execution_status=completed`
  - `message_send_status=sent`
  - `delivered=1`
  - `detail_json.scheduler.parse_kind=JsonTriggered`
  - `response_preview` 在列出 CAI 跌破 52 周低点、当前价、盘中低点和成交量后，直接写出 `建议动作：无条件止损...不建议抄底或持有等待反弹`。

## 端到端链路

1. Feishu heartbeat scheduler 触发 `CAI破位预警`。
2. function-calling heartbeat runner 返回合法 `JsonTriggered`。
3. scheduler 将模型正文作为用户可见预警发送，台账落成 `completed + sent + delivered=1`。
4. 最终送达文本从价格/阈值提醒升级为直接交易指令，缺少“仅供分析参考”“需结合仓位与风险评估”“触发/失效条件”等边界。

## 期望效果

- 破位预警应报告已核验的价格、阈值、成交量、时间口径与风险事实。
- 如给动作建议，只能表达为条件化的风险管理框架，例如“若用户原本以该阈值作为止损线，应复核仓位和风险承受能力”，不能替用户下达“无条件止损”这类确定性交易指令。
- 对涉及买卖、止损、加仓、减仓的输出，应明确保持分析参考口径，并提供触发条件、证伪条件和风险边界。

## 当前实现效果

- 真实窗口已成功送达的 `CAI破位预警` 曾直接输出 `无条件止损`。
- 这不是发送失败、重复投递或 JSON 解析失败；链路本身成功，但用户可见内容越过投研助手的动作边界。
- 同窗其它 heartbeat 能正常 `noop` 或送达，说明问题集中在 heartbeat 预警文案约束与最终出站安全边界。

## 用户影响

- 用户可能把系统预警理解为直接交易指令，而不是风险提示或分析参考。
- 该问题发生在自动化 heartbeat 推送里，用户没有即时追问澄清上下文；错误口径会以主动通知形式影响风险管理决策。
- 定为 `P2`：主投递链路没有阻断，但它影响金融投研输出正确性和风险管理安全边界，不属于只影响表达观感的 `P3`。

## 根因判断

- heartbeat prompt / 输出约束允许模型在破位场景中生成过强动作词。
- scheduler 当前只校验结构化状态和基础出站净化，没有对 `无条件止损`、`必须卖出`、`立即买入` 等直接交易指令做渠道级降级或改写。
- 该根因不同于 `scheduler_heartbeat_retrigger_duplicate_alerts.md` 的重复提醒，也不同于 `scheduler_heartbeat_unknown_status_silent_skip.md` 的结构化解析漂移。

## 下一步建议

- 在 heartbeat 系统提示中增加“预警只报告事实和条件，不输出无条件买卖/止损指令”的硬约束。
- 在 scheduler 出站前增加轻量 guard：命中直接交易指令词时，将动作改写为条件化风险提示，或加上明确的分析参考边界。
- 增加回归样本，覆盖 `无条件止损`、`立即清仓`、`马上买入` 等不应原样外发的自动预警文案。
