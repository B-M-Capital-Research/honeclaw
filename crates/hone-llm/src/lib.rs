//! Hone LLM — provider trait, profile resolver, and OpenAI-compatible backends
//!
//! 提供与大语言模型交互的抽象层。

pub mod openai_compatible;
pub mod openrouter;
pub mod provider;
pub mod resolver;

pub use openai_compatible::OpenAiCompatibleProvider;
pub use openrouter::OpenRouterProvider;
pub use provider::{
    ChatResponse, ChatStreamEvent, FunctionCall, LlmProvider, LlmRequestOptions, Message, ToolCall,
};
pub use resolver::{CreatedLlmProvider, LlmResolver};
