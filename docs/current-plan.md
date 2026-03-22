# Current Plan Index

最后更新：2026-03-22
状态：有 3 个活跃任务

## 说明

- 本文件默认只保留满足准入标准的活跃任务索引；若临时保留“最近完成”，应在对应 handoff 落稳后尽快移除
- 只有需要持续跟踪的任务，才对应一份 `docs/current-plans/*.md`
- 单次 commit / sync / rebase、轻量脚本修补、无行为变化的小补丁等小任务不进入本索引
- 任务完成后：
  - 从本索引移除
  - 如需交接，更新或合并到 `docs/handoffs/*.md`
- 历史完成事项统一到 `docs/handoffs/` 查阅

## 活跃任务

- **ACP 对齐的 Agent Runtime 全栈重构**（2026-03-17）
  - 计划：`docs/current-plans/acp-runtime-refactor.md`
  - 状态：进行中（ACP runners 已接入 Hone MCP bridge；`gemini_acp initialize timeout` 已定位并修复）
- **用户上传文件追踪与 pageIndex 结合评估**（2026-03-13）
  - 计划：`docs/current-plans/file-upload-tracking.md`
  - 状态：进行中
- **大文件物理拆分重构**（2026-03-22）
  - 计划：`docs/current-plans/large-files-refactor.md`
  - 状态：进行中
## 最近完成

- **多渠道附件工程化卡点**（2026-03-22）
  - 计划/handoff：`docs/current-plans/channel-attachment-gate.md`、`docs/handoffs/2026-03-22-channel-attachment-gate.md`
  - 结果：共享附件 ingest 现统一拦截超限附件与异常图片；通用附件 5MB、图片 3MB，且图片会按最长边、总像素、长宽比做二次校验；被拒附件不会进入 prompt 与 KB，渠道 ack 会明确汇总拦截原因
  - 验证：`cargo test -p hone-channels`、`cargo check -p hone-channels -p hone-discord -p hone-feishu -p hone-telegram`

- **真群聊共享 Session 落地**（2026-03-19）
  - 计划/handoff：`docs/current-plans/group-shared-session.md`、`docs/handoffs/2026-03-19-group-shared-session.md`
  - 结果：群聊会话归属从 actor 扩展为显式 `SessionIdentity`；Telegram / Feishu / Discord 群消息现按“每个群一个 session”共享上下文，群输入带发言人标识，群 session 使用独立的恢复窗口与压缩阈值；Web 控制台改为按真实 `session_id` 浏览会话，并将群共享 session 标记为只读浏览
  - 验证：`cargo check -p hone-discord -p hone-feishu -p hone-telegram -p hone-imessage -p hone-web-api`、`cargo test -p hone-memory -p hone-channels`、`cargo test -p hone-channels -p hone-memory -p hone-web-api --no-run`、`bun run typecheck`（`packages/app`）

- **群聊回复追加链路统一**（2026-03-19）
  - 计划/handoff：`docs/current-plans/group-reply-append-chain.md`、`docs/handoffs/2026-03-19-group-reply-append-chain.md`
  - 结果：Discord / Telegram / Feishu 的群聊占位符统一保留为 `@用户 + 正在思考中...`；群聊 tool reasoning 不再覆盖占位符；最终首条回复统一补 `@用户`，多段回复会串成 reply 链，避免被中间消息打断
  - 验证：`cargo check -p hone-discord -p hone-feishu -p hone-telegram`、`cargo test -p hone-discord -p hone-telegram`

- **渠道运行态心跳替代 pid 判活**（2026-03-18）
  - handoff：`docs/handoffs/2026-03-18-channel-heartbeat-status.md`
  - 结果：四个渠道二进制都会每 30 秒写一次带 `pid` 的 `runtime/*.heartbeat.json`；`/api/channels` 已改为基于心跳新鲜度呈现运行状态，不再依赖 `runtime/*.pid` + `kill -0`
  - 验证：`cargo check -p hone-core -p hone-web-api -p hone-desktop -p hone-discord -p hone-feishu -p hone-telegram -p hone-imessage`、`cargo test -p hone-core -p hone-web-api`

