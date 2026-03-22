//! 通用 Agent 接口定义
//!
//! 将 function_calling / gemini_cli / codex_cli 抽象为统一接口。

use crate::ActorIdentity;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// 已执行的工具调用记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallMade {
    pub name: String,
    pub arguments: Value,
    pub result: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

/// Agent 流式事件（预留给未来的流式输出）
#[derive(Debug, Clone)]
pub enum AgentEvent {
    /// 文本 token
    Token { content: String },
    /// 工具调用开始
    ToolCall {
        id: String,
        name: String,
        arguments: Value,
    },
    /// 工具调用结果
    ToolResult {
        id: String,
        name: String,
        result: Value,
    },
    /// 错误
    Error { message: String },
    /// 完成
    Done {
        full_response: String,
        tool_calls_made: Vec<ToolCallMade>,
    },
}

/// Agent 同步响应
#[derive(Debug, Clone)]
pub struct AgentResponse {
    pub content: String,
    pub tool_calls_made: Vec<ToolCallMade>,
    pub iterations: u32,
    pub success: bool,
    pub error: Option<String>,
}

/// Agent 消息格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// Agent 上下文管理
#[derive(Debug, Clone)]
pub struct AgentContext {
    pub session_id: String,
    pub messages: Vec<AgentMessage>,
    pub metadata: HashMap<String, Value>,
}

impl AgentContext {
    pub fn new(session_id: String) -> Self {
        Self {
            session_id,
            messages: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn add_user_message(&mut self, content: &str) {
        self.messages.push(AgentMessage {
            role: "user".to_string(),
            content: Some(content.to_string()),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        });
    }

    pub fn add_assistant_message(&mut self, content: &str, tool_calls: Option<Vec<Value>>) {
        self.messages.push(AgentMessage {
            role: "assistant".to_string(),
            content: Some(content.to_string()),
            tool_calls,
            tool_call_id: None,
            name: None,
        });
    }

    pub fn add_tool_result(&mut self, tool_call_id: &str, tool_name: &str, result: &str) {
        self.messages.push(AgentMessage {
            role: "tool".to_string(),
            content: Some(result.to_string()),
            tool_calls: None,
            tool_call_id: Some(tool_call_id.to_string()),
            name: Some(tool_name.to_string()),
        });
    }

    pub fn set_actor_identity(&mut self, actor: &ActorIdentity) {
        self.metadata.insert(
            "actor".to_string(),
            serde_json::to_value(actor).unwrap_or(Value::Null),
        );
    }

    pub fn actor_identity(&self) -> Option<ActorIdentity> {
        self.metadata
            .get("actor")
            .cloned()
            .and_then(|value| serde_json::from_value(value).ok())
    }

    /// 转换为 LLM 消息格式 (如果需要)
    pub fn to_messages(&self) -> Vec<Value> {
        self.messages
            .iter()
            .map(|m| serde_json::to_value(m).unwrap_or_default())
            .collect()
    }
}

/// 可插拔 Agent 接口
#[async_trait]
pub trait Agent: Send + Sync {
    /// 运行单次交互
    async fn run(&self, user_input: &str, context: &mut AgentContext) -> AgentResponse;
}
