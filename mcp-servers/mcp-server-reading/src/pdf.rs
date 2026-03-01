//! PDF 解析模块

use crate::chunk::Chunker;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// PDF 解析错误
#[derive(Error, Debug)]
pub enum PdfError {
    #[error("文件不存在: {0}")]
    FileNotFound(String),

    #[error("PDF 解析失败: {0}")]
    ParseError(String),

    #[error("IO 错误: {0}")]
    IoError(#[from] std::io::Error),
}

/// PDF 文档信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfDocument {
    /// 文件路径
    pub file_path: String,
    /// 提取的文本内容
    pub text: String,
    /// 页数
    pub page_count: usize,
    /// 文档结构（章节、段落）
    pub structure: DocumentStructure,
}

/// 文档结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentStructure {
    /// 章节列表
    pub sections: Vec<Section>,
    /// 元数据
    pub metadata: DocumentMetadata,
}

/// 章节
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section {
    /// 章节标题
    pub title: String,
    /// 章节类型
    pub section_type: SectionType,
    /// 章节内容
    pub content: String,
    /// 起始页码
    pub start_page: Option<usize>,
}

/// 章节类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SectionType {
    /// 标题
    Title,
    /// 摘要
    Abstract,
    /// 引言
    Introduction,
    /// 相关工作
    RelatedWork,
    /// 方法
    Methods,
    /// 实验
    Experiments,
    /// 结果
    Results,
    /// 讨论
    Discussion,
    /// 结论
    Conclusion,
    /// 参考文献
    References,
    /// 附录
    Appendix,
    /// 其他
    Other,
}

impl std::fmt::Display for SectionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SectionType::Title => write!(f, "Title"),
            SectionType::Abstract => write!(f, "Abstract"),
            SectionType::Introduction => write!(f, "Introduction"),
            SectionType::RelatedWork => write!(f, "Related Work"),
            SectionType::Methods => write!(f, "Methods"),
            SectionType::Experiments => write!(f, "Experiments"),
            SectionType::Results => write!(f, "Results"),
            SectionType::Discussion => write!(f, "Discussion"),
            SectionType::Conclusion => write!(f, "Conclusion"),
            SectionType::References => write!(f, "References"),
            SectionType::Appendix => write!(f, "Appendix"),
            SectionType::Other => write!(f, "Other"),
        }
    }
}

/// 文档元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    /// 标题
    pub title: Option<String>,
    /// 作者
    pub authors: Vec<String>,
    /// 摘要
    pub abstract_text: Option<String>,
    /// 关键词
    pub keywords: Vec<String>,
}

/// PDF 解析器
pub struct PdfParser;

impl PdfParser {
    /// 解析 PDF 文件
    pub fn parse(file_path: &str) -> Result<PdfDocument, PdfError> {
        let path = std::path::Path::new(file_path);
        if !path.exists() {
            return Err(PdfError::FileNotFound(file_path.to_string()));
        }

        // 读取 PDF 文件
        let data = std::fs::read(path)?;

        // 使用 pdf-extract 提取文本
        let text = pdf_extract::extract_text_from_mem(&data)
            .map_err(|e| PdfError::ParseError(e.to_string()))?;

        // 计算页数（通过换页符估算）
        let page_count = text.matches("\x0C").count() + 1;

        // 提取文档结构
        let structure = Self::extract_structure(&text);

        Ok(PdfDocument {
            file_path: file_path.to_string(),
            text,
            page_count,
            structure,
        })
    }

    /// 从文件名提取标题
    pub fn extract_title_from_filename(file_path: &str) -> Option<String> {
        let path = std::path::Path::new(file_path);
        path.file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
    }

