//! SEC Company Facts point-in-time historical backfill.
//!
//! This channel exists for decision calibration, not alerts. It reads only SEC
//! submissions and XBRL Company Facts, binds each value to the accession that
//! made it public, and writes idempotent `SecFiling` events into `EventStore`.
//! No LLM prose is admitted as a reported financial fact.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use chrono::{DateTime, NaiveDate, Utc};
use hone_core::config::SecCompanyFactsConfig;
use reqwest::Client;
use serde_json::Value;
use tracing::{info, warn};

use crate::earnings_claim::{EarningsClaimDisposition, EarningsClaimInput, EarningsClaimKind};
use crate::event::{EventKind, MarketEvent, Severity};
use crate::store::EventStore;

const SEC_TICKERS_URL: &str = "https://www.sec.gov/files/company_tickers.json";
const SEC_DATA_BASE: &str = "https://data.sec.gov";
const SEC_ARCHIVES_BASE: &str = "https://www.sec.gov/Archives/edgar/data";
const BACKFILL_SOURCE: &str = "sec.companyfacts.point_in_time";

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SecCompanyFactsBackfillReport {
    pub requested_symbols: usize,
    pub resolved_symbols: usize,
    pub fetched_companies: usize,
    pub generated_events: usize,
    pub inserted_events: usize,
    pub duplicate_events: usize,
    pub failed_symbols: Vec<String>,
    /// Symbols that resolved and fetched successfully but had no admissible
    /// 10-Q/10-K/20-F financial facts under the supported taxonomy/currency
    /// rules. Keeping this distinct from transport failures makes coverage
    /// gaps visible instead of silently looking like an empty success.
    pub no_supported_financial_facts_symbols: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SecCompanyFactsBackfiller {
    client: Client,
    config: SecCompanyFactsConfig,
    tickers_url: String,
    data_base: String,
    archives_base: String,
}

impl SecCompanyFactsBackfiller {
    pub fn new(config: SecCompanyFactsConfig) -> anyhow::Result<Self> {
        let user_agent = config.user_agent.trim();
        if user_agent.is_empty() || !user_agent.contains('@') {
            anyhow::bail!("SEC Company Facts user_agent must include a contact email");
        }
        let client = Client::builder()
            .user_agent(user_agent)
            .timeout(Duration::from_secs(30))
            .build()
            .context("build SEC Company Facts client")?;
        Ok(Self {
            client,
            config,
            tickers_url: SEC_TICKERS_URL.into(),
            data_base: SEC_DATA_BASE.into(),
            archives_base: SEC_ARCHIVES_BASE.into(),
        })
    }

    #[cfg(test)]
    fn with_bases(mut self, tickers_url: &str, data_base: &str, archives_base: &str) -> Self {
        self.tickers_url = tickers_url.into();
        self.data_base = data_base.into();
        self.archives_base = archives_base.into();
        self
    }

    pub async fn fetch_events(&self) -> anyhow::Result<Vec<MarketEvent>> {
        Ok(self.fetch_event_batch().await?.events)
    }

    async fn fetch_event_batch(&self) -> anyhow::Result<SecEventBatch> {
        let ticker_payload = self.fetch_json(&self.tickers_url).await?;
        let ticker_map = ticker_cik_map(&ticker_payload);
        let requested_symbols = normalized_symbols(&self.config.symbols);
        let mut events = Vec::new();
        let mut resolved_symbols = 0;
        let mut fetched_companies = 0;
        let mut failed_symbols = Vec::new();
        let mut no_supported_financial_facts_symbols = Vec::new();
        for symbol in &requested_symbols {
            let Some(cik) = ticker_map.get(symbol).copied() else {
                warn!(symbol, "SEC ticker map did not resolve symbol");
                failed_symbols.push(symbol.clone());
                continue;
            };
            resolved_symbols += 1;
            let submissions_url = format!("{}/submissions/CIK{cik:010}.json", self.data_base);
            let facts_url = format!("{}/api/xbrl/companyfacts/CIK{cik:010}.json", self.data_base);
            let fetched = tokio::try_join!(
                self.fetch_json(&submissions_url),
                self.fetch_json(&facts_url)
            );
            let (submissions, company_facts) = match fetched {
                Ok(payloads) => payloads,
                Err(error) => {
                    warn!(symbol, "SEC company history fetch failed: {error:#}");
                    failed_symbols.push(symbol.clone());
                    continue;
                }
            };
            fetched_companies += 1;
            let company_events = events_from_sec_payloads(
                symbol,
                cik,
                &submissions,
                &company_facts,
                self.config.history_filings.clamp(1, 40),
                &self.archives_base,
            );
            if company_events.is_empty() {
                warn!(
                    symbol,
                    "SEC company fetched but no supported financial facts were admitted"
                );
                no_supported_financial_facts_symbols.push(symbol.clone());
            }
            events.extend(company_events);
        }
        events.sort_by_key(|event| (event.occurred_at, event.id.clone()));
        Ok(SecEventBatch {
            requested_symbols: requested_symbols.len(),
            resolved_symbols,
            fetched_companies,
            failed_symbols,
            no_supported_financial_facts_symbols,
            events,
        })
    }

    pub async fn backfill_into_store(
        &self,
        store: &EventStore,
    ) -> anyhow::Result<SecCompanyFactsBackfillReport> {
        let batch = self.fetch_event_batch().await?;
        let mut report = SecCompanyFactsBackfillReport {
            requested_symbols: batch.requested_symbols,
            resolved_symbols: batch.resolved_symbols,
            fetched_companies: batch.fetched_companies,
            generated_events: batch.events.len(),
            failed_symbols: batch.failed_symbols,
            no_supported_financial_facts_symbols: batch.no_supported_financial_facts_symbols,
            ..Default::default()
        };
        for event in batch.events {
            if store.insert_event(&event)? {
                report.inserted_events += 1;
            } else {
                report.duplicate_events += 1;
            }
        }
        Ok(report)
    }

    async fn fetch_json(&self, url: &str) -> anyhow::Result<Value> {
        self.client
            .get(url)
            .send()
            .await
            .with_context(|| format!("request {url}"))?
            .error_for_status()
            .with_context(|| format!("SEC response {url}"))?
            .json::<Value>()
            .await
            .with_context(|| format!("decode SEC JSON {url}"))
    }
}

#[derive(Debug)]
struct SecEventBatch {
    requested_symbols: usize,
    resolved_symbols: usize,
    fetched_companies: usize,
    failed_symbols: Vec<String>,
    no_supported_financial_facts_symbols: Vec<String>,
    events: Vec<MarketEvent>,
}

pub(crate) fn spawn_sec_company_facts_backfill(
    backfiller: SecCompanyFactsBackfiller,
    store: Arc<EventStore>,
    task_runs_dir: Option<Arc<PathBuf>>,
) {
    let refresh = Duration::from_secs(
        backfiller
            .config
            .refresh_hours
            .clamp(1, 24 * 30)
            .saturating_mul(3_600),
    );
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(refresh);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        loop {
            ticker.tick().await;
            let started_at = Utc::now();
            match tokio::time::timeout(
                Duration::from_secs(180),
                backfiller.backfill_into_store(&store),
            )
            .await
            {
                Ok(Ok(report)) => {
                    info!(
                        requested_symbols = report.requested_symbols,
                        resolved_symbols = report.resolved_symbols,
                        events = report.generated_events,
                        inserted = report.inserted_events,
                        duplicate = report.duplicate_events,
                        no_supported_financial_facts =
                            report.no_supported_financial_facts_symbols.len(),
                        "SEC Company Facts point-in-time backfill complete"
                    );
                    if let Some(dir) = task_runs_dir.as_deref() {
                        hone_core::task_observer::record_ok(
                            dir,
                            "backfill.sec_company_facts",
                            started_at,
                            report.inserted_events as u64,
                        );
                    }
                }
                Ok(Err(error)) => {
                    warn!("SEC Company Facts backfill failed: {error:#}");
                    if let Some(dir) = task_runs_dir.as_deref() {
                        hone_core::task_observer::record_failed(
                            dir,
                            "backfill.sec_company_facts",
                            started_at,
                            &format!("{error:#}"),
                        );
                    }
                }
                Err(_) => warn!("SEC Company Facts backfill timed out after 180s"),
            }
        }
    });
}

