# Bug: Feishu 直聊会话在 Multi-Agent Answer 阶段返回空回复后，链路仍记成功并发送空消息

- **发现时间**: 2026-04-15 18:02 CST
- **Bug Type**: System Error
- **严重等级**: P1
- **状态**: Fixing
- **证据来源**:
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

## 下一步建议（更新于 2026-04-19 23:10 CST）

- 把 `empty_success_exhausted` 视为当前主监控信号，而不再只盯 `reply.chars=0`；否则会误判为已彻底收口。
- 继续区分两层结论：
  - 用户侧止血是否成立：看是否还出现空 assistant / 空分段发送。
  - 根因是否修复：看 `empty successful response` 重试与 `persisted as fallback` 是否仍在真实会话里出现。
- 结合同轮 `local_list_files/local_search_files` 连续失败链路排查，为何简单个股问答仍会在有 `data_fetch` 的前提下走到空成功收口，而不是形成可消费答案。
