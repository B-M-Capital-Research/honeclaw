//! Paid/manual eight-call replay for shared transcript facts + actor continuity.
//!
//! The fixture contains only official IR URLs and research baselines. Source
//! documents are fetched into memory, reviewed once, and dropped. Neither this
//! executable nor its JSON output prints or persists transcript bodies.

use std::collections::BTreeMap;
use std::io::{Cursor, Read};
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
use hone_event_engine::earnings_transcript::{
    EarningsTranscriptReview, EarningsTranscriptReviewer, LlmEarningsTranscriptReviewer,
    apply_earnings_transcript_review,
};
use hone_event_engine::{EventKind, MarketEvent, Severity};
use hone_llm::provider::{ChatResult, TokenUsage};
use hone_llm::{ChatResponse, LlmProvider, LlmRequestOptions, Message, OpenRouterProvider};
use hone_memory::{
    AppendEventInput, AppendResearchEventInput, CompanyProfileStorage, CoverageTier,
    CreateProfileInput, IndustryTemplate, ResearchItemKind, ResearchItemStatus,
    ResearchLedgerUpdate, TrackingConfig, research_item_id,
};
use quick_xml::Reader;
use quick_xml::events::Event as XmlEvent;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

const CONFIG_PATH: &str = "./config.yaml";
const DEFAULT_MODEL: &str = "x-ai/grok-4.5";
const FIXTURE: &str = include_str!(
    "../../../tests/fixtures/event_engine/earnings_transcript_baseline_2026-08-06.json"
);
const MAX_SOURCE_BYTES: usize = 5 * 1024 * 1024;
const MAX_TRANSCRIPT_CHARS: usize = 120_000;

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
    calls: Vec<CallFixture>,
}

#[derive(Debug, Clone, Deserialize)]
struct CallFixture {
    date: String,
    period: String,
    format: String,
    url: String,
}

#[derive(Debug, Clone, Copy, Default, Serialize)]
struct MeterSnapshot {
    calls: u64,
    prompt_tokens: u64,
    completion_tokens: u64,
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
    source_format: String,
    transcript_chars: usize,
    ok: bool,
    elapsed_seconds: f64,
    fact_contract_score: u8,
    continuity_contract_score: u8,
    contract_max: u8,
    qa_direct: usize,
    qa_partial: usize,
    qa_evaded: usize,
    review: Option<EarningsTranscriptReview>,
    continuity_outcome: Option<EarningsContinuityOutcome>,
    ledger_items_after: Vec<LedgerItemSnapshot>,
    error: Option<String>,
}

#[derive(Debug, Serialize)]
struct LedgerItemSnapshot {
    kind: ResearchItemKind,
    statement: String,
    status: ResearchItemStatus,
    latest_assessment: String,
    due_at: Option<String>,
    evidence: Vec<String>,
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
    let fixture: Fixture = serde_json::from_str(FIXTURE).context("parse transcript fixture")?;
    let model = std::env::var("HONE_EARNINGS_TRANSCRIPT_MODEL")
        .unwrap_or_else(|_| DEFAULT_MODEL.to_string());
    let selected = std::env::var("HONE_EARNINGS_TRANSCRIPT_COMPANIES")
        .ok()
        .map(|value| {
            value
                .split(',')
                .map(|item| item.trim().to_ascii_uppercase())
                .filter(|item| !item.is_empty())
                .collect::<Vec<_>>()
        });
    let concurrency = std::env::var("HONE_EARNINGS_TRANSCRIPT_CONCURRENCY")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(2)
        .clamp(1, 4);
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
    let expected_samples = companies
        .iter()
        .map(|company| company.calls.len())
        .sum::<usize>();

    let options = LlmRequestOptions {
        max_tokens: Some(3_800),
        temperature: Some(0.2),
        reasoning: Some(json!({"effort": "low"})),
        response_format: Some(json!({"type": "json_object"})),
        ..Default::default()
    };
    let inner =
        OpenRouterProvider::from_config_with_model_and_options(&config, &model, 3_800, options)
            .with_context(|| format!("construct provider for {model}"))?;
    let provider = Arc::new(MeteredProvider {
        inner,
        usage: Mutex::new(MeterSnapshot::default()),
    });
    let client = reqwest::Client::builder()
        .user_agent("honeclaw-transcript-baseline/1.0")
        .timeout(std::time::Duration::from_secs(90))
        .build()
        .context("build source client")?;
    let prices = fetch_model_prices(&client).await.unwrap_or(Value::Null);
    let root = tempfile::tempdir().context("create benchmark profile root")?;
    let started = Instant::now();

