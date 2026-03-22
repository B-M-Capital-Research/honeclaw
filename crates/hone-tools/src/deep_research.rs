//! DeepResearchTool — 深度个股研究工具（管理员专属）
//!
//! 通过内部研究 API 启动对指定公司的深度研究任务。
//! API 端点通过 `DEEP_RESEARCH_API_URL` 环境变量配置，
//! 默认为 `http://127.0.0.1:18200/api/research/start`。

use async_trait::async_trait;
use serde_json::Value;

use crate::base::{Tool, ToolParameter};

/// DeepResearchTool — 启动深度个股研究任务
pub struct DeepResearchTool {
    /// 研究 API 端点（POST）
    api_url: String,
    /// 可选 Bearer 令牌（从环境变量 DEEP_RESEARCH_API_KEY 读取）
    api_key: String,
    http: reqwest::Client,
}

impl DeepResearchTool {
    pub fn new(api_url: &str, api_key: &str) -> Self {
        Self {
            api_url: api_url.to_string(),
            api_key: api_key.to_string(),
            http: reqwest::Client::new(),
        }
    }

    /// 从环境变量构造，优先读 DEEP_RESEARCH_API_URL；Key 可选
    pub fn from_env() -> Self {
        let api_url = std::env::var("DEEP_RESEARCH_API_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:18200/api/research/start".to_string());
        let api_key = std::env::var("DEEP_RESEARCH_API_KEY").unwrap_or_default();
        Self::new(&api_url, &api_key)
    }
}

#[async_trait]
impl Tool for DeepResearchTool {
    fn name(&self) -> &str {
        "deep_research"
    }

    fn description(&self) -> &str {
        "【管理员专属】启动对指定公司的深度个股研究任务。系统将异步执行约 1-2 小时的全面研究，完成后可在「个股研究」页面查看报告。调用后返回 task_id，系统每分钟自动汇报进度（最多 15 分钟）。"
    }

    fn parameters(&self) -> Vec<ToolParameter> {
        vec![ToolParameter {
            name: "company_name".to_string(),
            param_type: "string".to_string(),
            description: "公司名称、英文名或股票代码，例如：\"英伟达\"、\"NVIDIA\"、\"NVDA\"、\"比亚迪\"、\"AAPL\"".to_string(),
            required: true,
            r#enum: None,
            items: None,
        }]
    }

    async fn execute(&self, args: Value) -> hone_core::HoneResult<Value> {
        let company_name = args
            .get("company_name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim();

        if company_name.is_empty() {
            return Ok(serde_json::json!({
                "success": false,
                "error": "company_name 不能为空，请提供公司名、英文名或股票代码"
            }));
        }

        tracing::info!(
            "[DeepResearchTool] 启动深度研究 company={} api_url={}",
            company_name,
            self.api_url
        );

        let body = serde_json::json!({
            "company_name": company_name
        });

        let mut req = self
            .http
            .post(&self.api_url)
            .header("Content-Type", "application/json")
            .json(&body)
            .timeout(std::time::Duration::from_secs(30));

        if !self.api_key.is_empty() {
            req = req.header("Authorization", format!("Bearer {}", self.api_key));
        }

        let resp = match req.send().await {
            Ok(r) => r,
            Err(e) => {
                tracing::error!("[DeepResearchTool] API 请求失败: {}", e);
                return Ok(serde_json::json!({
                    "success": false,
                    "error": format!("研究 API 请求失败: {e}。请确认 DEEP_RESEARCH_API_URL 已正确配置（当前: {}）", self.api_url)
                }));
            }
        };

        let status = resp.status();
        let raw: Value = match resp.json().await {
            Ok(v) => v,
            Err(e) => {
                tracing::error!("[DeepResearchTool] 响应解析失败: {}", e);
                return Ok(serde_json::json!({
                    "success": false,
                    "error": format!("研究 API 响应解析失败: {e}")
                }));
            }
        };

        if !status.is_success() {
            let err_msg = raw
                .get("error")
                .or_else(|| raw.get("message"))
                .and_then(|v| v.as_str())
                .unwrap_or("未知错误");
            tracing::error!(
                "[DeepResearchTool] API 返回错误 status={} error={}",
                status,
                err_msg
            );
            return Ok(serde_json::json!({
                "success": false,
                "error": format!("研究 API 返回错误 (HTTP {}): {}", status, err_msg),
                "raw": raw
            }));
        }

        // 提取 task_id（兼容多种字段名）
        let task_id = raw
            .get("task_id")
            .or_else(|| raw.get("taskId"))
            .or_else(|| raw.get("id"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        tracing::info!(
            "[DeepResearchTool] 研究任务已启动 company={} task_id={}",
            company_name,
            task_id
        );

        Ok(serde_json::json!({
            "success": true,
            "task_id": task_id,
            "company_name": company_name,
            "message": format!("已成功启动 {} 的深度研究任务，系统将每分钟汇报一次进度，最多监控 15 分钟。完整报告约需 1-2 小时，完成后可在「个股研究」页面查阅。", company_name),
            "raw": raw
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_name_and_description() {
        let tool = DeepResearchTool::new("http://127.0.0.1:18200/api/research/start", "");
        assert_eq!(tool.name(), "deep_research");
        assert!(!tool.description().is_empty());
    }

    #[test]
    fn parameters_have_company_name() {
        let tool = DeepResearchTool::new("http://127.0.0.1:18200/api/research/start", "");
        let params = tool.parameters();
        assert_eq!(params.len(), 1);
        assert_eq!(params[0].name, "company_name");
        assert!(params[0].required);
    }

    #[tokio::test]
    async fn execute_empty_company_name_returns_error() {
        let tool = DeepResearchTool::new("http://127.0.0.1:18200/api/research/start", "");
        let result = tool
            .execute(serde_json::json!({"company_name": ""}))
            .await
            .expect("execute should not panic");
        assert_eq!(result["success"], false);
        assert!(
            result["error"]
                .as_str()
                .unwrap_or("")
                .contains("company_name")
        );
    }

    #[tokio::test]
    async fn execute_network_failure_returns_structured_error() {
        // 使用一个必然失败的端口
        let tool = DeepResearchTool::new("http://127.0.0.1:19", "");
        let result = tool
            .execute(serde_json::json!({"company_name": "NVIDIA"}))
            .await
            .expect("execute should not panic");
        assert_eq!(result["success"], false);
        let err = result["error"].as_str().unwrap_or_default();
        assert!(!err.is_empty(), "error should have message");
    }
}
