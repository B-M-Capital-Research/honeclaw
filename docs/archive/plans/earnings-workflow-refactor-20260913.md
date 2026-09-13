# 财报工作流重构与 TEM 验收

- title: 财报工作流重构与 TEM 验收
- status: archived
- created_at: 2026-09-13
- updated_at: 2026-09-13
- owner: Codex
- related_files: skills/earnings-research/, packages/app/src/pages/chat.tsx, packages/app/src/lib/public-chat.ts, crates/hone-web-api/src/routes/public.rs, crates/hone-channels/src/agent_session/, crates/hone-channels/src/runners/opencode_acp.rs
- related_docs: docs/runbooks/backend-deployment.md, docs/repo-map.md, docs/invariants.md

## Goal

两个快捷入口保留弹窗，仅输入公司名字或代码；自动获取财报与电话会资料，沿用用户指定 Dify 原工作流阶段及 Prompt，移除额外内容门禁，加强 PDF 技术稳定性；TEM 两模式真实验收后部署生产。

## Scope

- [x] 拉取 origin/main；原目录有未提交改动及本地提交，使用独立 worktree，基线 bbdc828c。
- [x] 读取两个 Dify 当前工作流，核对本地 Prompt 与阶段。
- [x] 精简弹窗，落实内置材料检索；清理额外内容约束。
- [x] 诊断并修复 PDF 渲染、失败恢复与附件持久化。
- [x] TEM 本地 canary 发现 OpenCode 全局 enabled_providers 覆盖专用路由；在 crates/hone-channels/src/runners/opencode_acp.rs 修复显式 OpenRouter 路由并增加回归。

- [x] canary 确认现有 web_search 只返回摘要；添加显式可选原文检索及公开 URL 读取（生产搜索代理不提供正文；校验公网 DNS、逐跳重定向及大小/时限；保持默认轻量搜索），配套 provider 请求与正文标注回归。

- [x] 第二轮 canary 仍跳过原文读取：在工作流输入准备阶段自动搜取原始材料作为上下文（取材失败披露、不中止报告），保留原 Prompt 与生成阶段；增加只读预取/重试复用回归。

- [x] 第三轮发现通用 system prompt 仍强制其它投研技能；专用财报使用独立系统提示与原 Skill 上下文，去掉通用技能推荐/模板干扰并回归证明。

- [x] Linux 原文 canary 定位到 IR CDN 对请求 User-Agent 的兼容差异；修复公开读取客户端标识并保留真实原文回归，再重跑双模式。

- [x] 同一原 Prompt 的模型对照 canary：原文已完整输入，但 Gemini 仍遗漏盈利驱动、并购单位出错；测试现有配置支持的 `anthropic/claude-opus-4.8`，仅在对照结果支持时切换财报专用模型，不增加内容门禁或修改原 Prompt。

## Validation

- [x] 相关前端、Rust、PDF 技术回归与仓库 CI 契约。
- [x] TEM 财报前瞻、财报分析各一轮真实调用；保存正文/PDF、耗时、调用与错误证据；逐页检查 PDF。
- [x] 精确 revision 部署，核对前后端版本、健康、附件下载与生产会话。

## Documentation Sync

- [x] 更新 docs/repo-map.md、docs/invariants.md、docs/decisions.md 中受影响行为及 docs/runbooks/backend-deployment.md 操作变化。
- [x] 记录 docs/handoffs/2026-09-13-earnings-workflow-refactor.md 验证与部署证据。
- [x] 完成后移除活跃索引，归档计划至 docs/archive/plans/ 并更新 docs/archive/index.md。

## Risks / Open Questions

保留身份授权、actor 隔离、输入/路径/进程安全与 artifact 归属等技术边界；不引入内容裁判。需核对当前 Dify 登录态、生产模型与部署入口。原目录用户改动不带入本任务。

## Completion

代码 `161fe4b3` 已上主干并部署；原 Prompt 对照、双模式真实 canary、逐页 PDF 检查、浏览器公司输入与线上 OSS/重启下载验证完成。完整结果、历史全仓失败、模型配置和回滚入口见 `docs/handoffs/2026-09-13-earnings-workflow-refactor.md`。
