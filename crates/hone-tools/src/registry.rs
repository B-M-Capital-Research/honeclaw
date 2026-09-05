//! ToolRegistry — 工具注册与发现
//!
//! 管理所有可用工具的注册表。

use serde_json::Value;
use std::collections::HashMap;

use crate::base::Tool;
use crate::guard::ToolExecutionGuard;

/// 工具注册表
pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
    guard: ToolExecutionGuard,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self::new_with_guard(ToolExecutionGuard::disabled())
    }

    pub fn new_with_guard(guard: ToolExecutionGuard) -> Self {
        Self {
            tools: HashMap::new(),
            guard,
        }
    }

    /// 注册一个工具
    pub fn register(&mut self, tool: Box<dyn Tool>) {
        let name = tool.name().to_string();
        tracing::debug!("注册工具: {}", name);
        self.tools.insert(name, tool);
    }

    /// 获取工具
    pub fn get(&self, name: &str) -> Option<&dyn Tool> {
        self.tools.get(name).map(|t| t.as_ref())
    }

    /// 列出所有工具名称
    pub fn list_tool_names(&self) -> Vec<&str> {
        self.tools.keys().map(|k| k.as_str()).collect()
    }

    /// 获取所有工具的 OpenAI schema
    pub fn get_tools_schema(&self) -> Vec<Value> {
        let mut schemas = self
            .tools
            .values()
            .map(|tool| tool.to_openai_schema())
            .collect::<Vec<_>>();
        schemas.sort_by(|left, right| {
            let left_name = tool_schema_name(left);
            let right_name = tool_schema_name(right);
            tool_schema_priority(left_name).cmp(&tool_schema_priority(right_name))
        });
        schemas
    }

    /// 执行指定工具
    ///
    /// 执行前后均输出 INFO 级别日志，方便跨 runner 统一追踪工具调用链路。
    pub async fn execute_tool(&self, name: &str, args: Value) -> hone_core::HoneResult<Value> {
        let tool = self.tools.get(name).ok_or_else(|| {
            tracing::warn!("[ToolRegistry] tool_not_found name={}", name);
            hone_core::HoneError::Tool(format!("工具不存在: {name}"))
        })?;

        tracing::info!("[ToolRegistry] tool_execute_start name={}", name);

        if let Err(err) = self.guard.check(name, &args) {
            tracing::warn!(
                "[ToolRegistry] tool_execute_blocked name={} error={}",
                name,
                err
            );
            return Err(err);
        }

        match tool.execute(args).await {
            Ok(result) => {
                tracing::info!("[ToolRegistry] tool_execute_success name={}", name);
                Ok(result)
            }
            Err(e) => {
                tracing::error!(
                    "[ToolRegistry] tool_execute_error name={} error={}",
                    name,
                    e
                );
                Err(e)
            }
        }
    }

    /// 工具数量
    pub fn len(&self) -> usize {
        self.tools.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }
}

fn tool_schema_name(schema: &Value) -> &str {
    schema
        .get("function")
        .and_then(|function| function.get("name"))
        .and_then(Value::as_str)
        .unwrap_or_default()
}

/// Tool order is a model hint, not an execution policy. Structured market data
/// is shown before open Web search so an Agent reading a named-security request
/// sees the authoritative quote path first; no call is required or blocked by
/// this ordering.
fn tool_schema_priority(name: &str) -> u8 {
    match name {
        "data_fetch" => 0,
        "web_search" => 2,
        _ => 1,
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::ToolParameter;
    use async_trait::async_trait;

    struct NamedTool(&'static str);

    #[async_trait]
    impl Tool for NamedTool {
        fn name(&self) -> &str {
            self.0
        }

        fn description(&self) -> &str {
            "test tool"
        }

        fn parameters(&self) -> Vec<ToolParameter> {
            Vec::new()
        }

        async fn execute(&self, _args: Value) -> hone_core::HoneResult<Value> {
            Ok(Value::Null)
        }
    }

    #[test]
    fn schemas_present_market_data_before_open_search_without_filtering_tools() {
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(NamedTool("web_search")));
        registry.register(Box::new(NamedTool("portfolio")));
        registry.register(Box::new(NamedTool("data_fetch")));

        let schemas = registry.get_tools_schema();
        let names = schemas.iter().map(tool_schema_name).collect::<Vec<_>>();

        assert_eq!(names, ["data_fetch", "portfolio", "web_search"]);
        assert_eq!(names.len(), registry.len(), "ordering must not gate tools");
    }
}
