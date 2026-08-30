//! Cached daily macro and AI traffic-light reports.
//!
//! Generation is server-owned and runs at 20:00 in the runtime timezone. Public routes
//! only read durable snapshots; opening a dashboard never starts research.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use axum::Json;
use axum::extract::{Path as AxumPath, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, NaiveDate, Utc};
use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::sync::Mutex;
use tracing::{info, warn};

use crate::routes::public_finance_calendar::fetch_fmp_json_once;
use crate::state::AppState;

const REFRESH_HOUR: u32 = 20;
const REFRESH_MINUTE: u32 = 0;
const STALE_AFTER_HOURS: i64 = 36;
const MODEL_VERSION: &str = "hone-daily-signals-v2";
const DISCLAIMER: &str = "本报告是宏观与产业周期风险框架，不构成个股交易建议。模型评分存在数据滞后、口径差异和估算误差。";
const FRED_CSV_BASE: &str = "https://fred.stlouisfed.org/graph/fredgraph.csv";
const FRED_USER_AGENT: &str = "honeclaw/0.15 (+https://github.com/B-M-Capital-Research/honeclaw)";
const SEC_COMPANYFACTS_BASE: &str = "https://data.sec.gov/api/xbrl/companyfacts";
const INCOMPLETE_RETRY_SECS: i64 = 15 * 60;
/// Retries one report date gets before the worker goes back to waiting for
/// 20:00. Each one re-fetches 17 FRED series and four SEC companyfacts
/// documents, so an uncapped fifteen-minute loop would repeat that ~96 times on
/// a day upstream is unreachable, against endpoints that throttle by user
/// agent. Four covers a typical upstream blip.
const MAX_INCOMPLETE_RETRIES: u32 = 4;
/// Points kept per market-confirmation line. Ten years of daily closes is ~2500
/// observations per series; at this cap the wire cost stays near the existing
/// sparkline budget while a one-year window still draws from ~35 points.
const MARKET_TREND_MAX_POINTS: usize = 400;

static REFRESH_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReportKind {
    Macro,
    Ai,
}

