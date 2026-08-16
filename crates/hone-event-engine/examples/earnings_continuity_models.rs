//! Paid, manual 24-event replay for earnings quality + actor continuity.
//!
//! The fixture stores only SEC URLs and research hypotheses. This example fetches
//! first-party exhibits live, runs the production quality reviewer, then runs the
//! production A-tier continuity reconciler sequentially for each company.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::{NaiveDate, TimeZone, Utc};
use futures::StreamExt;
use futures::stream::{self, BoxStream};
use hone_core::config::HoneConfig;
use hone_core::{ActorIdentity, HoneResult};
use hone_event_engine::earnings_continuity::{
    EarningsContinuityOutcome, EarningsContinuityReconciler, LlmEarningsContinuityReconciler,
};
use hone_event_engine::pollers::earnings_quality::{
    EarningsQualityReview, EarningsQualityReviewer, LlmEarningsQualityReviewer,
    apply_earnings_quality_review,
};
use hone_event_engine::pollers::sec_enrichment::extract_filing_llm_context;
use hone_event_engine::{EventKind, MarketEvent, Severity};
use hone_llm::provider::{ChatResult, TokenUsage};
use hone_llm::{ChatResponse, LlmProvider, LlmRequestOptions, Message, OpenRouterProvider};
use hone_memory::{
    AppendEventInput, AppendResearchEventInput, CompanyProfileStorage, CoverageTier,
    CreateProfileInput, IndustryTemplate, ResearchItemKind, ResearchItemStatus,
    ResearchLedgerUpdate, TrackingConfig, research_item_id,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

const CONFIG_PATH: &str = "./config.yaml";
const DEFAULT_MODEL: &str = "x-ai/grok-4.5";
const FIXTURE: &str = include_str!(
    "../../../tests/fixtures/event_engine/earnings_continuity_baseline_2026-08-06.json"
);

#[derive(Debug, Clone, Deserialize)]
struct Fixture {
    companies: Vec<CompanyFixture>,
}

#[derive(Debug, Clone, Deserialize)]
struct CompanyFixture {
    symbol: String,
    company_name: String,
    company_type: String,
    thesis: String,
    focus_metrics: Vec<String>,
    seed_questions: Vec<String>,
    events: Vec<EventFixture>,
}

#[derive(Debug, Clone, Deserialize)]
struct EventFixture {
    date: String,
    period: String,
    url: String,
}

#[derive(Debug, Clone, Copy, Default, Serialize)]
struct MeterSnapshot {
    calls: u64,
    prompt_tokens: u64,
    completion_tokens: u64,
}

impl MeterSnapshot {
    fn delta(self, before: Self) -> Self {
        Self {
            calls: self.calls.saturating_sub(before.calls),
            prompt_tokens: self.prompt_tokens.saturating_sub(before.prompt_tokens),
            completion_tokens: self
                .completion_tokens
                .saturating_sub(before.completion_tokens),
        }
    }
}

struct MeteredProvider {
    inner: OpenRouterProvider,
    usage: Mutex<MeterSnapshot>,
}

impl MeteredProvider {
    fn snapshot(&self) -> MeterSnapshot {
        *self.usage.lock().unwrap()
    }

    fn record(&self, usage: Option<&TokenUsage>) {
        let mut total = self.usage.lock().unwrap();
        total.calls += 1;
        if let Some(usage) = usage {
            total.prompt_tokens += usage.prompt_tokens.unwrap_or(0) as u64;
            total.completion_tokens += usage.completion_tokens.unwrap_or(0) as u64;
        }
    }
}

#[async_trait]
impl LlmProvider for MeteredProvider {
    async fn chat(&self, messages: &[Message], model: Option<&str>) -> HoneResult<ChatResult> {
        let result = self.inner.chat(messages, model).await;
        if let Ok(result) = &result {
            self.record(result.usage.as_ref());
            if std::env::var("HONE_EARNINGS_CONTINUITY_DEBUG").as_deref() == Ok("1")
                && messages
                    .first()
                    .and_then(|message| message.content.as_deref())
                    .is_some_and(|prompt| prompt.contains("季度连续性审计员"))
            {
                eprintln!("CONTINUITY_DEBUG {}", result.content);
            }
        }
        result
    }

    async fn chat_with_tools(
        &self,
        messages: &[Message],
        tools: &[Value],
        model: Option<&str>,
    ) -> HoneResult<ChatResponse> {
        let result = self.inner.chat_with_tools(messages, tools, model).await;
        if let Ok(result) = &result {
            self.record(result.usage.as_ref());
        }
        result
    }

    fn chat_stream<'a>(
        &'a self,
        _: &'a [Message],
        _: Option<&'a str>,
    ) -> BoxStream<'a, HoneResult<String>> {
        Box::pin(stream::empty())
    }
}

