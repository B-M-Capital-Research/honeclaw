//! DataFetchTool — 金融数据获取工具
//!
//! 通过 Financial Modeling Prep (FMP) API 获取金融数据，支持多 Key 自动 fallback：
//! - 依次尝试 `fmp.api_keys` 和 `fmp.api_key` 合并后的 Key 列表
//! - 若 Key 认证或配额不可用（HTTP 401/403/429 或响应含相关错误）则切换到下一个
//! - 所有 Key 均失败时返回最后一次的错误信息

use async_trait::async_trait;
use chrono::{DateTime, Duration, NaiveDate, NaiveDateTime, NaiveTime, TimeZone};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration as StdDuration, Instant};

use crate::base::{Tool, ToolParameter};

const MAX_FMP_TRANSPORT_ERROR_CHARS: usize = 300;
const FMP_TTL_EXTENDED_HOURS: StdDuration = StdDuration::from_secs(30);
const FMP_TTL_FAST: StdDuration = StdDuration::from_secs(5 * 60);
const FMP_TTL_NEWS: StdDuration = StdDuration::from_secs(15 * 60);
const FMP_TTL_PROFILE: StdDuration = StdDuration::from_secs(24 * 60 * 60);
const FMP_TTL_FINANCIALS: StdDuration = StdDuration::from_secs(6 * 60 * 60);
const FMP_TTL_EARNINGS: StdDuration = StdDuration::from_secs(60 * 60);
const MAX_FMP_SYMBOL_INPUT_BYTES: usize = 512;
const SEC_COMPANY_TICKERS_URL: &str = "https://www.sec.gov/files/company_tickers.json";
const SEC_COMPANY_FACTS_BASE_URL: &str = "https://data.sec.gov/api/xbrl/companyfacts";
const SEC_SUBMISSIONS_BASE_URL: &str = "https://data.sec.gov/submissions";
const SEC_USER_AGENT: &str = "HONE investment research support@hone-claw.com";

/// Resolve the request exactly as `DataFetchTool::execute` does. Callers that
/// observe DataFetch attempts must use these helpers instead of re-parsing
/// aliases independently, otherwise a conflicting or wrongly typed field can
/// make telemetry describe a different provider request than the executor.
pub fn effective_data_fetch_data_type(args: &Value) -> &str {
    args.get("data_type")
        .and_then(Value::as_str)
        .unwrap_or("quote")
}

pub fn effective_data_fetch_target(args: &Value) -> &str {
    let data_type = effective_data_fetch_data_type(args);
    let selected = if data_type == "search" {
        args.get("query")
            .or_else(|| args.get("ticker"))
            .or_else(|| args.get("symbol"))
    } else {
        args.get("ticker")
            .or_else(|| args.get("symbol"))
            .or_else(|| {
                // Some OpenAI-compatible providers keep an exact-symbol
                // lookup in `query` even after switching from search to a
                // symbol-scoped data type. Accept that alias only when the
                // same call explicitly declares exact_symbol; natural names,
                // missing match modes, and conflicting typed fields still
                // fail closed.
                (args.get("identity_match").and_then(Value::as_str) == Some("exact_symbol"))
                    .then(|| args.get("query"))
                    .flatten()
            })
    };
    selected.and_then(Value::as_str).unwrap_or("")
}

pub fn data_fetch_data_type_uses_security_target(data_type: &str) -> bool {
    matches!(
        data_type,
        "search"
            | "quote"
            | "quote_short"
            | "extended_hours"
            | "profile"
            | "snapshot"
            | "earnings_outlook"
            | "financials"
            | "news"
            | "crypto_quote"
            | "etf_holdings"
            | "valuation"
            | "segments"
            | "peers"
            | "ownership"
            | "corporate_actions"
            | "press_releases"
            | "transcript"
    )
}

pub fn effective_data_fetch_security_target(args: &Value) -> Option<&str> {
    let data_type = effective_data_fetch_data_type(args);
    data_fetch_data_type_uses_security_target(data_type)
        .then(|| effective_data_fetch_target(args).trim())
        .filter(|target| !target.is_empty())
}

/// Encode every provider symbol as URL data before it is interpolated into an
/// endpoint path or query parameter. Commas remain separators for the FMP
/// batch endpoints, while characters such as `/`, `?`, `#`, `%`, and `^` are
/// encoded inside each symbol rather than being allowed to change URL
/// structure.
fn encode_fmp_symbols(value: &str, allow_empty: bool) -> Result<String, String> {
    validated_fmp_symbols(value, allow_empty)?
        .into_iter()
        .map(|symbol| {
            Ok(url::form_urlencoded::byte_serialize(symbol.as_bytes()).collect::<String>())
        })
        .collect::<Result<Vec<_>, String>>()
        .map(|symbols| symbols.join(","))
}

pub fn validated_data_fetch_symbols(value: &str) -> Result<Vec<String>, String> {
    validated_fmp_symbols(value, false)
}

fn validated_fmp_symbols(value: &str, allow_empty: bool) -> Result<Vec<String>, String> {
    let value = value.trim();
    if value.is_empty() {
        return if allow_empty {
            Ok(Vec::new())
        } else {
            Err("证券代码不能为空".to_string())
        };
    }
    if value.len() > MAX_FMP_SYMBOL_INPUT_BYTES || value.chars().any(char::is_control) {
        return Err("证券代码格式无效".to_string());
    }

    value
        .split(',')
        .map(str::trim)
        .map(|symbol| {
            if symbol.is_empty() {
                return Err("证券代码格式无效".to_string());
            }
            Ok(symbol.to_string())
        })
        .collect()
}

pub fn validated_data_fetch_search_query(value: &str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() {
        return Err("search query 不能为空".to_string());
    }
    if value.len() > MAX_FMP_SYMBOL_INPUT_BYTES || value.chars().any(char::is_control) {
        return Err("search query 无效或过长".to_string());
    }
    Ok(value.to_string())
}

#[derive(Clone)]
struct CachedFmpValue {
    expires_at: Instant,
    value: Value,
}

enum FmpFetchError {
    /// 当前 key 的认证或配额不可用，可以安全地尝试下一个 key。
    KeyRejected(String),
    /// 与 key 无关的 provider、传输或解析失败，继续轮询只会放大延迟。
    NonRetryable(String),
}

/// DataFetchTool — 金融数据获取（FMP，多 Key fallback）
pub struct DataFetchTool {
    /// 有效 API Key 列表（过滤空值、去重后）
    keys: Vec<String>,
    base_url: String,
    timeout: u64,
    http: reqwest::Client,
    cache: Arc<Mutex<HashMap<String, CachedFmpValue>>>,
    official_fallback_enabled: bool,
}

fn fmp_base_url_is_loopback(base_url: &str) -> bool {
    let Some(host) = url::Url::parse(base_url)
        .ok()
        .and_then(|parsed| parsed.host_str().map(str::to_string))
    else {
        return false;
    };
    let host = host
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
        .unwrap_or(&host);
    host.eq_ignore_ascii_case("localhost")
        || host
            .parse::<std::net::IpAddr>()
            .is_ok_and(|address| address.is_loopback())
}

fn fmp_http_client(base_url: &str) -> reqwest::Client {
    let builder = reqwest::Client::builder();
    let builder = if fmp_base_url_is_loopback(base_url) {
        // Local provider adapters and test stubs must never be sent through a
        // workstation HTTP proxy. Apart from making tests environment-bound,
        // proxying a loopback URL can silently contact the wrong process.
        builder.no_proxy()
    } else {
        builder
    };
    builder.build().expect("build FMP HTTP client")
}

impl DataFetchTool {
    pub fn new(keys: Vec<String>, base_url: &str, timeout: u64) -> Self {
        let pool = hone_core::ApiKeyPool::new(keys);
        let base_url = base_url.trim_end_matches('/').to_string();
        Self {
            keys: pool.keys().to_vec(),
            http: fmp_http_client(&base_url),
            base_url,
            timeout,
            cache: Arc::new(Mutex::new(HashMap::new())),
            official_fallback_enabled: true,
        }
    }

    pub fn from_config(config: &hone_core::config::HoneConfig) -> Self {
        let pool = config.fmp.effective_key_pool();
        let base_url = config.fmp.base_url.trim_end_matches('/').to_string();
        Self {
            keys: pool.keys().to_vec(),
            http: fmp_http_client(&base_url),
            base_url,
            timeout: config.fmp.timeout,
            cache: Arc::new(Mutex::new(HashMap::new())),
            official_fallback_enabled: config.fmp.official_fallback_enabled,
        }
    }

