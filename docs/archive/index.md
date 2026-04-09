# Archive Index

Last updated: 2026-04-09

Use this file as the historical entry point for completed or paused work that should remain discoverable.

## 2026-03-31

### macOS DMG Release 打包收口

- Status: done
- Date: 2026-03-31
- Plan: N/A
- Handoff: `docs/handoffs/2026-03-31-macos-dmg-release-packaging.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `make_dmg_release.sh`
- Current conclusion: 新增 `make_dmg_release.sh` 并真实产出 Apple Silicon / Intel 两套 DMG；release 包内置 `hone-mcp` 与 macOS `opencode`，并补齐 packaged/runtime 启动环境与启动锁重试路径
- Next entry point: `docs/handoffs/2026-03-31-macos-dmg-release-packaging.md`

### 定时任务输出净化与 Tavily 失败隔离

- Status: done
- Date: 2026-03-31
- Plan: N/A
- Handoff: `docs/handoffs/2026-03-31-scheduler-output-and-search-failure-hygiene.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-tools`, `cargo test -p hone-channels`
- Current conclusion: heartbeat / 定时任务会抽出真正 JSON 结果；Tavily 临时失败会返回脱敏 unavailable 结构，且不再持久化进会话工具上下文
- Next entry point: `docs/handoffs/2026-03-31-scheduler-output-and-search-failure-hygiene.md`

## 2026-03-29

### 额度与定时任务可靠性修复

- Status: done
- Date: 2026-03-29
- Plan: N/A
- Handoff: `docs/handoffs/2026-03-29-quota-scheduler-reliability.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-memory`, `cargo test -p hone-channels`
- Current conclusion: 普通用户每日额度调整为 12；非 heartbeat 定时任务补上“同日单次补触发”；heartbeat JSON 解析失败会安全抑制
- Next entry point: `docs/handoffs/2026-03-29-quota-scheduler-reliability.md`

## 2026-03-27

### 单一聊天范围配置与群聊忙碌态控制

- Status: done
- Date: 2026-03-27
- Plan: N/A
- Handoff: `docs/handoffs/2026-03-27-chat-scope-busy-guard.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo check -p hone-core -p hone-channels -p hone-discord -p hone-feishu -p hone-telegram`, `cargo test -p hone-core -p hone-channels`
- Current conclusion: `dm_only` 收敛为 `chat_scope`；群聊忙碌态在显式触发场景具备统一控制
- Next entry point: `docs/handoffs/2026-03-27-chat-scope-busy-guard.md`

## 2026-03-26

### 子模型配置与心跳任务调度

- Status: done
- Date: 2026-03-26
- Plan: N/A
- Handoff: `docs/handoffs/2026-03-26-heartbeat-submodel-scheduler.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-memory -p hone-scheduler -p hone-tools -p hone-core -p hone-web-api -p hone-channels`, `cargo check -p hone-desktop`
- Current conclusion: Desktop 支持 OpenRouter 子模型配置，会话压缩切到子模型，cron 新增 heartbeat 任务类型
- Next entry point: `docs/handoffs/2026-03-26-heartbeat-submodel-scheduler.md`

### Session SQLite 影子写入与运行时切换

- Status: done
- Date: 2026-03-26
- Plan: N/A
- Handoff: `docs/handoffs/2026-03-26-session-sqlite-cutover.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `bash tests/regression/ci/test_session_sqlite_migration.sh`
- Current conclusion: SessionStorage 已支持 `json | sqlite` 切换；SQLite shadow write 与 runtime 主读都已接入
- Next entry point: `docs/handoffs/2026-03-26-session-sqlite-cutover.md`

## 2026-03-24

### 群聊预触发窗口统一改造

