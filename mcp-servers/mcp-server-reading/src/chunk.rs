//! 文献切片模块
//!
//! 将长文本分割成适合向量检索和语义理解的小块

use serde::{Deserialize, Serialize};

/// 文献切片
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextChunk {
    /// 切片内容
    pub content: String,
    /// 切片索引
    pub index: usize,
    /// 切片类型
    pub chunk_type: ChunkType,
    /// 页码
    pub page_number: Option<usize>,
}

/// 切片类型
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// 文献切片器
pub struct Chunker {
    /// 最大块大小（字符数）
    max_chunk_size: usize,
    /// 块之间的重叠大小
    overlap_size: usize,
}

impl Chunker {
    /// 创建新的切片器
    pub fn new(max_chunk_size: usize, overlap_size: usize) -> Self {
        Self {
            max_chunk_size,
            overlap_size,
        }
    }

    /// 默认切片器（适合大多数文献）
    pub fn default() -> Self {
        Self {
            max_chunk_size: 1000,
            overlap_size: 200,
        }
    }

    /// 按段落切片
    pub fn chunk_by_paragraph(&self, text: &str) -> Vec<TextChunk> {
        let mut chunks = Vec::new();
        let mut current_chunk = String::new();
        let mut current_size = 0;
        let mut chunk_index = 0;

        // 按双换行符分割段落
        for paragraph in text.split("\n\n") {
            let paragraph = paragraph.trim();
            if paragraph.is_empty() {
                continue;
            }

            let para_size = paragraph.chars().count();

            // 如果单个段落超过最大大小，需要进一步分割
            if para_size > self.max_chunk_size {
                // 先保存当前块
                if !current_chunk.is_empty() {
                    chunks.push(TextChunk {
                        content: current_chunk.clone(),
                        index: chunk_index,
                        chunk_type: ChunkType::Other,
                        page_number: None,
                    });
                    chunk_index += 1;
                    current_chunk = String::new();
                    current_size = 0;
                }

                // 分割大段落
                let sub_chunks = self.split_large_paragraph(paragraph);
                for sub_chunk in sub_chunks {
                    chunks.push(TextChunk {
                        content: sub_chunk,
                        index: chunk_index,
                        chunk_type: ChunkType::Other,
                        page_number: None,
                    });
                    chunk_index += 1;
                }
                continue;
            }

            // 检查是否需要开始新块
            if current_size + para_size > self.max_chunk_size && !current_chunk.is_empty() {
                chunks.push(TextChunk {
                    content: current_chunk.clone(),
                    index: chunk_index,
                    chunk_type: ChunkType::Other,
                    page_number: None,
                });
                chunk_index += 1;
                current_chunk = String::new();
                current_size = 0;
            }

            // 添加段落到当前块
            if !current_chunk.is_empty() {
                current_chunk.push_str("\n\n");
            }
            current_chunk.push_str(paragraph);
            current_size += para_size;
        }

        // 添加最后一个块
        if !current_chunk.is_empty() {
            chunks.push(TextChunk {
                content: current_chunk,
                index: chunk_index,
                chunk_type: ChunkType::Other,
                page_number: None,
            });
        }

        chunks
    }

    /// 分割过大的段落
    fn split_large_paragraph(&self, paragraph: &str) -> Vec<String> {
        let mut chunks = Vec::new();
        let chars: Vec<char> = paragraph.chars().collect();
        let mut start = 0;

        while start < chars.len() {
            let end = std::cmp::min(start + self.max_chunk_size, chars.len());

            // 尝试在句子边界分割
            let mut split_pos = end;
            if end < chars.len() {
                // 从 end 向前查找句子结束符
                for i in (start..end).rev() {
                    let c = chars[i];
                    if c == '.' || c == '!' || c == '?' || c == '\n' {
                        split_pos = i + 1;
                        break;
                    }
                }
            }

            let chunk: String = chars[start..split_pos].iter().collect();
            chunks.push(chunk);

            // 移动到下一个块（带重叠）
            start = if split_pos + self.overlap_size < chars.len() {
                split_pos + self.overlap_size
            } else {
                split_pos
            };

            // 避免死循环
            if start <= end && end >= chars.len() {
                break;
            }
        }

        chunks
    }

    /// 按句子切片（保留上下文重叠）
    pub fn chunk_by_sentence(&self, text: &str) -> Vec<TextChunk> {
        let mut chunks = Vec::new();
        let mut current_chunk = String::new();
        let mut current_size = 0;
        let mut chunk_index = 0;

        // 简单的句子分割（按句号、问号、感叹号）
        let mut sentence_start = 0;
        let chars: Vec<char> = text.chars().collect();
        let mut sentence_end = 0;

        while sentence_end < chars.len() {
            let c = chars[sentence_end];

            // 检测句子结束
            if c == '.' || c == '?' || c == '!' {
                // 跳过句子结束符后的空白
                sentence_end += 1;
                while sentence_end < chars.len() && chars[sentence_end].is_whitespace() {
                    sentence_end += 1;
                }

                // 提取句子
                let sentence: String = chars[sentence_start..sentence_end]
                    .iter()
                    .collect();
                let sentence_size = sentence.chars().count();

                // 检查是否需要开始新块
                if !current_chunk.is_empty() && current_size + sentence_size > self.max_chunk_size {
                    chunks.push(TextChunk {
                        content: current_chunk.clone(),
                        index: chunk_index,
                        chunk_type: ChunkType::Other,
                        page_number: None,
                    });
                    chunk_index += 1;
                    current_chunk = String::new();
                    current_size = 0;
                }

                if !current_chunk.is_empty() {
                    current_chunk.push(' ');
                }
                current_chunk.push_str(&sentence);
                current_size += sentence_size;

                sentence_start = sentence_end;
            } else {
                sentence_end += 1;
            }
        }

        // 添加最后一个块
        if !current_chunk.is_empty() {
            chunks.push(TextChunk {
                content: current_chunk,
                index: chunk_index,
                chunk_type: ChunkType::Other,
                page_number: None,
            });
        }

        chunks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_by_paragraph() {
        let chunker = Chunker::default();
        let text = "First paragraph.\n\nSecond paragraph.\n\nThird paragraph.";
        let chunks = chunker.chunk_by_paragraph(text);

        assert!(!chunks.is_empty());
        assert!(chunks.len() <= 3);
    }

    #[test]
    fn test_chunk_by_sentence() {
        let chunker = Chunker::default();
        let text = "First sentence. Second sentence. Third sentence.";
        let chunks = chunker.chunk_by_sentence(text);

        assert!(!chunks.is_empty());
    }

    #[test]
    fn test_large_paragraph_splitting() {
        let chunker = Chunker::new(100, 20);
        let text = "A".repeat(500); // 500 字符
        let chunks = chunker.chunk_by_paragraph(&text);

        assert!(chunks.len() > 1, "Large paragraph should be split");
        // 验证每个块不超过限制（最后一个块可能较大）
        for (i, chunk) in chunks.iter().enumerate() {
            if i < chunks.len() - 1 {
                assert!(
                    chunk.content.len() <= 100,
                    "Chunk {} exceeds limit: {}",
                    i,
                    chunk.content.len()
                );
            }
        }
    }
}
