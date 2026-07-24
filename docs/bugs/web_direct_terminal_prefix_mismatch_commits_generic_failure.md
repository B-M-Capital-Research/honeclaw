# Bug: Web direct terminal prefix mismatch commits generic research failure

## 发现时间

2026-07-22 23:02 CST

## Bug Type

System Error

## 严重等级

P2

## 状态

New

## GitHub Issue

无，当前不是 P1。

## 证据来源

- `data/sessions.sqlite3`
  - `2026-07-25 07:02 CST` 巡检确认 2026-07-24 23:30 代码级修复后，同一 Web direct terminal failure 链路仍在普通宏观问题上复发；状态从 `Fixed` 回退为 `New/P2`。
  - `session_id=Actor_web__direct__web-user-be13e1f84d14`
    - `2026-07-25T06:54:21.677033+08:00` 用户追问“近期美股科技板块已经连续下跌一段时间了，结合上面说的加息背景，你预计本轮回调会持续多久”。
    - `2026-07-25T06:55:35.326267+08:00` assistant final 只返回“本轮研究未能完成，暂未形成可供参考的标的结论。”，`metadata_json` 为 `service_owned_initial_prefix=true`、`error_kind=AgentFailed`、`terminal_stream_incomplete=true`、`run_failed=true`。
    - `2026-07-25T06:56:59.526419+08:00` 用户补充科技股、光模块、存储、AI 等上下文；`2026-07-25T06:58:26.790930+08:00` assistant 再次只返回同一通用研究失败，metadata 仍为 `AgentFailed / terminal_stream_incomplete=true`。
    - `2026-07-25T07:00:18.100694+08:00` 用户进一步拆短为“你预计加息导致的本轮回调会持续多久”，触发 auto compact 后 `2026-07-25T07:01:29.956863+08:00` 才恢复正文输出，说明不是 Web direct 全局不可用，而是较复杂 direct 轮次 terminal stream / finalization 收口失败。
  - 同窗 `data/sessions.sqlite3` 按真实 `timestamp` 新增 15 条 user / 8 条 assistant / 6 条 system compact，覆盖 4 个更新 session；最近 assistant 到 07:01，普通 assistant final 污染扫描未见 `<think>`、本机路径、raw tool JSON、panic 或 provider 原始错误。
  - 判断：最新样本未直接暴露 `committed terminal prefix mismatch` 字符串，但用户可见症状与 metadata 仍是 Web direct terminal stream incomplete 后提交通用研究失败；与本缺陷同一 Web direct terminal failure 收口链路相邻，先补入原文档，不新建重复缺陷。
  - 严重等级维持 `P2`：连续阻断同一 Web direct 明确问题多轮回答，但同窗 Feishu direct / scheduler 与同会话短问仍有成功收口，未见全渠道停摆、错投、敏感信息泄露或持久化数据破坏，因此不是 `P1`，不创建 GitHub Issue。

- `data/sessions.sqlite3`
  - `2026-07-25 03:02 CST` 巡检确认本缺陷继续在 Web direct 普通投研 / 宏观问题上复发，状态维持 `New/P2`。
  - `session_id=Actor_web__direct__web-user-5bb05078acd4`
    - `2026-07-25T01:24:09.809730+08:00` 用户问 `HIMS怎么回事？为什么大跌？财报前还会反弹么？`。
    - `2026-07-25T01:25:32.678801+08:00` assistant final 只返回“本轮研究未能完成，暂未形成可供参考的标的结论。”，`metadata_json` 为 `service_owned_initial_prefix=true`、`error_kind=AgentFailed`、`terminal_stream_incomplete=true`、`run_failed=true`。
  - `session_id=Actor_web__direct__web-user-be13e1f84d14`
    - `2026-07-25T01:28:15.353224+08:00` 用户询问宏观环境、加息影响和特斯拉下跌原因。
    - `2026-07-25T01:29:34.952182+08:00` assistant final 只返回同一通用研究失败，`metadata_json` 同样标记 `AgentFailed / terminal_stream_incomplete=true`。
    - `2026-07-25T01:30:10.720715+08:00` 用户拆短为美股宏观 / 加息问题，`2026-07-25T01:31:11.555710+08:00` assistant 再次只返回同一通用研究失败。
    - `2026-07-25T01:31:35.961296+08:00` / `2026-07-25T01:32:55.792933+08:00` 用户进一步拆成更短问题后，01:32 / 01:33 才恢复正文输出，说明不是 Web direct 全局不可用，而是较复杂 direct 轮次 terminal stream / finalization 收口失败。
  - 同窗 `data/sessions.sqlite3` 按真实 `timestamp` 新增 16 条 user / 9 条 assistant / 6 条 system compact，覆盖 5 个更新 session；ordinary assistant final 污染扫描未见 `<think>`、本机路径、raw tool JSON、panic 或 provider 原始错误。
  - 判断：最新样本未直接暴露 `committed terminal prefix mismatch` 字符串，但用户可见症状与 metadata 仍是 Web direct terminal stream incomplete 后提交通用研究失败；与本缺陷同一 Web direct terminal failure 收口链路相邻，先补入原文档，不新建重复缺陷。
  - 严重等级维持 `P2`：连续阻断多个明确 Web direct 问题，但同窗 Feishu scheduler / Web direct 短问仍有成功收口，未见全渠道停摆、错投、敏感信息泄露或持久化数据破坏，因此不是 `P1`，不创建 GitHub Issue。

