//! PDF 解析模块
//!
//! 提供从 PDF 文件中提取文本和元数据的功能

use crate::{Error, Result};
use std::path::Path;

/// PDF 文档信息
#[derive(Debug, Clone)]
pub struct PdfDocument {
    /// 文件路径
    pub file_path: String,
    /// 提取的文本内容
    pub text: String,
    /// 页数
    pub page_count: usize,
}

/// PDF 解析器
pub struct PdfParser;

impl PdfParser {
    /// 解析 PDF 文件
    pub fn parse(file_path: &str) -> Result<PdfDocument> {
        let path = Path::new(file_path);
        if !path.exists() {
            return Err(Error::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("文件不存在: {}", file_path),
            )));
        }

        // 读取 PDF 文件
        let data = std::fs::read(path).map_err(|e| Error::Io(e))?;

        // 使用 pdf-extract 提取文本
        let text = pdf_extract::extract_text_from_mem(&data)
            .map_err(|e| Error::Storage(format!("PDF 解析失败: {}", e)))?;

        // 计算页数（通过换页符估算）
        let page_count = text.matches("\x0C").count() + 1;

        Ok(PdfDocument {
            file_path: file_path.to_string(),
            text,
            page_count,
        })
    }

    /// 从文件名提取标题
    pub fn extract_title_from_filename(file_path: &str) -> Option<String> {
        let path = Path::new(file_path);
        path.file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
    }
}

/// 文献切片器
pub struct PaperChunker {
    /// 最大块大小（字符数）
    max_chunk_size: usize,
    /// 块之间的重叠大小
    overlap_size: usize,
}

impl PaperChunker {
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
    pub fn chunk_by_paragraph(&self, text: &str) -> Vec<String> {
        let mut chunks = Vec::new();
        let mut current_chunk = String::new();
        let mut current_size = 0;

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
                    chunks.push(current_chunk.clone());
                    current_chunk = String::new();
                    current_size = 0;
                }

                // 分割大段落
                let sub_chunks = self.split_large_paragraph(paragraph);
                chunks.extend(sub_chunks);
                continue;
            }

            // 检查是否需要开始新块
            if current_size + para_size > self.max_chunk_size && !current_chunk.is_empty() {
                chunks.push(current_chunk.clone());
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
            chunks.push(current_chunk);
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_title_from_filename() {
        let title = PdfParser::extract_title_from_filename("/path/to/My Research Paper.pdf");
        assert_eq!(title, Some("My Research Paper".to_string()));
    }

    #[test]
    fn test_chunk_by_paragraph() {
        let chunker = PaperChunker::default();
        let text = "First paragraph.\n\nSecond paragraph.\n\nThird paragraph.";
        let chunks = chunker.chunk_by_paragraph(text);

        assert!(!chunks.is_empty());
        assert!(chunks.len() <= 3);
    }
}