    /// Nasdaq's public quote endpoint is a deliberately narrow fallback for
    /// local HONE installations that have no FMP subscription. It restores
    /// exact-symbol identity and a dated current-price anchor; it must never be
    /// presented as a replacement for statements, estimates or valuation
    /// data. Unsupported data types continue to fail closed below.
    async fn fetch_nasdaq_quote_row(&self, symbol: &str) -> Result<Value, String> {
        let symbols = validated_data_fetch_symbols(symbol)?;
        let [symbol] = symbols.as_slice() else {
            return Err("Nasdaq 降级行情一次只接受一个证券代码".to_string());
        };
        if !symbol
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '.' | '-'))
        {
            return Err("Nasdaq 降级行情仅支持标准美股证券代码".to_string());
        }
        let encoded = url::form_urlencoded::byte_serialize(symbol.as_bytes()).collect::<String>();
        let url = format!("https://api.nasdaq.com/api/quote/{encoded}/info?assetclass=stocks");
        let response = self
            .http
            .get(url)
            .header(
                reqwest::header::USER_AGENT,
                "Mozilla/5.0 (HONE investment research)",
            )
            .header(reqwest::header::ACCEPT, "application/json")
            .timeout(std::time::Duration::from_secs(self.timeout.min(12)))
            .send()
            .await
            .map_err(|error| format!("Nasdaq 行情请求失败: {error}"))?;
        if !response.status().is_success() {
            return Err(format!("Nasdaq 行情请求失败（HTTP {}）", response.status()));
        }
        let payload = response
            .json::<Value>()
            .await
            .map_err(|error| format!("Nasdaq 行情 JSON 解析失败: {error}"))?;
        nasdaq_quote_row(symbol, &payload)
            .ok_or_else(|| format!("Nasdaq 没有返回 {symbol} 的可用同代码行情"))
    }

    async fn fetch_nasdaq_quote_rows(&self, symbols: &str) -> Vec<Value> {
        let Ok(symbols) = validated_data_fetch_symbols(symbols) else {
            return Vec::new();
        };
        futures::future::join_all(
            symbols
                .iter()
                .map(|symbol| self.fetch_nasdaq_quote_row(symbol)),
        )
        .await
        .into_iter()
        .filter_map(Result::ok)
        .collect()
    }

    async fn fetch_sec_json(&self, cache_key: &str, url: &str) -> Result<Value, String> {
        if let Some(value) = self.cached_value(cache_key) {
            return Ok(value);
        }
        let response = self
            .http
            .get(url)
            .header(reqwest::header::USER_AGENT, SEC_USER_AGENT)
            .header(reqwest::header::ACCEPT, "application/json")
            .timeout(std::time::Duration::from_secs(self.timeout.min(20)))
            .send()
            .await
            .map_err(|error| format!("SEC 官方数据请求失败: {error}"))?;
        if !response.status().is_success() {
            return Err(format!(
                "SEC 官方数据请求失败（HTTP {}）",
                response.status()
            ));
        }
        let value = response
            .json::<Value>()
            .await
            .map_err(|error| format!("SEC 官方数据 JSON 解析失败: {error}"))?;
        self.store_cache_value(cache_key.to_string(), FMP_TTL_FINANCIALS, value.clone());
        Ok(value)
    }

    async fn fetch_sec_financials(&self, symbol: &str) -> Result<Value, String> {
        let symbols = validated_data_fetch_symbols(symbol)?;
        let [symbol] = symbols.as_slice() else {
            return Err("SEC 财务降级链路一次只接受一个证券代码".to_string());
        };
        if !symbol
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '.' | '-'))
        {
            return Err("SEC 财务降级链路仅支持标准美股证券代码".to_string());
        }

        let tickers = self
            .fetch_sec_json("sec:company_tickers", SEC_COMPANY_TICKERS_URL)
            .await?;
        let wanted = symbol.to_ascii_uppercase();
        let company = tickers
            .as_object()
            .and_then(|rows| {
                rows.values().find(|row| {
                    row.get("ticker")
                        .and_then(Value::as_str)
                        .is_some_and(|ticker| ticker.eq_ignore_ascii_case(&wanted))
                })
            })
            .ok_or_else(|| format!("SEC 官方代码表未找到 {wanted}"))?;
        let cik = company
            .get("cik_str")
            .and_then(Value::as_u64)
            .ok_or_else(|| format!("SEC 官方代码表缺少 {wanted} 的 CIK"))?;
        let cik_padded = format!("CIK{cik:010}");
        let url = format!("{SEC_COMPANY_FACTS_BASE_URL}/{cik_padded}.json");
        let facts = self
            .fetch_sec_json(&format!("sec:companyfacts:{cik_padded}"), &url)
            .await?;
        Ok(normalize_sec_company_facts(&wanted, cik, &url, &facts))
    }

    /// Resolve an exact US ticker from the SEC's official ticker registry.
    ///
    /// Nasdaq's public quote endpoint is useful but occasionally returns an
    /// empty/transient response. Identity resolution must not collapse with
    /// the quote provider: a recent SEC filer can still be researched from its
    /// filings while the current-price field is disclosed as unavailable.
    async fn fetch_sec_identity_row(&self, symbol: &str) -> Result<Value, String> {
        let symbols = validated_data_fetch_symbols(symbol)?;
        let [symbol] = symbols.as_slice() else {
            return Err("SEC 身份降级链路一次只接受一个证券代码".to_string());
        };
        let wanted = symbol.to_ascii_uppercase();
        let tickers = self
            .fetch_sec_json("sec:company_tickers", SEC_COMPANY_TICKERS_URL)
            .await?;
        let company = tickers
            .as_object()
            .and_then(|rows| {
                rows.values().find(|row| {
                    row.get("ticker")
                        .and_then(Value::as_str)
                        .is_some_and(|ticker| ticker.eq_ignore_ascii_case(&wanted))
                })
            })
            .ok_or_else(|| format!("SEC 官方代码表未找到 {wanted}"))?;
        Ok(serde_json::json!({
            "symbol": wanted,
            "name": company.get("title").cloned().unwrap_or(Value::Null),
            "exchange": Value::Null,
            "currency": "USD",
            "cik": company.get("cik_str").cloned().unwrap_or(Value::Null),
            "identity_source_url": SEC_COMPANY_TICKERS_URL,
            "identity_scope": "exact_symbol_current_sec_registrant",
        }))
    }

    async fn fetch_sec_latest_earnings_filing(&self, symbol: &str) -> Result<Value, String> {
        let symbols = validated_data_fetch_symbols(symbol)?;
        let [symbol] = symbols.as_slice() else {
            return Err("SEC 财报附件降级链路一次只接受一个证券代码".to_string());
        };
        let tickers = self
            .fetch_sec_json("sec:company_tickers", SEC_COMPANY_TICKERS_URL)
            .await?;
        let wanted = symbol.to_ascii_uppercase();
        let company = tickers
            .as_object()
            .and_then(|rows| {
                rows.values().find(|row| {
                    row.get("ticker")
                        .and_then(Value::as_str)
                        .is_some_and(|ticker| ticker.eq_ignore_ascii_case(&wanted))
                })
            })
            .ok_or_else(|| format!("SEC 官方代码表未找到 {wanted}"))?;
        let cik = company
            .get("cik_str")
            .and_then(Value::as_u64)
            .ok_or_else(|| format!("SEC 官方代码表缺少 {wanted} 的 CIK"))?;
        let submissions_url = format!("{SEC_SUBMISSIONS_BASE_URL}/CIK{cik:010}.json");
        let submissions = self
            .fetch_sec_json(&format!("sec:submissions:CIK{cik:010}"), &submissions_url)
            .await?;
        let filing = latest_sec_earnings_filing_row(&submissions)
            .ok_or_else(|| format!("SEC submissions 未找到 {wanted} 的近期财报型申报"))?;
        let accession_compact = filing.accession.replace('-', "");
        let archive_base =
            format!("https://www.sec.gov/Archives/edgar/data/{cik}/{accession_compact}");
        let index_url = format!("{archive_base}/index.json");
        let index = self
            .fetch_sec_json(&format!("sec:filing-index:{accession_compact}"), &index_url)
            .await?;
        let exhibit = sec_earnings_exhibit_name(&index, &filing.primary_document);
        let exhibit_url = exhibit
            .as_deref()
            .map(|name| format!("{archive_base}/{name}"));
        // Preserve the authoritative filing metadata even if SEC temporarily
        // throttles the exhibit body. Dropping the whole object made callers
        // incorrectly call the older Company Facts quarter “latest”.
        let exhibit_excerpt = if let Some(url) = exhibit_url.as_deref() {
            self.fetch_sec_html_excerpt(url).await.ok()
        } else {
            None
        };
        let coverage = if exhibit_excerpt.is_some() {
            "latest_sec_filing_with_earnings_exhibit"
        } else {
            "latest_sec_filing_metadata_only"
        };
        Ok(serde_json::json!({
            "ticker": wanted,
            "form": filing.form,
            "filing_date": filing.filing_date,
            "report_date": filing.report_date,
            "items": filing.items,
            "accession_number": filing.accession,
            "primary_document_url": format!("{archive_base}/{}", filing.primary_document),
            "earnings_exhibit_url": exhibit_url,
            "earnings_exhibit_excerpt": exhibit_excerpt,
            "submissions_source_url": submissions_url,
            "coverage": coverage,
            "evidence_policy": "A recent 8-K/6-K earnings exhibit may be newer than SEC Company Facts. When its filing date is later, do not call the older XBRL period the latest reported quarter. Cite the SEC exhibit URL and preserve its stated reporting period."
        }))
    }

    async fn fetch_sec_html_excerpt(&self, url: &str) -> Result<String, String> {
        let response = self
            .http
            .get(url)
            .header(reqwest::header::USER_AGENT, SEC_USER_AGENT)
            .header(reqwest::header::ACCEPT, "text/html,application/xhtml+xml")
            .timeout(std::time::Duration::from_secs(self.timeout.min(20)))
            .send()
            .await
            .map_err(|error| format!("SEC 财报附件请求失败: {error}"))?;
        if !response.status().is_success() {
            return Err(format!(
                "SEC 财报附件请求失败（HTTP {}）",
                response.status()
            ));
        }
        let html = response
            .text()
            .await
            .map_err(|error| format!("SEC 财报附件读取失败: {error}"))?;
        // Earnings releases commonly put condensed balance-sheet and cash-flow
        // tables after the narrative and non-GAAP reconciliations. A 14k slice
        // captured the headline revenue but cut off OCF, capex and debt — the
        // exact fields an investment answer needs. The prompt layer applies its
        // own tighter bound after prioritising this filing.
        Ok(sec_html_to_text_excerpt(&html, 26_000))
    }

    async fn fetch_sec_valuation(&self, symbol: &str) -> Result<Value, String> {
        let (quote, financials) = tokio::join!(
            self.fetch_nasdaq_quote_row(symbol),
            self.fetch_sec_financials(symbol)
        );
        let quote = quote?;
        let financials = financials?;
        Ok(build_sec_valuation_inputs(symbol, quote, financials))
    }

    async fn fetch_without_fmp(&self, data_type: &str, target: &str) -> Value {
        match data_type {
            "quote" | "quote_short" => {
                let data = self.fetch_nasdaq_quote_rows(target).await;
                if data.is_empty() {
                    serde_json::json!({
                        "error": "未配置 FMP，且 Nasdaq 没有返回可用的同代码行情",
                        "fallback_source": "Nasdaq official quote API"
                    })
                } else {
                    serde_json::json!({
                        "data_type": data_type,
                        "ticker": target,
                        "data": data,
                        "fallback_source": "Nasdaq official quote API",
                        "coverage": "quote_only_without_fmp",
                    })
                }
            }
            "search" => match self.fetch_nasdaq_quote_row(target).await {
                Ok(row) => serde_json::json!({
                    "data_type": "search",
                    "ticker": target,
                    "data": [{
                        "symbol": row.get("symbol").cloned().unwrap_or(Value::Null),
                        "name": row.get("name").cloned().unwrap_or(Value::Null),
                        "exchange": row.get("exchange").cloned().unwrap_or(Value::Null),
                        "currency": row.get("currency").cloned().unwrap_or(Value::Null),
                        "stockExchange": row.get("exchange").cloned().unwrap_or(Value::Null),
                    }],
                    "fallback_source": "Nasdaq official quote API",
                    "coverage": "exact_symbol_only_without_fmp",
                }),
                Err(nasdaq_error) => match self.fetch_sec_identity_row(target).await {
                    Ok(row) => serde_json::json!({
                        "data_type": "search",
                        "ticker": target,
                        "data": [row],
                        "fallback_source": "SEC official company ticker registry",
                        "coverage": "exact_symbol_identity_without_current_quote",
                        "quote_gap": nasdaq_error,
                    }),
                    Err(sec_error) => serde_json::json!({
                        "error": "Nasdaq 与 SEC 官方代码表均未返回可用的精确代码身份",
                        "errors": {
                            "nasdaq": nasdaq_error,
                            "sec": sec_error,
                        },
                        "fallback_source": "Nasdaq official quote API + SEC official company ticker registry",
                        "coverage": "unavailable_without_fmp",
                    }),
                },
            },
            "snapshot" => match self.fetch_nasdaq_quote_row(target).await {
                Ok(row) => {
                    let profile = serde_json::json!([{
                        "symbol": row.get("symbol").cloned().unwrap_or(Value::Null),
                        "companyName": row.get("name").cloned().unwrap_or(Value::Null),
                        "exchangeShortName": row.get("exchange").cloned().unwrap_or(Value::Null),
                        "currency": row.get("currency").cloned().unwrap_or(Value::Null),
                        "isActivelyTrading": true,
                    }]);
                    let mut payload = self.build_snapshot_response(
                        target,
                        Ok(Value::Array(vec![row])),
                        Ok(profile),
                        Err("未配置 FMP，Nasdaq 降级链路不提供公司新闻".to_string()),
                    );
                    payload["fallback_source"] =
                        Value::String("Nasdaq official quote API".to_string());
                    payload["coverage"] =
                        Value::String("quote_and_identity_only_without_fmp".to_string());
                    payload
                }
                Err(error) => serde_json::json!({
                    "error": error,
                    "fallback_source": "Nasdaq official quote API",
                }),
            },
            "profile" => match self.fetch_nasdaq_quote_row(target).await {
                Ok(row) => serde_json::json!({
                    "data_type": "profile",
                    "ticker": target,
                    "data": [{
                        "symbol": row.get("symbol").cloned().unwrap_or(Value::Null),
                        "companyName": row.get("name").cloned().unwrap_or(Value::Null),
                        "exchangeShortName": row.get("exchange").cloned().unwrap_or(Value::Null),
                        "currency": row.get("currency").cloned().unwrap_or(Value::Null),
                        "isActivelyTrading": true,
                    }],
                    "fallback_source": "Nasdaq official quote API",
                    "coverage": "identity_only_without_fmp",
                }),
                Err(error) => serde_json::json!({
                    "error": error,
                    "fallback_source": "Nasdaq official quote API",
                }),
            },
            "financials" => match self.fetch_sec_financials(target).await {
                Ok(data) => data,
                Err(error) => serde_json::json!({
                    "error": error,
                    "fallback_source": "SEC Company Facts API",
                    "coverage": "unavailable_without_fmp",
                }),
            },
            "valuation" => match self.fetch_sec_valuation(target).await {
                Ok(data) => data,
                Err(error) => serde_json::json!({
                    "error": error,
                    "fallback_source": "Nasdaq quote + SEC Company Facts APIs",
                    "coverage": "unavailable_without_fmp",
                }),
            },
            "earnings_outlook" => {
                let (quote, financials, latest_sec_filing) = tokio::join!(
                    self.fetch_nasdaq_quote_row(target),
                    self.fetch_sec_financials(target),
                    self.fetch_sec_latest_earnings_filing(target)
                );
                let quote_error = quote.as_ref().err().cloned();
                let financials_error = financials.as_ref().err().cloned();
                let filing_error = latest_sec_filing.as_ref().err().cloned();
                let quote_value = quote.ok().map(|row| Value::Array(vec![row]));
                let financials_value = financials.ok();
                let filing_value = latest_sec_filing.ok();
                if quote_value.is_some() || financials_value.is_some() || filing_value.is_some() {
                    let current_official_filing = filing_value.clone();
                    serde_json::json!({
                        "data_type": "earnings_outlook",
                        "ticker": target,
                        "current_official_filing": current_official_filing,
                        "data": {
                            "quote": quote_value,
                            "financials": financials_value,
                            "latest_sec_filing": filing_value,
                            "analyst_estimates": Value::Null,
                            "price_target_consensus": Value::Null,
                            "ratings_snapshot": Value::Null,
                        },
                        "coverage": {
                            "quote": if quote_error.is_none() { "available" } else { "unavailable_without_fmp" },
                            "financials": if financials_error.is_none() { "available" } else { "unavailable" },
                            "latest_sec_filing": if filing_error.is_none() { "available" } else { "unavailable" },
                            "analyst_estimates": "unavailable_without_fmp",
                            "price_target_consensus": "unavailable_without_fmp",
                            "ratings_snapshot": "unavailable_without_fmp",
                        },
                        "component_errors": {
                            "quote": quote_error,
                            "financials": financials_error,
                            "latest_sec_filing": filing_error,
                        },
                        "fallback_source": "Nasdaq quote + SEC Company Facts + SEC submissions/filing archives",
                        "evidence_policy": "SEC facts are filed historical statements, not analyst estimates. Preserve each fact's period, form and filed date; do not turn cumulative cash-flow contexts into a standalone quarter.",
                    })
                } else {
                    serde_json::json!({
                        "error": "未配置 FMP，且 Nasdaq/SEC 降级链路未返回任何可用组件",
                        "errors": {
                            "quote": quote_error,
                            "financials": financials_error,
                            "latest_sec_filing": filing_error,
                        },
                        "fallback_source": "Nasdaq quote + SEC Company Facts + SEC submissions/filing archives",
                    })
                }
            }
            _ => serde_json::json!({
                "error": format!(
                    "未配置 FMP API Key；官方降级链路提供精确代码 search、quote、profile、snapshot、SEC financials、valuation 与 earnings_outlook，不提供 {data_type}"
                ),
                "coverage": "unsupported_without_fmp",
            }),
        }
    }

    /// 用指定 key 执行一次 FMP 请求
    async fn fetch_with_key(&self, key: &str, url: &str) -> Result<Value, FmpFetchError> {
        let connector = if url.contains('?') { "&" } else { "?" };
        let full_url = format!("{}{connector}apikey={}", url, key);

        let response = self
            .http
            .get(&full_url)
            .timeout(std::time::Duration::from_secs(self.timeout))
            .send()
            .await
            .map_err(|e| FmpFetchError::NonRetryable(format_fmp_transport_error("请求", &e)))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| FmpFetchError::NonRetryable(format_fmp_transport_error("响应读取", &e)))?;

        // 认证或配额失败需要保留多 key fallback 语义；其它非 2xx 则必须作为
        // provider error 返回，不能继续把错误响应体解析成一份成功的金融数据。
        if status == 401 || status == 403 {
            return Err(FmpFetchError::KeyRejected(format!(
                "FMP API Key 无效（HTTP {}）",
                status.as_u16()
            )));
        }
        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(FmpFetchError::KeyRejected(
                "FMP API Key 配额受限（HTTP 429）".to_string(),
            ));
        }
        if !status.is_success() {
            return Err(FmpFetchError::NonRetryable(format_fmp_provider_error(
                status, &body,
            )));
        }

        let response_json: Value = serde_json::from_str(&body).map_err(|e| {
            let prefix = sanitize_fmp_error_detail(&body)
                .chars()
                .take(200)
                .collect::<String>();
            FmpFetchError::NonRetryable(format!("FMP JSON 解析失败: {e}; body_prefix={prefix}"))
        })?;

        // FMP 在 HTTP 2xx 时也可能通过 "Error Message" 返回失败。认证或配额
        // 问题继续触发 key fallback；其它非空错误也不能被当作成功数据。
        if let Some(err_msg) = response_json
            .get("Error Message")
            .and_then(nonempty_fmp_error_message)
        {
            if fmp_error_message_triggers_key_fallback(&err_msg) {
                return Err(FmpFetchError::KeyRejected(format!(
                    "FMP API Key 被拒绝: {}",
                    sanitize_fmp_error_detail(&err_msg)
                )));
            }
            return Err(FmpFetchError::NonRetryable(format_fmp_provider_error(
                status,
                &format!("Error Message: {err_msg}"),
            )));
        }

        Ok(response_json)
    }

    fn build_url(&self, data_type: &str, ticker: &str) -> Result<String, String> {
        match data_type {
            "quote" => Ok(format!(
                "{}/v3/quote/{}",
                self.base_url,
                encode_fmp_symbols(ticker, false)?
            )),
            "quote_short" => Ok(format!(
                "{}/stable/batch-quote-short?symbols={}",
                self.stable_base_url(),
                encode_fmp_symbols(ticker, false)?
            )),
            "extended_hours" => Ok(format!(
                "{}/v3/historical-chart/1min/{}?extended=true",
                self.base_url,
                encode_fmp_symbols(ticker, false)?
            )),
            "profile" => Ok(format!(
                "{}/v3/profile/{}",
                self.base_url,
                encode_fmp_symbols(ticker, false)?
            )),
            "search" => {
                let ticker = validated_data_fetch_search_query(ticker)?;
                let query =
                    url::form_urlencoded::byte_serialize(ticker.as_bytes()).collect::<String>();
                Ok(format!(
                    "{}/v3/search?query={}&limit=10",
                    self.base_url, query
                ))
            }
            "financials" => Ok(format!(
                "{}/v3/income-statement/{}?limit=4",
                self.base_url,
                encode_fmp_symbols(ticker, false)?
            )),
            "news" => {
                if ticker.is_empty() {
                    Ok(format!("{}/v3/stock_news?limit=10", self.base_url))
                } else {
                    Ok(format!(
                        "{}/v3/stock_news?tickers={}&limit=10",
                        self.base_url,
                        encode_fmp_symbols(ticker, false)?
                    ))
                }
            }
            "gainers_losers" => Ok(format!("{}/v3/stock_market/actives", self.base_url)),
            "sector_performance" => Ok(format!("{}/v3/sector-performance", self.base_url)),
            "crypto_quote" => Ok(format!(
                "{}/v3/quote/{}",
                self.base_url,
                encode_fmp_symbols(ticker, false)?
            )),
            "etf_holdings" => Ok(format!(
                "{}/v3/etf-holder/{}",
                self.base_url,
                encode_fmp_symbols(ticker, false)?
            )),
            "earnings_calendar" => Err(
                "earnings_calendar 需要显式窗口，通过 build_earnings_calendar_url 构造".to_string(),
            ),
            "earnings_outlook" => Err(
                "earnings_outlook 通过证券级财报、预期、目标价、评级和行情聚合获取，不映射单一端点"
                    .to_string(),
            ),
            "snapshot" => {
                Err("snapshot 通过聚合 quote/profile/news 获取，不映射单一端点".to_string())
            }
            _ => Err(format!("不支持的数据类型: {data_type}")),
        }
    }

    fn resolve_earnings_window(&self, args: &Value) -> Result<(NaiveDate, NaiveDate), String> {
        let today = hone_core::beijing_now().date_naive();
        let default_to = today + Duration::days(14);

        let from = if let Some(value) = args.get("from").and_then(|v| v.as_str()) {
            NaiveDate::parse_from_str(value, "%Y-%m-%d")
                .map_err(|err| format!("from 日期格式无效，应为 YYYY-MM-DD: {err}"))?
        } else {
            today
        };
        let to = if let Some(value) = args.get("to").and_then(|v| v.as_str()) {
            NaiveDate::parse_from_str(value, "%Y-%m-%d")
                .map_err(|err| format!("to 日期格式无效，应为 YYYY-MM-DD: {err}"))?
        } else {
            default_to
        };

        if to < from {
            return Err("earnings_calendar 的 to 日期不能早于 from 日期".to_string());
        }

        Ok((from, to))
    }

    fn build_earnings_calendar_url(&self, from: NaiveDate, to: NaiveDate) -> String {
        format!(
            "{}/v3/earning_calendar?from={}&to={}",
            self.base_url,
            from.format("%Y-%m-%d"),
            to.format("%Y-%m-%d")
        )
    }

    /// Every aggregate data type is a list of `(key, path)` pairs against the
    /// stable API. Adding a capability means adding a row to
    /// `stable_bundle_components`, not another hand-written `tokio::join!` and
    /// another coverage map — the reason earlier gaps were closed one endpoint
    /// at a time is that each one cost a dispatcher change.
    fn stable_bundle_components(
        &self,
        data_type: &str,
        symbol: &str,
        args: &Value,
    ) -> Option<Vec<(&'static str, String)>> {
        let stable = self.stable_base_url();
        let s = |path: &str| format!("{stable}/stable/{path}");
        let limit = args
            .get("limit")
            .and_then(Value::as_u64)
            .unwrap_or(0)
            .clamp(0, 100);
        Some(match data_type {
            // Official trailing metrics, ratios, enterprise value and the
            // published health scores. Hone used to recompute a subset of this
            // by hand and had no source at all for the rest.
            "valuation" => vec![
                (
                    "key_metrics_ttm",
                    s(&format!("key-metrics-ttm?symbol={symbol}")),
                ),
                ("ratios_ttm", s(&format!("ratios-ttm?symbol={symbol}"))),
                (
                    "enterprise_value",
                    s(&format!("enterprise-values?symbol={symbol}&limit=2")),
                ),
                (
                    "financial_scores",
                    s(&format!("financial-scores?symbol={symbol}")),
                ),
                ("shares_float", s(&format!("shares-float?symbol={symbol}"))),
                (
                    "discounted_cash_flow",
                    s(&format!("discounted-cash-flow?symbol={symbol}")),
                ),
                (
                    "market_capitalization",
                    s(&format!("market-capitalization?symbol={symbol}")),
                ),
            ],
            // Which product lines and which regions the revenue comes from.
            "segments" => vec![
                (
                    "product_segmentation",
                    s(&format!("revenue-product-segmentation?symbol={symbol}")),
                ),
                (
                    "geographic_segmentation",
                    s(&format!("revenue-geographic-segmentation?symbol={symbol}")),
                ),
            ],
            "ownership" => vec![
                (
                    "institutional_positions",
                    s(&format!(
                        "institutional-ownership/symbol-positions-summary?symbol={symbol}"
                    )),
                ),
                (
                    "insider_statistics",
                    s(&format!("insider-trading/statistics?symbol={symbol}")),
                ),
                (
                    "insider_trades",
                    s(&format!(
                        "insider-trading/search?symbol={symbol}&page=0&limit=20"
                    )),
                ),
            ],
            "corporate_actions" => vec![
                (
                    "dividends",
                    s(&format!("dividends?symbol={symbol}&limit=12")),
                ),
                ("splits", s(&format!("splits?symbol={symbol}&limit=8"))),
            ],
            "press_releases" => vec![(
                "press_releases",
                s(&format!(
                    "news/press-releases?symbols={symbol}&page=0&limit={}",
                    if limit == 0 { 10 } else { limit }
                )),
            )],
            "transcript" => {
                let mut components = vec![(
                    "transcript_dates",
                    s(&format!("earning-call-transcript-dates?symbol={symbol}")),
                )];
                // Management's own words for a named quarter, when the caller
                // already knows which one it wants.
                if let (Some(year), Some(quarter)) = (
                    args.get("year").and_then(Value::as_u64),
                    args.get("quarter").and_then(Value::as_u64),
                ) {
                    components.push((
                        "transcript",
                        s(&format!(
                            "earning-call-transcript?symbol={symbol}&year={year}&quarter={quarter}"
                        )),
                    ));
                }
                components
            }
            // Macro context is symbol-independent.
            "macro" => vec![
                ("treasury_rates", s("treasury-rates")),
                ("gdp", s("economic-indicators?name=GDP")),
                ("cpi", s("economic-indicators?name=CPI")),
                (
                    "unemployment",
                    s("economic-indicators?name=unemploymentRate"),
                ),
                ("federal_funds", s("economic-indicators?name=federalFunds")),
            ],
            "market_hours" => vec![("all_exchange_market_hours", s("all-exchange-market-hours"))],
            _ => return None,
        })
    }

    /// The sector/industry P/E snapshot is what turns "43x" into "43x against
    /// an industry at 21x". It needs the profile's industry, so it is fetched
    /// after the peer stage rather than as a plain component.
    async fn fetch_sector_industry_pe(&self, payload: &Value) -> Option<Value> {
        let industry = payload
            .pointer("/data/peer_quotes/0/industry")
            .or_else(|| payload.pointer("/data/peers/0/industry"))
            .and_then(Value::as_str)?;
        let encoded = urlencoding_encode(industry);
        self.fetch_from_url_cached(
            &format!(
                "{}/stable/industry-pe-snapshot?industry={encoded}",
                self.stable_base_url()
            ),
            ttl_for_data_type("valuation"),
            "valuation",
        )
        .await
        .ok()
        .filter(has_meaningful_fmp_value)
    }

    /// Runs every component of a bundle concurrently and reports, per key,
    /// whether it actually returned something. A component that failed stays a
    /// disclosed gap instead of vanishing.
    async fn fetch_stable_bundle(
        &self,
        data_type: &str,
        components: Vec<(&'static str, String)>,
    ) -> (Value, Value, Option<Value>) {
        let ttl = ttl_for_data_type(data_type);
        let results = futures::future::join_all(
            components
                .iter()
                .map(|(_, url)| self.fetch_from_url_cached(url, ttl, data_type)),
        )
        .await;

        let mut data = serde_json::Map::new();
        let mut coverage = serde_json::Map::new();
        let mut errors = serde_json::Map::new();
        for ((key, _), result) in components.iter().zip(results.into_iter()) {
            match result {
                Ok(value) if has_meaningful_fmp_value(&value) => {
                    coverage.insert((*key).to_string(), Value::String("available".to_string()));
                    data.insert((*key).to_string(), value);
                }
                Ok(_) => {
                    coverage.insert((*key).to_string(), Value::String("empty".to_string()));
                }
                Err(err) => {
                    coverage.insert((*key).to_string(), Value::String("unavailable".to_string()));
                    errors.insert((*key).to_string(), Value::String(err));
                }
            }
        }
        (
            Value::Object(data),
            Value::Object(coverage),
            (!errors.is_empty()).then(|| Value::Object(errors)),
        )
    }

    fn build_financials_component_url(
        &self,
        component: &str,
        ticker: &str,
    ) -> Result<String, String> {
        let symbol = encode_fmp_symbols(ticker, false)?;
        let base = &self.base_url;
        match component {
            "income_annual" => Ok(format!("{base}/v3/income-statement/{symbol}?limit=4")),
            "income_quarter" => Ok(format!(
                "{base}/v3/income-statement/{symbol}?period=quarter&limit=8"
            )),
            "balance_sheet_quarter" => Ok(format!(
                "{base}/v3/balance-sheet-statement/{symbol}?period=quarter&limit=5"
            )),
            "cash_flow_quarter" => Ok(format!(
                "{base}/v3/cash-flow-statement/{symbol}?period=quarter&limit=8"
            )),
            "analyst_estimates" => Ok(format!(
                "{}/stable/analyst-estimates?symbol={symbol}&period=quarter&page=0&limit=8",
                self.stable_base_url()
            )),
            "financial_growth" => Ok(format!(
                "{}/stable/financial-growth?symbol={symbol}&period=quarter&limit=8",
                self.stable_base_url()
            )),
            _ => Err(format!("不支持的 financials 组件: {component}")),
        }
    }

    /// One income statement is not financial evidence. A quarter-over-quarter
    /// read, an operating cash-flow line and a balance sheet are what separate
    /// "revenue was X" from an answer that can discuss the business, so all of
    /// them are fetched together rather than left for a follow-up round that
    /// the research budget may never grant.
    async fn fetch_financials_bundle(&self, ticker: &str) -> Result<Value, String> {
        let annual_url = self.build_financials_component_url("income_annual", ticker)?;
        let quarter_url = self.build_financials_component_url("income_quarter", ticker)?;
        let balance_url = self.build_financials_component_url("balance_sheet_quarter", ticker)?;
        let cash_flow_url = self.build_financials_component_url("cash_flow_quarter", ticker)?;
        let estimates_url = self.build_financials_component_url("analyst_estimates", ticker)?;
        let growth_url = self.build_financials_component_url("financial_growth", ticker)?;
        let ttl = ttl_for_data_type("financials");
        // Every component keeps the `financials` cache label: the caching
        // policy refuses to memoize an empty payload for this data type, and a
        // component-specific label would silently opt out of that rule.
        let (annual, quarterly, balance_sheet, cash_flow, estimates, growth) = tokio::join!(
            self.fetch_from_url_cached(&annual_url, ttl, "financials"),
            self.fetch_from_url_cached(&quarter_url, ttl, "financials"),
            self.fetch_from_url_cached(&balance_url, ttl, "financials"),
            self.fetch_from_url_cached(&cash_flow_url, ttl, "financials"),
            self.fetch_from_url_cached(&estimates_url, ttl, "financials"),
            self.fetch_from_url_cached(&growth_url, ttl, "financials"),
        );
        // The annual statement is the only component the older evidence
        // normalizer reads, so a total failure there stays a failure.
        let annual = annual?;
        Ok(build_financials_bundle(
            annual,
            quarterly.ok(),
            balance_sheet.ok(),
            cash_flow.ok(),
            estimates.ok(),
            growth.ok(),
        ))
    }

    fn build_earnings_outlook_url(&self, component: &str, ticker: &str) -> Result<String, String> {
        let symbol = encode_fmp_symbols(ticker, false)?;
        let stable = self.stable_base_url();
        match component {
            "earnings" => Ok(format!("{stable}/stable/earnings?symbol={symbol}")),
            "analyst_estimates" => Ok(format!(
                "{stable}/stable/analyst-estimates?symbol={symbol}&period=quarter&page=0&limit=8"
            )),
            "price_target_consensus" => Ok(format!(
                "{stable}/stable/price-target-consensus?symbol={symbol}"
            )),
            "ratings_snapshot" => Ok(format!("{stable}/stable/ratings-snapshot?symbol={symbol}")),
            "price_target_summary" => Ok(format!(
                "{stable}/stable/price-target-summary?symbol={symbol}"
            )),
            "grades_consensus" => Ok(format!("{stable}/stable/grades-consensus?symbol={symbol}")),
            _ => Err(format!("不支持的 earnings_outlook 组件: {component}")),
        }
    }

    fn stable_base_url(&self) -> String {
        self.base_url
            .strip_suffix("/api")
            .unwrap_or(&self.base_url)
            .trim_end_matches('/')
            .to_string()
    }

    async fn fetch_data_type(&self, data_type: &str, ticker: &str) -> Result<Value, String> {
        let url = self.build_url(data_type, ticker)?;
        self.fetch_from_url_cached(&url, ttl_for_data_type(data_type), data_type)
            .await
    }

    async fn fetch_from_url_cached(
        &self,
        url: &str,
        ttl: Option<StdDuration>,
        data_type: &str,
    ) -> Result<Value, String> {
        let cache_key = fmp_cache_key_for_url(url);
        if let Some(ttl) = ttl
            && let Some(value) = self.cached_value(&cache_key)
        {
            tracing::info!(
                tool = "data_fetch",
                data_type,
                cache_key = %cache_key,
                ttl_secs = ttl.as_secs(),
                "FMP data_fetch cache hit"
            );
            return Ok(value);
        }

        let mut last_err = String::new();

        for key in &self.keys {
            match self.fetch_with_key(key, &url).await {
                Ok(data) => {
                    if let Some(ttl) = ttl
                        && should_cache_fmp_value(data_type, &data)
                    {
                        self.store_cache_value(cache_key.clone(), ttl, data.clone());
                    }
                    return Ok(data);
                }
                Err(FmpFetchError::KeyRejected(error)) => last_err = error,
                Err(FmpFetchError::NonRetryable(error)) => return Err(error),
            }
        }

        Err(format!(
            "所有 FMP API Key 均失败（共 {} 个）。最后错误：{}",
            self.keys.len(),
            last_err
        ))
    }

    fn cached_value(&self, cache_key: &str) -> Option<Value> {
        let Ok(mut cache) = self.cache.lock() else {
            return None;
        };
        let Some(entry) = cache.get(cache_key) else {
            return None;
        };
        if entry.expires_at <= Instant::now() {
            cache.remove(cache_key);
            return None;
        }
        Some(entry.value.clone())
    }

    fn store_cache_value(&self, cache_key: String, ttl: StdDuration, value: Value) {
        let Ok(mut cache) = self.cache.lock() else {
            return;
        };
        cache.insert(
            cache_key,
            CachedFmpValue {
                expires_at: Instant::now() + ttl,
                value,
            },
        );
    }

    fn build_snapshot_response(
        &self,
        ticker: &str,
        quote: Result<Value, String>,
        profile: Result<Value, String>,
        news: Result<Value, String>,
    ) -> Value {
        let mut errors = serde_json::Map::new();

        let quote_value = match quote {
            Ok(value) => value,
            Err(err) => {
                errors.insert("quote".to_string(), Value::String(err));
                Value::Null
            }
        };
        let profile_value = match profile {
            Ok(value) => value,
            Err(err) => {
                errors.insert("profile".to_string(), Value::String(err));
                Value::Null
            }
        };
        let news_value = match news {
            Ok(value) => value,
            Err(err) => {
                errors.insert("news".to_string(), Value::String(err));
                Value::Null
            }
        };

        let all_failed = quote_value.is_null() && profile_value.is_null() && news_value.is_null();
        let listing_evidence = security_listing_evidence(ticker, &quote_value, &profile_value);

        let mut payload = serde_json::json!({
            "data_type": "snapshot",
            "ticker": ticker,
            "data": {
                "quote": quote_value,
                "profile": profile_value,
                "news": news_value,
            },
            "hone_security_listing_evidence": listing_evidence,
        });

        if !errors.is_empty() {
            payload["errors"] = Value::Object(errors);
        }
        if all_failed {
            payload["error"] =
                Value::String("snapshot 聚合失败：quote/profile/news 均未获取成功".to_string());
        }

        payload
    }

    fn build_earnings_outlook_response(
        &self,
        ticker: &str,
        quote: Result<Value, String>,
        profile: Result<Value, String>,
        earnings: Result<Value, String>,
        analyst_estimates: Result<Value, String>,
        price_target_consensus: Result<Value, String>,
        ratings_snapshot: Result<Value, String>,
        financials: Result<Value, String>,
    ) -> Value {
        let mut errors = serde_json::Map::new();
        let mut component = |name: &str, result: Result<Value, String>| match result {
            Ok(value) => value,
            Err(err) => {
                errors.insert(name.to_string(), Value::String(err));
                Value::Null
            }
        };

        let quote_value = component("quote", quote);
        let profile_value = component("profile", profile);
        let earnings_value = component("earnings", earnings);
        let estimates_value = component("analyst_estimates", analyst_estimates);
        let target_value = component("price_target_consensus", price_target_consensus);
        let ratings_value = component("ratings_snapshot", ratings_snapshot);
        let financials_value = component("financials", financials);
        let current_price = first_positive_number(&quote_value, &["price"]);
        let target_quality = price_target_consensus_quality(&target_value, current_price);
        let listing_evidence = security_listing_evidence(ticker, &quote_value, &profile_value);
        let valuation_basis = valuation_basis_quality(&quote_value, &financials_value);

        let coverage = [
            ("quote", &quote_value),
            ("profile", &profile_value),
            ("earnings", &earnings_value),
            ("analyst_estimates", &estimates_value),
            ("price_target_consensus", &target_value),
            ("ratings_snapshot", &ratings_value),
            ("financials", &financials_value),
        ]
        .into_iter()
        .map(|(name, value)| {
            let available = if name == "financials" {
                financials_component_available(value)
            } else {
                has_meaningful_fmp_value(value)
            };
            (
                name.to_string(),
                Value::String(
                    if available {
                        "available"
                    } else {
                        "unavailable"
                    }
                    .to_string(),
                ),
            )
        })
        .collect::<serde_json::Map<_, _>>();

        let all_failed = [
            &quote_value,
            &profile_value,
            &earnings_value,
            &estimates_value,
            &target_value,
            &ratings_value,
        ]
        .into_iter()
        .all(|value| !has_meaningful_fmp_value(value))
            && !financials_component_available(&financials_value);

        let mut payload = serde_json::json!({
            "data_type": "earnings_outlook",
            "ticker": ticker,
            "data": {
                "quote": quote_value,
                "profile": profile_value,
                "earnings": earnings_value,
                "analyst_estimates": estimates_value,
                "price_target_consensus": target_value,
                "ratings_snapshot": ratings_value,
                "financials": financials_value,
            },
            "coverage": Value::Object(coverage),
            "hone_security_listing_evidence": listing_evidence,
            "hone_target_consensus_quality": target_quality,
            "hone_valuation_basis": valuation_basis,
            "evidence_policy": "Use only component fields whose Hone quality flags authorize that claim type. Missing or quarantined components must be disclosed; do not infer them from another component. An active_listing result is current-turn provider evidence and must not be contradicted by stale acquisition or delisting memory."
        });

        if !errors.is_empty() {
            payload["errors"] = Value::Object(errors);
        }
        if all_failed {
            payload["error"] = Value::String(
                "earnings_outlook 聚合失败：所有证券级财报证据组件均未获取成功".to_string(),
            );
        }
        payload
    }
}

fn format_fmp_transport_error(operation: &str, error: &reqwest::Error) -> String {
    let detail = sanitize_fmp_error_detail(&error.to_string());
    if detail.is_empty() {
        format!("FMP {operation}失败")
    } else {
        format!("FMP {operation}失败: {detail}")
    }
}

fn format_fmp_provider_error(status: reqwest::StatusCode, body: &str) -> String {
    let body_prefix = sanitize_fmp_error_detail(body)
        .chars()
        .take(200)
        .collect::<String>();
    if body_prefix.trim().is_empty() {
        format!("FMP provider error（HTTP {}）", status.as_u16())
    } else {
        format!(
            "FMP provider error（HTTP {}）: body_prefix={body_prefix}",
            status.as_u16()
        )
    }
}

fn sanitize_fmp_error_detail(text: &str) -> String {
    let redacted = redact_fmp_query_secrets(&redact_url_userinfo(text));
    if redacted.chars().count() <= MAX_FMP_TRANSPORT_ERROR_CHARS {
        return redacted;
    }
    redacted
        .chars()
        .take(MAX_FMP_TRANSPORT_ERROR_CHARS)
        .collect::<String>()
        + "..."
}

