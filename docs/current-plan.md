# Current Plan Index

最后更新：2026-07-22
状态：有 8 个活跃任务

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

- **Public Community Edge 生产分阶段上线**
  - 状态：`in_progress`
  - 计划：`docs/current-plans/public-community-edge-production-rollout.md`
  - 摘要：私有 R2 快照已发布；全新的 `hone-public-community-edge` 已部署到精确路由并保持无 secret、无启用变量的 fail-closed `503`。实现提交 `385e35b0` / `100f5608` 已推到 `main`；自动 Pages 构建仍将 edge discovery 编译移除。精确 `100f5608` 的五个运行二进制、public bundle、skills/soul 和哈希 manifest 已准备在独立不可变目录，当前旧后端仍运行 `d58ef12b` 且新 edge-session 为 `404`。下一步只由外部服务执行受控重启，先验证 `mode=off` 的 `200 enabled=false`；共享 secret、backend `shadow/prefer`、Worker 激活和 Pages discovery 均未开始

- **跨市场 ticker 解析架构修复**
  - 状态：`in_progress`
  - 计划：`docs/current-plans/ticker-resolution-architecture.md`
  - 摘要：系统按更新后的 ADR 0004 / D-2026-07-19-08 收口跨市场 ticker 与 Interactive 自然 Agent 循环。主 Agent 从完整原话识别全部点名标的，为每个标的声明稳定 `entity_route` 和 call-scoped `identity_match`，普通小写/混合大小写 ticker 仍走 normalized exact-symbol；实体与证据 ledger 只驱动真实业务工具的 `Required → Auto`，不再暴露 `finish_research`，也不再执行 handoff、opaque locator 纠正、tool-free terminal、终稿审计、第二次生成、固定拒答或答案回写。工具轮继续缓冲以避免 preamble/reset/闪烁；同一 Agent 在同一上下文加载 DataFetch/Web 结果后自然输出唯一 DirectFinal，服务端只做安全清理并保证可见/持久化字节一致。最多四条/4000 字的近期用户原话仅用于追问指代，历史 assistant/tool/行情不会进入本轮事实链；未知模型工具也不会再闪 ToolStatus 错误。报价源时间优先使用 `hone_quote_time.beijing`；`market_date_new_york` 不能推出“纽交所/收盘价”，交易所只能来自结构化 exchange 字段；关系强度没有当前证据时必须中性表述。精确 `d4d44735` 的失败 canary 中十次 DataFetch/Web 均成功，但整轮仍有七次模型调用、耗时 92.435 秒，说明根因是 finish/correction/terminal 完成链而非 FMP/DataFetch/Tavily。自然 DirectFinal 实现与完整本地门禁已通过，并已无冲突 rebase 最新 Web 主干；精确构建部署和 relationship/valuation/single-ticker/non-finance canary 仍 pending。umbrella 任务之后仍需处理 scheduler 800G/NAND/AST/SEC P2，因此保持 `in_progress`、不归档
  - 2026-07-21 TTFT 跟进：原问题 `大A有没有类似CRWV、Nebius这样的数据中心的标的` 约 `99.45s` 才完成，含 `14.377s` 同步预压缩与约 `39.45s` 被完整缓冲的最终模型流。D-2026-07-21-01 将无 compact/summary 的快路径限定为本轮自包含且带显式证券种子的请求，从同一 Session 快照恢复最多四条 user-only 指代及独立的 active invoked-skill prompt，并要求模型把独立候选发现放进同一 tool-call batch（执行仍串行）。Web 仅在 canonical 头及后续正文安全完整行被唯一发布 sink 成功接收后逐段累加 committed prefix；精确构建、重部署和原问题 fresh-actor canary 正在执行，umbrella 仍因 scheduler P2 保持 `in_progress`

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
