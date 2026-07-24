# Bug: Feishu 直聊自动 compact 后仍无法完成新话题回答，旧会话会反复卡在“仍无法继续”

- **发现时间**: 2026-04-18 00:20 CST
- **Bug Type**: System Error
- **严重等级**: P2
- **状态**: New

## 2026-07-24 运行态复核

- 状态维持 `New`，严重等级维持 `P2`。
- `2026-07-24 16:47-16:48 CST`，Feishu direct 旧会话 `Actor_feishu__direct__ou_5fa7fc023b9aa2a550a3568c8ffc4d7cdc` 收到短问 `分析一下INTC最新财报，并出操作建议`。
- `data/sessions.sqlite3` 显示本轮先写入 `Conversation compacted` 和 compact summary，并恢复多条 skill context；assistant final 于 `2026-07-24T16:48:52.779937+08:00` 仍只返回 `当前会话上下文过长。我已经自动尝试压缩历史，但这次仍无法继续。请直接继续提问重点、发送 <absolute-path>/compact，或开启一个新会话后再试。`
- 用户没有得到 INTC 财报分析或操作建议；`<absolute-path>/compact` 占位符仍进入用户可见失败文案。
- 判断：这是同一 Feishu direct 旧会话 compact / context-overflow recovery 路径复发，不新建重复缺陷。同窗其它 Web / Feishu direct 有 assistant 收口，未见错投、敏感信息泄露或全渠道不可用，因此仍为功能性 `P2`，非 P1。

## 2026-07-23 运行态复核

- 状态维持 `New`，严重等级维持 `P2`。
- `2026-07-23 06:35-06:36 CST`，Feishu direct 旧会话 `Actor_feishu__direct__ou_5fa7fc023b9aa2a550a3568c8ffc4d7cdc` 收到短问 `分析一下谷歌最新财报`。
- runtime 首轮已执行 `data_fetch search` 与 `data_fetch earnings_calendar`，随后记录 `context overflow detected, compacting and retrying`，并强制 compact 12 条历史消息。
- compact 后重试继续成功执行 `data_fetch search`、`data_fetch earnings_calendar GOOGL` 与 `web_search query="Alphabet Google GOOGL Q2 2026 earnings results financial performance"`。
- `2026-07-23 06:36:24` 日志记录 `entity_resolution.agent_loop ... answer_preserved=true` 后，仍以 `upstream HTTP 400: invalid params, request entity too large (2013)` 失败；assistant final 只返回 `抱歉，这次处理失败了。请稍后再试。`
- 判断：这是同一 compact / 旧上下文恢复路径复发，不新建重复缺陷。用户短问没有得到财报分析，但同窗其它会话可正常收口，未见错投、敏感信息泄露或全渠道不可用，因此仍为功能性 `P2`，非 P1。

## 2026-07-21 状态回退结论

- 状态从 `Fixed` 回退为 `New`，严重等级维持 `P2`。
- `2026-07-21 10:49-10:54 CST`，Feishu direct 旧会话 `Actor_feishu__direct__ou_5fa8018fa4a74b5594223b48d579b2a33b` 连续两次收到很短的新问题：
  - `2026-07-21T10:49:10.769196+08:00` 用户问“谷歌财报是什么时候”。
  - `2026-07-21T10:50:19.087095+08:00` assistant final 仍返回“当前会话上下文过长...仍无法继续”，且落库文本把建议命令写成 `发送 <absolute-path>/compact`。
  - `2026-07-21T10:53:26.208866+08:00` 用户几乎同题重试“谷歌财报是什么时候？”。
  - `2026-07-21T10:54:33.206495+08:00` assistant 再次返回同一失败文案和 `<absolute-path>/compact` 占位符。
- `data/runtime/logs/web.log.2026-07-21` 同窗显示第一轮先主动 compact 旧会话，然后执行 `data_fetch earnings_calendar`，检测到 `context overflow` 后又执行 `context_overflow_recovery` 强制 compact；重试阶段继续执行 `data_fetch search` 与 `data_fetch earnings_calendar GOOGL`，最终仍失败。第二轮同题重试也再次触发 context overflow recovery，成功执行 `data_fetch search` / `data_fetch earnings_calendar GOOGL` 后仍失败。
- 这晚于 2026-07-18 代码级修复，说明 compact 后剥离历史 tool transcript 的修复不足以稳定恢复旧 Feishu direct 会话的新话题短问答；同时 `<absolute-path>` 占位符外泄仍在用户可见 final 中复发。
- 这是功能性缺陷：用户提出的是单句财报日期查询，工具结果已经执行，但最终没有回答问题；有新会话绕行路径，未见错投、数据破坏或敏感信息泄露，因此维持 `P2`，非 `P1`，不创建 GitHub Issue。

