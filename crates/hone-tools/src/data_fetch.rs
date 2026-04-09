//! DataFetchTool — 金融数据获取工具
//!
//! 通过 Financial Modeling Prep (FMP) API 获取金融数据，支持多 Key 自动 fallback：
//! - 依次尝试 `fmp.api_keys` 和 `fmp.api_key` 合并后的 Key 列表
//! - 若 Key 无效（HTTP 401/403 或响应含认证错误）则切换到下一个
//! - 所有 Key 均失败时返回最后一次的错误信息

use async_trait::async_trait;
use chrono::{Duration, NaiveDate};
use serde_json::Value;

use crate::base::{Tool, ToolParameter};

/// DataFetchTool — 金融数据获取（FMP，多 Key fallback）
pub struct DataFetchTool {
    /// 有效 API Key 列表（过滤空值、去重后）
    keys: Vec<String>,
    base_url: String,
    timeout: u64,
    http: reqwest::Client,
}

impl DataFetchTool {
    pub fn new(keys: Vec<String>, base_url: &str, timeout: u64) -> Self {
        let pool = hone_core::ApiKeyPool::new(keys);
        Self {
            keys: pool.keys().to_vec(),
            base_url: base_url.trim_end_matches('/').to_string(),
            timeout,
            http: reqwest::Client::new(),
        }
    }

    pub fn from_config(config: &hone_core::config::HoneConfig) -> Self {
        let pool = config.fmp.effective_key_pool();
        Self {
            keys: pool.keys().to_vec(),
            base_url: config.fmp.base_url.trim_end_matches('/').to_string(),
            timeout: config.fmp.timeout,
            http: reqwest::Client::new(),
        }
    }

    /// 用指定 key 执行一次 FMP 请求
    async fn fetch_with_key(&self, key: &str, url: &str) -> Result<Value, String> {
        let connector = if url.contains('?') { "&" } else { "?" };
        let full_url = format!("{}{connector}apikey={}", url, key);

        let resp = self
            .http
            .get(&full_url)
            .timeout(std::time::Duration::from_secs(self.timeout))
            .send()
            .await
            .map_err(|e| format!("FMP API 请求失败: {e}"))?;

        let status = resp.status();
        let body = resp
            .text()
            .await
            .map_err(|e| format!("FMP 响应读取失败: {e}"))?;
        let data: Value = serde_json::from_str(&body).map_err(|e| {
            let prefix = body.chars().take(200).collect::<String>();
            format!("FMP JSON 解析失败: {e}; body_prefix={prefix}")
        })?;

        // HTTP 401/403 → key 无效，触发 fallback
        if status == 401 || status == 403 {
            return Err(format!("FMP API Key 无效（HTTP {status}）"));
        }

        // FMP 在 HTTP 200 时也可能返回认证错误（"Error Message" 字段）
        if let Some(err_msg) = data.get("Error Message").and_then(|v| v.as_str()) {
            let lower = err_msg.to_lowercase();
            if lower.contains("invalid api key")
                || lower.contains("api key")
                || lower.contains("limit reach")
                || lower.contains("upgrade")
            {
                return Err(format!("FMP API Key 被拒绝: {err_msg}"));
            }
        }

        Ok(data)
    }

    fn build_url(&self, data_type: &str, ticker: &str) -> Result<String, String> {
        match data_type {
            "quote" => Ok(format!("{}/v3/quote/{}", self.base_url, ticker)),
            "profile" => Ok(format!("{}/v3/profile/{}", self.base_url, ticker)),
            "search" => Ok(format!(
                "{}/v3/search?query={}&limit=10",
                self.base_url, ticker
            )),
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

    async fn fetch_data_type(&self, data_type: &str, ticker: &str) -> Result<Value, String> {
        let url = self.build_url(data_type, ticker)?;
        let mut last_err = String::new();

        for key in &self.keys {
            match self.fetch_with_key(key, &url).await {
                Ok(data) => return Ok(data),
                Err(e) => last_err = e,
            }
        }

        Err(format!(
            "所有 FMP API Key 均失败（共 {} 个）。最后错误：{}",
            self.keys.len(),
            last_err
        ))
    }

    async fn fetch_from_url(&self, url: &str) -> Result<Value, String> {
        let mut last_err = String::new();

        for key in &self.keys {
            match self.fetch_with_key(key, url).await {
                Ok(data) => return Ok(data),
                Err(e) => last_err = e,
            }
        }

        Err(format!(
            "所有 FMP API Key 均失败（共 {} 个）。最后错误：{}",
            self.keys.len(),
            last_err
        ))
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

#[async_trait]
impl Tool for DataFetchTool {
    fn name(&self) -> &str {
        "data_fetch"
    }

    fn description(&self) -> &str {
        "获取金融数据（股票/ETF/加密货币的行情、基本面、新闻等）。支持的数据类型：quote（实时行情）、profile（公司概况）、snapshot（聚合快照：quote + profile + news）、financials（财务数据）、news（新闻）、gainers_losers（涨跌榜）、sector_performance（板块表现）、crypto_quote（加密货币行情）、etf_holdings（ETF 持仓）、earnings_calendar（财报日历，默认查询当前北京时间起未来 14 天，也支持 from/to 覆盖窗口）。"
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
                name: "ticker".to_string(),
                param_type: "string".to_string(),
                description: "股票/ETF/加密货币代码（如 AAPL, BTCUSD）".to_string(),
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
            .get("ticker")
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
            return match self.fetch_from_url(&url).await {
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
    use super::DataFetchTool;
    use crate::base::Tool;
    use chrono::{Duration, NaiveDate};
    use serde_json::json;

    #[test]
    fn test_url_building() {
        let tool = DataFetchTool::new(vec!["test_key".to_string()], "https://example.com/api", 30);

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
    }

    #[test]
    fn snapshot_is_exposed_in_tool_schema() {
        let tool = DataFetchTool::new(vec!["test_key".to_string()], "https://example.com/api", 30);
        let parameters = tool.parameters();
        let data_type = parameters
            .iter()
            .find(|parameter| parameter.name == "data_type")
            .expect("data_type parameter");
        let enum_values = data_type.r#enum.as_ref().expect("enum values");
        assert!(enum_values.iter().any(|value| value == "snapshot"));
    }

    #[test]
    fn snapshot_response_aggregates_quote_profile_and_news() {
        let tool = DataFetchTool::new(vec!["test_key".to_string()], "https://example.com/api", 30);
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
        let tool = DataFetchTool::new(vec!["test_key".to_string()], "https://example.com/api", 30);
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
        let tool = DataFetchTool::new(vec!["test_key".to_string()], "https://example.com/api", 30);
        let (from, to) = tool
            .resolve_earnings_window(&json!({ "data_type": "earnings_calendar" }))
            .expect("default earnings window");
        let today = hone_core::beijing_now().date_naive();
        assert_eq!(from, today);
        assert_eq!(to, today + Duration::days(14));
    }

    #[test]
    fn resolve_earnings_window_respects_explicit_dates() {
        let tool = DataFetchTool::new(vec!["test_key".to_string()], "https://example.com/api", 30);
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
        let tool = DataFetchTool::new(vec!["test_key".to_string()], "https://example.com/api", 30);
        let from = NaiveDate::from_ymd_opt(2026, 4, 9).unwrap();
        let to = NaiveDate::from_ymd_opt(2026, 4, 23).unwrap();
        let url = tool.build_earnings_calendar_url(from, to);
        assert_eq!(
            url,
            "https://example.com/api/v3/earning_calendar?from=2026-04-09&to=2026-04-23"
        );
    }
}
