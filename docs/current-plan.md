# Current Plan Index

最后更新：2026-04-29
状态：有 7 个活跃任务

## 说明

- 本文件只保留满足准入标准的活跃任务索引，不再混入“最近完成”
- 每个活跃任务必须对应一份 `docs/current-plans/*.md`
- 历史完成事项统一从 `docs/archive/index.md` 查入口，再按需查看对应 `docs/handoffs/*.md` 或 `docs/archive/plans/*.md`
- 任务退出活跃态后：
  - 从本索引移除
  - 如需交接，更新或新增 `docs/handoffs/*.md`
  - 如需长期检索，补充到 `docs/archive/index.md`
  - 如已有计划页，移入 `docs/archive/plans/*.md`

## 活跃任务

- **Active Bug Burn-down 2026-04-28**
  - 状态：`in_progress`
  - 计划：`docs/current-plans/active-bug-burn-down-2026-04-28.md`
  - 摘要：集中清理 `docs/bugs/README.md` 活跃缺陷；本轮已从 21 个活跃缺陷降到 3 个，继续闭环 cron schedule/prompt 错配、OpenAI-compatible numeric 4xx 错误解析、heartbeat 重复提醒/近阈值误报/空结构输出/迭代预算收口、原油 heartbeat 来源归因约束、disabled channel zombie pid 清理、event-engine Feishu current-app open_id 解析、Web scheduler 失败落库、Feishu scheduler running/pending 终态覆盖、Feishu scheduler 内部错误外发抑制，并复核 event-engine classifier / convergence guard 已具备代码收口；剩余项继续按共享契约加固，不对不可控网络、凭据或模型波动做脆弱特判
- **Chart Visualization Skill 与多通道 PNG 投递**
  - 状态：`in_progress`
  - 计划：`docs/current-plans/chart-visualization-skill.md`
  - 摘要：新增 `chart_visualization` skill 与 Python PNG 渲染器，扩展 `skill_tool` 结构化 artifact 契约，统一 `file:///abs/path.png` 助手可见媒体标记，并让 Web / Feishu / Telegram / Discord 在保留 text-image-text 顺序的同时正确渲染或上传本地图表
- **Feishu P1 直聊与定时任务可靠性修复批次**
  - 状态：`in_progress`
  - 计划：`docs/current-plans/feishu-p1-reliability-batch.md`
  - 摘要：集中修复当前活跃的 Feishu `P1` 缺陷；本轮已补 Feishu 出站短重试、scheduler `running/pending` 起始台账、空回复失败态、timeout partial 清洗、compact marker 可见净化，以及 multi-agent 对“定时任务/短澄清”请求的工具分流与可信本地结果直返
- **Feishu 直聊 placeholder 假启动与 release runner 生效链路修复**
  - 状态：`in_progress`
  - 计划：`docs/current-plans/feishu-direct-placeholder-followup-fix.md`
  - 摘要：继续修复 Feishu 私聊消息只发 placeholder 不进主链路的问题，同时收口 release app 仍读取 legacy config 导致 runner 改完不立即生效，并修复 desktop UI 缺少 `codex_acp` 入口造成的 runner 观测不一致
- **Canonical Config 与 Runtime Apply 统一改造**
  - 状态：`in_progress`
  - 计划：`docs/current-plans/canonical-config-runtime-apply.md`
  - 摘要：canonical config、effective-config、CLI 管理面、安装 / onboarding、标准 Homebrew tap 与 OpenCode 本机配置继承已落地；当前继续收口 `hone-cli onboard` 渠道回退体验、安装版 Web 静态资源打包，以及 desktop bundled 模式下的 live/component/full apply 语义
- **Skill Runtime 对齐 Claude Code**
  - 状态：`in_progress`
  - 计划：`docs/current-plans/skill-runtime-align-claude-code.md`
  - 摘要：核心 skill runtime 已迁到“listing 披露 + 调用时完整注入 + slash/direct invoke + session 恢复”模型；本轮进一步补上 stage-aware skill 可见性、`HONE_SKILLS_DIR` 透传与 `cron_job` 可执行性对齐，确保当前会话里看得见的 skill 默认都能真正调用；hooks 真执行、watcher 热重载与更细粒度 turn enforcement 仍待 runner / infra 继续补齐
- **ACP 对齐的 Agent Runtime 全栈重构**
  - 状态：`in_progress`
  - 计划：`docs/current-plans/acp-runtime-refactor.md`
  - 摘要：ACP runners 已接入 Hone MCP bridge；runner timeout 已收敛到顶层 `step=3 分钟 / overall=20 分钟` 两档，`session/load timeout` 也已改为自动回退新 session；当前继续收口 ACP transcript 边界、compact 防泄漏与 system prompt reseed 语义。session 持久化已切到 `version=4 + user/assistant + content[] + status` 统一模型，codex/opencode 可互相切换恢复，`codex_cli` 也纳入同一 normalized 持久化契约，`multi-agent` 会把 search/answer 两阶段 transcript 一并回写 session；`gemini_acp` 已禁用并给出迁移提示