fn normalized_symbols(symbols: &[String]) -> Vec<String> {
    let mut values = symbols
        .iter()
        .map(|symbol| symbol.trim().trim_start_matches('$').to_ascii_uppercase())
        .filter(|symbol| !symbol.is_empty())
        .collect::<Vec<_>>();
    values.sort();
    values.dedup();
    values
}

fn ticker_cik_map(payload: &Value) -> HashMap<String, u64> {
    payload
        .as_object()
        .into_iter()
        .flat_map(|object| object.values())
        .filter_map(|entry| {
            Some((
                entry.get("ticker")?.as_str()?.trim().to_ascii_uppercase(),
                entry.get("cik_str")?.as_u64()?,
            ))
        })
        .collect()
}

#[derive(Debug, Clone)]
struct FilingRecord {
    accession: String,
    form: String,
    filing_date: String,
    accepted_at: DateTime<Utc>,
    report_date: String,
    primary_document: String,
}

fn filings_from_submissions(payload: &Value, limit: usize) -> Vec<FilingRecord> {
    let Some(recent) = payload.pointer("/filings/recent") else {
        return Vec::new();
    };
    let arrays = |key: &str| recent.get(key).and_then(Value::as_array);
    let (Some(accessions), Some(forms), Some(filing_dates), Some(report_dates), Some(documents)) = (
        arrays("accessionNumber"),
        arrays("form"),
        arrays("filingDate"),
        arrays("reportDate"),
        arrays("primaryDocument"),
    ) else {
        return Vec::new();
    };
    let accepted = arrays("acceptanceDateTime");
    accessions
        .iter()
        .enumerate()
        .filter_map(|(index, accession)| {
            let form = forms.get(index)?.as_str()?;
            // Foreign private issuers file annual financial statements on
            // Form 20-F.  Keep 6-K excluded here: unlike 10-Q/10-K/20-F it
            // can represent many unrelated current reports, so admitting it
            // without a statement-level classifier would crowd out audited
            // filings and create false financial observations.
            if !matches!(form, "10-Q" | "10-K" | "20-F") {
                return None;
            }
            let filing_date = filing_dates.get(index)?.as_str()?.to_string();
            Some(FilingRecord {
                accession: accession.as_str()?.to_string(),
                form: form.to_string(),
                accepted_at: accepted
                    .and_then(|values| values.get(index))
                    .and_then(Value::as_str)
                    .and_then(parse_sec_datetime)
                    .or_else(|| parse_sec_date(&filing_date))?,
                filing_date,
                report_date: report_dates
                    .get(index)
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                primary_document: documents.get(index)?.as_str()?.to_string(),
            })
        })
        .take(limit)
        .collect()
}

