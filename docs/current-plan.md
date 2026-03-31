# Current Plan Index

最后更新：2026-03-31
状态：有 8 个活跃任务

## 说明

- 本文件默认只保留满足准入标准的活跃任务索引；若临时保留“最近完成”，应在对应 handoff 落稳后尽快移除
- 只有需要持续跟踪的任务，才对应一份 `docs/current-plans/*.md`
- 单次 commit / sync / rebase、轻量脚本修补、无行为变化的小补丁等小任务不进入本索引
- 任务完成后：
  - 从本索引移除
  - 如需交接，更新或合并到 `docs/handoffs/*.md`
- 历史完成事项统一到 `docs/handoffs/` 查阅

## 活跃任务

- **Skill Runtime 对齐 Claude Code**（2026-03-31）
  - 计划：`docs/current-plans/skill-runtime-align-claude-code.md`
  - 状态：进行中（核心 skill runtime 已迁到“listing 披露 + 调用时完整注入 + slash/direct invoke + session 恢复”模型；hooks 真执行、turn-scope tool enforcement、watcher 热重载仍待 runner / infra 继续补齐）
- **Windows 桌面端打包可用性**（2026-03-28）
  - 计划：`docs/current-plans/windows-desktop-packaging.md`
  - 状态：进行中（已切换到跨平台 sidecar 准备脚本；待在具备 Rust/Bun 的 Windows 环境完成真实打包验证）
- **ACP 对齐的 Agent Runtime 全栈重构**（2026-03-17）
  - 计划：`docs/current-plans/acp-runtime-refactor.md`
  - 状态：进行中（ACP runners 已接入 Hone MCP bridge；`gemini_acp initialize timeout` 已定位并修复）
- **用户上传文件追踪与 pageIndex 结合评估**（2026-03-13）
  - 计划：`docs/current-plans/file-upload-tracking.md`
  - 状态：进行中
- **大文件物理拆分重构**（2026-03-22）
  - 计划：`docs/current-plans/large-files-refactor.md`
  - 状态：进行中
- **Desktop 渠道监听状态与多进程 PID 对齐**（2026-03-28）
  - 计划：`docs/current-plans/desktop-channel-status-multiprocess.md`
  - 状态：进行中（heartbeat 已改为后端主动上报主路径，`/api/channels` 已支持多进程聚合与 PID 展示；desktop 角标下拉已提供“清理多余进程”快捷按钮）
- **Desktop / Runtime 启动锁收口**（2026-03-29）
  - 计划：`docs/current-plans/desktop-runtime-startup-locks.md`
  - 状态：进行中（为桌面主进程、bundled backend 与各渠道 listener 增加统一启动锁，要求任一锁冲突时整体拒绝启动）
- **Desktop 启动锁冲突体验优化方案**（2026-03-29）
  - 计划：`docs/current-plans/desktop-startup-lock-ux-strategy.md`
  - 状态：进行中（先输出不改代码的策略方案，目标是把“锁冲突直接报错”升级为自动接管、分层恢复和可解释降级的启动体验）
## 最近完成

- **macOS DMG Release 打包收口**（2026-03-31）
  - handoff：`docs/handoffs/2026-03-31-macos-dmg-release-packaging.md`
  - 结果：新增 `make_dmg_release.sh` 并真实产出 Apple Silicon / Intel 两套 DMG；release 包现会内置 `hone-mcp` 与 macOS `opencode`，desktop packaged/runtime 启动时会补齐 app sandbox data/runtime/sandbox 环境，并对 bundled runtime 启动锁冲突做一次按 pid 的定向清理重试
  - 验证：`cargo test -p hone-channels runners::tests::resolve_opencode_command_prefers_bundled_env_override -- --exact`、`cargo test -p hone-channels sandbox::tests::sandbox_base_dir_prefers_hone_data_dir_before_temp -- --exact`、`cargo check -p hone-desktop -p hone-channels -p hone-mcp`、`node --check scripts/prepare_tauri_sidecar.mjs`、`bash -n make_dmg_release.sh`、`./make_dmg_release.sh x86_64-apple-darwin`、`./make_dmg_release.sh aarch64-apple-darwin`、`./launch.sh --desktop`

