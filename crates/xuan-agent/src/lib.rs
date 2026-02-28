#![allow(dead_code, unused_imports)]
//! # XuanAgent
//!
//! 科研 AI Agent 库，提供文献管理、学术写作辅助和科研知识管理功能。
//!
//! # Example
//!
//! ```rust,no_run
//! use xuan_agent::{XuanAgent, Config};
//!
//! #[tokio::main]
//! async fn main() -> xuan_agent::Result<()> {
//!     // 从 .env 加载配置
//!     let config = Config::from_env()?;
//!
//!     // 创建 Agent 实例
//!     let mut agent = XuanAgent::new(config).await?;
//!
//!     // 对话
//!     let response = agent.chat("你好").await?;
//!     println!("{}", response);
//!
//!     Ok(())
//! }
//! ```

pub mod agent;
pub mod config;
pub mod error;
pub mod llm;
pub mod mcp;
pub mod tools;
pub mod storage;
pub mod pdf;

// Re-export commonly used types
pub use agent::XuanAgent;
pub use config::{AiProviderConfig, Config, DbConfig, SystemPromptConfig};
pub use error::{Error, Result};
pub use storage::{Chunk, EmbeddingService, Paper, SearchResult, SurrealDBStorage};
pub use pdf::{PdfDocument, PdfParser, PaperChunker};
