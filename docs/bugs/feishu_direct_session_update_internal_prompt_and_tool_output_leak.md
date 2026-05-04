# Bug: Feishu 直聊 `session/update` 会把系统提示、skill prompt 与工具原始输出直接外发

- **发现时间**: 2026-05-05 00:01 CST
- **Bug Type**: System Error
- **严重等级**: P1
- **状态**: Fixed
- **GitHub Issue**: [#31](https://github.com/B-M-Capital-Research/honeclaw/issues/31)

## 证据来源

- 最近一小时真实会话：
  - `data/sessions/Actor_feishu__direct__ou_5fa8018fa4a74b5594223b48d579b2a33b.json`
  - 同一 direct session 在 `2026-05-05 00:00-00:01 CST` 连续收到 `AAOI`、`TEM`、`RKLB` 三个每日动态监控触发 user turn；会话 JSON 末尾可见 `AAOI` 与 `TEM` 的 assistant final 已正常落库，而最新 `RKLB` user turn 也已进入会话
  - 说明这不是“最终 assistant 正文污染”或会话停摆；至少到落库层，主链路仍能写入正常 final
- 最近一小时运行日志：
  - `data/runtime/logs/acp-events.log`
  - `2026-05-04T16:00:00.701458+00:00` 同一 Feishu direct session 触发 `session/load`
  - `2026-05-04T16:00:00.701458+00:00` 之后，同一会话先后出现两类 `session/update` 泄漏：
    - `2026-05-04T16:00:00.701458+00:00` 后的首个 `agent_message_chunk` 直接以 `### System Instructions ###` 开头，展开完整系统提示、领域边界、技能索引与当前会话元信息
    - `2026-05-04T16:01:41.006377+00:00` 的 `tool_call_update.rawOutput` 直接外发 `【Invoked Skill Context】`、`Skill: Stock Research (stock_research)`、`Base directory for this skill: /Users/fengming2/Desktop/honeclaw/skills/stock_research` 及整段 skill prompt
    - `2026-05-04T16:01:41.006991+00:00` 的 `tool_call_update.rawOutput` 继续外发 `data_fetch(snapshot, AAOI)` 返回的大段结构化 JSON，包含 quote/profile/news
    - `2026-05-04T16:01:41.113041+00:00` 的 `tool_call_update.rawOutput` 直接外发 `apply_patch verification failed` 原始工具报错
    - `2026-05-04T16:01:41.146593+00:00` 与 `2026-05-04T16:01:41.147039+00:00` 又继续外发 `web_search` 的原始返回，包含长篇抓取正文与 URL 列表
- 与已有缺陷的去重结论：
  - [`web_direct_session_update_prompt_echo_leak.md`](./web_direct_session_update_prompt_echo_leak.md) 只覆盖 Web `agent_message_chunk` prompt echo
  - [`web_direct_tool_call_raw_output_leak.md`](./web_direct_tool_call_raw_output_leak.md) 只覆盖 Web `tool_call_update.rawOutput`
  - [`feishu_attachment_internal_transcript_leak.md`](./feishu_attachment_internal_transcript_leak.md) 已修的是附件场景下“最终 assistant transcript 落库污染”
  - 本单是普通 Feishu direct/scheduler 会话在 live `session/update` 通道泄漏内部 prompt、绝对路径、工具回显与原始报错，影响渠道、事件边界与复现条件都不同

## 端到端链路

1. Feishu direct session 到点执行每日动态监控任务，runner 正常执行 `session/load` 与后续研究工具调用。
2. 最终会话 JSON 仍按既有收口逻辑只写入正常 assistant final，因此静态落库表面看起来正常。
3. 但 live `session/update` 事件在同一轮先后把 `agent_message_chunk`、`tool_call_update.rawOutput`、原始工具报错直接外发。
4. 外发内容包含系统提示、skill prompt、绝对路径、结构化工具数据和原始错误，不是用户应见的进度摘要。
5. 结果是：即使最终落库答案表面正常，实时 Feishu 回复链路仍可能把内部实现细节暴露给用户。

## 期望效果

- Feishu `session/update` 只能外发用户可见正文或简短进度，不应发送系统提示、技能索引、skill prompt、工具原始回显或本机绝对路径。
- `tool_call_update.rawOutput`、内部 JSON payload、原始工具报错应保留在诊断层，不应进入用户侧渠道事件。
- 即使最终 final 会被后置净化，live 更新链路也必须在 chunk / tool-update 层提前拦截内部内容。

## 当前实现效果

- 最近一小时真实 Feishu direct session 已证明，当前线上并非只有 Web 会发生 `session/update` 泄漏；Feishu 也会在 live 事件流中重放系统提示与工具原始输出。
- 泄漏内容同时覆盖：
  - `### System Instructions ###` 与 `turn-0 可用技能索引`
  - `【Invoked Skill Context】` 与完整 `stock_research` skill prompt
  - `Base directory for this skill: /Users/.../skills/stock_research`
  - `data_fetch` / `web_search` 原始结构化结果
  - `apply_patch verification failed` 这类原始工具错误
- 同一会话 JSON 最终 assistant final 仍能保持正常，说明问题集中在“实时外发边界”而非最终持久化层。

## 用户影响

- 这是功能性缺陷，不是单纯格式不佳。
- 用户侧一旦消费或渲染这些 Feishu `session/update` 事件，就会直接看到内部 prompt、工具协议、本机路径与原始错误，产品边界已经失守。
- 该缺陷可能暴露内部运行策略、技能实现和本机目录结构，同时也会把大段原始工具数据误当作回复发送，干扰用户完成任务。
- 之所以定级为 `P1`，是因为问题同时涉及内部信息泄漏与用户可见实时链路失控，而不是单纯“回答质量偏弱”的 `P3`。

## 根因判断

- 线上 Feishu 渠道的 `session/update` 外发路径与最终 session JSON 收口路径明显分离；后者已能只保留 final，前者仍在透传内部 event payload。
- 当前用户态净化很可能只覆盖了最终回复或部分 Web emitter，没有覆盖 Feishu `agent_message_chunk` 与 `tool_call_update.rawOutput` 的 live 出站。
- 同窗内同时出现 prompt echo、skill rawOutput、结构化工具结果和原始报错，说明缺口不是某个单一工具，而是 `session/update` 事件总体缺少字段级用户态裁剪。

## 修复记录（2026-05-05 03:04 CST）

- 在 `crates/hone-channels/src/agent_session/emitter.rs` 将用户态事件净化扩展到 `RunEvent::StreamDelta`：
  - 命中内部 prompt / skill context marker 的 chunk 会被抑制；
  - 若内部 marker 前存在正常用户可见前缀，仅保留前缀；
  - 结构化 JSON / array payload 与典型 ACP/provider 内部错误细节不会转发给监听者；
  - 路径相对化与 sandbox 外绝对路径遮蔽继续沿用共享规则。
- 在 `crates/hone-channels/src/runners/acp_common/ingest.rs` 扩展 `agent_message_chunk` marker 集合，直接阻断 `【Invoked Skill Context】` 与 `Base directory for this skill:` 污染 `full_reply` / `pending_assistant_content`。
- 新增回归测试覆盖：
  - Feishu channel 的 `StreamDelta` 内部 skill context 整段抑制；
  - 可见 `OK` 前缀保留、内部 suffix 截断；
  - ACP ingest 层不再把 invoked skill context chunk 写入回复状态。

## 当前验证（2026-05-05 03:04 CST）

- 已通过：
  - `rustfmt --edition 2024 --check crates/hone-channels/src/agent_session/emitter.rs crates/hone-channels/src/agent_session/tests.rs crates/hone-channels/src/runners/acp_common/ingest.rs crates/hone-channels/src/runners/acp_common/tests.rs`
  - `cargo test -p hone-channels session_event_emitter_ -- --nocapture`
  - `cargo test -p hone-channels acp_common --lib -- --nocapture`
  - `cargo check -p hone-channels --tests`
  - `cargo test -p hone-channels --lib -- --nocapture`

## 后续建议

- 该修复是本地可验证的共享边界加固，不依赖生产日志或线上 Feishu 运行态。
- GitHub Issue [#31](https://github.com/B-M-Capital-Research/honeclaw/issues/31) 建议在本轮提交推送后复测并关闭。
