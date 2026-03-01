//! 自动标签系统
//!
//! 使用 LLM 为文献生成标签和分类

use crate::{Config, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 标签类别
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TagCategory {
    /// 研究领域
    #[serde(rename = "field")]
    Field,
    /// 方法论
    #[serde(rename = "method")]
    Method,
    /// 应用领域
    #[serde(rename = "application")]
    Application,
    /// 数据类型
    #[serde(rename = "dataType")]
    DataType,
    /// 关键词
    #[serde(rename = "keyword")]
    Keyword,
    /// 其他
    #[serde(rename = "other")]
    Other,
}

/// 标签
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    /// 标签名称
    pub name: String,
    /// 置信度 (0-1)
    pub confidence: f64,
    /// 标签类别
    pub category: TagCategory,
}

/// 标签生成请求
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TagGenerationRequest {
    title: String,
    abstract_text: String,
    authors: Vec<String>,
}

/// 标签生成响应
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TagGenerationResponse {
    tags: Vec<Tag>,
}

/// 自动标签服务
pub struct TaggingService {
    config: Config,
    /// 预定义的领域标签
    field_tags: Vec<String>,
    /// 预定义的方法标签
    method_tags: Vec<String>,
}

impl TaggingService {
    /// 创建新的标签服务
    pub fn new(config: Config) -> Self {
        let field_tags = vec![
            "计算机科学".to_string(),
            "人工智能".to_string(),
            "机器学习".to_string(),
            "深度学习".to_string(),
            "自然语言处理".to_string(),
            "计算机视觉".to_string(),
            "数据科学".to_string(),
            "生物信息学".to_string(),
            "物理学".to_string(),
            "数学".to_string(),
        ];

        let method_tags = vec![
            "神经网络".to_string(),
            "Transformer".to_string(),
            "强化学习".to_string(),
            "监督学习".to_string(),
            "无监督学习".to_string(),
            "实验研究".to_string(),
            "理论研究".to_string(),
            "统计分析".to_string(),
            "数值模拟".to_string(),
        ];

        Self {
            config,
            field_tags,
            method_tags,
        }
    }

    /// 为文献生成标签
    pub async fn generate_tags(&self, paper: &crate::storage::Paper) -> Result<Vec<Tag>> {
        let mut tags = Vec::new();

        // 基于标题和摘要生成关键词标签
        let text = format!("{} {}", paper.title, paper.abstract_text);
        let keywords = self.extract_keywords(&text);
        for keyword in keywords {
            tags.push(Tag {
                name: keyword,
                confidence: 0.8,
                category: TagCategory::Keyword,
            });
        }

        // 检测领域标签
        let field = self.detect_field(&text);
        if let Some(field_name) = field {
            tags.push(Tag {
                name: field_name,
                confidence: 0.9,
                category: TagCategory::Field,
            });
        }

        // 检测方法标签
        let methods = self.detect_methods(&text);
        for method_name in methods {
            tags.push(Tag {
                name: method_name,
                confidence: 0.85,
                category: TagCategory::Method,
            });
        }

        // 使用 LLM 生成额外标签（可选，需要调用 LLM API）
        // let llm_tags = self.generate_llm_tags(paper).await?;
        // tags.extend(llm_tags);

        // 按置信度排序
        tags.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());

