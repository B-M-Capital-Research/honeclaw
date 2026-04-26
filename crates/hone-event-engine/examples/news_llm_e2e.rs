//! Real e2e: FMP stock_news → poller → router-stage LLM arbitration.
//!
//! 跑法:cargo run --example news_llm_e2e -p hone-event-engine
//!
//! 流程:
//!   1. 从 ./config.yaml 读 fmp/openrouter 真 key。
//!   2. NewsPoller.poll() 调真 FMP /v3/stock_news,拉一页。
//!   3. 统计 source_class 分布(trusted / pr_wire / uncertain)与 legal_ad 命中数。
//!   4. 对每条 NewsCritical+Low+source_class=uncertain+非律所模板,真调
//!      OpenRouter `openai/gpt-oss-20b:nitro`,看返回是否能被解析为
//!      Important / NotImportant。
//!   5. 验证缓存:同一条新闻第二次 classify 不再 LLM 调用(LLM 计数稳定)。
//!
//! 不发 telegram、不写 sqlite,只做"链路真打通"验收。

use std::sync::Arc;

use anyhow::{Context, Result};
use hone_core::config::HoneConfig;
use hone_event_engine::fmp::FmpClient;
use hone_event_engine::news_classifier::{
    DEFAULT_IMPORTANCE_PROMPT, LlmNewsClassifier, NewsClassifier,
};
use hone_event_engine::pollers::news::NewsPoller;
use hone_event_engine::{EventKind, Severity};
use hone_llm::OpenRouterProvider;

const CONFIG_PATH: &str = "./config.yaml";
const LLM_MODEL: &str = "openai/gpt-oss-20b:nitro";
const DEFAULT_MAX_LLM_CALLS: usize = 8;

#[tokio::main]
async fn main() -> Result<()> {
    let cfg = HoneConfig::from_file(CONFIG_PATH).with_context(|| format!("加载 {CONFIG_PATH}"))?;

    let fmp = FmpClient::from_config(&cfg.fmp);
    if !fmp.has_keys() {
        anyhow::bail!("FMP key 未配置(config.yaml fmp.api_key 为空)");
    }

    let provider =
        OpenRouterProvider::from_config(&cfg).with_context(|| "构造 OpenRouterProvider")?;
    let provider: Arc<dyn hone_llm::LlmProvider> = Arc::new(provider);
    let classifier = LlmNewsClassifier::new(provider.clone(), LLM_MODEL);

    println!("== news_llm_e2e ==");
    println!("model      : {LLM_MODEL}");
    println!(
        "fmp keys   : {}",
        if fmp.has_keys() { "ok" } else { "MISSING" }
    );
    println!();

    // 可选:HONE_E2E_TICKERS=AAPL,GOOGL,... 限定到指定 ticker(用于持仓验证)
    let tickers: Vec<String> = std::env::var("HONE_E2E_TICKERS")
        .ok()
        .map(|s| {
            s.split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();
    let max_llm_calls: usize = std::env::var("HONE_E2E_MAX_LLM")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_MAX_LLM_CALLS);
    let page_limit: u32 = std::env::var("HONE_E2E_PAGE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(50);

    let mut poller = NewsPoller::new(
        fmp,
        hone_event_engine::source::SourceSchedule::FixedInterval(std::time::Duration::from_secs(
            60,
        )),
    )
    .with_page_limit(page_limit);
    if !tickers.is_empty() {
        poller = poller.with_tickers(tickers.clone());
        println!("限定 tickers: {}", tickers.join(","));
    }
    let events = hone_event_engine::source::EventSource::poll(&poller)
        .await
        .with_context(|| "FMP /v3/stock_news 调用失败")?;

    println!("FMP 返回事件数: {}", events.len());

    let mut trusted = 0usize;
    let mut pr_wire = 0usize;
    let mut uncertain = 0usize;
    let mut legal_ads = 0usize;
    let mut high = 0usize;
    let mut low = 0usize;

    for ev in &events {
        if ev.kind != EventKind::NewsCritical {
            continue;
        }
        match ev.severity {
            Severity::High => high += 1,
            Severity::Low => low += 1,
            _ => {}
        }
        match ev
            .payload
            .get("source_class")
            .and_then(|v| v.as_str())
            .unwrap_or("?")
        {
            "trusted" => trusted += 1,
            "pr_wire" => pr_wire += 1,
            "uncertain" => uncertain += 1,
            _ => {}
        }
        if ev
            .payload
            .get("legal_ad_template")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            legal_ads += 1;
        }
    }

    println!(
        "分布: trusted={trusted} pr_wire={pr_wire} uncertain={uncertain} legal_ad_template={legal_ads}"
    );
    println!("severity: high={high} low={low}");
    println!();

    // 选 uncertain + Low + 非律所模板的前 MAX_LLM_CALLS 条做 LLM 仲裁
    let mut candidates: Vec<_> = events
        .iter()
        .filter(|e| {
            e.kind == EventKind::NewsCritical
                && e.severity == Severity::Low
                && e.payload
                    .get("source_class")
                    .and_then(|v| v.as_str())
                    .map(|s| s == "uncertain")
                    .unwrap_or(false)
                && !e
                    .payload
                    .get("legal_ad_template")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
        })
        .collect();
    candidates.truncate(max_llm_calls);

    println!(
        "送入 LLM 的候选条数: {} (上限 {max_llm_calls})",
        candidates.len()
    );
    println!("使用 prompt: {DEFAULT_IMPORTANCE_PROMPT}");
    println!();

    let mut important = 0usize;
    let mut not_important = 0usize;
    let mut failed = 0usize;

    for (idx, ev) in candidates.iter().enumerate() {
        let symbols = if ev.symbols.is_empty() {
            "(无)".into()
        } else {
            ev.symbols.join(",")
        };
        print!(
            "[{i:>2}] sym={s:<10} src={src:<32} title={t} → ",
            i = idx + 1,
            s = symbols,
            src = ev.source.chars().take(32).collect::<String>(),
            t = ev.title.chars().take(60).collect::<String>(),
        );
        match classifier.classify(ev, DEFAULT_IMPORTANCE_PROMPT).await {
            Some(hone_event_engine::news_classifier::Importance::Important) => {
                println!("IMPORTANT");
                important += 1;
            }
            Some(hone_event_engine::news_classifier::Importance::NotImportant) => {
                println!("not_important");
                not_important += 1;
            }
            None => {
                println!("FAIL (network/parse)");
                failed += 1;
            }
        }
    }

    println!();
    println!("LLM 结果: important={important} not_important={not_important} failed={failed}");

    // 验证缓存:对同一批再跑一遍,LLM 不应被再次调用(由 cache 命中)
    if let Some(first) = candidates.first() {
        println!();
        println!("== 缓存验证 ==");
        let r1 = classifier.classify(first, DEFAULT_IMPORTANCE_PROMPT).await;
        let r2 = classifier.classify(first, DEFAULT_IMPORTANCE_PROMPT).await;
        println!(
            "同 event + 同 prompt 二次 classify: {:?} == {:?} (期望: 一致且无新 LLM 调用)",
            r1, r2
        );
        // 换 prompt 应该再调一次
        let r3 = classifier
            .classify(first, "仅与并购/重大监管处罚相关的事件视为重要")
            .await;
        println!("换 prompt 后: {:?} (LLM 应被再次调用)", r3);
    }

    Ok(())
}