- **定时任务输出净化与 Tavily 失败隔离**（2026-03-31）
  - handoff：`docs/handoffs/2026-03-31-scheduler-output-and-search-failure-hygiene.md`
  - 结果：heartbeat / 定时任务现在能从“前缀解释文本 + JSON”里抽出真正的 JSON 结果，不再把解释过程和控制输出一起发给用户；`web_search` 在 Tavily 不可用时改为返回脱敏的 unavailable 结构，同时这类临时失败结果不再持久化进会话工具上下文
  - 验证：`cargo test -p hone-tools`、`cargo test -p hone-channels`

- **额度与定时任务可靠性修复**（2026-03-29）
  - handoff：`docs/handoffs/2026-03-29-quota-scheduler-reliability.md`
  - 结果：普通用户每日对话额度从 20 调整为 12；非 heartbeat 定时任务新增“错过原始 5 分钟窗口后的同日单次补触发”能力，避免渠道 / 桌面进程在原定时刻后恢复时当天任务永久丢失；heartbeat 的 JSON 解析失败现在直接安全抑制，不再把 `{"` 之类的控制输出发给用户
  - 验证：`cargo test -p hone-memory`、`cargo test -p hone-channels`

- **单一聊天范围配置与群聊忙碌态控制**（2026-03-27）
  - handoff：`docs/handoffs/2026-03-27-chat-scope-busy-guard.md`
  - 结果：Feishu / Telegram / Discord 统一将 `dm_only` 收敛为 `chat_scope=DM_ONLY|GROUPCHAT_ONLY|ALL`，并兼容旧 `dm_only` 配置；三渠道群聊显式触发新增 busy 生命周期控制，当前一条消息仍在处理中时，新来的 `@bot` 会立即收到等待提示，同时问题文本会继续保留在群聊预触发窗口中
  - 验证：`cargo check -p hone-core -p hone-channels -p hone-discord -p hone-feishu -p hone-telegram`、`cargo test -p hone-core -p hone-channels`、`cargo test -p hone-discord -p hone-feishu -p hone-telegram --no-run`

- **子模型配置与心跳任务调度**（2026-03-26）
  - handoff：`docs/handoffs/2026-03-26-heartbeat-submodel-scheduler.md`
  - 结果：Desktop 设置页新增 OpenRouter 子模型配置；会话压缩切到子模型；cron 新增 `heartbeat` 任务类型与标签；heartbeat 任务按 30 分钟轮询、未命中时不投递，并在任务中心与 cron API 中正常显示
  - 验证：`cargo test -p hone-memory -p hone-scheduler -p hone-tools -p hone-core -p hone-web-api -p hone-channels`、`cargo check -p hone-desktop`、`npm run typecheck`（`packages/app`）

- **Session SQLite 影子写入与运行时切换**（2026-03-26）
  - handoff：`docs/handoffs/2026-03-26-session-sqlite-cutover.md`
  - 结果：SessionStorage 已支持 `json | sqlite` 运行时后端切换；SQLite shadow write 与 runtime 主读均已接入，`/api/users` 已改为统一走 SessionStorage，不再直扫 `data/sessions`；本机 runtime 已切到 SQLite，JSON 继续双写作回退镜像
  - 验证：`cargo test -p hone-memory`、`cargo test -p hone-channels --no-run`、`cargo test -p hone-web-api --no-run`、`bash tests/regression/ci/test_session_sqlite_migration.sh`、重启服务后验证 `/api/meta` `/api/users` `/api/history`

- **群聊预触发窗口统一改造**（2026-03-24）
  - handoff：`docs/handoffs/2026-03-24-group-pretrigger-window-unify.md`
  - 结果：Telegram / Discord / 飞书群聊统一为“未触发先静默缓存、显式 @/reply-to-bot 时再执行”的模型；共享层新增按群 session 维护的预触发滑动窗口，触发时会把最近 10 条、5 分钟内的群文本正式写入共享群 session；Discord 已移除 question-signal 与短窗批处理路径，群聊首条回复三渠道统一固定 mention 触发者
  - 验证：`cargo check -p hone-channels -p hone-discord -p hone-feishu -p hone-telegram`、`cargo test -p hone-channels -p hone-core`、`cargo test -p hone-discord -p hone-telegram --no-run`

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
