# Bug: Feishu scheduler 已保留答案后仍提交“状态无法确定”失败提示

## 发现时间

2026-07-23 03:01 CST

## Bug Type

System Error

## 严重等级

P2

## 状态

Fixed

## GitHub Issue

无，当前不是 P1。

## 证据来源

- `data/sessions.sqlite3`
  - 巡检窗口：2026-07-22 23:02-2026-07-23 03:01 CST。
  - `session_id=Actor_feishu__direct__ou_5fa8018fa4a74b5594223b48d579b2a33b`
  - `2026-07-23T00:00:01.538545+08:00` 用户侧触发 Feishu scheduler `TEM 每日动态监控`，要求监控 Tempus AI 的 AACR / 临床数据、药企合作、收入盈利拐点、财报公告与单日涨跌超 10% 价格异动。
  - `2026-07-23T00:00:32.054341+08:00` assistant final 只返回“这次操作可能已经执行，但执行器在返回最终确认前中断，当前状态无法确定...”，没有给出 TEM 本轮动态监控正文、已核验事实或“不触发推送”的业务结论。
  - 同一 session 随后的 `AAOI 每日动态监控` 与 `RKLB 每日动态监控` 分别在 00:01:00 / 00:01:52 正常收口，说明不是 Feishu scheduler 全链路不可用。
- `data/runtime/logs/web.log.2026-07-22`
  - `2026-07-23 00:00:11-00:00:26 CST` 同一 session 已成功执行 `data_fetch quote TEM`、`data_fetch news TEM`、`data_fetch earnings_calendar TEM`。
  - `2026-07-23 00:00:32.050 CST` 记录 `entity_resolution.agent_loop ... contract_built=false answer_preserved=true mode=observational`。
  - 紧接着 `MsgFlow/feishu failed`，错误文本就是最终用户可见的“执行器在返回最终确认前中断，当前状态无法确定”；随后 `session.persist_assistant ... detail=failed`。
  - 同窗没有 `stream disconnected before completion`、`scheduler_runner_timeout`、`max_iterations_exceeded` 或 provider 原始错误进入该条 final；问题发生在已保留答案后的最终确认 / 失败提交路径。

## 端到端链路

1. Feishu scheduler 到点触发 `TEM 每日动态监控`。
2. function-calling runner 成功完成 TEM 行情、新闻和财报日历工具调用。
3. agent loop 进入 observational 模式，且日志明确显示 `answer_preserved=true`。
4. 最终确认阶段没有恢复已保留答案，而是提交“状态无法确定”的通用失败提示。
5. 用户没有收到本轮 TEM 监控结论；后续 AAOI / RKLB 同类任务仍可正常回答。

## 期望效果

- observational scheduler 已经保留可用答案时，最终确认失败不应覆盖为“操作可能已经执行”的管理型失败提示。
- 对不涉及写入或重复启动研究任务的纯监控请求，应恢复已保留答案，或至少降级为包含 TEM 核验结果的简短摘要。
- 失败提示应准确区分“写操作状态不确定”和“只读监控报告未能最终提交”。

## 当前实现效果

- TEM 监控任务完成了多项只读工具调用，并已有 `answer_preserved=true`，但最终用户只看到状态不确定提示。
- 失败文案暗示可能发生重复写入 / 重复启动研究任务，不符合本轮只读监控语义。
- 同窗其它 scheduler 可收口，因此这是单任务 / 单路径完成率问题，不是全渠道不可用。

## 用户影响

- 这是功能性缺陷：用户预期收到 TEM 每日动态监控结论，实际只收到无法判断状态的失败提示。
- 没有证据显示错投、敏感信息外泄、数据破坏、全渠道停摆或连续大面积 scheduler 失败，因此定级为 P2，而不是 P1。

## 根因判断

- 直接根因是 Feishu scheduler 的 agent finalization / entity resolution 后处理在 `answer_preserved=true` 时仍进入失败提交路径。
- 与 `web_direct_terminal_prefix_mismatch_commits_generic_failure.md` 同属“已保留答案未恢复”问题族，但本轮受影响链路是 Feishu scheduler，错误终态不是 `committed terminal prefix mismatch`，而是面向写操作的不确定状态提示，因此单独建档。
- 与 `codex_acp_transport_disconnect_request_failure.md` 不同：本轮没有 ACP transport 断连或 scheduler runner timeout 证据。

## 修复记录

- 2026-07-24 07:01 CST 运行态复核：
  - 最近四小时唯一非文档代码提交为 `2c8cb316 fix scheduler preserved-answer failure fallback`，与本缺陷修复范围一致。
  - `data/runtime/logs/web.log.2026-07-23` 在 05:00、05:11、06:02 CST 出现 3 条 Feishu scheduler `answer_preserved=true`，对应 session 均随后 `session.persist_assistant detail=done` / `success=true`，没有再落成“状态无法确定”失败提示。
  - 同窗 06:31 CST Web scheduler 也出现 `answer_preserved=true` 并成功持久化；未检出 `执行器在返回最终确认前中断` 或 `状态无法确定`。
  - 因本窗没有复现失败覆盖，状态继续保持代码级 `Fixed`；仍需后续多窗口观察 live scheduler 是否持续加载该修复。

- 2026-07-23
  - `crates/hone-channels/src/response_finalizer.rs` 新增 `recover_failed_read_only_user_visible_output(...)`：只在已保留可见正文、且本轮工具调用全部属于显式已知只读工具时，允许从失败结果中恢复用户正文；继续显式排除“状态无法确定”、通用失败、额度/超时兜底和内部流程文案。
  - `crates/hone-channels/src/agent_session/core.rs` 失败持久化改为优先写入上述可恢复正文，避免 scheduler transcript 被通用失败或“状态无法确定”覆盖。
  - `crates/hone-channels/src/scheduler.rs` 普通 scheduler 失败分支新增只读正文恢复：当 `answer_preserved=true` 但最终确认失败时，优先外发已保留的业务正文；保留 skip-signal 和 commodity guard 约束，避免把恢复路径变成放宽校验。
  - 新增回归：
    - `failed_assistant_persisted_message_prefers_preserved_read_only_answer`
    - `failed_assistant_persisted_message_keeps_generic_failure_for_nonrecoverable_copy`
    - `scheduler_recovers_preserved_read_only_failure_answer`
    - `scheduler_does_not_recover_generic_failure_copy`

## 剩余风险

1. 这是代码级 `Fixed`，仍需后续真实运行态复核 `Feishu scheduler + answer_preserved=true` 样本，确认 live 进程加载后不再外发“状态无法确定”。
2. 该修复只覆盖“失败时已保留只读正文”的恢复；若 provider 在保留答案前就中断、或本轮存在未知/非只读工具，则仍会保持失败闭口，不在本单范围内。

## 验证

- `cargo test -p hone-channels failed_assistant_persisted_message_ --lib -- --nocapture`
- `cargo test -p hone-channels scheduler_recovers_preserved_read_only_failure_answer --lib -- --nocapture`
- `cargo test -p hone-channels scheduler_does_not_recover_generic_failure_copy --lib -- --nocapture`
- `cargo check -p hone-channels --tests`
