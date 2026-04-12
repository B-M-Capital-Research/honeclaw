# Archive Index

Last updated: 2026-04-12

Use this file as the historical entry point for completed or paused work that should remain discoverable.

## 2026-04-12

### 公司画像与长期基本面追踪

- Status: done
- Date: 2026-04-12
- Plan: `docs/archive/plans/company-portrait-tracking.md`, `docs/archive/plans/company-portrait-skill-framework.md`, `docs/archive/plans/company-research-actor-spaces.md`, `docs/archive/plans/remove-kb-memory-surface.md`
- Handoff: `docs/handoffs/2026-04-12-company-portrait-tracking.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo test -p hone-memory company_profile -- --nocapture`, `cargo check -p hone-memory -p hone-tools -p hone-web-api -p hone-channels`, `bun run --cwd packages/app typecheck`
- Current conclusion: Hone 已具备 Markdown 形式的公司画像与事件时间线、按 actor 展示的画像 Web 视图（允许彻底删除），以及更贴近投研档案的 `company_portrait` skill；画像文档现在直接落在 actor sandbox 的 `company_profiles/` 中，由 agent 使用 runner 原生文件读写维护，不再依赖专用 mutation tool、公共画像目录或 KB 记忆入口
- Next entry point: `docs/handoffs/2026-04-12-company-portrait-tracking.md`

### CLI 首装 Onboarding 与安装向导

- Status: done
- Date: 2026-04-12
- Plan: `docs/archive/plans/cli-onboarding-install-wizard.md`
- Handoff: `docs/handoffs/2026-04-12-cli-onboarding-install-wizard.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `docs/runbooks/hone-cli-install-and-start.md`, `bash tests/regression/manual/test_install_bundle_smoke.sh`, `cargo check -p hone-cli`, `cargo test -p hone-cli`
- Current conclusion: `hone-cli` 已支持首装 `onboard/setup` TUI，能够探测本机 runner、在不强迫 Hone 侧填写 OpenCode provider 配置的前提下切到 `opencode_acp`，并按渠道逐个引导启用与填写本地必填字段；GitHub release 安装脚本在交互终端下会询问是否立即运行该向导
- Next entry point: `docs/handoffs/2026-04-12-cli-onboarding-install-wizard.md`

### Desktop Rust Check 与 IDE 语法检查解耦

- Status: done
- Date: 2026-04-12
- Plan: `docs/archive/plans/desktop-rust-check-workflow.md`
- Handoff: `docs/handoffs/2026-04-12-desktop-rust-check-workflow.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo check --workspace --all-targets --exclude hone-desktop`, `HONE_SKIP_BUNDLED_RESOURCE_CHECK=1 cargo check -p hone-desktop`, `HONE_SKIP_BUNDLED_RESOURCE_CHECK=1 cargo check --workspace --all-targets`
- Current conclusion: 默认 workspace Rust 检查继续排除 `hone-desktop`；desktop crate 新增开发态 sidecar 校验豁免开关，VSCode rust-analyzer 默认携带该 env，因此 IDE / 本地 `cargo check` 不再被缺失的 Tauri bundled binaries 阻塞
- Next entry point: `docs/handoffs/2026-04-12-desktop-rust-check-workflow.md`

### Hone CLI Config MVP 与可安装启动流

- Status: done
- Date: 2026-04-12
- Plan: `docs/archive/plans/hone-cli-config-mvp.md`
- Handoff: `docs/handoffs/2026-04-12-hone-cli-config-mvp.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `docs/runbooks/hone-cli-install-and-start.md`, `cargo check --workspace --all-targets --exclude hone-desktop`, `cargo test -p hone-core`, `cargo test -p hone-cli`
- Current conclusion: `hone-cli` 已具备 `config / configure / models / channels / status / doctor / start` 管理面；shared runtime overlay service 已供 CLI 与 desktop 共用；macOS / release 安装链路支持 `hone-cli start`，且已补齐首次 runtime config seed 行为
- Next entry point: `docs/handoffs/2026-04-12-hone-cli-config-mvp.md`

### Local 私有 Workflow Runner（公司研报 v1）

- Status: done
- Date: 2026-04-12
- Plan: `docs/archive/plans/local-workflow-runner.md`
- Handoff: `docs/handoffs/2026-04-12-local-workflow-runner.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `cd local/workflow && bun test`, `cd local/workflow && bun run bootstrap-config`, `cd local/workflow && bun build app/app.js server/index.ts server/cli.ts --outdir /tmp/local-workflow-build`, `WORKFLOW_RUNNER_PORT=3213 bun run start`
- Current conclusion: 在 `local/workflow/` 下新增独立本地 workflow runner，并在后续迭代中补齐紧凑工作台、运行级 prompt override、SSE 去重续流、停止接口、单实例串行、Python UTF-8/旧版本注解兼容，以及结构化进度与节点详情观测；当前 `company_report` 入口既可在页面里运行/观察/停止，也可通过 `bun run client` 从本机其它位置发起并监听进度
- Next entry point: `docs/handoffs/2026-04-12-local-workflow-runner.md`

## 2026-04-11

### 金融自动化合同回归闭环

- Status: done
- Date: 2026-04-11
- Plan: `docs/archive/plans/finance-automation-contract-loop.md`
- Handoff: `docs/handoffs/2026-04-09-finance-automation-contract-loop-round1.md`
- Decision / ADR: N/A
- Related PRs / commits: N/A
- Related runbooks / regressions: `bash tests/regression/ci/test_finance_automation_contracts.sh`, `bash tests/regression/run_ci.sh`
- Current conclusion: finance 固定 9 样本合同切片已从 `success=5 review=1 fail=3` 收口到 `success=9 review=0 fail=0`；剩余 skill policy wording 漂移已全部修正
- Next entry point: `docs/handoffs/2026-04-09-finance-automation-contract-loop-round1.md`

### 大文件物理拆分重构

- Status: done
- Date: 2026-04-11
- Plan: `docs/archive/plans/large-files-refactor.md`
- Handoff: `docs/handoffs/2026-04-11-architecture-tightening-round1.md`
- Decision / ADR: `docs/decisions.md`
- Related PRs / commits: N/A
- Related runbooks / regressions: `cargo check --workspace --all-targets --exclude hone-desktop`, `cargo test --workspace --all-targets --exclude hone-desktop`, `bun run test:web`, `bash tests/regression/run_ci.sh`
- Current conclusion: runtime override和渠道启动已收口到共享层；desktop sidecar、Feishu / Telegram 渠道热点与前端 settings 纯状态逻辑已按职责拆开，验证矩阵已跑通
- Next entry point: `docs/handoffs/2026-04-11-architecture-tightening-round1.md`

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
