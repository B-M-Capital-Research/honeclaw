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
- 显式 ticker 最初只接受 `$TICKER` 确定性输入；此限制已被下方同日 bare-ticker 回归阶段替代，`$TICKER` 仍是最高置信度输入，但不再是普通代码的唯一入口。
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

## 2026-07-16 普通 ticker 回归修复阶段

### Root Cause

`7a18f552` 将 `$TICKER` 设为唯一确定性代码输入，普通 `NBIS/nbis` 全部依赖辅助模型返回严格 JSON。辅助输出包含 reasoning 或格式偏差时，请求会在 DataFetch 前直接返回“证券实体识别结果不完整”；同一缺陷也让多条 ticker heartbeat 周期性失败。

### Follow-up Changes

- 普通大写 ticker 与证券语境中的小写 ticker 先形成词法候选；`今天nbis怎么样`、`NBIS最近怎么样` 和多 ticker 比较不再等待辅助 JSON。
- 候选不是实体真相：每个代码仍必须由本轮 DataFetch search 返回 exact symbol，之后才允许查同 symbol quote、financials 或生成结论；不接受搜索首条猜测，也没有公司别名硬编码。
- assignment key/value、`Q1`–`Q4`、行业/指标缩写和无关小写词不进入快路径。复杂公司名仍走结构化提取；结构化模型明确返回空数组时，不会重新塞回未经确认的大写 token。
- 辅助模型响应可包含 reasoning 或多个 JSON 对象，解析器选择最后一个完整 `entities` 对象；普通 ticker 已确定时，辅助解析失败不再阻断。
- “怎么样”纳入单股深度意图，因此最新 NBIS 问法继续执行九章节回答契约，而不是退化为草率行情短答。

### Follow-up Verification

- `cargo test -p hone-channels investment_response_guard --lib --no-fail-fast`：19/19 passed。
- `cargo test -p hone-channels --no-fail-fast`：494/494 passed。
- 投研 CI 契约：16/16 success。
- 真实 MCP：NBIS search 精确返回 `Nebius Group N.V.`，同代码 quote 为正，financials 返回 4 组非空数据。
- 全量 source runtime binaries（CLI、MCP、Web、iMessage、Discord、Feishu、Telegram）构建通过。
- 部署后临时 Web actor 端到端输入“今天nbis怎么样”：成功完成，正文 5036 字符，`1`–`9` 九个编号章节齐全，无实体错误或 stream error。
- 服务重启到 supervisor `62767`、backend `62779`；8077/8088、管理端/用户端均为 HTTP 200，Discord/Feishu/Web 各单进程在线，Postgres/OSS healthy，重启后 fatal/error 扫描为 0。

### Follow-up Risk

普通词和短 ticker 天然可能重名，因此词法层只决定“是否值得 exact lookup”，不决定证券身份。后续若扩展语境规则，应继续增加正反回归，不得恢复静态公司映射、首结果命中或把辅助模型当唯一 ticker 入口。
