//! Hone LLM — LLM Provider trait + OpenRouter 实现
//!
//! 提供与大语言模型交互的抽象层。

pub mod openrouter;
pub mod provider;

pub use openrouter::OpenRouterProvider;
pub use provider::{ChatResponse, FunctionCall, LlmProvider, Message, ToolCall};
