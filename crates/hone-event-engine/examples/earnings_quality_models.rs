//! Manual OpenRouter model comparison for the earnings-quality contract.
//!
//! This example performs real paid calls. It never writes EventStore/delivery state.
//! Run only after explicit budget approval:
//!
//! ```text
//! HONE_EARNINGS_BENCHMARK_MODELS=x-ai/grok-4.3,openai/gpt-5.4 \
//! HONE_EARNINGS_BENCHMARK_SAMPLE_LIMIT=1 \
//! cargo run -p hone-event-engine --example earnings_quality_models
//! ```

use std::time::Instant;

use anyhow::{Context, Result};
use chrono::Utc;
use hone_core::config::HoneConfig;
use hone_event_engine::pollers::earnings_quality::{
    DEFAULT_EARNINGS_QUALITY_SYSTEM_PROMPT, EarningsQualityReview,
};
use hone_event_engine::pollers::sec_enrichment::extract_filing_llm_context;
use hone_event_engine::{EventKind, MarketEvent, Severity};
use hone_llm::{LlmProvider, LlmRequestOptions, Message, OpenRouterProvider};
use serde::Serialize;
use serde_json::{Value, json};

const CONFIG_PATH: &str = "./config.yaml";
const DEFAULT_MODELS: &str = "x-ai/grok-4.3";

#[derive(Clone, Copy)]
struct Sample {
    symbol: &'static str,
    date: &'static str,
    url: &'static str,
    actual_eps: f64,
    estimated_eps: f64,
}

const SAMPLES: &[Sample] = &[
    Sample {
        symbol: "SNDK",
        date: "2026-08-05",
        url: "https://www.sec.gov/Archives/edgar/data/2023554/000162828026053346/sndkq4-26ex991xpressrelease.htm",
        actual_eps: 39.25,
        estimated_eps: 34.96,
    },
    Sample {
        symbol: "AMD",
        date: "2026-08-04",
        url: "https://www.sec.gov/Archives/edgar/data/2488/000000248826000121/q22026991.htm",
        actual_eps: 1.66,
        estimated_eps: 1.62,
    },
    Sample {
        symbol: "BE",
        date: "2026-07-28",
        url: "https://www.sec.gov/Archives/edgar/data/1664703/000162828026050150/ex991_q226financialresults.htm",
        actual_eps: 0.78,
        estimated_eps: 0.39,
    },
];

#[derive(Serialize)]
struct BenchmarkResult {
    model: String,
    symbol: String,
    ok: bool,
    elapsed_seconds: f64,
    prompt_tokens: Option<u32>,
    completion_tokens: Option<u32>,
    estimated_cost_usd: Option<f64>,
    contract_score: u8,
    contract_max: u8,
    review: Option<EarningsQualityReview>,
    error: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = HoneConfig::from_file(CONFIG_PATH).context("load config.yaml")?;
    let models = std::env::var("HONE_EARNINGS_BENCHMARK_MODELS")
        .unwrap_or_else(|_| DEFAULT_MODELS.to_string())
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    let sample_start = std::env::var("HONE_EARNINGS_BENCHMARK_SAMPLE_START")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(0)
        .min(SAMPLES.len());
    let sample_limit = std::env::var("HONE_EARNINGS_BENCHMARK_SAMPLE_LIMIT")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(SAMPLES.len().saturating_sub(sample_start))
        .min(SAMPLES.len().saturating_sub(sample_start));
    if models.is_empty() || sample_limit == 0 {
        anyhow::bail!("at least one model and sample are required");
    }