#[derive(Debug, Serialize)]
struct ReplayResult {
    model: String,
    symbol: String,
    company_type: String,
    period: String,
    date: String,
    source_url: String,
    ok: bool,
    elapsed_seconds: f64,
    /// Present only when company concurrency is 1. With shared concurrent calls,
    /// before/after meter deltas overlap and would misattribute another company.
    usage: Option<MeterSnapshot>,
    estimated_cost_usd: Option<f64>,
    quality_contract_score: u8,
    continuity_contract_score: u8,
    contract_max: u8,
    active_items_before: usize,
    quality_review: Option<EarningsQualityReview>,
    continuity_outcome: Option<EarningsContinuityOutcome>,
    error: Option<String>,
}

#[derive(Debug, Serialize)]
struct ReplaySummary {
    kind: &'static str,
    model: String,
    samples: usize,
    passed: usize,
    failed: usize,
    average_contract_score: f64,
    contract_max: u8,
    total_usage: MeterSnapshot,
    estimated_total_cost_usd: Option<f64>,
    elapsed_seconds: f64,
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = HoneConfig::from_file(CONFIG_PATH).context("load config.yaml")?;
    let fixture: Fixture = serde_json::from_str(FIXTURE).context("parse continuity fixture")?;
    let model = std::env::var("HONE_EARNINGS_CONTINUITY_MODEL")
        .unwrap_or_else(|_| DEFAULT_MODEL.to_string());
    let selected = std::env::var("HONE_EARNINGS_CONTINUITY_COMPANIES")
        .ok()
        .map(|value| {
            value
                .split(',')
                .map(|item| item.trim().to_ascii_uppercase())
                .filter(|item| !item.is_empty())
                .collect::<Vec<_>>()
        });
    let event_limit = std::env::var("HONE_EARNINGS_CONTINUITY_EVENT_LIMIT")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(usize::MAX);
    let concurrency = std::env::var("HONE_EARNINGS_CONTINUITY_CONCURRENCY")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(2)
        .clamp(1, 6);
    let companies = fixture
        .companies
        .into_iter()
        .filter(|company| {
            selected.as_ref().is_none_or(|symbols| {
                symbols
                    .iter()
                    .any(|symbol| symbol.eq_ignore_ascii_case(&company.symbol))
            })
        })
        .collect::<Vec<_>>();
    if companies.is_empty() {
        anyhow::bail!("no companies selected");
    }

    let options = LlmRequestOptions {
        max_tokens: Some(3_600),
        temperature: Some(0.2),
        reasoning: Some(json!({"effort": "low"})),
        response_format: Some(json!({"type": "json_object"})),
        ..Default::default()
    };
    let inner =
        OpenRouterProvider::from_config_with_model_and_options(&config, &model, 3_600, options)
            .with_context(|| format!("construct provider for {model}"))?;
    let provider = Arc::new(MeteredProvider {
        inner,
        usage: Mutex::new(MeterSnapshot::default()),
    });
    let client = reqwest::Client::builder()
        .user_agent(&config.event_engine.sec_filings.enrichment.user_agent)
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .context("build SEC client")?;
    let prices = fetch_model_prices(&client).await.unwrap_or(Value::Null);
    let root = tempfile::tempdir().context("create benchmark profile root")?;
    let started = Instant::now();

    let tasks = futures::stream::iter(companies.into_iter().map(|company| {
        let root = root.path().to_path_buf();
        let client = client.clone();
        let provider = provider.clone();
        let model = model.clone();
        let prices = prices.clone();
        let sample_meter_isolated = concurrency == 1;
        async move {
            process_company(
                company,
                root,
                client,
                provider,
                model,
                prices,
                event_limit,
                sample_meter_isolated,
            )
            .await
        }
    }))
    .buffer_unordered(concurrency)
    .collect::<Vec<_>>()
    .await;