fn parse_sec_datetime(value: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|value| value.with_timezone(&Utc))
        .or_else(|| {
            chrono::NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S%.fZ")
                .ok()
                .map(|value| value.and_utc())
        })
}

fn parse_sec_date(value: &str) -> Option<DateTime<Utc>> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .ok()?
        .and_hms_opt(0, 0, 0)
        .map(|value| value.and_utc())
}

#[derive(Debug, Clone, Copy)]
enum FactShape {
    DurationQuarter,
    DurationYtd,
    Instant,
}

#[derive(Debug, Clone, Copy)]
struct MetricSpec {
    metric_id: &'static str,
    label_zh: &'static str,
    shape: FactShape,
    us_gaap_tags: &'static [&'static str],
    ifrs_tags: &'static [&'static str],
}

const BASE_METRICS: &[MetricSpec] = &[
    MetricSpec {
        metric_id: "revenue",
        label_zh: "营业收入",
        shape: FactShape::DurationQuarter,
        us_gaap_tags: &[
            "RevenueFromContractWithCustomerExcludingAssessedTax",
            "Revenues",
            "SalesRevenueNet",
        ],
        ifrs_tags: &["RevenueFromContractsWithCustomers", "Revenue"],
    },
    MetricSpec {
        metric_id: "inventory",
        label_zh: "存货",
        shape: FactShape::Instant,
        us_gaap_tags: &["InventoryNet"],
        ifrs_tags: &["Inventories"],
    },
    MetricSpec {
        metric_id: "accounts_receivable",
        label_zh: "应收账款",
        shape: FactShape::Instant,
        us_gaap_tags: &[
            "AccountsReceivableNetCurrent",
            "AccountsNotesAndLoansReceivableNetCurrent",
        ],
        ifrs_tags: &["TradeAndOtherCurrentReceivables", "CurrentTradeReceivables"],
    },
    MetricSpec {
        metric_id: "accounts_payable",
        label_zh: "应付账款",
        shape: FactShape::Instant,
        us_gaap_tags: &["AccountsPayableCurrent"],
        ifrs_tags: &[
            "TradeAndOtherCurrentPayables",
            "TradeAndOtherCurrentPayablesToTradeSuppliers",
        ],
    },
    MetricSpec {
        metric_id: "capital_expenditure",
        label_zh: "购建固定资产现金支出",
        shape: FactShape::DurationYtd,
        us_gaap_tags: &["PaymentsToAcquirePropertyPlantAndEquipment"],
        ifrs_tags: &[
            "PurchaseOfPropertyPlantAndEquipmentClassifiedAsInvestingActivities",
            "PurchaseOfPropertyPlantAndEquipmentIntangibleAssetsOtherThanGoodwillInvestmentPropertyAndOtherNoncurrentAssets",
        ],
    },
];

