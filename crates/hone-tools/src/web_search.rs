//! WebSearchTool — 网络搜索工具
//!
//! 通过 Tavily API 进行网络搜索，支持多 Key 自动 fallback：
//! - 依次尝试 `search.api_keys` 中的每个 Key
//! - 若 Key 无效（401/403/exceeded）则切换到下一个
//! - 所有 Key 均失败时返回最后一次的错误信息

use async_trait::async_trait;
use serde_json::Value;

use crate::base::{Tool, ToolParameter};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TavilyErrorKind {
    KeyRejected,
    TemporaryFailure,
}

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
            return Err(detail.to_string());
        }

        Ok(data)
    }

    fn classify_attempt_error(error: &str) -> TavilyErrorKind {
        let lower = error.to_lowercase();
        if lower.contains("invalid api key")
            || lower.contains("api key")
            || lower.contains("exceeded your plan")
            || lower.contains("quota")
            || lower.contains("rate limit")
            || lower.contains("upgrade your plan")
            || lower.contains("credits")
        {
            TavilyErrorKind::KeyRejected
        } else {
            TavilyErrorKind::TemporaryFailure
        }
    }

    fn final_user_error_message(
        &self,
        key_rejected_count: usize,
        temporary_failures: usize,
    ) -> String {
        if key_rejected_count > 0 && temporary_failures == 0 {
            format!(
                "Tavily 搜索当前不可用：已尝试 {} 个 API Key，但都因额度或鉴权被拒绝。请更新可用的 Tavily Key 后重试。",
                self.keys.len()
            )
        } else if temporary_failures > 0 && key_rejected_count == 0 {
            "Tavily 搜索当前暂时不可用，请稍后重试。".to_string()
        } else {
            format!(
                "Tavily 搜索当前不可用：已尝试 {} 个 API Key，但未获得可用响应。请稍后重试或检查 Tavily Key 配置。",
                self.keys.len()
            )
        }
    }

    fn unavailable_tool_result() -> Value {
        serde_json::json!({
            "status": "unavailable",
            "results": [],
            "answer": Value::Null
        })
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
            tracing::warn!(tool = "web_search", "tavily keys are empty");
            return Ok(Self::unavailable_tool_result());
        }

        let mut key_rejected_count = 0usize;
        let mut temporary_failures = 0usize;

        for (index, key) in self.keys.iter().enumerate() {
            match self.search_with_key(key, query).await {
                Ok(data) => return Ok(data),
                Err(e) => {
                    match Self::classify_attempt_error(&e) {
                        TavilyErrorKind::KeyRejected => key_rejected_count += 1,
                        TavilyErrorKind::TemporaryFailure => temporary_failures += 1,
                    }
                    tracing::warn!(
                        tool = "web_search",
                        key_index = index + 1,
                        key_count = self.keys.len(),
                        "tavily request failed for current api key: {}",
                        e
                    );
                    // 继续尝试下一个 key
                }
            }
        }

        // 所有 key 均失败
        tracing::warn!(
            tool = "web_search",
            key_count = self.keys.len(),
            key_rejected_count,
            temporary_failures,
            "{}",
            self.final_user_error_message(key_rejected_count, temporary_failures)
        );
        Ok(Self::unavailable_tool_result())
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

    #[test]
    fn classify_quota_error_as_key_rejected() {
        let error = "This request exceeds your plan's set usage limit. Please upgrade your plan or contact support@tavily.com";
        assert_eq!(
            WebSearchTool::classify_attempt_error(error),
            TavilyErrorKind::KeyRejected
        );
    }

    #[test]
    fn final_error_message_hides_raw_tavily_text() {
        let tool = WebSearchTool::new(vec!["key1".to_string(), "key2".to_string()], 5);
        let message = tool.final_user_error_message(2, 0);
        assert!(message.contains("已尝试 2 个 API Key"));
        assert!(!message.contains("support@tavily.com"));
        assert!(!message.contains("upgrade your plan"));
    }

    #[tokio::test]
    async fn execute_with_empty_keys_returns_sanitized_unavailable_payload() {
        let tool = WebSearchTool::new(vec![], 5);
        let result = tool
            .execute(serde_json::json!({"query": "oil"}))
            .await
            .expect("execute");
        assert_eq!(result["status"], "unavailable");
        assert!(result.get("error").is_none());
        assert!(result["results"].as_array().is_some());
    }
}
