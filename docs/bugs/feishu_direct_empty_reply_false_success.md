# Bug: Feishu 直聊会话在 Multi-Agent Answer 阶段返回空回复后，链路仍记成功并发送空消息

- **发现时间**: 2026-04-15 18:02 CST
- **Bug Type**: System Error
- **严重等级**: P1
- **状态**: Fixing
- **证据来源**:
  - 2026-04-26 09:52-09:57 最新真实直聊样本：
    - `session_id=Actor_feishu__direct__ou_5f39103ac18cf70a98afc6cfc7529120e5`
    - `2026-04-26T09:52:27.080336+08:00` 用户提问：`我的定时任务`
    - 同轮 `sidecar.log` 在 `2026-04-26T01:55:08.948024Z`、`2026-04-26T01:56:03.346475Z`、`2026-04-26T01:57:06.850713Z` 连续三次记录 `stop_reason=end_turn success=true reply_chars=0`
    - `2026-04-26T01:55:08.949266Z` 与 `2026-04-26T01:56:03.348988Z` 两次触发 `empty successful response, retrying`，最终在 `2026-04-26T01:57:06.852243Z` 落成 `empty successful response persisted as fallback`
    - `2026-04-26T09:57:06.856636+08:00` assistant 最终落库并发送的仍是通用 fallback：`这次没有成功产出完整回复。我已经自动重试过了，请再发一次，或换个问法。`
    - 同轮 `MsgFlow/feishu done ... success=true ... tools=7(data_fetch) reply.chars=35` 与 `step=reply.send ... segments.sent=1/1` 仍然存在，说明普通用户主动提问在执行 7 次行情工具后仍被伪成功遮蔽
  - 2026-04-26 08:35-08:38 最新真实直聊样本：
    - `session_id=Actor_feishu__direct__ou_5fe40dc70caa78ad6cb0185c21b53c4732`
    - `2026-04-26T08:35:31.796308+08:00` 用户追问：`比较下港股asmpt 太平洋和建滔集团`
    - 同轮 `sidecar.log` 先后记录 `2026-04-26T08:37:04.308008+08:00` 与 `2026-04-26T08:37:54.760643+08:00` 两次 `stop_reason=end_turn success=true reply_chars=0`
    - `2026-04-26T08:38:13.702451+08:00` 日志记录 `empty successful response persisted as fallback`
    - `2026-04-26T08:38:13.704613+08:00` assistant 最终落库并发送的仍是通用 fallback：`这次没有成功产出完整回复。我已经自动重试过了，请再发一次，或换个问法。`
    - 同轮 `MsgFlow/feishu done ... success=true ... reply.chars=35` 与 `step=reply.send ... segments.sent=1/1` 依旧存在，说明真实用户主动提问的主链路继续被伪成功遮蔽
  - 2026-04-23 18:53-18:55 最新真实直聊样本：
    - `session_id=Actor_feishu__direct__ou_5f5ffb1004abf2c344917ee093ffb14c15`
    - `2026-04-23T18:54:21.472617+08:00` 用户追问：`你能收到我刚才发给你的Md文件吗？`
    - `2026-04-23T18:54:44.823892+08:00` assistant 先回复“当前附件目录里只有之前的图片，没有新的 Markdown 文件落进来”，说明这一轮没有观察到新的附件落库。
    - 用户随后在 `2026-04-23T18:55:43.264137+08:00` 仅补一句 `这个`，`sidecar.log` 记录 `transitional planning sentence detected, treating as empty ... chars=124`，随即 `step=agent.run.fallback ... detail=planning_sentence_suppressed`。
    - `2026-04-23T18:55:55.196669+08:00` assistant 最终再次落库并发送通用 fallback：`这次没有成功产出完整回复。我已经自动重试过了，请再发一次，或换个问法。`
    - 同轮 `step=direct.busy ... sent` 说明用户在等系统确认“文件是否收到”时，追问被 busy 拦截后又落入同一空/无效 Answer 收口；用户无法继续推进排查。
  - 2026-04-23 10:36 最新真实直聊样本：
    - `session_id=Actor_feishu__direct__ou_5f680322a6dcbc688a7db633545beae42c`
    - `2026-04-23T10:36:51.771181+08:00` 用户输入：`帮我建腾讯控股ADR的画像`
    - `data/runtime/logs/sidecar.log` 同轮记录已实际执行 `mkdir -p company_profiles/tencent-holdings-adr/events`，并在 `2026-04-23 10:38:42` 通过 `Edit` 写入：
      - `data/agent-sandboxes/feishu/direct__ou_5f680322a6dcbc688a7db633545beae42c/company_profiles/tencent-holdings-adr/profile.md`
      - `data/agent-sandboxes/feishu/direct__ou_5f680322a6dcbc688a7db633545beae42c/company_profiles/tencent-holdings-adr/events/2026-04-23-initial-profile.md`
    - 但 `2026-04-23 10:38:47.439` 随后记录 `transitional planning sentence detected, treating as empty runner=codex_acp ... chars=43`，紧接着 `step=agent.run.fallback ... detail=planning_sentence_suppressed`。
    - `2026-04-23T10:38:47.440223+08:00` assistant 最终落库并发送的是通用 fallback：`这次没有成功产出完整回复。我已经自动重试过了，请再发一次，或换个问法。`
    - 同轮 `MsgFlow/feishu done ... success=true ... tools=15(...) reply.chars=35`，说明用户任务的业务副作用已发生，但用户侧看到的是“失败/请重试”，会误导用户以为画像没有创建；这是 Answer 空/无效成功根因的新形态，不是独立新缺陷。
  - 2026-04-21 23:34 最新真实直聊样本：
    - `session_id=Actor_feishu__direct__ou_5f01b20218487e01a6d48c881ce6893123`
    - `2026-04-21T23:34:43.526597+08:00` 用户只问：`你在吗`
    - `2026-04-21T23:34:58.039230+08:00` assistant 最终落库为通用 fallback：`这次没有成功产出完整回复。我已经自动重试过了，请再发一次，或换个问法。`
    - `data/runtime/logs/sidecar.log` 同步记录 `2026-04-21 23:34:58.038` `transitional planning sentence detected, treating as empty runner=codex_acp ... chars=69`
    - 这说明用户侧零字节外发仍被 fallback 止血，但 Answer 阶段仍会把无效/过渡性输出判成空结果；即使是最简单的在线确认问题，也会退化成“没成功产出完整回复”。
  - 最近一小时真实会话：`data/sessions.sqlite3` -> `session_messages`
    - `session_id=Actor_feishu__direct__ou_5ff08d714cd9398f4802f89c9e4a1bb2cb`
    - `2026-04-15T17:45:15.399804+08:00` 用户消息要求比较 `AT&T`、`T-Mobile`、`VSAT`、`IRDM`、`GSAT`、`ASTS` 在 2025 年财报中的手机通信收入，并输出投资报告格式的表格和文字分析
    - `2026-04-15T17:49:05.656906+08:00` 到 `2026-04-15T17:49:05.706807+08:00` 连续 12 次 `data_fetch` 工具成功返回
    - `2026-04-15T17:49:05.708643+08:00` assistant 消息长度为 `0`
    - 该空 assistant 消息仍落库了真实 `message_id=om_x100b52c1aca3f51cc3d6e91f9c1817a`
  - 最近一小时再次复现：
    - `session_id=Actor_feishu__direct__ou_5f3f69c84593eccd71142ed767a885f595`
    - `2026-04-15T21:30:00.519354+08:00` 定时任务 `Oil_Price_Monitor_Premarket` 触发后完成 16 次 `data_fetch/web_search`
    - `2026-04-15T21:34:59.005675+08:00` assistant 消息长度为 `0`
    - `data/runtime/logs/web.log` 对应记录：`21:34:58.950` `reply_chars=0`、`21:34:58.951` `empty reply`、`21:34:59.008` `done ... success=true ... reply.chars=0`
    - `session_id=Actor_feishu__direct__ou_5ff08d714cd9398f4802f89c9e4a1bb2cb`
    - `2026-04-15T21:49:18.094709+08:00` 用户再次提问“你是一个顶级金融分析师，帮我分析美光和闪迪”
    - `2026-04-15T21:52:43.409827+08:00` assistant 消息再次为空，且落库了新 `message_id=om_x100b52c7740cf850c4c79e49f6f1342`
  - 最近一小时运行日志：`data/runtime/logs/hone-feishu.release-restart.log`
    - `2026-04-15T09:48:56.045195Z` `multi_agent.search.done success=true iterations=3 tool_calls=12`
    - `2026-04-15T09:49:05.651091Z` `stop_reason=end_turn success=true reply_chars=0`
    - `2026-04-15T09:49:05.651145Z` `empty reply (stop_reason=end_turn), no stderr captured`
    - `2026-04-15T09:49:05.651283Z` `multi_agent.answer.done success=true`
    - `2026-04-15T09:49:05.712429Z` `MsgFlow/feishu ... success=true ... reply.chars=0`
    - `2026-04-15T09:49:08.365202Z` `step=reply.send ... detail=segments.sent=1/1`
  - 会话汇总记录：`data/sessions.sqlite3` -> `sessions`
    - `updated_at=2026-04-15T17:49:05.708644+08:00`
    - `last_message_at=2026-04-15T17:49:05.708643+08:00`
    - `last_message_preview` 为空
  - 2026-04-16 12:12-12:22 最近一小时回归复现：
    - `session_id=Actor_feishu__direct__ou_5f0e57a9914d61ae96d437cdeb65e43593`
    - `2026-04-16T12:08:38.607636+08:00` 用户提问：`亚马逊新推出的太空60，哪几家最值得投资`
    - 搜索阶段完成 8 次 `data_fetch/web_search`，但 `2026-04-16T12:12:57.027055+08:00` assistant 再次落库为空字符串
    - `data/runtime/logs/web.log` 对应记录：`12:12:56.976` `empty reply (stop_reason=end_turn)`，`12:12:57.034` `done ... success=true ... reply.chars=0`
    - `sessions.last_message_preview` 长度仍为 `0`，说明链路把空回复当作成功完成并落为会话最后一条消息
    - `session_id=Actor_feishu__direct__ou_5f2ccd43e67b89664af3a72e13f9d48773`
    - `2026-04-16T12:21:48.953881+08:00` 用户提问：`看看我的15支股票池的击球区和买卖评估`
    - 搜索阶段完成 `portfolio + local_list_files + 4 次 data_fetch` 共 7 次工具调用后，`2026-04-16T12:22:44.064368+08:00` assistant 再次为空
    - `data/runtime/logs/web.log` 对应记录：`12:22:44.048` `stop_reason=end_turn success=true reply_chars=0`、`12:22:44.048` `empty reply`、`12:22:44.065` `done ... success=true ... reply.chars=0`、`12:22:44.921` `step=reply.send ... segments.sent=1/1`
    - 两条会话都发生在此前标记“已修复”之后，说明空回复伪成功仍是当前真实用户链路中的活跃缺陷，而不是历史遗留记录
  - 对比同一时间窗异常样式：
    - 最近 90 分钟内仅发现两条空 assistant 消息，另一条是已单独登记的 `discord_scheduler_empty_reply_send_failed`
    - 本次是 Feishu 直聊、非 scheduler、用户主动提问链路，影响范围与已有 Discord 定时任务缺陷不同

