# Earnings Native E2E Runtime Validation

- title: 财报原生 Skill 真实全链路续验收
- status: done
- created_at: 2026-08-04
- updated_at: 2026-08-04
- owner: Codex
- related_files:
  - crates/hone-web-api/src/routes/public.rs
  - crates/hone-channels/src/execution.rs
  - crates/hone-channels/src/agent_session/core.rs
  - crates/hone-channels/src/turn_builder.rs
  - crates/hone-channels/src/investment_response_guard.rs
  - crates/hone-channels/src/mcp_bridge.rs
  - crates/hone-channels/src/core/bot_core.rs
  - skills/earnings-research/SKILL.md
  - skills/earnings-research/scripts/render_report_pdf.py
  - packages/app/src/pages/chat.tsx
- related_docs:
  - docs/current-plan.md
  - docs/handoffs/2026-08-04-earnings-research-chat-entry.md
  - docs/decisions.md

## Goal

使用当前工作区源码和真实管理员会话完成一次 SNDK 财报前瞻：由服务器强制加载原生 `earnings-research` skill，执行真实证据研究，返回 Workflow 格式正文与 PDF 附件，并从用户对话通过鉴权文件代理下载后逐页验证。

## Scope

- 确认正在运行的旧后端、当前源码临时运行时和管理员会话边界。
- 复现并定位上一轮外部搜索预检中断；修复必须是通用流程问题，不得为 SNDK 硬编码。
- 从真实 HONE 对话入口启动财报前瞻，不接受静态 fixture 或路由 mock 作为最终证明。
- 下载真实回复附件，检查 PDF 文件头、元数据、文本结构、水印、新闻密度、分享页与逐页渲染。

## Verification

- 聚焦 Rust/Web/Skill 回归覆盖根因与下载链路。
- 当前源码后端 `/api/meta`、管理员鉴权、SSE 终态和历史投影一致。
- 浏览器真实点击生成 PDF 卡片并触发下载；下载产物由 `pdfinfo`、文本提取和全页 PNG 检查验证。

## Result

- 当前源码隔离后端以真实数据库管理员会话从“财报前瞻”按钮完成 SNDK 原生 Codex turn；最终调用 `hone/skill_tool earnings-research`，生成并在同一助手消息附上 `sndk-fy2026-q4-earnings-preview-5882eda4.pdf`。
- 修复 slash skill 指令被实体发现和重执行策略当成用户输入的问题；数据库核验管理员现在拥有原生 prompt ownership，预检搜索查询被限制在供应商长度上限内。
- Native Codex 保留宿主 `skill_tool` 作为受控脚本执行边界；MCP 子进程获得从配置文件位置解析的绝对 `HONE_SKILLS_DIR`。PDF 不再从 actor 沙箱直接启动 Chrome，也不允许模型自造替代渲染器。
- 最终 PDF 为 A4、5 页、620338 bytes；全部页面可搜索且包含精确水印 `知识星球：巴芒科技`，近期新闻跨第 3–4 页，知识星球分享图为第 5 页。真实聊天卡片可点击下载。
- 验证通过：`hone-channels` 746 passed / 1 ignored（新增测试后聚焦回归另行通过）、`hone-web-api` earnings workflow 2/2、Web 80/80、PDF CI 与手工 Chromium 回归、当前源码生产构建、全页 PNG 视觉检查。
- `git diff --check` 曾在并行任务写入 `docs/current-plan.md` 与 `packages/app/playwright.config.ts` 的瞬时中间态看到冲突标记；文件随后恢复为无冲突内容。本任务未覆盖并行任务的改动。

## Documentation Sync

- 完成后追加现有 earnings handoff、更新 `docs/archive/index.md`，并归档本计划。
- 若改变长期运行或研究工作流约束，同步 `docs/decisions.md`、`docs/invariants.md` 或运行手册；否则明确说明无需更新。

## Risks

- 当前工作区包含大量用户未提交修改，不能用 revision-bound deploy 覆盖现有 8088；续验收使用隔离端口和复制的非生产会话数据。
- 真实研究依赖模型、搜索和浏览器；网络失败必须与业务逻辑失败区分，并保留可复现日志。
