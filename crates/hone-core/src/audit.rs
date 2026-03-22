use crate::beijing_now_rfc3339;
use crate::{ActorIdentity, HoneResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmAuditRecord {
    pub id: String,
    pub created_at: String,
    pub session_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actor: Option<ActorIdentity>,
    pub source: String,
    pub operation: String,
    pub provider: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u128>,
    pub request: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(default, skip_serializing_if = "Value::is_null")]
    pub metadata: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_tokens: Option<u32>,
}

impl LlmAuditRecord {
    pub fn new(
        session_id: impl Into<String>,
        actor: Option<ActorIdentity>,
        source: impl Into<String>,
        operation: impl Into<String>,
        provider: impl Into<String>,
        model: Option<String>,
        request: Value,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            created_at: beijing_now_rfc3339(),
            session_id: session_id.into(),
            actor,
            source: source.into(),
            operation: operation.into(),
            provider: provider.into(),
            model,
            success: false,
            latency_ms: None,
            request,
            response: None,
            error: None,
            metadata: Value::Null,
            prompt_tokens: None,
            completion_tokens: None,
            total_tokens: None,
        }
    }
}

pub trait LlmAuditSink: Send + Sync {
    fn record(&self, record: LlmAuditRecord) -> HoneResult<()>;
}