## 端到端链路

1. Feishu 用户在直聊中发起投研分析请求，要求输出表格和成文报告。
2. Multi-Agent 搜索阶段完成 12 次 `data_fetch`，上下文材料已经齐备。
3. Answer 阶段的 `opencode_acp` 以 `stop_reason=end_turn` 结束，但最终回复为空字符串。
4. 多代理链路仍把本轮 Answer 记为 `success=true`，随后主消息流继续持久化空 assistant 消息。
5. Feishu 发送链路继续执行 `reply.send segments.sent=1/1`，用户侧实际收到的是空消息，本轮任务等同未完成。

## 期望效果

- 用户主动提问的直聊会话在工具阶段成功后，应产出非空最终答复，至少满足基本可读性与结构要求。
- 一旦 Answer 阶段返回空字符串，链路应中止发送并把本轮执行明确标记为失败，而不是继续写入成功状态。
- 日志和落库记录应保留足够的错误摘要，避免出现“已成功发送但没有任何内容”的伪成功。

## 当前实现效果

- `2026-04-26 08:35` 的最新样本说明，即使用户只是发起一条普通的港股对比请求，链路仍会在两次 answer 都 `reply_chars=0` 后直接退化成通用 fallback；这不是“复杂画像/附件排查”的特例。
- `2026-04-21 23:34` 的最新样本说明，问题已经从“空字符串直接外发”缓解为“无效 Answer 被判空并返回 fallback”，但底层仍不能稳定为简单直聊生成可消费答复。
- `2026-04-23 10:36` 的最新样本进一步说明，fallback 止血会掩盖已经发生的业务副作用：画像文件实际已创建，但最终可见回复仍被替换成“没有成功产出完整回复”，用户无法确认任务完成情况，甚至可能重复请求造成画像重复写入或状态混乱。
- 真实会话已经证明：Feishu 直聊在搜索结果齐备的前提下，仍可能产出零字节 assistant 消息。
- `opencode_acp` 日志明确识别到 `empty reply`，但 `multi_agent.answer.done`、`MsgFlow/feishu done` 和 `reply.send` 仍全部走成功路径。
- 数据库最终同时留下“有真实消息 ID”和“assistant 内容为空”这两个互相矛盾的结果，说明空消息并未被链路拦截。
- 同一根因在最近一小时内至少再次影响了 2 条 Feishu 会话，其中一条是用户主动追问后的直聊主链路，一条是 Feishu 定时任务会话，说明问题不是单次偶发抖动。
- 2026-04-16 12:12 与 12:22 的两条新会话进一步证明：即便搜索阶段已经拿到完整数据，Answer 阶段仍会以 `stop_reason=end_turn` 产出空正文，而上层继续把这一轮记为 `success=true`。
- 这意味着此前“空成功判定已收紧”的修复结论并未稳定覆盖当前 Feishu 直聊链路，该缺陷应从 `Fixed` 恢复为活跃状态。