- **launch.sh 真实进程清理修复**（2026-03-18）
  - handoff：`docs/handoffs/2026-03-18-launch-process-cleanup-fix.md`
  - 结果：`launch.sh` 改为先构建再直接启动 `target/debug/hone-*`，pid 文件现在记录真实服务进程，不再只记录 `cargo run` 包装进程
  - 验证：`bash -n launch.sh`、`cargo build -p hone-console-page -p hone-imessage -p hone-discord -p hone-feishu -p hone-telegram`、直接启动 `target/debug/hone-console-page` 后核对 `ps -p <pid> -o pid=,comm=,args=`

- **Discord 重复“正在思考中”排查**（2026-03-18）
  - handoff：`docs/handoffs/2026-03-18-discord-double-thinking-investigation.md`
  - 结果：确认 Discord 单次 `opencode_acp` run 不会自行双发 thinking；这次案例的 direct session 在 705ms 内落下了两条完全相同的 user message，更符合入口被两个独立 consumer / 进程重复消费
  - 验证：`sed -n '1,220p' data/sessions/Actor_discord__direct__483641214445551626.json`、`pgrep -lf hone-discord`、`bash tests/regression/manual/test_opencode_acp_hone_mcp.sh`、直接驱动 `opencode acp` 统计 `tool_call_count=1`

- **Runner 切换到 Gemini 3.1 Pro**（2026-03-18）
  - handoff：`docs/handoffs/2026-03-18-opencode-gemini-runner.md`
  - 结果：最终将默认 runner 切到 `gemini_acp`，并固定 `gemini-3.1-pro-preview`；已同步更新 runtime 配置与项目根种子配置
  - 验证：`gemini --version`、`bash tests/regression/manual/test_gemini_streaming.sh`、`printf 'Reply with exactly: HONE_HONECLI_GEMINI_ACP_OK\nquit\n' | cargo run -q -p hone-cli`

- **IM 渠道共享入口收口**（2026-03-17）
  - 计划/handoff：`docs/current-plans/attachment-ingest-unify.md`、`docs/handoffs/2026-03-17-im-channel-core-refactor.md`
  - 结果：新增共享 `ingress` / `outbound` 抽象，统一 IM 渠道的 dedup、session 锁、actor scope、出站占位/分段/流式探针；Discord/飞书附件 ingest 与 KB 管线下沉到 `hone-channels`；Feishu/iMessage 去掉基于 `gemini_cli` 的执行分支，改为统一消费 `AgentSession` 流式事件
  - 验证：`cargo check -p hone-channels -p hone-imessage -p hone-feishu -p hone-telegram -p hone-discord`、`cargo test -p hone-channels`、`cargo check --workspace --all-targets`

- **文档计划与 handoff 清理**（2026-03-17）
  - handoff：`docs/handoffs/2026-03-17-doc-context-cleanup.md`
  - 结果：清空 `docs/current-plans/` 中已完成计划，合并零碎 handoff，并把本索引恢复为活跃任务入口
- **Legacy 兼容移除与数据迁移**（2026-03-17）
  - handoff：`docs/handoffs/2026-03-17-legacy-removal-and-migration.md`
- **项目清理（会话稳定性 / 渠道收敛）**（2026-03-17）
  - handoff：`docs/handoffs/2026-03-17-project-cleanup.md`
- **架构收敛与稳定性审计**（2026-03-17）
  - handoff：`docs/handoffs/2026-03-17-architecture-convergence-audit.md`
- **Identity 限额策略**（2026-03-17）
  - handoff：`docs/handoffs/2026-03-17-identity-quota-policy.md`
- **运行时管理员口令拦截**（2026-03-17）
  - handoff：`docs/handoffs/2026-03-17-register-admin-intercept.md`
