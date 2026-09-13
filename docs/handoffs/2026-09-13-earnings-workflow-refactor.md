# 财报工作流重构与 TEM 验收

- title: 财报工作流重构与 TEM 验收
- status: in_progress
- created_at: 2026-09-13
- updated_at: 2026-09-13
- owner: Codex
- related_files: `skills/earnings-research/`; `crates/hone-channels/src/earnings_materials.rs`; `crates/hone-channels/src/runners/opencode_acp.rs`; `crates/hone-tools/src/public_page.rs`; `crates/hone-tools/src/web_search.rs`; `packages/app/src/pages/chat.tsx`
- related_docs: `docs/current-plans/earnings-workflow-refactor-20260913.md`; `skills/earnings-research/references/workflow-source.md`; `docs/decisions.md#d-2026-09-13-01-company-only-earnings-workflows-and-reliable-pdf-completion`; `docs/runbooks/backend-deployment.md`

## Summary

用户要求两个弹窗只填公司、自动取得财报/电话会、遵从原 Dify Prompt、取消额外内容门禁、提高 PDF 稳定性，TEM 实测后上线。原工作目录含用户未提交改动和本地提交，本任务从 origin/main bbdc828c 建独立 worktree，未混入原目录改动。

## What Changed

- 移除弹窗材料上传，保留结构化 kind/company 请求与后端管理员/actor 边界。
- 逐节点只读对照用户给出的 Dify 与新闻子流程，恢复原 Prompt 案例、示例、措辞、查询 USER 与新闻查询 Prompt，移除后来添加的发布/单位自检；不把 Prompt 的章节/条数示例变成代码门禁。
- 自动取材先搜索发布财报及电话会候选，读取公开页面原文并注入当前轮；检索综合摘要与原文分开。缺失材料留作研究缺口，不拒绝报告。上下文与恢复保持当前轮隔离。
- 搜索默认仍为轻量摘要；可显式请求原文/更多结果或读取公开 URL。公网 DNS 固定、逐跳校验、禁止 ambient proxy、标准端口、时限/体积界限阻止私网读取；页面不执行脚本。
- 专用财报系统提示不再叠加通用投研 soul 和其他技能推荐，避免真实 canary 误入 hari-invest/company-thesis-ratings；原 Prompt 独立拥有内容与流程。
- 显式 OpenRouter 路由覆盖 OpenCode 全局 provider 过滤器，修复本地 canary `model not found`。
- PDF 使用两次 40 秒技术尝试、独立 Chromium profile、完整落盘检测和进程组回收、原子发布。修复 Chrome 完成打印后进程不退出而丢弃成品的问题；分享图片内嵌，重复表头、行分页、链接属性转义保留正文。

## Verification

- `cargo check --workspace --all-targets --exclude hone-desktop --exclude hone-user-app` 通过。
- 定向 Rust：channels 879、tools 235、web-api 378 项覆盖（tools 并行回归期间一次 PG 连接失败，单独复核结果追加于最终验收）。
- Web 单测 646 项通过；Public Community Edge Worker 类型检查及 45 项单测通过；`bun run build:web:public` 通过。
- Playwright 财报弹窗/请求与 PDF 命名下载：3 项通过。
- PDF CI 技术回归通过，包括生成后不退出、进程回收、截断产物拒收、原文件保护、临时目录清理；真实 Chrome 手工回归通过，2 页 A4。
- 全仓历史失败：hone-agent 两项 streaming mock 测试在 `agents/function_calling/src/lib.rs:8265` 触发 unreachable；hone-core `config_example_avoids_stale_config_knobs` 因 README_EN 既有 api_key/api_keys 文案断言失败；finance automation acceptance 40/50，失败项 18/22/40/42/23/24/28/29/37/43。相关实现未在本次修改。CI runner 在该项停止后，其余脚本单独逐项执行通过。
- TEM 初轮发现本地未接通生产 loopback 搜索代理、全局 provider 过滤器、摘要替代原文及 Markdown 参数被模型预转义的问题；这些轮次作为诊断，不能用 `success=true` 冒充最终内容验收。
- 最终 TEM 双模式、逐页 PDF 视觉检查、精确 revision 生产验收：执行中，完成后追加具体 run/artifact/hash/耗时。

## Risks / Follow-ups

- 公开来源可能限制访问或只提供音频；缺失原文由原研究流程披露。预取候选不证明公司/季度对应，模型需依据来源时间选择；未恢复任何内容 validator。
- 生产二进制、skills 与 public fallback 为三个独立产物；需要精确 SHA、文件模式及分享图片检查，不能只重启二进制。
- 真实 PDF 和下载截图保留在本地 `honeclaw-artifacts/earnings-20260913/`；最终产物哈希与下载结果会在此留存。诊断轮不能作为投资报告交付。

## Next Entry Point

按 active plan 完成最终双模式 canary、构建/部署、线上回读和归档。
