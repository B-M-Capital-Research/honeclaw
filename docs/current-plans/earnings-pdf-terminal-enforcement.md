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
  - `crates/hone-channels/src/runners/opencode_acp.rs`
  - `crates/hone-channels/src/tool_trace.rs`
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
- renderer 成功后由宿主直接发布同次成功调用中已校验的 Markdown 与附件路径，避免模型为补附件名再次生成整篇报告并耗尽时限。
- 对 OpenRouter/Gemini 在工具轮次间偶发的精确 `Corrupted thought signature` 做一次受限的新 OpenCode 会话恢复；当前轮仍保持历史隔离，且仅允许已知只读、无持久副作用、无可见前缀的任务重放。
- renderer 每轮一次返回普通报告的全部预检问题（仍有 32 条硬上限），避免 8 条分批反馈制造不必要的六轮返工和上下文压缩；若 OpenCode 在已确认 `side_effect_status=not_started`、无 artifact、其余调用全为已知只读的校验失败后结束，则放弃已耗尽会话并只做一次全新隔离当前轮重试。
- 增加至少一条自动化回归覆盖本次真实失败语句和等价无附件终稿。
- 部署精确 revision/技能，真实重跑 AAOI，检查 PDF 下载、页面刷新持久化和生产健康。

## Verification

- `skill-creator` `quick_validate.py skills/earnings-research`。
- 相关 Rust 单元/集成测试和 `tests/regression/ci/test_earnings_research_pdf_markdown.sh`。
- `bash tests/regression/run_ci.sh`；如修改 Rust，再运行仓库约定的 changed rustfmt、workspace check/test。
- 生产 AAOI：正文完成、一个 PDF 卡片、点击下载成功、刷新后仍存在；日志无文字降级成功终态。
- 生产故障复现确认：`ses_02df86fc1ffeMrWksTwhlo0D5e` 在首次只读 DataFetch 后由 Gemini 返回精确 `400 invalid_request: Corrupted thought signature`；回归必须证明仅该结构化错误触发一次全新隔离会话，且不压缩/带回旧聊天。
- 第二层生产复现确认：`ses_02de7db74ffe5VdG8XHEnt8AsO` 已跨过签名错误并完成数据/Web 取证，但 renderer 因每次只显示 8 条而连续六轮从 20→12→6→单项→单项→新闻数量返工；同会话三个 continuation 均为 0-token 空响应。回归必须证明普通错误一次完整返回、超大集合仍按 32 条截断，并且只有明确写入前拒绝的 renderer trace 可以触发一次新会话。
- 第三层生产复现确认：OpenCode ACP `1.18.13` 把成功传输的 MCP `rawOutput.output` 作为 JSON 字符串返回；若 runner 不解码，宿主只能看到 `Value::String`，会同时丢失 renderer 的成功 artifact 和安全失败字段。回归必须用该真实 envelope 证明字符串被解析成结构化结果后才进入 PDF 成功/重试判定。

## Documentation Sync

- 根据根因更新 `docs/invariants.md`、`docs/decisions.md` 或 `docs/repo-map.md`。
- 完成后更新 handoff、将本计划归档到 `docs/archive/plans/`、更新 `docs/archive/index.md` 并从 `docs/current-plan.md` 移除。

## Risks

- Gemini 严格修稿可能需要多轮和较长推理；不能通过放宽证据门禁换取表面成功。
- 已启动 renderer 的失败仍可能有不确定副作用；不得自动重放可能产生重复文件的调用。
- fresh-session 校验重试会重新取证并增加耗时，但不能携带模型上一会话的草稿或把未知工具当成只读；任何 artifact、未知状态或未知工具都必须阻断重试。
- 生产重启必须在活跃会话为零时进行，避免中断其他用户任务。