## 用户影响

- 这是功能性缺陷，不是单纯回答质量波动。用户明确要求的投研报告完全没有返回，任务实际失败。
- 问题发生在 Feishu 直聊主链路，而不是边缘后台任务，直接影响用户能否完成一次正常问答，因此定级为 `P1`。
- 该问题不属于 `P3` 质量类问题，因为它不是“答得不够好”，而是最终根本没有可消费内容。

## 根因判断

- `opencode_acp` 能识别 `reply_chars=0` 和 `empty reply`，但当前没有把这类结果升级为硬失败。
- 多代理封装层把空回复继续当作 `answer.done success=true`，导致上层消息流无法区分“正常完成”和“零字节完成”。
- Feishu 发送侧只看分段流程是否跑完，没有拦截空正文，因此把空 assistant 消息照常投递。

## 修复情况（2026-04-16）

- 已在 `crates/hone-channels/src/agent_session.rs` 收紧空成功判定：
  - `should_return_runner_result(...)` 不再把“只有 `tool_calls_made`、但正文为空”的结果视为有效成功
  - `run_runner_with_empty_success_retry(...)` 现在会对这类结果继续重试，重试耗尽后落回非空的 `EMPTY_SUCCESS_FALLBACK_MESSAGE`
- 这意味着即使多代理把搜索阶段的工具调用合并进最终 response，也不会再让空 answer 绕过兜底逻辑，Feishu 直聊不再写入或发送零字节 assistant 消息。

