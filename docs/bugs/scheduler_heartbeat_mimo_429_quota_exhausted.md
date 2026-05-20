# Bug: Heartbeat 使用 `mimo-v2.5-pro` 时批量触发 `HTTP 429 quota exhausted` 并漏发

- **发现时间**: 2026-05-20 19:04 CST
- **Bug Type**: System Error
- **严重等级**: P1
- **状态**: Fixed
- **GitHub Issue**: [#44](https://github.com/B-M-Capital-Research/honeclaw/issues/44)

## 证据来源

- GitHub Issue [#44](https://github.com/B-M-Capital-Research/honeclaw/issues/44) 与 bug 台账记录：最近四小时 heartbeat 任务批量命中 `mimo-v2.5-pro` 上游 `HTTP 429` / `quota exhausted`，多条监控检查落成 `execution_failed + skipped_error + delivered=0`。
- 受影响范围覆盖价格破位、持仓财报、重大新闻、板块关键事件、观察池等多个 heartbeat job；同窗直聊会话仍能正常收口，故障集中在 heartbeat provider quota / rate-limit 路径。
- 本轮修复不依赖当前机器生产日志、线上健康检查或真实投递状态；判断与验证基于 issue 摘要、现有 heartbeat 代码、配置解析和本地回归。

## 端到端链路

1. Heartbeat scheduler 在半点 / 整点窗口批量触发多条监控 job。
2. 公共 heartbeat runner 调用 auxiliary profile，对应 OpenAI-compatible provider `mimo-v2.5-pro`。
3. 上游返回 `HTTP 429` / `quota exhausted`。
4. `llm.providers.<name>.api_keys` 虽然支持配置多个 key，但非 OpenRouter provider 解析时只取第一把 key。
5. 第一把 key 被上游拒绝后，本地没有尝试同 provider 的后续 key，整轮 heartbeat 直接失败；失败分类也未把 `HTTP 429` / `rate limit exceeded` 稳定归入 quota exhaustion。

## 期望效果

- 非 OpenRouter 的 OpenAI-compatible provider 应和 OpenRouter 一样支持多 key 顺序 fallback。
- 单 key 的 429 / quota / rate-limit 失败不应在已配置备用 key 时压垮整轮 heartbeat。
- 如果所有 key 都不可用，heartbeat 仍应失败并保留可观测的 `provider_quota_exhausted` 分类，而不是伪装为 noop 或泛化 HTTP 错误。

## 用户影响

- 这是功能性 bug，不是质量性 bug。
- 用户可能错过价格破位、重大事件、持仓财报、板块关键事件和观察池等自动监控提醒。
- 定级为 `P1`：批量 heartbeat 执行失败直接影响自动告警送达链路；虽然直聊未受影响，但自动监控主功能在该 provider 路径上出现批量失效。

## 根因判断

- 这是可控代码缺陷，不是要为某次外部 429 写特判。
- 现有配置结构已经有 `llm.providers.<name>.api_key/api_keys`，但 `LlmResolver::provider_for_profile(...)` 对非 OpenRouter provider 只调用 `.first()`，后续 key 被忽略。
- `OpenAiCompatibleProvider` 自身只保存单个 client/key；非 streaming `chat` / `chat_with_tools` 无法跨 key fallback。
- 既有 `scheduler_heartbeat_openrouter_402_credit_exhaustion_skips_alerts.md` 关注 OpenRouter `HTTP 402` / token budget / credits 不足；本单是 mimo/OpenAI-compatible profile 的 `HTTP 429` key-pool fallback 缺口。
- 既有 `scheduler_heartbeat_mimo_param_incorrect_batch_failures.md` 关注同一模型的 `HTTP 400 Param Incorrect` / `reasoning_content` transcript 兼容问题；本单状态码、直接原因和修复边界不同。

## 修复情况

- 2026-05-20 修复：`OpenAiCompatibleProvider` 新增 key-pool 构造，非 streaming `chat` / `chat_with_tools` 会按配置 key 顺序尝试；单 key 传输错误仍保留一次短重试，HTTP 429 这类 provider 拒绝会直接尝试下一把 key。
- 2026-05-20 修复：`LlmResolver` 对非 OpenRouter profile 使用完整 `api_key/api_keys` pool，不再只取第一把 key。
- 2026-05-20 修复：heartbeat runner failure 分类补齐 `HTTP 429`、`code: 429`、`rate limit exceeded`、`too many requests`、`resource exhausted`，统一归入 `provider_quota_exhausted`。

## 验证

- `cargo test -p hone-llm chat_with_tools_falls_back_to_next_key_after_http_429 -- --nocapture`
- `cargo test -p hone-channels heartbeat_provider_429_quota_error_is_classified --lib -- --nocapture`
- `cargo test -p hone-llm openai_compatible -- --nocapture`
- `cargo test -p hone-llm resolver -- --nocapture`
- `cargo test -p hone-channels heartbeat_provider_ --lib -- --nocapture`
- `cargo check -p hone-llm -p hone-channels --tests`
- `rustfmt --edition 2024 --check crates/hone-llm/src/openai_compatible.rs crates/hone-llm/src/resolver.rs crates/hone-channels/src/scheduler.rs`
- `git diff --check`

## 未验证项 / 后续建议

- 如果配置只有一把已耗尽 key，本地仍会正确失败；这是外部额度不可用，不应继续降低模型预算或写一次性特殊兼容。
- 建议部署后确认 auxiliary profile 的 OpenAI-compatible provider 如需抗 429，应在 `llm.providers.<name>.api_keys` 配置多个有效 key。
