//! 文献数据模型
//!
//! 定义文献、分块和搜索结果的数据结构

use serde::{Deserialize, Serialize};
use surrealdb_types::{RecordId, SurrealValue};

/// 文献元数据
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct Paper {
    /// 唯一标识符
    pub id: String,
    /// 标题
    pub title: String,
    /// 摘要
    #[serde(rename = "abstract")]
    pub abstract_text: String,
    /// 作者列表
    pub authors: Vec<String>,
    /// 发表年份
    pub year: Option<i32>,
    /// 期刊/会议名称
    pub journal: Option<String>,
    /// DOI
    pub doi: Option<String>,
    /// PDF 文件路径
    pub file_path: Option<String>,
    /// 标签
    pub tags: Vec<String>,
    /// 创建时间
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// 文献内容分块
///
/// 用于向量检索和语义搜索
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct Chunk {
    /// 唯一标识符
    pub id: String,
    /// 所属文献 ID
    pub paper_id: String,
    /// 分块内容
    pub content: String,
    /// 分块索引（在文献中的顺序）
    pub chunk_index: usize,
    /// 向量嵌入 (768 维，用于大多数嵌入模型)
    pub embedding: Option<Vec<f32>>,
    /// 页码（如果来自 PDF）
    pub page_number: Option<u32>,
    /// 分块类型
    pub chunk_type: ChunkType,
}

/// 分块类型
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub enum ChunkType {
    /// 标题
    Title,
    /// 摘要
    Abstract,
    /// 引言
    Introduction,
    /// 方法
    Methods,
    /// 结果
    Results,
    /// 讨论
    Discussion,
    /// 结论
    Conclusion,
    /// 参考文献
    References,
    /// 其他
    Other,
}

impl ChunkType {
    /// 从关键词推断分块类型
    pub fn from_text(text: &str) -> Self {
        let text_lower = text.to_lowercase();

        if text_lower.contains("abstract") || text_lower.contains("摘要") {
            ChunkType::Abstract
        } else if text_lower.contains("introduction") || text_lower.contains("引言") {
            ChunkType::Introduction
        } else if text_lower.contains("method") || text_lower.contains("方法") {
            ChunkType::Methods
        } else if text_lower.contains("result") || text_lower.contains("结果") {
            ChunkType::Results
        } else if text_lower.contains("discussion") || text_lower.contains("讨论") {
            ChunkType::Discussion
        } else if text_lower.contains("conclusion") || text_lower.contains("结论") {
            ChunkType::Conclusion
        } else if text_lower.contains("reference") || text_lower.contains("参考文献") {
            ChunkType::References
        } else {
            ChunkType::Other
        }
    }
}

/// 向量搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// 匹配的文献
    pub paper: PaperInfo,
    /// 匹配的分块
    pub chunk: ChunkInfo,
    /// 相似度分数 (0-1)
    pub similarity: f32,
}

/// 简化的文献信息（用于搜索结果）
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct PaperInfo {
    pub id: String,
    pub title: String,
    pub authors: Vec<String>,
    pub year: Option<i32>,
    pub journal: Option<String>,
}

/// 简化的分块信息（用于搜索结果）
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct ChunkInfo {
    pub id: String,
    pub content: String,
    pub chunk_index: usize,
    pub page_number: Option<u32>,
    pub chunk_type: ChunkType,
}

/// 统计结果包装器
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct CountResult {
    pub count: u64,
}

impl From<Paper> for PaperInfo {
    fn from(paper: Paper) -> Self {
        Self {
            id: paper.id,
            title: paper.title,
            authors: paper.authors,
            year: paper.year,
            journal: paper.journal,
        }
    }
}

impl From<Chunk> for ChunkInfo {
    fn from(chunk: Chunk) -> Self {
        Self {
            id: chunk.id,
            content: chunk.content,
            chunk_index: chunk.chunk_index,
            page_number: chunk.page_number,
            chunk_type: chunk.chunk_type,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_type_detection() {
        assert!(matches!(
            ChunkType::from_text("Abstract: This is..."),
            ChunkType::Abstract
        ));
        assert!(matches!(
            ChunkType::from_text("1. Introduction"),
            ChunkType::Introduction
        ));
        assert!(matches!(ChunkType::from_text("2. Methods"), ChunkType::Methods));
        assert!(matches!(ChunkType::from_text("3. Results"), ChunkType::Results));
    }
}
