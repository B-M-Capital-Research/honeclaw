# Bug: event-engine news classifier 403 errors downgraded uncertain-source review

## Summary

The event-engine LLM news classifier hit repeated OpenRouter 403 / response-deserialization failures, causing uncertain-source news classification to return `None` and fall back to the non-LLM low-priority path.

## Observed Symptoms

- The 17:44 backend start assembled the classifier with `google/gemini-3-flash-preview`:

```text
data/runtime/logs/web.log.2026-04-22:126:[2026-04-22 17:44:55.560] INFO  event engine sink: MultiChannelSink 已装配
data/runtime/logs/web.log.2026-04-22:127:[2026-04-22 17:44:55.560] INFO  event engine: news LLM classifier 装配 (model=google/gemini-3-flash-preview)
data/runtime/logs/web.log.2026-04-22:140:[2026-04-22 17:44:55.565] INFO  event engine: news LLM classifier 已装配 (uncertain-source 升 Medium)
```

- Immediately after that, `data/runtime/logs/web.log.2026-04-22` logged 14 classifier failures and 14 companion deserialization errors in about 3 seconds:

```text
data/runtime/logs/web.log.2026-04-22:188:[2026-04-22 17:44:59.774] ERROR failed deserialization of: {"error":{"message":"The request is prohibited due to a violation of provider Terms Of Service.","code":403,"metadata":{"provider_name":null}},"user_id":"user_2xJGNr7B5veqW3ACqXhf0EUf6Im"}
data/runtime/logs/web.log.2026-04-22:189:[2026-04-22 17:44:59.774] WARN  news LLM classifier call failed: LLM 错误: 所有 OpenRouter API Key 均失败（共 1 个）。最后错误：failed to deserialize api response: invalid type: integer `403`, expected a string at line 1 column 107
data/runtime/logs/web.log.2026-04-22:191:[2026-04-22 17:44:59.951] ERROR failed deserialization of: {"error":{"message":"The request is prohibited due to a violation of provider Terms Of Service.","code":403,"metadata":{"provider_name":null}},"user_id":"user_2xJGNr7B5veqW3ACqXhf0EUf6Im"}
data/runtime/logs/web.log.2026-04-22:192:[2026-04-22 17:44:59.951] WARN  news LLM classifier call failed: LLM 错误: 所有 OpenRouter API Key 均失败（共 1 个）。最后错误：failed to deserialize api response: invalid type: integer `403`, expected a string at line 1 column 107
```

- The same burst produced digest queue entries rather than LLM-upgraded medium evidence:

```text
data/runtime/logs/web.log.2026-04-22:190:[2026-04-22 17:44:59.777] INFO  digest queued
data/runtime/logs/web.log.2026-04-22:193:[2026-04-22 17:44:59.954] INFO  digest queued
data/runtime/logs/web.log.2026-04-22:196:[2026-04-22 17:45:00.119] INFO  digest queued
```

- `data/events.sqlite3` shows uncertain-source items continued to arrive after the previous巡检, such as `telegram.watcherguru` and `fmp.stock_news:*` rows with `source_class=uncertain`, so the failing path was relevant to real event routing.

## Hypothesis / Suspected Code Path

`crates/hone-web-api/src/lib.rs:66` documents that classifier setup failure degrades to no classifier, but runtime request failures are handled inside the classifier and are not surfaced as a health state.

```rust
/// 装配"不确定来源 NewsCritical → LLM 仲裁"分类器。
/// 走 OpenRouter 的 `openai/gpt-oss-20b:nitro`,key 复用 llm.openrouter.api_key。
/// 失败一律退化为 `None`(router 跳过 LLM 路径,uncertain 源新闻保持 Low)。
fn build_event_engine_news_classifier(
    core_cfg: &HoneConfig,
) -> Option<Arc<dyn hone_event_engine::NewsClassifier>> {
    match OpenRouterProvider::from_config(core_cfg) {
        Ok(provider) => {
            let provider: Arc<dyn LlmProvider> = Arc::new(provider);
            let classifier = hone_event_engine::LlmNewsClassifier::new(
```

`crates/hone-event-engine/src/news_classifier.rs:226` calls the provider and returns `None` on any error. That makes the router indistinguishable from "not important" / "no upgrade" for the current actor.

```rust
let messages = Self::build_messages(event, importance_prompt);
let result = self.provider.chat(&messages, Some(&self.model)).await;
match result {
    Ok(resp) => {
        let importance = Self::parse(&resp.content);
        if let Ok(mut cache) = self.cache.lock() {
            cache.insert(l1_key, importance);
        }
        if !title_norm.is_empty() {
            if let Ok(mut cache) = self.title_cache.lock() {
                cache.insert(l2_key, importance);
            }
        }
        Some(importance)
    }
    Err(e) => {
        tracing::warn!(
            event_id = %event.id,
            "news LLM classifier call failed: {e}"
        );
        None
```

`crates/hone-event-engine/src/router.rs:301` then treats `None` as no upgrade and proceeds with the original low severity, so important uncertain-source items can be delayed into digest or remain low.

```rust
let classifier = self.news_classifier.as_ref()?;
let prompt = prefs
    .news_importance_prompt
    .as_deref()
    .unwrap_or(&self.default_importance_prompt);
match classifier.classify(event, prompt).await {
    Some(Importance::Important) => {
        let mut upgraded = event.clone();
        upgraded.severity = Severity::Medium;
        tracing::info!(
            event_id = %event.id,
            "uncertain-source news upgraded Low→Medium by LLM classifier"
        );
        Some(upgraded)
    }
    _ => None,
}
```

## Evidence Gap

- Need event ids on the plain `news LLM classifier call failed` lines; structured fields may exist but are not visible in current text logs.
- Need a classifier health counter or circuit-breaker metric to distinguish transient single-event failures from a model/provider outage.
- This巡检 did not call OpenRouter or retry the model, so it cannot determine whether the 403 was model-specific, account-policy-specific, or a parser bug caused by numeric `code`.

## Severity

sev2. The failure directly affects quality routing for uncertain-source financial/social news; important items can miss the intended LLM upgrade path and be delayed or buried without a durable per-event classification result.

## Date Observed

2026-04-22T10:11:32Z

## Fix Update

- 2026-04-28: 复核当前 `crates/hone-event-engine/src/news_classifier.rs` 已在 provider error 与不可解析响应时走 `deterministic_fallback(event)`，并把 fallback 写入 L1/L2 cache；该路径不再返回 `None` 让 uncertain-source 新闻静默退回“未仲裁”。
- 同轮 `crates/hone-llm/src/openai_compatible.rs` 对 numeric `error.code` 增加 raw HTTP 兜底解析，后续 403/400 错误会保留真实 provider message，而不是只剩 `invalid type: integer`。
- 状态调整为 `Fixed`；若后续需要做 classifier health/circuit breaker，可另开观测增强项，不再阻塞本缺陷关闭。