    let tasks = futures::stream::iter(companies.into_iter().map(|company| {
        let root = root.path().to_path_buf();
        let client = client.clone();
        let provider = provider.clone();
        let model = model.clone();
        async move { process_company(company, root, client, provider, model).await }
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
        .map(|result| result.fact_contract_score + result.continuity_contract_score)
        .map(u64::from)
        .sum::<u64>();
    let usage = provider.snapshot();
    let failed = results.len().saturating_sub(passed);
    println!(
        "{}",
        serde_json::to_string(&ReplaySummary {
            kind: "summary",
            model: model.clone(),
            samples: results.len(),
            passed,
            failed,
            average_contract_score: if results.is_empty() {
                0.0
            } else {
                total_score as f64 / results.len() as f64
            },
            contract_max: 15,
            total_usage: usage,
            estimated_total_cost_usd: estimate_cost(&prices, &model, usage),
            elapsed_seconds: started.elapsed().as_secs_f64(),
        })?
    );
    if results.len() != expected_samples || failed > 0 {
        anyhow::bail!(
            "transcript baseline failed: {passed}/{} passed, expected {expected_samples} samples",
            results.len()
        );
    }
    Ok(())
}

async fn process_company(
    company: CompanyFixture,
    root: PathBuf,
    client: reqwest::Client,
    provider: Arc<MeteredProvider>,
    model: String,
) -> Vec<ReplayResult> {
    let actor = ActorIdentity::new(
        "benchmark-transcript",
        company.symbol.to_ascii_lowercase(),
        None::<&str>,
    )
    .expect("benchmark actor");
    let storage = CompanyProfileStorage::new(&root);
    let profile_id = create_profile(&storage, &actor, &company).await.profile_id;
    seed_questions(&storage, &actor, &profile_id, &company.seed_questions).await;
    let reviewer = LlmEarningsTranscriptReviewer::new(provider.clone(), model.clone());
    let continuity = LlmEarningsContinuityReconciler::new(
        provider,
        model.clone(),
        CompanyProfileStorage::new(&root),
    );
    let mut results = Vec::new();
    for call in company.calls.clone() {
        let started = Instant::now();
        let replay = replay_one(
            &company,
            &call,
            &actor,
            &profile_id,
            &storage,
            &client,
            &reviewer,
            &continuity,
        )
        .await;
        let result = match replay {
            Ok((
                review,
                outcome,
                transcript_chars,
                fact_score,
                continuity_score,
                ledger_items_after,
            )) => {
                let qa_direct = review
                    .qa_findings
                    .iter()
                    .filter(|item| item.answer_quality == "direct")
                    .count();
                let qa_partial = review
                    .qa_findings
                    .iter()
                    .filter(|item| item.answer_quality == "partial")
                    .count();
                let qa_evaded = review
                    .qa_findings
                    .iter()
                    .filter(|item| item.answer_quality == "evaded")
                    .count();
                ReplayResult {
                    model: model.clone(),
                    symbol: company.symbol.clone(),
                    company_type: company.company_type.clone(),
                    period: call.period,
                    date: call.date,
                    source_url: call.url,
                    source_format: call.format,
                    transcript_chars,
                    ok: fact_score == 10 && continuity_score == 5,
                    elapsed_seconds: started.elapsed().as_secs_f64(),
                    fact_contract_score: fact_score,
                    continuity_contract_score: continuity_score,
                    contract_max: 15,
                    qa_direct,
                    qa_partial,
                    qa_evaded,
                    review: Some(review),
                    continuity_outcome: Some(outcome),
                    ledger_items_after,
                    error: None,
                }
            }
            Err(error) => ReplayResult {
                model: model.clone(),
                symbol: company.symbol.clone(),
                company_type: company.company_type.clone(),
                period: call.period,
                date: call.date,
                source_url: call.url,
                source_format: call.format,
                transcript_chars: 0,
                ok: false,
                elapsed_seconds: started.elapsed().as_secs_f64(),
                fact_contract_score: 0,
                continuity_contract_score: 0,
                contract_max: 15,
                qa_direct: 0,
                qa_partial: 0,
                qa_evaded: 0,
                review: None,
                continuity_outcome: None,
                ledger_items_after: Vec::new(),
                error: Some(error.to_string()),
            },
        };
        results.push(result);
    }
    results
}

#[allow(clippy::too_many_arguments)]
async fn replay_one(
    company: &CompanyFixture,
    call: &CallFixture,
    actor: &ActorIdentity,
    profile_id: &str,
    storage: &CompanyProfileStorage,
    client: &reqwest::Client,
    reviewer: &LlmEarningsTranscriptReviewer,
    continuity: &LlmEarningsContinuityReconciler,
) -> Result<(
    EarningsTranscriptReview,
    EarningsContinuityOutcome,
    usize,
    u8,
    u8,
    Vec<LedgerItemSnapshot>,
)> {
    let bytes = client
        .get(&call.url)
        .send()
        .await
        .with_context(|| format!("fetch {} {} transcript", company.symbol, call.period))?
        .error_for_status()
        .with_context(|| format!("{} {} transcript status", company.symbol, call.period))?
        .bytes()
        .await
        .with_context(|| format!("read {} {} transcript", company.symbol, call.period))?;
    if bytes.len() > MAX_SOURCE_BYTES {
        anyhow::bail!("source document exceeds {} bytes", MAX_SOURCE_BYTES);
    }
    let transcript = extract_transcript(call.format.as_str(), bytes.to_vec()).await?;
    let transcript = normalize_transcript(&transcript);
    let transcript_chars = transcript.chars().count();
    if transcript_chars < 8_000 {
        anyhow::bail!("extracted transcript is too short: {transcript_chars} chars");
    }
    let transcript = transcript
        .chars()
        .take(MAX_TRANSCRIPT_CHARS)
        .collect::<String>();
    let mut event = sample_event(company, call)?;
    let review = reviewer
        .review(&event, &transcript)
        .await
        .context("shared transcript review failed")?;
    if !apply_earnings_transcript_review(&mut event, review.clone(), transcript_chars) {
        anyhow::bail!("transcript review was not applied");
    }
    drop(transcript);
    let before_mainline = storage
        .for_actor(actor)
        .get_profile(profile_id)
        .await
        .ok()
        .flatten()
        .and_then(|profile| profile.section("投资主线"));
    let outcome = continuity
        .reconcile(actor, &event)
        .await
        .context("actor transcript continuity failed")?;
    let after_profile = storage
        .for_actor(actor)
        .get_profile(profile_id)
        .await
        .ok()
        .flatten();
    let after_mainline = after_profile
        .as_ref()
        .and_then(|profile| profile.section("投资主线"));
    let ledger_items_after = after_profile
        .as_ref()
        .map(|profile| {
            profile
                .research_ledger()
                .items
                .into_iter()
                .map(|item| LedgerItemSnapshot {
                    kind: item.kind,
                    statement: item.statement,
                    status: item.status,
                    latest_assessment: item.latest_assessment,
                    due_at: item.due_at,
                    evidence: item.evidence.into_iter().take(2).collect(),
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let fact_score = fact_contract_score(&review, &event, transcript_chars);
    let continuity_score = continuity_contract_score(
        &outcome,
        before_mainline.as_deref(),
        after_mainline.as_deref(),
        after_profile.as_ref(),
        &event,
    );
    Ok((
        review,
        outcome,
        transcript_chars,
        fact_score,
        continuity_score,
        ledger_items_after,
    ))
}

async fn extract_transcript(format: &str, bytes: Vec<u8>) -> Result<String> {
    let format = format.to_string();
    tokio::task::spawn_blocking(move || match format.as_str() {
        "pdf" => std::panic::catch_unwind(|| pdf_extract::extract_text_from_mem(&bytes))
            .map_err(|_| anyhow::anyhow!("PDF parser panicked"))?
            .map_err(|error| anyhow::anyhow!("PDF extraction failed: {error}")),
        "docx" => extract_docx_text(&bytes),
        _ => anyhow::bail!("unsupported transcript format: {format}"),
    })
    .await
    .context("transcript extraction task failed")?
}

fn extract_docx_text(bytes: &[u8]) -> Result<String> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).context("open DOCX archive")?;
    let mut document = archive
        .by_name("word/document.xml")
        .context("DOCX document.xml missing")?;
    let mut xml = String::new();
    document
        .read_to_string(&mut xml)
        .context("read DOCX document.xml")?;
    let mut reader = Reader::from_str(&xml);
    reader.config_mut().trim_text(true);
    let mut output = String::new();
    loop {
        match reader.read_event() {
            Ok(XmlEvent::Text(text)) => {
                let decoded = text.unescape().context("decode DOCX text")?;
                output.push_str(&decoded);
                output.push(' ');
            }
            Ok(XmlEvent::End(tag)) if tag.name().as_ref() == b"w:p" => output.push('\n'),
            Ok(XmlEvent::Empty(tag))
                if matches!(tag.name().as_ref(), b"w:tab" | b"w:br" | b"w:cr") =>
            {
                output.push(' ')
            }
            Ok(XmlEvent::Eof) => break,
            Err(error) => return Err(anyhow::anyhow!("parse DOCX XML: {error}")),
            _ => {}
        }
    }
    Ok(output)
}

fn normalize_transcript(raw: &str) -> String {
    let mut lines = Vec::new();
    for line in raw.lines() {
        let line = line.split_whitespace().collect::<Vec<_>>().join(" ");
        if line.is_empty() || lines.last() == Some(&line) {
            continue;
        }
        lines.push(line);
    }
    lines.join("\n")
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
        .expect("create transcript benchmark profile")
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
            assessment: "电话会回放开始前的投资者核验问题。".to_string(),
            due_at: Some("next earnings call".to_string()),
            evidence: vec![],
        })
        .collect();
    storage
        .for_actor(actor)
        .append_research_event(
            profile_id,
            AppendResearchEventInput {
                event: AppendEventInput {
                    title: "电话会回放初始研究清单".to_string(),
                    event_type: "research_baseline".to_string(),
                    occurred_at: "2025-01-01T00:00:00Z".to_string(),
                    mainline_impact: "baseline".to_string(),
                    changed_sections: vec!["未决问题".to_string()],
                    refs: vec![],
                    what_happened: "建立电话会回放前的研究问题。".to_string(),
                    why_it_matters: "用于验证分析师问答能否解决长期研究问题。".to_string(),
                    mainline_effect: "不修改投资主线。".to_string(),
                    evidence: String::new(),
                    research_log: "benchmark seed".to_string(),
                    follow_up: "下一场电话会逐项核验。".to_string(),
                },
                research_object_key: Some(format!("baseline:{profile_id}")),
                research_updates: updates,
            },
        )
        .await
        .expect("seed transcript questions")
        .expect("seed transcript event");
}

fn sample_event(company: &CompanyFixture, call: &CallFixture) -> Result<MarketEvent> {
    let date = NaiveDate::parse_from_str(&call.date, "%Y-%m-%d")?;
    let occurred_at = Utc
        .from_local_datetime(&date.and_hms_opt(21, 0, 0).context("valid time")?)
        .single()
        .context("UTC datetime")?;
    Ok(MarketEvent {
        id: format!("transcript_replay:{}:{}", company.symbol, call.date),
        kind: EventKind::EarningsCallTranscript,
        severity: Severity::Low,
        symbols: vec![company.symbol.clone()],
        occurred_at,
        title: format!(
            "{} {} earnings call transcript",
            company.symbol, call.period
        ),
        summary: String::new(),
        url: Some(call.url.clone()),
        source: "benchmark.company_ir.transcript".to_string(),
        payload: json!({
            "period": call.period,
            "hone_earnings_research_object_key": format!("transcript:{}:{}", company.symbol, call.period)
        }),
    })
}

fn fact_contract_score(
    review: &EarningsTranscriptReview,
    event: &MarketEvent,
    transcript_chars: usize,
) -> u8 {
    let mut score = 0;
    score += u8::from(matches!(
        review.source_scope.as_str(),
        "prepared_and_qa" | "prepared_only" | "qa_only" | "unclear"
    ));
    score += u8::from(matches!(
        review.management_tone.as_str(),
        "more_confident" | "unchanged" | "more_cautious" | "mixed" | "unclear"
    ));
    score += u8::from((1..=4).contains(&review.prepared_findings.len()));
    score += u8::from((1..=4).contains(&review.qa_findings.len()));
    score += u8::from(review.qa_findings.iter().all(|item| {
        matches!(
            item.answer_quality.as_str(),
            "direct" | "partial" | "evaded" | "unclear"
        ) && !item.question_zh.is_empty()
            && !item.answer_zh.is_empty()
            && !item.evidence_zh.is_empty()
    }));
    score += u8::from(
        review
            .prepared_findings
            .iter()
            .all(|item| !item.finding_zh.is_empty() && !item.evidence_zh.is_empty()),
    );
    score += u8::from(review.commitments.len() <= 4 && review.unresolved_questions.len() <= 4);
    score += u8::from(
        event
            .payload
            .get("earnings_transcript_review_applied")
            .and_then(Value::as_bool)
            == Some(true),
    );
    score += u8::from(
        event
            .payload
            .get("earnings_transcript_source_chars")
            .and_then(Value::as_u64)
            == Some(transcript_chars as u64),
    );
    score += u8::from(event.summary.contains("主动披露") && event.summary.contains("分析师问答"));
    score
}

fn continuity_contract_score(
    outcome: &EarningsContinuityOutcome,
    before_mainline: Option<&str>,
    after_mainline: Option<&str>,
    after_profile: Option<&hone_memory::CompanyProfileDocument>,
    event: &MarketEvent,
) -> u8 {
    let mut score = 0;
    score += u8::from(outcome.checked_existing_items >= 2);
    score += u8::from(before_mainline == after_mainline);
    score += u8::from(outcome.active_questions_after <= 8);
    score += u8::from(outcome.active_commitments_after <= 6);
    score += u8::from(after_profile.is_some_and(|profile| {
        profile.events.iter().any(|profile_event| {
            profile_event.id == outcome.recorded_event_id
                && profile_event.metadata.event_type == "earnings_transcript_reconciliation"
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