## 2026-07-18 代码级修复

- `AgentSession` 的 context-overflow recovery 现在会在 compact 后的第二次 restore 中剥离历史 `tool` 与 `assistant.tool_calls` transcript，只保留 compact summary 与用户可见语义历史，避免旧会话里大体积本地检索 / 工具协议再次把 retry prompt 撑回 `context_window_exceeded` / `request entity too large (2013)`。
- 这次修复不改变正常 restore 路径，也不改变非 overflow 场景下的历史恢复语义；裁剪只发生在 `CONTEXT_OVERFLOW_POST_COMPACT_RESTORE_LIMIT` 的 recovery prompt。
- 新增回归 `context_overflow_retry_prunes_historical_tool_protocol_from_recovered_context`，验证 compact 后重试仍保留 summary 与历史 assistant 正文，但不再把旧 `tool` role 和搜索结果 payload 带回第二次请求。

## 2026-07-18 验证

- `cargo test -p hone-channels context_overflow_retry_prunes_historical_tool_protocol_from_recovered_context --lib -- --nocapture`
- `cargo test -p hone-channels context_overflow_ --lib -- --nocapture`
- `cargo check -p hone-channels --tests`

## 2026-07-18 状态更新

- 当前按代码级 `Fixed` 记录。
- 本轮未重启 live 服务，也没有用旧会话真实运行态再打一次 Feishu direct 样本，因此暂不更新为 `Closed`；后续若 2026-07-18 之后的真实旧会话 compact retry 继续复发，再按新样本回退为 `New`。

## 2026-07-13 状态回退结论

- 状态从 `Later` 调回 `New`，严重等级维持 `P2`。
- 15:01-19:01 CST `data/runtime/logs/web.log.2026-07-13` 显示同一 Feishu direct 旧会话 `Actor_feishu__direct__ou_5fa8018fa4a74b5594223b48d579b2a33b` 在 16:29 与 16:32 CST 两次触发 `context overflow detected, compacting and retrying`，并完成 `context overflow recovery compacted`。
- 两轮 compact 后仍分别落成失败，用户可见错误仍是 `当前会话上下文过长。我已经自动尝试压缩历史，但这次仍无法继续。请直接继续提问重点、发送 /compact，或开启一个新会话后再试。`
- 17:54 CST Feishu direct 生产探针 `Actor_feishu__direct__earnings_5fgate_5fprod_5fprobe_5f20260713_5fv2` 也在只执行 `data_fetch financials MU` 与 `data_fetch earnings_calendar MU` 后触发 context overflow；compact 后再次执行同类大结果工具，最终因 `request entity too large (2013)` 落为失败，并只持久化通用失败文案。
- 同窗 17:57 CST 新鲜 session `v3` 能通过 `data_fetch + web_search` 正常回答，说明问题集中在 compact / 大工具结果 / 旧上下文恢复路径，而不是 Feishu direct 全链路停摆。
- 这是功能性缺陷：用户在旧会话或 compact retry 路径中无法完成当前请求；但当前证据仍集中在单个旧会话和一个生产探针，存在新会话绕行路径，未见跨用户错投、数据破坏或敏感信息泄露，因此维持 `P2`，不是 `P1`，不创建 GitHub Issue。

## 修复进展（2026-04-26）

