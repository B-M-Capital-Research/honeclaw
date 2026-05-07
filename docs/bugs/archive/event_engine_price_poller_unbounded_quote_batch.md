# Bug: event-engine price poller sends the full watch pool in one FMP quote request

状态：`Fixed`

严重等级：`P0`

## Summary

`PricePoller` currently joins the entire watch pool into one FMP `/v3/quote/{symbols}` path. As the direct-actor portfolio pool grows, one request now contains roughly 100 symbols, including duplicates across market aliases and option-like symbols with spaces. When that oversized request times out or the tunnel rejects it, the whole `fmp.price` tick fails and task health reports the poller as unhealthy.

## Observed Symptoms

- `data/runtime/logs/web.log.2026-05-06:6105` recorded a failed quote poll at `2026-05-06 13:59:28.975 CST` with a very long `/api/v3/quote/...` URL ending in `operation timed out`.
- `data/runtime/logs/web.log.2026-05-04:7487` and `:7878` recorded the same long quote URL on `2026-05-05 03:08:18.134 CST` and `2026-05-05 07:12:36.480 CST`, both failing with tunnel transport errors.
- The failing request includes broad US/HK/CN/ETF/watch symbols plus option-style entries such as `MU 2026-06-18 C 520`, `RKLB 2026-07-17 C 140`, and `RXRX 2026-06-18 C 7/9`.
- Later `launch_web.latest` lines show `poller.fmp.price` can recover on later ticks, so this is not a permanent FMP outage, but each failed oversized request drops the full quote tick.

## Suspected Code Path

- `crates/hone-event-engine/src/subscription.rs:169-179` aggregates every direct actor's portfolio symbol into one sorted watch pool with no batching or quote-endpoint eligibility filter.
- `crates/hone-event-engine/src/pollers/price.rs:63-69` builds one URL via `symbols.join(",")` and calls `FmpClient::get_json` once.
- `crates/hone-event-engine/src/engine.rs:599-615` spawns `PricePoller` on the fixed price interval, so this path runs repeatedly as the pool grows.

## Why This Looks Like a Bug

FMP does support multi-symbol quote requests, but Hone currently treats the watch pool as an unbounded single request. That makes one bad symbol, one too-large URL, or one upstream/tunnel timeout fail the entire poller tick. A healthier implementation should either batch the pool into smaller chunks, filter symbols that cannot be quoted by FMP's equity quote endpoint, or tolerate partial batch failure.

## Expected Behavior

- Price polling should split large watch pools into bounded FMP quote batches.
- One failed batch should not discard successful quote batches from the same tick.
- Unsupported option-style symbols should not inflate the equity quote URL unless FMP quote support is explicitly verified for them.

## Verification

- Reproduced from local runtime logs only; no live FMP request was sent during this triage.
- Code inspection confirms the current request construction has no batch-size or URL-length guard.
- 2026-05-06 23:15 CST：已补代码修复与回归验证；未依赖线上 FMP 请求或生产健康状态。

## Risks

- Missed price alerts / 52-week events for all watched symbols during a failed tick.
- Repeated task-health noise as the watch pool grows.
- Higher latency and timeout probability when one request asks FMP for too many or unsupported symbols at once.

## Fix Record

- 2026-05-06 23:15 CST：`PricePoller::fetch` 改为先过滤 FMP equity quote path 不支持的 symbol（空值、超长、含空格/斜杠的 option-style 条目），再按固定批量与 URL path 长度上限拆分 `/v3/quote/{symbols}` 请求。
- 同轮修复让单个 quote batch 失败只记录 warning 并保留同一 tick 中其它成功 batch 的事件；只有所有 batch 都失败时才让 poller tick 返回错误，避免一个超长/坏请求丢弃整池 quote。
- 回归验证：
  - `cargo test -p hone-event-engine quote_symbol_batches_filter_unsupported_symbols_and_split --lib -- --nocapture`
  - `cargo test -p hone-event-engine quote_batch_collection --lib -- --nocapture`
  - `cargo check -p hone-event-engine --tests`
- 关联 GitHub Issue：无。
