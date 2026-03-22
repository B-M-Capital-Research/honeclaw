# 2026-03-17 架构收敛与稳定性审计

## 结果

- 审计了 `AgentSession`、`HoneBotCore`、KB 分析链路，以及 Feishu / Telegram / Discord 三个主要渠道入口
- 发现 1 个明确的会话持久化缺陷：Telegram 成功回复后手动再次写入 assistant 消息，导致 session 历史重复
- 发现 2 个较明显的架构收敛缺口：
  - KB 分析仍走 legacy `create_agent()`，没有复用统一 runner contract
  - Discord / Telegram / Feishu 的 placeholder reasoning 监听仍按 `gemini_cli` 特判挂载，没有真正消费统一 `ToolStatus`
- 发现 1 个稳定性治理缺口：只有 Feishu 对消息去重和同 session 串行处理做了明确保护，Telegram 与 Discord 直连问答路径仍可能并发踩同一 session
- 发现 1 个配置接线不一致点：Discord slash skill 搜索目录没有跟随 `HONE_DATA_DIR`，与实际工具注册目录可能漂移

## 主要问题

- `bins/hone-telegram/src/main.rs` 在 `AgentSession::run()` 成功后再次调用 `session_storage.add_message(..., "assistant", ...)`，而 `crates/hone-channels/src/agent_session.rs` 已统一持久化 assistant 消息
- `crates/hone-channels/src/kb_analysis.rs` 仍通过 `HoneBotCore::create_agent()` 运行分析；`crates/hone-channels/src/core.rs` 中该 legacy factory 会把 `gemini_acp` 降级到 `GeminiCliAgent`、把 `codex_acp` 降级到 `CodexCliAgent`、把 `opencode_acp` 落回 `FunctionCallingAgent`
- `bins/hone-feishu/src/main.rs` 已实现 `processed_msg_ids` 和 `session_locks`，但 `bins/hone-telegram/src/main.rs` 与 `bins/hone-discord/src/handlers.rs` 的即时问答路径未见等价保护；结合 `AgentSession::run()` 先 `restore_context` 再持久化 user/assistant 的顺序，并发消息会共享旧上下文快照
- `bins/hone-telegram/src/main.rs`、`bins/hone-discord/src/handlers.rs`、`bins/hone-discord/src/group_reply.rs` 仍只在 `agent.runner == "gemini_cli"` 时挂 reasoning listener，未把统一 runner 的 `ToolStatus` 能力真正下沉为渠道通用行为
- Discord 的 skill 自动补全目录来自 `bins/hone-discord/src/utils.rs` 的硬编码 `./data/custom_skills`，而真实运行时工具注册使用 `HONE_DATA_DIR/custom_skills`

## 建议的后续动作

- 优先修复 Telegram assistant 重复持久化，避免继续污染既有 session 历史
- 给 `AgentSession` 增加可选的 session 级运行锁，或在共享层抽出统一的 channel ingress guard，收口 Feishu/Telegram/Discord 的并发治理
- 把 KB 分析改造为基于统一 runner contract 的轻量执行入口，至少不要再通过 legacy factory 重新映射 runner
- 抽出共享的 attachment-to-KB ingest helper，以及共享的 reasoning placeholder listener 接线，减少渠道入口重复逻辑
- 统一 skill 目录解析函数，避免入口层再拷贝一份目录推导逻辑

## 验证

- 未运行测试；本次只做代码阅读和架构审计

## 风险与未覆盖项

- 并发问题目前基于代码路径推断，没有通过压力脚本复现
- Discord 群聊链路已有 channel worker，不在本次“未串行化”结论范围内；风险主要在 Telegram 与 Discord 直连问答路径
- 未检查前端页面与桌面宿主的全部能力一致性，本次重心在 Rust runtime 和渠道入口
