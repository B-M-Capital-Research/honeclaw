# Market Move Date Grounding Handoff

- title: 涨跌归因当日新闻与研究流启动交接
- status: in_progress
- created_at: 2026-08-22
- updated_at: 2026-08-22
- owner: Codex
- related_files:
  - `crates/hone-channels/src/investment_response_guard.rs`
  - `crates/hone-tools/src/web_search.rs`
  - `agents/function_calling/src/lib.rs`
- related_docs:
  - `docs/archive/plans/market-move-date-grounding-2026-08-22.md`
  - `docs/archive/index.md`
- related_prs: none

## Summary

修复已完成并进入发布流程：涨跌归因预取从 `week/general` 改为 Tavily `day/news`，从 provider 结果读取 `published_date`；带服务端涨跌日期锚点的请求从第一轮进入现有投资研究证据流，第一轮为 Required 工具轮，不能再由 Web-only 普通路径直接作答。

## What Changed

- WebSearch 新增可选 `topic=general|news|finance`，原样保留 news 结果的 `published_date`，并提示查询日期不等于文章发布日期。
- 两路涨跌归因预取（用户原话检索和身份锚定检索）都使用 `time_range=day, topic=news`；普通投研预取仍保持 `week`。
- FunctionCalling Agent 在发现现有涨跌归因日期锚点时立即激活 Agent-owned finance loop；非涨跌问题仍由原 DataFetch 边界激活。
- 宽市场旧回归契约同步为首轮 `Required`、证据满足后 `Auto`。

## Verification

- `hone-agent`：152/152。
- `investment_response_guard`：136/136。
- WebSearch：19/19。
- `cargo check -p hone-tools -p hone-agent -p hone-channels`：通过。
- `hone-tools` 全包的 26 个失败均因本机无可用 PostgreSQL 测试服务；同包其余 168 项通过，WebSearch 子集独立全绿。完整 PostgreSQL-backed 门禁由 GitHub CI 承担。

## Deployment Evidence

- 待填写：实现 commit、GitHub CI、不可变 GHCR digest、GCE revision、`/api/meta`、渠道 worker 与真实 MRVL canary。

## Risks / Follow-ups

- 发布过程中不修改生产数据库，不打正式版本 tag。
- 上线后用真实 `mrvl下跌原因是啥呢` canary 核对：搜索参数为 day/news、每条新闻读取 `published_date`（若 provider 提供）、首轮 Required、存在 DataFetch 当前轮证据，且不再把较早文章归到目标日期。

## Next Entry Point

从本 handoff 的 Deployment Evidence 和上述三个实现文件进入；失败时按 `docs/runbooks/backend-deployment.md` 回滚到切换前 release。
