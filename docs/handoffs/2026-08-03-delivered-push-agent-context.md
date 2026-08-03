# Confirmed Proactive Delivery Context Handoff

- title: Confirmed Proactive Delivery Context Handoff
- status: done
- created_at: 2026-08-03
- updated_at: 2026-08-03
- owner: shared
- related_files:
  - `crates/hone-event-engine/src/store.rs`
  - `crates/hone-event-engine/src/router/dispatch.rs`
  - `crates/hone-event-engine/src/unified_digest/scheduler.rs`
  - `crates/hone-channels/src/agent_session/core.rs`
  - `crates/hone-channels/src/runners/types.rs`
  - `crates/hone-channels/src/scheduler.rs`
  - `bins/hone-{discord,telegram,feishu}/src/scheduler.rs`
  - `crates/hone-web-api/src/routes/events.rs`
- related_docs:
  - `docs/archive/plans/delivered-push-agent-context.md`
  - `docs/decisions.md#d-2026-08-03-01-project-confirmed-proactive-deliveries-into-the-next-interactive-turn`
  - `docs/invariants.md`
  - `docs/repo-map.md`
- related_prs: pushed directly to `main`; no PR, deployment, release, or tag

## Summary

真实送达的事件推送与 scheduled/heartbeat 渠道消息现在会作为一次性事实进入同一 actor 的下一次交互 Agent 上下文。用户消息仍按原文字节持久化；系统提示词、历史对话、工具调用/结果和 compact summary 不会因此重新包装成用户输入。

## What Changed

- `EventStore::log_confirmed_delivery` 是唯一纳入边界，直接接收 typed `ActorIdentity`。普通审计即使字符串状态为 `sent` 也不能创建上下文；确认接口在一个 SQLite 事务内追加审计和 `delivered_push_context` journal。
- Journal 按 actor/source 去重，按毫秒送达顺序领取，带 turn lease、成功 complete、失败 release、过期恢复、原生 session observation、20 条与 12,000 字符预算；升级前审计不回填。多连接有五秒 busy timeout，claim 使用 immediate transaction。
- Interactive ingress 在等待 session lock 前固定 cutoff。`/compact`、quota rejection、scheduled 和 heartbeat 不领取；内部 retry/overflow recovery 复用同一批。
- NativePersistent 在当前时间与 `【本轮用户输入】` 之间投影明确的“此前已送达事实”块；StructuredReplay/EphemeralCompiledPrompt 添加带 `subtype=delivered_push_context` 的 assistant/context 消息。每条正文渲染上限 4,000 字符。
- Event-engine immediate/digest、Discord/Telegram 分段 ACK、Feishu send success、Web durable push/history 与 iMessage HTTP success 都接到统一确认接口。分段渠道只记录已发送成功的前缀。
- 同一原生 session 已经生成并保留 scheduled assistant 输出时只推进消费位点；OpenCode/fresh replay 或其它 session 仍获得显式上下文。

## Verification

- 定向回归覆盖：P1/P2 → U1 → U2、用户持久化字节不变、actor/scope 隔离、cutoff race、retry dedupe、失败释放、compact/配额不消费、历史不回填、两 SQLite 连接与正文预算、直接 Router ACK。
- Codex ACP `1.1.7` executable JSON-RPC 回归覆盖 compact signal 后的 current-turn-only prompt；OpenCode ACP `1.18.11` 回归覆盖 assistant/context 与未改写 user section，版本和 2026-08-01 真实 capture 边界写在测试中。
- `cargo check --workspace --all-targets --exclude hone-desktop --exclude hone-user-app` 通过。
- `cargo test --workspace --all-targets --exclude hone-desktop --exclude hone-user-app` 通过；其中 `hone-event-engine` 556 项、`hone-channels` 721 项，凭据/主机依赖项按既有规则 ignored。
- `bun run test:web`：344 项通过；Public Community Edge typecheck 与 45 项测试通过；`bash tests/regression/run_ci.sh` 通过。
- 真实 OpenCode `1.18.11` 使用隔离免费模型完成 `initialize → session/new → session/set_model → session/prompt → Hone MCP → end_turn`。默认 OpenAI OAuth 另行返回 `401 Token refresh failed`，确认是外部凭据状态；未改动全局 OpenCode 配置。

## Risks / Follow-ups

- 本地/当前单机 runtime 依赖各进程共享 canonical `events.sqlite3`。若云端拆成不共享文件系统的多节点，必须在同一 store 接口下增加权威 Cloud PG journal；不能退回内存队列。
- 外部 runner 没有可与 SQLite 合并的 prompt ACK，因此语义是“Agent 成功后消费、失败可恢复”的 at-least-once，不宣称跨进程崩溃下端到端 exactly-once。
- 富文本卡片与 fallback 正文可能保留不同细节；当前尽量保存实际发送正文/成功分段，不保存渠道协议 JSON。
- 本次按用户要求只推送主干，没有部署本地或 GCE。默认 OpenCode OAuth 若要作为生产模型使用，需要单独重新登录后再跑同一手工探针。

## Next Entry Point

先从 `EventStore::log_confirmed_delivery` 与 `claim_delivered_push_context_with_native_observation` 检查送达/领取真相，再看 `AgentSession::claim_delivered_push_context` 和 `RunnerConversationInput::prepare` 的投影；渠道漏记从各 scheduler 的 `record_confirmed_scheduled_delivery` 调用位置排查。