    /// 提取文档结构
    fn extract_structure(text: &str) -> DocumentStructure {
        let mut sections = Vec::new();
        let mut metadata = DocumentMetadata {
            title: None,
            authors: Vec::new(),
            abstract_text: None,
            keywords: Vec::new(),
        };

        // 简单的章节识别逻辑
        let lines: Vec<&str> = text.lines().collect();
        let mut current_section = String::new();
        let mut current_section_type = SectionType::Other;
        let mut in_abstract = false;

        for (i, line) in lines.iter().enumerate() {
            let line = line.trim();

            // 检测标题（大写、编号等）
            if Self::is_section_header(line) {
                // 保存上一个章节
                if !current_section.is_empty() {
                    sections.push(Section {
                        title: current_section_type.clone().to_string(),
                        section_type: current_section_type.clone(),
                        content: current_section.clone(),
                        start_page: None,
                    });
                }

                // 识别新章节类型
                current_section_type = Self::identify_section_type(line);
                current_section = String::new();
            } else {
                // 检测摘要
                if line.to_lowercase().starts_with("abstract") {
                    in_abstract = true;
                    current_section_type = SectionType::Abstract;
                    continue;
                }

                if in_abstract && line.is_empty() {
                    in_abstract = false;
                    if !current_section.is_empty() {
                        metadata.abstract_text = Some(current_section.clone());
                        current_section.clear();
                    }
                    continue;
                }

                if !current_section.is_empty() {
                    current_section.push('\n');
                }
                current_section.push_str(line);
            }
        }

        // 添加最后一个章节
        if !current_section.is_empty() {
            sections.push(Section {
                title: current_section_type.to_string(),
                section_type: current_section_type,
                content: current_section,
                start_page: None,
            });
        }

        DocumentStructure {
            sections,
            metadata,
        }
    }

    /// 判断是否为章节标题
    fn is_section_header(line: &str) -> bool {
        // 简单的启发式规则
        let patterns = [
            "Introduction",
            "Abstract",
            "Conclusion",
            "References",
            "Related Work",
            "Method",
            "Experiment",
            "Result",
            "Discussion",
            "引言",
            "摘要",
            "结论",
            "参考文献",
            "方法",
            "实验",
            "结果",
            "讨论",
        ];

        let line_upper = line.to_uppercase();

        // 检查是否匹配已知模式
        for pattern in &patterns {
            if line_upper.contains(&pattern.to_uppercase()) {
                return true;
            }
        }

        // 检查编号格式（如 "1. Introduction"）
        if line.len() < 100 && line.chars().next().map_or(false, |c| c.is_ascii_digit()) {
            return true;
        }

        false
    }

    /// 识别章节类型
    fn identify_section_type(line: &str) -> SectionType {
        let line_lower = line.to_lowercase();

        if line_lower.contains("abstract") || line_lower.contains("摘要") {
            SectionType::Abstract
        } else if line_lower.contains("introduction") || line_lower.contains("引言") {
            SectionType::Introduction
        } else if line_lower.contains("related work") || line_lower.contains("相关工作") {
            SectionType::RelatedWork
        } else if line_lower.contains("method") || line_lower.contains("方法") {
            SectionType::Methods
        } else if line_lower.contains("experiment") || line_lower.contains("实验") {
            SectionType::Experiments
        } else if line_lower.contains("result") || line_lower.contains("结果") {
            SectionType::Results
        } else if line_lower.contains("discussion") || line_lower.contains("讨论") {
            SectionType::Discussion
        } else if line_lower.contains("conclusion") || line_lower.contains("结论") {
            SectionType::Conclusion
        } else if line_lower.contains("reference") || line_lower.contains("参考文献") {
            SectionType::References
        } else if line_lower.contains("appendix") || line_lower.contains("附录") {
            SectionType::Appendix
        } else if line_lower.contains("title") && line.len() < 100 {
            SectionType::Title
        } else {
            SectionType::Other
        }
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
    fn test_identify_section_type() {
        assert!(matches!(
            PdfParser::identify_section_type("1. Introduction"),
            SectionType::Introduction
        ));
        assert!(matches!(
            PdfParser::identify_section_type("Abstract"),
            SectionType::Abstract
        ));
        assert!(matches!(
            PdfParser::identify_section_type("3. Methods"),
            SectionType::Methods
        ));
    }
}