- Status: done
- Date: 2026-03-24
- Plan: N/A
- Handoff: `docs/handoffs/2026-03-24-group-pretrigger-window-unify.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo check -p hone-channels -p hone-discord -p hone-feishu -p hone-telegram`, `cargo test -p hone-channels -p hone-core`
- Current conclusion: Telegram / Discord / 飞书群聊统一为“未触发先静默缓存、显式触发再执行”的预触发窗口模型
- Next entry point: `docs/handoffs/2026-03-24-group-pretrigger-window-unify.md`

## 2026-03-22

### 多渠道附件工程化卡点

- Status: archived
- Date: 2026-03-22
- Plan: `docs/archive/plans/channel-attachment-gate.md`
- Handoff: `docs/handoffs/2026-03-22-channel-attachment-gate.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-channels`, `cargo check -p hone-channels -p hone-discord -p hone-feishu -p hone-telegram`
- Current conclusion: 共享附件 ingest 已统一拦截超限附件与异常图片，并把拦截原因透出到渠道 ack
- Next entry point: `docs/handoffs/2026-03-22-channel-attachment-gate.md`

## 2026-03-19

### 真群聊共享 Session 落地

- Status: archived
- Date: 2026-03-19
- Plan: `docs/archive/plans/group-shared-session.md`
- Handoff: `docs/handoffs/2026-03-19-group-shared-session.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo check -p hone-discord -p hone-feishu -p hone-telegram -p hone-imessage -p hone-web-api`, `cargo test -p hone-memory -p hone-channels`
- Current conclusion: 群聊会话归属改为显式 `SessionIdentity`；三渠道群消息共享上下文，Web 控制台按真实 `session_id` 浏览
- Next entry point: `docs/handoffs/2026-03-19-group-shared-session.md`

### 群聊回复追加链路统一

- Status: archived
- Date: 2026-03-19
- Plan: `docs/archive/plans/group-reply-append-chain.md`
- Handoff: `docs/handoffs/2026-03-19-group-reply-append-chain.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo check -p hone-discord -p hone-feishu -p hone-telegram`, `cargo test -p hone-discord -p hone-telegram`
- Current conclusion: 群聊占位符、首条 `@用户` 与多段 reply 链已在 Discord / Telegram / Feishu 统一
- Next entry point: `docs/handoffs/2026-03-19-group-reply-append-chain.md`

## 2026-03-18

### 渠道运行态心跳替代 pid 判活

- Status: done
- Date: 2026-03-18
- Plan: N/A
- Handoff: `docs/handoffs/2026-03-18-channel-heartbeat-status.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo check -p hone-core -p hone-web-api -p hone-desktop -p hone-discord -p hone-feishu -p hone-telegram -p hone-imessage`, `cargo test -p hone-core -p hone-web-api`
- Current conclusion: `/api/channels` 已改为基于 `runtime/*.heartbeat.json` 的心跳新鲜度呈现状态
- Next entry point: `docs/handoffs/2026-03-18-channel-heartbeat-status.md`

### launch.sh 真实进程清理修复

- Status: done
- Date: 2026-03-18
- Plan: N/A
- Handoff: `docs/handoffs/2026-03-18-launch-process-cleanup-fix.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `bash -n launch.sh`, `cargo build -p hone-console-page -p hone-imessage -p hone-discord -p hone-feishu -p hone-telegram`
- Current conclusion: `launch.sh` 已直接启动真实 debug 二进制，pid 文件改为记录真实服务进程
- Next entry point: `docs/handoffs/2026-03-18-launch-process-cleanup-fix.md`

### Discord 重复“正在思考中”排查

- Status: done
- Date: 2026-03-18
- Plan: N/A
- Handoff: `docs/handoffs/2026-03-18-discord-double-thinking-investigation.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `bash tests/regression/manual/test_opencode_acp_hone_mcp.sh`
- Current conclusion: 结论偏向入口被多个 consumer / 进程重复消费，而不是单次 `opencode_acp` run 自行双发 thinking
- Next entry point: `docs/handoffs/2026-03-18-discord-double-thinking-investigation.md`

### Runner 切换到 Gemini 3.1 Pro