    let client = reqwest::Client::builder()
        .user_agent(&config.event_engine.sec_filings.enrichment.user_agent)
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .context("build SEC client")?;
    let prices = fetch_model_prices(&client).await.unwrap_or(Value::Null);
    let mut contexts = Vec::with_capacity(sample_limit);
    for sample in SAMPLES.iter().skip(sample_start).take(sample_limit) {
        let html = client
            .get(sample.url)
            .send()
            .await
            .with_context(|| format!("fetch {} SEC exhibit", sample.symbol))?
            .error_for_status()
            .with_context(|| format!("{} SEC exhibit status", sample.symbol))?
            .text()
            .await
            .with_context(|| format!("read {} SEC exhibit", sample.symbol))?;
        let context = extract_filing_llm_context(&html, "8-K", sample.symbol, 9_000);
        if context.trim().is_empty() {
            anyhow::bail!("{} SEC context is empty", sample.symbol);
        }
        contexts.push((*sample, context));
    }

    for model in models {
        let options = LlmRequestOptions {
            max_tokens: Some(1_800),
            temperature: Some(0.2),
            response_format: Some(json!({"type": "json_object"})),
            ..Default::default()
        };
        let provider =
            OpenRouterProvider::from_config_with_model_and_options(&config, &model, 1_800, options)
                .with_context(|| format!("construct provider for {model}"))?;
        for (sample, context) in &contexts {
            let event = sample_event(sample);
            let messages = review_messages(&event, context);
            let started = Instant::now();
            let response = provider.chat(&messages, Some(&model)).await;
            let elapsed_seconds = started.elapsed().as_secs_f64();
            let result = match response {
                Ok(response) => {
                    let prompt_tokens = response
                        .usage
                        .as_ref()
                        .and_then(|usage| usage.prompt_tokens);
                    let completion_tokens = response
                        .usage
                        .as_ref()
                        .and_then(|usage| usage.completion_tokens);
                    match parse_review(&response.content) {
                        Some(review) => BenchmarkResult {
                            model: model.clone(),
                            symbol: sample.symbol.to_string(),
                            ok: true,
                            elapsed_seconds,
                            prompt_tokens,
                            completion_tokens,
                            estimated_cost_usd: estimate_cost(
                                &prices,
                                &model,
                                prompt_tokens,
                                completion_tokens,
                            ),
                            contract_score: contract_score(&review),
                            contract_max: 10,
                            review: Some(review),
                            error: None,
                        },
                        None => BenchmarkResult {
                            model: model.clone(),
                            symbol: sample.symbol.to_string(),
                            ok: false,
                            elapsed_seconds,
                            prompt_tokens,
                            completion_tokens,
                            estimated_cost_usd: estimate_cost(
                                &prices,
                                &model,
                                prompt_tokens,
                                completion_tokens,
                            ),
                            contract_score: 0,
                            contract_max: 10,
                            review: None,
                            error: Some(format!(
                                "unparseable response: {}",
                                response.content.chars().take(240).collect::<String>()
                            )),
                        },
                    }
                }
                Err(error) => BenchmarkResult {
                    model: model.clone(),
                    symbol: sample.symbol.to_string(),
                    ok: false,
                    elapsed_seconds,
                    prompt_tokens: None,
                    completion_tokens: None,
                    estimated_cost_usd: None,
                    contract_score: 0,
                    contract_max: 10,
                    review: None,
                    error: Some(error.to_string()),
                },
            };
            println!("{}", serde_json::to_string(&result)?);
        }
    }
    Ok(())
}

fn sample_event(sample: &Sample) -> MarketEvent {
    let delta = sample.actual_eps - sample.estimated_eps;
    let surprise_pct = if sample.estimated_eps.abs() > f64::EPSILON {
        delta / sample.estimated_eps.abs() * 100.0
    } else {
        0.0
    };
    MarketEvent {
        id: format!("earnings_surprise:{}:{}", sample.symbol, sample.date),
        kind: EventKind::EarningsReleased,
        severity: Severity::High,
        symbols: vec![sample.symbol.to_string()],
        occurred_at: Utc::now(),
        title: format!("{} 财报 EPS surprise 候选", sample.symbol),
        summary: format!(
            "EPS actual {:.2}, estimate {:.2}, surprise {surprise_pct:.2}%",
            sample.actual_eps, sample.estimated_eps
        ),
        url: Some(sample.url.to_string()),
        source: "benchmark.earnings_quality".to_string(),
        payload: json!({
            "date": sample.date,
            "actualEarningResult": sample.actual_eps,
            "estimatedEarning": sample.estimated_eps,
            "computed_eps_delta": delta,
            "computed_eps_surprise_pct": surprise_pct,
        }),
    }
}

