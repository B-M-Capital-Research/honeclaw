# Current Plan Index

最后更新：2026-07-19
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

- **跨市场 ticker 解析架构修复**
  - 状态：`in_progress`
  - 计划：`docs/current-plans/ticker-resolution-architecture.md`
  - 摘要：系统审计并统一修复普通美股、缩写冲突、share class、数字开头国际代码、交易所后缀和指数/加密代码的提取、规范化、DataFetch 精确解析与失败语义；Interactive 发布否决已移除。当前阶段按 ADR 0004 / D-2026-07-19-03 把研究与结束权统一在同一个 Agent business+finish loop：同一 Agent 完整读取原问题，为每个点名标的声明稳定、不同且区分大小写的 `entity_route`，对每个标的分别 search，并在每次调用中明确 call-scoped `identity_match=exact_symbol|name_or_alias`，后续 refinement、quote、profile/snapshot 等调用原样复用该路线。服务端不按问法分隔符、大小写、长度或 ticker 形状猜实体/匹配模式；首个 exact-symbol 声明形成持久同代码约束，跨模块代码等价统一使用 `hone-core::provider_symbol` 的有限 provider 方言。任何可解析的显式 route 提及都会立即留下 pending 路线，但必须先完成自己的有效 search 和结果才接收 quote/profile/非 search 证据。未带 route 的兼容 search 只按区分大小写的逐字 `query` / `refines_query` / `supersedes_query` 单源迁移，后两者严格互斥；迁移只保留尝试历史、逐字别名和既有 exact constraint，绝不继承旧候选、quote、profile/asset-route 或 post 证据。空结果连续计数与 post follow-up 按当前候选代重置，失败补查不能把 CWY 证据转成 CRWV，未带 symbol 的 follow-up 也只在全局仅有一条 active route 时归属。DataFetch 执行器和账本共享字段优先级、类型和目标校验 helper，错误类型、冲突字段、畸形批量 symbol 或错误大小写工具名都不能形成虚假证据。结构账本只控制显式 `finish_research` schema 与 business-tool `Required → Auto`，绝不参与 DirectFinal 接受、Interactive 发布、重试或正文持久化；自然 `Stop + Done` 终稿始终原样采用。两条完成路径共享证据契约、稳定 Session 时间前缀和 Agent 关系结论删除式自检；该自检不是服务端过滤或改写。Interactive 服务端仅做协议/路径安全清洗与媒体稳定化，不做市场文案改写、完整性拒答或工具结果重构，并保证可见正文与持久化一致。FunctionCalling runner 同时执行 step deadline 与全循环不重置的 absolute overall deadline，工具失败/超时保留失败 `ToolCallMade`，不确定写操作立即停止同循环重放。最终 route 修复后的完整回归、精确提交部署和 production relationship/valuation/non-finance canary 正在收口，之后再处理 scheduler 800G/NAND/AST/SEC P2

- **Active Bug Burn-down 2026-04-28**
  - 状态：`in_progress`
  - 计划：`docs/current-plans/active-bug-burn-down-2026-04-28.md`
  - 摘要：集中清理 `docs/bugs/README.md` 活跃缺陷；2026-06-09 远端先关闭 3 条文案污染 P3，本轮继续验证并修复剩余 4 条活跃 bug，当前活跃待修复队列清空
- **Chart Visualization Skill 与多通道 PNG 投递**
  - 状态：`in_progress`
  - 计划：`docs/current-plans/chart-visualization-skill.md`
  - 摘要：新增 `chart_visualization` skill 与 Python PNG 渲染器，扩展 `skill_tool` 结构化 artifact 契约，统一 `file:///abs/path.png` 助手可见媒体标记，并让 Web / Feishu / Telegram / Discord 在保留 text-image-text 顺序的同时正确渲染或上传本地图表
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
  - 摘要：ACP runners 已接入 Hone MCP bridge；runner timeout 已收敛到顶层 `step=3 分钟 / overall=20 分钟` 两档，`session/load timeout` 也已改为自动回退新 session；当前继续收口 ACP transcript 边界、compact 防泄漏、system prompt reseed 语义，以及 ACP 子进程 / `hone-mcp` 生命周期泄漏修复。session 持久化已切到 `version=4 + user/assistant + content[] + status` 统一模型，codex/opencode 可互相切换恢复，`codex_cli` 也纳入同一 normalized 持久化契约；旧 `function_calling` 与 `multi-agent` 已退休，`gemini_acp` 已禁用并给出迁移提示