fn ttl_for_data_type(data_type: &str) -> Option<StdDuration> {
    match data_type {
        "extended_hours" => Some(FMP_TTL_EXTENDED_HOURS),
        "quote" | "quote_short" | "crypto_quote" | "gainers_losers" | "sector_performance" => {
            Some(FMP_TTL_FAST)
        }
        "news" => Some(FMP_TTL_NEWS),
        "profile" | "search" | "etf_holdings" => Some(FMP_TTL_PROFILE),
        "financials" | "valuation" | "segments" | "ownership" | "corporate_actions" => {
            Some(FMP_TTL_FINANCIALS)
        }
        "peers" | "market_hours" | "macro" => Some(FMP_TTL_PROFILE),
        "press_releases" | "transcript" => Some(FMP_TTL_NEWS),
        "earnings_calendar" | "earnings_outlook" => Some(FMP_TTL_EARNINGS),
        _ => None,
    }
}

fn should_cache_fmp_value(data_type: &str, value: &Value) -> bool {
    if !matches!(
        data_type,
        "financials"
            | "valuation"
            | "segments"
            | "peers"
            | "ownership"
            | "corporate_actions"
            | "press_releases"
            | "transcript"
            | "profile"
            | "search"
            | "etf_holdings"
            | "quote"
            | "quote_short"
            | "extended_hours"
            | "crypto_quote"
    ) {
        return true;
    }

    has_meaningful_fmp_value(value)
}

fn normalize_extended_hours_bar(ticker: &str, response: &Value) -> Result<Value, String> {
    let bars = response
        .as_array()
        .ok_or_else(|| "FMP 盘前盘后行情响应格式无效：预期为分钟 K 线数组".to_string())?;

    let mut parsed = bars
        .iter()
        .filter_map(|bar| {
            let date = bar.get("date")?.as_str()?.trim();
            let timestamp = parse_extended_hours_timestamp(date)?;
            let price = bar.get("close")?.as_f64().filter(|value| *value > 0.0)?;
            let high = bar.get("high")?.as_f64().filter(|value| *value > 0.0)?;
            let low = bar.get("low")?.as_f64().filter(|value| *value > 0.0)?;
            let open = bar
                .get("open")
                .and_then(Value::as_f64)
                .filter(|value| *value > 0.0)
                .unwrap_or(price);
            let volume = bar.get("volume")?.as_f64().filter(|value| *value >= 0.0)?;
            Some((timestamp, date.to_string(), open, price, high, low, volume))
        })
        .collect::<Vec<_>>();
    parsed.sort_by_key(|(timestamp, ..)| *timestamp);

    let latest = parsed
        .last()
        .cloned()
        .ok_or_else(|| "FMP 盘前盘后行情没有可用的最新分钟 bar".to_string())?;

    // A single latest bar cannot answer "盘后跌了多少": at a New York
    // pre-market morning the latest bar is a pre bar, and the post session
    // where the move happened would be discarded. Summarize every session
    // window in the data instead. Keep the adjacent-window calculation for
    // audit, but also publish session-specific canonical denominators so a
    // regular close can never be compared with that morning's pre-market bar
    // and mislabeled as the daily close-to-close move.
    let mut windows: Vec<(
        chrono::NaiveDate,
        u8,
        &'static str,
        f64,
        f64,
        f64,
        f64,
        f64,
        i64,
    )> = Vec::new();
    for (timestamp, _, open, close, high, low, volume) in &parsed {
        let session = extended_hours_session(timestamp.time());
        if session == "closed" {
            continue;
        }
        let order = match session {
            "pre" => 0u8,
            "regular" => 1,
            _ => 2,
        };
        let date = timestamp.date();
        match windows
            .iter_mut()
            .find(|(d, o, ..)| *d == date && *o == order)
        {
            Some(window) => {
                window.4 = *close;
                window.5 = window.5.max(*high);
                window.6 = window.6.min(*low);
                window.7 += *volume;
                window.8 = timestamp.and_utc().timestamp();
            }
            None => windows.push((
                date,
                order,
                session,
                *open,
                *close,
                *high,
                *low,
                *volume,
                timestamp.and_utc().timestamp(),
            )),
        }
    }
    windows.sort_by_key(|(date, order, ..)| (*date, *order));
    let start = windows.len().saturating_sub(8);
    let windows = &windows[start..];

    let mut summaries = Vec::with_capacity(windows.len());
    let mut previous_session_close: Option<f64> = None;
    let mut previous_regular_close: Option<f64> = None;
    let mut latest_regular_date: Option<NaiveDate> = None;
    for (date, _, session, open, close, high, low, volume, _) in windows {
        let mut summary = serde_json::json!({
            "date_new_york": date.format("%Y-%m-%d").to_string(),
            "session": session,
            "open": open,
            "close": close,
            "high": high,
            "low": low,
            "volume": volume,
        });
        if let Some(reference) = previous_session_close.filter(|reference| *reference > 0.0) {
            let pct = (close - reference) / reference * 100.0;
            summary["pct_change_vs_prev_session_close"] =
                serde_json::json!((pct * 100.0).round() / 100.0);
        }
        match *session {
            "pre" => {
                if let Some(reference) = previous_regular_close.filter(|value| *value > 0.0) {
                    let pct = (close - reference) / reference * 100.0;
                    summary["pct_change_vs_previous_regular_close"] =
                        serde_json::json!((pct * 100.0).round() / 100.0);
                    summary["canonical_change_basis"] =
                        Value::String("premarket_close_vs_previous_regular_close".to_string());
                }
            }
            "regular" => {
                if let Some(reference) = previous_regular_close.filter(|value| *value > 0.0) {
                    let pct = (close - reference) / reference * 100.0;
                    summary["pct_change_close_to_close"] =
                        serde_json::json!((pct * 100.0).round() / 100.0);
                    summary["previous_regular_close"] = serde_json::json!(reference);
                    summary["canonical_change_basis"] =
                        Value::String("regular_close_vs_previous_regular_close".to_string());
                }
                previous_regular_close = Some(*close);
                latest_regular_date = Some(*date);
            }
            "post" if latest_regular_date == Some(*date) => {
                if let Some(reference) = previous_regular_close.filter(|value| *value > 0.0) {
                    let pct = (close - reference) / reference * 100.0;
                    summary["pct_change_vs_regular_close"] =
                        serde_json::json!((pct * 100.0).round() / 100.0);
                    summary["canonical_change_basis"] =
                        Value::String("postmarket_close_vs_same_day_regular_close".to_string());
                }
            }
            _ => {}
        }
        previous_session_close = Some(*close);
        summaries.push(summary);
    }

    let now_new_york = hone_core::beijing_now().with_timezone(&chrono_tz::America::New_York);
    Ok(serde_json::json!({
        "symbol": ticker.trim().to_ascii_uppercase(),
        "price": latest.3,
        "date": latest.1,
        "session": extended_hours_session(latest.0.time()),
        "high": latest.4,
        "low": latest.5,
        "volume": latest.6,
        "hone_session_summaries": summaries,
        "hone_session_change_policy": "常规交易日涨跌只使用 regular 窗口的 pct_change_close_to_close（本日常规收盘相对上一常规收盘）；盘前只使用 pct_change_vs_previous_regular_close；盘后只使用 pct_change_vs_regular_close。pct_change_vs_prev_session_close 仅描述相邻窗口变化，尤其 regular 相对 pre 的数值绝不能写成日涨跌幅。",
        "hone_now_new_york": now_new_york.format("%Y-%m-%d %H:%M %Z").to_string(),
        "hone_now_session": extended_hours_session(now_new_york.time()),
    }))
}

fn parse_extended_hours_timestamp(value: &str) -> Option<NaiveDateTime> {
    if let Ok(timestamp) = DateTime::parse_from_rfc3339(value) {
        return Some(timestamp.naive_local());
    }

    for format in [
        "%Y-%m-%d %H:%M:%S%.f",
        "%Y-%m-%dT%H:%M:%S%.f",
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%dT%H:%M:%S",
    ] {
        if let Ok(timestamp) = NaiveDateTime::parse_from_str(value, format) {
            return Some(timestamp);
        }
    }

    None
}

fn extended_hours_session(time: NaiveTime) -> &'static str {
    let pre_open = NaiveTime::from_hms_opt(4, 0, 0).expect("valid premarket open time");
    let regular_open = NaiveTime::from_hms_opt(9, 30, 0).expect("valid market open time");
    let regular_close = NaiveTime::from_hms_opt(16, 0, 0).expect("valid market close time");
    let post_close = NaiveTime::from_hms_opt(20, 0, 0).expect("valid postmarket close time");

    if time >= pre_open && time < regular_open {
        "pre"
    } else if time >= regular_open && time <= regular_close {
        "regular"
    } else if time > regular_close && time <= post_close {
        "post"
    } else {
        "closed"
    }
}

fn normalize_quote_timestamp_metadata(mut value: Value) -> Value {
    match &mut value {
        Value::Array(items) => {
            for item in items {
                attach_quote_timestamp_metadata(item);
                attach_quote_evidence_quality(item);
            }
        }
        Value::Object(_) => {
            attach_quote_timestamp_metadata(&mut value);
            attach_quote_evidence_quality(&mut value);
        }
        _ => {}
    }
    value
}

fn parse_nasdaq_display_number(value: &str) -> Option<f64> {
    value
        .trim()
        .trim_start_matches('$')
        .trim_end_matches('%')
        .replace(',', "")
        .parse::<f64>()
        .ok()
        .filter(|number| number.is_finite())
}

fn nasdaq_quote_timestamp(value: &str) -> Option<i64> {
    let value = value.trim();
    let date = NaiveDate::parse_from_str(value, "%b %e, %Y")
        .or_else(|_| NaiveDate::parse_from_str(value, "%b %d, %Y"))
        .ok()?;
    // Nasdaq's delayed public endpoint currently exposes the market date but
    // not an intraday clock. Normalize a completed regular-session quote to
    // the official 16:00 New York close and label that transformation below;
    // callers must not describe it as a realtime timestamp.
    chrono_tz::America::New_York
        .from_local_datetime(&date.and_hms_opt(16, 0, 0)?)
        .single()
        .map(|timestamp| timestamp.timestamp())
}

fn nasdaq_quote_row(expected_symbol: &str, payload: &Value) -> Option<Value> {
    let data = payload.get("data")?;
    let symbol = data.get("symbol")?.as_str()?.trim();
    if !hone_core::provider_symbols_equivalent(expected_symbol, symbol) {
        return None;
    }
    let primary = data.get("primaryData")?;
    let price = parse_nasdaq_display_number(primary.get("lastSalePrice")?.as_str()?)?;
    if price <= 0.0 {
        return None;
    }
    let change = primary
        .get("netChange")
        .and_then(Value::as_str)
        .and_then(parse_nasdaq_display_number);
    let change_percentage = primary
        .get("percentageChange")
        .and_then(Value::as_str)
        .and_then(parse_nasdaq_display_number);
    let market_date = primary
        .get("lastTradeTimestamp")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let timestamp = market_date.and_then(nasdaq_quote_timestamp);
    let previous_close = change
        .map(|change| price - change)
        .filter(|value| value.is_finite() && *value > 0.0);
    let mut row = serde_json::json!({
        "symbol": symbol,
        "name": data.get("companyName").cloned().unwrap_or(Value::Null),
        "price": price,
        "change": change,
        "changesPercentage": change_percentage,
        "previousClose": previous_close,
        "timestamp": timestamp,
        "exchange": data.get("exchange").cloned().unwrap_or(Value::Null),
        "currency": "USD",
        "volume": primary
            .get("volume")
            .and_then(Value::as_str)
            .and_then(parse_nasdaq_display_number),
        "hone_source": {
            "provider": "Nasdaq official quote API",
            "url_scope": "api.nasdaq.com/api/quote/{symbol}/info",
            "is_realtime": primary.get("isRealTime").and_then(Value::as_bool),
            "provider_market_date": market_date,
            "coverage": "price_and_identity_only",
        },
    });
    if let Some(timestamp) = timestamp
        && let Some(utc) = DateTime::from_timestamp(timestamp, 0)
    {
        let new_york = utc.with_timezone(&chrono_tz::America::New_York);
        let beijing = utc.with_timezone(&chrono_tz::Asia::Shanghai);
        row["hone_quote_time"] = serde_json::json!({
            "unix_seconds": timestamp,
            "new_york": new_york.format("%Y-%m-%d %H:%M:%S %:z").to_string(),
            "beijing": beijing.format("%Y-%m-%d %H:%M:%S %:z").to_string(),
            "market_date_new_york": new_york.format("%Y-%m-%d").to_string(),
            "source": "Nasdaq provider market date normalized to the 16:00 New York regular close; delayed close, not a realtime intraday timestamp"
        });
    }
    attach_quote_evidence_quality(&mut row);
    Some(row)
}

fn normalize_sec_company_facts(symbol: &str, cik: u64, source_url: &str, payload: &Value) -> Value {
    let us_gaap = payload.pointer("/facts/us-gaap").unwrap_or(&Value::Null);
    let dei = payload.pointer("/facts/dei").unwrap_or(&Value::Null);
    let definitions: [(&str, &Value, &[&str]); 13] = [
        (
            "revenue",
            us_gaap,
            &[
                "RevenueFromContractWithCustomerExcludingAssessedTax",
                "Revenues",
                "SalesRevenueNet",
            ],
        ),
        ("gross_profit", us_gaap, &["GrossProfit"]),
        ("operating_income", us_gaap, &["OperatingIncomeLoss"]),
        ("net_income", us_gaap, &["NetIncomeLoss"]),
        (
            "operating_cash_flow",
            us_gaap,
            &["NetCashProvidedByUsedInOperatingActivities"],
        ),
        (
            "capital_expenditures",
            us_gaap,
            &["PaymentsToAcquirePropertyPlantAndEquipment"],
        ),
        (
            "cash_and_equivalents",
            us_gaap,
            &[
                "CashAndCashEquivalentsAtCarryingValue",
                "CashCashEquivalentsRestrictedCashAndRestrictedCashEquivalents",
            ],
        ),
        ("long_term_debt_current", us_gaap, &["LongTermDebtCurrent"]),
        (
            "long_term_debt_noncurrent",
            us_gaap,
            &["LongTermDebtNoncurrent", "LongTermDebt"],
        ),
        (
            "interest_expense",
            us_gaap,
            &["InterestExpenseDebt", "InterestExpenseNonOperating"],
        ),
        (
            "stockholders_equity",
            us_gaap,
            &[
                "StockholdersEquity",
                "StockholdersEquityIncludingPortionAttributableToNoncontrollingInterest",
            ],
        ),
        (
            "remaining_performance_obligation",
            us_gaap,
            &["RevenueRemainingPerformanceObligation"],
        ),
        (
            "shares_outstanding",
            dei,
            &["EntityCommonStockSharesOutstanding"],
        ),
    ];

    let mut metrics = serde_json::Map::new();
    let mut latest_metric_summary = serde_json::Map::new();
    let mut latest_filed: Option<String> = None;
    for (name, namespace, tags) in definitions {
        let Some((tag, fact)) = tags
            .iter()
            .find_map(|tag| namespace.get(*tag).map(|fact| (*tag, fact)))
        else {
            metrics.insert(
                name.to_string(),
                serde_json::json!({"status": "not_reported"}),
            );
            continue;
        };
        let observations = sec_fact_observations(fact, 5);
        for filed in observations
            .iter()
            .filter_map(|row| row.get("filed").and_then(Value::as_str))
        {
            if latest_filed
                .as_deref()
                .is_none_or(|current| filed > current)
            {
                latest_filed = Some(filed.to_string());
            }
        }
        if let Some(latest) = observations.first() {
            latest_metric_summary.insert(name.to_string(), latest.clone());
        }
        metrics.insert(
            name.to_string(),
            serde_json::json!({
                "status": if observations.is_empty() { "not_reported" } else { "available" },
                "taxonomy_tag": tag,
                "label": fact.get("label").cloned().unwrap_or(Value::Null),
                "description": fact.get("description").cloned().unwrap_or(Value::Null),
                "observations": observations,
                "concept_source_url": format!(
                    "https://data.sec.gov/api/xbrl/companyconcept/CIK{cik:010}/{}/{tag}.json",
                    if std::ptr::eq(namespace, us_gaap) { "us-gaap" } else { "dei" }
                ),
            }),
        );
    }

    serde_json::json!({
        "data_type": "financials",
        "ticker": symbol,
        "data": {
            "entity_name": payload.get("entityName").cloned().unwrap_or(Value::Null),
            "cik": cik,
            "fiscal_year_end": payload.get("fiscalYearEnd").cloned().unwrap_or(Value::Null),
            "latest_filed_date": latest_filed,
            "latest_metric_summary": latest_metric_summary,
            "metrics": metrics,
        },
        "fallback_source": "SEC Company Facts API",
        "source_url": source_url,
        "coverage": "filed_historical_facts_only_without_fmp",
        "hone_sec_fact_policy": "Each observation preserves start/end, filed date, SEC form, fiscal period and frame. A duration longer than roughly one quarter may be year-to-date or annual; do not present it as a standalone quarter. Compare facts only when their periods and units align. SEC Company Facts does not provide management guidance, analyst estimates or a complete debt maturity schedule.",
    })
}

#[derive(Debug)]
struct SecEarningsFilingRow {
    form: String,
    filing_date: String,
    report_date: String,
    accession: String,
    primary_document: String,
    items: String,
}

fn latest_sec_earnings_filing_row(submissions: &Value) -> Option<SecEarningsFilingRow> {
    let recent = submissions.pointer("/filings/recent")?;
    let forms = recent.get("form")?.as_array()?;
    let filing_dates = recent.get("filingDate")?.as_array()?;
    let report_dates = recent.get("reportDate")?.as_array()?;
    let accessions = recent.get("accessionNumber")?.as_array()?;
    let documents = recent.get("primaryDocument")?.as_array()?;
    let items = recent.get("items").and_then(Value::as_array);
    for index in 0..forms.len() {
        let form = forms.get(index)?.as_str()?;
        let filing_items = items
            .and_then(|values| values.get(index))
            .and_then(Value::as_str)
            .unwrap_or("");
        let is_earnings_filing = matches!(form, "10-Q" | "10-Q/A")
            || (matches!(form, "8-K" | "8-K/A")
                && (filing_items.contains("2.02") || filing_items.contains("7.01")))
            || matches!(form, "6-K" | "6-K/A");
        if !is_earnings_filing {
            continue;
        }
        return Some(SecEarningsFilingRow {
            form: form.to_string(),
            filing_date: filing_dates.get(index)?.as_str()?.to_string(),
            report_date: report_dates
                .get(index)
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            accession: accessions.get(index)?.as_str()?.to_string(),
            primary_document: documents.get(index)?.as_str()?.to_string(),
            items: filing_items.to_string(),
        });
    }
    None
}

fn sec_earnings_exhibit_name(index: &Value, primary_document: &str) -> Option<String> {
    let items = index.pointer("/directory/item")?.as_array()?;
    items
        .iter()
        .filter_map(|item| item.get("name").and_then(Value::as_str))
        .filter(|name| !name.eq_ignore_ascii_case(primary_document))
        .filter(|name| {
            let lower = name.to_ascii_lowercase();
            (lower.ends_with(".htm") || lower.ends_with(".html"))
                && !lower.contains("-index")
                && (lower.contains("earnings")
                    || lower.contains("results")
                    || lower.contains("press")
                    || lower.starts_with("ex99")
                    || lower.starts_with("ex-99"))
        })
        .min_by_key(|name| {
            let lower = name.to_ascii_lowercase();
            usize::from(!lower.contains("earnings")) + usize::from(!lower.contains("press"))
        })
        .map(str::to_string)
}

fn sec_html_to_text_excerpt(html: &str, max_chars: usize) -> String {
    let mut text = String::with_capacity(html.len().min(max_chars.saturating_mul(8)));
    let mut in_tag = false;
    for character in html.chars() {
        match character {
            '<' => {
                in_tag = true;
                text.push(' ');
            }
            '>' => in_tag = false,
            _ if !in_tag => text.push(character),
            _ => {}
        }
    }
    let decoded = text
        .replace("&nbsp;", " ")
        .replace("&#160;", " ")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">");
    let compact = decoded.split_whitespace().collect::<Vec<_>>().join(" ");
    let chars = compact.chars().collect::<Vec<_>>();
    if chars.len() <= max_chars {
        return compact;
    }

    // SEC earnings exhibits put the headline near the front but frequently put
    // the balance sheet and cash-flow tables many pages later. A simple prefix
    // silently removed exactly the OCF/capex/debt evidence needed for a deep
    // company answer. Keep a concise headline plus deterministic windows around
    // the standard table labels; this remains verbatim filing text, not an LLM
    // summary or a company-specific parser.
    let head_len = (max_chars / 4).max(1).min(chars.len());
    let mut ranges = vec![(0usize, head_len)];
    let lower = compact.to_ascii_lowercase();
    for marker in [
        "cash flow from operating activities",
        "cash flows from operating activities",
        "net cash provided by operating activities",
        "free cash flow",
        "purchases of property, plant and equipment",
        "capital expenditures",
        "condensed consolidated balance sheets",
        "cash and cash equivalents",
        "long-term debt",
        "end market summary",
    ] {
        let Some(byte_index) = lower.find(marker) else {
            continue;
        };
        let center = lower[..byte_index].chars().count();
        let start = center.saturating_sub(220);
        let end = (center + 2_200).min(chars.len());
        if ranges
            .iter()
            .any(|(existing_start, existing_end)| start < *existing_end && end > *existing_start)
        {
            continue;
        }
        ranges.push((start, end));
    }
    ranges.sort_unstable_by_key(|(start, _)| *start);
    let mut output = String::new();
    for (start, end) in ranges {
        if output.chars().count() >= max_chars {
            break;
        }
        if !output.is_empty() {
            output.push_str(" … ");
        }
        let remaining = max_chars.saturating_sub(output.chars().count());
        output.extend(chars[start..end].iter().take(remaining));
    }
    output
}

fn sec_fact_observations(fact: &Value, limit: usize) -> Vec<Value> {
    let Some(units) = fact.get("units").and_then(Value::as_object) else {
        return Vec::new();
    };
    let preferred = ["USD", "shares", "USD/shares", "pure"];
    let Some((unit, rows)) = preferred
        .iter()
        .find_map(|unit| {
            units
                .get(*unit)
                .and_then(Value::as_array)
                .map(|rows| (*unit, rows))
        })
        .or_else(|| {
            units
                .iter()
                .find_map(|(unit, value)| value.as_array().map(|rows| (unit.as_str(), rows)))
        })
    else {
        return Vec::new();
    };
    let allowed_forms = ["10-Q", "10-Q/A", "10-K", "10-K/A", "20-F", "20-F/A", "6-K"];
    let mut rows = rows
        .iter()
        .filter(|row| {
            row.get("form")
                .and_then(Value::as_str)
                .is_some_and(|form| allowed_forms.contains(&form))
                && row.get("val").is_some_and(Value::is_number)
                && row.get("end").and_then(Value::as_str).is_some()
        })
        .cloned()
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| {
        let left_key = format!(
            "{}:{}:{}",
            left.get("filed").and_then(Value::as_str).unwrap_or(""),
            left.get("end").and_then(Value::as_str).unwrap_or(""),
            left.get("frame").and_then(Value::as_str).unwrap_or("")
        );
        let right_key = format!(
            "{}:{}:{}",
            right.get("filed").and_then(Value::as_str).unwrap_or(""),
            right.get("end").and_then(Value::as_str).unwrap_or(""),
            right.get("frame").and_then(Value::as_str).unwrap_or("")
        );
        right_key.cmp(&left_key)
    });
    let mut seen = std::collections::HashSet::new();
    rows.into_iter()
        .filter_map(|row| {
            let key = format!(
                "{}:{}:{}:{}",
                row.get("start").and_then(Value::as_str).unwrap_or("point"),
                row.get("end").and_then(Value::as_str).unwrap_or(""),
                row.get("form").and_then(Value::as_str).unwrap_or(""),
                row.get("val").map(Value::to_string).unwrap_or_default()
            );
            if !seen.insert(key) {
                return None;
            }
            let start = row.get("start").and_then(Value::as_str);
            let end = row.get("end").and_then(Value::as_str);
            let duration_days = start.zip(end).and_then(|(start, end)| {
                NaiveDate::parse_from_str(end, "%Y-%m-%d")
                    .ok()?
                    .signed_duration_since(NaiveDate::parse_from_str(start, "%Y-%m-%d").ok()?)
                    .num_days()
                    .checked_add(1)
            });
            let period_kind = match duration_days {
                None => "point_in_time",
                Some(days) if (70..=110).contains(&days) => "standalone_quarter",
                Some(days) if (160..=310).contains(&days) => "year_to_date",
                Some(days) if (330..=390).contains(&days) => "annual",
                Some(_) => "other_duration",
            };
            Some(serde_json::json!({
                "value": row.get("val").cloned().unwrap_or(Value::Null),
                "unit": unit,
                "start": row.get("start").cloned().unwrap_or(Value::Null),
                "end": row.get("end").cloned().unwrap_or(Value::Null),
                "duration_days": duration_days,
                "period_kind": period_kind,
                "filed": row.get("filed").cloned().unwrap_or(Value::Null),
                "form": row.get("form").cloned().unwrap_or(Value::Null),
                "fiscal_year": row.get("fy").cloned().unwrap_or(Value::Null),
                "fiscal_period": row.get("fp").cloned().unwrap_or(Value::Null),
                "frame": row.get("frame").cloned().unwrap_or(Value::Null),
                "accession_number": row.get("accn").cloned().unwrap_or(Value::Null),
            }))
        })
        .take(limit)
        .collect()
}

fn sec_latest_metric_value(financials: &Value, metric: &str) -> Option<f64> {
    financials
        .pointer(&format!("/data/metrics/{metric}/observations/0/value"))
        .and_then(Value::as_f64)
        .filter(|value| value.is_finite())
}