fn review_messages(event: &MarketEvent, context: &str) -> Vec<Message> {
    let payload = serde_json::to_string(&event.payload).unwrap_or_else(|_| "{}".to_string());
    let user = format!(
        "Ticker: {}\nCandidate title: {}\nEPS trigger summary: {}\nRaw EPS payload: {}\n\nSEC earnings-release excerpt:\n{}",
        event.symbols.first().cloned().unwrap_or_default(),
        event.title,
        event.summary,
        payload,
        context
    );
    vec![
        Message {
            role: "system".to_string(),
            content: Some(DEFAULT_EARNINGS_QUALITY_SYSTEM_PROMPT.to_string()),
            reasoning_content: None,
            tool_calls: None,
            tool_call_id: None,
            name: None,
        },
        Message {
            role: "user".to_string(),
            content: Some(user),
            reasoning_content: None,
            tool_calls: None,
            tool_call_id: None,
            name: None,
        },
    ]
}

fn parse_review(content: &str) -> Option<EarningsQualityReview> {
    let trimmed = content.trim();
    let candidate = if trimmed.starts_with("```") {
        trimmed
            .lines()
            .skip(1)
            .take_while(|line| !line.trim_start().starts_with("```"))
            .collect::<Vec<_>>()
            .join("\n")
    } else if let (Some(start), Some(end)) = (trimmed.find('{'), trimmed.rfind('}')) {
        trimmed[start..=end].to_string()
    } else {
        trimmed.to_string()
    };
    serde_json::from_str(&candidate).ok()
}

fn contract_score(review: &EarningsQualityReview) -> u8 {
    let mut score = 0;
    score += u8::from(matches!(
        review.conclusion.as_str(),
        "positive" | "mixed_positive" | "neutral" | "mixed_negative" | "negative" | "unclear"
    ));
    score += u8::from(matches!(
        review.route.as_str(),
        "immediate" | "digest" | "suppress"
    ));
    score += u8::from(review.confidence.is_finite() && (0.0..=1.0).contains(&review.confidence));
    let headline_chars = review.headline_zh.chars().count();
    score += u8::from(headline_chars > 0 && headline_chars <= 28);
    score += u8::from(!review.summary_zh.trim().is_empty());
    score += u8::from((2..=3).contains(&review.evidence.len()));
    score += u8::from((1..=2).contains(&review.risks.len()));
    score += u8::from((1..=2).contains(&review.unknowns.len()));
    score += u8::from((1..=3).contains(&review.follow_ups.len()));
    score += u8::from(review.override_eps_only);
    score
}

async fn fetch_model_prices(client: &reqwest::Client) -> Result<Value> {
    let response: Value = client
        .get("https://openrouter.ai/api/v1/models")
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    Ok(response)
}

fn estimate_cost(
    prices: &Value,
    model: &str,
    prompt_tokens: Option<u32>,
    completion_tokens: Option<u32>,
) -> Option<f64> {
    let entry = prices
        .get("data")?
        .as_array()?
        .iter()
        .find(|entry| entry.get("id").and_then(Value::as_str) == Some(model))?;
    let prompt_price = entry
        .get("pricing")?
        .get("prompt")?
        .as_str()?
        .parse::<f64>()
        .ok()?;
    let completion_price = entry
        .get("pricing")?
        .get("completion")?
        .as_str()?
        .parse::<f64>()
        .ok()?;
    Some(prompt_tokens? as f64 * prompt_price + completion_tokens? as f64 * completion_price)
}
