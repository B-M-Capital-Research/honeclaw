# Current Plan Index

最后更新：2026-04-13
状态：有 9 个活跃任务

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

- **Canonical Config 与 Runtime Apply 统一改造**
  - 状态：`in_progress`
  - 计划：`docs/current-plans/canonical-config-runtime-apply.md`
  - 摘要：canonical config、effective-config、CLI 管理面、安装 / onboarding、标准 Homebrew tap 与 OpenCode 本机配置继承已落地；当前继续收口 `hone-cli onboard` 渠道回退体验、安装版 Web 静态资源打包，以及 desktop bundled 模式下的 live/component/full apply 语义
- **Skill Runtime 对齐 Claude Code**
  - 状态：`in_progress`
  - 计划：`docs/current-plans/skill-runtime-align-claude-code.md`
  - 摘要：核心 skill runtime 已迁到“listing 披露 + 调用时完整注入 + slash/direct invoke + session 恢复”模型；hooks 真执行、turn-scope tool enforcement、watcher 热重载仍待 runner / infra 继续补齐
- **Windows 桌面端打包可用性**
  - 状态：`in_progress`
  - 计划：`docs/current-plans/windows-desktop-packaging.md`
  - 摘要：已切换到跨平台 sidecar 准备脚本；待在具备 Rust/Bun 的 Windows 环境完成真实打包验证
- **ACP 对齐的 Agent Runtime 全栈重构**
  - 状态：`in_progress`
  - 计划：`docs/current-plans/acp-runtime-refactor.md`
  - 摘要：ACP runners 已接入 Hone MCP bridge；`gemini_acp initialize timeout` 已定位并修复，runner timeout 已收敛到顶层 `step=3 分钟 / overall=20 分钟` 两档，`session/load timeout` 也已改为自动回退新 session，仍需继续收口 runner contract 与全栈行为对齐
- **用户上传文件追踪与 pageIndex 结合评估**
  - 状态：`in_progress`
  - 计划：`docs/current-plans/file-upload-tracking.md`
  - 摘要：继续评估上传文件追踪与 `pageIndex` 联动方案，待补实现范围与验证矩阵
- **Desktop 渠道监听状态与多进程 PID 对齐**
  - 状态：`in_progress`
  - 计划：`docs/current-plans/desktop-channel-status-multiprocess.md`
  - 摘要：heartbeat 已改为后端主动上报主路径，`/api/channels` 已支持多进程聚合与 PID 展示；desktop 角标下拉已提供“清理多余进程”快捷按钮
- **Desktop / Runtime 启动锁收口**
  - 状态：`in_progress`
  - 计划：`docs/current-plans/desktop-runtime-startup-locks.md`
  - 摘要：为桌面主进程、bundled backend 与各渠道 listener 增加统一启动锁，要求任一锁冲突时整体拒绝启动
- **Desktop 启动锁冲突体验优化方案**
  - 状态：`in_progress`
  - 计划：`docs/current-plans/desktop-startup-lock-ux-strategy.md`
  - 摘要：先输出不改代码的策略方案，目标是把“锁冲突直接报错”升级为自动接管、分层恢复和可解释降级的启动体验
- **主系统 `/report` 指令桥接本地研报 Workflow**
  - 状态：`in_progress`
  - 计划：`docs/current-plans/report-command-bridge.md`
  - 摘要：在主系统各渠道入口增加 `/report 公司名` 与 `/report 进度` 预拦截，桥接到本地 private workflow runner 的 `company_report` 启动与进度查询接口
