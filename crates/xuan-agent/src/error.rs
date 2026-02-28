//! XuanAgent 错误类型定义

use thiserror::Error;

/// XuanAgent 错误类型
#[derive(Error, Debug)]
pub enum Error {
    /// 配置错误
    #[error("配置错误: {0}")]
    Config(String),

    /// MCP 相关错误
    #[error("MCP 错误: {0}")]
    Mcp(String),

    /// 存储错误
    #[error("存储错误: {0}")]
    Storage(String),

    /// LLM 相关错误
    #[error("LLM 错误: {0}")]
    Llm(String),

    /// IO 错误
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    /// JSON 解析错误
    #[error("JSON 解析错误: {0}")]
    Json(#[from] serde_json::Error),

    /// HTTP 请求错误
    #[error("HTTP 请求错误: {0}")]
    Http(#[from] reqwest::Error),
}

/// XuanAgent 结果类型
pub type Result<T> = std::result::Result<T, Error>;