## 修复结论复核（2026-04-16 13:01 CST）

- 最近一小时的两条真实 Feishu 直聊会话已经证明，上述修复结论不能成立为“已修复”：
  - `Actor_feishu__direct__ou_5f0e57a9914d61ae96d437cdeb65e43593`
  - `Actor_feishu__direct__ou_5f2ccd43e67b89664af3a72e13f9d48773`
- 两条会话都在 `opencode_acp` 明确记录 `empty reply` 后，仍然继续走到了 `success=true`、`reply.chars=0` 与 `reply.send segments.sent=1/1`。
- 因此本缺陷状态恢复为 `New`；后续需要重新核对当前空成功判定是否只覆盖了部分 runner/response 形态，或在最新启动配置下出现了回归路径。

## 下一步建议

- 重新比对 `reply_chars=0` 但 `success=true` 的最新日志路径，确认当前 Feishu 直聊链路为何没有落到 `EMPTY_SUCCESS_FALLBACK_MESSAGE`。
- 在 bug 修复前，继续把 `reply.chars=0`、`empty reply`、`segments.sent=1/1` 组合视为高优先级回归信号；若 scheduler 或其它渠道也出现同类模式，再分别更新对应文档状态。

## 回归验证

- `cargo test -p hone-channels should_return_runner_result_ -- --nocapture`
- `cargo test -p hone-channels empty_success_with_tool_calls_uses_fallback_after_retries -- --nocapture`
- `cargo check -p hone-channels`
- `rustfmt --edition 2024 --check crates/hone-channels/src/agent_session.rs`

