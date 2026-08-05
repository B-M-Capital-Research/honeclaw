# Earnings PDF 终态强制与生产修复交接

- title: Earnings PDF 终态强制与生产修复交接
- status: done
- created_at: 2026-08-05
- updated_at: 2026-08-05
- owner: Codex
- related_files:
  - `crates/hone-channels/src/agent_session/`
  - `crates/hone-channels/src/runners/opencode_acp.rs`
  - `crates/hone-channels/src/tool_trace.rs`
  - `crates/hone-tools/src/skill_tool.rs`
  - `skills/earnings-research/scripts/render_report_pdf.py`
- related_docs:
  - `docs/archive/plans/earnings-pdf-terminal-enforcement.md`
  - `docs/invariants.md`
  - `docs/decisions.md`
  - `docs/repo-map.md`
- related_prs: direct `main` commits `c70be6c0`, `f24d0f76`, `bb002a43`, `82c835b7`, `9935d383`, `f5a384b2`; no PR, release, or tag

## Summary

生产 AAOI 财报前瞻不再把 renderer 校验失败说明当作成功终稿。财报入口现在只有在官方 renderer 返回经过校验的 Markdown 与 PDF、宿主收集并持久化附件后才完成；真实 AAOI 已在聊天生成、下载并通过刷新持久化验收。

## What Changed

- renderer 普通预检问题一次完整返回，避免八条一批制造无效返工；成功结果携带可信 `validated_report_markdown`，宿主直接投影该正文与附件。
- 财报响应在 PDF 终态前保持缓冲，模型的失败解释、无附件正文或普通 ACP `end_turn` 均不能发布为成功。
- 精确 `Corrupted thought signature` 与明确写入前 renderer 拒绝各允许一次隔离的新 OpenCode 会话恢复；未知工具、artifact 或不确定副作用继续 fail closed。
- OpenCode ACP `1.18.13` 的 `rawOutput.output` JSON 字符串在 runner 边界解码。该缺口曾同时遮蔽安全重试字段和成功 PDF artifact，是最后一次线上泛化失败的直接根因。

## Verification

- 完整 workspace check/test、changed rustfmt、CI-safe regression 与 earnings renderer regression 均通过。
- GitHub Runtime Image run `31013111954` 发布 exact revision `f5a384b2932b6602840968bc8c0a910f154008ee`，immutable digest `sha256:d7a11aef6b4b968bd172692ddfd5a29e4cfcd2a0d0f262f10afce499fcfab4ff`；生产 `/api/meta`、技能、云存储与服务健康通过。
- 真实消息 `12fb473d-c2c3-4db7-ba40-e6b3a756e2f1` 成功生成 `AAOI-preview-fdb23cd7.pdf`。文件 4 页 A4、595,130 bytes、SHA-256 `2abfee7d1ee62b238cefa02b3287aec712d59f076d57d9c6ca4d9b11ae6935be`；浏览器下载成功，刷新后附件仍存在，逐页检查水印、新闻页、分享图与排版无缺陷。

## Risks / Follow-ups

- 严格校验可能带来数分钟延迟与较高模型成本；本次为约 380 秒和 USD 0.87。应基于真实 renderer 错误统计提高首轮命中率，不得把校验降级为警告。
- 生产磁盘在暂存新 runtime 后约 2.6 GiB 可用。后续部署应继续保留当前、前一版和一个已知良好回滚，仅按 runbook 显式清理已解析且非 current 的旧 runtime。

## Next Entry Point

若再次出现财报 PDF 失败，先按消息 ID 查看同轮 `ToolCallMade` 的已解码 renderer 结果，再区分内容预检、基础设施失败和不确定副作用；不要从用户可见兜底文案反推根因。OpenCode 升级时必须用真实 `rawOutput` envelope 重跑 runner contract。