    let mut results = tasks.into_iter().flatten().collect::<Vec<_>>();
    results.sort_by(|left, right| {
        left.symbol
            .cmp(&right.symbol)
            .then_with(|| left.date.cmp(&right.date))
    });
    for result in &results {
        println!("{}", serde_json::to_string(result)?);
    }
    let passed = results.iter().filter(|result| result.ok).count();
    let total_score = results
        .iter()
        .map(|result| result.quality_contract_score + result.continuity_contract_score)
        .map(u64::from)
        .sum::<u64>();
    let usage = provider.snapshot();
    let summary = ReplaySummary {
        kind: "summary",
        model: model.clone(),
        samples: results.len(),
        passed,
        failed: results.len().saturating_sub(passed),
        average_contract_score: if results.is_empty() {
            0.0
        } else {
            total_score as f64 / results.len() as f64
        },
        contract_max: 18,
        total_usage: usage,
        estimated_total_cost_usd: estimate_cost(&prices, &model, usage),
        elapsed_seconds: started.elapsed().as_secs_f64(),
    };
    println!("{}", serde_json::to_string(&summary)?);
    Ok(())
}

async fn process_company(
    company: CompanyFixture,
    root: PathBuf,
    client: reqwest::Client,
    provider: Arc<MeteredProvider>,
    model: String,
    prices: Value,
    event_limit: usize,
    sample_meter_isolated: bool,
) -> Vec<ReplayResult> {
    let actor = ActorIdentity::new(
        "benchmark",
        company.symbol.to_ascii_lowercase(),
        None::<&str>,
    )
    .expect("benchmark actor");
    let storage = CompanyProfileStorage::new(&root);
    let profile = create_profile(&storage, &actor, &company).await;
    seed_questions(
        &storage,
        &actor,
        &profile.profile_id,
        &company.seed_questions,
    )
    .await;
    let quality = LlmEarningsQualityReviewer::new(provider.clone(), model.clone());
    let continuity = LlmEarningsContinuityReconciler::new(
        provider.clone(),
        model.clone(),
        CompanyProfileStorage::new(&root),
    );
    let mut results = Vec::new();

    for sample in company.events.clone().into_iter().take(event_limit) {
        let started = Instant::now();
        let before_usage = provider.snapshot();
        let before_profile = storage
            .for_actor(&actor)
            .get_profile(&profile.profile_id)
            .await
            .ok()
            .flatten();
        let before_mainline = before_profile
            .as_ref()
            .and_then(|profile| profile.section("投资主线"));
        let active_before = before_profile
            .as_ref()
            .map(|profile| {
                profile
                    .research_ledger()
                    .items
                    .iter()
                    .filter(|item| item.status.is_active())
                    .count()
            })
            .unwrap_or(0);
        let replay = replay_one(&company, &sample, &actor, &client, &quality, &continuity).await;
        let usage = provider.snapshot().delta(before_usage);
        let reported_usage = sample_meter_isolated.then_some(usage);
        let elapsed_seconds = started.elapsed().as_secs_f64();
        let after_profile = storage
            .for_actor(&actor)
            .get_profile(&profile.profile_id)
            .await
            .ok()
            .flatten();
        let after_mainline = after_profile
            .as_ref()
            .and_then(|profile| profile.section("投资主线"));
        let result = match replay {
            Ok((review, outcome, event)) => {
                let quality_score = quality_contract_score(&review);
                let continuity_score = continuity_contract_score(
                    &outcome,
                    active_before,
                    before_mainline.as_deref(),
                    after_mainline.as_deref(),
                    after_profile.as_ref(),
                    &event,
                );
                ReplayResult {
                    model: model.clone(),
                    symbol: company.symbol.clone(),
                    company_type: company.company_type.clone(),
                    period: sample.period,
                    date: sample.date,
                    source_url: sample.url,
                    ok: quality_score == 10 && continuity_score == 8,
                    elapsed_seconds,
                    usage: reported_usage,
                    estimated_cost_usd: reported_usage
                        .and_then(|usage| estimate_cost(&prices, &model, usage)),
                    quality_contract_score: quality_score,
                    continuity_contract_score: continuity_score,
                    contract_max: 18,
                    active_items_before: active_before,
                    quality_review: Some(review),
                    continuity_outcome: Some(outcome),
                    error: None,
                }
            }
            Err(error) => ReplayResult {
                model: model.clone(),
                symbol: company.symbol.clone(),
                company_type: company.company_type.clone(),
                period: sample.period,
                date: sample.date,
                source_url: sample.url,
                ok: false,
                elapsed_seconds,
                usage: reported_usage,
                estimated_cost_usd: reported_usage
                    .and_then(|usage| estimate_cost(&prices, &model, usage)),
                quality_contract_score: 0,
                continuity_contract_score: 0,
                contract_max: 18,
                active_items_before: active_before,
                quality_review: None,
                continuity_outcome: None,
                error: Some(error.to_string()),
            },
        };
        results.push(result);
    }
    results
}

