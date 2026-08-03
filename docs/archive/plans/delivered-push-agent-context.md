# 已送达主动推送进入下一轮 Agent 上下文

- title: 已送达主动推送进入下一轮 Agent 上下文
- status: done
- created_at: 2026-08-03
- updated_at: 2026-08-03
- owner: shared
- related_files:
  - `crates/hone-event-engine/src/store.rs`
  - `crates/hone-channels/src/core/bot_core.rs`
  - `crates/hone-channels/src/agent_session/core.rs`
  - `crates/hone-channels/src/runners/types.rs`
  - `crates/hone-channels/src/runners/tests.rs`
  - `tests/fixtures/acp/`
  - `docs/repo-map.md`
  - `docs/invariants.md`
  - `docs/decisions.md`
- related_docs:
  - `docs/current-plans/acp-runtime-refactor.md`
  - `docs/adr/0002-agent-runtime-acp-refactor.md`

## Goal

把事件引擎已经真实送达、但尚未被交互 Agent 观察到的主动推送，作为有类型、可去重的一次性上下文事件，在同一 actor 的下一轮交互用户消息之前交给 Agent；保持用户原始消息、系统提示词、历史对话和工具协议边界不变。

## Scope

- 以渠道成功 ACK 后显式调用 `EventStore::log_confirmed_delivery` 和实际正文为唯一纳入边界；普通 `delivery_log` 审计即使写成 `status=sent` 也不能隐式改变会话语义，`queued`、`failed`、`dryrun`、空正文不进入 Agent 上下文。
- 为已送达推送建立独立的待消费状态，按 `channel + scope + user` 隔离，并按真实送达顺序领取。
- 仅交互轮领取；定时任务、heartbeat、`/compact` 和配额拒绝不消费待处理推送。
- 用户消息仍按原文持久化；推送上下文只进入执行层。原生持久会话在当前 `session/prompt` 中投影为明确的“此前已送达事实”段，Replay Runner 投影为带元数据的 assistant/context 消息。
- 同一交互轮的内部重试复用同一批推送；成功后消费，失败后释放给下一轮。compact 不重播已消费推送，也不会删除尚未消费的推送。
- 不回填功能上线前的历史 `delivery_log`，避免首次交互突然注入旧通知。

## Validation

- [x] P1、P2 真实送达后，下一次交互 U1 按 `P1 → P2 → U1` 顺序进入 Agent 执行上下文，持久化的 U1 字节不变。
- [x] U1 成功后 U2 不再获得 P1/P2；同一 delivery/source 的重复写入不重复进入上下文。
- [x] `queued`、`failed`、`dryrun`、普通审计 `sent`、空正文和功能上线前历史记录均不进入上下文。
- [x] U1 到达后才送达的 P3 不进入 U1，只进入下一轮；不同 user/channel/scope 之间没有泄漏。
- [x] Runner 失败时领取被释放；同一轮内部 retry/overflow recovery 不重复领取或改变顺序。
- [x] compact 后已消费推送不补发，compact 前已送达但未消费的推送仍保留给下一轮；配额拒绝也不消费。
- [x] Codex ACP 1.1.7 外部 JSON-RPC 边界证明 `session/prompt` 只包含当前时间、待消费推送事实和当前用户输入，不包含 system/history/tool replay。
- [x] OpenCode ACP 1.18.11 基线证明推送以 assistant/context 角色进入 fresh-session replay，当前 user turn 不被改写；测试注明版本和采集边界。
- [x] 运行定向 Rust 测试、改动文件格式检查、workspace `cargo check/test`、Web 测试、Edge 测试和 CI-safe regression。

## Verification Result

- `hone-event-engine` 556 项通过；`hone-channels` 720 项通过后新增 compact/quota 边界，workspace 最终 721 项通过。
- Workspace all-target `cargo check/test`（排除 Apple release-only crates）、Web 344 项、Edge 45 项、CI-safe regression 全部通过。
- 本机 OpenCode `1.18.11` 的真实 `initialize → session/new → session/set_model → session/prompt → Hone MCP → end_turn` 使用隔离免费探针模型通过；本机默认 OpenAI OAuth 同时被真实边界确认返回 `401 Token refresh failed`，属于外部凭据状态，不影响版本/协议验收，也未修改全局配置。

## Documentation Sync

- 更新 `docs/repo-map.md`，记录 `delivery_log → delivery context claim → Canonical Turn → Runner projection` 数据流。
- 更新 `docs/invariants.md`，明确真实送达、actor 隔离、一次消费、用户原文与 ACP current-turn-only 边界。
- 更新 `docs/decisions.md`，记录为什么复用 append-only delivery audit 并单独维护上下文消费状态，而不修改用户消息或回放全历史。
- 完成后新增 `docs/handoffs/2026-08-03-delivered-push-agent-context.md`，把本计划移到 `docs/archive/plans/`，更新 `docs/archive/index.md` 并从 `docs/current-plan.md` 移除。

## Risks / Open Questions

- 外部 Runner 不提供与本地事务统一的 prompt ACK；本任务采用“成功轮消费、失败轮释放”，保证用户可恢复和同轮重试稳定，但不宣称跨进程崩溃下不可实现的端到端 exactly-once。
- 富文本渠道实际卡片与 fallback 正文细节不同；Agent 上下文尽量保留真实已发送正文，分段渠道只登记获得 ACK 的前缀，并保留 delivery/source ID，不注入渠道协议 JSON。
- 事件引擎 SQLite 是当前 delivery audit 真相源；若未来把事件引擎拆到不共享存储的独立节点，需要为同一存储接口增加 Cloud PG 实现，不能退回进程内队列。
