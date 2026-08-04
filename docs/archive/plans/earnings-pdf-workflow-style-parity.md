# Earnings PDF Workflow Style Parity

- title: 财报 PDF Workflow 视觉对齐与对话下载验收
- status: done
- created_at: 2026-08-04
- updated_at: 2026-08-04
- owner: Codex
- related_files:
  - skills/earnings-research/scripts/render_report_pdf.py
  - packages/app/src/pages/chat.tsx
  - tests/regression/ci/test_earnings_research_pdf_markdown.sh
- related_docs:
  - docs/current-plan.md
  - docs/decisions.md
  - docs/handoffs/2026-08-04-earnings-research-chat-entry.md

## Goal

以用户提供的 SNDK Workflow PDF 为视觉基准，使 HONE 财报 PDF 在封面、字体、字号、标题层级、正文密度、分页和新闻页上尽量一致；同时证明生成的 PDF 会作为助手附件进入用户对话，并可通过当前用户鉴权下载。

## Scope

- 渲染并逐页检查参考 PDF，记录可复用的版式特征。
- 调整原生 earnings PDF 渲染器，不改变现有 Workflow 内容契约、水印、免责声明和知识星球分享图要求。
- 使用当前管理员用户端与本地后端验证附件卡片、文件代理、下载事件和下载文件完整性。
- 输出新版 PDF 与关键对话截图。

## Verification

- `python3 -m py_compile skills/earnings-research/scripts/render_report_pdf.py` 通过。
- `bash tests/regression/ci/test_earnings_research_pdf_markdown.sh` 与 `bash tests/regression/manual/test_earnings_research_pdf.sh` 通过。
- 新版 SNDK PDF 经 Chromium 生成，`pdfinfo` 确认为 6 页 A4、可搜索文本；六页全部渲染检查通过，正文五页、新闻跨两页、分享图一页。
- `bunx playwright test e2e/public-chat-pdf-download.spec.ts --project=public` 通过：管理员对话显示 PDF 卡片，鉴权 URL 返回 `application/pdf` 与 `%PDF-` 文件头，点击触发中文文件名下载。
- `bun test packages/app/src/pages/chat.test.ts packages/app/src/lib/public-chat.test.ts` 共 54 项通过；`bun run typecheck:web` 通过。

## Documentation Sync

- 已更新 `docs/decisions.md`、现有 earnings handoff 与 `docs/archive/index.md`，并归档本计划。
- 下载仍复用既有当前用户鉴权文件代理，模块边界未改变，因此无需更新 `docs/repo-map.md` 或 `docs/invariants.md`。

## Risks

- 参考 PDF 可能包含专有字体或固定画布排版；优先匹配视觉层级和页面节奏，不通过位图化牺牲可搜索文本。
- 本地 8088 可能仍服务旧前端 bundle；验收应使用当前源码的 dev/public UI 与同一后端，而不是把旧静态包误判为当前实现。
- 实测本地 8088 当前仍是旧 revision `39ce9ce`；临时启动当前源码后端时，真实 SNDK 回合又在外部搜索预检阶段中断。因此本轮已证明 PDF 版式与聊天下载契约，但未把“当前源码真实研究生成到下载”误记为已通过，下一次原生全链路验收需要先升级运行时并稳定预检。