- `data/sessions.sqlite3`
  - `2026-07-24 23:02 CST` 巡检确认本缺陷继续在 Web direct 普通投研问题上复发，状态维持 `New/P2`。
  - `session_id=Actor_web__direct__web-user-400794904801`
    - `2026-07-24T22:57:33.921840+08:00` 用户问 `美股ai科技股盘中为什么暴跌`。
    - `2026-07-24T22:59:02.988003+08:00` assistant final 只返回“本轮研究未能完成，暂未形成可供参考的标的结论。”，`metadata_json` 为 `service_owned_initial_prefix=true`、`error_kind=AgentFailed`、`terminal_stream_incomplete=true`、`run_failed=true`。
    - `2026-07-24T22:59:36.375119+08:00` 用户 follow-up `盘中走势分析`，assistant 只要求补充具体标的，没有结合上一轮“美股 AI 科技股盘中暴跌”的上下文继续分析。
  - 同窗 `data/sessions.sqlite3` 按真实 `timestamp` 新增 72 条 user / 42 条 assistant / 9 条 system compact，覆盖 26 个更新 session；未见全渠道不可用、长期 user-only 残留、错投、敏感信息泄露或数据破坏。
  - 判断：最新样本未直接暴露 `committed terminal prefix mismatch` 字符串，但用户可见症状与 metadata 仍是 Web direct agent finalization / terminal stream incomplete 后提交通用研究失败；与本缺陷同一 Web direct terminal failure 收口链路相邻，先补入原文档，不新建重复缺陷。
  - 严重等级维持 `P2`：单轮 Web direct 明确问题没有完成，后续 follow-up 也未自动恢复上下文；但同窗其它会话正常收口，非 P1，不创建 GitHub Issue。

- `data/sessions.sqlite3` / `data/runtime/logs/web.log.2026-07-23`
  - `2026-07-24 03:02 CST` 巡检确认本缺陷在代码级 `Fixed` 后真实运行态复发，状态回退为 `New/P2`。
  - `session_id=Actor_web__direct__web-user-266454c88ed6`
    - `2026-07-24T02:50:16.904767+08:00` 用户问 `CIFR基本面怎么样，为什么连续四天大涨`。
    - runtime 已完成 `data_fetch search`、`web_search "CIFR C3.ai stock news July 2026 four day rally"`、`data_fetch snapshot CIFR`、`data_fetch news CIFR`、`web_search "Cipher Mining CIFR stock rally July 2026"`、`data_fetch financials CIFR`。
    - `2026-07-24 02:51:48` runtime 记录 `entity_resolution.agent_loop ... contract_built=false answer_preserved=true mode=observational`，随后 `failed ... error="committed terminal prefix mismatch"`，并 `session.persist_assistant ... detail=committed_prefix_after_terminal_failure`。
    - `2026-07-24T02:51:48.710306+08:00` assistant final 只返回“本轮研究未能完成，暂未形成可供参考的标的结论。”，`metadata_json` 为 `run_failed=true`、`terminal_stream_incomplete=true`、`error_kind=AgentFailed`。
  - 同窗 `data/sessions.sqlite3` 按真实 `timestamp` 新增 6 条 user / 6 条 assistant，覆盖 4 个更新 session；未见全渠道不可用、错投、敏感信息泄露或数据破坏，因此维持功能性 P2，非 P1，不创建 GitHub Issue。
  - 判断：这仍是已保留答案后被 terminal prefix mismatch 覆盖成通用研究失败的同一根因，不新建重复缺陷；本次复发说明 2026-07-22 的 tail-only 恢复未覆盖 `contract_built=false + observational answer_preserved=true` 形态。

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

