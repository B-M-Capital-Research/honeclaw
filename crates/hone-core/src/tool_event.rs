use async_trait::async_trait;

#[async_trait]
pub trait ToolExecutionObserver: Send + Sync {
    async fn on_tool_start(&self, tool_name: &str, reasoning: Option<String>);
    async fn on_tool_finish(&self, tool_name: &str, success: bool);
}