## 当前修复进展（2026-04-17 10:40 CST）

- `crates/hone-channels/src/agent_session.rs` 已补“净化后为空”的成功收口：即便 runner 表面 `success=true`，只要用户可见正文在净化后为空，也会改写为 `EMPTY_SUCCESS_FALLBACK_MESSAGE`，不再持久化空 assistant。
- 同轮还补了 `bins/hone-feishu/src/outbound.rs` 的 Feishu `update/reply` 失败回退，避免“session 已有非空 fallback，但 placeholder 更新失败导致用户侧继续只看到空结果/旧占位”。
- 自动化验证已通过：
  - `cargo test -p hone-channels`
  - `cargo test -p hone-feishu`
- 由于当前还缺少新的真实 Feishu 回归样本，本单状态先更新为 `Fixing` 而不是 `Fixed`；下一条真实直聊若不再出现 `reply.chars=0 + success=true`，再考虑关闭。

## 最新真实样本复核（2026-04-19 23:10 CST）

- `data/sessions.sqlite3` -> `session_messages`
  - `session_id=Actor_feishu__direct__ou_5f1ed3244e3a7b34789cea10eeabe4da98`
  - `2026-04-19T22:57:43.695601+08:00` 用户提问：`闪迪还能涨到多少`
  - `2026-04-19T22:59:24.453973+08:00` assistant 最终落库为通用 fallback：`这次没有成功产出完整回复。我已经自动重试过了，请再发一次，或换个问法。`
- `data/runtime/logs/web.log`
  - `2026-04-19 22:58:09.561` 到 `22:58:09.569` 首轮 search 中连续触发 `Tool: hone/local_list_files`、3 次 `Tool: hone/local_search_files`，并全部记录 `runner.stage=acp.tool_failed`
  - `2026-04-19 22:58:16.866` 记录 `empty successful response, retrying ... attempt=1/2`
  - `2026-04-19 22:58:52.632` 再次记录 `empty successful response, retrying ... attempt=2/2`
  - `2026-04-19 22:59:24.445` 最终记录 `empty successful response persisted as fallback` 与 `step=agent.run.fallback ... detail=empty_success_exhausted`
  - `2026-04-19 22:59:24.456` 同轮 `done ... success=true ... reply.chars=35`
  - `2026-04-19 22:59:25.476` `step=reply.send ... detail=segments.sent=1/1`

## 当前修复结论（2026-04-19 23:10 CST）

