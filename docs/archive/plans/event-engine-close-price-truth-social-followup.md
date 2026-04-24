# Event Engine Close Price 与 Truth Social 后续修复

- title: Event Engine Close Price 与 Truth Social 后续修复
- status: done
- created_at: 2026-04-24
- updated_at: 2026-04-24
- owner: Codex
- related_files:
  - `crates/hone-event-engine/src/pollers/social/truth_social.rs`
  - `crates/hone-event-engine/src/pollers/price.rs`
  - `crates/hone-event-engine/src/router.rs`
  - `tests/fixtures/event_engine/news_classifier_baseline_2026-04-23.json`
  - `scripts/diagnose_event_engine_daily_pushes.py`
- related_docs:
  - `docs/bugs/truth_social_poller_opaque_json_decode_stalls_source.md`
  - `docs/bugs/event_engine_close_price_alerts_never_immediate.md`
  - `docs/bugs/event_engine_social_source_decode_failures.md`
  - `docs/handoffs/2026-04-24-event-engine-close-price-truth-social-followup.md`

## Goal

先补 Truth Social poller 对非 JSON / 非 2xx 响应的可观测性，再修复 `price_close` 高波动被固定压成 digest 的规则；完成后用真实模型 baseline 和 `telegram::::8039067465` 的 2026-04-23 推送校准导出复核升级 / 降级样本。

## Scope

- Truth Social HTTP 响应失败时保留 status、content-type 和截断 body prefix，避免只剩 opaque JSON decode error。
- `price_close` 超过系统高阈值或用户价格 override 时能进入 High / immediate 路由，同时保留普通收盘波动 digest 化。
- 审阅昨日实际投递记录，列出应降级、应升级、应过滤或可入 baseline 的样本。

## Validation

- `rtk cargo test -p hone-event-engine truth_social --lib`
- `rtk cargo test -p hone-event-engine close_quote --lib`
- `rtk cargo test -p hone-event-engine per_actor_price_threshold_can_promote_closing_move --lib`
- `rtk cargo test -p hone-event-engine --lib`
- `rtk cargo fmt --all -- --check`
- `rtk bash tests/regression/manual/test_event_engine_news_classifier_baseline.sh`
- `rtk env RUN_EVENT_ENGINE_LLM_BASELINE=1 EVENT_ENGINE_NEWS_CLASSIFIER_MODEL=amazon/nova-lite-v1 bash tests/regression/manual/test_event_engine_news_classifier_baseline.sh`
- `rtk python3 scripts/diagnose_event_engine_daily_pushes.py --date 2026-04-23 --actor telegram::::8039067465`
- `rtk python3 scripts/diagnose_event_engine_daily_pushes.py --date 2026-04-23 --actor telegram::::8039067465 --include-body`

## Documentation Sync

- 已更新 `docs/bugs/truth_social_poller_opaque_json_decode_stalls_source.md`、`docs/bugs/event_engine_close_price_alerts_never_immediate.md`、`docs/bugs/event_engine_social_source_decode_failures.md` 与 `docs/bugs/README.md`。
- 已新增 handoff：`docs/handoffs/2026-04-24-event-engine-close-price-truth-social-followup.md`。
- 已从 `docs/current-plan.md` 移出，并补充到 `docs/archive/index.md`。

## Risks / Open Questions

- Truth Social 断流的真实外部响应仍需等补偿日志上线后观察；本轮只解决错误透明度。
- 昨日 Telegram 推送校准发现 BE +4% 低强度价格波动和若干 analyst/macro/news 噪声样本，已在 handoff 留作下一轮降噪入口。
