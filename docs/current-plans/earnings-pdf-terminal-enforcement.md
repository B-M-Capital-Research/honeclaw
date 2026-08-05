# Earnings PDF 终态强制与生产修复

- title: Earnings PDF 终态强制与生产修复
- status: in_progress
- created_at: 2026-08-05
- updated_at: 2026-08-05
- owner: Codex
- related_files:
  - `skills/earnings-research/SKILL.md`
  - `skills/earnings-research/scripts/render_report_pdf.py`
  - `crates/hone-tools/src/skill_tool.rs`
  - `crates/hone-channels/src/agent_session/`
  - `crates/hone-web-api/src/routes/`
  - `tests/regression/ci/test_earnings_research_pdf_markdown.sh`
- related_docs:
  - `docs/current-plan.md`
  - `docs/invariants.md`
  - `docs/decisions.md`
  - `docs/handoffs/2026-08-05-earnings-pdf-terminal-enforcement.md`

## Goal

定位并修复生产 AAOI 财报前瞻在 renderer 校验失败后，把“PDF 渲染暂时遇到格式和证据链完整性限制，无法成功生成”发布为成功终稿的问题。财报入口只有在官方 renderer 成功、生成 PDF 被宿主持久化并投影为可下载附件后才能完成；可修正的证据/格式错误必须留在同一工具循环中继续修复，不得文字降级。

## Scope

- 从生产日志和会话记录确认失败发生在哪个 renderer 校验、工具调用或终态分类边界。
- 复现文字降级路径，确定 Skill 指令、工具副作用状态、runner 终态或 Web 投影中的实际缺口。
- 用最小通用修复强制 PDF 成功终态，保留超时、不可恢复基础设施错误和不确定副作用的 fail-closed 边界。
- 增加至少一条自动化回归覆盖本次真实失败语句和等价无附件终稿。
- 部署精确 revision/技能，真实重跑 AAOI，检查 PDF 下载、页面刷新持久化和生产健康。

## Verification

- `skill-creator` `quick_validate.py skills/earnings-research`。
- 相关 Rust 单元/集成测试和 `tests/regression/ci/test_earnings_research_pdf_markdown.sh`。
- `bash tests/regression/run_ci.sh`；如修改 Rust，再运行仓库约定的 changed rustfmt、workspace check/test。
- 生产 AAOI：正文完成、一个 PDF 卡片、点击下载成功、刷新后仍存在；日志无文字降级成功终态。

## Documentation Sync

- 根据根因更新 `docs/invariants.md`、`docs/decisions.md` 或 `docs/repo-map.md`。
- 完成后更新 handoff、将本计划归档到 `docs/archive/plans/`、更新 `docs/archive/index.md` 并从 `docs/current-plan.md` 移除。

## Risks

- Gemini 严格修稿可能需要多轮和较长推理；不能通过放宽证据门禁换取表面成功。
- 已启动 renderer 的失败仍可能有不确定副作用；不得自动重放可能产生重复文件的调用。
- 生产重启必须在活跃会话为零时进行，避免中断其他用户任务。
