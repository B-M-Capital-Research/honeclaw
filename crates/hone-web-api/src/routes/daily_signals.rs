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
    pub reason: String,
    pub threshold: String,
    pub trend: Vec<TrendPoint>,
    pub evidence: Vec<EvidencePoint>,
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
    pub market_date: Option<String>,
    pub data_cutoff: Option<String>,
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
    if let Err(response) = crate::routes::public::require_public_user(&state, &headers) {
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
    if let Err(response) = crate::routes::public::require_public_user(&state, &headers) {
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
        .any(|kind| !latest_is_date(&state, kind, &today));
    if missing {
        refresh_all(&state, false).await;
    }
    loop {
        let now = Utc::now();
        let scheduled = next_refresh(now);
        let incomplete = [ReportKind::Macro, ReportKind::Ai]
            .into_iter()
            .any(|kind| !latest_is_date(&state, kind, &report_date(now)));
        let next = worker_wake_at(now, incomplete);
        let wait = (next - Utc::now())
            .to_std()
            .unwrap_or_else(|_| Duration::from_secs(60));
        info!(next_refresh = %next, incomplete, "daily signal worker waiting");
        tokio::time::sleep(wait).await;
        refresh_all(&state, next == scheduled).await;
    }
}

async fn refresh_all(state: &AppState, force: bool) {
    let lock = REFRESH_LOCK.get_or_init(|| Mutex::new(()));
    let Ok(_guard) = lock.try_lock() else {
        info!("daily signal refresh already running; duplicate skipped");
        return;
    };

    for kind in [ReportKind::Macro, ReportKind::Ai] {
        let today = report_date(Utc::now());
        if !force && latest_is_date(state, kind, &today) {
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
        let final_report = preserve_success_when_incomplete(prior, generated);
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

    let mut dimensions = Vec::new();
    let mut evidence = Vec::new();
    let mut weighted = 0.0;
    let mut weights = 0.0;
    let mut negative_leaders = 0;
    let mut latest_period: Option<String> = None;
    for spec in specs {
        let dimension = if let Some(series) = by_id.remove(spec.id) {
            macro_dimension(&series, fetched_at, &mut latest_period)
        } else {
            unavailable_dimension(&spec, fetched_at)
        };
        if let Some(score) = dimension.score {
            weighted += score * spec.weight;
            weights += spec.weight;
            if spec.role == "leading" && score < 50.0 {
                negative_leaders += 1;
            }
        }
        evidence.extend(dimension.evidence.clone());
        dimensions.push(dimension);
    }
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
    let summary = macro_summary(score, phase, negative_leaders, &status);
    let now = Utc::now();
    let alerts = macro_alerts(&dimensions, raw_score);
    DailySignalReport {
        kind: "macro".to_string(),
        title: "宏观红绿灯".to_string(),
        report_date: report_date(now),
        market_date: latest_period.clone(),
        data_cutoff: latest_period,
        generated_at: now,
        generated_at_local: local_time(now),
        timezone: hone_core::runtime_timezone_name(),
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
        alerts,
        evidence,
        sources: vec![SourceLink {
            label: "FRED · Federal Reserve Bank of St. Louis".to_string(),
            url: "https://fred.stlouisfed.org/".to_string(),
            source_type: "primary_aggregator".to_string(),
        }],
        full_report: format!(
            "宏观链条按实际可支配收入/实际工资 → 实际消费 → 制造业生产 → 企业利润/标普确认 → 实际资本开支排序。就业和 GDP 仅作滞后确认。当前判断：{summary}"
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

fn macro_dimension(
    series: &FredSeries,
    fetched_at: DateTime<Utc>,
    latest_period: &mut Option<String>,
) -> SignalDimension {
    let latest = series.points.last().expect("checked non-empty");
    if latest_period
        .as_ref()
        .is_none_or(|period| latest.period > *period)
    {
        *latest_period = Some(latest.period.clone());
    }
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
    let is_financial_risk = matches!(series.spec.id, "DGS10" | "DGS30" | "FEDFUNDS" | "VIXCLS");
    let trend_label = if is_financial_risk {
        if short_change > 0.25 {
            "风险上升"
        } else if short_change < -0.25 {
            "风险缓解"
        } else {
            "持平"
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
    SignalDimension {
        id: series.spec.id.to_lowercase(),
        label: series.spec.label.to_string(),
        role: series.spec.role.to_string(),
        score,
        signal: signal.to_string(),
        trend_label: trend_label.to_string(),
        reason: if is_financial_risk {
            format!(
                "最新值 {:.2}{}；近三个月变化 {:+.2}。利率和波动率上升按金融条件收紧处理，不按增长加速计分。",
                latest.value, series.spec.unit, short_change
            )
        } else {
            format!(
                "最新值 {:.2}；{display}。健康分只反映增长方向与动量，不把缺失值记为零。",
                latest.value
            )
        },
        threshold: if is_financial_risk {
            "收益率、政策利率或 VIX 越高且继续上行，金融条件健康分越低；缺失值不参与总分。"
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
        reason: "本次抓取未取得有效观测，未以零值代替。".to_string(),
        threshold: "有有效观测后才参与总分。".to_string(),
        trend: vec![],
        evidence: vec![EvidencePoint {
            label: spec.label.to_string(),
            value: None,
            display_value: "—".to_string(),
            unit: spec.unit.to_string(),
            period: None,
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

fn vix_health_score(level: f64) -> f64 {
    if level <= 15.0 {
        88.0
    } else if level <= 20.0 {
        72.0
    } else if level <= 30.0 {
        46.0
    } else {
        22.0
    }
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

fn macro_summary(score: Option<f64>, phase: &str, negative_leaders: usize, status: &str) -> String {
    match score {
        Some(value) => format!(
            "宏观健康分 {value:.1}，阶段为“{phase}”；{negative_leaders} 个领先维度处于收缩区，数据状态为 {status}。"
        ),
        None => "可用宏观序列不足，正式分数保持空缺，等待下一次成功抓取。".to_string(),
    }
}

fn macro_alerts(dimensions: &[SignalDimension], raw: Option<f64>) -> Vec<String> {
    let mut alerts = Vec::new();
    if raw.is_some_and(|value| value >= 6.0) {
        alerts.push("宏观原始风险分进入 6/10 以上警戒区。".to_string());
    }
    let weak = dimensions
        .iter()
        .filter(|item| item.role == "leading" && item.score.is_some_and(|score| score < 50.0))
        .count();
    if weak >= 3 {
        alerts.push(format!("{weak} 个领先维度同步转弱，放缓正在扩散。"));
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
    let now = Utc::now();
    DailySignalReport {
        kind: "ai".to_string(),
        title: "AI 红绿灯".to_string(),
        report_date: report_date(now),
        market_date: None,
        data_cutoff: company_scores
            .iter()
            .filter_map(|item| facts.get(&item.symbol).and_then(|fact| fact.date.clone()))
            .max(),
        generated_at: now,
        generated_at_local: local_time(now),
        timezone: hone_core::runtime_timezone_name(),
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
        generated_at: now,
        generated_at_local: local_time(now),
        timezone: hone_core::runtime_timezone_name(),
        next_refresh_at: next_refresh(now),
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
    report.comparison_week = delta(report.score, history.get(6).and_then(|item| item.score));
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
fn downsample(points: &[TrendPoint], max: usize) -> Vec<TrendPoint> {
    if points.len() <= max {
        return points.to_vec();
    }
    let step = (points.len() as f64 / max as f64).ceil() as usize;
    points.iter().step_by(step).cloned().collect()
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

fn worker_wake_at(now: DateTime<Utc>, incomplete: bool) -> DateTime<Utc> {
    let scheduled = next_refresh(now);
    if incomplete {
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
fn latest_is_date(state: &AppState, kind: ReportKind, date: &str) -> bool {
    std::fs::read(latest_path(state, kind))
        .ok()
        .and_then(|bytes| serde_json::from_slice::<DailySignalReport>(&bytes).ok())
        .is_some_and(|report| report.report_date == date && report.status != "framework_only")
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

    #[test]
    fn higher_rates_and_vix_reduce_health() {
        assert!(rate_health_score(3.5, -0.2) > rate_health_score(5.2, 0.4));
        assert!(policy_rate_health_score(2.0, 0.0) > policy_rate_health_score(5.8, 0.3));
        assert!(vix_health_score(14.0) > vix_health_score(35.0));
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
            worker_wake_at(now, true),
            now + chrono::Duration::seconds(INCOMPLETE_RETRY_SECS)
        );
        assert_eq!(
            hone_core::local_time_at(worker_wake_at(now, false)).hour(),
            20
        );
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
