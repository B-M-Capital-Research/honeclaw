//! Tool trait 定义
//!
//! 所有工具需要实现 Tool trait，定义遵循 OpenAI Function Calling 的 JSON Schema。

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// 工具参数定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParameter {
    pub name: String,
    /// string, number, boolean, object, array
    #[serde(rename = "type")]
    pub param_type: String,
    pub description: String,
    #[serde(default = "default_true")]
    pub required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#enum: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Value>,
}

fn default_true() -> bool {
    true
}

/// Tool trait
///
/// 所有工具需要实现：
/// - name: 工具名称（英文，用于 Function Calling）
/// - description: 工具描述（给 LLM 看，说明用途）
/// - parameters: 参数列表
/// - execute: 执行方法
#[async_trait]
pub trait Tool: Send + Sync {
    /// 工具名称
    fn name(&self) -> &str;

    /// 工具描述
    fn description(&self) -> &str;

    /// 工具参数列表
    fn parameters(&self) -> Vec<ToolParameter>;

    /// 执行工具
    async fn execute(&self, args: Value) -> hone_core::HoneResult<Value>;

    /// 转换为 OpenAI Function Calling 格式
    fn to_openai_schema(&self) -> Value {
        let params = self.parameters();
        let mut properties = serde_json::Map::new();
        let mut required = Vec::new();

        for param in &params {
            let mut prop = serde_json::Map::new();
            prop.insert("type".to_string(), Value::String(param.param_type.clone()));
            prop.insert(
                "description".to_string(),
                Value::String(param.description.clone()),
            );

            if let Some(ref enum_values) = param.r#enum {
                prop.insert(
                    "enum".to_string(),
                    Value::Array(
                        enum_values
                            .iter()
                            .map(|v| Value::String(v.clone()))
                            .collect(),
                    ),
                );
            }

            if let Some(ref items) = param.items {
                prop.insert("items".to_string(), items.clone());
            }

            properties.insert(param.name.clone(), Value::Object(prop));

            if param.required {
                required.push(Value::String(param.name.clone()));
            }
        }

        serde_json::json!({
            "type": "function",
            "function": {
                "name": self.name(),
                "description": self.description(),
                "parameters": {
                    "type": "object",
                    "properties": properties,
                    "required": required
                }
            }
        })
    }
}