- 已确认 `CONTEXT_OVERFLOW_FALLBACK_MESSAGE` 的代码常量为用户态 `/compact` 文案，不再包含 `<absolute-path>` 占位符；最新线上样本里的占位符需要随新构建复核是否消失。
- 已将 Answer 阶段空成功重试耗尽路径改为 `success=false + error`，避免“有 search 结果但 answer 两次空回复”继续被上层当作正常成功。
- 已在共享净化层剥离独立 `Context compacted` / `Conversation compacted` marker 行，减少自动 compact 标记进入最终回复的概率。
- 状态调整为 `Later`：当前已完成可落地止血，不再占活跃修复队列；若旧会话在 compact 后仍稳定无法完成新话题，再改回 `New`。
- **证据来源**:
  - 2026-04-26 10:54-10:57 最新样本：
    - `session_id=Actor_feishu__direct__ou_5f2ccd43e67b89664af3a72e13f9d48773`
    - `2026-04-26T10:54:31.930285+08:00` 用户在旧直聊会话切到新问题：`你帮我分析一下英特尔，因为我想周一买一些，是否合适？`
    - `data/runtime/logs/sidecar.log` 记录本轮 search 先成功执行 `data_fetch snapshot INTC`、`local_list_files path="company_profiles"` 与 `data_fetch financials INTC`，随后 `2026-04-26T02:56:43.375084Z` 落成 `stage=search.done success=true iterations=3 tool_calls=3`
    - 紧接着 answer 阶段两次都再次记录 `stop_reason=end_turn success=true reply_chars=0`：
      - `2026-04-26T02:56:47.183635Z`
      - 首轮已触发 `empty successful response, retrying attempt=2/2`
    - `2026-04-26T02:57:03.474700Z` 重试后的 search 又回落成 `stage=search.done success=false iterations=2 tool_calls=2`
    - `2026-04-26T02:57:03.522915Z` 最终日志仍以 `当前会话上下文过长...请直接继续提问重点、发送 /compact，或开启一个新会话后再试。` 收口；`2026-04-26T10:57:03.554786+08:00` 的 `session_messages` / `sessions.last_message_preview` 却向用户写成 `发送 <absolute-path>/compact`
    - 结论：最新一小时内，同类旧会话失败不但仍然活跃，而且在“有完整 search 结果”的前提下依旧无法进入有效 answer，并继续外泄新的占位符格式
  - 2026-04-26 09:47-09:50 最新样本：
    - `session_id=Actor_feishu__direct__ou_5f62439dbed2b381c0023e70a381dbd768`
    - `2026-04-26T09:47:44.192950+08:00` 用户在旧会话里切到新问题：`今周美股是否需要减仓位？`
    - `2026-04-26T01:49:45.707117Z` 与 `2026-04-26T01:50:14.754148Z` 两次 answer 都再次记录 `stop_reason=end_turn success=true reply_chars=0`
    - 同轮 `sidecar.log` 记录 `empty successful response, retrying attempt=1/2` 与 `attempt=2/2`，随后 `2026-04-26T01:50:34.817286Z` search 重试仍落成 `success=false iterations=2 tool_calls=7`
    - `2026-04-26T01:50:34.841693Z` 日志最终仍以 `当前会话上下文过长。我已经自动尝试压缩历史，但这次仍无法继续。请直接继续提问重点、发送 /compact，或开启一个新会话后再试。` 收口
    - `data/sessions.sqlite3` 中同轮 assistant final 与 `sessions.last_message_preview` 最终却写成 `发送 <absolute-path>/compact`，说明旧缺陷不仅回归，用户可见 fallback 文案还出现了新的占位符泄露
  - `data/sessions.sqlite3` -> `session_messages`
    - 2026-04-18 22:47-22:58 最近一小时最新样本：
      - `session_id=Actor_feishu__direct__ou_5f44eaaa05cec98860b5336c3bddcc22d1`
      - `2026-04-18T22:47:53.149897+08:00` 用户先追问：`预判cai、tem近期的财报情况？`
      - `2026-04-18T22:49:08.243577+08:00` assistant 成功返回 `CAI/TEM` 财报预判，说明同一会话并非彻底坏死
      - `2026-04-18T22:51:47.988753+08:00` 用户继续追问：`预判crwv、nbis近期的财报情况？`
      - `2026-04-18T22:56:28.118999+08:00` assistant 再次成功返回 `CRWV/NBIS` 财报预判
      - `2026-04-18T22:56:48.845347+08:00` 用户紧接着切到新请求：`预判下Google近期的财报情况？`
      - `2026-04-18T22:57:29.142582+08:00` / `22:57:29.142598+08:00` 会话再次写入 `Conversation compacted` 与 `【Compact Summary】...`，summary 继续以 `role=system` 正常落库
      - `2026-04-18T22:58:07.274822+08:00` assistant 又回落成统一文案：`当前会话上下文过长。我已经自动尝试压缩历史，但这次仍无法继续。...`
      - 这说明缺陷在最近一小时仍活跃，但形态已从“会话持续卡死”演变为“同一旧会话里成功与 compact fallback 交替抖动”
    - 2026-04-18 21:43-21:45 最新样本：
      - `session_id=Actor_feishu__direct__ou_5f44eaaa05cec98860b5336c3bddcc22d1`
      - `2026-04-18T21:43:16.972+08:00` 用户在同一条 Feishu 直聊追问：`预判cai、tem近期的财报情况？`
      - `2026-04-18T21:44:35.880+08:00` 会话再次写入 `Conversation compacted` 与 `【Compact Summary】...`，summary 继续以 `role=system` 正常落库
      - `2026-04-18T21:45:08.918+08:00` assistant 仍返回同一条“当前会话上下文过长...仍无法继续”文案；这次失败对象已不再是 `UNH`，而是新的 `CAI/TEM` 财报预判请求
    - `session_id=Actor_feishu__direct__ou_5fba037d8699a7194dfe01a1fda5ced052`
    - 2026-04-18 最近一小时新增样本：
      - `2026-04-18T12:15:59.407329+08:00` 用户再次直接追问：`预测联合健康财报`
      - `2026-04-18T12:16:35.750413+08:00` / `12:16:35.750432+08:00` 会话再次写入 `Conversation compacted` 与 `【Compact Summary】...`，summary 继续以 `role=system` 正常落库
      - `2026-04-18T12:16:58.610053+08:00` assistant 第四次返回同一条“当前会话上下文过长...仍无法继续”文案，说明问题在最新一小时仍处于活跃状态
    - `2026-04-17T19:22:29.098516+08:00` 用户提问：`请预测联合健康财报会怎样？`
    - `2026-04-17T19:23:32.488338+08:00` assistant 返回：`当前会话上下文过长。我已经自动尝试压缩历史，但这次仍无法继续。请直接继续提问重点、发送 /compact，或开启一个新会话后再试。`
    - `2026-04-17T22:13:12.458203+08:00` 用户再次追问：`请预测联合健康这一季的财报会怎样？`
    - `2026-04-17T22:14:34.973152+08:00` assistant 再次返回同样的“仍无法继续”文案
    - 最近一小时最新样本：`2026-04-17T23:54:40.706923+08:00` 用户明确切换新话题：`开启新的话题：请预测联合健康的财报`
    - `2026-04-17T23:55:10.242164+08:00` / `23:55:10.242188+08:00` 会话写入 `Conversation compacted` 与 `【Compact Summary】...`，且 compact summary 已正确以 `role=system` 落库
    - `2026-04-17T23:55:32.986749+08:00` assistant 仍第三次返回同一条“当前会话上下文过长...仍无法继续”文案，用户始终没有拿到 `UNH` 财报预测结果
  - `data/runtime/logs/hone-feishu.release-restart.log`
    - `2026-04-18T14:56:48.848Z` 收到用户新请求 `预判下Google近期的财报情况？`
    - `2026-04-18T14:57:14.977Z` 首轮 search 落成 `stage=search.done success=false iterations=2 tool_calls=3`
    - `2026-04-18T14:57:15.046Z` 再次记录 `context overflow detected, compacting and retrying`
    - `2026-04-18T14:57:29.148Z` 记录 `context overflow recovery compacted=true`
    - `2026-04-18T14:57:36.946Z` compact 后重试再次执行 `local_search_files query="Google GOOGL Alphabet"`
    - `2026-04-18T14:57:37.128Z` 同轮记录 `tool_execute_error name=local_search_files error=IO 错误: stream did not contain valid UTF-8`
    - `2026-04-18T14:58:07.148Z` 重试后的 search 再次落成 `stage=search.done success=false iterations=2 tool_calls=2`
    - `2026-04-18T14:58:07.236Z` 最终仍以统一 fallback 收口，没有产出 `Google` 财报预判结果
    - `2026-04-18T13:43:51.261Z` 记录 `context overflow detected, compacting and retrying`
    - `2026-04-18T13:44:35.880Z` 记录 `context overflow recovery compacted=true`，说明这轮 `CAI/TEM` 新问题也确实又执行了一次自动 compact
    - `2026-04-18T13:44:47.820Z` compact 后重试仅重新执行 `data_fetch earnings_calendar`
    - `2026-04-18T13:45:08.834Z` 重试后的 search 再次落成 `stage=search.done success=false iterations=2 tool_calls=1`
    - `2026-04-18T13:45:08.918Z` 最终仍以同一条产品化 fallback 收口，没有产出 `CAI/TEM` 财报预判结果
    - `2026-04-18T04:16:22.952Z` 再次记录 `context overflow detected, compacting and retrying`
    - `2026-04-18T04:16:35.753Z` 记录 `context overflow recovery compacted=true`，说明本轮确实又执行了一次自动 compact
    - `2026-04-18T04:16:44.558Z` compact 后重试先执行 `local_search_files query="UnitedHealth UNH" path="company_profiles"`，立即报 `文件不存在: company_profiles`
    - `2026-04-18T04:16:58.491Z` 重试后的 search 再次落成 `stage=search.done success=false iterations=2 tool_calls=2`
    - `2026-04-18T04:16:58.585Z` 最终仍以同一条产品化 fallback 收口，没有产出 `UNH` 财报预测结果
    - `2026-04-17T15:54:44.989342Z` 同轮搜索阶段先执行 `local_search_files query="UnitedHealth UNH" path="company_profiles"`，立即报 `文件不存在: company_profiles`
    - `2026-04-17T15:54:59.747075Z` 记录 `context overflow detected, compacting and retrying`
    - `2026-04-17T15:55:10.246979Z` 记录 `context overflow recovery compacted=true`，本轮已完成自动 compact
    - `2026-04-17T15:55:32.938584Z` compact 后重试的 search 仍落成 `stage=search.done success=false iterations=2 tool_calls=3`
    - `2026-04-17T15:55:32.975618Z` 最终整轮以产品化失败文案收口，而不是输出用户请求的 `UNH` 财报预测
  - 已修复旧缺陷对照：
    - `docs/bugs/context_overflow_recovery_gap.md` 已说明“如果后续出现 compact 成功率不足，应单独登记新缺陷”

