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
        self.tools.values().map(|t| t.to_openai_schema()).collect()
    }

    /// 执行指定工具
    ///
    /// 执行前后均输出 INFO 级别日志，格式与 `FunctionCallingAgent` 保持一致，
    /// 方便跨 agent 模式统一追踪工具调用链路。
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

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}