fn build_sec_valuation_inputs(symbol: &str, quote: Value, financials: Value) -> Value {
    let price = quote.get("price").and_then(Value::as_f64);
    let shares = sec_latest_metric_value(&financials, "shares_outstanding");
    let cash = sec_latest_metric_value(&financials, "cash_and_equivalents");
    let debt_current = sec_latest_metric_value(&financials, "long_term_debt_current");
    let debt_noncurrent = sec_latest_metric_value(&financials, "long_term_debt_noncurrent");
    let market_cap = price.zip(shares).map(|(price, shares)| price * shares);
    let total_debt = match (debt_current, debt_noncurrent) {
        (Some(current), Some(noncurrent)) => Some(current + noncurrent),
        (Some(current), None) => Some(current),
        (None, Some(noncurrent)) => Some(noncurrent),
        (None, None) => None,
    };
    let enterprise_value = market_cap
        .zip(total_debt)
        .map(|(market_cap, debt)| market_cap + debt - cash.unwrap_or(0.0));
    let sec_source_url = financials.get("source_url").cloned().unwrap_or(Value::Null);
    let latest_filed_date = financials
        .pointer("/data/latest_filed_date")
        .cloned()
        .unwrap_or(Value::Null);
    serde_json::json!({
        "data_type": "valuation",
        "ticker": symbol,
        "data": {
            "quote": quote,
            "sec_source_url": sec_source_url,
            "latest_sec_filed_date": latest_filed_date,
            "derived_inputs": {
                "current_price": price,
                "shares_outstanding": shares,
                "market_cap": market_cap,
                "cash_and_equivalents": cash,
                "long_term_debt_current": debt_current,
                "long_term_debt_noncurrent": debt_noncurrent,
                "total_debt": total_debt,
                "enterprise_value": enterprise_value,
            },
        },
        "fallback_source": "Nasdaq quote + SEC Company Facts APIs",
        "coverage": "valuation_inputs_only_without_fmp",
        "hone_valuation_policy": "Market capitalization is current Nasdaq price multiplied by the latest SEC shares-outstanding fact. Enterprise value adds the available current and noncurrent long-term-debt facts and subtracts available cash. Verify that debt facts share the same period end before using EV. This fallback does not provide consensus forecasts, peer multiples, historical multiple bands or a DCF fair value; disclose those gaps instead of manufacturing them.",
    })
}

fn attach_quote_timestamp_metadata(value: &mut Value) {
    let Value::Object(fields) = value else {
        return;
    };
    let Some(timestamp) = fields.get("timestamp").and_then(Value::as_i64) else {
        return;
    };
    let Some(utc) = DateTime::from_timestamp(timestamp, 0) else {
        return;
    };
    let new_york = utc.with_timezone(&chrono_tz::America::New_York);
    let beijing = utc.with_timezone(&chrono_tz::Asia::Shanghai);
    fields.insert(
        "hone_quote_time".to_string(),
        serde_json::json!({
            "unix_seconds": timestamp,
            "new_york": new_york.format("%Y-%m-%d %H:%M:%S %:z").to_string(),
            "beijing": beijing.format("%Y-%m-%d %H:%M:%S %:z").to_string(),
            "market_date_new_york": new_york.format("%Y-%m-%d").to_string(),
            "source": "provider Unix timestamp converted by Hone; use `beijing` for the user-visible quote time; this metadata does not establish a market session"
        }),
    );
}

fn attach_quote_evidence_quality(value: &mut Value) {
    let Value::Object(fields) = value else {
        return;
    };
    let mut warnings = Vec::new();
    let symbol_ok = fields
        .get("symbol")
        .and_then(Value::as_str)
        .is_some_and(|value| !value.trim().is_empty());
    let price = finite_number(fields.get("price"));
    let price_ok = price.is_some_and(|value| value > 0.0);

    let previous_close = finite_number(fields.get("previousClose"));
    let change = finite_number(fields.get("change"));
    let change_percent = finite_number(fields.get("changesPercentage"));
    let change_inputs_complete = price.is_some()
        && previous_close.is_some_and(|value| value > 0.0)
        && change.is_some()
        && change_percent.is_some();
    if !change_inputs_complete {
        warnings.push("quote_change_inputs_incomplete");
    }
    let change_consistent = match (price, previous_close, change) {
        (Some(price), Some(previous_close), Some(change)) if previous_close > 0.0 => {
            approximately_equal(price - previous_close, change, 0.02, 0.002)
        }
        _ => false,
    };
    if !change_consistent {
        warnings.push("quote_change_mismatch");
    }
    let change_percent_consistent = match (previous_close, change, change_percent) {
        (Some(previous_close), Some(change), Some(change_percent)) if previous_close > 0.0 => {
            approximately_equal(change / previous_close * 100.0, change_percent, 0.2, 0.01)
        }
        _ => false,
    };
    if !change_percent_consistent {
        warnings.push("quote_change_percentage_mismatch");
    }

    let day_low = finite_number(fields.get("dayLow"));
    let day_high = finite_number(fields.get("dayHigh"));
    let day_range_consistent = ordered_positive_range(day_low, day_high)
        && price.map_or(true, |price| {
            value_within_optional_range(price, day_low, day_high)
        });
    if !day_range_consistent {
        warnings.push("quote_day_range_mismatch");
    }
    let year_low = finite_number(fields.get("yearLow"));
    let year_high = finite_number(fields.get("yearHigh"));
    let year_range_consistent = ordered_positive_range(year_low, year_high)
        && price.map_or(true, |price| {
            value_within_optional_range(price, year_low, year_high)
        });
    if !year_range_consistent {
        warnings.push("quote_year_range_mismatch");
    }

    let market_cap_consistent = match (
        price,
        finite_number(fields.get("sharesOutstanding")),
        finite_number(fields.get("marketCap")),
    ) {
        (Some(price), Some(shares), Some(market_cap))
            if price > 0.0 && shares > 0.0 && market_cap > 0.0 =>
        {
            let ratio = market_cap / (price * shares);
            (0.5..=2.0).contains(&ratio)
        }
        _ => true,
    };
    if !market_cap_consistent {
        warnings.push("quote_market_cap_dimensional_mismatch");
    }
    if !symbol_ok {
        warnings.push("quote_symbol_missing");
    }
    if !price_ok {
        warnings.push("quote_price_invalid");
    }

    let usable_for_change_claims = symbol_ok
        && price_ok
        && change_inputs_complete
        && change_consistent
        && change_percent_consistent;
    let canonical_change = match (price, previous_close, change) {
        (Some(price), Some(previous_close), Some(change))
            if usable_for_change_claims && previous_close > 0.0 =>
        {
            serde_json::json!({
                "basis": "regular_close_vs_previous_regular_close",
                "price": price,
                "previous_regular_close": previous_close,
                "absolute_change": change,
                "change_percentage": (change / previous_close * 10_000.0).round() / 100.0,
                "provider_change_percentage": change_percent,
                "usable": true,
            })
        }
        _ => serde_json::json!({
            "basis": "regular_close_vs_previous_regular_close",
            "usable": false,
            "reason": "missing_or_internally_inconsistent_quote_change_fields",
        }),
    };
    fields.insert(
        "hone_canonical_regular_change".to_string(),
        canonical_change,
    );
    fields.insert(
        "hone_evidence_quality".to_string(),
        serde_json::json!({
            "usable_for_price_claims": symbol_ok && price_ok,
            "usable_for_change_claims": usable_for_change_claims,
            "usable_for_range_claims": symbol_ok
                && price_ok
                && day_range_consistent
                && year_range_consistent,
            "usable_for_market_cap_claims": symbol_ok && price_ok && market_cap_consistent,
            "warnings": warnings,
            "policy": "A false flag quarantines only that claim type. Exact regular-session change claims require complete and arithmetically consistent price, previousClose, change and changesPercentage fields. Use hone_canonical_regular_change.change_percentage for the close-to-close daily move; preserve raw provider fields only for audit and never substitute a premarket/session-to-session denominator."
        }),
    );

    // Provider money fields are raw units. Converting 45_570_000_000 into 亿 is
    // arithmetic, and arithmetic done in prose is where a market cap becomes
    // ten times too large. Render it once, here, and have the answer copy it.
    let currency = fields
        .get("currency")
        .and_then(Value::as_str)
        .unwrap_or("USD");
    let mut display = serde_json::Map::new();
    if let Some(market_cap) = finite_number(fields.get("marketCap")) {
        display.insert(
            "market_cap".to_string(),
            Value::String(chinese_scaled_money(market_cap, currency)),
        );
    }
    if let Some(shares) = finite_number(fields.get("sharesOutstanding")) {
        display.insert(
            "shares_outstanding".to_string(),
            Value::String(chinese_scaled_count(shares)),
        );
    }
    if !display.is_empty() {
        display.insert(
            "policy".to_string(),
            Value::String(
                "这些字符串是服务端按原始字段换算好的中文计数单位，发布金额或股数时直接引用，不要自己把 marketCap 之类的原始数字换算成亿或万亿。".to_string(),
            ),
        );
        fields.insert("hone_display".to_string(), Value::Object(display));
    }
}

/// 1e8 is 一亿 and 1e12 is 一万亿. Getting this wrong by one power of ten is
/// invisible in prose and changes the entire conclusion.
fn chinese_scaled_money(value: f64, currency: &str) -> String {
    let unit = match currency.to_ascii_uppercase().as_str() {
        "USD" => "美元",
        "CNY" | "RMB" => "元",
        "HKD" => "港元",
        "EUR" => "欧元",
        "JPY" => "日元",
        "KRW" => "韩元",
        "GBP" => "英镑",
        other => return format!("{} {other}", scaled_with_chinese_magnitude(value)),
    };
    format!("{}{unit}", scaled_with_chinese_magnitude(value))
}

fn chinese_scaled_count(value: f64) -> String {
    format!("{}股", scaled_with_chinese_magnitude(value))
}

fn scaled_with_chinese_magnitude(value: f64) -> String {
    // Each step is a factor of 10_000, so a value that rounds to 10000.00 in
    // one unit is really 1.00 of the next one; printing "10000.00 万" reads as
    // 一亿 and defeats the point of rendering it at all.
    const UNITS: [(f64, &str); 3] = [(1e4, "万"), (1e8, "亿"), (1e12, "万亿")];
    let magnitude = value.abs();
    let mut chosen: Option<(f64, &'static str)> = None;
    for (divisor, unit) in UNITS {
        if magnitude >= divisor {
            chosen = Some((divisor, unit));
        }
    }
    let Some((divisor, unit)) = chosen else {
        return format!("{value:.2} ");
    };
    let scaled = value / divisor;
    if (scaled.abs() * 100.0).round() / 100.0 >= 10_000.0
        && let Some(index) = UNITS
            .iter()
            .position(|(candidate, _)| *candidate == divisor)
        && let Some((next_divisor, next_unit)) = UNITS.get(index + 1)
    {
        return format!("{:.2} {next_unit}", value / next_divisor);
    }
    format!("{scaled:.2} {unit}")
}

fn urlencoding_encode(raw: &str) -> String {
    raw.bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (byte as char).to_string()
            }
            b' ' => "%20".to_string(),
            other => format!("%{other:02X}"),
        })
        .collect()
}

/// Altman Z and Piotroski are published as bare numbers. Without their bands a
/// reader cannot tell whether 2.4 is good, and the model is left to supply a
/// threshold from memory — so the bands travel with the score.
fn financial_score_semantics(data: &Value) -> Value {
    let scores = data
        .get("financial_scores")
        .and_then(|value| value.as_array().and_then(|rows| rows.first()))
        .or_else(|| data.get("financial_scores"));
    let altman = scores.and_then(|row| statement_number(row, &["altmanZScore"]));
    let piotroski = scores.and_then(|row| statement_number(row, &["piotroskiScore"]));
    let altman_band = altman.map(|value| {
        if value > 2.99 {
            "safe_zone"
        } else if value >= 1.81 {
            "grey_zone"
        } else {
            "distress_zone"
        }
    });
    serde_json::json!({
        "altman_z_score": optional_number(altman),
        "altman_band": altman_band.map_or(Value::Null, |band| Value::String(band.to_string())),
        "altman_bands": "safe_zone >2.99；grey_zone 1.81-2.99；distress_zone <1.81。该模型面向制造业，对金融、地产与部分轻资产科技公司解释力有限，引用时说明这一限制。",
        "piotroski_score": optional_number(piotroski),
        "piotroski_scale": "0-9 分，衡量盈利能力、杠杆与经营效率的九项二元检验；7 分及以上通常视为基本面稳健，3 分及以下偏弱。",
        "policy": "这些是 provider 发布的标准化评分，属于本轮可引用证据；引用时写明分数、所属区间与模型适用性限制，不要改写成自制评级，也不要在没有取到分数时凭记忆给出。"
    })
}

fn statement_rows(value: Option<&Value>) -> &[Value] {
    value.and_then(Value::as_array).map_or(&[], Vec::as_slice)
}

fn statement_number(row: &Value, keys: &[&str]) -> Option<f64> {
    keys.iter()
        .find_map(|key| row.get(*key))
        .and_then(Value::as_f64)
        .filter(|value| value.is_finite())
}

fn statement_period_label(row: &Value) -> Option<String> {
    let period = row.get("period").and_then(Value::as_str)?;
    let year = row
        .get("calendarYear")
        .and_then(|year| {
            year.as_str()
                .map(str::to_string)
                .or_else(|| year.as_i64().map(|value| value.to_string()))
        })
        .unwrap_or_default();
    Some(if year.is_empty() {
        period.to_string()
    } else {
        format!("{period} {year}")
    })
}

fn pct_change(current: Option<f64>, previous: Option<f64>) -> Option<f64> {
    match (current, previous) {
        (Some(current), Some(previous)) if previous.abs() > f64::EPSILON => {
            Some(((current - previous) / previous.abs() * 10_000.0).round() / 100.0)
        }
        _ => None,
    }
}

fn optional_number(value: Option<f64>) -> Value {
    value.map_or(Value::Null, |value| {
        serde_json::Number::from_f64(value).map_or(Value::Null, Value::Number)
    })
}

/// Sums the four most recent reported quarters. This is the only trailing
/// window the turn can prove, which makes it the reference for deciding whether
/// a provider's own TTM aggregate has caught up with the latest release.
///
/// It is also the only window under which companies on different fiscal
/// calendars can be compared. Publishing one company's FY2025 beside another's
/// FY2026 — a two-year gap for the same industry cycle — is what makes a
/// comparison table wrong while every individual number in it is right, so the
/// margins are computed here rather than left to be read off annual reports.
fn trailing_twelve_month_summary(quarterly: &[Value]) -> Option<Value> {
    let window = quarterly.get(..4)?;
    let mut revenue = 0.0;
    let mut net_income = 0.0;
    let mut eps = 0.0;
    let mut gross_profit = 0.0;
    let mut operating_income = 0.0;
    let mut eps_complete = true;
    let mut gross_complete = true;
    let mut operating_complete = true;
    let mut periods = Vec::with_capacity(4);
    for row in window {
        revenue += statement_number(row, &["revenue"])?;
        net_income += statement_number(row, &["netIncome"])?;
        match statement_number(row, &["epsdiluted", "epsDiluted", "eps"]) {
            Some(value) => eps += value,
            None => eps_complete = false,
        }
        match statement_number(row, &["grossProfit"]) {
            Some(value) => gross_profit += value,
            None => gross_complete = false,
        }
        match statement_number(row, &["operatingIncome"]) {
            Some(value) => operating_income += value,
            None => operating_complete = false,
        }
        periods.push(Value::String(
            row.get("date")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
        ));
    }
    let reported_currency = window
        .first()
        .and_then(|row| row.get("reportedCurrency"))
        .and_then(Value::as_str)
        .unwrap_or("USD");
    let round4 = |value: f64| (value * 10_000.0).round() / 10_000.0;
    let margin = |total: f64, complete: bool| {
        (complete && revenue.abs() > f64::EPSILON)
            .then(|| (total / revenue * 10_000.0).round() / 100.0)
    };
    Some(serde_json::json!({
        "basis": "last_4_reported_quarters",
        "period_ends": periods,
        "latest_period_end": window.first().and_then(|row| row.get("date")).cloned().unwrap_or(Value::Null),
        "latest_period_label": window.first().and_then(statement_period_label).map_or(Value::Null, Value::String),
        "revenue": optional_number(Some(round4(revenue))),
        "net_income": optional_number(Some(round4(net_income))),
        "eps_diluted": optional_number(eps_complete.then(|| round4(eps))),
        "gross_margin_pct": optional_number(margin(gross_profit, gross_complete)),
        "operating_margin_pct": optional_number(margin(operating_income, operating_complete)),
        "hone_display": {
            "revenue": chinese_scaled_money(revenue, reported_currency),
            "net_income": chinese_scaled_money(net_income, reported_currency),
            "policy": "直接引用这些换算好的字符串，不要自己把原始金额换算成亿或万亿。"
        },
        "note": "由本轮已发布的四个季度直接相加得到。它既用于校验 provider 的 TTM 口径是否已含最新季度，也是跨公司对比唯一同口径的窗口：各公司财年结束月份不同，直接并列各自的 FY 标签会把不同年份的周期位置混在一张表里。对比多家公司时用本窗口并标注 period_ends。"
    }))
}

/// The latest quarter with its own sequential and year-over-year comparisons.
/// Without this the turn can state a revenue number but cannot say whether the
/// business accelerated or decelerated, which is most of what the question is.
fn latest_quarter_summary(quarterly: &[Value], cash_flow: &[Value]) -> Option<Value> {
    let latest = quarterly.first()?;
    let previous = quarterly.get(1);
    let year_ago = quarterly.get(4);
    let revenue = statement_number(latest, &["revenue"]);
    let gross_profit = statement_number(latest, &["grossProfit"]);
    let gross_margin_pct = match (gross_profit, revenue) {
        (Some(profit), Some(revenue)) if revenue.abs() > f64::EPSILON => {
            Some((profit / revenue * 10_000.0).round() / 100.0)
        }
        _ => None,
    };
    let operating_cash_flow = cash_flow
        .first()
        .and_then(|row| statement_number(row, &["operatingCashFlow"]));
    Some(serde_json::json!({
        "period_end": latest.get("date").cloned().unwrap_or(Value::Null),
        "period_label": statement_period_label(latest).map_or(Value::Null, Value::String),
        "revenue": optional_number(revenue),
        "revenue_qoq_pct": optional_number(pct_change(revenue, previous.and_then(|row| statement_number(row, &["revenue"])))),
        "revenue_yoy_pct": optional_number(pct_change(revenue, year_ago.and_then(|row| statement_number(row, &["revenue"])))),
        "gross_margin_pct": optional_number(gross_margin_pct),
        "operating_income": optional_number(statement_number(latest, &["operatingIncome"])),
        "net_income": optional_number(statement_number(latest, &["netIncome"])),
        "eps_diluted": optional_number(statement_number(latest, &["epsdiluted", "epsDiluted", "eps"])),
        "operating_cash_flow": optional_number(operating_cash_flow),
        "note": "环比对比上一披露季度，同比对比四个季度之前；两者都来自本轮同一份季度序列"
    }))
}

/// Sums the four estimate periods that fall strictly after the latest reported
/// quarter. Nothing in the pipeline previously turned analyst estimates into a
/// number, so a forward multiple had no source at all and every valuation
/// comparison had to fall back to a trailing one — which is meaningless across
/// companies sitting at different points of the same cycle.
fn forward_twelve_month_summary(
    estimates: &[Value],
    latest_reported_end: Option<&str>,
) -> Option<Value> {
    let mut forward = estimates
        .iter()
        .filter_map(|row| {
            let date = row.get("date").and_then(Value::as_str)?;
            // "Forward" is defined by the reported window, not by a clock the
            // provider and Hone might disagree about.
            latest_reported_end
                .is_none_or(|latest| date > latest)
                .then_some((date, row))
        })
        .collect::<Vec<_>>();
    forward.sort_by_key(|(date, _)| *date);
    let window = forward.get(..4)?;

    let mut eps = 0.0;
    let mut revenue = 0.0;
    let mut eps_complete = true;
    let mut revenue_complete = true;
    let mut analyst_counts = Vec::new();
    let mut period_ends = Vec::new();
    for (date, row) in window {
        match statement_number(row, &["epsAvg", "estimatedEpsAvg"]) {
            Some(value) => eps += value,
            None => eps_complete = false,
        }
        match statement_number(row, &["revenueAvg", "estimatedRevenueAvg"]) {
            Some(value) => revenue += value,
            None => revenue_complete = false,
        }
        if let Some(count) = statement_number(row, &["numAnalystsEps", "numberAnalystEstimatedEps"])
        {
            analyst_counts.push(count);
        }
        period_ends.push(Value::String((*date).to_string()));
    }
    let round4 = |value: f64| (value * 10_000.0).round() / 10_000.0;
    Some(serde_json::json!({
        "basis": "next_4_estimated_quarters",
        "period_ends": period_ends,
        "eps": optional_number(eps_complete.then(|| round4(eps))),
        "revenue": optional_number(revenue_complete.then(|| round4(revenue))),
        "min_analyst_count": optional_number(analyst_counts.iter().copied().fold(None, |acc: Option<f64>, value| {
            Some(acc.map_or(value, |current: f64| current.min(value)))
        })),
        "note": "分析师一致预期，不是已实现业绩。窗口取严格晚于最新已披露季度的四个预期季度；analyst_count 过低时该预期的代表性有限，应在结论中说明。"
    }))
}

fn build_financials_bundle(
    annual: Value,
    quarterly: Option<Value>,
    balance_sheet: Option<Value>,
    cash_flow: Option<Value>,
    estimates: Option<Value>,
    growth: Option<Value>,
) -> Value {
    // Derive everything that reads the rows before the arrays are moved into
    // the payload.
    let ttm_summary = trailing_twelve_month_summary(statement_rows(quarterly.as_ref()));
    let latest_summary = latest_quarter_summary(
        statement_rows(quarterly.as_ref()),
        statement_rows(cash_flow.as_ref()),
    );
    let latest_reported_end = ttm_summary
        .as_ref()
        .and_then(|ttm| ttm.get("latest_period_end"))
        .and_then(Value::as_str)
        .map(str::to_string);
    let forward_summary = forward_twelve_month_summary(
        statement_rows(estimates.as_ref()),
        latest_reported_end.as_deref(),
    );
    let coverage = [
        ("annual_income_statement", Some(&annual)),
        ("quarterly_income_statement", quarterly.as_ref()),
        ("quarterly_balance_sheet", balance_sheet.as_ref()),
        ("quarterly_cash_flow", cash_flow.as_ref()),
        ("analyst_estimates", estimates.as_ref()),
        ("financial_growth", growth.as_ref()),
    ]
    .into_iter()
    .map(|(name, value)| {
        (
            name.to_string(),
            Value::String(
                if value.is_some_and(has_meaningful_fmp_value) {
                    "available"
                } else {
                    "unavailable"
                }
                .to_string(),
            ),
        )
    })
    .collect::<serde_json::Map<_, _>>();

    let mut payload = serde_json::json!({
        // `data` stays the annual income statement array so every existing
        // consumer of this payload keeps reading the same shape.
        "data": annual,
        "hone_statement_coverage": Value::Object(coverage),
        "hone_financials_policy": "覆盖状态为 unavailable 的报表本轮确实没有取到，必须按缺口披露，不得由其它报表或记忆推算。金额单位以 provider 原始字段为准；毛利率等比率字段已换算为百分数。"
    });
    if let Some(quarterly) = quarterly.filter(has_meaningful_fmp_value) {
        payload["hone_quarterly_income_statement"] = quarterly;
    }
    if let Some(balance_sheet) = balance_sheet.filter(has_meaningful_fmp_value) {
        payload["hone_quarterly_balance_sheet"] = balance_sheet;
    }
    if let Some(cash_flow) = cash_flow.filter(has_meaningful_fmp_value) {
        payload["hone_quarterly_cash_flow"] = cash_flow;
    }
    if let Some(ttm) = ttm_summary {
        payload["hone_ttm"] = ttm;
    }
    if let Some(latest) = latest_summary {
        payload["hone_latest_quarter"] = latest;
    }
    if let Some(estimates) = estimates.filter(has_meaningful_fmp_value) {
        payload["hone_analyst_estimates"] = estimates;
    }
    if let Some(forward) = forward_summary {
        payload["hone_forward"] = forward;
    }
    if let Some(growth) = growth.filter(has_meaningful_fmp_value) {
        payload["hone_financial_growth"] = growth;
    }
    payload
}

