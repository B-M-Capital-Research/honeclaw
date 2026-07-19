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
  - 摘要：系统继续按 ADR 0004 / D-2026-07-19-05 / D-2026-07-19-06 / D-2026-07-19-07 收口跨市场 ticker、同一 Agent 研究循环、当前轮证据绑定和 Web 单次提交。Interactive 发布否决、固定拒答和答案回写保持移除；服务端不按自然语言分隔符、大小写、长度或 ticker 形状猜实体。主 Agent 从完整原话识别全部点名标的，为每个标的声明稳定 `entity_route` 和 call-scoped `identity_match`，首轮只并行 search；finance 激活后的工具轮只继续真实工具，结构 route 覆盖后继续工具或进入 finish-only `finish_research`。终局前由运行时从匹配的本轮 invocation/result 构造 citable source catalog，仅保留成功非 search DataFetch 标量数据与具有原始 URL/可引用摘要的 Web 结果，并把真实 opaque ID 动态枚举进 flat schema；工具名和 data type 不是 ID。唯一 Web result number + verbatim excerpt 可机械恢复 generic Web ID，DataFetch 不猜 ID。catalog 非空但 locator 无法解析或 handoff 没有有效 related fact/scoped fallback 时，同一 Agent 在任何终稿生成前最多收到一次隐藏的 locator-only 纠正；valid sibling facts 保留，纠正后仍没有有效 fact/fallback 则失败，绝不进入 unrestricted empty-evidence terminal。只有本轮 citable catalog 为空、真实来源均不可用时才允许直接 gaps-only 并披露具体缺项。混合业务工具与 finish 仍仅执行业务工具；terminal 仍只接收当前 system/user、精确 Session 时间和 resolved/scoped evidence，完整 DirectFinal 仍原样接受，服务端不做 post-answer gate。精确 `ae4d9a8f` 已完成完整门禁、不可变构建、零活跃会话重启和 PG/OSS/API/channel 健康检查；正确 direct canary 单流且历史一致，但 TTFT/完成约 101.748 秒，generic `web_search`/`quote`/`profile` ID 使所有引用被丢弃，空 evidence terminal 随后反转来源方向并扩写无证据关系，因此内容验收失败。D07 focused verification 已通过 hone-agent 101/101、hone-channels 642/642、finance static 39/39；完整门禁、精确重建部署和 relationship/valuation/non-finance canary 仍 pending，umbrella 任务之后仍需处理 scheduler 800G/NAND/AST/SEC P2

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
