# Earnings PDF 下载与前瞻结论校准

- title: Earnings PDF 下载与前瞻结论校准
- status: done
- created_at: 2026-08-04
- updated_at: 2026-08-04
- owner: Codex
- related_files:
  - `crates/hone-web-api/src/routes/public.rs`
  - `crates/hone-web-api/src/routes/files.rs`
  - `packages/app/src/pages/chat.tsx`
  - `packages/app/e2e/public-chat-pdf-download.spec.ts`
  - `skills/earnings-research/SKILL.md`
  - `skills/earnings-research/scripts/render_report_pdf.py`
  - `tests/regression/ci/test_earnings_research_pdf_markdown.sh`
- related_docs:
  - `docs/current-plan.md`
  - `docs/invariants.md`
  - `docs/repo-map.md`

## Goal

修复本地 Web 会话中生成 PDF 路径被用户可见脱敏后无法下载的问题，并让财报前瞻的独立预测、容差与历史指引偏差具备可验证的数值桥，避免流程因共识锚定和任意宽中性带系统性落入“与分析师持平”。

## Scope

- 允许当前登录用户把 `<absolute-path>/<filename>` 安全解析回自己的 actor sandbox 根目录文件，不扩大到任意目录或跨用户路径。
- 给真实文件下载响应补齐正确 MIME 与附件文件名，并保留前端命名下载体验。
- 扩展 `preview_audit`：明确管理层指引/分部模型锚、逐项数值增量、历史指引偏差应用与可复算的中性带依据。
- 渲染器只校验证据和算术，不要求不同公司必须产生不同结论；相同结论必须由各自独立桥接数字支持。
- 保留旧 Workflow 的章节骨架，但放宽正文句式：开头在结论后用两到三句交代判断幅度、主变量与置信边界，后续按公司业务逻辑自由组织，禁止只剩一个结论短句或复用固定填空句型。
- 复核 ANET、ALAB、AMD 现有报告为何同时持平，并在新契约下重新跑真实样本验证。

## Validation

- Rust 单元测试：公开文件占位路径仅解析到当前 actor sandbox，拒绝跨目录与未知文件；下载响应 MIME/文件名正确。
- 前端单元/E2E：PDF 卡片可见、接口返回 `%PDF-`、点击后产生期望文件名的下载。
- Skill CI 回归：合法 beat/miss/inline 样本通过；共识复制、无法复算的预测桥、任意容差与缺失历史偏差处理被拒绝。
- 文风回归：整体分析必须在首句给结论并提供足量公司特定解释，既不回退普通问答模板，也不把各章节锁成统一句式。
- 真实本地验收：管理员入口重新运行至少一个财报前瞻，聊天卡片可直接下载；对 ANET、ALAB、AMD 结论逐一复核。
- PDF 验收：最终样本逐页渲染，无截断、重叠、缺失水印或分享页。

## Documentation Sync

- 行为约束变化时更新 `docs/invariants.md`。
- 数据流/入口变化时更新 `docs/repo-map.md`。
- 完成后新增 `docs/handoffs/2026-08-04-earnings-pdf-download-and-call-calibration.md`，把本计划移入 `docs/archive/plans/`，并更新 `docs/archive/index.md` 与 `docs/current-plan.md`。

## Risks / Open Questions

- 占位路径解析必须保持 fail-closed，不能用文件名扫描整个工作区或允许跨 actor 读取。
- 不能把“避免全是持平”实现成强制结论多样化；真实证据仍可能支持多个公司同向。
- 新 `preview_audit` 是严格契约，Skill 文案、渲染器和回归样本必须原子更新，避免运行期因旧字段失败。

## Outcome

- PDF 卡片改为带鉴权的 Blob 下载，并向用户显示下载中、已开始下载或失败原因；脱敏占位路径仅能恢复到当前 actor sandbox 内同名文件。
- 财报前瞻从“预测值贴共识 + 任意宽中性带”升级为可复算的锚、逐项 bridge、历史偏差和容差成分，且同时校验报告展示单位。
- Workflow 章节骨架继续稳定，开头和 `1.2.1` 改为公司特定、可事实先行的表达，不要求不同公司结论不同，也不要求同一套句序。
- 自动化、真实浏览器点击、真实 ANET 全流程与四页 PDF 逐页验收均已完成；ANET 新样本仍为持平，但已由独立数字桥支持，而非默认模板结论。