/// A provider's trailing `eps`/`pe` fields keep the pre-release window for days
/// after a company reports, which is exactly when people ask about the print.
/// The turn already knows the four reported quarters, so it can say whether the
/// provider aggregate has caught up instead of publishing a multiple computed
/// against a quarter that no longer exists.
///
/// It also derives the forward multiples, because a trailing multiple compares
/// two companies at different points of the same cycle and says nothing useful:
/// a trough-year EPS produces a flattering P/E and a peak-year EPS a punishing
/// one, on identical businesses.
pub fn valuation_basis_quality(quote: &Value, financials: &Value) -> Value {
    let quote_row = quote
        .as_array()
        .and_then(|rows| rows.first())
        .unwrap_or(quote);
    let provider_eps = statement_number(quote_row, &["eps"]);
    let provider_pe = statement_number(quote_row, &["pe"]);
    let price = statement_number(quote_row, &["price"]);
    let ttm = financials.get("hone_ttm");
    let recomputed_eps = ttm.and_then(|ttm| statement_number(ttm, &["eps_diluted"]));
    let latest_period = ttm
        .and_then(|ttm| ttm.get("latest_period_end"))
        .cloned()
        .unwrap_or(Value::Null);

    // Without both numbers there is nothing to compare; say so rather than
    // implying the provider figure was checked.
    let (includes_latest, recomputed_pe) = match (provider_eps, recomputed_eps) {
        (Some(provider), Some(recomputed)) if recomputed.abs() > f64::EPSILON => {
            let drift = (provider - recomputed).abs() / recomputed.abs();
            let recomputed_pe = price
                .filter(|_| recomputed.abs() > f64::EPSILON)
                .map(|price| (price / recomputed * 100.0).round() / 100.0);
            (Some(drift <= 0.10), recomputed_pe)
        }
        _ => (None, None),
    };

    let usable = includes_latest.unwrap_or(false);
    let mut warnings = Vec::new();
    if includes_latest == Some(false) {
        warnings.push("provider_ttm_excludes_latest_reported_quarter");
    }
    if includes_latest.is_none() {
        warnings.push("provider_ttm_basis_unverified");
    }

    let forward = financials.get("hone_forward");
    let forward_eps = forward.and_then(|forward| statement_number(forward, &["eps"]));
    let forward_revenue = forward.and_then(|forward| statement_number(forward, &["revenue"]));
    let market_cap = statement_number(quote_row, &["marketCap"]);
    let forward_pe = match (price, forward_eps) {
        (Some(price), Some(eps)) if eps.abs() > f64::EPSILON => {
            Some((price / eps * 100.0).round() / 100.0)
        }
        _ => None,
    };
    let forward_ps = match (market_cap, forward_revenue) {
        (Some(cap), Some(revenue)) if revenue.abs() > f64::EPSILON => {
            Some((cap / revenue * 100.0).round() / 100.0)
        }
        _ => None,
    };
    if forward_eps.is_none() && forward_revenue.is_none() {
        warnings.push("forward_estimates_unavailable");
    }

    serde_json::json!({
        "provider_ttm_eps": optional_number(provider_eps),
        "provider_pe": optional_number(provider_pe),
        "recomputed_ttm_eps": optional_number(recomputed_eps),
        "recomputed_pe": optional_number(recomputed_pe),
        "latest_reported_period_end": latest_period,
        "provider_ttm_includes_latest_reported_quarter": includes_latest.map_or(Value::Null, Value::Bool),
        "usable_for_multiple_claims": usable,
        "forward_eps": optional_number(forward_eps),
        "forward_revenue": optional_number(forward_revenue),
        "forward_pe": optional_number(forward_pe),
        "forward_ps": optional_number(forward_ps),
        "forward_period_ends": forward.and_then(|forward| forward.get("period_ends")).cloned().unwrap_or(Value::Null),
        "warnings": warnings,
        "policy": "usable_for_multiple_claims=false 时，不得直接发布 provider 的 pe/eps 倍数。要么改用 recomputed_pe / recomputed_ttm_eps 并写明是按本轮已披露四个季度重算，要么说明该倍数尚未包含最新季度。财报发布后数日内 provider 的 TTM 常常仍是上一口径。forward_pe / forward_ps 由现价（或市值）除以 hone_forward 的未来四个季度一致预期得到，是预期不是已实现业绩，须标注 forward_period_ends；跨公司比较倍数时优先用 forward 或同一 TTM 窗口，不要并列各自财年的 trailing 倍数。"
    })
}

fn price_target_consensus_quality(value: &Value, current_price: Option<f64>) -> Value {
    let row = value
        .as_array()
        .and_then(|items| items.first())
        .or_else(|| value.as_object().map(|_| value));
    let low = row.and_then(|row| first_positive_number(row, &["targetLow", "low"]));
    let high = row.and_then(|row| first_positive_number(row, &["targetHigh", "high"]));
    let consensus = row.and_then(|row| {
        first_positive_number(
            row,
            &["targetConsensus", "consensus", "priceTargetConsensus"],
        )
    });
    let median = row.and_then(|row| {
        first_positive_number(row, &["targetMedian", "median", "priceTargetMedian"])
    });

    let range_consistent = ordered_positive_range(low, high);
    let consensus_in_range = consensus
        .map(|value| value_within_optional_range(value, low, high))
        .unwrap_or(true);
    let median_in_range = median
        .map(|value| value_within_optional_range(value, low, high))
        .unwrap_or(true);
    let has_target = consensus.or(median).is_some();
    let usable = has_target && range_consistent && consensus_in_range && median_in_range;
    let anchor = consensus.or(median);
    let ratio = anchor
        .zip(current_price)
        .and_then(|(target, current)| (current > 0.0).then_some(target / current));
    let magnitude_warning = ratio.is_some_and(|ratio| !(0.25..=4.0).contains(&ratio));
    let mut warnings = Vec::new();
    if !has_target {
        warnings.push("target_consensus_missing");
    }
    if !range_consistent || !consensus_in_range || !median_in_range {
        warnings.push("target_consensus_internal_range_mismatch");
    }
    if magnitude_warning {
        warnings.push("target_consensus_extreme_vs_current_quote");
    }

    serde_json::json!({
        "usable_for_target_claims": usable,
        "requires_independent_corroboration": magnitude_warning,
        "target_to_current_price_ratio": ratio,
        "warnings": warnings,
        "policy": "Internal range failures quarantine target claims. An extreme target/current ratio remains visible but requires an independent source before publication."
    })
}

fn first_positive_number(value: &Value, keys: &[&str]) -> Option<f64> {
    let row = value
        .as_array()
        .and_then(|items| items.first())
        .unwrap_or(value);
    keys.iter()
        .find_map(|key| finite_number(row.get(*key)))
        .filter(|value| *value > 0.0)
}

fn security_listing_evidence(ticker: &str, quote: &Value, profile: &Value) -> Value {
    let quote_row = quote
        .as_array()
        .and_then(|items| items.first())
        .unwrap_or(quote);
    let profile_row = profile
        .as_array()
        .and_then(|items| items.first())
        .unwrap_or(profile);
    let quote_symbol = quote_row.get("symbol").and_then(Value::as_str);
    let profile_symbol = profile_row.get("symbol").and_then(Value::as_str);
    let quote_matches =
        quote_symbol.is_some_and(|symbol| hone_core::provider_symbols_equivalent(ticker, symbol));
    let profile_matches =
        profile_symbol.is_some_and(|symbol| hone_core::provider_symbols_equivalent(ticker, symbol));
    let current_quote_available =
        quote_matches && first_positive_number(quote_row, &["price"]).is_some();
    let is_actively_trading = profile_row
        .get("isActivelyTrading")
        .and_then(Value::as_bool);
    let exchange = profile_row
        .get("exchangeShortName")
        .or_else(|| profile_row.get("exchange"))
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty());
    let status = if quote_matches
        && profile_matches
        && current_quote_available
        && is_actively_trading == Some(true)
        && exchange.is_some()
    {
        "active_listing"
    } else if profile_matches && is_actively_trading == Some(false) && !current_quote_available {
        "inactive_listing"
    } else {
        "unverified"
    };

    serde_json::json!({
        "status": status,
        "requested_symbol": ticker,
        "quote_symbol": quote_symbol,
        "profile_symbol": profile_symbol,
        "company_name": profile_row.get("companyName").and_then(Value::as_str),
        "exchange": exchange,
        "profile_is_actively_trading": is_actively_trading,
        "current_quote_available": current_quote_available,
        "policy": "active_listing is current-turn same-symbol provider evidence that the security is currently listed and trading. It overrides stale model memory about an earlier acquisition or delisting; old corporate history alone cannot negate a later relisting or spin-off listing. If a current authoritative regulatory filing conflicts, fetch and disclose that evidence instead of deciding from memory."
    })
}

fn finite_number(value: Option<&Value>) -> Option<f64> {
    value
        .and_then(Value::as_f64)
        .filter(|value| value.is_finite())
}

fn approximately_equal(left: f64, right: f64, absolute: f64, relative: f64) -> bool {
    (left - right).abs() <= absolute.max(left.abs().max(right.abs()) * relative)
}

fn ordered_positive_range(low: Option<f64>, high: Option<f64>) -> bool {
    match (low, high) {
        (Some(low), Some(high)) => low > 0.0 && high > 0.0 && low <= high,
        _ => true,
    }
}

fn value_within_optional_range(value: f64, low: Option<f64>, high: Option<f64>) -> bool {
    low.is_none_or(|low| value >= low) && high.is_none_or(|high| value <= high)
}

fn has_meaningful_fmp_value(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::String(text) => !text.trim().is_empty(),
        Value::Array(items) => items.iter().any(has_meaningful_fmp_value),
        Value::Object(fields) => fields.values().any(has_meaningful_fmp_value),
        Value::Bool(_) | Value::Number(_) => true,
    }
}

/// The financials bundle always carries its own policy text, so the generic
/// "is anything in here" check would call it available even when every
/// statement failed. Its own coverage map is the authority when present.
fn financials_component_available(value: &Value) -> bool {
    match value
        .get("hone_statement_coverage")
        .and_then(Value::as_object)
    {
        Some(coverage) => coverage
            .values()
            .any(|state| state.as_str() == Some("available")),
        None => has_meaningful_fmp_value(value),
    }
}

fn nonempty_fmp_error_message(value: &Value) -> Option<String> {
    match value {
        Value::Null => None,
        Value::String(message) if message.trim().is_empty() => None,
        Value::Array(items) if items.is_empty() => None,
        Value::Object(fields) if fields.is_empty() => None,
        Value::String(message) => Some(message.clone()),
        other => Some(other.to_string()),
    }
}

fn fmp_error_message_triggers_key_fallback(message: &str) -> bool {
    let lower = message.to_ascii_lowercase();
    lower.contains("invalid api key")
        || lower.contains("api key")
        || lower.contains("apikey")
        || lower.contains("limit reach")
        || lower.contains("rate limit")
        || lower.contains("quota")
        || lower.contains("upgrade")
}

fn fmp_cache_key_for_url(url: &str) -> String {
    let Ok(parsed) = reqwest::Url::parse(url) else {
        return strip_apikey_like_params(url);
    };

    let pairs = parsed
        .query_pairs()
        .filter(|(key, _)| !is_fmp_api_key_param(key))
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect::<Vec<_>>();
    let mut sanitized = parsed;
    sanitized.set_query(None);
    if !pairs.is_empty() {
        {
            let mut query = sanitized.query_pairs_mut();
            for (key, value) in pairs {
                query.append_pair(&key, &value);
            }
        }
    }
    sanitized.to_string()
}

fn is_fmp_api_key_param(key: &str) -> bool {
    matches!(key.to_ascii_lowercase().as_str(), "apikey" | "api_key") || key == "apiKey"
}

fn strip_apikey_like_params(url: &str) -> String {
    let Some((prefix, query)) = url.split_once('?') else {
        return url.to_string();
    };
    let kept = query
        .split('&')
        .filter(|part| {
            let key = part.split_once('=').map(|(key, _)| key).unwrap_or(part);
            !is_fmp_api_key_param(key)
        })
        .collect::<Vec<_>>();
    if kept.is_empty() {
        prefix.to_string()
    } else {
        format!("{prefix}?{}", kept.join("&"))
    }
}

fn redact_url_userinfo(text: &str) -> String {
    let mut remaining = text;
    let mut output = String::with_capacity(text.len());
    while let Some(index) = remaining.find("://") {
        let authority_start = index + 3;
        let authority = &remaining[authority_start..];
        let authority_end = authority
            .char_indices()
            .find_map(|(idx, ch)| {
                (ch.is_whitespace() || matches!(ch, '/' | '?' | '#' | ')')).then_some(idx)
            })
            .unwrap_or(authority.len());
        let authority_slice = &authority[..authority_end];
        if let Some(at_index) = authority_slice.rfind('@') {
            output.push_str(&remaining[..authority_start]);
            output.push_str("<redacted>@");
            remaining = &remaining[authority_start + at_index + 1..];
        } else {
            output.push_str(&remaining[..authority_start]);
            remaining = &remaining[authority_start..];
        }
    }
    output.push_str(remaining);
    output
}

fn redact_fmp_query_secrets(text: &str) -> String {
    let mut output = text.to_string();
    for key in ["apikey", "api_key", "apiKey"] {
        output = redact_delimited_fmp_secret_value(&output, &format!("{key}="));
        output = redact_delimited_fmp_secret_value(&output, &format!("{key}:"));
        output = redact_fmp_json_string_field(&output, key);
    }
    output
}

fn redact_delimited_fmp_secret_value(text: &str, needle: &str) -> String {
    let mut remaining = text;
    let mut output = String::with_capacity(text.len());
    while let Some(index) = remaining.find(needle) {
        let value_start = index + needle.len();
        output.push_str(&remaining[..value_start]);
        let leading_whitespace = remaining[value_start..]
            .chars()
            .take_while(|ch| ch.is_whitespace())
            .map(char::len_utf8)
            .sum::<usize>();
        output.push_str(&remaining[value_start..value_start + leading_whitespace]);
        output.push_str("<redacted>");
        let value_tail = remaining[value_start + leading_whitespace..]
            .char_indices()
            .find_map(|(idx, ch)| {
                (ch == '&'
                    || ch == ')'
                    || ch == ','
                    || ch == ';'
                    || ch == '"'
                    || ch == '\''
                    || ch == '}'
                    || ch == ']'
                    || ch.is_whitespace())
                .then_some(idx)
            })
            .unwrap_or(remaining[value_start + leading_whitespace..].len());
        remaining = &remaining[value_start + leading_whitespace + value_tail..];
    }
    output.push_str(remaining);
    output
}

fn redact_fmp_json_string_field(text: &str, key: &str) -> String {
    let key_marker = format!("\"{key}\"");
    let mut remaining = text;
    let mut output = String::with_capacity(text.len());
    while let Some(index) = remaining.find(&key_marker) {
        let after_key = index + key_marker.len();
        let tail = &remaining[after_key..];
        let Some((colon_offset, _)) = tail.char_indices().find(|(_, ch)| !ch.is_whitespace())
        else {
            break;
        };
        if !tail[colon_offset..].starts_with(':') {
            output.push_str(&remaining[..after_key]);
            remaining = &remaining[after_key..];
            continue;
        }
        let after_colon = &tail[colon_offset + 1..];
        let Some((quote_offset, _)) = after_colon
            .char_indices()
            .find(|(_, ch)| !ch.is_whitespace())
        else {
            break;
        };
        if !after_colon[quote_offset..].starts_with('"') {
            output.push_str(&remaining[..after_key]);
            remaining = &remaining[after_key..];
            continue;
        }
        let value_start = after_key + colon_offset + 1 + quote_offset + 1;
        output.push_str(&remaining[..value_start]);
        output.push_str("<redacted>");
        let value_tail = remaining[value_start..]
            .char_indices()
            .find_map(|(idx, ch)| (ch == '"').then_some(idx))
            .unwrap_or(remaining[value_start..].len());
        remaining = &remaining[value_start + value_tail..];
    }
    output.push_str(remaining);
    output
}

#[async_trait]
impl Tool for DataFetchTool {
    fn name(&self) -> &str {
        "data_fetch"
    }