/// Kept in a separate immutable event family so an existing v1 corpus can be
/// expanded without rewriting history or duplicating its revenue/working-
/// capital claims under a new event identity.
const FINANCIAL_QUALITY_METRICS: &[MetricSpec] = &[
    MetricSpec {
        metric_id: "gross_profit",
        label_zh: "毛利润",
        shape: FactShape::DurationQuarter,
        us_gaap_tags: &["GrossProfit"],
        ifrs_tags: &["GrossProfit"],
    },
    MetricSpec {
        metric_id: "operating_income",
        label_zh: "营业利润",
        shape: FactShape::DurationQuarter,
        us_gaap_tags: &["OperatingIncomeLoss"],
        ifrs_tags: &["ProfitLossFromOperatingActivities"],
    },
    MetricSpec {
        metric_id: "operating_cash_flow",
        label_zh: "经营活动现金流",
        shape: FactShape::DurationYtd,
        us_gaap_tags: &["NetCashProvidedByUsedInOperatingActivities"],
        ifrs_tags: &["CashFlowsFromUsedInOperatingActivities"],
    },
    MetricSpec {
        metric_id: "cash_and_equivalents",
        label_zh: "现金及现金等价物",
        shape: FactShape::Instant,
        us_gaap_tags: &[
            "CashAndCashEquivalentsAtCarryingValue",
            "CashCashEquivalentsRestrictedCashAndRestrictedCashEquivalents",
        ],
        ifrs_tags: &["CashAndCashEquivalents"],
    },
    MetricSpec {
        metric_id: "long_term_debt",
        label_zh: "长期债务总额",
        shape: FactShape::Instant,
        us_gaap_tags: &[
            "LongTermDebtAndFinanceLeaseObligations",
            "LongTermDebtAndCapitalLeaseObligations",
            "LongTermDebt",
        ],
        ifrs_tags: &["LongtermBorrowings"],
    },
];

const IFRS_CURRENCY_PREFERENCE: &[&str] = &["USD", "EUR", "TWD", "GBP"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FilingAccountingContext {
    UsGaapUsd,
    Ifrs { currency: &'static str },
}

impl FilingAccountingContext {
    fn taxonomy(self) -> &'static str {
        match self {
            Self::UsGaapUsd => "us-gaap",
            Self::Ifrs { .. } => "ifrs-full",
        }
    }

    fn basis_label(self) -> &'static str {
        match self {
            Self::UsGaapUsd => "US-GAAP",
            Self::Ifrs { .. } => "IFRS",
        }
    }

    fn currency(self) -> &'static str {
        match self {
            Self::UsGaapUsd => "USD",
            Self::Ifrs { currency } => currency,
        }
    }

    fn tags(self, spec: &MetricSpec) -> &'static [&'static str] {
        match self {
            Self::UsGaapUsd => spec.us_gaap_tags,
            Self::Ifrs { .. } => spec.ifrs_tags,
        }
    }
}

