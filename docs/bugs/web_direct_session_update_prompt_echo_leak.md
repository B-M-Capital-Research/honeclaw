# Bug: Web 直聊流式 `session/update` 会把完整系统提示与技能索引当成正文 chunk 外发

- **发现时间**: 2026-05-02 02:20 CST
- **Bug Type**: System Error
- **严重等级**: P1
- **状态**: Fixed
- **GitHub Issue**: [#28](https://github.com/B-M-Capital-Research/honeclaw/issues/28)

## 证据来源

- 最近一小时真实复发：
  - `data/sessions/Actor_web__direct__web-user-e05f5e5f74a3.json`
  - `2026-05-04T22:04:48.104718+08:00` 用户输入：`心跳检测，请简短回复 OK`
  - `2026-05-04T22:05:03.185109+08:00` 最终 assistant 落库仍只有 `OK`
  - 但 `data/runtime/logs/acp-events.log` 显示同一轮 `2026-05-04T14:04:49.150625+00:00` 再次收到 `session/update -> agent_message_chunk`，正文直接以 `### System Instructions ###` 开头，并再次展开完整系统提示、领域边界、skill 索引与 `【Session 上下文】`
  - 说明 2026-05-02 标记为 `Fixed` 后，Web `session/update` prompt echo 仍可在真实 live session 中复发；只是最终会话文件继续被后置收口为 `OK`
- 最近一小时真实会话：
  - `data/sessions/Actor_web__direct__web-user-e05f5e5f74a3.json`
  - `2026-05-02T02:03:12.015076+08:00` 用户输入：`心跳检测，请简短回复 OK`
  - `2026-05-02T02:03:25.140545+08:00` 最终 assistant 落库仅为 `OK`
- 最近一小时运行日志：
  - `data/runtime/logs/acp-events.log`
  - `2026-05-01T18:03:16.054696+00:00` 同一 Web session 收到 `session/update -> agent_message_chunk`
  - 该 chunk 正文直接以 `### System Instructions ###` 开头，并展开整段系统提示、领域边界、skill context、turn-0 技能索引、当前会话时间与 `会话 ID`
  - 同轮后续仍在 `2026-05-01T18:03:16.570403+00:00` 继续记录携带相同 prompt 包的 `content.text`，最终到 `2026-05-01T18:03:33.765689+00:00` 才以 `stopReason=end_turn` 收口
- 对照说明：
  - 会话 JSON 尾部最终只有 `OK`，说明这次不是“最终持久化正文污染”
  - 问题集中在 Web 实时 `session/update` 流：中途向客户端外发了不应暴露的 prompt/skill 内部文本

## 端到端链路

1. Web 用户发送简单健康检查：`心跳检测，请简短回复 OK`。
2. 运行链路本应只回一个极短的 `OK`。
3. 但同一轮 `acp-events.log` 先向 Web 侧写出 `agent_message_chunk`，正文不是答案，而是整段 `### System Instructions ###` prompt 包。
4. 该 chunk 内含系统规则、技能说明、turn-0 技能索引和会话元信息。
5. 最终落库仍只有 `OK`，说明后置持久化净化或收口逻辑工作了，但实时流式外发边界已经失守。

## 期望效果

- Web `session/update` 流只能向客户端外发用户可见正文，不应发送 `### System Instructions ###`、skill prompt、工具契约、技能索引或内部会话元信息。
- 即使 runner 中途产生 prompt echo 或 context 包回流，也必须在 chunk 级别被拦截或清空，而不是等最终 `final` 才净化。
- 简单 `OK` 健康检查不应在中间阶段产生任何长篇内部文本。

## 当前实现效果

- `2026-05-04 22:04` 的最新真实 Web 会话说明，这条缺陷并没有被稳定修掉：最终落库虽然仍被收口成 `OK`，但实时 `session/update` 事件再次整段外发系统提示。
- 最近一小时这条 Web 会话最终落库仍是正常的 `OK`，但流式更新里先后出现了超长内部 prompt 文本。
- 泄露内容不是单一一句状态标记，而是完整的系统约束与技能索引，敏感度明显高于普通格式噪音。
- 这说明当前链路对最终 answer 有一定净化，但对 `session/update -> agent_message_chunk` 没有同等级防护。

## 用户影响

- 这是功能性安全边界缺陷，不是单纯文案质量问题。
- 如果前端实时渲染这些 chunk，用户会直接看到内部系统提示、技能清单和运行约束，属于明确的内部 prompt 泄露。
- 即使前端未完整渲染，服务端事件流与排障视图也已被污染，说明产品边界不可靠。
- 因为问题涉及内部提示外泄且在当前活跃窗口真实复现，所以定级为 `P1`。

## 根因判断

- 最新复发形态说明，当前过滤并没有覆盖 Web `session/load` 后重放到前端的 `session/update` 事件，或相关防护没有真正进入当前 live runtime。
- 现象表明“最终回复净化”和“流式 chunk 外发”走了不同边界：
  - 最终持久化/收口只保留了 `OK`
  - 中途 `session/update` 却把 prompt 包当作普通 `agent_message_chunk` 发出
- 更像是 runner/桥接层把内部 `content.text` 或 prompt 包错误映射成了用户可见消息更新，而不是单纯的历史落库污染。
- 旧的 prompt echo 止血主要覆盖“最终回复前缀是 `### System Instructions ###`”的场景；这次复现说明 chunk 级外发链路仍有独立缺口。

## 修复记录

- 2026-05-02 03:13: 在 `crates/hone-channels/src/runners/acp_common/ingest.rs` 的 ACP `agent_message_chunk` ingest 层增加 chunk 级 prompt echo 过滤。命中 `### System Instructions ###`、`### Skill Context ###`、`【Session 上下文】`、`turn-0 可用技能索引` 等内部 prompt 标记时，不再写入 `full_reply` / `pending_assistant_content`，也不再 emit 用户可见 `StreamDelta`；若同一 chunk 中内部标记前已有真实可见前缀，则只保留前缀并截断后续内部内容。
- 回归测试覆盖完整内部 prompt chunk 丢弃、`OK` 前缀保留、既有 compact/boundary chunk 原样透传。
- 验证：`cargo test -p hone-channels acp_common --lib -- --nocapture` 通过。
- 关联 GitHub Issue：[#28](https://github.com/B-M-Capital-Research/honeclaw/issues/28)。

## 后续建议

- 先把本单从 `Fixed` 改回 `New`，修复时重点核对 `session/load` / 历史 event replay 是否绕过了 2026-05-02 新增的 chunk 级过滤。
- 若后续出现新的 prompt echo 变体，优先扩展 ACP ingest 层的内部标记集合，保持“用户可见正文”和“调试/内部 prompt”通道分离。
- 用同一条 `心跳检测，请简短回复 OK` 在 Web 前端做一次实时复现，确认用户侧是否能肉眼看到泄露 chunk。

## 修复记录（2026-05-04 23:06 CST）

- 根因复核确认最新复发集中在 Codex ACP 的旧远端 session 复用：`session/load` 会把历史 `session/update` 回放重新写进本轮 ACP 流，其中包含旧的 `agent_message_chunk` prompt 包。
- 已将 `crates/hone-channels/src/runners/codex_acp.rs` 与 `opencode_acp` 对齐：即使 session metadata 中存在旧 `codex_acp_session_id`，也不再执行 `session/load`，每轮都新建 ACP session，并把 Hone 本地恢复的 transcript/context 注入 prompt。
- 既有 ACP ingest 层 prompt echo 过滤继续保留，负责拦截当前轮 runner 自身产生的内部 prompt chunk；本次修复则切断旧 session replay 这一入口。
- 状态更新为 `Fixed`；关联 GitHub Issue [#28](https://github.com/B-M-Capital-Research/honeclaw/issues/28) 建议复测后关闭。

## 当前验证（2026-05-04 23:06 CST）

- 已通过：
  - `cargo test -p hone-channels codex_acp_does_not_reuse_remote_session_metadata -- --nocapture`
  - `cargo test -p hone-channels acp_common --lib -- --nocapture`
  - `cargo test -p hone-channels session_event_emitter_ -- --nocapture`
  - `cargo check -p hone-channels --tests`
  - `rustfmt --edition 2024 --check crates/hone-channels/src/runners/codex_acp.rs crates/hone-channels/src/runners/tests.rs`
