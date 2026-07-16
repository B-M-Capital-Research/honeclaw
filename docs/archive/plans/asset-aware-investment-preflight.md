# 资产类型感知的投研预检修复

- title: 资产类型感知的投研预检修复
- status: archived
- created_at: 2026-07-16
- updated_at: 2026-07-16
- owner: Codex
- related_files:
  - crates/hone-channels/src/investment_response_guard.rs
  - crates/hone-channels/src/agent_session/core.rs
  - crates/hone-channels/src/agent_session/tests.rs
  - crates/hone-channels/src/prompt.rs
  - crates/hone-tools/src/data_fetch.rs
  - soul.md
  - skills/stock_research/SKILL.md
  - tests/regression/ci/test_finance_automation_contracts.sh
  - tests/regression/manual/test_entity_search_live.sh
- related_docs:
  - docs/decisions.md
  - docs/invariants.md
  - docs/repo-map.md
  - docs/handoffs/2026-07-16-entity-first-investment-pipeline.md
- related_prs: N/A

## Goal

修复 `INTL / Main International ETF` 被公司利润表门禁误判为“财务数据不稳定”的线上回归，并把实体优先流程扩展为公司、ETF/基金、加密资产的结构化证据路由。FMP 对 ETF income statement 和加密资产 stock profile 返回成功空数组属于资产类型语义，不是 provider 故障。

## Completed Scope

- 保留 DataFetch exact-symbol search 和 positive same-symbol quote 门禁；普通 ticker 不依赖辅助模型严格 JSON。
- 非 crypto 证券先核验 exact-symbol profile；结构化 `isEtf/isFund` 决定基金路线，明确的公司 profile 决定股票路线，未知类型保持 fail-closed。
- 公司深度路线使用 profile、meaningful financials、news；ETF/基金使用 profile、holdings、news；crypto 从 exact search 的 `CRYPTO/CCC` 市场字段确认，使用 quote、news，不要求 stock profile。
- ETF/基金和 crypto 禁止调用公司 financials / earnings calendar；crypto 额外禁止 ETF holdings。初稿的禁用调用跨 reset/retry 累积，不能由干净重试洗掉。
- 公司、基金、crypto 分别使用适配的九章节回答；多标的按 symbol 独立路由。章节必须有实质内容、数据时间、事实/推断、正确现价和动作触发条件。
- 当前价格在全文任一处与本轮 quote 冲突都会失败；支持常见中英文价格表达、币种和 provider 日期，同时排除历史年份、均线周期等非当前价数字。
- 投资门禁先对 raw runner output 使用统一的 `sanitize_user_visible_output`，再校验与 SSE / 最终发送一致的 canonical visible content，避免 `<think>` 内部编号污染正式章节解析。
- DataFetch 不缓存 search、quote、profile、financials、holdings 的 semantic-empty payload；只有认证、配额和限流错误轮换 FMP key，transport / parse / server / ordinary provider 错误停止而不跨 key 放大。

## Verification

- `cargo test -p hone-channels investment_response_guard --lib --no-fail-fast`：32/32 passed。
- `cargo test -p hone-channels --no-fail-fast -- --test-threads=1`：509/509 passed；doc tests 0 failed、2 ignored。
- `cargo test -p hone-tools data_fetch -- --nocapture`：24/24 passed。
- `bash tests/regression/ci/test_finance_automation_contracts.sh`：17/17 success。
- `bash tests/regression/manual/test_entity_search_live.sh`：NBIS exact company + positive quote + 4 期财务；INTL exact ETF + 30.495 quote + `isEtf=true` + 9 项 holdings + successful empty financials；BTCUSD exact crypto + positive quote + successful empty stock profile。
- 全量 source runtime binaries（CLI、MCP、Web、iMessage、Discord、Feishu、Telegram）构建通过；`cargo fmt --all` 与 `git diff --check` 通过。
- 部署后 Web E2E `现在intl怎么看`：`deep_analysis=Fund`，HTTP 200，`success=true`，一次生成无 reset；可见正文 3474 字符，1–9 节齐全，含 `数据时间：北京时间 2026-07-16`、现价 `30.495`、基金目标和持仓证据，无“无法稳定核验/证券实体识别不完整/投研完整性检查”错误。
- 服务完整重启到 supervisor `63086`、backend `63098`；8077/8088 正常监听，Web/Discord/Feishu 各单进程在线，Postgres/OSS healthy，最终启动后 panic/fatal/ERROR 扫描为空。

## Documentation Sync

- 扩展 `D-2026-07-16-01` 的资产类型、provider empty/error 和跨 retry 审计决策。
- 更新 `docs/invariants.md` 与 `docs/repo-map.md` 的证据路由、缓存和 visible-content 校验边界。
- 同日实体优先 handoff 追加本阶段；本计划退出活跃索引并归档到此路径，历史入口写入 `docs/archive/index.md`。

## Risks

- profile 缺失时不能因 financials 为空擅自推断 ETF；未知资产类型继续保守失败。
- FMP 的 symbol news 可能混入正文提到相同字符串但实体不同的新闻；生成侧只能将精确实体匹配的内容作为证券新闻事实，后续可增加 news-level entity filter。
- canonical visible content 目前在 AgentSession 门禁入口生成；长期可把 visible content 作为 runner result 的显式字段，raw content 只进入审计和内部上下文。
