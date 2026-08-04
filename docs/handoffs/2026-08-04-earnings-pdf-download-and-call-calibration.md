# Earnings PDF 下载与前瞻结论校准交接

- title: Earnings PDF 下载与前瞻结论校准交接
- status: done
- created_at: 2026-08-04
- updated_at: 2026-08-05
- owner: Codex
- related_files:
  - `crates/hone-web-api/src/routes/public.rs`
  - `crates/hone-web-api/src/routes/files.rs`
  - `packages/app/src/pages/chat.tsx`
  - `packages/app/src/lib/api.ts`
  - `skills/earnings-research/SKILL.md`
  - `skills/earnings-research/scripts/render_report_pdf.py`
  - `tests/regression/ci/test_earnings_research_pdf_markdown.sh`
- related_docs:
  - `docs/archive/plans/earnings-pdf-download-and-call-calibration.md`
  - `docs/invariants.md`
  - `docs/repo-map.md`
- related_prs: none

## Summary

修复了聊天中财报 PDF 卡片点击无反应，并把财报前瞻从易受共识锚定影响的统一“持平”模板，升级为可复算且允许公司特定自由表达的 Workflow 契约。真实 ANET、ALAB、AMD 本地流程均完成浏览器生成、内容审计和逐页视觉验收。

## What Changed

- 前端文件卡片使用登录态请求取得 PDF Blob，再触发具名下载；按钮显示下载中、已开始或可读错误，不再依赖无反馈的原生链接导航。
- 公共文件接口可把响应脱敏后的 `<absolute-path>/<filename>` 恢复为当前登录 actor sandbox 内同名生成文件；路径、根目录和 symlink 均经过 canonical containment，不能越权到其它 actor 或任意主机目录。共享文件代理同时修复 macOS `/var` 与 `/private/var` 的 canonical alias 误拒绝。
- `preview_audit` 新增指导/分部模型锚、逐项预测 bridge、收入历史偏差、容差组成和展示 scale/unit。渲染器重新计算结论，并核对 base-unit 与报告显示单位的每个数字；无法复算、复制共识、任意容差或单位错配均拒绝生成。
- 报告继续遵循旧 Workflow 的章节和新闻页结构，但不固定各段句序。开头必须在结论所在段落提供数字距离、业务驱动与置信边界；`1.2.1` 可从事实、历史或因果链切入，只要求最终结论与 audit 一致。

## Verification

- `cargo test -p hone-web-api`：184 passed，2 ignored。
- `rustfmt --edition 2024 --check crates/hone-web-api/src/routes/files.rs crates/hone-web-api/src/routes/public.rs`：通过。
- `bun run test:web`：364 passed。
- `bun run build:web:public`：通过。
- `python3 -m py_compile skills/earnings-research/scripts/render_report_pdf.py`：通过。
- `bash tests/regression/ci/test_earnings_research_pdf_markdown.sh`：beat / inline / miss 正例和错误容差、断裂 bridge、共识复制、薄弱开头、结论冲突、显示值/scale 错配等反例全部通过。
- 本地管理员真实运行 ANET 财报前瞻，执行 35 次工具调用；旧、新 PDF 卡片点击后均显示“已开始下载”。新报告开头以预测收入、共识距离、EPS、需求与交付边界解释“持平”，`1.2.1` 采用事实先行表达。
- 真实 ANET PDF 为四页 A4；逐页渲染检查无截断、重叠，近期新闻页、`知识星球：巴芒科技` 水印与知识星球分享页齐全。
- 2026-08-05 fresh actor 实跑 ANET、ALAB、AMD：三份报告分别为 4、4、5 页 A4；每份均含 8 条近期新闻，完整落在分享页前的单独一页。逐页检查无截断、重叠、乱码或水印缺失。

## Risks / Follow-ups

- “多个公司结论相同”本身不构成错误；禁止为了多样性强制 beat/miss。后续应看每家公司独立 bridge、容差和历史偏差是否支持结论。
- 首轮 ANET 视觉验收暴露模型把 USD billion 增量直接标作“亿美元”的显示单位错误；该样本促成最终 scale/unit 硬校验。最终代码会拒绝同类 PDF，部署后应重新跑一份 ANET 生成新的可交付样本。
- 全仓 `cargo fmt --all --check` 仍会报告未由本任务修改的 `crates/hone-channels/src/agent_session/artifacts.rs` 两处既有格式差异；本任务没有替用户改写该文件，两个实际改动的 Rust 文件已单独通过 rustfmt 检查。
- 下载、结论校准与新闻页密度修正均已提交并推送到 `main`；正式 release/tag 仍需单独授权。

## Next Entry Point

从 `docs/archive/plans/earnings-pdf-download-and-call-calibration.md` 与本交接继续。部署后，用 fresh actor 各跑 ANET、ALAB、AMD；先审计三份 `preview_audit`，再比较最终 call，不要把“结论不同”当验收条件。
