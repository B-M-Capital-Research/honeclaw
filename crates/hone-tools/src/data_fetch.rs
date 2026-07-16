//! DataFetchTool — 金融数据获取工具
//!
//! 通过 Financial Modeling Prep (FMP) API 获取金融数据，支持多 Key 自动 fallback：
//! - 依次尝试 `fmp.api_keys` 和 `fmp.api_key` 合并后的 Key 列表
//! - 若 Key 认证或配额不可用（HTTP 401/403/429 或响应含相关错误）则切换到下一个
//! - 所有 Key 均失败时返回最后一次的错误信息

use async_trait::async_trait;
use chrono::{Duration, NaiveDate};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration as StdDuration, Instant};

use crate::base::{Tool, ToolParameter};

const MAX_FMP_TRANSPORT_ERROR_CHARS: usize = 300;
const FMP_TTL_FAST: StdDuration = StdDuration::from_secs(5 * 60);
const FMP_TTL_NEWS: StdDuration = StdDuration::from_secs(15 * 60);
const FMP_TTL_PROFILE: StdDuration = StdDuration::from_secs(24 * 60 * 60);
const FMP_TTL_FINANCIALS: StdDuration = StdDuration::from_secs(6 * 60 * 60);
const FMP_TTL_EARNINGS: StdDuration = StdDuration::from_secs(60 * 60);

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
}

