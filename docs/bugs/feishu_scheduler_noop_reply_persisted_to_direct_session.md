# Bug: Feishu scheduler 命中 `skip_signal` 后仍把未发送长文落进 direct session，污染后续上下文

- **发现时间**: 2026-04-26 00:12 CST
- **Bug Type**: System Error
- **严重等级**: P2
- **状态**: Fixed
- **证据来源**:
  - 最近一小时真实会话：`data/sessions.sqlite3` -> `session_messages`
    - `session_id=Actor_feishu__direct__ou_5fa8018fa4a74b5594223b48d579b2a33b`
    - `ordinal=31` / `2026-04-26T00:00:50.776235+08:00` 注入 `[定时任务触发] 任务名称：TEM 每日动态监控`
    - `ordinal=32` / `2026-04-26T00:01:30.411132+08:00` 新增 assistant final，正文明确写 `TEM 今日未出现新的公司级实质催化或风险证伪信号，按规则可跳过正式推送`
    - `ordinal=33` / `2026-04-26T00:01:30.415962+08:00` 紧接着又注入 `[定时任务触发] 任务名称：RKLB 每日动态监控`
    - `ordinal=34` / `2026-04-26T00:02:11.652155+08:00` 再次新增 assistant final，正文明确写 `RKLB 今日未出现新的公司级实质催化或风险证伪信号，按规则可跳过正式推送`
  - 最近一小时会话索引：`data/sessions.sqlite3` -> `sessions`
    - 同一 `session_id` 的 `updated_at=2026-04-26T00:02:11.652628+08:00`
    - `last_message_role=assistant`
    - `last_message_preview` 已被未送达的 `RKLB` 简报覆盖，开头即 `数据已经核验完。RKLB 今天没有新的公司级硬催化...按规则可跳过正式推送`
  - 最近一小时调度台账：`data/sessions.sqlite3` -> `cron_job_runs`
    - `run_id=6361` / `job_id=j_379acc40` / `job_name=TEM 每日动态监控` / `executed_at=2026-04-26T00:01:30.415080+08:00`
    - 本轮已记为 `execution_status=noop`、`message_send_status=skipped_noop`、`should_deliver=0`、`delivered=0`
    - `run_id=6362` / `job_id=j_5f0b686a` / `job_name=RKLB 每日动态监控` / `executed_at=2026-04-26T00:02:11.656192+08:00`
    - 本轮同样已记为 `noop + skipped_noop`
  - 最近一小时运行日志：`data/runtime/logs/web.log.2026-04-25`
    - `2026-04-26 00:01:30.414` 先记录 `step=session.persist_assistant ... detail=done`
    - 同秒随即记录 `SchedulerDiag skip_signal job_id=j_379acc40 job=TEM 每日动态监控`
    - 同秒再记录 `[Feishu] 心跳任务未命中，本轮不发送: job=TEM 每日动态监控`
    - `2026-04-26 00:02:11.655` 对 `RKLB 每日动态监控` 再次出现完全同样的顺序：`session.persist_assistant` -> `skip_signal` -> `本轮不发送`
    - 说明当前不是“消息真的发出去了”，而是“发送被抑制后，assistant final 仍照常写入 direct session”
  - 相关既有缺陷：
    - [`feishu_scheduler_daily_monitor_skip_rule_broken.md`](./feishu_scheduler_daily_monitor_skip_rule_broken.md)
    - [`session_persist_assistant_transcript_pollution.md`](./session_persist_assistant_transcript_pollution.md)

## 端到端链路

1. Feishu 每日动态监控到点后，把任务 prompt 作为 `[定时任务触发]` user turn 注入 actor 的 direct session。
2. 模型生成长文，并在正文里明确判断“按规则可跳过正式推送”。
3. 调度器随后识别到 `skip_signal`，把 `cron_job_runs` 正确收口成 `noop + skipped_noop`，且日志明确写“本轮不发送”。
4. 但 `session.persist_assistant` 发生在 `skip_signal` 之前，这条本应只用于内部判断的 assistant final 已经写入 direct session。
5. 结果是：用户虽然没有收到这条消息，session 历史与 `last_message_preview` 却已经被未发送长文污染。