    fn description(&self) -> &str {
        "获取金融数据（股票/ETF/加密货币的实体、行情、基本面和新闻等）。公司或证券分析必须先用 search，并由主 Agent 完整分析用户点名的标的，为每个标的分配一个本轮稳定且互不复用的 `entity_route`；每个标的分别发起 search（可并行，禁止拼成一个 query），后续 refinement、quote、profile/snapshot 与其它该标的调用继续携带同一个 `entity_route`。每一次 search 都必须由 Agent 在该次调用中明示 call-scoped `identity_match=exact_symbol`（query 是 ticker）或 `name_or_alias`（query 是公司名/别名）；旧调用的声明不会继承，服务端也不按大小写、长度或分隔符猜测。显式 ticker 路线会持续受同代码约束，即使后来用公司名补查，也不能被名称中提及该 ticker 的其它产品替代；`BRK/B`、`BRK-B`、`BRK.B` 等有限 provider 分隔写法视为同代码。路线只是内部关联键，不是实体结论。中文名或别名搜索为空时，应在同路线换用正式英文名或标准 ticker；可把原始空 query 逐字放进 `refines_query`。若早先 search 漏了路线键，后续显式路线 search 用 `supersedes_query` 逐字指向那个旧 query，服务端只迁移这一条，不猜别名关系。`refines_query` 与 `supersedes_query` 严格互斥、每次最多填写一个；二者同时出现会使本次实体 search 无效。search 结果只证明实体候选，不能单独证明客户、供应商、合同或新闻因果。quote/crypto_quote 中的 `hone_quote_time.beijing` 是 Hone 从 provider Unix timestamp 规范化得到的用户可见北京时间，应优先原样使用；普通 quote 的该字段不证明盘前/盘后时段。quote 中只有 `hone_evidence_quality.usable_for_change_claims=true` 时，才可使用 `hone_canonical_regular_change.change_percentage` 作为常规盘 close-to-close 日涨跌；原始 changesPercentage 不得绕过质量标记。只有 `extended_hours` 的规范化 bar 与 hone_session_summaries 可以核验美股扩展时段：regular 日涨跌只用 pct_change_close_to_close，pre 只用 pct_change_vs_previous_regular_close，post 只用 pct_change_vs_regular_close；pct_change_vs_prev_session_close 绝不能冒充日涨跌。snapshot 与 earnings_outlook 返回的 `hone_security_listing_evidence.status=active_listing` 是本轮同代码当前上市证据，不得被模型关于旧收购或退市的记忆覆盖。涨跌归因或目标交易日问题必须使用 quote；quote_short 只是低带宽简版批量行情，可能缺少 previousClose、change、changesPercentage、exchange 或 timestamp，不能用来证明涨跌幅、目标日期、交易所或交易时段。支持的数据类型：search（实体搜索）、quote（带完整性标记的常规盘行情）、quote_short（低带宽简版行情）、extended_hours（按纽约日期和时段规范化的盘前/常规盘/盘后行情）、profile、snapshot、earnings_outlook、financials、valuation、segments、peers、ownership、corporate_actions、press_releases、transcript、macro、market_hours、news、gainers_losers、sector_performance、crypto_quote、etf_holdings、earnings_calendar。"
    }

    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter {
                name: "data_type".to_string(),
                param_type: "string".to_string(),
                description: "数据类型。单一证券的财报时间、预期、评级和目标价研究优先使用 earnings_outlook；它返回 profile、各组件覆盖状态、数值质量标记和当前上市证据。hone_security_listing_evidence.status=active_listing 时不得用旧收购/退市记忆否认当前上市。全市场某个日期窗口才使用 earnings_calendar。quote/snapshot 内的 hone_evidence_quality 对价格、涨跌、区间和市值声明分别授权；只有 usable_for_change_claims=true 才可使用 hone_canonical_regular_change 的常规盘 close-to-close 涨跌，false 的字段组不得用于精确结论。".to_string(),
                required: true,
                r#enum: Some(vec![
                    "quote".into(),
                    "quote_short".into(),
                    "extended_hours".into(),
                    "profile".into(),
                    "snapshot".into(),
                    "earnings_outlook".into(),
                    "financials".into(),
                    "valuation".into(),
                    "segments".into(),
                    "peers".into(),
                    "ownership".into(),
                    "corporate_actions".into(),
                    "press_releases".into(),
                    "transcript".into(),
                    "macro".into(),
                    "market_hours".into(),
                    "news".into(),
                    "gainers_losers".into(),
                    "sector_performance".into(),
                    "crypto_quote".into(),
                    "etf_holdings".into(),
                    "earnings_calendar".into(),
                    "search".into(),
                ]),
                items: None,
            },
            ToolParameter {
                name: "query".to_string(),
                param_type: "string".to_string(),
                description:
                    "仅 search 使用的公司名、别名或证券代码查询词（如 NVIDIA、英伟达、NVDA）"
                        .to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "entity_route".to_string(),
                param_type: "string".to_string(),
                description: "公司/证券研究的内部路线键。先完整分析用户点名的标的，为每个标的选一个稳定且不同的短键（如 coreweave、nvidia），并在该标的的 search/refinement/quote/profile/snapshot/earnings_outlook 等调用中原样复用；不得把两个标的共用一条路线。宽泛市场数据可省略。".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "identity_match".to_string(),
                param_type: "string".to_string(),
                description: "仅 search 使用且证券研究的每一次 search 都必须在该次调用中填写；它是 call-scoped，旧 search 的值不会继承。query 是明确 ticker 时填 exact_symbol；query 是公司名、中文名或别名时填 name_or_alias。由读完整问题的 Agent 决定，不得按字符串大小写或长度猜测。".to_string(),
                required: false,
                r#enum: Some(vec!["exact_symbol".into(), "name_or_alias".into()]),
                items: None,
            },
            ToolParameter {
                name: "refines_query".to_string(),
                param_type: "string".to_string(),
                description: "仅 refinement search 使用；逐字且区分大小写地填写本轮先前返回空结果、且当前调用正在补查的原始 query。实际 query 必须非空、与这里不同，并直接对应返回的 symbol/name；不得填写或查询其它实体。与 supersedes_query 严格互斥，每次最多填写一个。".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "supersedes_query".to_string(),
                param_type: "string".to_string(),
                description: "仅用于给早先漏写 entity_route 的 search 补路线键；逐字且区分大小写地填写那次旧 query。可用于非空或空结果，服务端最多只迁移该精确 query，不推断别名。与 refines_query 严格互斥，每次最多填写一个。".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "ticker".to_string(),
                param_type: "string".to_string(),
                description: "已确认的股票/ETF/加密货币代码；search 优先使用 query".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "symbol".to_string(),
                param_type: "string".to_string(),
                description: "股票代码（别名，如 AAPL）".to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "from".to_string(),
                param_type: "string".to_string(),
                description:
                    "仅 earnings_calendar 使用的开始日期，格式 YYYY-MM-DD；默认当前北京时间日期"
                        .to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
            ToolParameter {
                name: "to".to_string(),
                param_type: "string".to_string(),
                description:
                    "仅 earnings_calendar 使用的结束日期，格式 YYYY-MM-DD；默认开始日期后 14 天"
                        .to_string(),
                required: false,
                r#enum: None,
                items: None,
            },
        ]
    }

    async fn execute(&self, args: Value) -> hone_core::HoneResult<Value> {
        let data_type = effective_data_fetch_data_type(&args);
        let ticker = effective_data_fetch_target(&args);

        if self.keys.is_empty() {
            if !self.official_fallback_enabled {
                return Ok(serde_json::json!({
                    "error": "未配置 FMP API Key（请在 config.yaml 中设置 fmp.api_keys）"
                }));
            }
            return Ok(self.fetch_without_fmp(data_type, ticker).await);
        }

        if data_type == "snapshot" {
            let stable = self.stable_base_url();
            let price_change_url = format!("{stable}/stable/stock-price-change?symbol={ticker}");
            let aftermarket_url = format!("{stable}/stable/aftermarket-quote?symbol={ticker}");
            // These were three sequential round-trips for no reason; nothing
            // downstream depends on an earlier one.
            let (quote, profile, news, price_change, aftermarket) = tokio::join!(
                self.fetch_data_type("quote", ticker),
                self.fetch_data_type("profile", ticker),
                self.fetch_data_type("news", ticker),
                self.fetch_from_url_cached(&price_change_url, ttl_for_data_type("quote"), "quote"),
                self.fetch_from_url_cached(
                    &aftermarket_url,
                    ttl_for_data_type("extended_hours"),
                    "extended_hours"
                ),
            );
            let mut payload = self.build_snapshot_response(
                ticker,
                quote.map(normalize_quote_timestamp_metadata),
                profile,
                news,
            );
            // Multi-period performance answers "how has this done lately"
            // without spending another research round on a chart.
            if let Some(change) = price_change.ok().filter(has_meaningful_fmp_value) {
                payload["data"]["price_change"] = change;
                payload["hone_price_change_semantics"] = Value::String(
                    "price_change 各字段是相对当前价的区间涨跌幅（1D/5D/1M/3M/6M/YTD/1Y/3Y/5Y/10Y/max），单位为百分数；它是区间表现，不是某一天的涨跌。".to_string(),
                );
            }
            // The provider's own post-market quote, independent of the minute
            // bars Hone aggregates itself.
            if let Some(after) = aftermarket.ok().filter(has_meaningful_fmp_value) {
                payload["data"]["aftermarket_quote"] = after;
                payload["hone_aftermarket_semantics"] = Value::String(
                    "aftermarket_quote 是 provider 直接给出的盘后买卖盘与时间戳，与 extended_hours 的分钟汇总互为印证；两者不一致时以时间戳更新者为准并说明来源。".to_string(),
                );
            }
            return Ok(payload);
        }

        if data_type == "extended_hours" {
            let stable = self.stable_base_url();
            let after_quote_url = format!("{stable}/stable/aftermarket-quote?symbol={ticker}");
            let after_trade_url = format!("{stable}/stable/aftermarket-trade?symbol={ticker}");
            let extended_ttl = ttl_for_data_type("extended_hours");
            let (bars, after_quote, after_trade) = tokio::join!(
                self.fetch_data_type("extended_hours", ticker),
                self.fetch_from_url_cached(&after_quote_url, extended_ttl, "extended_hours"),
                self.fetch_from_url_cached(&after_trade_url, extended_ttl, "extended_hours"),
            );
            let bars = match bars {
                Ok(bars) => bars,
                Err(err) => return Ok(serde_json::json!({ "error": err })),
            };
            let mut normalized = match normalize_extended_hours_bar(ticker, &bars) {
                Ok(bar) => bar,
                Err(err) => return Ok(serde_json::json!({ "error": err })),
            };
            if let Some(quote) = after_quote.ok().filter(has_meaningful_fmp_value) {
                normalized["hone_aftermarket_quote"] = quote;
            }
            if let Some(trade) = after_trade.ok().filter(has_meaningful_fmp_value) {
                normalized["hone_aftermarket_trade"] = trade;
            }
            return Ok(serde_json::json!({
                "data_type": data_type,
                "ticker": ticker,
                "data": normalized
            }));
        }

        if data_type == "earnings_outlook" {
            let earnings_url = match self.build_earnings_outlook_url("earnings", ticker) {
                Ok(url) => url,
                Err(err) => return Ok(serde_json::json!({ "error": err })),
            };
            let estimates_url = self
                .build_earnings_outlook_url("analyst_estimates", ticker)
                .expect("validated earnings symbol");
            let targets_url = self
                .build_earnings_outlook_url("price_target_consensus", ticker)
                .expect("validated earnings symbol");
            let ratings_url = self
                .build_earnings_outlook_url("ratings_snapshot", ticker)
                .expect("validated earnings symbol");
            let target_summary_url = self
                .build_earnings_outlook_url("price_target_summary", ticker)
                .expect("validated earnings symbol");
            let grades_url = self
                .build_earnings_outlook_url("grades_consensus", ticker)
                .expect("validated earnings symbol");
            let (
                quote,
                profile,
                earnings,
                analyst_estimates,
                price_target_consensus,
                ratings_snapshot,
                financials,
                price_target_summary,
                grades_consensus,
            ) = tokio::join!(
                self.fetch_data_type("quote", ticker),
                self.fetch_data_type("profile", ticker),
                self.fetch_from_url_cached(
                    &earnings_url,
                    ttl_for_data_type(data_type),
                    "earnings_outlook_earnings"
                ),
                self.fetch_from_url_cached(
                    &estimates_url,
                    ttl_for_data_type(data_type),
                    "earnings_outlook_analyst_estimates"
                ),
                self.fetch_from_url_cached(
                    &targets_url,
                    ttl_for_data_type(data_type),
                    "earnings_outlook_price_target_consensus"
                ),
                self.fetch_from_url_cached(
                    &ratings_url,
                    ttl_for_data_type(data_type),
                    "earnings_outlook_ratings_snapshot"
                ),
                self.fetch_financials_bundle(ticker),
                self.fetch_from_url_cached(
                    &target_summary_url,
                    ttl_for_data_type(data_type),
                    "earnings_outlook"
                ),
                self.fetch_from_url_cached(
                    &grades_url,
                    ttl_for_data_type(data_type),
                    "earnings_outlook"
                ),
            );
            let mut payload = self.build_earnings_outlook_response(
                ticker,
                quote.map(normalize_quote_timestamp_metadata),
                profile,
                earnings,
                analyst_estimates,
                price_target_consensus,
                ratings_snapshot,
                financials,
            );
            // A single consensus target hides the spread and the revision
            // direction; the rating distribution hides how lopsided it is.
            if let Some(summary) = price_target_summary.ok().filter(has_meaningful_fmp_value) {
                payload["data"]["price_target_summary"] = summary;
            }
            if let Some(grades) = grades_consensus.ok().filter(has_meaningful_fmp_value) {
                payload["data"]["grades_consensus"] = grades;
            }
            return Ok(payload);
        }

        // Peer comparison is two-stage: the registry names the peers, then one
        // batch quote prices them. Doing it here means "is it expensive?" has
        // an anchor instead of being answered from the model's memory of who
        // the competitors are.
        if data_type == "peers" {
            let stable = self.stable_base_url();
            let peers = self
                .fetch_from_url_cached(
                    &format!("{stable}/stable/stock-peers?symbol={ticker}"),
                    ttl_for_data_type(data_type),
                    data_type,
                )
                .await;
            let peer_symbols = peers
                .as_ref()
                .ok()
                .and_then(Value::as_array)
                .map(|rows| {
                    rows.iter()
                        .filter_map(|row| row.get("symbol").and_then(Value::as_str))
                        .take(8)
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            let quotes = if peer_symbols.is_empty() {
                None
            } else {
                self.fetch_from_url_cached(
                    &format!(
                        "{stable}/stable/batch-quote?symbols={}",
                        peer_symbols.join(",")
                    ),
                    ttl_for_data_type("quote"),
                    "quote",
                )
                .await
                .ok()
            };
            let mut payload = serde_json::json!({
                "data_type": data_type,
                "ticker": ticker,
                "data": {
                    "peers": peers.unwrap_or(Value::Null),
                    "peer_quotes": quotes.unwrap_or(Value::Null),
                },
                "hone_peer_policy": "peers 来自 provider 的同业列表，不是模型记忆里的竞争对手；引用时写明是 provider 同业分类。对比倍数时同业与本标的必须用同一口径，并注意各家财年结束月份不同。"
            });
            if let Some(pe) = self.fetch_sector_industry_pe(&payload).await {
                payload["data"]["industry_pe_snapshot"] = pe;
            }
            return Ok(payload);
        }

        if let Some(components) = self.stable_bundle_components(data_type, ticker, &args) {
            let (data, coverage, errors) = self.fetch_stable_bundle(data_type, components).await;
            let mut payload = serde_json::json!({
                "data_type": data_type,
                "ticker": ticker,
                "data": data,
                "coverage": coverage,
                "evidence_policy": "coverage 为 unavailable/empty 的组件本轮确实没有数据，必须按缺口披露，不得由其它组件或记忆推算。"
            });
            if let Some(errors) = errors {
                payload["errors"] = errors;
            }
            if data_type == "valuation" {
                payload["hone_score_semantics"] = financial_score_semantics(&payload["data"]);
            }
            return Ok(payload);
        }

        if data_type == "financials" {
            return match self.fetch_financials_bundle(ticker).await {
                Ok(mut payload) => {
                    payload["data_type"] = Value::String(data_type.to_string());
                    payload["ticker"] = Value::String(ticker.to_string());
                    Ok(payload)
                }
                Err(err) => Ok(serde_json::json!({ "error": err })),
            };
        }

        if data_type == "earnings_calendar" {
            let (from, to) = match self.resolve_earnings_window(&args) {
                Ok(window) => window,
                Err(err) => return Ok(serde_json::json!({ "error": err })),
            };
            let url = self.build_earnings_calendar_url(from, to);
            return match self
                .fetch_from_url_cached(&url, ttl_for_data_type(data_type), data_type)
                .await
            {
                Ok(data) => Ok(serde_json::json!({
                    "data_type": data_type,
                    "ticker": ticker,
                    "request_window": {
                        "from": from.format("%Y-%m-%d").to_string(),
                        "to": to.format("%Y-%m-%d").to_string(),
                    },
                    "data": data
                })),
                Err(err) => Ok(serde_json::json!({ "error": err })),
            };
        }

        let _url = match self.build_url(data_type, ticker) {
            Ok(url) => url,
            Err(err) => return Ok(serde_json::json!({"error": err})),
        };

        match self.fetch_data_type(data_type, ticker).await {
            Ok(data) => {
                let data = if data_type == "extended_hours" {
                    match normalize_extended_hours_bar(ticker, &data) {
                        Ok(bar) => bar,
                        Err(err) => return Ok(serde_json::json!({ "error": err })),
                    }
                } else if matches!(data_type, "quote" | "quote_short" | "crypto_quote") {
                    normalize_quote_timestamp_metadata(data)
                } else {
                    data
                };
                Ok(serde_json::json!({
                    "data_type": data_type,
                    "ticker": ticker,
                    "data": data
                }))
            }
            Err(err) => Ok(serde_json::json!({ "error": err })),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DataFetchTool, build_sec_valuation_inputs, chinese_scaled_money,
        data_fetch_data_type_uses_security_target, effective_data_fetch_data_type,
        effective_data_fetch_security_target, effective_data_fetch_target, extended_hours_session,
        financial_score_semantics, fmp_base_url_is_loopback, forward_twelve_month_summary,
        latest_sec_earnings_filing_row, nasdaq_quote_row, nonempty_fmp_error_message,
        normalize_extended_hours_bar, normalize_quote_timestamp_metadata,
        normalize_sec_company_facts, price_target_consensus_quality, sanitize_fmp_error_detail,
        sec_earnings_exhibit_name, sec_html_to_text_excerpt, security_listing_evidence,
        should_cache_fmp_value, ttl_for_data_type, validated_data_fetch_search_query,
        validated_data_fetch_symbols, valuation_basis_quality,
    };
    use crate::base::Tool;
    use crate::test_support::{assert_text_contains_all, assert_text_contains_none};
    use chrono::{DateTime, Duration, NaiveDate};
    use serde_json::json;
    use std::net::SocketAddr;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    #[test]
    fn loopback_fmp_adapters_bypass_workstation_proxies() {
        assert!(fmp_base_url_is_loopback("http://127.0.0.1:8080/api"));
        assert!(fmp_base_url_is_loopback("http://[::1]:8080/api"));
        assert!(fmp_base_url_is_loopback("http://localhost:8080/api"));
        assert!(!fmp_base_url_is_loopback(
            "https://financialmodelingprep.com/api"
        ));
    }

    #[test]
    fn quote_timestamp_metadata_exposes_unambiguous_new_york_and_beijing_times() {
        let timestamp = DateTime::parse_from_rfc3339("2026-07-17T20:00:00Z")
            .expect("valid quote timestamp")
            .timestamp();
        let normalized = normalize_quote_timestamp_metadata(json!([{
            "symbol": "CRWV",
            "price": 73.21,
            "timestamp": timestamp
        }]));
        let quote_time = &normalized[0]["hone_quote_time"];

        assert_eq!(quote_time["unix_seconds"], timestamp);
        assert_eq!(quote_time["new_york"], "2026-07-17 16:00:00 -04:00");
        assert_eq!(quote_time["beijing"], "2026-07-18 04:00:00 +08:00");
        assert_eq!(quote_time["market_date_new_york"], "2026-07-17");
        assert!(
            quote_time.get("session").is_none(),
            "a regular quote timestamp must not be promoted into extended-hours session evidence"
        );
    }

    #[test]
    fn nasdaq_quote_fallback_requires_exact_symbol_and_marks_delayed_close() {
        let payload = json!({
            "data": {
                "symbol": "CRWV",
                "companyName": "CoreWeave, Inc.",
                "exchange": "NASDAQ-GS",
                "primaryData": {
                    "lastSalePrice": "$90.32",
                    "netChange": "+2.13",
                    "percentageChange": "+2.42%",
                    "lastTradeTimestamp": "Aug 11, 2026",
                    "isRealTime": false,
                    "volume": "46,381,731"
                }
            }
        });

        let row = nasdaq_quote_row("CRWV", &payload).expect("exact Nasdaq quote");
        assert_eq!(row["symbol"], "CRWV");
        assert_eq!(row["price"], 90.32);
        assert_eq!(row["changesPercentage"], 2.42);
        assert_eq!(row["previousClose"], 88.19);
        assert_eq!(row["hone_source"]["is_realtime"], false);
        assert!(
            row["hone_quote_time"]["source"]
                .as_str()
                .expect("time source")
                .contains("delayed close")
        );
        assert_eq!(
            row["hone_evidence_quality"]["usable_for_price_claims"],
            true
        );
        assert!(nasdaq_quote_row("MU", &payload).is_none());
    }

    #[test]
    fn sec_fallback_preserves_periods_and_builds_only_available_valuation_inputs() {
        let facts = json!({
            "entityName": "Example Corp",
            "fiscalYearEnd": "1231",
            "facts": {
                "us-gaap": {
                    "RevenueFromContractWithCustomerExcludingAssessedTax": {
                        "label": "Revenue",
                        "description": "Revenue",
                        "units": {"USD": [{
                            "start": "2026-04-01", "end": "2026-06-30",
                            "val": 2_078_000_000_f64, "accn": "0001-26-000001",
                            "fy": 2026, "fp": "Q2", "form": "10-Q",
                            "filed": "2026-08-11", "frame": "CY2026Q2"
                        }]}
                    },
                    "LongTermDebtCurrent": {
                        "label": "Current debt", "units": {"USD": [{
                            "end": "2026-06-30", "val": 1_000_000_000_f64,
                            "accn": "0001-26-000001", "fy": 2026, "fp": "Q2",
                            "form": "10-Q", "filed": "2026-08-11"
                        }]}
                    },
                    "LongTermDebtNoncurrent": {
                        "label": "Noncurrent debt", "units": {"USD": [{
                            "end": "2026-06-30", "val": 3_000_000_000_f64,
                            "accn": "0001-26-000001", "fy": 2026, "fp": "Q2",
                            "form": "10-Q", "filed": "2026-08-11"
                        }]}
                    },
                    "CashAndCashEquivalentsAtCarryingValue": {
                        "label": "Cash", "units": {"USD": [{
                            "end": "2026-06-30", "val": 500_000_000_f64,
                            "accn": "0001-26-000001", "fy": 2026, "fp": "Q2",
                            "form": "10-Q", "filed": "2026-08-11"
                        }]}
                    }
                },
                "dei": {
                    "EntityCommonStockSharesOutstanding": {
                        "label": "Shares outstanding", "units": {"shares": [{
                            "end": "2026-08-01", "val": 100_000_000_f64,
                            "accn": "0001-26-000001", "fy": 2026, "fp": "Q2",
                            "form": "10-Q", "filed": "2026-08-11"
                        }]}
                    }
                }
            }
        });
        let financials =
            normalize_sec_company_facts("TEST", 123, "https://data.sec.gov/example.json", &facts);
        assert_eq!(
            financials["data"]["metrics"]["revenue"]["observations"][0]["duration_days"],
            91
        );
        assert_eq!(
            financials["data"]["latest_metric_summary"]["revenue"]["period_kind"],
            "standalone_quarter"
        );
        assert_eq!(financials["data"]["latest_filed_date"], "2026-08-11");
        assert_eq!(
            financials["data"]["metrics"]["gross_profit"]["status"],
            "not_reported"
        );

        let valuation =
            build_sec_valuation_inputs("TEST", json!({"symbol":"TEST", "price": 20.0}), financials);
        assert_eq!(
            valuation["data"]["derived_inputs"]["market_cap"],
            2_000_000_000_f64
        );
        assert_eq!(
            valuation["data"]["derived_inputs"]["total_debt"],
            4_000_000_000_f64
        );
        assert_eq!(
            valuation["data"]["derived_inputs"]["enterprise_value"],
            5_500_000_000_f64
        );
        assert_eq!(valuation["coverage"], "valuation_inputs_only_without_fmp");
    }

    #[test]
    fn quote_quality_quarantines_only_inconsistent_claim_groups() {
        let normalized = normalize_quote_timestamp_metadata(json!([{
            "symbol": "MU",
            "price": 920.95,
            "previousClose": 900.0,
            "change": 20.95,
            "changesPercentage": 0.23,
            "dayLow": 910.0,
            "dayHigh": 930.0,
            "yearLow": 60.0,
            "yearHigh": 180.0,
            "marketCap": 1_000_000_000_000_f64,
            "sharesOutstanding": 1_100_000_000_f64
        }]));
        let quality = &normalized[0]["hone_evidence_quality"];

        assert_eq!(quality["usable_for_price_claims"], true);
        assert_eq!(quality["usable_for_change_claims"], false);
        assert_eq!(
            normalized[0]["hone_canonical_regular_change"]["usable"],
            false
        );
        assert_eq!(quality["usable_for_range_claims"], false);
        assert_eq!(quality["usable_for_market_cap_claims"], true);
        assert!(
            quality["warnings"]
                .as_array()
                .expect("quality warnings")
                .iter()
                .any(|warning| warning == "quote_year_range_mismatch")
        );
    }

    #[test]
    fn quote_quality_requires_complete_consistent_change_inputs() {
        let incomplete = normalize_quote_timestamp_metadata(json!([{
            "symbol": "MRVL",
            "price": 237.04,
            "changesPercentage": -5.57
        }]));
        assert_eq!(
            incomplete[0]["hone_evidence_quality"]["usable_for_change_claims"],
            false
        );
        assert_eq!(
            incomplete[0]["hone_canonical_regular_change"]["usable"],
            false
        );

        let consistent = normalize_quote_timestamp_metadata(json!([{
            "symbol": "MRVL",
            "price": 237.04,
            "previousClose": 251.01,
            "change": -13.97,
            "changesPercentage": -5.57
        }]));
        assert_eq!(
            consistent[0]["hone_evidence_quality"]["usable_for_change_claims"],
            true
        );
        assert_eq!(
            consistent[0]["hone_canonical_regular_change"]["basis"],
            "regular_close_vs_previous_regular_close"
        );
        assert_eq!(
            consistent[0]["hone_canonical_regular_change"]["change_percentage"],
            -5.57
        );
    }

    #[test]
    fn target_consensus_extreme_ratio_requires_independent_corroboration() {
        let quality = price_target_consensus_quality(
            &json!([{
                "targetLow": 120.0,
                "targetHigh": 180.0,
                "targetConsensus": 158.0,
                "targetMedian": 160.0
            }]),
            Some(920.95),
        );

        assert_eq!(quality["usable_for_target_claims"], true);
        assert_eq!(quality["requires_independent_corroboration"], true);
        assert!(
            quality["warnings"]
                .as_array()
                .expect("target warnings")
                .iter()
                .any(|warning| warning == "target_consensus_extreme_vs_current_quote")
        );
    }

    #[test]
    fn target_consensus_internal_range_mismatch_is_quarantined() {
        let quality = price_target_consensus_quality(
            &json!([{
                "targetLow": 180.0,
                "targetHigh": 120.0,
                "targetConsensus": 158.0
            }]),
            Some(150.0),
        );

        assert_eq!(quality["usable_for_target_claims"], false);
        assert_eq!(quality["requires_independent_corroboration"], false);
    }

    fn tool_with_test_key() -> DataFetchTool {
        DataFetchTool::new(vec!["test_key".to_string()], "https://example.com/api", 30)
    }

    async fn spawn_scripted_http_server(
        responses: Vec<(&'static str, &'static str)>,
    ) -> (SocketAddr, Arc<AtomicUsize>) {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind scripted test server");
        let addr = listener.local_addr().expect("scripted server local addr");
        let request_count = Arc::new(AtomicUsize::new(0));
        let request_count_for_server = request_count.clone();

        tokio::spawn(async move {
            for (status, body) in responses {
                let Ok((mut socket, _)) = listener.accept().await else {
                    break;
                };
                request_count_for_server.fetch_add(1, Ordering::SeqCst);
                let mut buf = [0_u8; 4096];
                let _ = socket.read(&mut buf).await;
                let response = format!(
                    "HTTP/1.1 {status}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = socket.write_all(response.as_bytes()).await;
                let _ = socket.shutdown().await;
            }
        });

        (addr, request_count)
    }

    /// `financials` fans out to four statement endpoints concurrently, so a
    /// strictly sequential script can no longer express "this endpoint returns
    /// empty first". Routes are matched on a request-target substring and each
    /// keeps its own reply sequence; the last reply repeats once exhausted.
    async fn spawn_path_scripted_http_server(
        routes: Vec<(&'static str, Vec<&'static str>)>,
    ) -> (SocketAddr, Arc<std::sync::Mutex<Vec<String>>>) {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind path-scripted test server");
        let addr = listener
            .local_addr()
            .expect("path-scripted server local addr");
        let requests = Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
        let requests_for_server = requests.clone();

        tokio::spawn(async move {
            let mut cursors = std::collections::HashMap::<&'static str, usize>::new();
            loop {
                let Ok((mut socket, _)) = listener.accept().await else {
                    break;
                };
                let mut buf = [0_u8; 4096];
                let read = socket.read(&mut buf).await.unwrap_or(0);
                let request = String::from_utf8_lossy(&buf[..read]).to_string();
                let target = request
                    .lines()
                    .next()
                    .and_then(|line| line.split_whitespace().nth(1))
                    .unwrap_or("")
                    .to_string();
                requests_for_server
                    .lock()
                    .expect("record request")
                    .push(target.clone());
                let body = routes
                    .iter()
                    .find(|(needle, _)| target.contains(needle))
                    .map(|(needle, bodies)| {
                        let cursor = cursors.entry(needle).or_insert(0);
                        let body = bodies[(*cursor).min(bodies.len().saturating_sub(1))];
                        *cursor += 1;
                        body
                    })
                    .unwrap_or("[]");
                let response = format!(
                    "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = socket.write_all(response.as_bytes()).await;
                let _ = socket.shutdown().await;
            }
        });

        (addr, requests)
    }

    async fn spawn_truncated_body_server() -> (SocketAddr, Arc<AtomicUsize>) {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind truncated-body test server");
        let addr = listener.local_addr().expect("truncated-body local addr");
        let request_count = Arc::new(AtomicUsize::new(0));
        let request_count_for_server = request_count.clone();

        tokio::spawn(async move {
            loop {
                let Ok((mut socket, _)) = listener.accept().await else {
                    break;
                };
                request_count_for_server.fetch_add(1, Ordering::SeqCst);
                let mut buf = [0_u8; 4096];
                let _ = socket.read(&mut buf).await;
                let response = "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: 100\r\nconnection: close\r\n\r\n{";
                let _ = socket.write_all(response.as_bytes()).await;
                let _ = socket.shutdown().await;
            }
        });

        (addr, request_count)
    }

    #[test]
    fn build_url_supports_plain_and_existing_query_paths() {
        let tool = tool_with_test_key();

        let url1 = tool.build_url("quote", "AAPL").expect("quote url");
        let full_url1 = format!("{}?apikey=test_key", url1);
        assert_eq!(
            full_url1,
            "https://example.com/api/v3/quote/AAPL?apikey=test_key"
        );

        let url2 = tool
            .build_url("financials", "AAPL")
            .expect("financials url");
        let full_url2 = format!("{}&apikey=test_key", url2);
        assert_eq!(
            full_url2,
            "https://example.com/api/v3/income-statement/AAPL?limit=4&apikey=test_key"
        );

        let search_url = tool
            .build_url("search", "Nebius Group / 英伟达")
            .expect("search url");
        assert_eq!(
            search_url,
            "https://example.com/api/v3/search?query=Nebius+Group+%2F+%E8%8B%B1%E4%BC%9F%E8%BE%BE&limit=10"
        );

        let url3 = tool
            .build_url("quote_short", "AAPL,MSFT")
            .expect("quote_short url");
        let full_url3 = format!("{}&apikey=test_key", url3);
        assert_eq!(
            full_url3,
            "https://example.com/stable/batch-quote-short?symbols=AAPL,MSFT&apikey=test_key"
        );

        let extended_url = tool
            .build_url("extended_hours", "ISRG")
            .expect("extended-hours url");
        assert_eq!(
            extended_url,
            "https://example.com/api/v3/historical-chart/1min/ISRG?extended=true"
        );
        assert_eq!(
            tool.build_url("quote", "^GSPC").expect("index quote url"),
            "https://example.com/api/v3/quote/%5EGSPC"
        );
        assert_eq!(
            tool.build_url("quote", "AAPL,BRK-B")
                .expect("batch quote url"),
            "https://example.com/api/v3/quote/AAPL,BRK-B"
        );
        assert_eq!(
            ttl_for_data_type("extended_hours")
                .expect("extended-hours ttl")
                .as_secs(),
            30
        );
        assert_eq!(
            tool.build_earnings_outlook_url("earnings", "COHR")
                .expect("earnings URL"),
            "https://example.com/stable/earnings?symbol=COHR"
        );
        assert_eq!(
            tool.build_earnings_outlook_url("analyst_estimates", "COHR")
                .expect("estimates URL"),
            "https://example.com/stable/analyst-estimates?symbol=COHR&period=quarter&page=0&limit=8"
        );
    }

    #[test]
    fn effective_request_parser_matches_executor_precedence_and_types() {
        for (args, expected_type, expected_target) in [
            (json!({}), "quote", ""),
            (
                json!({"data_type":"quote","ticker":"CWY","symbol":"CRWV"}),
                "quote",
                "CWY",
            ),
            (
                json!({"data_type":"quote","ticker":["CRWV"],"symbol":"CRWV"}),
                "quote",
                "",
            ),
            (
                json!({
                    "data_type":"quote",
                    "query":"SPY",
                    "identity_match":"exact_symbol"
                }),
                "quote",
                "SPY",
            ),
            (json!({"data_type":"quote","query":"SPY"}), "quote", ""),
            (
                json!({
                    "data_type":"quote",
                    "query":"SPY",
                    "identity_match":"name_or_alias"
                }),
                "quote",
                "",
            ),
            (
                json!({
                    "data_type":"quote",
                    "ticker":"QQQ",
                    "symbol":"DIA",
                    "query":"SPY",
                    "identity_match":"exact_symbol"
                }),
                "quote",
                "QQQ",
            ),
            (
                json!({
                    "data_type":"quote",
                    "ticker":["QQQ"],
                    "query":"SPY",
                    "identity_match":"exact_symbol"
                }),
                "quote",
                "",
            ),
            (
                json!({
                    "data_type":"quote",
                    "query":["SPY"],
                    "identity_match":"exact_symbol"
                }),
                "quote",
                "",
            ),
            (
                json!({"data_type":"search","query":null,"ticker":"CRWV"}),
                "search",
                "",
            ),
            (
                json!({"data_type":"search","ticker":"CRWV"}),
                "search",
                "CRWV",
            ),
        ] {
            assert_eq!(effective_data_fetch_data_type(&args), expected_type);
            assert_eq!(effective_data_fetch_target(&args), expected_target);
        }

        assert_eq!(
            effective_data_fetch_security_target(&json!({
                "data_type":"quote",
                "ticker":" CRWV "
            })),
            Some("CRWV")
        );
        assert_eq!(
            effective_data_fetch_security_target(&json!({
                "data_type":"quote",
                "query":" SPY ",
                "identity_match":"exact_symbol"
            })),
            Some("SPY")
        );
        assert!(
            effective_data_fetch_security_target(&json!({
                "data_type":"gainers_losers",
                "ticker":"CRWV"
            }))
            .is_none()
        );
        assert!(data_fetch_data_type_uses_security_target("search"));
        assert!(!data_fetch_data_type_uses_security_target(
            "sector_performance"
        ));
        assert_eq!(
            validated_data_fetch_symbols(" CRWV,NVDA ").expect("valid symbols"),
            ["CRWV", "NVDA"]
        );
        for invalid in ["CRWV,", ",CRWV", "CRWV\nNVDA"] {
            assert!(
                validated_data_fetch_symbols(invalid).is_err(),
                "{invalid:?}"
            );
        }
        assert_eq!(
            validated_data_fetch_search_query(" CoreWeave ").expect("valid query"),
            "CoreWeave"
        );
        assert!(validated_data_fetch_search_query("\n").is_err());

        let tool = tool_with_test_key();
        assert!(tool.build_url("search", "").is_err());
        assert!(tool.build_url("search", "  ").is_err());
    }

    #[test]
    fn symbol_path_input_is_encoded_and_structural_injection_is_rejected() {
        let tool = tool_with_test_key();
        assert_eq!(
            tool.build_url("quote", "BTC/USD?x=1#fragment")
                .expect("encoded quote url"),
            "https://example.com/api/v3/quote/BTC%2FUSD%3Fx%3D1%23fragment"
        );
        assert!(tool.build_url("quote", "../,").is_err());
        assert!(tool.build_url("profile", "\nAAPL").is_ok());
        assert!(tool.build_url("profile", "AAPL\nMSFT").is_err());
    }

    #[test]
    fn fmp_transport_error_detail_redacts_apikey_query_param() {
        let detail = sanitize_fmp_error_detail(
            "error sending request for url (https://example.com/api/v3/quote/AAPL?apikey=test_key)",
        );
        assert_eq!(
            detail,
            "error sending request for url (https://example.com/api/v3/quote/AAPL?apikey=<redacted>)"
        );
    }

    #[test]
    fn fmp_error_detail_redacts_api_key_aliases() {
        let detail = sanitize_fmp_error_detail(
            "https://example.com/api/v3/quote/AAPL?api_key=one&apiKey=two&apikey=three apiKey: header-four",
        );
        assert_eq!(
            detail,
            "https://example.com/api/v3/quote/AAPL?api_key=<redacted>&apiKey=<redacted>&apikey=<redacted> apiKey: <redacted>"
        );
    }

    #[test]
    fn fmp_error_detail_redacts_api_key_aliases_before_semicolon_delimiter() {
        let detail = sanitize_fmp_error_detail(
            "https://example.com/api/v3/quote/AAPL?api_key=one;apiKey=two apikey: three;",
        );
        assert_eq!(
            detail,
            "https://example.com/api/v3/quote/AAPL?api_key=<redacted>;apiKey=<redacted> apikey: <redacted>;"
        );
    }

    #[test]
    fn fmp_error_detail_redacts_json_api_key_aliases() {
        let detail = sanitize_fmp_error_detail(
            r#"backend failed {"api_key":"one","apiKey":"two","apikey":"three","safe":"kept"}"#,
        );

        assert_text_contains_all(
            &detail,
            &[
                "\"api_key\":\"<redacted>\"",
                "\"apiKey\":\"<redacted>\"",
                "\"apikey\":\"<redacted>\"",
                "\"safe\":\"kept\"",
            ],
        );
        assert_text_contains_none(&detail, &["\"one\"", "\"two\"", "\"three\""]);
    }

    #[test]
    fn fmp_error_detail_redacts_url_userinfo() {
        let detail = sanitize_fmp_error_detail(
            "error sending request for url (https://user:secret@example.com/api/v3/quote/AAPL)",
        );
        assert_eq!(
            detail,
            "error sending request for url (https://<redacted>@example.com/api/v3/quote/AAPL)"
        );
    }

    #[test]
    fn snapshot_is_exposed_in_tool_schema() {
        let tool = tool_with_test_key();
        let parameters = tool.parameters();
        let data_type = parameters
            .iter()
            .find(|parameter| parameter.name == "data_type")
            .expect("data_type parameter");
        let enum_values = data_type.r#enum.as_ref().expect("enum values");
        assert!(enum_values.iter().any(|value| value == "snapshot"));
        assert!(enum_values.iter().any(|value| value == "earnings_outlook"));
        assert!(enum_values.iter().any(|value| value == "quote_short"));
        assert!(enum_values.iter().any(|value| value == "extended_hours"));
        assert!(enum_values.iter().any(|value| value == "search"));
        assert!(
            tool.parameters()
                .iter()
                .any(|parameter| parameter.name == "query")
        );
        let identity_match = parameters
            .iter()
            .find(|parameter| parameter.name == "identity_match")
            .expect("identity_match parameter");
        assert_eq!(
            identity_match.r#enum.as_deref(),
            Some(["exact_symbol".to_string(), "name_or_alias".to_string()].as_slice())
        );
        assert!(
            parameters
                .iter()
                .any(|parameter| parameter.name == "entity_route")
        );
        assert!(
            parameters
                .iter()
                .any(|parameter| parameter.name == "supersedes_query")
        );
        assert!(tool.description().contains("必须先用 search"));
    }

    #[test]
    fn snapshot_response_aggregates_quote_profile_and_news() {
        let tool = tool_with_test_key();
        let payload = tool.build_snapshot_response(
            "AAPL",
            Ok(json!([{ "symbol": "AAPL", "price": 100.0 }])),
            Ok(json!([{
                "symbol": "AAPL",
                "companyName": "Apple Inc.",
                "exchangeShortName": "NASDAQ",
                "isActivelyTrading": true
            }])),
            Ok(json!([{ "title": "Example headline" }])),
        );

        assert_eq!(payload["data_type"], "snapshot");
        assert_eq!(payload["ticker"], "AAPL");
        assert_eq!(payload["data"]["quote"][0]["symbol"], "AAPL");
        assert_eq!(payload["data"]["profile"][0]["companyName"], "Apple Inc.");
        assert_eq!(payload["data"]["news"][0]["title"], "Example headline");
        assert_eq!(
            payload["hone_security_listing_evidence"]["status"],
            "active_listing"
        );
        assert!(payload.get("error").is_none());
    }

    #[test]
    fn sndk_active_listing_evidence_overrides_stale_delisting_memory() {
        let tool = tool_with_test_key();
        let payload = tool.build_snapshot_response(
            "sndk",
            Ok(json!([{ "symbol": "SNDK", "price": 245.81 }])),
            Ok(json!([{
                "symbol": "SNDK",
                "companyName": "Sandisk Corporation",
                "exchange": "NASDAQ Global Select",
                "exchangeShortName": "NASDAQ",
                "isActivelyTrading": true
            }])),
            Ok(json!([])),
        );

        let evidence = &payload["hone_security_listing_evidence"];
        assert_eq!(evidence["status"], "active_listing");
        assert_eq!(evidence["requested_symbol"], "sndk");
        assert_eq!(evidence["quote_symbol"], "SNDK");
        assert_eq!(evidence["profile_symbol"], "SNDK");
        assert_eq!(evidence["exchange"], "NASDAQ");
        assert_eq!(evidence["profile_is_actively_trading"], true);
        assert_eq!(evidence["current_quote_available"], true);
        assert!(
            evidence["policy"]
                .as_str()
                .is_some_and(|policy| policy.contains("stale model memory"))
        );
    }

    #[test]
    fn sndk_earnings_outlook_carries_current_listing_evidence() {
        let tool = tool_with_test_key();
        let payload = tool.build_earnings_outlook_response(
            "sndk",
            Ok(json!([{ "symbol": "SNDK", "price": 245.81 }])),
            Ok(json!([{
                "symbol": "SNDK",
                "companyName": "Sandisk Corporation",
                "exchangeShortName": "NASDAQ",
                "isActivelyTrading": true
            }])),
            Ok(json!([{ "symbol": "SNDK", "date": "2026-08-06" }])),
            Ok(json!([])),
            Ok(json!([])),
            Ok(json!([])),
            Ok(json!([])),
        );

        assert_eq!(payload["data_type"], "earnings_outlook");
        assert_eq!(payload["coverage"]["profile"], "available");
        assert_eq!(payload["data"]["profile"][0]["symbol"], "SNDK");
        assert_eq!(
            payload["hone_security_listing_evidence"]["status"],
            "active_listing"
        );
        assert_eq!(
            payload["hone_security_listing_evidence"]["requested_symbol"],
            "sndk"
        );
    }

    #[test]
    fn listing_evidence_stays_unverified_when_quote_and_profile_conflict() {
        let evidence = security_listing_evidence(
            "SNDK",
            &json!([{ "symbol": "SNDK", "price": 245.81 }]),
            &json!([{
                "symbol": "SNDK",
                "exchangeShortName": "NASDAQ",
                "isActivelyTrading": false
            }]),
        );

        assert_eq!(evidence["status"], "unverified");
        assert_eq!(evidence["current_quote_available"], true);
        assert_eq!(evidence["profile_is_actively_trading"], false);
    }

    #[test]
    fn extended_hours_normalization_keeps_latest_bar_and_session_summaries() {
        let payload = normalize_extended_hours_bar(
            "isrg",
            &json!([
                {"date":"2026-07-16 16:01:00","open":395.5,"close":395.0,"high":396.0,"low":394.5,"volume":1000},
                {"date":"2026-07-16 18:49:00","open":364.4,"close":363.25,"high":364.0,"low":362.5,"volume":2500},
                {"date":"2026-07-16 18:48:00","open":364.6,"close":364.5,"high":365.0,"low":364.0,"volume":2200},
                {"date":"invalid","close":999.0,"high":999.0,"low":999.0,"volume":1}
            ]),
        )
        .expect("normalized extended-hours payload");

        assert_eq!(payload["symbol"], "ISRG");
        assert_eq!(payload["price"], 363.25);
        assert_eq!(payload["date"], "2026-07-16 18:49:00");
        assert_eq!(payload["session"], "post");
        let summaries = payload["hone_session_summaries"]
            .as_array()
            .expect("session summaries");
        assert_eq!(summaries.len(), 1, "{summaries:?}");
        assert_eq!(summaries[0]["session"], "post");
        assert_eq!(summaries[0]["date_new_york"], "2026-07-16");
        assert_eq!(summaries[0]["open"], 395.5);
        assert_eq!(summaries[0]["close"], 363.25);
        assert_eq!(summaries[0]["high"], 396.0);
        assert_eq!(summaries[0]["low"], 362.5);
        assert_eq!(summaries[0]["volume"], 5700.0);
    }

    /// The reported failure: at a New York pre-market morning the user asks
    /// why a stock fell after hours. The latest bar is a pre bar, so keeping
    /// only it discards the post session where the move happened. Summaries
    /// must carry yesterday's post window with its change from the regular
    /// close, and today's pre window with its change from the post close.
    #[test]
    fn extended_hours_summaries_expose_the_post_market_move_across_days() {
        let payload = normalize_extended_hours_bar(
            "cohr",
            &json!([
                {"date":"2026-08-04 15:59:00","open":100.2,"close":100.0,"high":100.5,"low":99.8,"volume":5000},
                {"date":"2026-08-04 16:05:00","open":99.0,"close":96.0,"high":99.2,"low":95.8,"volume":800},
                {"date":"2026-08-04 19:55:00","open":90.4,"close":90.0,"high":90.6,"low":89.9,"volume":1200},
                {"date":"2026-08-05 04:20:00","open":90.5,"close":92.0,"high":92.1,"low":90.3,"volume":400}
            ]),
        )
        .expect("normalized extended-hours payload");

        // Latest bar is this morning's pre bar…
        assert_eq!(payload["session"], "pre");
        assert_eq!(payload["price"], 92.0);
        // …but the post session survives, with the drop quantified against the
        // regular close: (90 - 100) / 100 = -10%.
        let summaries = payload["hone_session_summaries"]
            .as_array()
            .expect("session summaries");
        let sessions: Vec<(&str, &str)> = summaries
            .iter()
            .map(|s| {
                (
                    s["date_new_york"].as_str().unwrap(),
                    s["session"].as_str().unwrap(),
                )
            })
            .collect();
        assert_eq!(
            sessions,
            [
                ("2026-08-04", "regular"),
                ("2026-08-04", "post"),
                ("2026-08-05", "pre"),
            ]
        );
        assert_eq!(summaries[1]["close"], 90.0);
        assert_eq!(summaries[1]["pct_change_vs_prev_session_close"], -10.0);
        assert_eq!(summaries[1]["pct_change_vs_regular_close"], -10.0);
        // Pre continues from the post close: (92 - 90) / 90 = +2.22%.
        assert_eq!(summaries[2]["pct_change_vs_prev_session_close"], 2.22);
        // But the canonical pre-market comparison remains the prior regular
        // close: (92 - 100) / 100 = -8%.
        assert_eq!(summaries[2]["pct_change_vs_previous_regular_close"], -8.0);
        // The first window has no in-data reference and must not invent one.
        assert!(
            summaries[0]
                .get("pct_change_vs_prev_session_close")
                .is_none()
        );
    }

    #[test]
    fn mrvl_regular_daily_change_cannot_use_the_premarket_denominator() {
        let payload = normalize_extended_hours_bar(
            "mrvl",
            &json!([
                {"date":"2026-08-20 15:59:00","open":250.0,"close":251.01,"high":252.0,"low":249.0,"volume":5000},
                {"date":"2026-08-21 09:29:00","open":252.0,"close":252.48,"high":253.0,"low":251.5,"volume":400},
                {"date":"2026-08-21 15:59:00","open":249.0,"close":237.04,"high":250.0,"low":233.28,"volume":9000},
                {"date":"2026-08-21 19:59:00","open":237.0,"close":236.10,"high":237.2,"low":235.8,"volume":600}
            ]),
        )
        .expect("normalized MRVL session payload");

        let summaries = payload["hone_session_summaries"]
            .as_array()
            .expect("session summaries");
        let regular = summaries
            .iter()
            .find(|summary| {
                summary["date_new_york"] == "2026-08-21" && summary["session"] == "regular"
            })
            .expect("August 21 regular session");
        assert_eq!(regular["pct_change_vs_prev_session_close"], -6.12);
        assert_eq!(regular["pct_change_close_to_close"], -5.57);
        assert_eq!(regular["previous_regular_close"], 251.01);
        assert_eq!(
            regular["canonical_change_basis"],
            "regular_close_vs_previous_regular_close"
        );

        let post = summaries
            .iter()
            .find(|summary| {
                summary["date_new_york"] == "2026-08-21" && summary["session"] == "post"
            })
            .expect("August 21 post-market session");
        assert_eq!(post["pct_change_vs_regular_close"], -0.4);
        assert!(
            payload["hone_session_change_policy"]
                .as_str()
                .expect("session policy")
                .contains("绝不能写成日涨跌幅")
        );
    }

    #[test]
    fn extended_hours_session_respects_actual_us_trading_boundaries() {
        let time = |hour, minute| chrono::NaiveTime::from_hms_opt(hour, minute, 0).unwrap();
        assert_eq!(extended_hours_session(time(3, 59)), "closed");
        assert_eq!(extended_hours_session(time(4, 0)), "pre");
        assert_eq!(extended_hours_session(time(9, 29)), "pre");
        assert_eq!(extended_hours_session(time(9, 30)), "regular");
        assert_eq!(extended_hours_session(time(16, 0)), "regular");
        assert_eq!(extended_hours_session(time(16, 1)), "post");
        assert_eq!(extended_hours_session(time(20, 0)), "post");
        assert_eq!(extended_hours_session(time(20, 1)), "closed");
    }

    #[tokio::test]
    async fn extended_hours_execute_returns_normalized_bar_instead_of_all_minutes() {
        let (addr, request_count) = spawn_scripted_http_server(vec![(
            "200 OK",
            r#"[{"date":"2026-07-16 08:15:00","close":401.5,"high":402.0,"low":401.0,"volume":300},{"date":"2026-07-16 08:16:00","close":402.25,"high":402.5,"low":401.75,"volume":400}]"#,
        )])
        .await;
        let tool = DataFetchTool::new(
            vec!["test_key".to_string()],
            &format!("http://{addr}/api"),
            30,
        );

        let payload = tool
            .execute(json!({"data_type":"extended_hours","ticker":"ISRG"}))
            .await
            .expect("extended-hours payload");
        assert_eq!(payload["data_type"], "extended_hours");
        assert_eq!(payload["data"]["symbol"], "ISRG");
        assert_eq!(payload["data"]["price"], 402.25);
        assert_eq!(payload["data"]["session"], "pre");
        assert!(payload["data"].is_object());
        assert_eq!(request_count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn snapshot_response_keeps_partial_errors_visible() {
        let tool = tool_with_test_key();
        let payload = tool.build_snapshot_response(
            "AAPL",
            Ok(json!([{ "symbol": "AAPL" }])),
            Err("profile failed".to_string()),
            Err("news failed".to_string()),
        );

        assert_eq!(payload["data"]["quote"][0]["symbol"], "AAPL");
        assert!(payload["data"]["profile"].is_null());
        assert!(payload["data"]["news"].is_null());
        assert_eq!(payload["errors"]["profile"], "profile failed");
        assert_eq!(payload["errors"]["news"], "news failed");
        assert!(payload.get("error").is_none());
    }

    #[test]
    fn earnings_outlook_keeps_partial_coverage_and_quality_visible() {
        let tool = tool_with_test_key();
        let payload = tool.build_earnings_outlook_response(
            "COHR",
            Ok(normalize_quote_timestamp_metadata(json!([{
                "symbol": "COHR",
                "price": 282.39
            }]))),
            Ok(json!([{
                "symbol": "COHR",
                "companyName": "Coherent Corp.",
                "exchangeShortName": "NYSE",
                "isActivelyTrading": true
            }])),
            Ok(json!([{"symbol":"COHR","date":"2026-08-12"}])),
            Err("estimates unavailable".to_string()),
            Ok(json!([{
                "symbol":"COHR",
                "targetLow":220.0,
                "targetHigh":360.0,
                "targetConsensus":300.0
            }])),
            Ok(json!([{"symbol":"COHR","rating":"Buy"}])),
            Ok(json!([{"symbol":"COHR","revenue":1_500_000_000_u64}])),
        );

        assert_eq!(payload["data_type"], "earnings_outlook");
        assert_eq!(payload["coverage"]["quote"], "available");
        assert_eq!(payload["coverage"]["profile"], "available");
        assert_eq!(payload["coverage"]["analyst_estimates"], "unavailable");
        assert_eq!(
            payload["errors"]["analyst_estimates"],
            "estimates unavailable"
        );
        assert_eq!(
            payload["hone_target_consensus_quality"]["usable_for_target_claims"],
            true
        );
        assert_eq!(
            payload["hone_security_listing_evidence"]["status"],
            "active_listing"
        );
        assert!(payload.get("error").is_none());
    }

    #[test]
    fn resolve_earnings_window_defaults_to_today_plus_14_days() {
        let tool = tool_with_test_key();
        let (from, to) = tool
            .resolve_earnings_window(&json!({ "data_type": "earnings_calendar" }))
            .expect("default earnings window");
        let today = hone_core::beijing_now().date_naive();
        assert_eq!(from, today);
        assert_eq!(to, today + Duration::days(14));
    }

    #[test]
    fn resolve_earnings_window_respects_explicit_dates() {
        let tool = tool_with_test_key();
        let (from, to) = tool
            .resolve_earnings_window(&json!({
                "data_type": "earnings_calendar",
                "from": "2026-04-10",
                "to": "2026-04-17"
            }))
            .expect("explicit earnings window");
        assert_eq!(from, NaiveDate::from_ymd_opt(2026, 4, 10).unwrap());
        assert_eq!(to, NaiveDate::from_ymd_opt(2026, 4, 17).unwrap());
    }

    #[test]
    fn build_earnings_calendar_url_uses_dynamic_dates() {
        let tool = tool_with_test_key();
        let from = NaiveDate::from_ymd_opt(2026, 4, 9).unwrap();
        let to = NaiveDate::from_ymd_opt(2026, 4, 23).unwrap();
        let url = tool.build_earnings_calendar_url(from, to);
        assert_eq!(
            url,
            "https://example.com/api/v3/earning_calendar?from=2026-04-09&to=2026-04-23"
        );
    }

    #[test]
    fn fmp_cache_key_strips_api_key_params() {
        let key = super::fmp_cache_key_for_url(
            "https://example.com/api/v3/quote/AAPL?apikey=secret&limit=10&api_key=two&apiKey=three",
        );
        assert_eq!(key, "https://example.com/api/v3/quote/AAPL?limit=10");
    }

    #[test]
    fn critical_entity_and_market_data_empty_values_are_not_cacheable() {
        for data_type in [
            "financials",
            "profile",
            "search",
            "etf_holdings",
            "quote",
            "quote_short",
            "extended_hours",
            "crypto_quote",
        ] {
            assert!(!should_cache_fmp_value(data_type, &json!(null)));
            assert!(!should_cache_fmp_value(data_type, &json!([])));
            assert!(!should_cache_fmp_value(data_type, &json!({})));
            assert!(!should_cache_fmp_value(data_type, &json!([{}])));
            assert!(!should_cache_fmp_value(data_type, &json!({ "data": [] })));
        }

        assert!(should_cache_fmp_value(
            "financials",
            &json!([{ "symbol": "AAPL" }])
        ));
        assert!(should_cache_fmp_value(
            "financials",
            &json!({ "symbol": "AAPL" })
        ));

        assert!(should_cache_fmp_value(
            "profile",
            &json!([{ "symbol": "AAPL" }])
        ));

        // 新闻等非实体/行情关键路径保持原有缓存行为，包括合法空响应。
        assert!(should_cache_fmp_value("news", &json!(null)));
    }

    #[test]
    fn error_message_field_is_nonempty_for_string_and_structured_errors() {
        assert_eq!(nonempty_fmp_error_message(&json!(null)), None);
        assert_eq!(nonempty_fmp_error_message(&json!("  ")), None);
        assert_eq!(nonempty_fmp_error_message(&json!([])), None);
        assert_eq!(nonempty_fmp_error_message(&json!({})), None);
        assert_eq!(
            nonempty_fmp_error_message(&json!("temporarily unavailable")),
            Some("temporarily unavailable".to_string())
        );
        assert_eq!(
            nonempty_fmp_error_message(&json!({ "code": "upstream_failure" })),
            Some(r#"{"code":"upstream_failure"}"#.to_string())
        );
    }

    #[tokio::test]
    async fn non_success_status_is_reported_as_provider_error_before_json_parsing() {
        let (addr, request_count) = spawn_scripted_http_server(vec![
            (
                "500 Internal Server Error",
                "upstream unavailable apikey=must-not-leak",
            ),
            ("500 Internal Server Error", "must not request second key"),
            ("500 Internal Server Error", "must not request third key"),
        ])
        .await;
        let tool = DataFetchTool::new(
            vec![
                "key_1".to_string(),
                "key_2".to_string(),
                "key_3".to_string(),
            ],
            &format!("http://{addr}/api"),
            30,
        );

        let payload = tool
            .execute(json!({"data_type": "quote", "ticker": "AAPL"}))
            .await
            .expect("provider error payload");
        let error = payload["error"].as_str().expect("error string");

        assert!(error.contains("FMP provider error（HTTP 500）"));
        assert!(error.contains("apikey=<redacted>"));
        assert!(!error.contains("must-not-leak"));
        assert!(!error.contains("JSON 解析失败"));
        assert!(!error.contains("所有 FMP API Key 均失败"));
        assert_eq!(request_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn transport_failure_does_not_fan_out_across_keys() {
        let (addr, request_count) = spawn_truncated_body_server().await;
        let tool = DataFetchTool::new(
            vec![
                "key_1".to_string(),
                "key_2".to_string(),
                "key_3".to_string(),
            ],
            &format!("http://{addr}/api"),
            30,
        );

        let payload = tool
            .execute(json!({"data_type": "quote", "ticker": "AAPL"}))
            .await
            .expect("transport error payload");
        let error = payload["error"].as_str().expect("error string");

        assert!(error.contains("FMP 响应读取失败"));
        assert!(!error.contains("所有 FMP API Key 均失败"));
        assert_eq!(request_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn json_parse_failure_does_not_fan_out_across_keys() {
        let (addr, request_count) = spawn_scripted_http_server(vec![
            ("200 OK", "not-json-1"),
            ("200 OK", "not-json-2"),
            ("200 OK", "not-json-3"),
        ])
        .await;
        let tool = DataFetchTool::new(
            vec![
                "key_1".to_string(),
                "key_2".to_string(),
                "key_3".to_string(),
            ],
            &format!("http://{addr}/api"),
            30,
        );

        let payload = tool
            .execute(json!({"data_type": "quote", "ticker": "AAPL"}))
            .await
            .expect("parse error payload");
        let error = payload["error"].as_str().expect("error string");

        assert!(error.contains("FMP JSON 解析失败"));
        assert!(!error.contains("所有 FMP API Key 均失败"));
        assert_eq!(request_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn authentication_statuses_still_fall_back_to_later_keys() {
        let (addr, request_count) = spawn_scripted_http_server(vec![
            ("401 Unauthorized", "not-json"),
            ("403 Forbidden", "still-not-json"),
            ("429 Too Many Requests", "quota exhausted"),
            (
                "200 OK",
                r#"[{"symbol":"AAPL","price":100.0,"timestamp":1784318400}]"#,
            ),
        ])
        .await;
        let tool = DataFetchTool::new(
            vec![
                "bad_key_1".to_string(),
                "bad_key_2".to_string(),
                "quota_key".to_string(),
                "working_key".to_string(),
            ],
            &format!("http://{addr}/api"),
            30,
        );

        let payload = tool
            .execute(json!({"data_type": "quote", "ticker": "AAPL"}))
            .await
            .expect("fallback quote payload");

        assert_eq!(payload["data"][0]["symbol"], "AAPL");
        assert_eq!(payload["data"][0]["price"], 100.0);
        assert_eq!(
            payload["data"][0]["hone_quote_time"]["beijing"],
            "2026-07-18 04:00:00 +08:00"
        );
        assert!(
            payload["data"][0]["hone_quote_time"]
                .get("session")
                .is_none()
        );
        assert_eq!(request_count.load(Ordering::SeqCst), 4);
    }

    #[tokio::test]
    async fn quote_short_and_crypto_quote_convert_time_without_inventing_sessions() {
        let (addr, request_count) = spawn_scripted_http_server(vec![
            (
                "200 OK",
                r#"[{"symbol":"AAPL","price":100.0,"timestamp":1784318400}]"#,
            ),
            (
                "200 OK",
                r#"[{"symbol":"BTCUSD","price":120000.0,"timestamp":1784318400}]"#,
            ),
        ])
        .await;
        let tool = DataFetchTool::new(
            vec!["working_key".to_string()],
            &format!("http://{addr}/api"),
            30,
        );

        let quote_short = tool
            .execute(json!({"data_type": "quote_short", "ticker": "AAPL"}))
            .await
            .expect("short quote payload");
        let crypto_quote = tool
            .execute(json!({"data_type": "crypto_quote", "ticker": "BTCUSD"}))
            .await
            .expect("crypto quote payload");

        for payload in [&quote_short, &crypto_quote] {
            assert_eq!(
                payload["data"][0]["hone_quote_time"]["beijing"],
                "2026-07-18 04:00:00 +08:00"
            );
            assert!(
                payload["data"][0]["hone_quote_time"]
                    .get("session")
                    .is_none(),
                "ordinary and continuously traded quotes must not inherit US extended-hours labels"
            );
        }
        assert_eq!(request_count.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn non_auth_error_message_in_success_response_is_provider_error() {
        let (addr, request_count) = spawn_scripted_http_server(vec![
            (
                "200 OK",
                r#"{"Error Message":"temporary upstream calculation failure"}"#,
            ),
            (
                "200 OK",
                r#"{"Error Message":"must not request second key"}"#,
            ),
            (
                "200 OK",
                r#"{"Error Message":"must not request third key"}"#,
            ),
        ])
        .await;
        let tool = DataFetchTool::new(
            vec![
                "key_1".to_string(),
                "key_2".to_string(),
                "key_3".to_string(),
            ],
            &format!("http://{addr}/api"),
            30,
        );

        let payload = tool
            .execute(json!({"data_type": "quote", "ticker": "AAPL"}))
            .await
            .expect("provider error payload");
        let error = payload["error"].as_str().expect("error string");

        assert!(error.contains("FMP provider error（HTTP 200）"));
        assert!(error.contains("temporary upstream calculation failure"));
        assert!(!error.contains("所有 FMP API Key 均失败"));
        assert!(payload.get("data").is_none());
        assert_eq!(request_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn auth_and_quota_error_messages_still_fall_back_to_later_keys() {
        let (addr, request_count) = spawn_scripted_http_server(vec![
            ("200 OK", r#"{"Error Message":"Invalid API KEY."}"#),
            (
                "200 OK",
                r#"{"Error Message":"Limit Reach. Please upgrade your plan."}"#,
            ),
            ("200 OK", r#"[{"symbol":"AAPL","price":101.0}]"#),
        ])
        .await;
        let tool = DataFetchTool::new(
            vec![
                "bad_key".to_string(),
                "quota_key".to_string(),
                "working_key".to_string(),
            ],
            &format!("http://{addr}/api"),
            30,
        );

        let payload = tool
            .execute(json!({"data_type": "quote", "ticker": "AAPL"}))
            .await
            .expect("fallback quote payload");

        assert_eq!(payload["data"][0]["symbol"], "AAPL");
        assert_eq!(payload["data"][0]["price"], 101.0);
        assert_eq!(request_count.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn empty_financials_are_refetched_then_nonempty_result_is_cached() {
        let (addr, requests) = spawn_path_scripted_http_server(vec![(
            "income-statement/AAPL?limit=4",
            vec![
                "[]",
                r#"[{"symbol":"AAPL","date":"2025-09-30","revenue":1000}]"#,
            ],
        )])
        .await;
        let tool = DataFetchTool::new(
            vec!["test_key".to_string()],
            &format!("http://{addr}/api"),
            30,
        );

        let first = tool
            .execute(json!({"data_type": "financials", "ticker": "AAPL"}))
            .await
            .expect("first financials payload");
        let second = tool
            .execute(json!({"data_type": "financials", "ticker": "AAPL"}))
            .await
            .expect("second financials payload");
        let third = tool
            .execute(json!({"data_type": "financials", "ticker": "AAPL"}))
            .await
            .expect("cached financials payload");

        assert_eq!(first["data"], json!([]));
        assert_eq!(second["data"][0]["symbol"], "AAPL");
        assert_eq!(third, second);
        let annual_requests = requests
            .lock()
            .expect("read requests")
            .iter()
            .filter(|target| target.contains("income-statement/AAPL?limit=4"))
            .count();
        // Empty is refetched, the nonempty answer is cached from then on.
        assert_eq!(annual_requests, 2);
    }

    #[tokio::test]
    async fn financials_return_quarterly_statements_with_a_trailing_window() {
        let (addr, _requests) = spawn_path_scripted_http_server(vec![
            (
                "income-statement/SNDK?limit=4",
                vec![r#"[{"symbol":"SNDK","date":"2026-06-30","period":"FY","calendarYear":"2026","revenue":20248,"netIncome":11433}]"#],
            ),
            (
                "income-statement/SNDK?period=quarter",
                vec![
                    r#"[
                      {"symbol":"SNDK","date":"2026-06-30","period":"Q4","calendarYear":"2026","revenue":8965,"grossProfit":7584,"operatingIncome":7100,"netIncome":6903,"epsdiluted":43.97},
                      {"symbol":"SNDK","date":"2026-03-31","period":"Q3","calendarYear":"2026","revenue":5950,"grossProfit":4665,"operatingIncome":3900,"netIncome":3615,"epsdiluted":23.03},
                      {"symbol":"SNDK","date":"2025-12-31","period":"Q2","calendarYear":"2026","revenue":3400,"grossProfit":1900,"operatingIncome":1200,"netIncome":800,"epsdiluted":5.10},
                      {"symbol":"SNDK","date":"2025-09-30","period":"Q1","calendarYear":"2026","revenue":1933,"grossProfit":700,"operatingIncome":200,"netIncome":115,"epsdiluted":1.66},
                      {"symbol":"SNDK","date":"2025-06-30","period":"Q4","calendarYear":"2025","revenue":1901,"grossProfit":498,"operatingIncome":-90,"netIncome":-23,"epsdiluted":-0.16}
                    ]"#,
                ],
            ),
            (
                "cash-flow-statement/SNDK",
                vec![r#"[{"symbol":"SNDK","date":"2026-06-30","operatingCashFlow":7126,"freeCashFlow":6000}]"#],
            ),
            ("balance-sheet-statement/SNDK", vec!["[]"]),
        ])
        .await;
        let tool = DataFetchTool::new(
            vec!["test_key".to_string()],
            &format!("http://{addr}/api"),
            30,
        );

        let payload = tool
            .execute(json!({"data_type": "financials", "ticker": "SNDK"}))
            .await
            .expect("financials payload");

        // The annual array keeps its historical shape for existing consumers.
        assert_eq!(payload["data"][0]["symbol"], "SNDK");
        assert_eq!(
            payload["hone_statement_coverage"]["quarterly_income_statement"],
            "available"
        );
        // A statement that genuinely failed must stay disclosed as a gap.
        assert_eq!(
            payload["hone_statement_coverage"]["quarterly_balance_sheet"],
            "unavailable"
        );
        assert!(payload.get("hone_quarterly_balance_sheet").is_none());

        // 8965 + 5950 + 3400 + 1933
        assert_eq!(payload["hone_ttm"]["revenue"], 20248.0);
        assert_eq!(payload["hone_ttm"]["eps_diluted"], 73.76);
        assert_eq!(payload["hone_ttm"]["latest_period_end"], "2026-06-30");

        let latest = &payload["hone_latest_quarter"];
        assert_eq!(latest["period_label"], "Q4 2026");
        // 8965 / 5950 - 1
        assert_eq!(latest["revenue_qoq_pct"], 50.67);
        // 8965 / 1901 - 1
        assert_eq!(latest["revenue_yoy_pct"], 371.59);
        assert_eq!(latest["gross_margin_pct"], 84.6);
        assert_eq!(latest["operating_cash_flow"], 7126.0);
    }

    #[test]
    fn a_provider_trailing_window_that_predates_the_latest_release_is_quarantined() {
        // The provider still carries the pre-release TTM EPS while the turn has
        // already read the quarter that made it obsolete.
        let quote = json!([{"symbol":"SNDK","price":1350.50,"eps":29.63,"pe":45.58}]);
        let financials = json!({
            "hone_ttm": {"eps_diluted": 73.76, "latest_period_end": "2026-06-30"}
        });

        let basis = valuation_basis_quality(&quote, &financials);

        assert_eq!(
            basis["provider_ttm_includes_latest_reported_quarter"],
            false
        );
        assert_eq!(basis["usable_for_multiple_claims"], false);
        assert_eq!(basis["recomputed_ttm_eps"], 73.76);
        // 1350.50 / 73.76
        assert_eq!(basis["recomputed_pe"], 18.31);
        assert_eq!(
            basis["warnings"][0],
            "provider_ttm_excludes_latest_reported_quarter"
        );
    }

    #[test]
    fn a_provider_trailing_window_matching_the_reported_quarters_stays_usable() {
        let quote = json!([{"symbol":"SNDK","price":1350.50,"eps":73.76,"pe":18.31,"marketCap":209_000_000_000.0}]);
        let financials = json!({
            "hone_ttm": {"eps_diluted": 73.76},
            "hone_forward": {
                "eps": 180.0,
                "revenue": 41_800_000_000.0,
                "period_ends": ["2026-09-30", "2026-12-31", "2027-03-31", "2027-06-30"]
            }
        });

        let basis = valuation_basis_quality(&quote, &financials);

        assert_eq!(basis["provider_ttm_includes_latest_reported_quarter"], true);
        assert_eq!(basis["usable_for_multiple_claims"], true);
        assert_eq!(basis["warnings"], json!([]));
        // 1350.50 / 180 — the multiple that actually compares across a cycle.
        assert_eq!(basis["forward_pe"], 7.5);
        // 209_000 / 41_800
        assert_eq!(basis["forward_ps"], 5.0);
        assert_eq!(basis["forward_period_ends"][0], "2026-09-30");
    }

    /// A trailing multiple built on a trough year and one built on a peak year
    /// describe the same business completely differently, which is how a
    /// comparison table ends up with a 19.8x and a 43.4x that mean nothing
    /// beside each other.
    #[test]
    fn forward_estimates_use_only_periods_after_the_latest_reported_quarter() {
        let estimates = json!([
            // Already reported — must never enter the forward window.
            {"date":"2026-06-30","epsAvg":40.0,"revenueAvg":8_800_000_000.0,"numAnalystsEps":21},
            {"date":"2026-03-31","epsAvg":22.0,"revenueAvg":5_900_000_000.0,"numAnalystsEps":20},
            {"date":"2026-09-30","epsAvg":45.0,"revenueAvg":10_500_000_000.0,"numAnalystsEps":19},
            {"date":"2026-12-31","epsAvg":47.0,"revenueAvg":11_000_000_000.0,"numAnalystsEps":17},
            {"date":"2027-03-31","epsAvg":44.0,"revenueAvg":10_200_000_000.0,"numAnalystsEps":12},
            {"date":"2027-06-30","epsAvg":44.0,"revenueAvg":10_100_000_000.0,"numAnalystsEps":9}
        ]);
        let rows = estimates.as_array().expect("rows");

        let forward =
            forward_twelve_month_summary(rows, Some("2026-06-30")).expect("forward window");

        assert_eq!(
            forward["period_ends"],
            json!(["2026-09-30", "2026-12-31", "2027-03-31", "2027-06-30"])
        );
        // 45 + 47 + 44 + 44
        assert_eq!(forward["eps"], 180.0);
        assert_eq!(forward["revenue"], 41_800_000_000.0f64);
        // Thin coverage on the far quarters is disclosed, not averaged away.
        assert_eq!(forward["min_analyst_count"], 9.0);

        // Fewer than four forward periods is not a trailing-twelve-month
        // estimate and must not be published as one.
        assert!(forward_twelve_month_summary(rows, Some("2026-12-31")).is_none());
    }

    #[test]
    fn an_unverifiable_trailing_window_is_not_reported_as_confirmed() {
        let quote = json!([{"symbol":"SNDK","price":1350.50,"eps":29.63}]);

        let basis = valuation_basis_quality(&quote, &json!({}));

        assert_eq!(
            basis["provider_ttm_includes_latest_reported_quarter"],
            serde_json::Value::Null
        );
        assert_eq!(basis["usable_for_multiple_claims"], false);
        assert_eq!(basis["warnings"][0], "provider_ttm_basis_unverified");
    }

    #[tokio::test]
    async fn empty_profile_is_refetched_then_nonempty_result_is_cached() {
        let (addr, request_count) = spawn_scripted_http_server(vec![
            ("200 OK", "[]"),
            (
                "200 OK",
                r#"[{"symbol":"INTL","companyName":"Main International ETF","isEtf":true}]"#,
            ),
        ])
        .await;
        let tool = DataFetchTool::new(
            vec!["test_key".to_string()],
            &format!("http://{addr}/api"),
            30,
        );

        let first = tool
            .execute(json!({"data_type": "profile", "ticker": "INTL"}))
            .await
            .expect("first profile payload");
        let second = tool
            .execute(json!({"data_type": "profile", "ticker": "INTL"}))
            .await
            .expect("second profile payload");
        let third = tool
            .execute(json!({"data_type": "profile", "ticker": "INTL"}))
            .await
            .expect("cached profile payload");

        assert_eq!(first["data"], json!([]));
        assert_eq!(second["data"][0]["isEtf"], true);
        assert_eq!(third, second);
        assert_eq!(request_count.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn repeated_snapshot_reuses_child_fetch_cache() {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind test server");
        let addr = listener.local_addr().expect("local addr");
        let request_count = Arc::new(AtomicUsize::new(0));
        let request_count_for_server = request_count.clone();
        tokio::spawn(async move {
            loop {
                let Ok((mut socket, _)) = listener.accept().await else {
                    break;
                };
                request_count_for_server.fetch_add(1, Ordering::SeqCst);
                tokio::spawn(async move {
                    let mut buf = [0_u8; 4096];
                    let n = socket.read(&mut buf).await.unwrap_or(0);
                    let request = String::from_utf8_lossy(&buf[..n]);
                    let body = if request.contains("/profile/") {
                        r#"[{"symbol":"AAPL","companyName":"Apple Inc."}]"#
                    } else if request.contains("/stock_news") {
                        r#"[{"title":"Apple headline"}]"#
                    } else {
                        r#"[{"symbol":"AAPL","price":100.0,"timestamp":1784318400}]"#
                    };
                    let response = format!(
                        "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    let _ = socket.write_all(response.as_bytes()).await;
                    let _ = socket.shutdown().await;
                });
            }
        });

        let tool = DataFetchTool::new(
            vec!["test_key".to_string()],
            &format!("http://{addr}/api"),
            30,
        );

        let first = tool
            .execute(json!({"data_type": "snapshot", "ticker": "AAPL"}))
            .await
            .expect("first snapshot");
        let second = tool
            .execute(json!({"data_type": "snapshot", "ticker": "AAPL"}))
            .await
            .expect("second snapshot");

        assert_eq!(first["data"]["quote"][0]["symbol"], "AAPL");
        assert_eq!(
            first["data"]["quote"][0]["hone_quote_time"]["beijing"],
            "2026-07-18 04:00:00 +08:00"
        );
        assert!(
            first["data"]["quote"][0]["hone_quote_time"]
                .get("session")
                .is_none()
        );
        assert_eq!(second["data"]["profile"][0]["companyName"], "Apple Inc.");
        // quote, profile, news, stock-price-change, aftermarket-quote — all
        // issued once and served from cache on the second call. The multi-period
        // performance and the provider's own post-market quote ride along with
        // the snapshot instead of costing a separate research round.
        assert_eq!(request_count.load(Ordering::SeqCst), 5);
        assert!(first["data"].get("price_change").is_some());
        assert!(first["data"].get("aftermarket_quote").is_some());
    }

    #[tokio::test]
    async fn a_bundle_reports_each_component_it_failed_to_get() {
        // A capability that silently drops a failed component reads as "this
        // company has no insider trades" rather than "that call failed".
        let (addr, _requests) = spawn_path_scripted_http_server(vec![
            (
                "key-metrics-ttm",
                vec![r#"[{"symbol":"SNDK","returnOnEquityTTM":1.24,"evToEBITDATTM":9.1}]"#],
            ),
            (
                "financial-scores",
                vec![r#"[{"symbol":"SNDK","altmanZScore":2.4,"piotroskiScore":7}]"#],
            ),
            ("ratios-ttm", vec!["[]"]),
        ])
        .await;
        let tool = DataFetchTool::new(
            vec!["test_key".to_string()],
            &format!("http://{addr}/api"),
            30,
        );

        let payload = tool
            .execute(json!({"data_type": "valuation", "ticker": "SNDK"}))
            .await
            .expect("valuation payload");

        assert_eq!(payload["data"]["key_metrics_ttm"][0]["evToEBITDATTM"], 9.1);
        assert_eq!(payload["coverage"]["key_metrics_ttm"], "available");
        // Present but empty is a different fact from never returned, and both
        // are different from "available".
        assert_eq!(payload["coverage"]["ratios_ttm"], "empty");
        assert_eq!(payload["coverage"]["discounted_cash_flow"], "empty");

        // The bands travel with the score so 2.4 is not left to be judged from
        // whatever threshold the model remembers.
        assert_eq!(payload["hone_score_semantics"]["altman_z_score"], 2.4);
        assert_eq!(payload["hone_score_semantics"]["altman_band"], "grey_zone");
        assert_eq!(payload["hone_score_semantics"]["piotroski_score"], 7.0);
    }

    #[tokio::test]
    async fn symbol_independent_bundles_resolve_without_a_ticker() {
        // Treasury rates and exchange hours are not properties of a security;
        // requiring a ticker for them would make the macro context reachable
        // only inside a company question.
        let (addr, _requests) = spawn_path_scripted_http_server(vec![
            (
                "treasury-rates",
                vec![r#"[{"date":"2026-08-06","month3":4.1,"year10":4.35}]"#],
            ),
            (
                "all-exchange-market-hours",
                vec![r#"[{"exchange":"NASDAQ","openingHour":"09:30","closingHour":"16:00"}]"#],
            ),
        ])
        .await;
        let tool = DataFetchTool::new(
            vec!["test_key".to_string()],
            &format!("http://{addr}/api"),
            30,
        );

        let macro_payload = tool
            .execute(json!({"data_type": "macro"}))
            .await
            .expect("macro payload");
        assert_eq!(macro_payload["data"]["treasury_rates"][0]["year10"], 4.35);
        assert_eq!(macro_payload["coverage"]["treasury_rates"], "available");

        let hours = tool
            .execute(json!({"data_type": "market_hours"}))
            .await
            .expect("market hours payload");
        assert_eq!(
            hours["data"]["all_exchange_market_hours"][0]["exchange"],
            "NASDAQ"
        );

        assert!(!data_fetch_data_type_uses_security_target("macro"));
        assert!(!data_fetch_data_type_uses_security_target("market_hours"));
        assert!(data_fetch_data_type_uses_security_target("valuation"));
        assert!(data_fetch_data_type_uses_security_target("segments"));
        assert!(data_fetch_data_type_uses_security_target("peers"));
    }

    /// A 45.57bn market cap was published as 4557 亿 — ten times too large —
    /// because converting a raw provider integer into 亿 was left to prose.
    /// One power of ten is invisible in a sentence and changes the conclusion.
    #[test]
    fn money_is_rendered_into_chinese_units_by_the_server() {
        let normalized = normalize_quote_timestamp_metadata(json!([{
            "symbol": "NBIS",
            "price": 189.88,
            "marketCap": 45_570_000_000.0_f64,
            "sharesOutstanding": 240_000_000.0_f64,
            "currency": "USD"
        }]));

        let display = &normalized[0]["hone_display"];
        // 45_570_000_000 / 1e8 = 455.7, not 4557.
        assert_eq!(display["market_cap"], "455.70 亿美元");
        assert_eq!(display["shares_outstanding"], "2.40 亿股");

        // The magnitude boundaries themselves.
        assert_eq!(
            chinese_scaled_money(1_020_000_000_000.0, "USD"),
            "1.02 万亿美元"
        );
        // 9999.9999 万 rounds to 10000.00 万, which reads as 一亿; it is promoted.
        assert_eq!(chinese_scaled_money(99_999_999.0, "USD"), "1.00 亿美元");
        assert_eq!(chinese_scaled_money(99_990_000.0, "USD"), "9999.00 万美元");
        assert_eq!(chinese_scaled_money(100_000_000.0, "USD"), "1.00 亿美元");
        // A negative figure keeps its sign rather than flipping magnitude.
        assert_eq!(
            chinese_scaled_money(-2_500_000_000.0, "USD"),
            "-25.00 亿美元"
        );
        // Non-USD listings keep their own unit instead of being called 美元.
        assert_eq!(
            chinese_scaled_money(1_061_000_000_000_000.0, "KRW"),
            "1061.00 万亿韩元"
        );
        // An unmapped currency falls back to the ISO code rather than guessing.
        assert!(chinese_scaled_money(5_000_000_000.0, "SEK").contains("SEK"));
    }

    #[test]
    fn altman_bands_follow_the_published_thresholds() {
        let band = |score: f64| {
            financial_score_semantics(&json!({"financial_scores": [{"altmanZScore": score}]}))
                ["altman_band"]
                .as_str()
                .map(str::to_string)
        };
        assert_eq!(band(3.0), Some("safe_zone".to_string()));
        assert_eq!(band(2.99), Some("grey_zone".to_string()));
        assert_eq!(band(1.81), Some("grey_zone".to_string()));
        assert_eq!(band(1.80), Some("distress_zone".to_string()));
        // No score means no band, rather than a default that reads as a verdict.
        assert_eq!(
            financial_score_semantics(&json!({}))["altman_band"],
            serde_json::Value::Null
        );
    }

    #[test]
    fn sec_recent_earnings_filing_finds_fresh_8k_and_press_exhibit() {
        let submissions = json!({
            "filings": {"recent": {
                "form": ["4", "8-K", "10-Q"],
                "filingDate": ["2026-08-11", "2026-08-11", "2026-05-08"],
                "reportDate": ["", "2026-08-11", "2026-03-31"],
                "accessionNumber": ["a", "0001769628-26-000362", "b"],
                "primaryDocument": ["form4.xml", "crwv-20260811.htm", "crwv-20260331.htm"],
                "items": ["", "2.02,9.01", ""]
            }}
        });
        let filing = latest_sec_earnings_filing_row(&submissions).expect("earnings filing");
        assert_eq!(filing.form, "8-K");
        assert_eq!(filing.filing_date, "2026-08-11");
        assert_eq!(filing.accession, "0001769628-26-000362");

        let index = json!({"directory":{"item":[
            {"name":"0001769628-26-000362-index.html"},
            {"name":"crwv-20260811.htm"},
            {"name":"coreweave2q26earningspress.htm"}
        ]}});
        assert_eq!(
            sec_earnings_exhibit_name(&index, "crwv-20260811.htm").as_deref(),
            Some("coreweave2q26earningspress.htm")
        );
    }

    #[test]
    fn sec_html_excerpt_preserves_financial_text_without_markup() {
        let html = "<html><body><h1>Second Quarter Results</h1><table><tr><td>Revenue</td><td>$2,575&nbsp;million</td></tr></table></body></html>";
        let text = sec_html_to_text_excerpt(html, 200);
        assert_eq!(text, "Second Quarter Results Revenue $2,575 million");
        assert!(!text.contains('<'));
    }

    #[test]
    fn sec_html_excerpt_keeps_late_cash_flow_and_balance_sheet_windows() {
        let filler = "narrative ".repeat(2_000);
        let html = format!(
            "<html><body><h1>Fiscal Results</h1><p>{filler}</p><h2>Condensed Consolidated Balance Sheets</h2><p>Cash and cash equivalents $4,762</p><h2>Cash Flow from Operating Activities</h2><p>$11,671</p><p>Purchases of property, plant and equipment $177</p></body></html>"
        );
        let text = sec_html_to_text_excerpt(&html, 2_000);
        assert!(text.starts_with("Fiscal Results"), "{text}");
        assert!(
            text.contains("Condensed Consolidated Balance Sheets"),
            "{text}"
        );
        assert!(text.contains("Cash and cash equivalents $4,762"), "{text}");
        assert!(
            text.contains("Cash Flow from Operating Activities"),
            "{text}"
        );
        assert!(
            text.contains("Purchases of property, plant and equipment $177"),
            "{text}"
        );
    }
}
