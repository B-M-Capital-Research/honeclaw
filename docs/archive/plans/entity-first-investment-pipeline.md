# 投研实体优先执行管线改造

- title: 投研实体优先执行管线改造
- status: archived
- created_at: 2026-07-16
- updated_at: 2026-07-16
- owner: Codex
- related_files:
  - crates/hone-tools/src/data_fetch.rs
  - crates/hone-channels/src/investment_response_guard.rs
  - crates/hone-channels/src/agent_session/
  - crates/hone-channels/src/scheduler.rs
  - crates/hone-channels/src/prompt.rs
  - skills/stock_research/SKILL.md
  - skills/portfolio_management/SKILL.md
  - tests/regression/ci/test_finance_automation_contracts.sh
  - tests/regression/manual/test_entity_search_live.sh
- related_docs:
  - docs/decisions.md
  - docs/invariants.md
  - docs/repo-map.md
  - docs/handoffs/2026-07-16-entity-first-investment-pipeline.md

## Goal

把投研请求从“正则猜 ticker 后做精确匹配”重构为固定的实体优先管线：先从当前请求提取公司/证券实体，通过 DataFetch 确认规范实体，再基于同一份结构化实体结果规划证据、回答格式和最终校验。多标的、定时任务和心跳任务不得通过文本前缀绕过实体解析。

## Completed Scope

- 建立结构化 `AgentTurnOrigin`、实体提及、规范证券实体和 prepared investment contract。
- DataFetch 正式暴露 `search/query` 契约并对查询安全编码。
- AgentSession 只准备一次实体/回答契约和证据，执行、context recovery 与 response retry 复用同一结果。
- scheduler 将任务正文和 heartbeat 类型结构化传入；`REPEAT` 等 envelope metadata 不参与实体识别。
- 单标的、多标的、中文名/别名、歧义、宏观/行业无实体和失败关闭均有回归覆盖。
- 单股深度回答继续执行九章节格式；多标的回答新增逐标的、时间与风险/证伪校验。

## Verification

- `cargo test -p hone-channels --no-fail-fast`：488 passed，0 failed。
- `cargo test -p hone-tools --no-fail-fast`：123 passed，0 failed，1 ignored。
- `cargo build -p hone-cli -p hone-mcp -p hone-console-page -p hone-imessage -p hone-discord -p hone-feishu -p hone-telegram`：通过。
- `bash tests/regression/ci/test_finance_automation_contracts.sh`：16 success，0 review，0 fail。
- `bash scripts/ci/check_fmt_changed.sh` 与 `git diff --check`：通过。
- `bash scripts/diagnose_fmp_tavily.sh --fmp-symbol NBIS ...`：FMP 1/1、Tavily 1/1 healthy。
- `tests/regression/manual/test_entity_search_live.sh`：真实搜索精确返回 `NBIS / Nebius Group N.V.`，同标的 quote 为正。
- 重启健康检查：新 supervisor `84863`；8077/8088 监听；Web、Discord、Feishu 均为单进程 `running`；Postgres 与 OSS health 均为 `ok=true`；启动后无 error/panic。

## Documentation Sync

- `D-2026-07-16-01` supersede 旧单标的范围和文本启发式架构。
- `docs/invariants.md` 与 `docs/repo-map.md` 已同步实体优先主数据流。
- 本计划已移出活跃索引并归档；交接与历史索引已补齐。

## Risks / Follow-ups

- 数据提供方对别名覆盖不一致时保持歧义或失败关闭，禁止为了提高召回率恢复“首条即真”或大写词猜 ticker。
- 超大 scheduler 观察池仍可能产生较多 search 请求；后续性能优化必须复用已确认实体缓存，同时每轮行情仍需重新核验。
- 新增资产类别或非 FMP 市场时，应扩展实体 provider/候选字段，不应在 guard 中加硬编码公司别名。