## 期望效果

- 命中 `skip_signal` 或其它 `should_deliver=0` 的 scheduler 任务，不应把对应长文写入用户 direct session。
- 若调度链路需要保留内部分析文本供排障，应写入独立诊断字段，而不是复用用户会话历史。
- `sessions.last_message_preview` 应只反映真实对用户可见、或至少真实已送达的 assistant 内容。

## 当前实现效果

- `TEM` 与 `RKLB` 两个最新样本都已经正确“不发送”，说明旧的外发缺陷已止血。
- 但这两轮仍都在同一个 direct session 里新增了 assistant final，且 `sessions.last_message_preview` 直接指向最后一条未送达的 `RKLB` 简报。
- 从日志顺序看，问题出在 `session.persist_assistant` 先于 `skip_signal` 收口执行，导致“不该发”的内容虽然没出站，却已经进入会话状态。

## 用户影响

- 这不是单纯文案质量问题，而是会话状态污染问题，因此定级为 `P2`，不是 `P3`。
- 后续 direct 问答的 `restore_context`、compact、摘要索引和人工排障都可能把这些未送达的 scheduler 长文当成真实 assistant 历史继续消费。
- 用户当下未直接看到错误消息，但后续回答有被旧的未送达监控简报串线、挤占上下文预算或误导状态判断的风险。

## 根因判断

- scheduler 的“写库”和“是否发送”仍是分叉流程：assistant 最终文本先通过通用 `session.persist_assistant` 落库，之后才由 scheduler 判定 `skip_signal` 并中止发送。
- 旧修复只解决了“命中 skip 时不要投递”，没有同步收紧“命中 skip 时不要写入用户会话历史”的状态边界。
- 该问题与 `session_persist_assistant_transcript_pollution` 不同：这里落库的文本已是净化后的 final，但它本身不应进入 direct session，因为本轮实际没有出站。

## 修复情况（2026-04-26 18:06 CST）

- 已在 [`/Users/fengming2/Desktop/honeclaw/crates/hone-channels/src/scheduler.rs`](/Users/fengming2/Desktop/honeclaw/crates/hone-channels/src/scheduler.rs) 为 `skip_signal` 收口补回滚逻辑：
  - 非 heartbeat scheduler 命中 `skip_signal` 后，会立即撤回刚刚通过通用成功路径落进 session 的 assistant final；
  - 回滚只在“最后一条消息确实是本轮 skip 长文”时生效，避免误删用户后续真实对话。
- 已在 [`/Users/fengming2/Desktop/honeclaw/memory/src/session.rs`](/Users/fengming2/Desktop/honeclaw/memory/src/session.rs) 增加原子 `remove_last_message_if_matches(...)`，供 scheduler 安全回滚尾部污染消息。
- 这次修复只收紧“未送达长文不得进入 direct session”的状态边界，不改变 `cron_job_runs` 的 `noop + skipped_noop` 收口，也不影响真正需要投递的 scheduler 内容。

## 验证

- `cargo test -p hone-memory remove_last_message_if_matches_only_removes_matching_tail -- --nocapture`
- `cargo test -p hone-channels scheduler::tests -- --nocapture`
- `cargo check -p hone-channels -p hone-memory`

## 后续观察点

- 需要下一条真实 Feishu `skip_signal` 样本复核：确认 `session_messages` 与 `sessions.last_message_preview` 不再新增未送达 assistant final。
- 本次未改变 scheduler 注入的 `[定时任务触发]` user turn；若后续发现这些 internal trigger 也会稳定污染 direct 上下文，应另立缺陷单独收口。