- Status: done
- Date: 2026-03-18
- Plan: N/A
- Handoff: `docs/handoffs/2026-03-18-opencode-gemini-runner.md`
- Decision / ADR: `docs/adr/0002-agent-runtime-acp-refactor.md`
- Related PRs / commits: N/A
- Related runbooks / regressions: `bash tests/regression/manual/test_gemini_streaming.sh`
- Current conclusion: 默认 runner 已切到 `gemini_acp`，模型固定为 `gemini-3.1-pro-preview`
- Next entry point: `docs/handoffs/2026-03-18-opencode-gemini-runner.md`

## 2026-03-17

### IM 渠道共享入口收口

- Status: archived
- Date: 2026-03-17
- Plan: `docs/archive/plans/attachment-ingest-unify.md`
- Handoff: `docs/handoffs/2026-03-17-im-channel-core-refactor.md`
- Decision / ADR: `docs/adr/0002-agent-runtime-acp-refactor.md`
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo check -p hone-channels -p hone-imessage -p hone-feishu -p hone-telegram -p hone-discord`, `cargo test -p hone-channels`
- Current conclusion: 共享 `ingress` / `outbound` 抽象已收口；Discord / 飞书附件 ingest 与 KB 管线下沉到 `hone-channels`
- Next entry point: `docs/handoffs/2026-03-17-im-channel-core-refactor.md`

### 文档计划与 handoff 清理

- Status: done
- Date: 2026-03-17
- Plan: N/A
- Handoff: `docs/handoffs/2026-03-17-doc-context-cleanup.md`
- Decision / ADR: `docs/adr/0001-repo-context-contract.md`
- Related PRs / commits: N/A
- Related runbooks / regressions: N/A
- Current conclusion: 清空已完成计划、合并零碎 handoff，并把 `docs/current-plan.md` 恢复为活跃任务入口
- Next entry point: `docs/handoffs/2026-03-17-doc-context-cleanup.md`

### Legacy 兼容移除与数据迁移

- Status: done
- Date: 2026-03-17
- Plan: N/A
- Handoff: `docs/handoffs/2026-03-17-legacy-removal-and-migration.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: N/A
- Current conclusion: 历史 handoff 已补回入口，具体迁移细节见 handoff
- Next entry point: `docs/handoffs/2026-03-17-legacy-removal-and-migration.md`

### 项目清理（会话稳定性 / 渠道收敛）

- Status: done
- Date: 2026-03-17
- Plan: N/A
- Handoff: `docs/handoffs/2026-03-17-project-cleanup.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: N/A
- Current conclusion: 历史 handoff 已补回入口，具体清理结论见 handoff
- Next entry point: `docs/handoffs/2026-03-17-project-cleanup.md`

### 架构收敛与稳定性审计

- Status: done
- Date: 2026-03-17
- Plan: N/A
- Handoff: `docs/handoffs/2026-03-17-architecture-convergence-audit.md`
- Decision / ADR: `docs/adr/0002-agent-runtime-acp-refactor.md`
- Related PRs / commits: N/A
- Related runbooks / regressions: N/A
- Current conclusion: 历史 handoff 已补回入口，具体审计结论见 handoff
- Next entry point: `docs/handoffs/2026-03-17-architecture-convergence-audit.md`

### Identity 限额策略

- Status: done
- Date: 2026-03-17
- Plan: N/A
- Handoff: `docs/handoffs/2026-03-17-identity-quota-policy.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: N/A
- Current conclusion: 历史 handoff 已补回入口，具体策略结论见 handoff
- Next entry point: `docs/handoffs/2026-03-17-identity-quota-policy.md`

### 运行时管理员口令拦截

- Status: done
- Date: 2026-03-17
- Plan: N/A
- Handoff: `docs/handoffs/2026-03-17-register-admin-intercept.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: N/A
- Current conclusion: 历史 handoff 已补回入口，具体拦截链路见 handoff
- Next entry point: `docs/handoffs/2026-03-17-register-admin-intercept.md`