- 这条最新真实样本说明，`2026-04-17` 的止血修补已经覆盖了“零字节 assistant 直接落库/外发”的用户侧坏态：
  - 最新会话没有再出现 `reply.chars=0`、空 assistant 落库或空分段发送。
  - Feishu 用户侧收到的是非空 fallback，而不是空白消息。
- 但底层 `empty_success` 根因并没有消失：
  - `codex_acp` 在同一条简单个股问题上仍连续两次返回空成功，最终只能靠 `EMPTY_SUCCESS_FALLBACK_MESSAGE` 收口。
  - 同轮还伴随多次 `local_list_files/local_search_files` 失败，说明当前 answer/search 组合仍会把“已有部分工具动作但无最终正文”的坏态带到生产。
- 因此本单继续维持 `Fixing`：
  - “空消息伪成功”这一原始用户侧症状已被止血，不再适合回退到 `New`；
  - 但“runner 空成功仍活跃，只是改由 fallback 遮蔽”的根因仍未修复，暂不能转为 `Fixed`。

## 最新真实样本复核（2026-04-22 00:00 CST）

- 本轮巡检没有发现 `reply.chars=0 + segments.sent=1/1` 的新空消息外发样本。
- 但 `2026-04-21 23:34` 的 `你在吗` 会话证明底层坏态仍活跃：`codex_acp` 产出的 69 字过渡性文本被 `transitional planning sentence detected` 判空，最终只能给用户返回通用 fallback。
- 这不是新的独立缺陷，而是同一 Answer 空/无效成功根因的新表现：用户侧不再收到空白消息，但仍没有拿到对简单问题的正常回答。
- 因此本单继续维持 `Fixing`，不转 `Fixed`。

## 最新真实样本复核（2026-04-23 11:03 CST）

- 本轮巡检没有发现新的 `reply.chars=0 + segments.sent=1/1` 空白外发样本，说明用户侧零字节消息止血仍成立。
- 但 `2026-04-23 10:36` 的腾讯控股 ADR 画像样本证明，`planning_sentence_suppressed` 仍会把真实任务收口成通用失败 fallback：
  - 业务动作已经执行，`profile.md` 和事件文件已写入 actor sandbox；
  - 最终用户只收到“这次没有成功产出完整回复”，无法知道画像已经创建；
  - 主流程仍记录 `success=true` 与 `reply.chars=35`。
- 因此本单继续维持 `Fixing`。当前待修范围不只是“避免空白消息”，还需要让空/无效 Answer 的 fallback 与真实业务副作用一致：如果任务已完成，应给出完成确认；如果不能确认，应避免把已执行写操作伪装成纯失败重试。

## 最新真实样本复核（2026-04-23 19:01 CST）

- `2026-04-23 18:53-18:55` 的 Feishu 直聊会话 `Actor_feishu__direct__ou_5f5ffb1004abf2c344917ee093ffb14c15` 再次证明，`planning_sentence_suppressed` 不是只会出现在“画像写文件”这类复杂任务里：
  - 18:54 用户只是确认 `.md` 文件是否送达，系统给出“当前附件目录里没有新的 Markdown 文件”的正常答复；
  - 18:55 用户补一句 `这个` 想继续指认问题，日志随即记录 `transitional planning sentence detected ... chars=124`；
  - 最终仍被收口成通用 fallback，用户看不到任何与“文件/附件链路”相关的可操作确认。
- 这说明当前坏态已经影响到故障排查类对话本身：即使前一轮已经定位到“没看到附件”，后续一句极短澄清也可能被当成过渡句吞掉，导致用户只能反复重发或改问法。
- 用户侧“零字节消息不再外发”的止血仍成立，但真实会话可见层仍无法稳定完成简单跟进问答，因此本单继续维持 `Fixing`。

## 最新真实样本复核（2026-04-26 08:38 CST）