async fn replay_one(
    company: &CompanyFixture,
    sample: &EventFixture,
    actor: &ActorIdentity,
    client: &reqwest::Client,
    quality: &LlmEarningsQualityReviewer,
    continuity: &LlmEarningsContinuityReconciler,
) -> Result<(
    EarningsQualityReview,
    EarningsContinuityOutcome,
    MarketEvent,
)> {
    let html = client
        .get(&sample.url)
        .send()
        .await
        .with_context(|| format!("fetch {} {} SEC exhibit", company.symbol, sample.period))?
        .error_for_status()
        .with_context(|| format!("{} {} SEC status", company.symbol, sample.period))?
        .text()
        .await
        .with_context(|| format!("read {} {} SEC exhibit", company.symbol, sample.period))?;
    let context = extract_filing_llm_context(&html, "8-K", &company.symbol, 9_000);
    if context.trim().is_empty() {
        anyhow::bail!("empty SEC context");
    }
    let mut event = sample_event(company, sample)?;
    let review = quality
        .review(&event, &context)
        .await
        .context("quality review failed")?;
    if !apply_earnings_quality_review(
        &mut event,
        review.clone(),
        Some(sample.url.clone()),
        0.0,
        0.9,
    ) {
        anyhow::bail!("quality review was not applied");
    }
    let outcome = continuity
        .reconcile(actor, &event)
        .await
        .context("continuity review failed")?;
    Ok((review, outcome, event))
}

async fn create_profile(
    storage: &CompanyProfileStorage,
    actor: &ActorIdentity,
    company: &CompanyFixture,
) -> hone_memory::CompanyProfileDocument {
    let mut sections = BTreeMap::new();
    sections.insert("投资主线".to_string(), company.thesis.clone());
    sections.insert(
        "关键经营指标".to_string(),
        company
            .focus_metrics
            .iter()
            .map(|metric| format!("- {metric}"))
            .collect::<Vec<_>>()
            .join("\n"),
    );
    storage
        .for_actor(actor)
        .create_profile(CreateProfileInput {
            company_name: company.company_name.clone(),
            stock_code: Some(company.symbol.clone()),
            sector: None,
            aliases: vec![],
            industry_template: IndustryTemplate::General,
            tracking: Some(TrackingConfig {
                enabled: true,
                coverage_tier: CoverageTier::A,
                investment_horizon: "long_term".to_string(),
                cadence: "earnings_cycle".to_string(),
                focus_metrics: company.focus_metrics.clone(),
            }),
            initial_sections: sections,
        })
        .await
        .expect("create benchmark profile")
        .0
}

