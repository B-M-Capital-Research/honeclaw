- title: 明确公司投研的行情优先取证
- status: done
- created_at: 2026-08-22
- updated_at: 2026-08-22
- owner: Codex
- related_files:
  - `crates/hone-channels/src/prompt.rs`
  - `agents/function_calling/src/lib.rs`
  - `crates/hone-tools/src/data_fetch.rs`
  - `crates/hone-tools/src/web_search.rs`
  - `crates/hone-tools/src/registry.rs`
  - `docs/invariants.md`
- related_docs:
  - `docs/handoffs/2026-08-22-market-data-source-priority.md`
  - `docs/archive/index.md`

## Goal

当用户明确点名公司或证券时，把结构化行情数据置于开放网页搜索之前：先完成实体解析，再优先加载包含报价、公司身份、报价时间、涨跌口径和可得扩展时段信息的行情快照，之后才用网页搜索补充事件、公告、关系和因果证据。

## Scope

- 通过全局投研提示、工具描述、严格 function-calling 轮次提示和工具展示顺序形成一致的软优先级。
- 复用 `snapshot` / `quote` / `extended_hours` 现有能力，不新增数据源或第二套研究流程。
- 将严格 function-calling 的涨跌幅读取统一到服务端 `hone_change_basis.pct`。
- 不新增“缺行情禁止作答”、机械完整性检查、重试循环或终稿拒绝门禁；provider 无覆盖时允许基于已有证据和公开搜索自然回答并披露缺口。

## Validation

- DataFetch tests: 57 passed.
- WebSearch tests: 19 passed.
- Registry-related tests: 5 passed.
- Function-calling Agent tests: 153 passed.
- Pure channel priority-policy test: 1 passed.
- `cargo check -p hone-channels`, rustfmt and `git diff --check`: passed.
- PostgreSQL-backed broader tests were not rerun successfully because Docker/PostgreSQL are unavailable on this host; see the handoff for exact partial results.

## Documentation Sync

- Added the durable soft-priority invariant to `docs/invariants.md`.
- Added `docs/handoffs/2026-08-22-market-data-source-priority.md`.
- Removed the task from `docs/current-plan.md` and indexed it in `docs/archive/index.md`.
- No ADR added: the change implements existing source-authority and generative-workflow principles without a new architecture decision.

## Risks / Open Questions

- Observe production tool traces for model compliance and latency.
- Keep missing structured market data non-blocking; do not add a validator or retry loop.
- Re-run PostgreSQL-backed repository tests when a supported local database is available.
