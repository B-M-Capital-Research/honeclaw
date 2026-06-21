# FMP/Tavily Usage Throttle

- title: FMP/Tavily Usage Throttle
- status: done
- created_at: 2026-06-21
- updated_at: 2026-06-21
- owner: Codex
- related_files:
  - `crates/hone-tools/src/web_search.rs`
  - `crates/hone-tools/src/data_fetch.rs`
  - `crates/hone-event-engine/src/pollers/price.rs`
  - `agents/function_calling/src/lib.rs`
  - `crates/hone-channels/src/scheduler.rs`
  - `config.example.yaml`
- related_docs:
  - `docs/archive/index.md`
- related_prs:

## Summary

本次落地 FMP/Tavily 用量治理：减少 Tavily 失败 key 放大、降低 Tavily 响应体和 credit 风险，为 FMP data_fetch 加短 TTL 缓存，并把 FMP price poller 限制在美股常规交易窗口内运行。目标是保留关键盘中事件监控，同时避免非交易时段和重复 heartbeat 持续消耗外部 API。

## What Changed

- Tavily `web_search` 改为 Bearer auth，低带宽请求体固定 `search_depth=basic`、`max_results<=3`、关闭 answer/raw/images，并开启 `include_usage` 记录 credit。
- Tavily key 池增加短期熔断：`401/403` 冷却 24 小时，`429/432` 冷却 6 小时；单次搜索不再遍历所有坏 key，全部熔断时直接结构化返回 unavailable。
- FMP `data_fetch` 增加基于脱敏 URL 的 TTL 缓存；`snapshot` 复用 quote/profile/news 子请求缓存；新增可选 `quote_short` 类型给非事件引擎轻量调用使用。
- FMP price poller 只在美东周一至周五 09:30-16:05 运行；当前实现未内置美股假日/半日历。
- Heartbeat function-calling runner 增加工具预算：单轮最多 3 次工具调用，其中 `web_search<=1`、`data_fetch<=2`；prompt 同步要求优先本地事件、组合、文件和 FMP 缓存。
- 默认配置降频：`price_secs=900`、`news_secs=1800`、`global_digest.fetch_full_text=false`、`search.max_results=3`。

## Verification

- `cargo test -p hone-tools --lib`
- `cargo test -p hone-agent --lib`
- `cargo test -p hone-event-engine pollers::price --lib`
- `cargo test -p hone-channels heartbeat_tool --lib`
- `cargo test -p hone-channels heartbeat_prompt_keeps_legacy_empty_json_example_literal --lib`
- `cargo test -p hone-channels execution::tests --lib`
- `cargo check --workspace --all-targets --exclude hone-desktop`
- `bash scripts/diagnose_fmp_tavily.sh --tavily-query 'health check'`

## Risks / Follow-ups

- Price poller market-window guard does not model NYSE/Nasdaq holidays or half-days;如果后续需要更精确，需要接入交易日历。
- Runtime `config.yaml` 是本机忽略文件；部署前应确认生产进程实际读取的配置已经包含降频项。
- Heartbeat 合并/降优先级属于调度治理事项，本次只加工具预算和 prompt 约束，没有迁移既有任务定义。
- 发布后需要通过 `/api/meta`、`/api/channels`、`/api/public/auth/me` 和日志确认新服务已启动且通道健康。

## Next Entry Point

如后续继续压 FMP 用量，优先看 `crates/hone-event-engine/src/pollers/price.rs` 的 batch quote 频率和交易日历；如继续压 Tavily，用 `crates/hone-tools/src/web_search.rs` 的 usage 日志核对 credit 与逻辑调用比例。