fn events_from_sec_payloads(
    symbol: &str,
    cik: u64,
    submissions: &Value,
    facts: &Value,
    history_filings: usize,
    archives_base: &str,
) -> Vec<MarketEvent> {
    filings_from_submissions(submissions, history_filings)
        .into_iter()
        .flat_map(|filing| {
            let Some(accounting_context) = accounting_context_for_filing(&filing, facts) else {
                return Vec::new();
            };
            [
                ("base-v1", "sec-companyfacts", BASE_METRICS),
                (
                    "financial-quality-v2",
                    "sec-companyfacts-financial-quality-v2",
                    FINANCIAL_QUALITY_METRICS,
                ),
            ]
            .into_iter()
            .filter_map(|(metric_set_version, event_prefix, metrics)| {
                event_for_metric_set(
                    symbol,
                    cik,
                    &filing,
                    facts,
                    archives_base,
                    metric_set_version,
                    event_prefix,
                    metrics,
                    accounting_context,
                )
            })
            .collect::<Vec<_>>()
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn event_for_metric_set(
    symbol: &str,
    cik: u64,
    filing: &FilingRecord,
    facts: &Value,
    archives_base: &str,
    metric_set_version: &str,
    event_prefix: &str,
    metrics: &[MetricSpec],
    accounting_context: FilingAccountingContext,
) -> Option<MarketEvent> {
    let claims = metrics
        .iter()
        .filter_map(|metric| claim_for_filing(metric, filing, facts, accounting_context))
        .collect::<Vec<_>>();
    if claims.is_empty() {
        return None;
    }
    let accession_path = filing.accession.replace('-', "");
    let document = filing.primary_document.trim_start_matches('/');
    let url = format!("{archives_base}/{cik}/{accession_path}/{document}");
    Some(MarketEvent {
        id: format!("{event_prefix}:{symbol}:{}", filing.accession),
        kind: EventKind::SecFiling {
            form: filing.form.clone(),
        },
        severity: Severity::Low,
        symbols: vec![symbol.to_string()],
        occurred_at: filing.accepted_at,
        title: format!("{symbol} {} point-in-time financial facts", filing.form),
        summary: format!(
            "SEC filed {} · period ended {} · {} structured facts",
            filing.filing_date,
            filing.report_date,
            claims.len()
        ),
        url: Some(url),
        source: BACKFILL_SOURCE.into(),
        payload: serde_json::json!({
            "point_in_time_backfill": true,
            "cik": cik,
            "accession": filing.accession,
            "filing_date": filing.filing_date,
            "accepted_at": filing.accepted_at,
            "report_date": filing.report_date,
            "primary_document": filing.primary_document,
            "metric_set_version": metric_set_version,
            "accounting_taxonomy": accounting_context.basis_label(),
            "reporting_currency": accounting_context.currency(),
            "earnings_filing_claims": claims,
        }),
    })
}

fn accounting_context_for_filing(
    filing: &FilingRecord,
    facts: &Value,
) -> Option<FilingAccountingContext> {
    let us_gaap = FilingAccountingContext::UsGaapUsd;
    let us_gaap_coverage = context_metric_coverage(filing, facts, us_gaap);
    let ifrs = (filing.form == "20-F")
        .then(|| {
            IFRS_CURRENCY_PREFERENCE
                .iter()
                .copied()
                .map(|currency| {
                    let context = FilingAccountingContext::Ifrs { currency };
                    (context_metric_coverage(filing, facts, context), context)
                })
                // `max_by_key` selects the last value on a tie. Reverse the
                // preference iterator so USD remains the deterministic tie
                // winner, followed by EUR, TWD and GBP.
                .rev()
                .max_by_key(|(coverage, _)| *coverage)
                .unwrap_or((0, FilingAccountingContext::Ifrs { currency: "USD" }))
        })
        .filter(|(coverage, _)| *coverage > 0);

    match ifrs {
        Some((ifrs_coverage, context)) if ifrs_coverage > us_gaap_coverage => Some(context),
        _ if us_gaap_coverage > 0 => Some(us_gaap),
        Some((_, context)) => Some(context),
        None => None,
    }
}

fn context_metric_coverage(
    filing: &FilingRecord,
    facts: &Value,
    context: FilingAccountingContext,
) -> usize {
    BASE_METRICS
        .iter()
        .chain(FINANCIAL_QUALITY_METRICS)
        .filter(|spec| fact_for_metric(spec, filing, facts, context).is_some())
        .count()
}

fn fact_for_metric<'a>(
    spec: &MetricSpec,
    filing: &FilingRecord,
    facts: &'a Value,
    context: FilingAccountingContext,
) -> Option<(&'static str, &'a Value)> {
    for tag in context.tags(spec) {
        let Some(units) = facts
            .pointer(&format!(
                "/facts/{}/{tag}/units/{}",
                context.taxonomy(),
                context.currency()
            ))
            .and_then(Value::as_array)
        else {
            continue;
        };
        if let Some(candidate) = best_fact(units, filing, spec.shape) {
            return Some((tag, candidate));
        }
    }
    None
}

fn claim_for_filing(
    spec: &MetricSpec,
    filing: &FilingRecord,
    facts: &Value,
    accounting_context: FilingAccountingContext,
) -> Option<EarningsClaimInput> {
    let (tag, candidate) = fact_for_metric(spec, filing, facts, accounting_context)?;
    let raw_value = candidate.get("val").and_then(Value::as_f64)?;
    if !raw_value.is_finite() {
        return None;
    }
    let currency = accounting_context.currency();
    let value_millions = raw_value / 1_000_000.0;
    let value_text = format!("{} {currency} million", format_number(value_millions));
    let start = candidate.get("start").and_then(Value::as_str).unwrap_or("");
    let end = candidate.get("end").and_then(Value::as_str).unwrap_or("");
    let fy = candidate.get("fy").and_then(Value::as_i64);
    let fp = candidate.get("fp").and_then(Value::as_str).unwrap_or("");
    let fiscal = match fy {
        Some(fy) if !fp.is_empty() => format!("FY{fy} {fp}"),
        Some(fy) => format!("FY{fy}"),
        None => filing.form.clone(),
    };
    let period = if start.is_empty() {
        format!("{fiscal} ending {end}")
    } else {
        format!("{fiscal} {start} to {end}")
    };
    Some(EarningsClaimInput {
        claim_kind: EarningsClaimKind::ReportedFact,
        metric_id: spec.metric_id.into(),
        metric_basis: format!("{}:{tag}", accounting_context.basis_label()),
        period,
        numeric_value: Some(value_millions),
        unit: format!("{currency}_millions"),
        value_text: value_text.clone(),
        speaker: String::new(),
        evidence_zh: format!(
            "SEC XBRL（{}）披露{}为{}",
            accounting_context.basis_label(),
            spec.label_zh,
            value_text
        ),
        source_locator: format!(
            "XBRL {}:{tag}; currency {currency}; accession {}",
            accounting_context.basis_label(),
            filing.accession
        ),
        disposition: EarningsClaimDisposition::Active,
    })
}

fn best_fact<'a>(units: &'a [Value], filing: &FilingRecord, shape: FactShape) -> Option<&'a Value> {
    units
        .iter()
        .filter(|fact| fact.get("accn").and_then(Value::as_str) == Some(&filing.accession))
        .filter(|fact| fact.get("form").and_then(Value::as_str) == Some(&filing.form))
        .filter(|fact| fact.get("val").and_then(Value::as_f64).is_some())
        .max_by_key(|fact| fact_score(fact, filing, shape))
}

