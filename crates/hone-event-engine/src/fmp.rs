//! FMP 最小 HTTP 客户端。
//!
//! 实现 multi-key fallback + 401/403 自动切换下一把 Key（与 `hone-tools/data_fetch.rs`
//! 一致的语义），供 pollers 复用。不做任何参数校验或 endpoint 封装，只负责把
//! path+query 变成带 apikey 的完整 URL，并返回解析后的 JSON。

use reqwest::StatusCode;
use serde_json::Value;
use std::time::Duration;

use hone_core::config::FmpConfig;

#[derive(Clone)]
pub struct FmpClient {
    keys: Vec<String>,
    base_url: String,
    timeout: Duration,
    http: reqwest::Client,
}

impl FmpClient {
    pub fn from_config(cfg: &FmpConfig) -> Self {
        let pool = cfg.effective_key_pool();
        // 显式启用 gzip:earning_calendar / stock_dividend_calendar 未压缩响应
        // 体可达数 MB,在 30s timeout 内拉不完(参考 v0.4.x 修复记录)。
        let http = reqwest::Client::builder()
            .gzip(true)
            .build()
            .expect("reqwest client init");
        Self {
            keys: pool.keys().to_vec(),
            base_url: cfg.base_url.trim_end_matches('/').to_string(),
            timeout: Duration::from_secs(cfg.timeout),
            http,
        }
    }

    /// 是否有可用的 Key。
    pub fn has_keys(&self) -> bool {
        !self.keys.is_empty()
    }

    /// `path_with_query` 形如 `"/v3/earning_calendar?from=2026-04-21&to=2026-05-05"`
    /// （以 `/` 开头）。函数拼接 base_url + apikey 后 GET。
    pub async fn get_json(&self, path_with_query: &str) -> anyhow::Result<Value> {
        if self.keys.is_empty() {
            anyhow::bail!("FMP API Key 未配置");
        }

        let url_base = format!("{}{}", self.base_url, path_with_query);
        let mut last_err: Option<anyhow::Error> = None;

        for key in &self.keys {
            let sep = if url_base.contains('?') { '&' } else { '?' };
            let full_url = format!("{url_base}{sep}apikey={key}");
            match self.fetch_once(&full_url).await {
                Ok(v) => return Ok(v),
                Err(e) => last_err = Some(e),
            }
        }

        Err(last_err.unwrap_or_else(|| anyhow::anyhow!("FMP: 无可用 Key")))
    }

    async fn fetch_once(&self, url: &str) -> anyhow::Result<Value> {
        let resp = self.http.get(url).timeout(self.timeout).send().await?;
        let status = resp.status();
        let body = resp.text().await?;
        let data: Value = serde_json::from_str(&body).map_err(|e| {
            let prefix: String = body.chars().take(200).collect();
            anyhow::anyhow!("FMP JSON 解析失败: {e}; body_prefix={prefix}")
        })?;

        if status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN {
            anyhow::bail!("FMP Key 无效（HTTP {status}）");
        }
        if let Some(err_msg) = data.get("Error Message").and_then(|v| v.as_str()) {
            let lower = err_msg.to_lowercase();
            if lower.contains("invalid api key")
                || lower.contains("api key")
                || lower.contains("limit reach")
                || lower.contains("upgrade")
            {
                anyhow::bail!("FMP Key 被拒绝: {err_msg}");
            }
        }
        Ok(data)
    }
}
