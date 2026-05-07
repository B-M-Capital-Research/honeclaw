# Bug: SEC filing enrichment 复用全局 OpenRouter max_tokens 触发 `HTTP 402`

- **发现时间**: 2026-05-07 11:15 CST
- **Bug Type**: System Error
- **严重等级**: P2
- **状态**: Fixed
- **GitHub Issue**: N/A

## 证据来源

- 运行日志：
  - `data/runtime/logs/web.log.2026-05-07`
  - `2026-05-07 11:15:45-11:16:08 CST` 多次记录 OpenRouter `HTTP 402`：
    - `This request requires more credits, or fewer max_tokens`
    - `You requested up to 30000 tokens, but can only afford 26248`
  - 同一窗口紧接 `sec_enrichment LLM call failed`，说明失败链路是 SEC filing enrichment，不是前一行 `poller.fmp.analyst_grade`。
- live API smoke：
  - 当前 OpenRouter key 可认证，网络路径可用。
  - `x-ai/grok-4.1-fast` + `max_tokens=800` 成功返回 `ok`。
  - 同一模型 + `max_tokens=30000` 即使只有一句短 prompt，也返回 `HTTP 402`，提示当前 key 只能承受约 `26244` 输出 token 的单次预算。

## 根因判断

- SEC filing enrichment 的配置字段 `event_engine.sec_filings.enrichment.max_summary_tokens` 默认是 `800`，目标只是一段约 200 字中文摘要。
- 旧实现中 `LlmSecFilingSummarizer` 复用 `global_digest_provider`，而该 provider 由 `OpenRouterProvider::from_config(...)` 构建，实际使用全局 `llm.openrouter.max_tokens`。
- 当前部署全局值为 `32768`，OpenRouter 对请求做最坏情况预授权时按约 `30000` 输出 token 预算校验，超过当前 key 的单次可承受预算后返回 `HTTP 402`。
- 这不是账户余额或 API key 连通性问题，而是短摘要链路错误继承了长输出预算。

## 用户影响

- SEC filing 事件本身不会丢失；enrichment 是 best-effort，失败后 renderer 回退到原始 form/link body。
- 用户会少看到 filing 的 LLM 摘要；运行日志会持续出现 OpenRouter 402，并可能消耗调试注意力。
- 若同一 tick 有多条 filing，旧实现会对每条都发起过大的预授权请求，放大 provider 错误噪声。

## 修复记录（2026-05-07 11:26 CST）

- 状态更新为 `Fixed`。
- `EventEngine` 新增 `with_sec_filings_enrichment_provider(...)`，SEC filing enrichment 可使用独立 LLM provider。
- `hone-web-api` 在装配 event-engine 时使用 `OpenRouterProvider::from_config_with_max_tokens(...)` 为 SEC filing enrichment 单独创建 provider，completion cap 来自 `sec_filings.enrichment.max_summary_tokens`。
- `llm.openrouter.max_tokens` 与 global digest provider 行为保持不变，避免影响普通长输出路径。
- `max_summary_tokens` 被安全 clamp 到 `1..=u16::MAX`，避免无效或溢出配置。

## 验证

- 通过：`cargo test -p hone-web-api sec_filings_enrichment --lib`
- 通过：`cargo test -p hone-event-engine sec_filings_enrichment --lib`
- 通过：`cargo check -p hone-web-api`
- 通过：`rustfmt --edition 2024 --config skip_children=true --check crates/hone-web-api/src/lib.rs crates/hone-event-engine/src/engine.rs`
- 未通过但非本次改动阻塞：`cargo fmt --all -- --check`，失败 diff 位于既有 `bins/hone-cli/src/i18n.rs`、`bins/hone-cli/src/main.rs`、`bins/hone-cli/src/onboard.rs` 格式漂移，本次未修改这些文件。

## 后续

- Global digest / mainline distill 仍复用全局 OpenRouter provider；如果后续在这些短摘要/结构化输出路径继续出现 `HTTP 402`，应为它们单独设计 per-path token cap，而不是降低全局长输出预算。
