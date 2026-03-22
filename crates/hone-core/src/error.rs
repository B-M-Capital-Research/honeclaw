//! 全局错误类型

use thiserror::Error;

/// Hone 统一错误类型
#[derive(Debug, Error)]
pub enum HoneError {
    #[error("配置错误: {0}")]
    Config(String),

    #[error("LLM 错误: {0}")]
    Llm(String),

    #[error("工具执行错误: {0}")]
    Tool(String),

    #[error("存储错误: {0}")]
    Storage(String),

    #[error("集成错误: {0}")]
    Integration(String),

    #[error("渠道错误: {0}")]
    Channel(String),

    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("序列化错误: {0}")]
    Serialization(String),

    #[error("HTTP 错误: {0}")]
    Http(String),

    #[error("超时错误: {0}")]
    Timeout(String),

    #[error("{0}")]
    Other(String),
}

/// Hone 统一 Result 类型
pub type HoneResult<T> = Result<T, HoneError>;
