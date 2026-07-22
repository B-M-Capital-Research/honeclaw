# Bug: Web direct terminal prefix mismatch commits generic research failure

## 发现时间

2026-07-22 23:02 CST

## Bug Type

System Error

## 严重等级

P2

## 状态

Fixed

## GitHub Issue

无，当前不是 P1。

## 证据来源

- `data/sessions.sqlite3` / `data/runtime/logs/web.log.2026-07-22`
  - 后续本地夜间窗口继续复发，状态一度维持 `New / P2`。
  - `session_id=Actor_web__direct__web-user-5bb05078acd4`
    - `2026-07-22T23:48:49.234724+08:00` 用户问 `LITE Call 20261218 750 成本116，我今天要卖出止盈么？`。
    - `2026-07-22T23:49:53.052662+08:00` assistant final 只返回“本轮研究未能完成，暂未形成可供参考的标的结论。”
    - 同一夜间窗口里用户原问题重试。
    - 重试轮 assistant 再次只返回同一失败提示。
  - runtime 两轮均已完成 `data_fetch search` / `data_fetch quote LITE`，第二轮还完成 `data_fetch snapshot LITE` 与 `web_search`；随后均记录 `entity_resolution.agent_loop ... contract_built=true entities=LITE answer_preserved=true`，再因 `committed terminal prefix mismatch` 写入 `committed_prefix_after_terminal_failure`。
  - 本轮问题直接阻断 LITE 期权止盈决策回答，但同窗其它 Web / Feishu 会话仍有正常 assistant 收口，未见错投、敏感信息外泄或全渠道不可用；严重等级仍为 P2，非 P1，不创建 GitHub Issue。

- `data/sessions.sqlite3`
  - 巡检窗口：2026-07-22 19:01-23:02 CST。
  - `session_id=Actor_web__direct__web-user-0545ade83537`
    - `2026-07-22T22:17:22.311440+08:00` 用户粘贴 A 股复盘长文并要求回答“周四关注方向，反弹结束还是正常分歧如何理解”。
    - `2026-07-22T22:17:46.394961+08:00` assistant final 只返回“本轮研究未能完成，暂未形成可供参考的标的结论。”
    - `2026-07-22T22:18:11.205115+08:00` 用户原文重试。
    - `2026-07-22T22:19:22.270641+08:00` assistant 再次只返回同一失败提示。
    - `2026-07-22T22:19:57.067865+08:00` 用户追问“为啥 你回答下思考题啊”。
    - `2026-07-22T22:22:48.675671+08:00` assistant 才输出 3915 字业务正文，说明同会话链路可恢复，但前两轮明确任务没有完成。
- `data/runtime/logs/web.log.2026-07-22`
  - `22:17:46` 同 session 记录 `entity_resolution.agent_loop ... contract_built=false answer_preserved=true ... missing_explicit_seeds=456`，随后 `failed ... error="committed terminal prefix mismatch"`，并 `session.persist_assistant ... detail=committed_prefix_after_terminal_failure`。
  - `22:19:22` 重试轮再次记录 `contract_built=false answer_preserved=true ... missing_explicit_seeds=456`，随后同样 `committed terminal prefix mismatch`，再次持久化 terminal failure 前缀。
  - `22:20:35` compact 后第三轮重跑，`22:22:48` 记录 `success=true ... tools=14(data_fetch,web_search) reply.chars=3798`。

## 端到端链路

1. Web direct 收到用户明确的投研 / 复盘解读请求。
2. function-calling runner 进入 agent loop，并在中间阶段保留了可用答案信号：`answer_preserved=true`。
3. entity / terminal finalization 阶段检测到 `committed terminal prefix mismatch`。
4. 上层没有恢复已保留答案，也没有要求模型直接用已有信息回答，而是把 terminal failure 前缀持久化为 assistant final。
5. 用户连续两轮只看到通用研究失败提示，直到第三轮追问并 compact 后才得到正文。

## 期望效果

- 如果 runner 已经保留可用答案，terminal prefix mismatch 不应直接覆盖成通用失败。
- finalization 失败时应优先恢复已保留答案，或生成明确、可操作的降级摘要。
- 同一用户重试同一问题时，不应在相同 terminal prefix mismatch 上重复失败。

## 当前实现效果

- LITE 期权止盈请求曾在后续夜间窗口连续两轮复发同类 `committed terminal prefix mismatch`，且两轮都已有 `answer_preserved=true` 与 LITE 实体 contract。
- `answer_preserved=true` 后仍被 `committed terminal prefix mismatch` 改写成失败 final。
- 失败文案没有解释用户如何调整问题，也没有给出已有材料的部分结论。
- 同一会话连续两次复发，直到 compact 后才恢复。

## 修复情况

- 已在 `crates/hone-channels/src/agent_session/core.rs` 为 committed prefix 收口新增恢复分支：
  - 终稿仍带已提交 prefix 时，继续只做原有前导空白对齐。
  - 终稿只剩非空正文 tail、未携带 prefix 时，改为恢复成 `committed prefix + 正文`，不再直接降级成通用研究失败。
  - 若正文为空或正文里已出现冲突 prefix，仍保持 fail-closed。
- 已在 `crates/hone-channels/src/agent_session/tests.rs` 补回归，覆盖：
  - committed prefix + tail-only 终稿恢复；
  - 恢复后 Web direct 正常持久化 / 投递，不再落成 generic failure；
  - 既有 committed prefix 成功路径不回归。

## 验证

- `cargo test -p hone-channels committed_prefix_recovery_prepends_a_missing_prefix_only_for_tail_only_content --lib -- --nocapture`
- `cargo test -p hone-channels service_prefix_tail_only_final_response_is_recovered_without_generic_failure --lib -- --nocapture`
- `cargo test -p hone-channels service_prefix_and_final_tail_are_visible_and_persisted_byte_identically --lib -- --nocapture`
- `cargo check -p hone-channels --tests`

## 用户影响

- 用户明确要求回答思考题，却连续两次没有得到答案，需要额外追问。
- 这是功能性缺陷：单轮 Web direct 任务被 terminal finalization 失败阻断。
- 定级为 `P2`：它阻断当前用户任务，但同窗 28 个更新 session 中没有长期 user-only 残留、错投、敏感信息泄露或全渠道不可用；同一会话第三轮可恢复，因此不是 P1。

## 根因判断

- 直接根因是 Web direct agent finalization 的 terminal prefix 一致性检查失败后，系统选择提交失败前缀，而不是恢复 `answer_preserved=true` 的正文。
- `missing_explicit_seeds=456` 可能说明该轮被 entity / evidence contract 判定为缺少显式证券种子，但用户任务本身是文章观点解读，不一定需要强制证券实体 contract。
- 该缺陷不同于 `feishu_function_calling_max_iterations_generic_failure.md`：本轮没有 `max_iterations_exceeded:10`，失败发生在 terminal prefix / finalization 阶段。

## 后续观察

1. 继续观察同类 Web direct 长文解读 / 期权止盈问答，确认 live 路径不再把 answer-preserved 终稿降级成通用失败。
2. 若后续仍出现 `committed terminal prefix mismatch` 且正文为空，再单独排查上游 terminal synthesis / recovery 何时丢失正文。