async fn seed_questions(
    storage: &CompanyProfileStorage,
    actor: &ActorIdentity,
    profile_id: &str,
    questions: &[String],
) {
    let updates = questions
        .iter()
        .map(|statement| ResearchLedgerUpdate {
            item_id: research_item_id(&ResearchItemKind::OpenQuestion, statement),
            kind: ResearchItemKind::OpenQuestion,
            statement: statement.clone(),
            status: ResearchItemStatus::Open,
            assessment: "历史回放开始前的投资者核验问题。".to_string(),
            due_at: Some("next earnings".to_string()),
            evidence: vec![],
        })
        .collect();
    storage
        .for_actor(actor)
        .append_research_event(
            profile_id,
            AppendResearchEventInput {
                event: AppendEventInput {
                    title: "历史回放初始研究清单".to_string(),
                    event_type: "research_baseline".to_string(),
                    occurred_at: "2025-01-01T00:00:00Z".to_string(),
                    mainline_impact: "baseline".to_string(),
                    changed_sections: vec!["未决问题".to_string()],
                    refs: vec![],
                    what_happened: "建立历史回放前的研究问题。".to_string(),
                    why_it_matters: "用于验证问题能否跨季度被重新核对。".to_string(),
                    mainline_effect: "不修改投资主线。".to_string(),
                    evidence: String::new(),
                    research_log: "benchmark seed".to_string(),
                    follow_up: "下一次财报逐项核验。".to_string(),
                },
                research_object_key: Some(format!("baseline:{profile_id}")),
                research_updates: updates,
            },
        )
        .await
        .expect("seed research questions")
        .expect("seed event");
}

fn sample_event(company: &CompanyFixture, sample: &EventFixture) -> Result<MarketEvent> {
    let date = NaiveDate::parse_from_str(&sample.date, "%Y-%m-%d")?;
    let occurred_at = Utc
        .from_local_datetime(&date.and_hms_opt(21, 0, 0).context("valid time")?)
        .single()
        .context("UTC datetime")?;
    Ok(MarketEvent {
        id: format!("earnings_replay:{}:{}", company.symbol, sample.date),
        kind: EventKind::EarningsReleased,
        severity: Severity::High,
        symbols: vec![company.symbol.clone()],
        occurred_at,
        title: format!("{} {} 财报历史回放", company.symbol, sample.period),
        summary: "历史财报事件回放；本样本不提供 EPS 市场共识。".to_string(),
        url: Some(sample.url.clone()),
        source: "benchmark.sec.earnings".to_string(),
        payload: json!({
            "period": sample.period,
            "benchmark_without_eps_consensus": true
        }),
    })
}

fn quality_contract_score(review: &EarningsQualityReview) -> u8 {
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
    score += u8::from((1..=28).contains(&review.headline_zh.chars().count()));
    score += u8::from(!review.summary_zh.trim().is_empty());
    score += u8::from((2..=3).contains(&review.evidence.len()));
    score += u8::from((1..=2).contains(&review.risks.len()));
    score += u8::from((1..=2).contains(&review.unknowns.len()));
    score += u8::from((1..=3).contains(&review.follow_ups.len()));
    score += u8::from(review.override_eps_only);
    score
}

fn continuity_contract_score(
    outcome: &EarningsContinuityOutcome,
    active_before: usize,
    before_mainline: Option<&str>,
    after_mainline: Option<&str>,
    after_profile: Option<&hone_memory::CompanyProfileDocument>,
    event: &MarketEvent,
) -> u8 {
    let mut score = 0;
    score += u8::from(!outcome.recorded_event_id.is_empty());
    score += u8::from(outcome.checked_existing_items == active_before.min(14));
    score += u8::from(matches!(
        outcome.thesis_effect.as_str(),
        "strengthen" | "unchanged" | "watch" | "weaken" | "insufficient_baseline"
    ));
    score += u8::from(before_mainline == after_mainline);
    score += u8::from(!outcome.research_object_key.trim().is_empty());
    score += u8::from(outcome.active_questions_after <= 8);
    score += u8::from(outcome.active_commitments_after <= 6);
    score += u8::from(after_profile.is_some_and(|profile| {
        profile.events.iter().any(|profile_event| {
            profile_event.id == outcome.recorded_event_id
                && event.url.as_ref().is_some_and(|url| {
                    profile_event
                        .metadata
                        .refs
                        .iter()
                        .any(|source| source == url)
                })
        })
    }));
    score
}

async fn fetch_model_prices(client: &reqwest::Client) -> Result<Value> {
    Ok(client
        .get("https://openrouter.ai/api/v1/models")
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?)
}

fn estimate_cost(prices: &Value, model: &str, usage: MeterSnapshot) -> Option<f64> {
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
    Some(
        usage.prompt_tokens as f64 * prompt_price
            + usage.completion_tokens as f64 * completion_price,
    )
}