## 端到端链路

1. 用户在同一条 Feishu 直聊里连续追问联合健康（`UNH`）财报预测，并在最新一次显式说明“开启新的话题”。
2. 搜索阶段先尝试读取画像与行情信息，其中画像读取仍命中 `company_profiles` 路径错误。
3. runner 检测到上下文溢出后触发 `context_overflow_recovery`，确实执行了一次自动 compact 和重试。
4. 但 compact 后的重试仍没有完成 search/answer，最终只向用户返回“当前会话上下文过长...仍无法继续”的统一失败提示。
5. 当 compact 后的重试再次失败时，用户侧收到统一 fallback，而不是当前问题的实际答案。

## 期望效果

- 当会话进入新的独立话题时，自动 compact 应足以把旧上下文压缩到可继续回答的范围，而不是持续卡在 fallback 文案。
- 即使第一次自动 compact 后仍不足，也应尽量避免让同一 session 在相邻问题间出现“这轮成功、下一轮又失败”的抖动态。
- 用户明确说明“开启新的话题”后，系统应更积极地收缩旧上下文，优先完成当前问题，而不是反复要求用户再开新会话。

## 当前实现效果

- `2026-07-21` 运行态显示，自动 compact / context-overflow recovery 均确实执行，但两次短问题仍只给失败 fallback，没有消费已执行的 `data_fetch` 结果回答财报日期。
- 用户可见 fallback 仍包含 `<absolute-path>/compact` 占位符，说明失败文案净化或常量替换仍有运行态漏网。
- 旧的“底层报错外泄”问题已经修复，最新会话里用户看到的是产品化提示，而不是 provider 原始错误。
- 但这轮真实样本证明，自动 compact 只是把失败文案变得可接受，并没有稳定恢复主功能链路。
- 同一个 `UNH` 话题在 `19:22`、`22:13`、`23:54` 三次尝试中都没有产出答案，说明这不是单次 provider 抖动。
- `2026-04-18 12:15` 的第四次复现说明，这种粘滞失败已经跨越到第二天中午；即使用户把问题压缩成更短的 `预测联合健康财报`，系统依然在 compact 后停在相同 fallback。
- `2026-04-18 21:45` 的最新样本进一步说明，这种粘滞失败并不依赖 `UNH` 这个具体主题；同一会话切到新的 `CAI/TEM` 财报预判后，compact 成功执行但 search 仍再次失败，用户继续只拿到统一 fallback。
- `2026-04-18 22:47-22:58` 的最新小时窗又说明，缺陷已经表现为“同一旧会话内成功与失败交替”而不是彻底卡死：`CAI/TEM` 与 `CRWV/NBIS` 已能答出，但紧接着 `Google` 财报预判又在 compact 后回落成统一 fallback。
- 最新一轮 `23:55` 的 compact summary 已经是 `role=system`，表明这也不是旧的 compact summary 污染回灌问题原样回归。

