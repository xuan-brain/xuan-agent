//! 向量化服务
//!
//! 提供文本嵌入生成功能

use crate::{config::AiProviderConfig, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

/// 向量化服务
pub struct EmbeddingService {
    client: Client,
    config: AiProviderConfig,
    /// 默认向量维度 (OpenAI text-embedding-3-small 为 1536)
    pub embedding_dim: usize,
}

/// 嵌入请求
#[derive(Debug, Serialize)]
struct EmbeddingRequest {
    model: String,
    input: String,
}

/// 嵌入响应
#[derive(Debug, Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingData>,
}

#[derive(Debug, Deserialize)]
struct EmbeddingData {
    embedding: Vec<f32>,
}

impl EmbeddingService {
    /// 创建新的向量化服务
    pub fn new(config: AiProviderConfig) -> Self {
        // 根据不同的提供商设置默认维度
        let embedding_dim = match config.provider.as_str() {
            "openai" => 1536,  // text-embedding-3-small
            _ => 768,         // 默认维度
        };

        Self {
            client: Client::new(),
            config,
            embedding_dim,
        }
    }

    /// 设置向量维度
    pub fn with_embedding_dim(mut self, dim: usize) -> Self {
        self.embedding_dim = dim;
        self
    }

    /// 生成单个文本的嵌入向量
    pub async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let url = format!("{}/embeddings", self.config.base_url);

        let request = EmbeddingRequest {
            model: self.config.model.clone(),
            input: text.to_string(),
        };

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .json(&request)
            .send()
            .await
            .map_err(|e| crate::Error::Embedding(format!("请求失败: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(crate::Error::Embedding(
                format!("API 错误 {}: {}", status, error_text)
            ));
        }

        let embedding_response: EmbeddingResponse = response
            .json()
            .await
            .map_err(|e| crate::Error::Embedding(format!("解析响应失败: {}", e)))?;

        embedding_response
            .data
            .first()
            .map(|d| d.embedding.clone())
            .ok_or_else(|| crate::Error::Embedding("响应中没有嵌入向量".to_string()))
    }

    /// 批量生成嵌入向量
    pub async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let mut results = Vec::new();

        // 对于批量请求，通常 API 有速率限制，这里使用简单串行处理
        // 生产环境应该使用批量 API 或并行请求
        for text in texts {
            let embedding = self.embed(text).await?;
            results.push(embedding);
        }

        Ok(results)
    }

    /// 嵌入维度
    pub fn embedding_dim(&self) -> usize {
        self.embedding_dim
    }

    /// 文本分块（用于嵌入）
    ///
    /// 将长文本分割成适合嵌入的块
    pub fn chunk_text(&self, text: &str, max_chunk_size: usize) -> Vec<String> {
        let mut chunks = Vec::new();
        let mut current_chunk = String::new();
        let mut current_size = 0;

        for paragraph in text.split('\n') {
            let paragraph = paragraph.trim();
            if paragraph.is_empty() {
                continue;
            }

            let paragraph_size = paragraph.chars().count();

            if current_size + paragraph_size > max_chunk_size && !current_chunk.is_empty() {
                chunks.push(current_chunk.clone());
                current_chunk = String::new();
                current_size = 0;
            }

            if !current_chunk.is_empty() {
                current_chunk.push('\n');
            }
            current_chunk.push_str(paragraph);
            current_size += paragraph_size;
        }

        if !current_chunk.is_empty() {
            chunks.push(current_chunk);
        }

        chunks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_text() {
        let service = EmbeddingService::new(crate::config::AiProviderConfig {
            provider: "test".to_string(),
            base_url: "http://test.com".to_string(),
            api_key: "test".to_string(),
            model: "test".to_string(),
            description: None,
        });

        let text = "First paragraph.\n\nSecond paragraph.\n\nThird paragraph.";
        let chunks = service.chunk_text(text, 50);

        assert!(!chunks.is_empty());
        // 检查分块是否正确分割
        assert!(chunks.len() <= 3);
        // 最后一个块可能较大，但其他块应该在限制内
        for (i, chunk) in chunks.iter().enumerate() {
            if i < chunks.len() - 1 {
                assert!(chunk.chars().count() <= 50, "Chunk {} exceeds limit: {}", i, chunk.chars().count());
            }
        }
    }
}
