//! 存储模块
//!
//! 提供文献持久化和向量检索功能

pub mod models;
pub mod surrealdb;
pub mod embedding;

// 重新导出常用类型
pub use models::{Chunk, ChunkType, CountResult, Paper, PaperInfo, SearchResult};
pub use surrealdb::SurrealDBStorage;
pub use embedding::EmbeddingService;
