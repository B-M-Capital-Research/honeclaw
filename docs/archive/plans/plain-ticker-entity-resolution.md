# 普通 ticker 实体解析回归修复

- title: 普通 ticker 实体解析回归修复
- status: archived
- created_at: 2026-07-16
- updated_at: 2026-07-16
- owner: Codex
- related_files:
  - crates/hone-channels/src/investment_response_guard.rs
  - crates/hone-channels/src/scheduler.rs
  - tests/regression/ci/test_finance_automation_contracts.sh
  - tests/regression/manual/test_entity_search_live.sh
- related_docs:
  - docs/decisions.md
  - docs/invariants.md
  - docs/repo-map.md
  - docs/handoffs/2026-07-16-entity-first-investment-pipeline.md
  - docs/archive/index.md
- related_prs: N/A

## Goal

修复普通股票代码依赖辅助模型 JSON 解析、导致“今天 nbis 怎么样”以及 ticker heartbeat 在 DataFetch 前失败的回归。普通 ticker token 只作为候选，必须由 DataFetch search 返回 exact symbol 后才能成为规范实体。

## Scope

- 支持大小写普通 ticker、点号/短横线 share class 和一轮多 ticker。
- ticker 快路径与复杂公司名结构化提取分层；辅助解析失败不能阻断已确定的普通代码问法。
- `REPEAT` assignment、报告季度、普通技术缩写、行业词和无关小写单词不直接成为实体。
- 保持中文公司名、歧义候选、多标的证据和九章节回答格式契约。

## Verification

- 投研 guard 单元回归：19/19 passed，覆盖 `NBIS`、`nbis`、多 ticker、`Q3`、`REPEAT`、`AI/GPU/HBM` 与 scheduler prompt。
- `hone-channels`：494/494 passed。
- 投研 CI 契约：16/16 success。
- 真实 NBIS MCP：exact search、positive same-symbol quote、non-empty financials 全部通过。
- CLI、MCP、Web 与全部渠道 source runtime binaries 构建通过。
- 部署后临时 Web actor 端到端探针成功，5036 字符回答包含完整 `1`–`9` 编号章节且无实体/stream 错误。
- supervisor `62767` 与 backend `62779` 在线；8077/8088、管理端、用户端均为 HTTP 200，Discord/Feishu/Web 各单进程，Postgres/OSS healthy，重启后 fatal/error 扫描为 0。

## Documentation Sync

- `D-2026-07-16-01` 已补充 bare ticker 候选与 exact-symbol 边界。
- `docs/invariants.md` 与 `docs/repo-map.md` 已同步长期约束和主数据流。
- 同日实体优先 handoff 已追加本阶段记录，归档索引已新增入口。

## Risks

- 普通词与短 ticker 可能重名；语境只决定是否发起 exact lookup，DataFetch exact symbol 才能确定实体。
- scheduler 任务正文可能包含技术缩写；候选扫描限于证券语境和任务主题区，未核验候选不得生成公司特定数字。

## Outcome

状态从 `in_progress` 转为 `archived`。代码、回归、真实数据、构建、部署与健康检查均已完成，无剩余阻塞。
