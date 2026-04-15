use async_trait::async_trait;
use serde_json::Value;

#[async_trait]
pub trait ToolExecutionObserver: Send + Sync {
    async fn on_tool_start(&self, tool_name: &str, arguments: &Value, reasoning: Option<String>);
    async fn on_tool_finish(&self, tool_name: &str, arguments: &Value, success: bool);
}