impl ReportKind {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "macro" => Some(Self::Macro),
            "ai" => Some(Self::Ai),
            _ => None,
        }
    }

    fn slug(self) -> &'static str {
        match self {
            Self::Macro => "macro",
            Self::Ai => "ai",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct TrendPoint {
    pub period: String,
    pub value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct EvidencePoint {
    pub label: String,
    pub value: Option<f64>,
    pub display_value: String,
    pub unit: String,
    pub period: Option<String>,
    pub released_at: Option<String>,
    pub fetched_at: String,
    pub source: String,
    pub source_url: String,
    pub provenance: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SignalDimension {
    pub id: String,
    pub label: String,
    pub role: String,
    pub score: Option<f64>,
    pub signal: String,
    pub trend_label: String,
    /// Observation date the score was computed from, e.g. `2026-04-01`.
    ///
    /// The panel renders dimensions, not evidence, so before this field the
    /// only place a date existed was `evidence[0].period` — behind a collapsed
    /// `<details>`. A quarterly row four months old and a daily row one day old
    /// looked identical in the grid while the header printed a single cutoff.
    #[serde(default)]
    pub period: Option<String>,
    /// 日频 / 月频 / 季频, derived from `SeriesSpec::frequency`.
    #[serde(default)]
    pub frequency_label: String,
    /// Days between `period` and the report date. **Display metadata only** —
    /// nothing in the scoring path reads it, and it must stay that way: an age
    /// penalty would silently turn a publication calendar into a verdict about
    /// the economy.
    #[serde(default)]
    pub lag_days: Option<i64>,
    pub reason: String,
    pub threshold: String,
    pub trend: Vec<TrendPoint>,
    pub evidence: Vec<EvidencePoint>,
}

/// The dimension whose observation is the oldest of the ones that scored.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct OldestDimension {
    pub id: String,
    pub label: String,
    pub period: String,
}

/// One index line on the market-confirmation chart.
///
/// Deliberately shaped so it cannot be scored: no `score`, `weight`, `role` or
/// `signal` field exists for the weighted loop in `generate_macro_report` to
/// pick up, even by accident. `SP500` already carries 0.06 of the macro weight;
/// drawing the same "did US equities go up" factor a second time and letting it
/// count again would inflate the health score in every bull market.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct MarketTrendSeries {
    pub id: String,
    pub label: String,
    /// The ETF that tracks this index. FRED carries indices, not ETFs, so the
    /// chart may never be labelled as a QQQ or SPY price.
    pub tracker: String,
    pub as_of: Option<String>,
    pub base_period: Option<String>,
    pub latest_value: Option<f64>,
    pub points: Vec<TrendPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct MetricScore {
    pub id: String,
    pub label: String,
    pub score: Option<f64>,
    pub display_value: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct AiCompanyScore {
    pub symbol: String,
    pub name: String,
    pub score: Option<f64>,
    pub signal: String,
    pub capex: Option<f64>,
    pub capex_growth: Option<f64>,
    pub capex_peak_status: String,
    pub coverage: usize,
    #[serde(default = "default_ai_metric_total")]
    pub metric_total: usize,
    pub metrics: Vec<MetricScore>,
}

fn default_ai_metric_total() -> usize {
    7
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HardwareSignal {
    pub symbol: String,
    pub segment: String,
    pub signal: String,
    pub score: Option<f64>,
    pub price: Option<f64>,
    pub change_percent: Option<f64>,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ReportChange {
    pub label: String,
    pub direction: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SourceLink {
    pub label: String,
    pub url: String,
    pub source_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct DailySignalReport {
    pub kind: String,
    pub title: String,
    pub report_date: String,
    /// Latest observation among the daily market and rate series only.
    ///
    /// Previously this was the max over *all* dimensions, which made it equal
    /// to `data_cutoff` by construction — the client's "print the market day
    /// when it differs from the cutoff" branch could never fire.
    pub market_date: Option<String>,
    /// Newest observation behind the score. Unchanged in meaning, because
    /// snapshots already on disk carry it and are not re-generated; the honest
    /// part of the fix is the two fields below, not a silent redefinition.
    pub data_cutoff: Option<String>,
    /// Oldest observation behind the score. With `data_cutoff` this is the real
    /// answer to "as of when": on 2026-08-28 the pair was 2026-04-01 ~
    /// 2026-08-28, and 26% of the weight sat on the older end.
    #[serde(default)]
    pub data_cutoff_oldest: Option<String>,
    /// Weight-median observation date: the date at which half the scoring
    /// weight is at least as new. This, not `data_cutoff`, is the date the
    /// headline score actually speaks for.
    #[serde(default)]
    pub data_cutoff_weighted: Option<String>,
    #[serde(default)]
    pub oldest_dimension: Option<OldestDimension>,
    pub generated_at: DateTime<Utc>,
    #[serde(alias = "generated_at_beijing")]
    pub generated_at_local: String,
    pub timezone: String,
    pub next_refresh_at: DateTime<Utc>,
    pub model_version: String,
    pub status: String,
    pub score: Option<f64>,
    pub raw_score: Option<f64>,
    pub signal: String,
    pub phase: String,
    pub summary: String,
    pub comparison_yesterday: Option<f64>,
    pub comparison_week: Option<f64>,
    pub changes: Vec<ReportChange>,
    pub dimensions: Vec<SignalDimension>,
    pub company_scores: Vec<AiCompanyScore>,
    pub hardware_signals: Vec<HardwareSignal>,
    /// Display-only index lines. `#[serde(default)]` so snapshots written
    /// before this field still deserialize — no MODEL_VERSION bump, which would
    /// have blanked the score and shown `framework_only` for a day.
    #[serde(default)]
    pub market_trend: Vec<MarketTrendSeries>,
    pub alerts: Vec<String>,
    pub evidence: Vec<EvidencePoint>,
    pub sources: Vec<SourceLink>,
    pub full_report: String,
    pub stale: bool,
    pub disclaimer: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ReportHistoryItem {
    pub report_date: String,
    pub generated_at_local: String,
    pub status: String,
    pub score: Option<f64>,
    pub raw_score: Option<f64>,
    pub signal: String,
    pub phase: String,
    pub summary: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct HistoryQuery {
    limit: Option<usize>,
}

#[derive(Debug, Clone)]
struct SeriesSpec {
    id: &'static str,
    label: &'static str,
    unit: &'static str,
    frequency: usize,
    role: &'static str,
    weight: f64,
}

#[derive(Debug, Clone)]
struct FredSeries {
    spec: SeriesSpec,
    points: Vec<TrendPoint>,
}

#[derive(Debug, Clone, Default)]
struct AiFinancialFact {
    date: Option<String>,
    source: String,
    revenue_growth: Option<f64>,
    gross_margin: Option<f64>,
    operating_margin: Option<f64>,
    free_cash_flow_margin: Option<f64>,
    capex: Option<f64>,
    capex_growth: Option<f64>,
    liquidity: Option<f64>,
    debt_to_revenue: Option<f64>,
}

pub(crate) async fn handle_get_daily_signal(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(kind): AxumPath<String>,
) -> Response {
    if let Err(response) = crate::routes::public::require_public_user(&state, &headers).await {
        return response;
    }
    let Some(kind) = ReportKind::parse(&kind) else {
        return (StatusCode::NOT_FOUND, "unknown report kind").into_response();
    };
    let report = read_latest(&state, kind)
        .await
        .unwrap_or_else(|| framework_report(kind));
    Json(mark_stale(normalize_report_contract(report, kind))).into_response()
}

pub(crate) async fn handle_get_daily_signal_history(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(kind): AxumPath<String>,
    Query(query): Query<HistoryQuery>,
) -> Response {
    if let Err(response) = crate::routes::public::require_public_user(&state, &headers).await {
        return response;
    }
    let Some(kind) = ReportKind::parse(&kind) else {
        return (StatusCode::NOT_FOUND, "unknown report kind").into_response();
    };
    let limit = query.limit.unwrap_or(14).clamp(1, 90);
    Json(json!({ "items": read_history(&state, kind, limit).await })).into_response()
}

/// Compact overview projection of the latest stored report. `None` when no
/// snapshot exists yet; the aggregator renders a waiting card instead.
pub(crate) async fn overview_card(
    state: &AppState,
    kind_slug: &str,
) -> Option<crate::routes::research_overview::OverviewCard> {
    let kind = ReportKind::parse(kind_slug)?;
    let report = read_latest(state, kind).await?;
    let report = mark_stale(normalize_report_contract(report, kind));
    let (title, kicker) = match kind {
        ReportKind::Macro => ("宏观红绿灯", "领先周期判断"),
        ReportKind::Ai => ("AI 红绿灯", "AI 增长可持续性"),
    };
    let mut card = crate::routes::research_overview::OverviewCard::waiting(
        &format!("daily-signal-{}", kind.slug()),
        title,
        kicker,
    );
    card.report_date = Some(report.report_date.clone());
    card.status = report.status.clone();
    card.signal = Some(report.signal.clone());
    card.score = report.score;
    card.summary = Some(crate::routes::research_overview::short_summary(
        &report.summary,
    ));
    card.generated_at = Some(report.generated_at);
    Some(card)
}

/// Generate a missing startup snapshot, then refresh at exactly 20:00 runtime-local time.
pub(crate) async fn daily_signal_worker(state: Arc<AppState>) {
    let today = report_date(Utc::now());
    let missing = [ReportKind::Macro, ReportKind::Ai]
        .into_iter()
        .any(|kind| !latest_is_complete(&state, kind, &today));
    if missing {
        refresh_all(&state, false, 0).await;
    }
    // Retries are counted per report date, so a day upstream is unreachable
    // cannot turn into an all-day poll; the counter resets when the date rolls.
    let mut retry_date = String::new();
    let mut retries = 0_u32;
    loop {
        let now = Utc::now();
        let today = report_date(now);
        if retry_date != today {
            retry_date = today.clone();
            retries = 0;
        }
        let scheduled = next_refresh(now);
        // Whether today's file *exists* is not the question. `refresh_all`
        // writes on every pass, and on a day FRED is unreachable the file it
        // writes is `preserve_success_when_incomplete`'s copy of yesterday —
        // stamped with today's date and `status = "stale"`. The old date test
        // excluded only `framework_only`, so it still caught a machine that had
        // never scored anything, but read those restamped days as finished:
        // once one scored snapshot existed on disk, the fifteen-minute retry
        // could no longer fire. The question is whether the file on disk
        // carries a score this run computed.
        let incomplete = [ReportKind::Macro, ReportKind::Ai]
            .into_iter()
            .any(|kind| !latest_is_complete(&state, kind, &today));
        let next = worker_wake_at(now, incomplete, retries);
        let wait = (next - Utc::now())
            .to_std()
            .unwrap_or_else(|_| Duration::from_secs(60));
        info!(next_refresh = %next, incomplete, retries, "daily signal worker waiting");
        tokio::time::sleep(wait).await;
        let scheduled_run = next == scheduled;
        if !scheduled_run {
            retries += 1;
        }
        refresh_all(&state, scheduled_run, retries).await;
    }
}

async fn refresh_all(state: &AppState, force: bool, retries: u32) {
    let lock = REFRESH_LOCK.get_or_init(|| Mutex::new(()));
    let Ok(_guard) = lock.try_lock() else {
        info!("daily signal refresh already running; duplicate skipped");
        return;
    };

    for kind in [ReportKind::Macro, ReportKind::Ai] {
        let today = report_date(Utc::now());
        if !force && latest_is_complete(state, kind, &today) {
            continue;
        }
        let prior = read_latest(state, kind)
            .await
            .map(|report| normalize_report_contract(report, kind));
        let mut generated = match kind {
            ReportKind::Macro => generate_macro_report(state).await,
            ReportKind::Ai => generate_ai_report(state).await,
        };
        apply_comparisons(
            &mut generated,
            prior.as_ref(),
            &read_history_reports(state, kind, 8).await,
        );
        let mut final_report = preserve_success_when_incomplete(prior, generated);
        // The panel prints this field as 「下次 XX-XX 20:00 更新」. Now that the
        // retry can actually fire, an incomplete day's next wake is fifteen
        // minutes out — until that day's retry budget is spent — so the promise
        // is written from the same function the worker sleeps on. A complete
        // report gets the value it always had.
        final_report.next_refresh_at =
            worker_wake_at(Utc::now(), !has_fresh_score(&final_report), retries);
        if let Err(error) = write_report(state, kind, &final_report).await {
            warn!(kind = kind.slug(), "daily signal write failed: {error}");
        } else {
            info!(kind = kind.slug(), status = %final_report.status, score = ?final_report.score, "daily signal refreshed");
        }
    }
}

async fn generate_macro_report(state: &AppState) -> DailySignalReport {
    let specs = macro_specs();
    let start = (hone_core::local_now().date_naive() - chrono::Duration::days(3660))
        .format("%Y-%m-%d")
        .to_string();
    let fetched_at = Utc::now();
    let mut by_id = fetch_fred_batch(&state.http_client, &specs, &start)
        .await
        .unwrap_or_else(|error| {
            warn!("FRED batch unavailable: {error}");
            HashMap::new()
        });

    // The market-confirmation chart is display-only, so it borrows the SP500
    // series the macro pass already fetched and only pays for what is missing.
    // This has to happen before the loop below, which drains `by_id`.
    let market_trend = {
        let missing = market_trend_specs()
            .into_iter()
            .filter(|spec| !by_id.contains_key(spec.id))
            .collect::<Vec<_>>();
        let mut source: HashMap<String, FredSeries> = if missing.is_empty() {
            HashMap::new()
        } else {
            fetch_fred_batch(&state.http_client, &missing, &start)
                .await
                .unwrap_or_else(|error| {
                    warn!("FRED market-trend batch unavailable: {error}");
                    HashMap::new()
                })
        };
        for spec in market_trend_specs() {
            if let Some(series) = by_id.get(spec.id) {
                source.insert(spec.id.to_string(), series.clone());
            }
        }
        market_trend_series(&source)
    };

    let mut dimensions = Vec::new();
    let mut evidence = Vec::new();
    let mut weighted = 0.0;
    let mut weights = 0.0;
    let mut negative_leaders = 0;
    // Vintage rows come from the dimensions that actually scored, so the dates
    // the header prints and the number it prints them next to are the same set.
    let mut vintages: Vec<VintageRow> = Vec::new();
    for spec in &specs {
        let dimension = if let Some(series) = by_id.remove(spec.id) {
            macro_dimension(&series, fetched_at)
        } else {
            unavailable_dimension(spec, fetched_at)
        };
        if let Some(score) = dimension.score {
            weighted += score * spec.weight;
            weights += spec.weight;
            if spec.role == "leading" && score < 50.0 {
                negative_leaders += 1;
            }
            if let Some(period) = dimension.period.clone() {
                vintages.push(VintageRow {
                    id: dimension.id.clone(),
                    label: dimension.label.clone(),
                    frequency_label: frequency_label(spec.frequency),
                    weight: spec.weight,
                    period,
                });
            }
        }
        evidence.extend(dimension.evidence.clone());
        dimensions.push(dimension);
    }
    let vintage = vintage_spread(&vintages);
    let score = (weights >= 0.55).then(|| round1(weighted / weights));
    let raw_score = score.map(|value| round1((100.0 - value) / 10.0));
    let (signal, phase) = macro_phase(score, negative_leaders);
    let status = coverage_status(
        dimensions
            .iter()
            .filter(|item| item.score.is_some())
            .count(),
        dimensions.len(),
    );
    let summary = macro_summary(score, phase, negative_leaders, &status, &vintage);
    let now = Utc::now();
    let alerts = macro_alerts(&dimensions, raw_score, negative_leaders);
    let mut sources = vec![SourceLink {
        label: "FRED · Federal Reserve Bank of St. Louis".to_string(),
        url: "https://fred.stlouisfed.org/".to_string(),
        source_type: "primary_aggregator".to_string(),
    }];
    sources.extend(
        // A row the fetch missed rides along as an empty placeholder so the
        // panel can name it; it must not also be listed as a source we read.
        market_trend
            .iter()
            .filter(|series| !series.points.is_empty())
            .map(|series| SourceLink {
                label: format!("FRED · {}", series.id),
                url: format!("https://fred.stlouisfed.org/series/{}", series.id),
                source_type: "reference_series".to_string(),
            }),
    );
    DailySignalReport {
        kind: "macro".to_string(),
        title: "宏观红绿灯".to_string(),
        report_date: report_date(now),
        // Only the daily market and rate rows can answer "which market day is
        // this"; the monthly and quarterly rows cannot, and folding them in is
        // what collapsed this field onto `data_cutoff`.
        market_date: vintages
            .iter()
            .filter(|row| row.frequency_label == "日频")
            .map(|row| row.period.clone())
            .max(),
        data_cutoff: vintage.latest.clone(),
        data_cutoff_oldest: vintage.oldest.clone(),
        data_cutoff_weighted: vintage.weighted_median.clone(),
        oldest_dimension: vintage.oldest_dimension.clone(),
        generated_at: now,
        generated_at_local: local_time(now),
        timezone: hone_core::runtime_timezone_name(),
        // Placeholder: `refresh_all` overwrites this with `worker_wake_at`
        // before the report is written. Do not consume `generate_*` directly.
        next_refresh_at: next_refresh(now),
        model_version: MODEL_VERSION.to_string(),
        status,
        score,
        raw_score,
        signal: signal.to_string(),
        phase: phase.to_string(),
        summary: summary.clone(),
        comparison_yesterday: None,
        comparison_week: None,
        changes: vec![],
        dimensions,
        company_scores: vec![],
        hardware_signals: vec![],
        market_trend,
        alerts,
        evidence,
        sources,
        full_report: format!(
            "宏观链条按实际可支配收入/实际工资 → 实际消费 → 制造业生产 → 企业利润/标普确认 → 实际资本开支排序。就业和 GDP 仅作滞后确认。\
             各维度频率不同：日频序列到昨日，月频到上月初，季频到上一季度初，因此「数据截止」是一个区间而不是一天，报告顶部同时给出最新、最旧与加权中位口径日。\
             市场确认图（纳斯达克 100 / 标普 500 指数）只作对照展示，不参与健康分。当前判断：{summary}"
        ),
        stale: false,
        disclaimer: DISCLAIMER.to_string(),
    }
}

fn macro_specs() -> Vec<SeriesSpec> {
    vec![
        SeriesSpec {
            id: "DSPIC96",
            label: "实际可支配收入",
            unit: "十亿美元（2017 链式美元，SAAR）",
            frequency: 12,
            role: "leading",
            weight: 0.10,
        },
        SeriesSpec {
            id: "LES1252881600Q",
            label: "实际周薪",
            unit: "1982–84 年美元",
            frequency: 4,
            role: "leading",
            weight: 0.07,
        },
        SeriesSpec {
            id: "PCEC96",
            label: "实际个人消费支出",
            unit: "十亿美元（2017 链式美元，SAAR）",
            frequency: 12,
            role: "leading",
            weight: 0.10,
        },
        SeriesSpec {
            id: "IPMAN",
            label: "制造业工业产出",
            unit: "指数 2017=100",
            frequency: 12,
            role: "leading",
            weight: 0.08,
        },
        SeriesSpec {
            id: "CP",
            label: "企业利润",
            unit: "十亿美元（SAAR）",
            frequency: 4,
            role: "confirmation",
            weight: 0.07,
        },
        SeriesSpec {
            id: "SP500",
            label: "标普 500 市场确认",
            unit: "指数",
            frequency: 252,
            role: "confirmation",
            weight: 0.06,
        },
        SeriesSpec {
            id: "PNFIC1",
            label: "实际非住宅资本开支",
            unit: "十亿美元（2017 链式美元，SAAR）",
            frequency: 4,
            role: "leading",
            weight: 0.08,
        },
        SeriesSpec {
            id: "PCEPILFE",
            label: "核心 PCE 价格",
            unit: "指数 2017=100",
            frequency: 12,
            role: "risk",
            weight: 0.07,
        },
        SeriesSpec {
            id: "UNRATE",
            label: "失业率",
            unit: "%",
            frequency: 12,
            role: "lagging",
            weight: 0.05,
        },
        SeriesSpec {
            id: "PAYEMS",
            label: "非农就业",
            unit: "千人",
            frequency: 12,
            role: "lagging",
            weight: 0.04,
        },
        SeriesSpec {
            id: "EMRATIO",
            label: "就业人口比",
            unit: "%",
            frequency: 12,
            role: "lagging",
            weight: 0.05,
        },
        SeriesSpec {
            id: "GDPC1",
            label: "实际 GDP",
            unit: "十亿美元（2017 链式美元，SAAR）",
            frequency: 4,
            role: "lagging",
            weight: 0.04,
        },
        SeriesSpec {
            id: "DGS10",
            label: "美国 10 年期国债收益率",
            unit: "%",
            frequency: 252,
            role: "financial_conditions",
            weight: 0.06,
        },
        SeriesSpec {
            id: "DGS30",
            label: "美国 30 年期国债收益率",
            unit: "%",
            frequency: 252,
            role: "financial_conditions",
            weight: 0.05,
        },
        SeriesSpec {
            id: "FEDFUNDS",
            label: "联邦基金有效利率",
            unit: "%",
            frequency: 12,
            role: "financial_conditions",
            weight: 0.05,
        },
        SeriesSpec {
            id: "VIXCLS",
            label: "VIX 波动率指数",
            unit: "指数",
            frequency: 252,
            role: "market_risk",
            weight: 0.03,
        },
    ]
}

/// Fetch descriptors for the market-confirmation chart. **Not part of
/// `macro_specs()` and never to be merged into it.**
///
/// Three separate reasons: `SP500` already holds 0.06 of the macro weight, so a
/// second US-equity row would double-count the same factor; `score_growth` puts
/// a high-beta index in its top band for years at a time, which is a near
/// constant bias rather than a signal; and `macro_specs()` weights are asserted
/// to sum to exactly 1.0, so a weighted entry here fails the contract test —
/// re-weighting all sixteen to make room would be a methodology change and a
/// MODEL_VERSION bump, which blanks every stored snapshot for a day.
///
/// FRED carries indices, never ETFs, so `QQQ` and `SPY` do not exist as series.
/// The labels say "index (what QQQ/SPY tracks)"; they must not be presented as
/// ETF prices, which differ by an order of magnitude and by fees and dividends.
fn market_trend_specs() -> Vec<SeriesSpec> {
    vec![
        SeriesSpec {
            id: "NASDAQ100",
            label: "纳斯达克 100 指数（QQQ 跟踪标的）",
            unit: "指数",
            frequency: 252,
            role: "display_only",
            weight: 0.0,
        },
        SeriesSpec {
            id: "SP500",
            label: "标普 500 指数（SPY 跟踪标的）",
            unit: "指数",
            frequency: 252,
            role: "display_only",
            weight: 0.0,
        },
    ]
}

fn market_trend_tracker(id: &str) -> &'static str {
    match id {
        "NASDAQ100" => "QQQ",
        "SP500" => "SPY",
        _ => "",
    }
}

/// One or two index lines on one shared date axis, rebased by the client.
///
/// The dates are intersected first and then sampled once, so the lines that are
/// drawn land on exactly the same observation dates. That is what makes
/// rebasing to a common base exact instead of approximate, and it means a
/// reader comparing two slopes is comparing the same trading days. FRED grants
/// only a rolling ten years of `SP500` while `NASDAQ100` goes back further, so
/// the shared base is whichever drawn series starts later — taking each series'
/// own first point would silently compare two different baselines. With one
/// line drawn there is no second baseline to get wrong, and the client
/// withdraws the relative reading instead of the chart.
///
/// A series the fetch missed comes back as a named row with no points rather
/// than taking the chart down with it. Dropping the S&P line because the Nasdaq
/// request failed removed the whole section, heading included — the same
/// disappearing-block failure the alerts empty state was written to prevent
/// (「整段消失会被读成 UI 坏了」), and the component's own empty-state sentence
/// could never mount to explain it. Nothing fetched at all is still nothing: no
/// axis, and no row worth naming.
fn market_trend_series(fetched: &HashMap<String, FredSeries>) -> Vec<MarketTrendSeries> {
    let specs = market_trend_specs();
    // Intersected over the series that did arrive, so the lines that are drawn
    // still share one date axis and one base.
    let mut common: Option<Vec<String>> = None;
    for spec in &specs {
        let Some(source) = fetched.get(spec.id) else {
            continue;
        };
        let have = source
            .points
            .iter()
            .map(|point| point.period.as_str())
            .collect::<std::collections::HashSet<_>>();
        common = Some(match common.take() {
            Some(mut common) => {
                common.retain(|period| have.contains(period.as_str()));
                common
            }
            None => source
                .points
                .iter()
                .map(|point| point.period.clone())
                .collect(),
        });
    }
    let Some(mut common) = common else {
        return vec![];
    };
    common.sort();
    common.dedup();
    if common.len() < 2 {
        return vec![];
    }
    let indices = sampled_indices(common.len(), MARKET_TREND_MAX_POINTS);
    let base_period = common.first().cloned();
    specs
        .iter()
        .map(|spec| {
            let points = fetched
                .get(spec.id)
                .map(|source| {
                    let by_period = source
                        .points
                        .iter()
                        .map(|point| (point.period.as_str(), point.value))
                        .collect::<HashMap<_, _>>();
                    indices
                        .iter()
                        .filter_map(|index| {
                            let period = common.get(*index)?;
                            Some(TrendPoint {
                                period: period.clone(),
                                value: *by_period.get(period.as_str())?,
                            })
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            MarketTrendSeries {
                id: spec.id.to_string(),
                label: spec.label.to_string(),
                tracker: market_trend_tracker(spec.id).to_string(),
                // The last *drawn* date, not the series' own last observation:
                // the label has to describe the line the reader is looking at.
                // All three stay empty on a missing row, so a placeholder can
                // never be mistaken for a line that happens to be flat.
                as_of: points.last().map(|point| point.period.clone()),
                base_period: (!points.is_empty()).then(|| base_period.clone()).flatten(),
                latest_value: points.last().map(|point| point.value),
                points,
            }
        })
        .collect()
}

/// One scored dimension's contribution to the report's data vintage.
#[derive(Debug, Clone)]
struct VintageRow {
    id: String,
    label: String,
    frequency_label: &'static str,
    weight: f64,
    period: String,
}

/// How old the data behind one report actually is.
///
/// A sixteen-series composite has no single cutoff date. Measured against
/// 2026-08-28, weight sat at 0 days (0.06), 1 day (0.14), 58 days (0.54) and
/// 149 days (0.26) — four fifths of the score came from observations more than
/// a month old and a quarter from five months back. Printing only the newest of
/// those dates turned a mixed vintage into a claim about yesterday, so the
/// report carries the whole spread instead.
#[derive(Debug, Clone, Default)]
struct VintageSpread {
    latest: Option<String>,
    oldest: Option<String>,
    weighted_median: Option<String>,
    oldest_dimension: Option<OldestDimension>,
    /// `(frequency label, newest period in that bucket, share of scoring weight)`,
    /// newest bucket first.
    buckets: Vec<(&'static str, String, f64)>,
}

fn vintage_spread(rows: &[VintageRow]) -> VintageSpread {
    if rows.is_empty() {
        return VintageSpread::default();
    }
    let oldest_row = rows
        .iter()
        .min_by(|a, b| a.period.cmp(&b.period))
        .expect("checked non-empty");
    let mut by_period = rows.iter().collect::<Vec<_>>();
    by_period.sort_by(|a, b| a.period.cmp(&b.period));
    let total_weight = rows.iter().map(|row| row.weight).sum::<f64>();
    // The date at which half the scoring weight is at least that new.
    let mut running = 0.0;
    let weighted_median = by_period
        .iter()
        .find(|row| {
            running += row.weight;
            running >= total_weight / 2.0
        })
        .map(|row| row.period.clone());

    let mut buckets: Vec<(&'static str, String, f64)> = Vec::new();
    for row in rows {
        match buckets
            .iter_mut()
            .find(|(label, _, _)| *label == row.frequency_label)
        {
            Some((_, period, weight)) => {
                if row.period > *period {
                    *period = row.period.clone();
                }
                *weight += row.weight;
            }
            None => buckets.push((row.frequency_label, row.period.clone(), row.weight)),
        }
    }
    buckets.sort_by(|a, b| b.1.cmp(&a.1));
    if total_weight > f64::EPSILON {
        for bucket in &mut buckets {
            bucket.2 /= total_weight;
        }
    }

    VintageSpread {
        latest: rows.iter().map(|row| row.period.clone()).max(),
        oldest: Some(oldest_row.period.clone()),
        weighted_median,
        oldest_dimension: Some(OldestDimension {
            id: oldest_row.id.clone(),
            label: oldest_row.label.clone(),
            period: oldest_row.period.clone(),
        }),
        buckets,
    }
}

/// The vintage spread as one sentence, for the summary the panel leads with.
fn vintage_sentence(spread: &VintageSpread) -> Option<String> {
    if spread.buckets.is_empty() {
        return None;
    }
    let distribution = spread
        .buckets
        .iter()
        .map(|(label, period, share)| format!("{label} {period}（占 {:.0}%）", share * 100.0))
        .collect::<Vec<_>>()
        .join("、");
    let median = spread
        .weighted_median
        .as_deref()
        .map(|period| format!("；加权中位口径日 {period}"))
        .unwrap_or_default();
    Some(format!("口径分布：{distribution}{median}。"))
}

fn frequency_label(frequency: usize) -> &'static str {
    match frequency {
        value if value >= 200 => "日频",
        value if value >= 40 => "周频",
        value if value >= 10 => "月频",
        value if value >= 3 => "季频",
        _ => "年频",
    }
}

/// Whole days between an observation date and the report date. Display only.
fn observation_lag_days(period: &str, as_of: DateTime<Utc>) -> Option<i64> {
    let period = NaiveDate::parse_from_str(period, "%Y-%m-%d").ok()?;
    Some(
        (hone_core::local_time_at(as_of).date_naive() - period)
            .num_days()
            .max(0),
    )
}

async fn fetch_fred_batch(
    client: &reqwest::Client,
    specs: &[SeriesSpec],
    start: &str,
) -> Result<HashMap<String, FredSeries>, String> {
    let mut set = tokio::task::JoinSet::new();
    for spec in specs.iter().cloned() {
        let client = client.clone();
        let start = start.to_string();
        set.spawn(async move { fetch_fred_series(&client, spec, &start).await });
    }
    let mut output = HashMap::new();
    let mut errors = Vec::new();
    while let Some(result) = set.join_next().await {
        match result {
            Ok(Ok((id, series))) => {
                output.insert(id, series);
            }
            Ok(Err(error)) => errors.push(error),
            Err(error) => errors.push(format!("series task failed: {error}")),
        }
    }
    if output.is_empty() {
        Err(errors.join("; "))
    } else {
        for error in errors {
            warn!("FRED series unavailable: {error}");
        }
        Ok(output)
    }
}

async fn fetch_fred_series(
    client: &reqwest::Client,
    spec: SeriesSpec,
    start: &str,
) -> Result<(String, FredSeries), String> {
    let url = format!("{FRED_CSV_BASE}?id={}&cosd={start}", spec.id);
    let mut last_error = String::new();
    for attempt in 1..=3 {
        match tokio::time::timeout(
            Duration::from_secs(20),
            client
                .get(&url)
                .version(reqwest::Version::HTTP_11)
                .header(reqwest::header::USER_AGENT, FRED_USER_AGENT)
                .send(),
        )
        .await
        {
            Ok(Ok(response)) if response.status().is_success() => {
                let body = response.text().await.map_err(|error| error.to_string())?;
                let series = fred_series_from_csv(spec.clone(), &body)?;
                return Ok((spec.id.to_string(), series));
            }
            Ok(Ok(response)) => last_error = format!("HTTP {}", response.status()),
            Ok(Err(error)) => last_error = error.to_string(),
            Err(_) => last_error = "request timed out".to_string(),
        }
        if attempt < 3 {
            tokio::time::sleep(Duration::from_millis(250 * attempt)).await;
        }
    }
    Err(format!("{}: {last_error}", spec.id))
}

fn fred_series_from_csv(spec: SeriesSpec, body: &str) -> Result<FredSeries, String> {
    let mut lines = body.lines();
    let headers = lines
        .next()
        .ok_or_else(|| format!("{} response has no header", spec.id))?
        .split(',')
        .map(str::trim)
        .collect::<Vec<_>>();
    let value_index = headers
        .iter()
        .position(|header| *header == spec.id)
        .ok_or_else(|| format!("{} response omitted the requested column", spec.id))?;
    let mut points = Vec::new();
    for line in lines {
        let cells = line.trim().split(',').collect::<Vec<_>>();
        let Some(period) = cells.first().filter(|value| !value.is_empty()) else {
            continue;
        };
        let Some(raw) = cells.get(value_index) else {
            continue;
        };
        let Ok(value) = raw.parse::<f64>() else {
            continue;
        };
        if value.is_finite() {
            points.push(TrendPoint {
                period: (*period).to_string(),
                value,
            });
        }
    }
    if points.len() < 2 {
        Err(format!(
            "{} response contained fewer than two observations",
            spec.id
        ))
    } else {
        Ok(FredSeries { spec, points })
    }
}

fn macro_dimension(series: &FredSeries, fetched_at: DateTime<Utc>) -> SignalDimension {
    let latest = series.points.last().expect("checked non-empty");
    let lag = series
        .spec
        .frequency
        .min(series.points.len().saturating_sub(1));
    let prior = &series.points[series.points.len() - 1 - lag];
    let yoy = percent_change(latest.value, prior.value);
    let momentum_lag = if series.spec.frequency >= 200 {
        63
    } else {
        (series.spec.frequency / 2).max(1)
    };
    let older_index = series.points.len().saturating_sub(1 + lag + momentum_lag);
    let older_base_index = older_index.saturating_sub(lag);
    let previous_yoy = if older_index > older_base_index {
        percent_change(
            series.points[older_index].value,
            series.points[older_base_index].value,
        )
    } else {
        None
    };
    let short_lag = if series.spec.frequency >= 200 {
        63
    } else {
        (series.spec.frequency / 4).max(1)
    }
    .min(series.points.len().saturating_sub(1));
    let short_prior = &series.points[series.points.len() - 1 - short_lag];
    let short_change = latest.value - short_prior.value;
    let mut score = score_growth(yoy, previous_yoy);
    if series.spec.id == "UNRATE" {
        score = score.map(|value| 100.0 - value);
    }
    if series.spec.id == "PCEPILFE" {
        score = yoy.map(|inflation| {
            if inflation <= 2.5 {
                82.0
            } else if inflation <= 3.5 {
                62.0
            } else if inflation <= 4.5 {
                42.0
            } else {
                22.0
            }
        });
    }
    if matches!(series.spec.id, "DGS10" | "DGS30") {
        score = Some(rate_health_score(latest.value, short_change));
    }
    if series.spec.id == "FEDFUNDS" {
        score = Some(policy_rate_health_score(latest.value, short_change));
    }
    if series.spec.id == "VIXCLS" {
        score = Some(vix_health_score(latest.value));
    }
    let score = score.map(round1);
    let signal = signal_for_health(score);
    // VIX is deliberately not in this set. `vix_health_score` reads the level
    // and nothing else, so the shared wording below — 「近三个月变化 …」 as the
    // reason and 「且继续上行」 as the threshold — is true of the three rate rows
    // and false of this one: a +1.0 three-month move inside a single band left
    // the score untouched while the card printed 风险上升.
    let is_rate_risk = matches!(series.spec.id, "DGS10" | "DGS30" | "FEDFUNDS");
    let is_volatility_band = series.spec.id == "VIXCLS";
    // Core PCE and unemployment already score inverted above — banded on the
    // inflation level, and 100 minus the growth score. The label did not follow:
    // it took the generic growth branch, so accelerating inflation read 改善 on
    // screen next to a falling health score, and rising unemployment did the
    // same. Nothing here touches a score; this only makes the words agree with
    // the number that is already correct.
    let is_inverted_level = matches!(series.spec.id, "PCEPILFE" | "UNRATE");
    let trend_label = if is_rate_risk {
        // 0.25 is a policy step on a rate quoted in percent; the same 0.25 on a
        // VIX quoted in points is noise, which is the other half of why VIX
        // cannot share this branch.
        if short_change > 0.25 {
            "风险上升"
        } else if short_change < -0.25 {
            "风险缓解"
        } else {
            "持平"
        }
    } else if is_volatility_band {
        // Compare bands, not levels: the score is a step function of the level,
        // so this label moves exactly when the score moves.
        match vix_band(latest.value).cmp(&vix_band(short_prior.value)) {
            std::cmp::Ordering::Greater => "风险上升",
            std::cmp::Ordering::Less => "风险缓解",
            std::cmp::Ordering::Equal => "同档持平",
        }
    } else if is_inverted_level {
        match (yoy, previous_yoy) {
            (Some(current), Some(previous)) if current > previous + 0.15 => "压力上升",
            (Some(current), Some(previous)) if current < previous - 0.15 => "压力缓解",
            (Some(_), Some(_)) => "持平",
            _ => "数据不足",
        }
    } else {
        match (yoy, previous_yoy) {
            (Some(current), Some(previous)) if current > previous + 0.15 => "改善",
            (Some(current), Some(previous)) if current < previous - 0.15 => "走弱",
            (Some(_), Some(_)) => "持平",
            _ => "数据不足",
        }
    };
    let display = yoy
        .map(|value| format!("同比 {value:.1}%"))
        .unwrap_or_else(|| "同比不可得".to_string());
    let frequency = frequency_label(series.spec.frequency);
    let lag_days = observation_lag_days(&latest.period, fetched_at);
    // Every reason opens with the vintage, because a quarterly row and a daily
    // row read identically once the numbers are on screen.
    let as_of = match lag_days {
        Some(days) => format!("口径 {}（{frequency}，滞后 {days} 天）。", latest.period),
        None => format!("口径 {}（{frequency}）。", latest.period),
    };
    SignalDimension {
        id: series.spec.id.to_lowercase(),
        label: series.spec.label.to_string(),
        role: series.spec.role.to_string(),
        score,
        signal: signal.to_string(),
        trend_label: trend_label.to_string(),
        period: Some(latest.period.clone()),
        frequency_label: frequency.to_string(),
        lag_days,
        reason: if is_rate_risk {
            format!(
                "{as_of}最新值 {:.2}{}；近三个月变化 {:+.2}。利率上升按金融条件收紧处理，不按增长加速计分。",
                latest.value, series.spec.unit, short_change
            )
        } else if is_volatility_band {
            format!(
                "{as_of}最新值 {:.2}；近三个月变化 {:+.2}（仅作展示，不进入计分）。健康分只按波动率水平分档（≤15 偏绿、≤20 偏黄、≤30 偏橙、更高偏红）。数据新旧只作展示，不影响本维度得分。",
                latest.value, short_change
            )
        } else if series.spec.id == "PCEPILFE" {
            format!(
                "{as_of}最新值 {:.2}；{display}。健康分按通胀水平分档（≤2.5% 偏绿、≤3.5% 偏黄、≤4.5% 偏橙、更高偏红），通胀走高就是分数走低，不按增长加速计分。数据新旧只作展示，不影响本维度得分。",
                latest.value
            )
        } else if series.spec.id == "UNRATE" {
            format!(
                "{as_of}最新值 {:.2}；{display}。健康分对失业率取反：失业率上行是压力上升、分数下降，不按增长加速计分。数据新旧只作展示，不影响本维度得分。",
                latest.value
            )
        } else {
            format!(
                "{as_of}最新值 {:.2}；{display}。健康分只反映增长方向与动量，不把缺失值记为零。数据新旧只作展示，不影响本维度得分。",
                latest.value
            )
        },
        threshold: if is_rate_risk {
            "收益率或政策利率越高且继续上行，金融条件健康分越低；缺失值不参与总分。".to_string()
        } else if is_volatility_band {
            "VIX 落进更高的波动率档，市场风险健康分越低；缺失值不参与总分。".to_string()
        } else if is_inverted_level {
            "这一维的方向与增长维相反：读数越高、压力越大、健康分越低；缺失值不参与总分。"
                .to_string()
        } else {
            "同比为正且动量改善偏绿；同比为正但动量转弱偏黄；收缩扩散偏橙/红。".to_string()
        },
        trend: downsample(&series.points, 120),
        evidence: vec![EvidencePoint {
            label: series.spec.label.to_string(),
            value: Some(latest.value),
            display_value: format!("{:.2}", latest.value),
            unit: series.spec.unit.to_string(),
            period: Some(latest.period.clone()),
            // Stays `None` on purpose, not as an oversight. `period` is the
            // month or quarter the observation describes; `released_at` would
            // be the day the agency published it, and the two differ by weeks.
            // `fredgraph.csv` carries only (date, value) and sends no
            // `Last-Modified`; the only endpoint with a publication date is
            // `api.stlouisfed.org/fred/series`, which requires a FRED api_key
            // this deployment does not configure. Guessing a release date from
            // the observation date would be a fabricated provenance claim, so
            // the field stays empty until a key exists.
            released_at: None,
            fetched_at: fetched_at.to_rfc3339(),
            source: "FRED / 原始发布机构".to_string(),
            source_url: format!("https://fred.stlouisfed.org/series/{}", series.spec.id),
            provenance: "reported_fact".to_string(),
        }],
    }
}

fn unavailable_dimension(spec: &SeriesSpec, fetched_at: DateTime<Utc>) -> SignalDimension {
    SignalDimension {
        id: spec.id.to_lowercase(),
        label: spec.label.to_string(),
        role: spec.role.to_string(),
        score: None,
        signal: "unknown".to_string(),
        trend_label: "等待数据".to_string(),
        period: None,
        // The publication cadence is known from the spec even when this run
        // fetched nothing, and it is the one thing worth saying about a blank.
        frequency_label: frequency_label(spec.frequency).to_string(),
        lag_days: None,
        reason: "本次抓取未取得有效观测，未以零值代替。".to_string(),
        threshold: "有有效观测后才参与总分。".to_string(),
        trend: vec![],
        evidence: vec![EvidencePoint {
            label: spec.label.to_string(),
            value: None,
            display_value: "—".to_string(),
            unit: spec.unit.to_string(),
            period: None,
            // See `macro_dimension`: no publication date is available without a
            // FRED api_key, and none is invented here.
            released_at: None,
            fetched_at: fetched_at.to_rfc3339(),
            source: "FRED".to_string(),
            source_url: format!("https://fred.stlouisfed.org/series/{}", spec.id),
            provenance: "unavailable".to_string(),
        }],
    }
}

fn score_growth(current: Option<f64>, previous: Option<f64>) -> Option<f64> {
    let current = current?;
    let momentum = previous.map(|value| current - value).unwrap_or(0.0);
    Some(if current >= 3.0 && momentum >= -0.2 {
        88.0
    } else if current > 0.0 && momentum >= 0.0 {
        78.0
    } else if current > 0.0 && momentum > -1.0 {
        62.0
    } else if current > -2.0 {
        42.0
    } else {
        20.0
    })
}

fn rate_health_score(level: f64, three_month_change: f64) -> f64 {
    let base = if level <= 3.5 {
        82.0
    } else if level <= 4.25 {
        67.0
    } else if level <= 5.0 {
        47.0
    } else {
        27.0
    };
    (base - (three_month_change * 16.0).clamp(-12.0, 12.0)).clamp(0.0, 100.0)
}

fn policy_rate_health_score(level: f64, three_month_change: f64) -> f64 {
    let base = if level <= 2.5 {
        82.0
    } else if level <= 4.0 {
        67.0
    } else if level <= 5.5 {
        45.0
    } else {
        25.0
    };
    (base - (three_month_change * 18.0).clamp(-12.0, 12.0)).clamp(0.0, 100.0)
}

/// Which volatility band a VIX level sits in, calmest first.
///
/// Split out so the card's words and its score cross at the same points: the
/// score is a step function of the level, and `macro_dimension` compares bands
/// to decide 风险上升 / 同档持平 / 风险缓解. One table, one set of edges.
fn vix_band(level: f64) -> usize {
    if level <= 15.0 {
        0
    } else if level <= 20.0 {
        1
    } else if level <= 30.0 {
        2
    } else {
        3
    }
}

fn vix_health_score(level: f64) -> f64 {
    [88.0, 72.0, 46.0, 22.0][vix_band(level)]
}

fn macro_phase(score: Option<f64>, negative_leaders: usize) -> (&'static str, &'static str) {
    match score {
        Some(value) if value >= 75.0 && negative_leaders == 0 => ("green", "扩张 / 再加速"),
        Some(value) if value >= 58.0 => ("yellow", "后周期分化 / 消费风险"),
        Some(value) if value >= 40.0 => ("orange", "放缓扩散"),
        Some(_) => ("red", "收缩 / 熊市确认"),
        None => ("unknown", "等待有效数据"),
    }
}

fn macro_summary(
    score: Option<f64>,
    phase: &str,
    negative_leaders: usize,
    status: &str,
    vintage: &VintageSpread,
) -> String {
    match score {
        Some(value) => {
            let spread = vintage_sentence(vintage).unwrap_or_default();
            format!(
                "宏观健康分 {value:.1}，阶段为“{phase}”；{negative_leaders} 个领先维度处于收缩区，数据状态为 {status}。{spread}"
            )
        }
        None => "可用宏观序列不足，正式分数保持空缺，等待下一次成功抓取。".to_string(),
    }
}

/// Alerts are decided per dimension, not per role.
///
/// The previous rule fired only on `role == "leading"`, which is why the single
/// red card in production — DGS30 at 23.6, role `financial_conditions` — could
/// never raise one: the two roles that exist purely to flag risk (`risk`,
/// `market_risk`) were excluded from the only risk rule. The panel showed "2 个
/// 领先维度处于收缩区" and a red card with an empty alert area on the same screen.
///
/// `negative_leaders` is passed in rather than recounted so the sentence in the
/// summary and the alert below can never disagree about the same number.
fn macro_alerts(
    dimensions: &[SignalDimension],
    raw: Option<f64>,
    negative_leaders: usize,
) -> Vec<String> {
    let describe = |dimension: &SignalDimension| {
        let score = dimension
            .score
            .map(|value| format!("{value:.1}"))
            .unwrap_or_else(|| "—".to_string());
        let period = dimension
            .period
            .clone()
            .unwrap_or_else(|| "待定".to_string());
        (score, period)
    };
    let mut alerts = Vec::new();
    let mut named = Vec::new();
    // Only `red` — the same thresholds `signal_for_health` already publishes, so
    // no second scale is introduced. Alerting on every merely weak dimension
    // would put red text on the panel most days and cost the alerts their point.
    for dimension in dimensions.iter().filter(|item| item.signal == "red") {
        let (score, period) = describe(dimension);
        named.push(dimension.id.clone());
        alerts.push(format!(
            "{} 亮红灯：健康分 {score}，口径 {period}（{}）。",
            dimension.label,
            if dimension.frequency_label.is_empty() {
                "频率未知"
            } else {
                &dimension.frequency_label
            }
        ));
    }
    // Core PCE gets one dedicated rule because its score is a step function of
    // year-over-year inflation in `macro_dimension`: <=2.5 → 82, <=3.5 → 62,
    // <=4.5 → 42, else 22. So `score <= 42` is exactly "同比高于 3.5%", and the
    // 42 band is orange — above the red rule above, and previously silent.
    if let Some(dimension) = dimensions.iter().find(|item| item.id == "pcepilfe")
        && !named.contains(&dimension.id)
        && dimension.score.is_some_and(|score| score <= 42.0)
    {
        let (score, period) = describe(dimension);
        alerts.push(format!(
            "核心 PCE 同比高于 3.5%：健康分 {score}，口径 {period}。"
        ));
    }
    if negative_leaders >= 2 {
        alerts.push(format!(
            "{negative_leaders} 个领先维度同步转弱，放缓正在扩散。"
        ));
    }
    if raw.is_some_and(|value| value >= 6.0) {
        alerts.push("宏观原始风险分进入 6/10 以上警戒区。".to_string());
    }
    alerts
}

async fn generate_ai_report(state: &AppState) -> DailySignalReport {
    let companies = [
        ("MSFT", "Microsoft"),
        ("AMZN", "Amazon"),
        ("META", "Meta"),
        ("GOOGL", "Alphabet"),
    ];
    let keys = state.core.config.fmp.effective_key_pool().keys().to_vec();
    let mut facts = if keys.is_empty() {
        HashMap::new()
    } else {
        fetch_ai_financials(state, &keys, &companies).await
    };
    if facts.len() < companies.len() {
        for (symbol, fact) in fetch_sec_ai_financials(state, &companies).await {
            facts.entry(symbol).or_insert(fact);
        }
    }
    let company_scores = companies
        .iter()
        .map(|(symbol, name)| ai_company_score(symbol, name, facts.get(*symbol)))
        .collect::<Vec<_>>();
    let available = company_scores
        .iter()
        .filter_map(|item| item.score)
        .collect::<Vec<_>>();
    let score = (!available.is_empty())
        .then(|| round1(available.iter().sum::<f64>() / available.len() as f64));
    let signal = signal_for_ai(score);
    let status = ai_coverage_status(&company_scores);
    let complete_companies = company_scores
        .iter()
        .filter(|company| company.coverage == company.metric_total)
        .count();
    let coverage_note = if status == "live" {
        format!(
            "{complete_companies}/{} 家公司七项指标完整",
            company_scores.len()
        )
    } else {
        format!(
            "{complete_companies}/{} 家公司七项指标完整，其余缺失项保持空白",
            company_scores.len()
        )
    };
    let phase = match signal {
        "green" => "投入与商业化可持续",
        "yellow" => "增长仍在但回报需验证",
        "red" => "现金流 / 融资压力警戒",
        _ => "等待有效数据",
    };
    let summary = match score {
        Some(value) => format!(
            "AI 可持续健康分 {value:.1}，当前为“{phase}”。数据覆盖：{coverage_note}。本版只使用可稳定核验的云厂商标准财务数据；AI 收入、RPO、订单和硬件兑现因缺少统一可靠口径，已退出评分。"
        ),
        None => "SEC 与已配置行情源本次均未取得足够的有效数据，AI 正式分数保持空缺。".to_string(),
    };
    let dimensions = ai_layers(&company_scores);
    let alerts = ai_alerts(&company_scores);
    let fact_dates = company_scores
        .iter()
        .filter_map(|item| facts.get(&item.symbol).and_then(|fact| fact.date.clone()))
        .collect::<Vec<_>>();
    let now = Utc::now();
    DailySignalReport {
        kind: "ai".to_string(),
        title: "AI 红绿灯".to_string(),
        report_date: report_date(now),
        market_date: None,
        data_cutoff: fact_dates.iter().max().cloned(),
        // Cloud filings land on different quarters, so the AI report has a
        // vintage spread too — smaller, but reported the same way rather than
        // collapsed onto the newest filing.
        data_cutoff_oldest: fact_dates.iter().min().cloned(),
        data_cutoff_weighted: None,
        oldest_dimension: None,
        generated_at: now,
        generated_at_local: local_time(now),
        timezone: hone_core::runtime_timezone_name(),
        // Placeholder: `refresh_all` overwrites this with `worker_wake_at`
        // before the report is written. Do not consume `generate_*` directly.
        next_refresh_at: next_refresh(now),
        model_version: MODEL_VERSION.to_string(),
        status: status.to_string(),
        score,
        raw_score: score,
        signal: signal.to_string(),
        phase: phase.to_string(),
        summary: summary.clone(),
        comparison_yesterday: None,
        comparison_week: None,
        changes: vec![],
        dimensions,
        company_scores,
        hardware_signals: vec![],
        market_trend: vec![],
        alerts,
        evidence: vec![],
        sources: ai_sources(&facts, !keys.is_empty()),
        full_report: format!(
            "AI 框架当前只保留需求旁证、商业化、融资能力和资本开支周期四个可核验层。硬件兑现、AI 收入、RPO、订单与专项商业化口径在没有稳定一手数据前不展示、不计分。当前判断：{summary}"
        ),
        stale: false,
        disclaimer: DISCLAIMER.to_string(),
    }
}

fn ai_coverage_status(company_scores: &[AiCompanyScore]) -> &'static str {
    if company_scores
        .iter()
        .all(|company| company.metric_total > 0 && company.coverage == company.metric_total)
    {
        "live"
    } else if company_scores.iter().any(|company| company.coverage > 0) {
        "partial"
    } else {
        "framework_only"
    }
}

async fn fetch_ai_financials(
    state: &AppState,
    keys: &[String],
    companies: &[(&str, &str)],
) -> HashMap<String, AiFinancialFact> {
    let mut set = tokio::task::JoinSet::new();
    let base = stable_base_url(&state.core.config.fmp.base_url);
    for (index, (symbol, _)) in companies.iter().enumerate() {
        let symbol = (*symbol).to_string();
        let client = state.http_client.clone();
        let key = keys[index % keys.len()].clone();
        let base = base.clone();
        let timeout = state.core.config.fmp.timeout;
        set.spawn(async move {
            let encoded_symbol = utf8_percent_encode(&symbol, NON_ALPHANUMERIC).to_string();
            let encoded_key = utf8_percent_encode(&key, NON_ALPHANUMERIC).to_string();
            let urls = ["income-statement", "cash-flow-statement", "balance-sheet-statement"].map(|endpoint| format!("{base}/stable/{endpoint}?symbol={encoded_symbol}&period=quarter&limit=5&apikey={encoded_key}"));
            let (income, cash, balance) = tokio::join!(
                fetch_fmp_json_once(&client, &urls[0], timeout),
                fetch_fmp_json_once(&client, &urls[1], timeout),
                fetch_fmp_json_once(&client, &urls[2], timeout),
            );
            ai_fact_from_values(
                income.as_ref().ok()?,
                cash.as_ref().ok()?,
                balance.as_ref().ok()?,
            )
            .map(|fact| (symbol, fact))
        });
    }
    let mut output = HashMap::new();
    while let Some(result) = set.join_next().await {
        if let Ok(Some((symbol, fact))) = result {
            output.insert(symbol, fact);
        }
    }
    output
}

async fn fetch_sec_ai_financials(
    state: &AppState,
    companies: &[(&str, &str)],
) -> HashMap<String, AiFinancialFact> {
    const CIKS: [(&str, &str); 4] = [
        ("MSFT", "0000789019"),
        ("AMZN", "0001018724"),
        ("META", "0001326801"),
        ("GOOGL", "0001652044"),
    ];
    let configured_user_agent = state
        .core
        .config
        .event_engine
        .sec_filings
        .enrichment
        .user_agent
        .trim();
    let user_agent = if configured_user_agent.is_empty() {
        "honeclaw daily-signals ops@honeclaw.local"
    } else {
        configured_user_agent
    };
    let requested = companies
        .iter()
        .map(|(symbol, _)| *symbol)
        .collect::<Vec<_>>();
    let mut set = tokio::task::JoinSet::new();
    for (symbol, cik) in CIKS {
        if !requested.contains(&symbol) {
            continue;
        }
        let client = state.http_client.clone();
        let url = format!("{SEC_COMPANYFACTS_BASE}/CIK{cik}.json");
        let user_agent = user_agent.to_string();
        set.spawn(async move {
            let response = tokio::time::timeout(
                Duration::from_secs(25),
                client
                    .get(&url)
                    .version(reqwest::Version::HTTP_11)
                    .header(reqwest::header::USER_AGENT, user_agent)
                    .send(),
            )
            .await
            .map_err(|_| "request timed out".to_string())?
            .map_err(|error| error.to_string())?;
            if !response.status().is_success() {
                return Err(format!("HTTP {}", response.status()));
            }
            let value = response
                .json::<Value>()
                .await
                .map_err(|error| error.to_string())?;
            sec_ai_fact_from_value(&value)
                .map(|fact| (symbol.to_string(), fact))
                .ok_or_else(|| "required Company Facts fields were unavailable".to_string())
        });
    }
    let mut output = HashMap::new();
    while let Some(result) = set.join_next().await {
        match result {
            Ok(Ok((symbol, fact))) => {
                output.insert(symbol, fact);
            }
            Ok(Err(error)) => warn!("SEC Company Facts unavailable: {error}"),
            Err(error) => warn!("SEC Company Facts task failed: {error}"),
        }
    }
    output
}

#[derive(Debug, Clone)]
struct SecFactPoint {
    start: Option<NaiveDate>,
    end: NaiveDate,
    filed: NaiveDate,
    value: f64,
}

impl SecFactPoint {
    fn duration_days(&self) -> Option<i64> {
        self.start.map(|start| (self.end - start).num_days() + 1)
    }
}

fn sec_ai_fact_from_value(value: &Value) -> Option<AiFinancialFact> {
    let revenue_tags = [
        "RevenueFromContractWithCustomerExcludingAssessedTax",
        "Revenues",
        "SalesRevenueNet",
    ];
    let revenue_points = sec_fact_points(value, &revenue_tags);
    let revenue = latest_duration_point(&revenue_points)?;
    let prior_revenue = prior_duration_point(&revenue_points, revenue);
    let gross_profit_points = sec_fact_points(value, &["GrossProfit"]);
    let gross_profit = matching_duration_point(&gross_profit_points, revenue.start?, revenue.end);
    let operating_income_points = sec_fact_points(value, &["OperatingIncomeLoss"]);
    let operating_income =
        matching_duration_point(&operating_income_points, revenue.start?, revenue.end);

    let operating_cash_points =
        sec_fact_points(value, &["NetCashProvidedByUsedInOperatingActivities"]);
    let operating_cash = latest_duration_point(&operating_cash_points);
    let capex_points = sec_fact_points(
        value,
        &[
            "PaymentsToAcquirePropertyPlantAndEquipment",
            "PaymentsToAcquireProductiveAssets",
        ],
    );
    let capex = operating_cash
        .and_then(|cash| matching_duration_point(&capex_points, cash.start?, cash.end));
    let prior_capex = capex.and_then(|point| prior_duration_point(&capex_points, point));
    let cashflow_revenue = operating_cash
        .and_then(|cash| matching_duration_point(&revenue_points, cash.start?, cash.end));

    let cash = latest_instant_point(
        value,
        &[
            "CashAndCashEquivalentsAtCarryingValue",
            "CashCashEquivalentsRestrictedCashAndRestrictedCashEquivalents",
        ],
        revenue.end,
    );
    let investments = latest_instant_point(
        value,
        &["ShortTermInvestments", "MarketableSecuritiesCurrent"],
        revenue.end,
    );
    let current_liabilities = latest_instant_point(value, &["LiabilitiesCurrent"], revenue.end);
    let debt_current = latest_instant_point(
        value,
        &[
            "LongTermDebtCurrent",
            "LongTermDebtAndFinanceLeaseObligationsCurrent",
        ],
        revenue.end,
    );
    let debt_noncurrent = latest_instant_point(
        value,
        &[
            "LongTermDebtNoncurrent",
            "LongTermDebtAndFinanceLeaseObligationsNoncurrent",
        ],
        revenue.end,
    );
    let annualized_revenue = revenue
        .duration_days()
        .filter(|days| *days > 0)
        .map(|days| revenue.value * 365.0 / days as f64);
    let free_cash_flow_margin = match (operating_cash, capex, cashflow_revenue) {
        (Some(cash), Some(capex), Some(period_revenue))
            if period_revenue.value.abs() > f64::EPSILON =>
        {
            Some((cash.value - capex.value.abs()) / period_revenue.value)
        }
        _ => None,
    };
    let liquid_assets = cash.map(|point| point.value).unwrap_or(0.0)
        + investments.map(|point| point.value).unwrap_or(0.0);
    let liquidity = current_liabilities.and_then(|liabilities| {
        (liabilities.value.abs() > f64::EPSILON).then_some(liquid_assets / liabilities.value)
    });
    let total_debt = debt_current.map(|point| point.value).unwrap_or(0.0)
        + debt_noncurrent.map(|point| point.value).unwrap_or(0.0);
    Some(AiFinancialFact {
        date: Some(revenue.end.to_string()),
        source: "SEC EDGAR Company Facts".to_string(),
        revenue_growth: prior_revenue
            .and_then(|prior| ratio_growth(Some(revenue.value), Some(prior.value))),
        gross_margin: gross_profit.and_then(|point| ratio(Some(point.value), Some(revenue.value))),
        operating_margin: operating_income
            .and_then(|point| ratio(Some(point.value), Some(revenue.value))),
        free_cash_flow_margin,
        capex: capex.map(|point| point.value.abs()),
        capex_growth: match (capex, prior_capex) {
            (Some(current), Some(prior)) => {
                ratio_growth(Some(current.value.abs()), Some(prior.value.abs()))
            }
            _ => None,
        },
        liquidity,
        debt_to_revenue: annualized_revenue
            .filter(|revenue| revenue.abs() > f64::EPSILON)
            .map(|revenue| total_debt / revenue),
    })
}

fn sec_fact_points(value: &Value, tags: &[&str]) -> Vec<SecFactPoint> {
    let today = Utc::now().date_naive();
    tags.iter()
        .filter_map(|tag| {
            value
                .pointer(&format!("/facts/us-gaap/{tag}/units/USD"))
                .and_then(Value::as_array)
        })
        .flatten()
        .filter(|row| {
            matches!(
                row.get("form").and_then(Value::as_str),
                Some("10-Q" | "10-K")
            )
        })
        .filter_map(|row| {
            let end = NaiveDate::parse_from_str(row.get("end")?.as_str()?, "%Y-%m-%d").ok()?;
            let filed = NaiveDate::parse_from_str(row.get("filed")?.as_str()?, "%Y-%m-%d").ok()?;
            let start = row
                .get("start")
                .and_then(Value::as_str)
                .and_then(|date| NaiveDate::parse_from_str(date, "%Y-%m-%d").ok());
            let value = row.get("val")?.as_f64().filter(|value| value.is_finite())?;
            (filed <= today).then_some(SecFactPoint {
                start,
                end,
                filed,
                value,
            })
        })
        .collect()
}

fn latest_duration_point(points: &[SecFactPoint]) -> Option<&SecFactPoint> {
    points
        .iter()
        .filter(|point| {
            point
                .duration_days()
                .is_some_and(|days| (60..=400).contains(&days))
        })
        .max_by(|a, b| {
            a.end
                .cmp(&b.end)
                .then_with(|| b.duration_days().cmp(&a.duration_days()))
                .then_with(|| a.filed.cmp(&b.filed))
        })
}

fn matching_duration_point(
    points: &[SecFactPoint],
    start: NaiveDate,
    end: NaiveDate,
) -> Option<&SecFactPoint> {
    points
        .iter()
        .filter(|point| point.start == Some(start) && point.end == end)
        .max_by_key(|point| point.filed)
}

fn prior_duration_point<'a>(
    points: &'a [SecFactPoint],
    current: &SecFactPoint,
) -> Option<&'a SecFactPoint> {
    let duration = current.duration_days()?;
    points
        .iter()
        .filter(|point| {
            let gap = (current.end - point.end).num_days();
            point.end < current.end
                && (330..=400).contains(&gap)
                && point
                    .duration_days()
                    .is_some_and(|days| (days - duration).abs() <= 10)
        })
        .min_by_key(|point| ((current.end - point.end).num_days() - 365).abs())
}

fn latest_instant_point<'a>(
    value: &'a Value,
    tags: &[&str],
    end: NaiveDate,
) -> Option<SecFactPoint> {
    sec_fact_points(value, tags)
        .into_iter()
        .filter(|point| point.end == end)
        .max_by_key(|point| point.filed)
}

fn ai_fact_from_values(income: &Value, cash: &Value, balance: &Value) -> Option<AiFinancialFact> {
    let income_rows = income.as_array()?;
    let current = income_rows.first()?;
    let prior = income_rows.get(4).or_else(|| income_rows.get(1));
    let cash_rows = cash.as_array()?;
    let cash_now = cash_rows.first()?;
    let cash_prior = cash_rows.get(4).or_else(|| cash_rows.get(1));
    let balance_now = balance.as_array()?.first()?;
    let revenue = number(current, "revenue");
    let prior_revenue = prior.and_then(|row| number(row, "revenue"));
    let capex = number(cash_now, "capitalExpenditure").map(f64::abs);
    let prior_capex = cash_prior
        .and_then(|row| number(row, "capitalExpenditure"))
        .map(f64::abs);
    Some(AiFinancialFact {
        date: current
            .get("date")
            .and_then(Value::as_str)
            .map(str::to_string),
        source: "Financial Modeling Prep".to_string(),
        revenue_growth: ratio_growth(revenue, prior_revenue),
        gross_margin: ratio(number(current, "grossProfit"), revenue),
        operating_margin: ratio(number(current, "operatingIncome"), revenue),
        free_cash_flow_margin: ratio(number(cash_now, "freeCashFlow"), revenue),
        capex,
        capex_growth: ratio_growth(capex, prior_capex),
        liquidity: ratio(
            number(balance_now, "cashAndShortTermInvestments"),
            number(balance_now, "totalCurrentLiabilities"),
        ),
        debt_to_revenue: ratio(number(balance_now, "totalDebt"), revenue),
    })
}

fn ai_company_score(symbol: &str, name: &str, fact: Option<&AiFinancialFact>) -> AiCompanyScore {
    let metrics = vec![
        scored_metric(
            "revenue_growth",
            "公司收入增长",
            fact.and_then(|f| f.revenue_growth),
            growth_score,
            true,
        ),
        scored_metric(
            "gross_margin",
            "毛利率",
            fact.and_then(|f| f.gross_margin),
            margin_score,
            true,
        ),
        scored_metric(
            "operating_margin",
            "经营利润率",
            fact.and_then(|f| f.operating_margin),
            margin_score,
            true,
        ),
        scored_metric(
            "fcf_margin",
            "自由现金流率",
            fact.and_then(|f| f.free_cash_flow_margin),
            margin_score,
            true,
        ),
        scored_metric(
            "capex_growth",
            "资本开支增速",
            fact.and_then(|f| f.capex_growth),
            capex_score,
            true,
        ),
        scored_metric(
            "liquidity",
            "流动性缓冲",
            fact.and_then(|f| f.liquidity),
            liquidity_score,
            false,
        ),
        scored_metric(
            "debt_load",
            "债务负担",
            fact.and_then(|f| f.debt_to_revenue),
            debt_score,
            false,
        ),
    ];
    let available = metrics
        .iter()
        .filter_map(|item| item.score)
        .collect::<Vec<_>>();
    let score = (available.len() >= 5)
        .then(|| round1(available.iter().sum::<f64>() / available.len() as f64 * 10.0));
    AiCompanyScore {
        symbol: symbol.to_string(),
        name: name.to_string(),
        score,
        signal: signal_for_ai(score).to_string(),
        capex: fact.and_then(|f| f.capex).map(round1),
        capex_growth: fact
            .and_then(|f| f.capex_growth)
            .map(|value| round1(value * 100.0)),
        capex_peak_status: capex_peak_status(fact).to_string(),
        coverage: available.len(),
        metric_total: metrics.len(),
        metrics,
    }
}

fn metric(id: &str, label: &str, score: Option<f64>, reason: &str) -> MetricScore {
    MetricScore {
        id: id.to_string(),
        label: label.to_string(),
        score,
        display_value: "—".to_string(),
        reason: reason.to_string(),
    }
}
fn scored_metric(
    id: &str,
    label: &str,
    value: Option<f64>,
    score_fn: fn(f64) -> f64,
    percent: bool,
) -> MetricScore {
    match value {
        Some(value) => MetricScore {
            id: id.to_string(),
            label: label.to_string(),
            score: Some(round1(score_fn(value))),
            display_value: if percent {
                format!("{:.1}%", value * 100.0)
            } else {
                format!("{value:.2}x")
            },
            reason: "基于最近季度标准财务报表，属于公司财务底座而非 AI 专项拆分。".to_string(),
        },
        None => metric(id, label, None, "本次未取得有效值，不以零值代替。"),
    }
}

fn growth_score(value: f64) -> f64 {
    if value >= 0.20 {
        9.0
    } else if value >= 0.10 {
        7.5
    } else if value >= 0.0 {
        6.0
    } else if value >= -0.10 {
        4.0
    } else {
        2.0
    }
}
fn margin_score(value: f64) -> f64 {
    (value * 20.0 + 4.0).clamp(0.0, 10.0)
}
fn capex_score(value: f64) -> f64 {
    if value <= 0.10 {
        8.0
    } else if value <= 0.30 {
        7.0
    } else if value <= 0.60 {
        5.5
    } else {
        3.5
    }
}
fn liquidity_score(value: f64) -> f64 {
    (value * 5.0).clamp(0.0, 10.0)
}
fn debt_score(value: f64) -> f64 {
    (10.0 - value * 3.0).clamp(0.0, 10.0)
}
fn capex_peak_status(fact: Option<&AiFinancialFact>) -> &'static str {
    match fact.and_then(|f| f.capex_growth) {
        Some(value) if value > 0.30 => "仍在加速",
        Some(value) if value > 0.05 => "增速放缓",
        Some(value) if value >= -0.05 => "接近平台",
        Some(_) => "同比回落",
        None => "未知",
    }
}

fn ai_layers(companies: &[AiCompanyScore]) -> Vec<SignalDimension> {
    let average = |ids: &[&str]| {
        let values = companies
            .iter()
            .flat_map(|company| company.metrics.iter())
            .filter(|metric| ids.contains(&metric.id.as_str()))
            .filter_map(|metric| metric.score)
            .collect::<Vec<_>>();
        (!values.is_empty())
            .then(|| round1(values.iter().sum::<f64>() / values.len() as f64 * 10.0))
    };
    [
        (
            "demand",
            "需求",
            average(&["revenue_growth"]),
            "只使用公司收入增长作为需求旁证，不把它冒充 AI 收入或订单。",
        ),
        (
            "commercialization",
            "商业化",
            average(&["gross_margin", "operating_margin"]),
            "使用毛利率和经营利润率验证商业化承受力，不推断 AI 专项收入。",
        ),
        (
            "financing",
            "融资能力",
            average(&["fcf_margin", "liquidity", "debt_load"]),
            "自由现金流、流动性和债务负担。",
        ),
        (
            "capex",
            "资本开支周期",
            average(&["capex_growth"]),
            "绝对额、同比增速和峰值状态。",
        ),
        (
            "evidence",
            "专项证据覆盖",
            Some(round1(
                companies
                    .iter()
                    .map(|item| item.coverage as f64 / item.metric_total.max(1) as f64 * 100.0)
                    .sum::<f64>()
                    / companies.len() as f64,
            )),
            "七项标准财务指标中有明确数据来源的覆盖率。",
        ),
    ]
    .into_iter()
    .map(|(id, label, score, reason)| SignalDimension {
        id: id.to_string(),
        label: label.to_string(),
        role: "ai_layer".to_string(),
        score,
        signal: signal_for_ai(score).to_string(),
        trend_label: if score.is_some() {
            "已更新".to_string()
        } else {
            "等待数据".to_string()
        },
        // AI layers aggregate four filings on different quarter ends, so no one
        // observation date belongs to a layer. The report-level cutoff range
        // carries the vintage instead of a made-up per-layer date.
        period: None,
        frequency_label: "季频".to_string(),
        lag_days: None,
        reason: reason.to_string(),
        threshold: "绿 80–100；黄 60–79；红 0–59；无证据为未知。".to_string(),
        trend: vec![],
        evidence: vec![],
    })
    .collect()
}

fn ai_alerts(companies: &[AiCompanyScore]) -> Vec<String> {
    let mut alerts = Vec::new();
    if companies
        .iter()
        .filter(|item| item.score.is_some_and(|score| score < 60.0))
        .count()
        >= 2
    {
        alerts.push("两家以上云厂商财务底座进入红灯区。".to_string());
    }
    alerts
}

fn ai_sources(facts: &HashMap<String, AiFinancialFact>, fmp_configured: bool) -> Vec<SourceLink> {
    let mut sources = Vec::new();
    if facts
        .values()
        .any(|fact| fact.source == "SEC EDGAR Company Facts")
    {
        sources.push(SourceLink {
            label: "SEC EDGAR · 公司标准财报".to_string(),
            url: "https://www.sec.gov/edgar/sec-api-documentation".to_string(),
            source_type: "primary_regulatory_filing".to_string(),
        });
    }
    if fmp_configured {
        sources.push(SourceLink {
            label: "Financial Modeling Prep · 财报与行情".to_string(),
            url: "https://site.financialmodelingprep.com/developer/docs".to_string(),
            source_type: "market_data_provider".to_string(),
        });
    }
    sources
}

fn framework_report(kind: ReportKind) -> DailySignalReport {
    let now = Utc::now();
    let (title, summary, dimensions) = match kind {
        ReportKind::Macro => {
            let fetched = Utc::now();
            (
                "宏观红绿灯",
                "尚无成功快照；框架已就绪，等待数据生成。",
                macro_specs()
                    .iter()
                    .map(|spec| unavailable_dimension(spec, fetched))
                    .collect(),
            )
        }
        ReportKind::Ai => (
            "AI 红绿灯",
            "尚无成功快照；框架已就绪，等待数据生成。",
            vec![],
        ),
    };
    DailySignalReport {
        kind: kind.slug().to_string(),
        title: title.to_string(),
        report_date: report_date(now),
        market_date: None,
        data_cutoff: None,
        data_cutoff_oldest: None,
        data_cutoff_weighted: None,
        oldest_dimension: None,
        generated_at: now,
        generated_at_local: local_time(now),
        timezone: hone_core::runtime_timezone_name(),
        // Served only when no snapshot exists at all — precisely the state the
        // worker retries out of, so promising tomorrow 20:00 would be the one
        // answer that is certainly wrong.
        next_refresh_at: worker_wake_at(now, true, 0),
        model_version: MODEL_VERSION.to_string(),
        status: "framework_only".to_string(),
        score: None,
        raw_score: None,
        signal: "unknown".to_string(),
        phase: "等待有效数据".to_string(),
        summary: summary.to_string(),
        comparison_yesterday: None,
        comparison_week: None,
        changes: vec![],
        dimensions,
        company_scores: vec![],
        hardware_signals: vec![],
        market_trend: vec![],
        alerts: vec![],
        evidence: vec![],
        sources: vec![],
        full_report: summary.to_string(),
        stale: false,
        disclaimer: DISCLAIMER.to_string(),
    }
}

fn normalize_report_contract(mut report: DailySignalReport, kind: ReportKind) -> DailySignalReport {
    // Coverage is a runtime truth, not a model-version migration detail. Older
    // snapshots may already carry the current version while still saying
    // `live` after one or more required company metrics became unavailable.
    if matches!(kind, ReportKind::Ai) {
        let coverage_status = ai_coverage_status(&report.company_scores);
        report.status = coverage_status.to_string();
        if coverage_status != "live"
            && !report.company_scores.is_empty()
            && !report.summary.contains("家云厂商指标完整")
        {
            let complete_companies = report
                .company_scores
                .iter()
                .filter(|company| {
                    company.metric_total > 0 && company.coverage == company.metric_total
                })
                .count();
            report.summary = format!(
                "{} 当前 {complete_companies}/{} 家云厂商指标完整，缺失项保持空白。",
                report.summary,
                report.company_scores.len()
            );
        }
    }
    if report.model_version == MODEL_VERSION {
        return report;
    }
    match kind {
        ReportKind::Macro => {
            let fetched = Utc::now();
            for spec in macro_specs() {
                if report
                    .dimensions
                    .iter()
                    .all(|item| item.id != spec.id.to_lowercase())
                {
                    report
                        .dimensions
                        .push(unavailable_dimension(&spec, fetched));
                }
            }
            report.score = None;
            report.raw_score = None;
            report.signal = "unknown".to_string();
            report.phase = "等待新版数据重算".to_string();
            report.status = "framework_only".to_string();
            report.stale = true;
            report.summary = "旧版宏观分数未包含长期利率、政策利率、就业人口比与 VIX，已停止沿用；等待新版快照重算。".to_string();
        }
        ReportKind::Ai => {
            const KEPT_METRICS: [&str; 7] = [
                "revenue_growth",
                "gross_margin",
                "operating_margin",
                "fcf_margin",
                "capex_growth",
                "liquidity",
                "debt_load",
            ];
            for company in &mut report.company_scores {
                company
                    .metrics
                    .retain(|metric| KEPT_METRICS.contains(&metric.id.as_str()));
                company.metric_total = KEPT_METRICS.len();
                company.coverage = company
                    .metrics
                    .iter()
                    .filter(|metric| metric.score.is_some())
                    .count();
                let available = company
                    .metrics
                    .iter()
                    .filter_map(|metric| metric.score)
                    .collect::<Vec<_>>();
                company.score = (available.len() >= 5)
                    .then(|| round1(available.iter().sum::<f64>() / available.len() as f64 * 10.0));
                company.signal = signal_for_ai(company.score).to_string();
            }
            report.hardware_signals.clear();
            report.dimensions = ai_layers(&report.company_scores);
            let available = report
                .company_scores
                .iter()
                .filter_map(|company| company.score)
                .collect::<Vec<_>>();
            report.score = (!available.is_empty())
                .then(|| round1(available.iter().sum::<f64>() / available.len() as f64));
            report.raw_score = report.score;
            report.signal = signal_for_ai(report.score).to_string();
            report.status = ai_coverage_status(&report.company_scores).to_string();
            report.phase = match report.signal.as_str() {
                "green" => "投入与商业化可持续",
                "yellow" => "增长仍在但回报需验证",
                "red" => "现金流 / 融资压力警戒",
                _ => "等待有效数据",
            }
            .to_string();
            let complete_companies = report
                .company_scores
                .iter()
                .filter(|company| {
                    company.metric_total > 0 && company.coverage == company.metric_total
                })
                .count();
            report.summary = match report.score {
                Some(score) => format!(
                    "AI 可持续健康分 {score:.1}。{complete_companies}/{} 家云厂商达到完整覆盖；本版只保留可稳定核验的标准财务因子，缺失值不按零处理。",
                    report.company_scores.len()
                ),
                None => "可验证财务因子不足，AI 正式分数保持空缺。".to_string(),
            };
        }
    }
    report.model_version = MODEL_VERSION.to_string();
    report
}

fn preserve_success_when_incomplete(
    prior: Option<DailySignalReport>,
    mut fresh: DailySignalReport,
) -> DailySignalReport {
    if fresh.score.is_none() {
        if let Some(prior) = prior.filter(|report| report.score.is_some()) {
            fresh.score = prior.score;
            fresh.raw_score = prior.raw_score;
            fresh.signal = prior.signal;
            fresh.phase = prior.phase;
            fresh.dimensions = prior.dimensions;
            fresh.company_scores = prior.company_scores;
            fresh.hardware_signals = prior.hardware_signals;
            fresh.status = "stale".to_string();
            fresh.stale = true;
            fresh.summary = format!(
                "今日没有取得足以重算的新增数据，阶段与分数沿用上次成功快照。{}",
                fresh.summary
            );
        }
    }
    fresh
}

fn apply_comparisons(
    report: &mut DailySignalReport,
    prior: Option<&DailySignalReport>,
    history: &[DailySignalReport],
) {
    report.comparison_yesterday = delta(report.score, prior.and_then(|item| item.score));
    // 「较一周」counts back through *other* days. A same-day retry has already
    // written today's history file, so taking the seventh element blind would
    // slide the baseline to six days back on exactly the days it gets read.
    let week_ago = history
        .iter()
        .filter(|item| item.report_date != report.report_date)
        .nth(6);
    report.comparison_week = delta(report.score, week_ago.and_then(|item| item.score));
    if let (Some(current), Some(previous)) = (report.score, prior.and_then(|item| item.score)) {
        let change = round1(current - previous);
        let direction = if change > 0.0 {
            "up"
        } else if change < 0.0 {
            "down"
        } else {
            "flat"
        };
        report.changes.push(ReportChange {
            label: "较昨日".to_string(),
            direction: direction.to_string(),
            detail: if change == 0.0 {
                "关键数据未改变，阶段保持不变。".to_string()
            } else {
                format!("健康分变化 {change:+.1}")
            },
        });
    }
}

fn delta(current: Option<f64>, prior: Option<f64>) -> Option<f64> {
    match (current, prior) {
        (Some(a), Some(b)) => Some(round1(a - b)),
        _ => None,
    }
}
fn signal_for_health(score: Option<f64>) -> &'static str {
    match score {
        Some(value) if value >= 75.0 => "green",
        Some(value) if value >= 55.0 => "yellow",
        Some(value) if value >= 40.0 => "orange",
        Some(_) => "red",
        None => "unknown",
    }
}
fn signal_for_ai(score: Option<f64>) -> &'static str {
    match score {
        Some(value) if value >= 80.0 => "green",
        Some(value) if value >= 60.0 => "yellow",
        Some(_) => "red",
        None => "unknown",
    }
}
fn coverage_status(available: usize, total: usize) -> String {
    if available == total {
        "live"
    } else if available > 0 {
        "partial"
    } else {
        "framework_only"
    }
    .to_string()
}
fn percent_change(current: f64, prior: f64) -> Option<f64> {
    (prior.abs() > f64::EPSILON).then(|| (current / prior - 1.0) * 100.0)
}
fn ratio(a: Option<f64>, b: Option<f64>) -> Option<f64> {
    match (a, b) {
        (Some(a), Some(b)) if b.abs() > f64::EPSILON => Some(a / b),
        _ => None,
    }
}
fn ratio_growth(a: Option<f64>, b: Option<f64>) -> Option<f64> {
    ratio(a, b).map(|value| value - 1.0)
}
fn number(value: &Value, key: &str) -> Option<f64> {
    value
        .get(key)
        .and_then(Value::as_f64)
        .filter(|value| value.is_finite())
}
fn round1(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}
/// Indices to keep when thinning a long series, always including the last one.
///
/// `step_by` alone drops up to `step - 1` trailing observations. That is how the
/// SP500 sparkline ended at 2026-08-10 while the evidence line on the same card
/// read 2026-08-28 (VIXCLS was 27 days out), under a dot the component labels
/// as "today". The last point is the one the score was computed from, so it is
/// never the one to drop.
fn sampled_indices(len: usize, max: usize) -> Vec<usize> {
    if len <= max {
        return (0..len).collect();
    }
    let step = (len as f64 / max as f64).ceil() as usize;
    let mut indices = (0..len).step_by(step.max(1)).collect::<Vec<_>>();
    if indices.last() != Some(&(len - 1)) {
        indices.push(len - 1);
    }
    indices
}
fn downsample(points: &[TrendPoint], max: usize) -> Vec<TrendPoint> {
    sampled_indices(points.len(), max)
        .into_iter()
        .map(|index| points[index].clone())
        .collect()
}
fn local_time(now: DateTime<Utc>) -> String {
    hone_core::local_time_at(now)
        .format("%Y-%m-%d %H:%M")
        .to_string()
}
fn report_date(now: DateTime<Utc>) -> String {
    hone_core::local_time_at(now).format("%Y-%m-%d").to_string()
}

fn next_refresh(now: DateTime<Utc>) -> DateTime<Utc> {
    crate::routes::research_store::next_local_refresh(now, REFRESH_HOUR, REFRESH_MINUTE)
}

/// When the worker sleeps until, and the same value the panel promises.
///
/// `retries_used` is what keeps the second branch from becoming a poll: an
/// incomplete pass buys a fifteen-minute retry only while the day's budget
/// lasts, after which a bad day waits for 20:00 like any other.
fn worker_wake_at(now: DateTime<Utc>, incomplete: bool, retries_used: u32) -> DateTime<Utc> {
    let scheduled = next_refresh(now);
    if incomplete && retries_used < MAX_INCOMPLETE_RETRIES {
        scheduled.min(now + chrono::Duration::seconds(INCOMPLETE_RETRY_SECS))
    } else {
        scheduled
    }
}

fn storage_root(state: &AppState) -> PathBuf {
    crate::routes::research_store::data_root(state).join("daily_signals")
}
fn latest_path(state: &AppState, kind: ReportKind) -> PathBuf {
    storage_root(state).join(kind.slug()).join("latest.json")
}
fn history_dir(state: &AppState, kind: ReportKind) -> PathBuf {
    storage_root(state).join(kind.slug()).join("history")
}
/// Whether a report carries a score computed from this run's own fetch.
///
/// `live` and `partial` are the two statuses a scoring pass can produce.
/// `stale` means the numbers were copied off the last good snapshot by
/// `preserve_success_when_incomplete`, and `framework_only` means nothing
/// scored at all; those two are the states the retry exists for.
fn has_fresh_score(report: &DailySignalReport) -> bool {
    report.score.is_some() && matches!(report.status.as_str(), "live" | "partial")
}

/// Whether today's stored snapshot is finished, not merely present.
///
/// The predicate this replaced was `report_date == date && status !=
/// "framework_only"`. That excluded only the case where nothing had ever
/// scored; it did not exclude the `stale` file `preserve_success_when_incomplete`
/// writes on a day the fetch failed — yesterday's numbers under today's date —
/// which is what the disk actually holds on the days the retry is for. The
/// version check is the same question one layer up: `normalize_report_contract`
/// blanks a snapshot from an older `MODEL_VERSION` when it is served, so a
/// snapshot the reader is shown as 「等待新版数据重算」 is not finished either.
fn snapshot_is_complete(report: &DailySignalReport, date: &str) -> bool {
    report.report_date == date && report.model_version == MODEL_VERSION && has_fresh_score(report)
}

fn latest_is_complete(state: &AppState, kind: ReportKind, date: &str) -> bool {
    std::fs::read(latest_path(state, kind))
        .ok()
        .and_then(|bytes| serde_json::from_slice::<DailySignalReport>(&bytes).ok())
        .is_some_and(|report| snapshot_is_complete(&report, date))
}
async fn read_latest(state: &AppState, kind: ReportKind) -> Option<DailySignalReport> {
    let bytes = tokio::fs::read(latest_path(state, kind)).await.ok()?;
    serde_json::from_slice(&bytes).ok()
}

pub(crate) async fn current_macro_report(state: &AppState) -> DailySignalReport {
    read_latest(state, ReportKind::Macro)
        .await
        .map(|report| normalize_report_contract(report, ReportKind::Macro))
        .map(mark_stale)
        .unwrap_or_else(|| framework_report(ReportKind::Macro))
}

async fn write_report(
    state: &AppState,
    kind: ReportKind,
    report: &DailySignalReport,
) -> Result<(), String> {
    let latest = latest_path(state, kind);
    let history = history_dir(state, kind).join(format!("{}.json", report.report_date));
    for path in [&history, &latest] {
        crate::routes::research_store::write_json_atomic(path, report)
            .await
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

async fn read_history(state: &AppState, kind: ReportKind, limit: usize) -> Vec<ReportHistoryItem> {
    read_history_reports(state, kind, limit)
        .await
        .into_iter()
        .map(|item| ReportHistoryItem {
            report_date: item.report_date,
            generated_at_local: item.generated_at_local,
            status: item.status,
            score: item.score,
            raw_score: item.raw_score,
            signal: item.signal,
            phase: item.phase,
            summary: item.summary,
        })
        .collect()
}
async fn read_history_reports(
    state: &AppState,
    kind: ReportKind,
    limit: usize,
) -> Vec<DailySignalReport> {
    let Ok(mut entries) = tokio::fs::read_dir(history_dir(state, kind)).await else {
        return vec![];
    };
    let mut paths = Vec::new();
    while let Ok(Some(entry)) = entries.next_entry().await {
        paths.push(entry.path());
    }
    paths.sort_by(|a, b| b.cmp(a));
    let mut reports = Vec::new();
    for path in paths.into_iter().take(limit) {
        if let Ok(bytes) = tokio::fs::read(path).await {
            if let Ok(report) = serde_json::from_slice(&bytes) {
                reports.push(report);
            }
        }
    }
    reports
}

fn mark_stale(mut report: DailySignalReport) -> DailySignalReport {
    if Utc::now() - report.generated_at > chrono::Duration::hours(STALE_AFTER_HOURS) {
        report.stale = true;
        if report.status != "framework_only" {
            report.status = "stale".to_string();
        }
    }
    report
}
fn stable_base_url(base_url: &str) -> String {
    let mut base = base_url.trim_end_matches('/').to_string();
    for suffix in ["/api/v3", "/api"] {
        if let Some(stripped) = base.strip_suffix(suffix) {
            base = stripped.to_string();
            break;
        }
    }
    base.trim_end_matches('/').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Timelike};

    #[test]
    fn thresholds_keep_macro_and_ai_semantics_separate() {
        assert_eq!(signal_for_health(Some(45.0)), "orange");
        assert_eq!(signal_for_ai(Some(59.9)), "red");
        assert_eq!(signal_for_ai(Some(80.0)), "green");
    }
    #[test]
    fn missing_values_do_not_become_zero() {
        assert_eq!(score_growth(None, Some(1.0)), None);
        assert_eq!(signal_for_ai(None), "unknown");
    }

    #[test]
    fn macro_contract_includes_rates_employment_and_vix() {
        let specs = macro_specs();
        for id in ["DGS10", "DGS30", "FEDFUNDS", "EMRATIO", "VIXCLS"] {
            assert!(specs.iter().any(|spec| spec.id == id), "missing {id}");
        }
        let total_weight = specs.iter().map(|spec| spec.weight).sum::<f64>();
        assert!((total_weight - 1.0).abs() < 1e-9);
    }

    fn dimension(id: &str, signal: &str, score: Option<f64>) -> SignalDimension {
        SignalDimension {
            id: id.to_string(),
            label: id.to_uppercase(),
            role: "financial_conditions".to_string(),
            score,
            signal: signal.to_string(),
            trend_label: "持平".to_string(),
            period: Some("2026-08-27".to_string()),
            frequency_label: "日频".to_string(),
            lag_days: Some(1),
            reason: String::new(),
            threshold: String::new(),
            trend: vec![],
            evidence: vec![],
        }
    }

    #[test]
    fn the_market_confirmation_chart_can_never_be_scored() {
        // Weight is asserted to total exactly 1.0 in the test above; the chart's
        // series must therefore stay outside `macro_specs()`.
        let scored = macro_specs();
        for spec in market_trend_specs() {
            // `SP500` is the one id allowed in both lists: the chart draws a
            // display-only copy of the series `macro_specs` already scores at
            // 0.06 under 市场确认. Every other chart id must be absent from the
            // scored list, or the chart's copy starts feeding the health score
            // — which is what fails here today if `NASDAQ100` is given weight.
            assert!(
                !scored
                    .iter()
                    .any(|item| item.id == spec.id && item.weight > 0.0)
                    || spec.id == "SP500",
                "{} is display-only and must not carry macro weight",
                spec.id
            );
            assert_eq!(spec.weight, 0.0);
            assert_eq!(spec.role, "display_only");
        }
        assert!(
            market_trend_specs()
                .iter()
                .any(|spec| spec.id == "NASDAQ100")
        );
    }

    #[test]
    fn market_trend_lines_share_one_date_axis_and_one_base() {
        let build = |id: &'static str, rows: &[(&str, f64)]| FredSeries {
            spec: SeriesSpec {
                id,
                label: id,
                unit: "指数",
                frequency: 252,
                role: "display_only",
                weight: 0.0,
            },
            points: rows
                .iter()
                .map(|(period, value)| TrendPoint {
                    period: (*period).to_string(),
                    value: *value,
                })
                .collect(),
        };
        let mut fetched = HashMap::new();
        // NASDAQ100 starts earlier and SP500 is missing one mid-week day, which
        // is exactly the shape FRED serves: a rolling ten-year S&P window and a
        // longer Nasdaq one.
        fetched.insert(
            "NASDAQ100".to_string(),
            build(
                "NASDAQ100",
                &[
                    ("2026-08-24", 100.0),
                    ("2026-08-25", 110.0),
                    ("2026-08-26", 120.0),
                    ("2026-08-27", 130.0),
                ],
            ),
        );
        fetched.insert(
            "SP500".to_string(),
            build("SP500", &[("2026-08-25", 200.0), ("2026-08-27", 210.0)]),
        );
        let series = market_trend_series(&fetched);
        assert_eq!(series.len(), 2);
        let periods = series
            .iter()
            .map(|item| {
                item.points
                    .iter()
                    .map(|p| p.period.clone())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        assert_eq!(periods[0], periods[1], "both lines must share a date axis");
        assert_eq!(periods[0], vec!["2026-08-25", "2026-08-27"]);
        for item in &series {
            assert_eq!(item.base_period.as_deref(), Some("2026-08-25"));
            assert_eq!(item.as_of.as_deref(), Some("2026-08-27"));
        }
        assert_eq!(series[0].tracker, "QQQ");
        assert_eq!(series[1].tracker, "SPY");
        // One line alone is still worth drawing. Dropping the S&P row because
        // the Nasdaq fetch failed took the section — heading included — off the
        // panel, which reads as a broken UI rather than a missing series. The
        // missing row survives as a named placeholder with no points, so the
        // client can say which one is gone.
        fetched.remove("SP500");
        let degraded = market_trend_series(&fetched);
        assert_eq!(degraded.len(), 2);
        assert_eq!(degraded[0].id, "NASDAQ100");
        assert_eq!(degraded[0].points.len(), 4);
        assert!(degraded[1].points.is_empty());
        assert_eq!(degraded[1].as_of, None);
        assert_eq!(
            degraded[1].base_period, None,
            "an empty row must not borrow the drawn row's base date"
        );
        // Nothing fetched at all is still nothing to draw.
        fetched.remove("NASDAQ100");
        assert!(market_trend_series(&fetched).is_empty());
    }

    #[test]
    fn thinning_a_series_never_drops_the_latest_observation() {
        let points = (0..100)
            .map(|day| TrendPoint {
                period: format!("2026-01-{:02}", day % 28 + 1),
                value: day as f64,
            })
            .collect::<Vec<_>>();
        let thinned = downsample(&points, 7);
        assert_eq!(
            thinned.last().map(|point| point.value),
            Some(99.0),
            "the observation the score was computed from must survive sampling"
        );
        assert_eq!(sampled_indices(4, 10), vec![0, 1, 2, 3]);
    }

    #[test]
    fn the_data_cutoff_is_a_spread_and_names_its_oldest_row() {
        let rows = vec![
            VintageRow {
                id: "sp500".into(),
                label: "标普 500 市场确认".into(),
                frequency_label: "日频",
                weight: 0.06,
                period: "2026-08-28".into(),
            },
            VintageRow {
                id: "dgs10".into(),
                label: "美国 10 年期国债收益率".into(),
                frequency_label: "日频",
                weight: 0.14,
                period: "2026-08-27".into(),
            },
            VintageRow {
                id: "pcec96".into(),
                label: "实际个人消费支出".into(),
                frequency_label: "月频",
                weight: 0.54,
                period: "2026-07-01".into(),
            },
            VintageRow {
                id: "les1252881600q".into(),
                label: "实际周薪".into(),
                frequency_label: "季频",
                weight: 0.26,
                period: "2026-04-01".into(),
            },
        ];
        let spread = vintage_spread(&rows);
        assert_eq!(spread.latest.as_deref(), Some("2026-08-28"));
        assert_eq!(spread.oldest.as_deref(), Some("2026-04-01"));
        // Four fifths of the weight is at least a month old, so the date the
        // headline speaks for is July, not the 08-28 the header used to print.
        assert_eq!(spread.weighted_median.as_deref(), Some("2026-07-01"));
        assert_eq!(
            spread
                .oldest_dimension
                .as_ref()
                .map(|item| item.label.clone()),
            Some("实际周薪".to_string())
        );
        assert_eq!(spread.buckets.len(), 3);
        assert_eq!(spread.buckets[0].0, "日频");
        assert_eq!(spread.buckets[0].1, "2026-08-28");
        let sentence = vintage_sentence(&spread).expect("buckets present");
        assert!(sentence.contains("季频 2026-04-01"), "{sentence}");
        assert!(sentence.contains("加权中位口径日 2026-07-01"), "{sentence}");
    }

    #[test]
    fn a_red_card_outside_the_leading_roles_still_raises_an_alert() {
        // DGS30 at 23.6 was the only red card in production and could not alert:
        // the rule read `role == "leading"` and its role is financial_conditions.
        let dimensions = vec![
            dimension("dgs30", "red", Some(23.6)),
            dimension("sp500", "green", Some(88.0)),
        ];
        let alerts = macro_alerts(&dimensions, Some(3.9), 2);
        assert!(
            alerts.iter().any(|alert| alert.contains("DGS30")),
            "{alerts:?}"
        );
        assert!(alerts.iter().any(|alert| alert.contains("23.6")));
        assert!(alerts.iter().any(|alert| alert.contains("2026-08-27")));
        // The summary says "2 个领先维度处于收缩区"; the alerts must agree.
        assert!(alerts.iter().any(|alert| alert.contains("2 个领先维度")));
    }

    #[test]
    fn inflation_above_the_band_alerts_without_double_reporting_a_red_card() {
        // PCEPILFE scores 42 for 3.5% < YoY <= 4.5% — orange, not red.
        let orange = vec![dimension("pcepilfe", "orange", Some(42.0))];
        assert!(
            macro_alerts(&orange, None, 0)
                .iter()
                .any(|alert| alert.contains("核心 PCE"))
        );
        // 62 is the 2.5–3.5% band and must stay quiet.
        let calm = vec![dimension("pcepilfe", "yellow", Some(62.0))];
        assert!(macro_alerts(&calm, None, 0).is_empty());
        // 22 is already a red card; it must not be announced twice.
        let red = vec![dimension("pcepilfe", "red", Some(22.0))];
        assert_eq!(macro_alerts(&red, None, 0).len(), 1);
    }

    /// Core PCE and unemployment score inverted — banded on the inflation level,
    /// and 100 minus the growth score — but the label took the generic growth
    /// branch, so production showed 改善 next to 3.3% core inflation while the
    /// health score was falling. The words have to agree with the number.
    #[test]
    fn inverted_dimensions_do_not_call_rising_pressure_an_improvement() {
        let rising_inflation = FredSeries {
            spec: SeriesSpec {
                id: "PCEPILFE",
                label: "核心 PCE 价格",
                unit: "指数 2017=100",
                frequency: 12,
                role: "risk",
                weight: 0.07,
            },
            // 13 monthly points so the year-over-year window exists, rising
            // faster in the last year than the one before it.
            points: (0..26)
                .map(|index| TrendPoint {
                    period: format!("2024-{:02}-01", (index % 12) + 1),
                    value: 100.0 + (index as f64) * (if index > 12 { 0.30 } else { 0.15 }),
                })
                .enumerate()
                .map(|(index, mut point)| {
                    point.period = format!("{}-{:02}-01", 2024 + index / 12, (index % 12) + 1);
                    point
                })
                .collect::<Vec<_>>(),
        };
        let dimension = macro_dimension(
            &rising_inflation,
            Utc.with_ymd_and_hms(2026, 4, 2, 0, 0, 0).unwrap(),
        );
        assert_ne!(
            dimension.trend_label, "改善",
            "accelerating core inflation must not read as an improvement"
        );
        assert!(
            dimension.reason.contains("通胀走高就是分数走低"),
            "reason must state this dimension's own polarity: {}",
            dimension.reason
        );
        assert!(
            !dimension.reason.contains("健康分只反映增长方向与动量"),
            "the growth wording is false for a level-banded dimension: {}",
            dimension.reason
        );
    }

    #[test]
    fn dimension_vintage_is_metadata_and_never_moves_a_score() {
        let spec = SeriesSpec {
            id: "LES1252881600Q",
            label: "实际周薪",
            unit: "1982–84 年美元",
            frequency: 4,
            role: "leading",
            weight: 0.07,
        };
        let points = (0..20)
            .map(|index| TrendPoint {
                period: format!("20{:02}-01-01", 10 + index),
                value: 100.0 + index as f64,
            })
            .collect::<Vec<_>>();
        let fresh = macro_dimension(
            &FredSeries {
                spec: spec.clone(),
                points: points.clone(),
            },
            Utc.with_ymd_and_hms(2029, 1, 2, 0, 0, 0).unwrap(),
        );
        let aged = macro_dimension(
            &FredSeries { spec, points },
            Utc.with_ymd_and_hms(2035, 1, 2, 0, 0, 0).unwrap(),
        );
        assert_eq!(fresh.frequency_label, "季频");
        assert_eq!(fresh.period.as_deref(), Some("2029-01-01"));
        assert!(aged.lag_days.unwrap() > fresh.lag_days.unwrap());
        assert_eq!(
            fresh.score, aged.score,
            "age is display metadata; letting it move the score would turn a \
             publication calendar into a verdict about the economy"
        );
        assert_eq!(fresh.signal, aged.signal);
    }

    #[test]
    fn higher_rates_and_vix_reduce_health() {
        assert!(rate_health_score(3.5, -0.2) > rate_health_score(5.2, 0.4));
        assert!(policy_rate_health_score(2.0, 0.0) > policy_rate_health_score(5.8, 0.3));
        assert!(vix_health_score(14.0) > vix_health_score(35.0));
    }

    /// VIX scores off the level alone, so the shared rate wording described a
    /// rule this dimension does not run: a three-month change in the reason,
    /// 「且继续上行」 in the threshold, and a ±0.25 label that flipped on moves
    /// the score cannot see.
    #[test]
    fn the_vix_card_only_claims_what_its_score_actually_reads() {
        let spec = SeriesSpec {
            id: "VIXCLS",
            label: "VIX 波动率指数",
            unit: "指数",
            frequency: 252,
            role: "market_risk",
            weight: 0.03,
        };
        let build = |latest: f64| FredSeries {
            spec: spec.clone(),
            points: (0..300_i64)
                .map(|index| TrendPoint {
                    period: (NaiveDate::from_ymd_opt(2025, 1, 1).unwrap()
                        + chrono::Duration::days(index))
                    .to_string(),
                    value: if index < 299 { 18.0 } else { latest },
                })
                .collect(),
        };
        let fetched_at = Utc.with_ymd_and_hms(2026, 8, 28, 0, 0, 0).unwrap();
        // 18.0 → 19.0 is a +1.0 three-month change that crosses no band.
        let inside = macro_dimension(&build(19.0), fetched_at);
        assert_eq!(
            inside.score,
            Some(vix_health_score(18.0)),
            "the score did not move, so the label must not say it did"
        );
        assert_eq!(inside.trend_label, "同档持平");
        // Crossing into the next band is what the score does react to.
        let crossed = macro_dimension(&build(22.0), fetched_at);
        assert_eq!(crossed.trend_label, "风险上升");
        assert!(crossed.score < inside.score);
        assert!(
            !inside.threshold.contains("且继续上行"),
            "no line of VIX scoring reads a direction: {}",
            inside.threshold
        );
        assert!(inside.reason.contains("仅作展示"), "{}", inside.reason);
        assert!(!inside.reason.contains("利率上升按金融条件收紧处理"));
    }

    /// `100.0 - score` for unemployment has never had a test. It is the whole
    /// reason a rising jobless rate is not read as an improving dimension, and
    /// it is one edit away from disappearing without a failure.
    #[test]
    fn unemployment_is_scored_and_labelled_upside_down() {
        let series = |id: &'static str, label: &'static str| FredSeries {
            spec: SeriesSpec {
                id,
                label,
                unit: "%",
                frequency: 12,
                role: "lagging",
                weight: 0.05,
            },
            // Forty monthly points, rising throughout and accelerating in the
            // last year: a growth dimension's best case.
            points: {
                let mut value = 4.0;
                (0..40)
                    .map(|index| {
                        let point = TrendPoint {
                            period: format!("{}-{:02}-01", 2023 + index / 12, index % 12 + 1),
                            value,
                        };
                        value *= if index >= 27 { 1.02 } else { 1.005 };
                        point
                    })
                    .collect()
            },
        };
        let fetched_at = Utc.with_ymd_and_hms(2026, 5, 2, 0, 0, 0).unwrap();
        let jobs = macro_dimension(&series("PAYEMS", "非农就业"), fetched_at);
        let unemployment = macro_dimension(&series("UNRATE", "失业率"), fetched_at);
        // Identical numbers, opposite meaning.
        assert_eq!(jobs.trend_label, "改善");
        assert_eq!(unemployment.trend_label, "压力上升");
        assert_eq!(
            unemployment.score,
            jobs.score.map(|score| 100.0 - score),
            "the unemployment card inverts the growth score; it does not share it"
        );
        assert!(unemployment.score.is_some_and(|score| score < 50.0));
        assert!(unemployment.reason.contains("健康分对失业率取反"));
        assert!(!unemployment.reason.contains("健康分只反映增长方向与动量"));
    }

    #[test]
    fn ai_contract_contains_only_seven_verifiable_metrics() {
        let company = ai_company_score("MSFT", "Microsoft", None);
        assert_eq!(company.metric_total, 7);
        assert_eq!(company.metrics.len(), 7);
        assert!(
            company
                .metrics
                .iter()
                .all(|metric| !matches!(metric.id.as_str(), "ai_revenue" | "rpo" | "monetization"))
        );
        assert!(
            ai_layers(&[company])
                .iter()
                .all(|item| item.id != "hardware")
        );
    }

    #[test]
    fn ai_status_requires_every_company_metric_to_be_present() {
        let mut complete = ai_company_score("MSFT", "Microsoft", None);
        complete.coverage = complete.metric_total;
        let mut partial = complete.clone();
        partial.symbol = "AMZN".to_string();
        partial.coverage = partial.metric_total - 1;

        assert_eq!(ai_coverage_status(&[complete.clone()]), "live");
        assert_eq!(ai_coverage_status(&[complete, partial]), "partial");
        assert_eq!(
            ai_coverage_status(&[ai_company_score("MSFT", "Microsoft", None)]),
            "framework_only"
        );
    }
    #[test]
    fn successful_snapshot_is_preserved_when_refresh_has_no_score() {
        let mut prior = framework_report(ReportKind::Macro);
        prior.score = Some(72.0);
        prior.signal = "yellow".to_string();
        let fresh = framework_report(ReportKind::Macro);
        let saved = preserve_success_when_incomplete(Some(prior), fresh);
        assert_eq!(saved.score, Some(72.0));
        assert_eq!(saved.status, "stale");
    }

    #[test]
    fn incomplete_snapshot_retries_before_the_daily_schedule() {
        let now = Utc.with_ymd_and_hms(2026, 8, 10, 10, 0, 0).unwrap();
        assert_eq!(
            worker_wake_at(now, true, 0),
            now + chrono::Duration::seconds(INCOMPLETE_RETRY_SECS)
        );
        assert_eq!(
            hone_core::local_time_at(worker_wake_at(now, false, 0)).hour(),
            20
        );
        // A day upstream is unreachable must not become an all-day poll.
        assert_eq!(
            worker_wake_at(now, true, MAX_INCOMPLETE_RETRIES),
            next_refresh(now)
        );
    }

    /// The test above hands `worker_wake_at` a flag by hand, so it stayed green
    /// while nothing on a running server could set that flag to true. This one
    /// computes `incomplete` the way the worker does — from the snapshot that
    /// was actually written.
    #[test]
    fn a_stale_snapshot_stamped_with_today_is_still_incomplete() {
        let today = "2026-08-10";
        let now = Utc.with_ymd_and_hms(2026, 8, 10, 10, 0, 0).unwrap();
        let mut prior = framework_report(ReportKind::Macro);
        prior.score = Some(72.0);
        prior.status = "live".to_string();
        let mut fresh = framework_report(ReportKind::Macro);
        fresh.report_date = today.to_string();
        let written = preserve_success_when_incomplete(Some(prior), fresh);
        // This is the file a FRED-is-down day leaves on disk: today's date,
        // yesterday's numbers.
        assert_eq!(written.report_date, today);
        assert_eq!(written.status, "stale");
        assert!(
            !snapshot_is_complete(&written, today),
            "a restamped snapshot must not count as today's work being done"
        );
        assert_eq!(
            worker_wake_at(now, !snapshot_is_complete(&written, today), 0),
            now + chrono::Duration::seconds(INCOMPLETE_RETRY_SECS),
            "the retry has to be reachable from a snapshot, not only from a literal"
        );
        // A pass that did score is complete even when some series were missing;
        // otherwise every partial day would refetch every fifteen minutes.
        let mut partial = framework_report(ReportKind::Macro);
        partial.report_date = today.to_string();
        partial.score = Some(61.5);
        partial.status = "partial".to_string();
        assert!(snapshot_is_complete(&partial, today));
        // Yesterday's complete snapshot is not today's, and neither is one the
        // reader would be served as 「等待新版数据重算」.
        assert!(!snapshot_is_complete(&partial, "2026-08-11"));
        let mut old_model = partial.clone();
        old_model.model_version = "hone-daily-signals-v1".to_string();
        assert!(!snapshot_is_complete(&old_model, today));
        // And the no-snapshot-at-all report promises the retry, not 20:00.
        let framework = framework_report(ReportKind::Macro);
        assert_eq!(
            framework.next_refresh_at,
            worker_wake_at(framework.generated_at, true, 0)
        );
    }

    #[test]
    fn the_week_delta_counts_back_through_other_days_only() {
        let mut report = framework_report(ReportKind::Macro);
        report.report_date = "2026-08-10".to_string();
        report.score = Some(70.0);
        // Eight files, newest first, with today's own retry snapshot at the
        // head — the shape the second pass of a degraded day reads.
        let history = (0..8_i32)
            .map(|back| {
                let mut item = framework_report(ReportKind::Macro);
                item.report_date = format!("2026-08-{:02}", 10 - back);
                item.score = Some(f64::from(60 + back));
                item
            })
            .collect::<Vec<_>>();
        apply_comparisons(&mut report, None, &history);
        // 2026-08-03 is seven days back and scores 67; 2026-08-04 scores 66.
        assert_eq!(report.comparison_week, Some(3.0));
    }

    #[test]
    fn fred_single_series_csv_is_parsed_without_batch_archive_handling() {
        let spec = SeriesSpec {
            id: "UNRATE",
            label: "失业率",
            unit: "%",
            frequency: 12,
            role: "lagging",
            weight: 0.05,
        };
        let series = fred_series_from_csv(
            spec,
            "observation_date,UNRATE\n2026-05-01,4.1\n2026-06-01,.\n2026-07-01,4.2\n",
        )
        .expect("valid FRED CSV");
        assert_eq!(series.points.len(), 2);
        assert_eq!(series.points[1].period, "2026-07-01");
        assert_eq!(series.points[1].value, 4.2);
    }

    #[test]
    fn sec_company_facts_builds_financial_baseline_without_market_api_key() {
        let duration = |start: &str, end: &str, value: f64| {
            json!({
                "start": start,
                "end": end,
                "val": value,
                "form": "10-Q",
                "filed": "2026-07-30"
            })
        };
        let instant = |end: &str, value: f64| {
            json!({
                "end": end,
                "val": value,
                "form": "10-Q",
                "filed": "2026-07-30"
            })
        };
        let unit = |rows: Vec<Value>| json!({ "units": { "USD": rows } });
        let value = json!({
            "facts": { "us-gaap": {
                "RevenueFromContractWithCustomerExcludingAssessedTax": unit(vec![
                    duration("2025-04-01", "2025-06-30", 100.0),
                    duration("2026-04-01", "2026-06-30", 120.0)
                ]),
                "GrossProfit": unit(vec![duration("2026-04-01", "2026-06-30", 60.0)]),
                "OperatingIncomeLoss": unit(vec![duration("2026-04-01", "2026-06-30", 30.0)]),
                "NetCashProvidedByUsedInOperatingActivities": unit(vec![duration("2026-04-01", "2026-06-30", 40.0)]),
                "PaymentsToAcquirePropertyPlantAndEquipment": unit(vec![
                    duration("2025-04-01", "2025-06-30", 10.0),
                    duration("2026-04-01", "2026-06-30", 15.0)
                ]),
                "CashAndCashEquivalentsAtCarryingValue": unit(vec![instant("2026-06-30", 50.0)]),
                "ShortTermInvestments": unit(vec![instant("2026-06-30", 25.0)]),
                "LiabilitiesCurrent": unit(vec![instant("2026-06-30", 75.0)]),
                "LongTermDebtCurrent": unit(vec![instant("2026-06-30", 5.0)]),
                "LongTermDebtNoncurrent": unit(vec![instant("2026-06-30", 15.0)])
            }}
        });

        let fact = sec_ai_fact_from_value(&value).expect("valid SEC fact");
        assert_eq!(fact.source, "SEC EDGAR Company Facts");
        assert!(
            fact.revenue_growth
                .is_some_and(|value| (value - 0.2).abs() < 1e-9)
        );
        assert_eq!(fact.gross_margin, Some(0.5));
        assert_eq!(fact.operating_margin, Some(0.25));
        assert_eq!(fact.free_cash_flow_margin, Some(0.20833333333333334));
        assert!(
            fact.capex_growth
                .is_some_and(|value| (value - 0.5).abs() < 1e-9)
        );
        assert_eq!(fact.liquidity, Some(1.0));
        assert!(fact.debt_to_revenue.is_some_and(|value| value < 0.05));
    }
}