## 用户影响

- 这是功能性缺陷，不是单纯文案或质量波动。用户明确提出了 `UNH` 财报预测请求，但连续三次都没有得到答案。
- 用户虽然收到了友好提示，不再暴露内部错误细节，但主任务仍会在部分新问题上直接失败，只能被迫重试、切换话题或开启新会话。
- 之所以定级为 `P2` 而不是 `P1`，是因为当前证据集中在单会话粘滞失败，仍有“开新会话”这一绕行路径，也没有发现误投递、数据损坏或跨用户影响。

## 根因判断

- 最新样本更集中地指向 context-overflow recovery 后的 retry 上下文仍过大或工具结果再次膨胀：两轮都能进入工具调用并拿到 earnings calendar / search 结果，但在 answer 前后仍被统一失败路径吞掉。
- `<absolute-path>/compact` 复发说明一部分旧 fallback 文案或落库净化链路仍没有使用 2026-04-26 之后的用户态 `/compact` 文案。
- 旧缺陷修复后，`AgentSession` 已能识别超窗并自动 compact；当前问题更像是 compact 粒度和保留窗口仍不足以让新话题顺利脱离旧上下文负担。
- 最新日志里 compact 后仍保留 6 条 recent items，且重试 search 继续携带画像读取与多次工具调用，说明 prompt 体积或上下文噪声在 retry 后仍可能超出可用预算。
- `company_profiles` 路径错误同时出现在这轮重试前，说明无效工具尝试也在放大 search 阶段的上下文和耗时，但它更像放大器，不足以单独解释“连续三次都无法完成回答”。
- `2026-04-18 21:45` 这轮没有再次命中 `company_profiles`，但 compact 后 search 依旧失败，说明该缺陷已经不再只依赖画像路径错误放大；更底层的问题仍是 compact 后保留窗口与重试搜索预算不足。
- `2026-04-18 22:57` 的最新失败又新增了 `local_search_files ... valid UTF-8` 读失败，说明旧会话里的本地检索异常也会继续放大 compact 重试链路，但当前仍更像放大器而不是唯一根因。
- 因此当前更可能是“会话瘦身策略不足 + 多代理搜索重试后上下文再膨胀 + 本地检索异常放大”的组合问题，而不是单一 provider 临时抖动。

