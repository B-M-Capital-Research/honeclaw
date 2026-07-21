# Bug: Feishu 普通 scheduler 未触发静默条件时仍发送完整报告

## 发现时间

2026-07-21 03:02 CST

## Bug Type

Business Error

## 严重等级

P2

## 状态

Fixed

## GitHub Issue

无，非 P1

## 证据来源

- `data/sessions.sqlite3`
  - 巡检窗口：2026-07-20 23:02-2026-07-21 03:02 CST。
  - 窗口内按真实 `timestamp` 新增 6 条 user / 6 条 assistant，覆盖 3 个 session，均以 assistant 收口。
  - `session_id=Actor_feishu__direct__ou_5f895bed1573d53053e89bfc382b523a44`
    - `ordinal=18` / `timestamp=2026-07-20T23:30:01.398154+08:00`：Feishu scheduler 任务 `科技成长股持仓买卖点日内预警` 明确要求校验 BE / RKLB / TEM / MSFT 的触发位，并写明“若未触发，则保持静默”。
    - `ordinal=19` / `timestamp=2026-07-20T23:30:26.018222+08:00`：assistant final 仍生成完整持仓报告，并在正文中自行判断 `TEM — $40 未破，静默`、`RKLB — $60 未破，静默`、`MSFT — $380 未破，静默`、`无纪律触发，全部静默`。
  - 同窗 `cron_job_runs` 无新增，`max(executed_at)` 仍停在 `2026-07-19T13:31:15.040172+08:00`；本条用户可见证据以 `session_messages` 为准。

## 端到端链路

1. Feishu 普通 scheduler 触发 `科技成长股持仓买卖点日内预警`。
2. 用户任务正文定义一组价格 / 技术条件，并要求未触发时静默。
3. assistant 执行行情与持仓判断。
4. assistant 在 final 中确认没有纪律触发，但仍把完整报告写入会话。
5. 用户收到一条本应静默的报告。

## 期望效果

当普通 scheduler 任务明确要求“若未触发，则保持静默”且模型判断没有触发条件时，链路应落成不投递或 no-op；最多只在内部台账记录本轮检查结果，不应向用户发送完整正文。

## 当前实现效果

截至 2026-07-21 的代码修复前，模型能识别未触发条件，但输出层没有把“全部静默 / 未触发”转成跳过发送，仍把完整分析正文作为 final 落库并面向用户可见。

## 用户影响

- 用户会收到本应静默的噪音提醒，削弱价格预警任务的可信度。
- 高频交易日任务可能反复推送“未触发”长报告，用户难以区分真正触发的买卖点提醒。
- 这是功能性缺陷：静默 / no-op 是该类任务的核心交付语义，不只是文字质量问题。

## 根因判断

当前证据指向普通 scheduler 的 skip-delivery 判定没有覆盖“模型 final 已确认未触发但仍生成正文”的场景。已有 heartbeat 结构化状态退化文档覆盖的是 `heartbeat=1` 的 JSON / noop 协议漂移；本次样本来自 `heartbeat=0` 普通 Feishu scheduler，链路和受影响范围不同，因此独立登记。

严重等级定为 P2：问题会导致监控任务错误投递噪音报告，影响功能语义和用户决策提醒可信度；但本窗没有错对象投递、数据破坏、敏感信息泄露、全渠道不可用或活跃 P1 证据。

## 下一步建议

1. 在普通 scheduler 出站前增加 skip-delivery 判定，识别“未触发 / 保持静默 / 全部静默 / 今日跳过推送”等明确 no-op 语义。
2. 区分用户要求的“静默不推送”和普通报告任务的“无重大更新但仍需简报”，避免误杀日常摘要。
3. 为 Feishu 普通 scheduler 增加回归：当任务正文包含“若未触发则保持静默”且 final 判断“全部静默”时，应记录 no-op 或 skipped，不发送用户可见正文。

## 修复记录

- 2026-07-21：普通 scheduler 出站链路已补“静默 no-op”判定。
  - 代码位置：`crates/hone-channels/src/scheduler.rs`
  - 修复内容：当任务正文明确要求“若未触发则保持静默/静默不推送”时，若 final 同时表达“未触发/未破/无纪律触发”与“静默/不推送”，出站层会回滚本轮 assistant 持久化并按 `should_deliver=false` 收口，不再向用户发送完整报告。
  - 回归覆盖：新增正反两条单元测试，覆盖“静默任务 + 全部静默”命中 skip，以及普通复盘任务不被误判为 skip。
  - 验证：`cargo test -p hone-channels silent_noop_signal_ --lib -- --nocapture`、`cargo test -p hone-channels skip_delivery_signal_detected --lib -- --nocapture`、`cargo check -p hone-channels --tests` 通过。
  - 说明：本轮未重启当前 Feishu / scheduler live 服务，因此状态先记为代码级 `Fixed`；若后续 2026-07-21 之后的真实运行窗仍出现同类“全部静默但照样投递”样本，再按新证据重新打开。