impl DataFetchTool {
    pub fn new(keys: Vec<String>, base_url: &str, timeout: u64) -> Self {
        let pool = hone_core::ApiKeyPool::new(keys);
        Self {
            keys: pool.keys().to_vec(),
            base_url: base_url.trim_end_matches('/').to_string(),
            timeout,
            http: reqwest::Client::new(),
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn from_config(config: &hone_core::config::HoneConfig) -> Self {
        let pool = config.fmp.effective_key_pool();
        Self {
            keys: pool.keys().to_vec(),
            base_url: config.fmp.base_url.trim_end_matches('/').to_string(),
            timeout: config.fmp.timeout,
            http: reqwest::Client::new(),
            cache: Arc::new(Mutex::new(HashMap::new())),
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
            "quote" => Ok(format!("{}/v3/quote/{}", self.base_url, ticker)),
            "quote_short" => Ok(format!(
                "{}/stable/batch-quote-short?symbols={}",
                self.stable_base_url(),
                ticker
            )),
            "profile" => Ok(format!("{}/v3/profile/{}", self.base_url, ticker)),
            "search" => {
                let query =
                    url::form_urlencoded::byte_serialize(ticker.as_bytes()).collect::<String>();
                Ok(format!(
                    "{}/v3/search?query={}&limit=10",
                    self.base_url, query
                ))
            }
            "financials" => Ok(format!(
                "{}/v3/income-statement/{}?limit=4",
                self.base_url, ticker
            )),
            "news" => {
                if ticker.is_empty() {
                    Ok(format!("{}/v3/stock_news?limit=10", self.base_url))
                } else {
                    Ok(format!(
                        "{}/v3/stock_news?tickers={}&limit=10",
                        self.base_url, ticker
                    ))
                }
            }
            "gainers_losers" => Ok(format!("{}/v3/stock_market/actives", self.base_url)),
            "sector_performance" => Ok(format!("{}/v3/sector-performance", self.base_url)),
            "crypto_quote" => Ok(format!("{}/v3/quote/{}", self.base_url, ticker)),
            "etf_holdings" => Ok(format!("{}/v3/etf-holder/{}", self.base_url, ticker)),
            "earnings_calendar" => Err(
                "earnings_calendar 需要显式窗口，通过 build_earnings_calendar_url 构造".to_string(),
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

        let mut payload = serde_json::json!({
            "data_type": "snapshot",
            "ticker": ticker,
            "data": {
                "quote": quote_value,
                "profile": profile_value,
                "news": news_value,
            }
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
        "quote" | "quote_short" | "crypto_quote" | "gainers_losers" | "sector_performance" => {
            Some(FMP_TTL_FAST)
        }
        "news" => Some(FMP_TTL_NEWS),
        "profile" | "search" | "etf_holdings" => Some(FMP_TTL_PROFILE),
        "financials" => Some(FMP_TTL_FINANCIALS),
        "earnings_calendar" => Some(FMP_TTL_EARNINGS),
        _ => None,
    }
}

fn should_cache_fmp_value(data_type: &str, value: &Value) -> bool {
    if !matches!(
        data_type,
        "financials"
            | "profile"
            | "search"
            | "etf_holdings"
            | "quote"
            | "quote_short"
            | "crypto_quote"
    ) {
        return true;
    }

    has_meaningful_fmp_value(value)
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
        "获取金融数据（股票/ETF/加密货币的实体、行情、基本面和新闻等）。公司或证券分析必须先用 search 按公司名、别名或代码解析标准实体，再用返回的 symbol 查询其它数据。支持的数据类型：search（实体搜索，返回 symbol/name/exchange/currency 候选）、quote（实时行情）、quote_short（低带宽简版批量行情）、profile（公司概况）、snapshot（聚合快照：quote + profile + news）、financials（财务数据）、news（新闻）、gainers_losers（涨跌榜）、sector_performance（板块表现）、crypto_quote（加密货币行情）、etf_holdings（ETF 持仓）、earnings_calendar（财报日历）。"
    }

    fn parameters(&self) -> Vec<ToolParameter> {
        vec![
            ToolParameter {
                name: "data_type".to_string(),
                param_type: "string".to_string(),
                description: "数据类型".to_string(),
                required: true,
                r#enum: Some(vec![
                    "quote".into(),
                    "quote_short".into(),
                    "profile".into(),
                    "snapshot".into(),
                    "financials".into(),
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
        let data_type = args
            .get("data_type")
            .and_then(|v| v.as_str())
            .unwrap_or("quote");
        let ticker = args
            .get(if data_type == "search" {
                "query"
            } else {
                "ticker"
            })
            .or_else(|| args.get("ticker"))
            .or_else(|| args.get("symbol"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if self.keys.is_empty() {
            return Ok(serde_json::json!({
                "error": "未配置 FMP API Key（请在 config.yaml 中设置 fmp.api_keys）"
            }));
        }

        if data_type == "snapshot" {
            let quote = self.fetch_data_type("quote", ticker).await;
            let profile = self.fetch_data_type("profile", ticker).await;
            let news = self.fetch_data_type("news", ticker).await;
            return Ok(self.build_snapshot_response(ticker, quote, profile, news));
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
            Ok(data) => Ok(serde_json::json!({
                "data_type": data_type,
                "ticker": ticker,
                "data": data
            })),
            Err(err) => Ok(serde_json::json!({ "error": err })),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DataFetchTool, nonempty_fmp_error_message, sanitize_fmp_error_detail,
        should_cache_fmp_value,
    };
    use crate::base::Tool;
    use crate::test_support::{assert_text_contains_all, assert_text_contains_none};
    use chrono::{Duration, NaiveDate};
    use serde_json::json;
    use std::net::SocketAddr;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

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
        assert!(enum_values.iter().any(|value| value == "quote_short"));
        assert!(enum_values.iter().any(|value| value == "search"));
        assert!(
            tool.parameters()
                .iter()
                .any(|parameter| parameter.name == "query")
        );
        assert!(tool.description().contains("必须先用 search"));
    }

    #[test]
    fn snapshot_response_aggregates_quote_profile_and_news() {
        let tool = tool_with_test_key();
        let payload = tool.build_snapshot_response(
            "AAPL",
            Ok(json!([{ "symbol": "AAPL", "price": 100.0 }])),
            Ok(json!([{ "symbol": "AAPL", "companyName": "Apple Inc." }])),
            Ok(json!([{ "title": "Example headline" }])),
        );

        assert_eq!(payload["data_type"], "snapshot");
        assert_eq!(payload["ticker"], "AAPL");
        assert_eq!(payload["data"]["quote"][0]["symbol"], "AAPL");
        assert_eq!(payload["data"]["profile"][0]["companyName"], "Apple Inc.");
        assert_eq!(payload["data"]["news"][0]["title"], "Example headline");
        assert!(payload.get("error").is_none());
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
            ("200 OK", r#"[{"symbol":"AAPL","price":100.0}]"#),
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
        assert_eq!(request_count.load(Ordering::SeqCst), 4);
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
        let (addr, request_count) = spawn_scripted_http_server(vec![
            ("200 OK", "[]"),
            (
                "200 OK",
                r#"[{"symbol":"AAPL","date":"2025-09-30","revenue":1000}]"#,
            ),
        ])
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
        assert_eq!(request_count.load(Ordering::SeqCst), 2);
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
                        r#"[{"symbol":"AAPL","price":100.0}]"#
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
        assert_eq!(second["data"]["profile"][0]["companyName"], "Apple Inc.");
        assert_eq!(request_count.load(Ordering::SeqCst), 3);
    }
}