        Ok(tags)
    }

    /// 提取关键词
    fn extract_keywords(&self, text: &str) -> Vec<String> {
        let mut keywords = Vec::new();
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut word_count: HashMap<String, usize> = HashMap::new();

        // 统计词频
        for word in words {
            let word_lower = word.to_lowercase();
            // 过滤掉常见停用词和短词
            if word_lower.len() > 3 && !is_stop_word(&word_lower) {
                *word_count.entry(word_lower).or_insert(0) += 1;
            }
        }

        // 取出现频率最高的词
        let mut count_vec: Vec<_> = word_count.iter().collect();
        count_vec.sort_by(|a, b| b.1.cmp(a.1));

        for (word, _count) in count_vec.iter().take(5) {
            keywords.push(word.to_string());
        }

        keywords
    }

    /// 检测研究领域
    fn detect_field(&self, text: &str) -> Option<String> {
        let text_lower = text.to_lowercase();

        for field in &self.field_tags {
            if text_lower.contains(&field.to_lowercase()) {
                return Some(field.clone());
            }
        }

        None
    }

    /// 检测方法
    fn detect_methods(&self, text: &str) -> Vec<String> {
        let mut methods = Vec::new();
        let text_lower = text.to_lowercase();

        for method in &self.method_tags {
            if text_lower.contains(&method.to_lowercase()) {
                methods.push(method.clone());
            }
        }

        methods
    }

    /// 使用 LLM 生成标签（高级功能）
    #[allow(dead_code)]
    async fn generate_llm_tags(&self, paper: &crate::storage::Paper) -> Result<Vec<Tag>> {
        use crate::llm::LlmService;

        let prompt = format!(
            r#"请为以下文献生成 3-5 个标签。

标题: {}
摘要: {}
作者: {}

请以 JSON 格式返回，格式如下：
{{
  "tags": [
    {{"name": "标签名", "confidence": 0.95, "category": "field"}},
    {{"name": "标签名", "confidence": 0.90, "category": "method"}}
  ]
}}

类别说明：
- field: 研究领域
- method: 研究方法
- application: 应用领域
- keyword: 关键词
"#,
            paper.title,
            paper.abstract_text,
            paper.authors.join(", ")
        );

        let llm_service = LlmService::new(self.config.ai_provider.clone());
        let response = llm_service.chat(&prompt).await?;

        // 解析响应
        let tag_response: TagGenerationResponse = serde_json::from_str(&response)?;

        Ok(tag_response.tags)
    }
}

/// 判断是否为停用词
fn is_stop_word(word: &str) -> bool {
    let stop_words = vec![
        "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for",
        "of", "with", "by", "from", "as", "is", "was", "are", "were", "be",
        "been", "being", "have", "has", "had", "do", "does", "did", "will",
        "would", "should", "could", "may", "might", "must", "can", "this",
        "that", "these", "those", "的", "了", "在", "是", "和", "与", "或",
        "但是", "然后", "因此", "因为", "所以", "如果", "虽然", "通过",
    ];

    stop_words.contains(&word)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_extract_keywords() {
        let service = TaggingService::new(Config {
            ai_provider: crate::config::AiProviderConfig {
                provider: "test".to_string(),
                base_url: "http://test".to_string(),
                api_key: "test".to_string(),
                model: "test".to_string(),
                description: None,
            },
            db: crate::config::DbConfig {
                connect: "ws://localhost:8000".to_string(),
                user: "root".to_string(),
                pass: "secret".to_string(),
                namespace: "test".to_string(),
                database: "test".to_string(),
            },
            system_prompt: None,
        });

        let text = "machine learning artificial intelligence neural networks deep learning";
        let keywords = service.extract_keywords(text);

        assert!(!keywords.is_empty());
    }

    #[test]
    fn test_detect_field() {
        let service = TaggingService::new(Config {
            ai_provider: crate::config::AiProviderConfig {
                provider: "test".to_string(),
                base_url: "http://test".to_string(),
                api_key: "test".to_string(),
                model: "test".to_string(),
                description: None,
            },
            db: crate::config::DbConfig {
                connect: "ws://localhost:8000".to_string(),
                user: "root".to_string(),
                pass: "secret".to_string(),
                namespace: "test".to_string(),
                database: "test".to_string(),
            },
            system_prompt: None,
        });

        let text = "This paper is about deep learning and neural networks";
        let field = service.detect_field(text);

        assert!(field.is_some());
    }

    #[test]
    fn test_is_stop_word() {
        assert!(is_stop_word("the"));
        assert!(is_stop_word("的"));
        assert!(!is_stop_word("neural"));
        assert!(!is_stop_word("network"));
    }
}
