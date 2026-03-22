# ACP Runtime Refactor - Handoff

日期：2026-03-17

## 本轮结果

- `AgentSession` 已新增统一入口 `run()`，渠道与 Web 主链路已切换到该入口。
- session 存储已升级为 versioned JSON v2：
  - 显式 `summary`
  - 显式 `runtime.prompt.frozen_time_beijing`
  - 旧 session 需通过迁移脚本批量升级后再交由运行时读取
- prompt 构建已拆为 bundle：
  - static system
  - session 固定上下文
  - 动态会话总结
- Web chat SSE 已升级为：
  - `run_started`
  - `assistant_delta`
  - `tool_call`
  - `run_error`
  - `run_finished`
- 前端会话流已切到新 SSE 协议，并改为增量拼接 assistant 消息。
- breaking 配置项已从 `agent.provider` 切换到 `agent.runner`，并新增 `agent.opencode` 配置块。

## 验证

- `cargo check -p hone-channels -p hone-web-api -p hone-memory -p hone-cli -p hone-telegram -p hone-discord -p hone-feishu -p hone-imessage`
- `cargo test -p hone-memory -p hone-channels`
- `bun run typecheck:web`
- `bun --cwd packages/app test src/lib/stream.test.ts`
- `cargo run --manifest-path Cargo.toml -p hone-cli`（临时 config 切到 `agent.runner=opencode_acp`，实测返回 `OK`）

## 未完成

- `opencode_acp` 已通过 `opencode acp` 的 stdio/JSON-RPC 接入；Hone 会把 ACP session id 回写到 session metadata，以支持后续轮次 `session/load` 复用。
- 当前 `opencode_acp` 仍是最小接入：权限请求默认拒绝一次，尚未接入 Hone 自身的文件系统/终端权限协商层。
- 现已补充独立迁移脚本链：`scripts/migrate_sessions.py`、`scripts/migrate_cron_jobs.py`、`scripts/migrate_skills.py` 与统一入口 `scripts/migrate_legacy_data.py`。

## 风险

- 这是 breaking 变更，所有外部依赖旧 `agent.provider`、旧 SSE 事件名或伪 `system` summary 的脚本/工具都需要同步调整。
- 目前各渠道 listener 仍是旧 `AgentSessionEvent` 适配层；如果后续继续做 ACP-native runner，应优先收敛到新的 canonical event 模型，而不是继续叠加旧事件变体。
