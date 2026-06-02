# Bug: Web scheduler ACP stream disconnects without final reply

- 发现时间：2026-06-02 23:06 CST
- Bug Type：System Error
- 严重等级：P2
- 状态：New
- GitHub Issue：无，非 P1

## 证据来源

- `data/runtime/logs/acp-events.log`
  - 时间窗：2026-06-02 20:00-21:05 CST。
  - Web scheduler actor `Actor_web__direct__web-user-ba50cb9401c0` 在 20:00 CST 收到任务 `20:00 持仓股重要新闻晚报` 的 `session/prompt`，20:01 CST 先输出 transport fallback 提示，20:04 CST `session/prompt` 对应 response 返回 `stream disconnected before completion` 内部错误，没有 `stopReason=end_turn`。
  - Web scheduler actor `Actor_web__direct__web-user-e05f5e5f74a3` 在 20:00 CST 收到任务 `英伟达每日消息` 的 `session/prompt`，20:01 CST 输出 transport fallback 提示，未见对应 `stopReason=end_turn`。
  - Web scheduler actor `Actor_web__direct__web-user-f40ae1caa720` 在 20:30 CST 与 21:05 CST 收到任务 `10万元计划投资提醒 + A股持仓观察` 的 `session/prompt`，均输出 transport fallback 提示，未见对应 `stopReason=end_turn`。
  - Web scheduler actor `Actor_web__direct__web-user-c394f2531362` 在 21:00 CST 收到任务 `持仓关键事件每日汇总` 的 `session/prompt`，21:01 CST 输出 transport fallback 提示，未见对应 `stopReason=end_turn`。
- `data/sessions.sqlite3`
  - 2026-06-02 19:02-23:02 CST 窗口内 Feishu direct 有 16 个 user turn 与 16 个 assistant final，最近 direct 会话均以 assistant 收口。
  - `cron_job_runs.max(executed_at)` 仍停在 `2026-06-01T00:26:00.908925+08:00`，本机 SQLite 没有记录本窗 Web scheduler 终态；结合当前 cloud mode 既有结论，本轮以 ACP 事件作为 Web scheduler 真实运行证据。
- 最近四小时无非文档代码提交。

## 端到端链路

1. Web scheduler 到点触发多个用户配置的定时任务。
2. ACP runner 成功初始化 session、设置模型并接收 `session/prompt`。
3. runner 在执行过程中从 WebSocket fallback 到 HTTPS transport。
4. 至少一条任务返回 `stream disconnected before completion` 内部错误；其余同窗任务停在 fallback 后没有最终 `end_turn`。
5. Web scheduler 未产出用户可见复盘正文，也未留下产品化失败回复作为定时任务结果。

## 期望效果

- Web scheduler 的每次触发都应有终态：成功时输出业务正文并 `end_turn`，失败时写入用户可见的产品化失败提示。
- ACP transport 断连应被 scheduler 外层识别为执行失败，并记录可审计失败状态，不能让任务只停在内部 runner 错误或无终态事件。
- 失败文案不应外露内部 URL、transport 细节或原始 runner 错误。

## 当前实现效果

- 20:00 CST 到 21:05 CST 的多个 Web scheduler prompt 没有对应 `stopReason=end_turn`。
- 其中一条 `session/prompt` response 明确返回 `stream disconnected before completion` 内部错误。
- 22:46 CST 之后 Web direct / Web scheduler 又有正常 `end_turn`，说明故障不是全局 Web ACP 永久不可用，而是本窗多条 scheduler 触发未被可靠收口。

## 用户影响

- 用户配置的 Web 定时复盘可能到点后没有收到任何正文或可理解失败提示。
- 这会直接影响 Web scheduler 的核心交付链路：定时任务触发了，但结果未送达、未收口。
- 定级为 `P2`：该问题阻断 Web scheduler 功能链路，但证据集中在 Web scheduler，Feishu direct 同窗正常收口，且没有跨渠道大面积不可用或数据安全影响，因此不定为 `P1`。

## 根因判断

- 直接根因证据是 ACP transport 在任务执行中断连，导致 `session/prompt` 未能完成。
- 初步判断 scheduler 外层缺少对 ACP stream disconnect / no-final-response 的超时收口与产品化失败落库，或失败只停留在 ACP response 内部错误，没有转换成 Web scheduler 用户态结果。
- 该问题不同于 `web_scheduler_mobile_push_not_delivered.md`：本轮不是手机系统通知能力边界，而是定时任务正文本身没有成功生成 / 收口。
- 该问题也不同于已归档的 unfinished tool send_failed：本轮没有看到已产出正文后 SSE 离线或工具未完成尾部，而是 ACP transport 断连和缺失最终 `end_turn`。

## 下一步建议

- 在 Web scheduler 执行入口增加 ACP prompt 级 watchdog：超过任务预算或收到 runner internal error 时，必须写入产品化失败回复并落成失败终态。
- 将 `stream disconnected before completion`、transport fallback 后长时间无 `end_turn`、`session/prompt` internal error 统一分类为 scheduler execution failure。
- 增加回归：模拟 ACP `session/prompt` 返回 internal error 或永不返回 `end_turn` 时，Web scheduler transcript 与台账都能留下非敏感失败提示。
