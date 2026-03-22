//! WebSearchTool — 网络搜索工具
//!
//! 通过 Tavily API 进行网络搜索，支持多 Key 自动 fallback：
//! - 依次尝试 `search.api_keys` 中的每个 Key
//! - 若 Key 无效（401/403/exceeded）则切换到下一个
//! - 所有 Key 均失败时返回最后一次的错误信息

use async_trait::async_trait;
use serde_json::Value;

use crate::base::{Tool, ToolParameter};

/// WebSearchTool — 网络搜索（Tavily，多 Key fallback）
pub struct WebSearchTool {
    /// 有效 API Key 列表（过滤空值后）
    keys: Vec<String>,
    max_results: u32,
    http: reqwest::Client,
}

impl WebSearchTool {
    pub fn new(keys: Vec<String>, max_results: u32) -> Self {
        let pool = hone_core::ApiKeyPool::new(keys);
        Self {
            keys: pool.keys().to_vec(),
            max_results,
            http: reqwest::Client::new(),
        }
    }

    pub fn from_config(config: &hone_core::config::HoneConfig) -> Self {
        let pool = hone_core::ApiKeyPool::new(config.search.api_keys.iter().cloned());
        Self {
            keys: pool.keys().to_vec(),
            max_results: config.search.max_results,
            http: reqwest::Client::new(),
        }
    }

    /// 用指定 key 执行一次 Tavily 搜索，返回结果或错误
    async fn search_with_key(&self, key: &str, query: &str) -> Result<Value, String> {
        let body = serde_json::json!({
            "api_key": key,
            "query": query,
            "search_depth": "basic",
            "max_results": self.max_results,
            "include_answer": true,
            "include_raw_content": false
        });

        let resp = self
            .http
            .post("https://api.tavily.com/search")
            .json(&body)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await
            .map_err(|e| format!("Tavily 网络请求失败: {e}"))?;

        let status = resp.status();
        let data: Value = resp
            .json()
            .await
            .map_err(|e| format!("Tavily 响应解析失败: {e}"))?;

        // HTTP 401/403 或响应体含认证错误 → key 无效，触发 fallback
        if status == 401 || status == 403 {
            return Err(format!("Tavily API Key 无效（HTTP {status}）"));
        }

        // Tavily 在 HTTP 200 时也可能返回错误
        if let Some(detail) = data.get("detail").and_then(|v| v.as_str()) {
            let detail_lower = detail.to_lowercase();
            if detail_lower.contains("invalid api key")
                || detail_lower.contains("api key")
                || detail_lower.contains("exceeded")
                || detail_lower.contains("quota")
            {
                return Err(format!("Tavily API Key 被拒绝: {detail}"));
            }
        }

        Ok(data)
    }
}

#[async_trait]
impl Tool for WebSearchTool {
    fn name(&self) -> &str {
        "web_search"
    }

    fn description(&self) -> &str {
        "搜索互联网获取最新信息。当需要查找实时新闻、股票消息、公司动态或任何需要最新数据的问题时使用。"
    }

    fn parameters(&self) -> Vec<ToolParameter> {
        vec![ToolParameter {
            name: "query".to_string(),
            param_type: "string".to_string(),
            description: "搜索关键词（英文效果更好），例如 'AAPL latest news' 或 'Bitcoin price prediction 2024'".to_string(),
            required: true,
            r#enum: None,
            items: None,
        }]
    }

    async fn execute(&self, args: Value) -> hone_core::HoneResult<Value> {
        let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");

        if self.keys.is_empty() {
            return Ok(serde_json::json!({
                "error": "未配置 Tavily API Key（请在 config.yaml 中设置 search.api_keys）"
            }));
        }

        let mut last_err = String::new();

        for key in &self.keys {
            match self.search_with_key(key, query).await {
                Ok(data) => return Ok(data),
                Err(e) => {
                    last_err = e;
                    // 继续尝试下一个 key
                }
            }
        }

        // 所有 key 均失败
        Ok(serde_json::json!({
            "error": format!("所有 Tavily API Key 均失败（共 {} 个）。最后错误：{}", self.keys.len(), last_err)
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hone_core::config::HoneConfig;

    #[test]
    fn test_from_config() {
        let mut config = HoneConfig::default();
        config.search.api_keys = vec!["config_key".to_string()];
        config.search.max_results = 10;

        let tool = WebSearchTool::from_config(&config);
        assert_eq!(tool.keys, vec!["config_key"]);
        assert_eq!(tool.max_results, 10);
    }

    #[test]
    fn test_from_config_multi_keys() {
        let mut config = HoneConfig::default();
        config.search.api_keys = vec!["key1".to_string(), "key2".to_string(), "".to_string()];
        config.search.max_results = 5;

        let tool = WebSearchTool::from_config(&config);
        // 空 key 被过滤
        assert_eq!(tool.keys, vec!["key1", "key2"]);
    }

    #[test]
    fn test_empty_keys() {
        let tool = WebSearchTool::new(vec![], 5);
        assert!(tool.keys.is_empty());
    }
}
