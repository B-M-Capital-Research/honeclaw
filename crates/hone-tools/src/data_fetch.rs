//! DataFetchTool — 金融数据获取工具
//!
//! 通过 Financial Modeling Prep (FMP) API 获取金融数据，支持多 Key 自动 fallback：
//! - 依次尝试 `fmp.api_keys` 和 `fmp.api_key` 合并后的 Key 列表
//! - 若 Key 无效（HTTP 401/403 或响应含认证错误）则切换到下一个
//! - 所有 Key 均失败时返回最后一次的错误信息

use async_trait::async_trait;
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
        let data: Value = resp
            .json()
            .await
            .map_err(|e| format!("FMP JSON 解析失败: {e}"))?;

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
}

#[async_trait]
impl Tool for DataFetchTool {
    fn name(&self) -> &str {
        "data_fetch"
    }

    fn description(&self) -> &str {
        "获取金融数据（股票/ETF/加密货币的行情、基本面、新闻等）。支持的数据类型：quote（实时行情）、profile（公司概况）、financials（财务数据）、news（新闻）、gainers_losers（涨跌榜）、sector_performance（板块表现）、crypto_quote（加密货币行情）、etf_holdings（ETF 持仓）、earnings_calendar（财报日历）。"
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

        let url = match data_type {
            "quote" => format!("{}/v3/quote/{}", self.base_url, ticker),
            "profile" => format!("{}/v3/profile/{}", self.base_url, ticker),
            "search" => format!("{}/v3/search?query={}&limit=10", self.base_url, ticker),
            "financials" => format!("{}/v3/income-statement/{}?limit=4", self.base_url, ticker),
            "news" => {
                if ticker.is_empty() {
                    format!("{}/v3/stock_news?limit=10", self.base_url)
                } else {
                    format!(
                        "{}/v3/stock_news?tickers={}&limit=10",
                        self.base_url, ticker
                    )
                }
            }
            "gainers_losers" => format!("{}/v3/stock_market/actives", self.base_url),
            "sector_performance" => format!("{}/v3/sector-performance", self.base_url),
            "crypto_quote" => format!("{}/v3/quote/{}", self.base_url, ticker),
            "etf_holdings" => format!("{}/v3/etf-holder/{}", self.base_url, ticker),
            "earnings_calendar" => format!(
                "{}/v3/earning_calendar?from=2024-01-01&to=2024-12-31",
                self.base_url
            ),
            _ => return Ok(serde_json::json!({"error": format!("不支持的数据类型: {data_type}")})),
        };

        if self.keys.is_empty() {
            return Ok(serde_json::json!({
                "error": "未配置 FMP API Key（请在 config.yaml 中设置 fmp.api_keys）"
            }));
        }

        let mut last_err = String::new();

        for key in &self.keys {
            match self.fetch_with_key(key, &url).await {
                Ok(data) => {
                    return Ok(serde_json::json!({
                        "data_type": data_type,
                        "ticker": ticker,
                        "data": data
                    }));
                }
                Err(e) => {
                    last_err = e;
                    // 继续尝试下一个 key
                }
            }
        }

        // 所有 key 均失败
        Ok(serde_json::json!({
            "error": format!("所有 FMP API Key 均失败（共 {} 个）。最后错误：{}", self.keys.len(), last_err)
        }))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_url_building() {
        let api_key = "test_key";
        let base_url = "https://example.com/api";

        // No query params
        let url1 = format!("{}/v3/quote/AAPL", base_url);
        let connector1 = if url1.contains('?') { "&" } else { "?" };
        let full_url1 = format!("{}{}apikey={}", url1, connector1, api_key);
        assert_eq!(
            full_url1,
            "https://example.com/api/v3/quote/AAPL?apikey=test_key"
        );

        // Has query params
        let url2 = format!("{}/v3/income-statement/AAPL?limit=4", base_url);
        let connector2 = if url2.contains('?') { "&" } else { "?" };
        let full_url2 = format!("{}{}apikey={}", url2, connector2, api_key);
        assert_eq!(
            full_url2,
            "https://example.com/api/v3/income-statement/AAPL?limit=4&apikey=test_key"
        );
    }
}