- `Actor_feishu__direct__ou_5fe40dc70caa78ad6cb0185c21b53c4732` 说明该根因在最新一小时仍活跃于普通用户主动提问主链路：
  - 用户问题是简单的 `比较下港股asmpt 太平洋和建滔集团`；
  - 两次 answer 都以 `reply_chars=0` 结束，最终只能靠通用 fallback 收口；
  - `done ... success=true ... reply.chars=35` 与 `reply.send segments.sent=1/1` 仍把本轮记为表面成功。
- 这说明本单当前待修范围仍然包括“让简单直聊稳定给出真实答案”，而不只是“避免零字节消息外发”。
- 因此本单继续维持 `Fixing`，严重等级继续保持 `P1`。

## 最新真实样本复核（2026-04-26 09:57 CST）

- `Actor_feishu__direct__ou_5f39103ac18cf70a98afc6cfc7529120e5` 说明该根因在最新一小时仍活跃于普通用户主动提问主链路，而且不是“无工具的简单问候”才会失败：
  - 用户问题是 `我的定时任务`；
  - search 阶段实际执行了 7 次 `data_fetch quote`；
  - answer 阶段连续 3 次都以 `reply_chars=0` 结束，最终只能靠通用 fallback 收口；
  - `done ... success=true ... reply.chars=35` 与 `reply.send segments.sent=1/1` 仍把本轮记为表面成功。
- 这说明当前坏态既能出现在无工具问答，也能出现在“搜索已拿到行情结果”的主动查询里；用户仍无法稳定拿到真实答复。
- 因此本单继续维持 `Fixing`，严重等级继续保持 `P1`。

## 最新真实样本复核（2026-04-26 13:11 CST）

- `Actor_feishu__direct__ou_5f39103ac18cf70a98afc6cfc7529120e5` 在最近一小时再次复现同一坏态，而且是同一真实用户在刚失败过“我的定时任务”之后继续追问：
  - `2026-04-26T13:10:39.109557+08:00` 用户提问：`我现在有哪些定时任务`
  - `data/runtime/logs/sidecar.log` 在 `2026-04-26T05:10:54.780804Z`、`2026-04-26T05:11:10.384535Z`、`2026-04-26T05:11:27.813764Z` 连续三次记录 `stop_reason=end_turn success=true reply_chars=0`
  - 同轮 `2026-04-26T05:10:54.781732Z`、`2026-04-26T05:11:10.385109Z` 先后两次进入 `empty successful response, retrying`，最终在 `2026-04-26T05:11:27.814273Z` 落成 `empty successful response persisted as fallback`
  - `2026-04-26T13:11:27.817153+08:00` assistant 最终落库并发送的仍是通用 fallback：`这次没有成功产出完整回复。我已经自动重试过了，请再发一次，或换个问法。`
  - 第一轮 answer 之前日志还记录了 `runner.stage=multi_agent.search.start`；整个坏态发生在用户主动追问“列出现有定时任务”的主链路，而不是后台 scheduler 噪音
- 这条 13:10-13:11 的新样本说明，当前根因不仅在一个小时内持续活跃，而且会对同一会话里的连续追问形成“失败后再失败”的粘滞态：
  - 09:52 的 `我的定时任务` 已失败一次；
  - 13:10 的 `我现在有哪些定时任务` 再次命中三次 `reply_chars=0`；
  - 用户在同一会话里仍拿不到任何可消费的任务列表答复。
- 因此本单继续维持 `Fixing`，严重等级继续保持 `P1`；本轮没有发现新根因，仍属于既有 `empty_success_exhausted` 活跃复现。

## 下一步建议（更新于 2026-04-19 23:10 CST）

- 把 `empty_success_exhausted` 视为当前主监控信号，而不再只盯 `reply.chars=0`；否则会误判为已彻底收口。
- 继续区分两层结论：
  - 用户侧止血是否成立：看是否还出现空 assistant / 空分段发送。
  - 根因是否修复：看 `empty successful response` 重试与 `persisted as fallback` 是否仍在真实会话里出现。
- 结合同轮 `local_list_files/local_search_files` 连续失败链路排查，为何简单个股问答仍会在有 `data_fetch` 的前提下走到空成功收口，而不是形成可消费答案。