## 下一步建议

- 优先审视 `context_overflow_recovery` 在 direct session 中的保留窗口、summary 长度与重试策略，确认简单新问题是否应丢弃更多旧活跃窗口和旧 skill context。
- 为“compact 成功但 retry 仍失败”的路径补独立可观测标记，区分是真正再次超窗、search 早停，还是 answer 阶段被短路。
- 修正所有用户可见 overflow fallback 来源，确保不再出现 `<absolute-path>/compact` 占位符。
- 给直聊场景补一条回归验证：同一 session 在长历史后切到新话题时，自动 compact 后仍应能完成至少一条新问题答复，而不是长期卡在统一 fallback。

## 2026-04-26 状态回退结论

- `2026-04-20` 的“已修复”结论不能继续成立：最新旧会话 `Actor_feishu__direct__ou_5f62439dbed2b381c0023e70a381dbd768` 在新问题 `今周美股是否需要减仓位？` 上再次复现 compact 后仍失败。
- `2026-04-26 10:57` 的 `INTC` 样本进一步说明，问题已经不是“search 直接失败才触发 fallback”这么简单：本轮 search 先完整成功，answer 两次都空返回，随后才二次回落成 compact fallback。
- 这次失败不是历史脏样本残留：
  - 会话先完成 auto compact；
  - 两次 answer 都走到 `reply_chars=0`；
  - 最终重试后的 search 仍 `success=false`，用户只收到“当前会话上下文过长”的 fallback。
- 同轮用户可见文案还把建议命令写成了 `发送 <absolute-path>/compact`，因此当前既有主功能失败，也有附带的格式占位符泄露。
- 因此本单状态从 `Fixed` 调整回 `Fixing`，严重等级继续保持 `P2`。