fn fact_score(fact: &Value, filing: &FilingRecord, shape: FactShape) -> i64 {
    let end = fact.get("end").and_then(Value::as_str).unwrap_or("");
    let mut score = if end == filing.report_date { 10_000 } else { 0 };
    let duration_days = fact_duration_days(fact);
    score += match shape {
        FactShape::Instant => {
            if fact.get("start").and_then(Value::as_str).is_none() {
                5_000
            } else {
                -5_000
            }
        }
        FactShape::DurationQuarter if filing.form == "10-Q" => duration_days
            .map(|days| 4_000 - (days - 91).abs())
            .unwrap_or(-5_000),
        FactShape::DurationQuarter => duration_days
            .map(|days| 4_000 - (days - 365).abs())
            .unwrap_or(-5_000),
        FactShape::DurationYtd if filing.form == "10-Q" => {
            duration_days.map(|days| 3_000 + days).unwrap_or(-5_000)
        }
        FactShape::DurationYtd => duration_days
            .map(|days| 4_000 - (days - 365).abs())
            .unwrap_or(-5_000),
    };
    if fact.get("frame").and_then(Value::as_str).is_some() {
        score += 20;
    }
    score
}

fn fact_duration_days(fact: &Value) -> Option<i64> {
    let start = NaiveDate::parse_from_str(fact.get("start")?.as_str()?, "%Y-%m-%d").ok()?;
    let end = NaiveDate::parse_from_str(fact.get("end")?.as_str()?, "%Y-%m-%d").ok()?;
    Some((end - start).num_days())
}

