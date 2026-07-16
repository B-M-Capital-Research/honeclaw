# 投研实体优先执行管线改造交接

- title: 投研实体优先执行管线改造交接
- status: done
- created_at: 2026-07-16
- updated_at: 2026-07-16
- owner: Codex
- related_files:
  - crates/hone-channels/src/investment_response_guard.rs
  - crates/hone-channels/src/agent_session/core.rs
  - crates/hone-channels/src/scheduler.rs
  - crates/hone-tools/src/data_fetch.rs
  - tests/regression/ci/test_finance_automation_contracts.sh
  - tests/regression/manual/test_entity_search_live.sh
- related_docs:
  - docs/archive/plans/entity-first-investment-pipeline.md
  - docs/decisions.md
  - docs/invariants.md
  - docs/repo-map.md
- related_prs: N/A

## Summary

投研链路已从“正则猜 ticker + 单股特例”改为实体优先的统一执行管线。当前请求先结构化提取全部命名证券，再逐一使用 DataFetch search 解析规范实体并核验同 symbol 行情，之后才允许生成公司特定数字或进入单股/比较回答契约。

## What Changed

- 移除 `REPEAT`/`AI` 一类缩写黑名单、`Nebius → NBIS` 硬编码、搜索首条候选和最终阶段重猜 ticker。
- 显式 ticker 只接受 `$TICKER` 确定性输入；其他实体识别失败时澄清并失败关闭，不以大写正则降级猜测。
- 中文名/别名经过结构化提取后使用 DataFetch search；候选按名称、symbol、交易所评分，相同分数的 share class 保持歧义。
- 引入 typed `AgentTurnOrigin`，scheduler/heartbeat 使用原始任务正文做实体输入，delivery envelope 不再参与。
- 实体、证据和回答契约每轮只准备一次，context overflow 与 heartbeat budget recovery 复用同一份 runtime suffix。
- 单股深度保留九章节强制格式；多标的增加所有 symbol、数据时间和风险/证伪的最终校验。

## Verification

- `hone-channels`：488/488 passed。
- `hone-tools`：123 passed，0 failed，1 ignored。
- 全量运行时二进制构建（CLI、MCP、Web、iMessage、Discord、Feishu、Telegram）通过；所有 `AgentRunOptions` 字面量已使用默认补全，typed origin 在 scheduler 内覆盖。
- 投研 CI 契约：16/16 success。
- 真实 NBIS MCP 链路：search 精确返回 `NBIS / Nebius Group N.V.`，quote 返回正数价格；FMP 与 Tavily 探针均 healthy。
- rustfmt changed-files 和 `git diff --check` 通过。
- 服务已完整重启到 supervisor `84863`：8077/8088 正常监听，Web/Discord/Feishu 单进程在线，Postgres/OSS healthy，启动后错误扫描为空。

## Risks / Follow-ups

- provider 搜索覆盖不足时会向用户澄清，这是故意的正确性优先行为。
- 大型 scheduler 观察列表的 search fan-out 可继续做缓存/批处理优化，但不能跳过每轮同 symbol 行情核验。
- 若引入新的证券数据源，应实现相同的候选字段和歧义语义，不要恢复公司名硬编码。

## Next Entry Point

先读 `D-2026-07-16-01` 和 `crates/hone-channels/src/investment_response_guard.rs`；真实 provider 回归从 `tests/regression/manual/test_entity_search_live.sh` 进入。