- 2026-07-25 06:54-06:58 CST 同一 Web direct 宏观回调问题连续两轮落成 `AgentFailed / terminal_stream_incomplete=true`，只给通用研究失败；07:00 用户拆短并触发 compact 后才恢复正文输出。
- 2026-07-24 02:50 CST CIFR 投研请求说明：即便工具调用已经完成、`answer_preserved=true`，仍可能因 `committed terminal prefix mismatch` 被覆盖成通用研究失败。
- LITE 期权止盈请求曾在后续夜间窗口连续两轮复发同类 `committed terminal prefix mismatch`，且两轮都已有 `answer_preserved=true` 与 LITE 实体 contract。
- `answer_preserved=true` 后仍被 `committed terminal prefix mismatch` 改写成失败 final。
- 失败文案没有解释用户如何调整问题，也没有给出已有材料的部分结论。
- 同一会话连续两次复发，直到 compact 后才恢复。

## 修复情况

- `2026-07-25 07:02 CST` 真实运行态在 2026-07-24 23:30 代码级补强后复发，状态从 `Fixed` 回退为 `New`。最新样本未直接记录 `committed terminal prefix mismatch`，但 metadata 与用户可见症状仍是同一 Web direct terminal stream incomplete / finalization failure 家族，需要继续修复或拆分更精确根因。
- `2026-07-24 03:02 CST` 真实运行态复发，状态曾从 `Fixed` 回退为 `New`。
- `2026-07-24 23:30 CST` 代码级补强：`crates/hone-channels/src/agent_session/core.rs` 的 `recover_response_with_committed_prefix(...)` 现在除了原有“tail-only 正文补回已提交 prefix”外，还会识别并替换冲突的首行 `数据时间：...；行情口径：...` header，避免模型重新写了一条时间首行时被直接降级成通用研究失败。
- 已在 `crates/hone-channels/src/agent_session/core.rs` 为 committed prefix 收口新增恢复分支：
  - 终稿仍带已提交 prefix 时，继续只做原有前导空白对齐。
  - 终稿只剩非空正文 tail、未携带 prefix 时，改为恢复成 `committed prefix + 正文`，不再直接降级成通用研究失败。
  - 终稿首行若是另一条冲突的 `数据时间：...；行情口径：...`，则用已提交 prefix 替换该首行并保留正文 tail。
  - 若正文为空，或正文里出现无法安全归一的冲突 prefix，仍保持 fail-closed。
- 已在 `crates/hone-channels/src/agent_session/tests.rs` 补回归，覆盖：
  - committed prefix + tail-only 终稿恢复；
  - committed prefix + 冲突时间首行恢复；
  - 恢复后 Web direct 正常持久化 / 投递，不再落成 generic failure；
  - 既有 committed prefix 成功路径不回归。

## 验证

- `cargo test -p hone-channels committed_prefix_recovery_prepends_a_missing_prefix_only_for_tail_only_content --lib -- --nocapture`
- `cargo test -p hone-channels committed_prefix_recovery_replaces_a_conflicting_time_first_header --lib -- --nocapture`
- `cargo test -p hone-channels service_prefix_tail_only_final_response_is_recovered_without_generic_failure --lib -- --nocapture`
- `cargo test -p hone-channels service_prefix_conflicting_time_header_is_recovered_without_generic_failure --lib -- --nocapture`
- `cargo test -p hone-channels service_prefix_and_final_tail_are_visible_and_persisted_byte_identically --lib -- --nocapture`
- `cargo check -p hone-channels --tests`

## 用户影响

- 用户明确要求回答思考题，却连续两次没有得到答案，需要额外追问。
- 这是功能性缺陷：单轮 Web direct 任务被 terminal finalization 失败阻断。
- 定级为 `P2`：它阻断当前用户任务，但同窗未见长期 user-only 残留、错投、敏感信息泄露或全渠道不可用；仍不是 P1。

## 根因判断

- 直接根因是 Web direct agent finalization 的 terminal prefix 一致性检查失败后，系统选择提交失败前缀，而不是恢复 `answer_preserved=true` 的正文。
- `missing_explicit_seeds=456` 可能说明该轮被 entity / evidence contract 判定为缺少显式证券种子，但用户任务本身是文章观点解读，不一定需要强制证券实体 contract。
- 该缺陷不同于 `feishu_function_calling_max_iterations_generic_failure.md`：本轮没有 `max_iterations_exceeded:10`，失败发生在 terminal prefix / finalization 阶段。

## 后续观察

1. 继续观察同类 Web direct 长文解读 / 期权止盈问答，确认 live 路径不再把 answer-preserved 终稿降级成通用失败。
2. 若后续仍出现 `committed terminal prefix mismatch` 且正文为空，再单独排查上游 terminal synthesis / recovery 何时丢失正文。