fn format_number(value: f64) -> String {
    let mut rendered = format!("{value:.6}");
    while rendered.contains('.') && rendered.ends_with('0') {
        rendered.pop();
    }
    if rendered.ends_with('.') {
        rendered.pop();
    }
    rendered
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::earnings_claim::source_claims_from_event;

    fn config() -> SecCompanyFactsConfig {
        SecCompanyFactsConfig {
            enabled: true,
            symbols: vec!["sndk".into(), "MU".into(), "SNDK".into()],
            history_filings: 8,
            refresh_hours: 24,
            user_agent: "HONE tests tests@example.com".into(),
        }
    }

    #[test]
    fn builds_point_in_time_claims_from_matching_accession_only() {
        let submissions = serde_json::json!({"filings":{"recent":{
            "accessionNumber":["0001-26-000001"], "form":["10-Q"],
            "filingDate":["2026-05-07"], "acceptanceDateTime":["2026-05-07T16:30:12.000Z"],
            "reportDate":["2026-03-31"], "primaryDocument":["sndk-20260331.htm"]
        }}});
        let facts = serde_json::json!({"facts":{"us-gaap":{
            "RevenueFromContractWithCustomerExcludingAssessedTax":{"units":{"USD":[
                {"start":"2026-01-01","end":"2026-03-31","val":2000000000.0,
                 "accn":"0001-26-000001","form":"10-Q","fy":2026,"fp":"Q1","frame":"CY2026Q1"},
                {"start":"2025-01-01","end":"2025-03-31","val":1000000000.0,
                 "accn":"old","form":"10-Q","fy":2025,"fp":"Q1"}
            ]}},
            "InventoryNet":{"units":{"USD":[
                {"end":"2026-03-31","val":750000000.0,"accn":"0001-26-000001",
                 "form":"10-Q","fy":2026,"fp":"Q1"}
            ]}}
        }}});
        let events =
            events_from_sec_payloads("SNDK", 2023554, &submissions, &facts, 8, SEC_ARCHIVES_BASE);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].id, "sec-companyfacts:SNDK:0001-26-000001");
        assert_eq!(
            events[0].occurred_at.to_rfc3339(),
            "2026-05-07T16:30:12+00:00"
        );
        assert!(events[0].url.as_deref().unwrap().contains("000126000001"));
        let claims = source_claims_from_event(&events[0]);
        assert_eq!(claims.len(), 2);
        assert_eq!(claims[0].metric_id, "revenue");
        assert_eq!(claims[0].numeric_value, Some(2000.0));
        assert_eq!(claims[1].metric_id, "inventory");
    }

    #[test]
    fn expands_cash_margin_and_debt_facts_without_merging_xbrl_definitions() {
        let submissions = serde_json::json!({"filings":{"recent":{
            "accessionNumber":["0001-26-000002"], "form":["10-Q"],
            "filingDate":["2026-08-07"], "acceptanceDateTime":["2026-08-07T20:15:00Z"],
            "reportDate":["2026-06-30"], "primaryDocument":["test-20260630.htm"]
        }}});
        let fact = |val: f64, instant: bool| {
            if instant {
                serde_json::json!({"end":"2026-06-30","val":val,"accn":"0001-26-000002","form":"10-Q","fy":2026,"fp":"Q2"})
            } else {
                serde_json::json!({"start":"2026-04-01","end":"2026-06-30","val":val,"accn":"0001-26-000002","form":"10-Q","fy":2026,"fp":"Q2","frame":"CY2026Q2"})
            }
        };
        let facts = serde_json::json!({"facts":{"us-gaap":{
            "GrossProfit":{"units":{"USD":[fact(900_000_000.0, false)]}},
            "OperatingIncomeLoss":{"units":{"USD":[fact(350_000_000.0, false)]}},
            "NetCashProvidedByUsedInOperatingActivities":{"units":{"USD":[{
                "start":"2026-01-01","end":"2026-06-30","val":700_000_000.0,
                "accn":"0001-26-000002","form":"10-Q","fy":2026,"fp":"Q2"
            }]}},
            "CashCashEquivalentsRestrictedCashAndRestrictedCashEquivalents":{"units":{"USD":[fact(1_200_000_000.0, true)]}},
            "LongTermDebt":{"units":{"USD":[fact(500_000_000.0, true)]}}
        }}});
        let events =
            events_from_sec_payloads("TEST", 1, &submissions, &facts, 8, SEC_ARCHIVES_BASE);
        assert_eq!(
            events[0].id,
            "sec-companyfacts-financial-quality-v2:TEST:0001-26-000002"
        );
        let claims = source_claims_from_event(&events[0]);
        assert_eq!(claims.len(), 5);
        let by_metric = claims
            .iter()
            .map(|claim| (claim.metric_id.as_str(), claim))
            .collect::<HashMap<_, _>>();
        assert_eq!(by_metric["gross_profit"].numeric_value, Some(900.0));
        assert_eq!(by_metric["operating_cash_flow"].numeric_value, Some(700.0));
        assert_eq!(
            by_metric["cash_and_equivalents"].metric_basis,
            "US-GAAP:CashCashEquivalentsRestrictedCashAndRestrictedCashEquivalents"
        );
        assert_eq!(
            by_metric["long_term_debt"].metric_basis,
            "US-GAAP:LongTermDebt"
        );
    }

    #[test]
    fn admits_20_f_facts_only_when_accession_and_form_match() {
        let submissions = serde_json::json!({"filings":{"recent":{
            "accessionNumber":["0002-26-000003", "0002-26-000002"],
            "form":["6-K", "20-F"],
            "filingDate":["2026-05-10", "2026-04-30"],
            "acceptanceDateTime":["2026-05-10T12:00:00Z", "2026-04-30T12:00:00Z"],
            "reportDate":["2026-03-31", "2025-12-31"],
            "primaryDocument":["foreign-6k.htm", "foreign-20f.htm"]
        }}});
        let facts = serde_json::json!({"facts":{"us-gaap":{
            "RevenueFromContractWithCustomerExcludingAssessedTax":{"units":{"USD":[
                {"start":"2025-01-01","end":"2025-12-31","val":8_400_000_000.0,
                 "accn":"0002-26-000002","form":"20-F","fy":2025,"fp":"FY"},
                {"start":"2025-01-01","end":"2025-12-31","val":99_000_000_000.0,
                 "accn":"different-accession","form":"20-F","fy":2025,"fp":"FY"}
            ]}},
            "GrossProfit":{"units":{"USD":[
                {"start":"2025-01-01","end":"2025-12-31","val":3_100_000_000.0,
                 "accn":"0002-26-000002","form":"20-F","fy":2025,"fp":"FY"}
            ]}}
        }}});

        let events =
            events_from_sec_payloads("FOREIGN", 2, &submissions, &facts, 8, SEC_ARCHIVES_BASE);
        assert_eq!(events.len(), 2);
        assert!(events.iter().all(|event| matches!(
            &event.kind,
            EventKind::SecFiling { form } if form == "20-F"
        )));
        let claims = events
            .iter()
            .flat_map(source_claims_from_event)
            .collect::<Vec<_>>();
        assert_eq!(claims.len(), 2);
        assert_eq!(
            claims
                .iter()
                .find(|claim| claim.metric_id == "revenue")
                .and_then(|claim| claim.numeric_value),
            Some(8_400.0)
        );
        assert!(
            !events
                .iter()
                .any(|event| event.id.contains("0002-26-000003"))
        );
    }

    #[test]
    fn admits_ifrs_20_f_in_one_original_reporting_currency() {
        let submissions = serde_json::json!({"filings":{"recent":{
            "accessionNumber":["0003-26-000001"], "form":["20-F"],
            "filingDate":["2026-03-05"], "acceptanceDateTime":["2026-03-05T12:00:00Z"],
            "reportDate":["2025-12-31"], "primaryDocument":["issuer-20f.htm"]
        }}});
        let facts = serde_json::json!({"facts":{"ifrs-full":{
            "RevenueFromContractsWithCustomers":{"units":{"EUR":[
                {"start":"2025-01-01","end":"2025-12-31","val":19_889_000_000.0,
                 "accn":"0003-26-000001","form":"20-F","fy":2025,"fp":"FY"}
            ]}},
            "GrossProfit":{"units":{"EUR":[
                {"start":"2025-01-01","end":"2025-12-31","val":8_659_000_000.0,
                 "accn":"0003-26-000001","form":"20-F","fy":2025,"fp":"FY"}
            ]}}
        }}});

        let events = events_from_sec_payloads("NOK", 3, &submissions, &facts, 8, SEC_ARCHIVES_BASE);
        assert_eq!(events.len(), 2);
        assert!(events.iter().all(|event| {
            event
                .payload
                .get("accounting_taxonomy")
                .and_then(Value::as_str)
                == Some("IFRS")
                && event
                    .payload
                    .get("reporting_currency")
                    .and_then(Value::as_str)
                    == Some("EUR")
        }));
        let claims = events
            .iter()
            .flat_map(source_claims_from_event)
            .collect::<Vec<_>>();
        assert_eq!(claims.len(), 2);
        assert!(claims.iter().all(|claim| claim.unit == "EUR_millions"));
        assert!(
            claims
                .iter()
                .all(|claim| claim.metric_basis.starts_with("IFRS:"))
        );
    }

    #[test]
    fn chooses_one_currency_for_an_ifrs_filing_and_prefers_usd_on_equal_coverage() {
        let submissions = serde_json::json!({"filings":{"recent":{
            "accessionNumber":["0004-26-000001"], "form":["20-F"],
            "filingDate":["2026-04-17"], "acceptanceDateTime":["2026-04-17T12:00:00Z"],
            "reportDate":["2025-12-31"], "primaryDocument":["issuer-20f.htm"]
        }}});
        let amount = |value: f64| {
            serde_json::json!({
                "start":"2025-01-01","end":"2025-12-31","val":value,
                "accn":"0004-26-000001","form":"20-F","fy":2025,"fp":"FY"
            })
        };
        let facts = serde_json::json!({"facts":{"ifrs-full":{
            "Revenue":{"units":{
                "USD":[amount(90_000_000_000.0)],
                "TWD":[amount(2_900_000_000_000.0)]
            }},
            "GrossProfit":{"units":{
                "USD":[amount(50_000_000_000.0)],
                "TWD":[amount(1_600_000_000_000.0)]
            }}
        }}});

        let events = events_from_sec_payloads("TSM", 4, &submissions, &facts, 8, SEC_ARCHIVES_BASE);
        assert_eq!(events.len(), 2);
        assert!(events.iter().all(|event| {
            event
                .payload
                .get("reporting_currency")
                .and_then(Value::as_str)
                == Some("USD")
        }));
        let claims = events
            .iter()
            .flat_map(source_claims_from_event)
            .collect::<Vec<_>>();
        assert!(claims.iter().all(|claim| claim.unit == "USD_millions"));
        assert_eq!(
            claims
                .iter()
                .find(|claim| claim.metric_id == "revenue")
                .and_then(|claim| claim.numeric_value),
            Some(90_000.0)
        );
    }

    #[test]
    fn normalizes_symbols_and_requires_sec_contact_email() {
        assert_eq!(normalized_symbols(&config().symbols), vec!["MU", "SNDK"]);
        let mut invalid = config();
        invalid.user_agent = "HONE".into();
        assert!(SecCompanyFactsBackfiller::new(invalid).is_err());
    }

    #[test]
    fn test_only_base_override_keeps_constructor_deterministic() {
        let backfiller = SecCompanyFactsBackfiller::new(config())
            .unwrap()
            .with_bases(
                "http://localhost/tickers",
                "http://localhost",
                "https://sec.test",
            );
        assert_eq!(backfiller.tickers_url, "http://localhost/tickers");
    }
}
