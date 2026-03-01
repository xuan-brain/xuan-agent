//! SurrealDB 存储实现
//!
//! 提供文献和向量的持久化存储

use super::models::{Chunk, CountResult, Paper, PaperInfo, SearchResult};
use crate::{config::DbConfig, Error, Result};
use serde_json::json;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;

/// SurrealDB 存储客户端
pub struct SurrealDBStorage {
    client: Surreal<Client>,
    namespace: String,
    database: String,
}

impl SurrealDBStorage {
    /// 连接到 SurrealDB
    pub async fn connect(config: &DbConfig) -> Result<Self> {
        // 解析地址，移除 ws://、wss://、http://、https:// 和 /rpc 后缀
        let addr = config
            .connect
            .replace("ws://", "")
            .replace("wss://", "")
            .replace("http://", "")
            .replace("https://", "")
            .replace("/rpc", "")
            .trim()
            .to_string();

        // 连接到 WebSocket
        let client = Surreal::new::<Ws>(&addr)
            .await
            .map_err(|e| Error::Storage(format!("连接失败: {}", e)))?;

        // 签入 - 使用 owned String
        client
            .signin(Root {
                username: config.user.clone(),
                password: config.pass.clone(),
            })
            .await
            .map_err(|e| Error::Storage(format!("认证失败: {}", e)))?;

        // 选择命名空间和数据库
        client
            .use_ns(&config.namespace)
            .use_db(&config.database)
            .await
            .map_err(|e| Error::Storage(format!("选择数据库失败: {}", e)))?;

        Ok(Self {
            client,
            namespace: config.namespace.clone(),
            database: config.database.clone(),
        })
    }

    /// 创建表和索引
    pub async fn init_schema(&self) -> Result<()> {
        // 创建 papers 表
        self.client
            .query("DEFINE TABLE paper SCHEMALESS")
            .await
            .map_err(|e| Error::Storage(format!("创建 paper 表失败: {}", e)))?;

        // 创建 chunks 表
        self.client
            .query("DEFINE TABLE chunk SCHEMALESS")
            .await
            .map_err(|e| Error::Storage(format!("创建 chunk 表失败: {}", e)))?;

        // 创建索引
        self.client
            .query("DEFINE INDEX title_idx ON paper FIELDS title")
            .await
            .map_err(|e| Error::Storage(format!("创建索引失败: {}", e)))?;

        self.client
            .query("DEFINE INDEX paper_id_idx ON chunk FIELDS paper_id")
            .await
            .map_err(|e| Error::Storage(format!("创建索引失败: {}", e)))?;

        Ok(())
    }

    /// 存储文献
    pub async fn store_paper(&self, paper: Paper) -> Result<String> {
        // 将 datetime 转换为字符串
        let created_at_str = paper.created_at.to_rfc3339();

        // 使用 SQL INSERT，每次都绑定一个值
        let sql = "CREATE paper CONTENT {
            title: $title,
            abstract_text: $abstract_text,
            authors: $authors,
            year: $year,
            journal: $journal,
            doi: $doi,
            file_path: $file_path,
            tags: $tags,
            created_at: $created_at
        }";

        // 不解析返回结果，直接执行
        self.client
            .query(sql)
            .bind(("title", paper.title))
            .bind(("abstract_text", paper.abstract_text))
            .bind(("authors", paper.authors))
            .bind(("year", paper.year))
            .bind(("journal", paper.journal))
            .bind(("doi", paper.doi))
            .bind(("file_path", paper.file_path))
            .bind(("tags", paper.tags))
            .bind(("created_at", created_at_str))
            .await
            .map_err(|e| Error::Storage(format!("存储文献失败: {}", e)))?;

        // 返回原始的 id
        Ok(paper.id)
    }

    /// 批量存储分块
    pub async fn store_chunks(&self, chunks: Vec<Chunk>) -> Result<Vec<String>> {
        for chunk in &chunks {
            let full_paper_id = chunk.paper_id.clone();
            let paper_uuid_part = full_paper_id.split(':').last().unwrap_or(&full_paper_id).to_string();

            // 将 ChunkType 转为字符串表示
            let chunk_type_str = format!("{:?}", chunk.chunk_type).to_lowercase();

            // 使用 SQL INSERT
            let sql = "CREATE chunk CONTENT {
                paper_id: $paper_id,
                content: $content,
                chunk_index: $chunk_index,
                embedding: $embedding,
                page_number: $page_number,
                chunk_type: $chunk_type
            }";

            self.client
                .query(sql)
                .bind(("paper_id", paper_uuid_part))
                .bind(("content", chunk.content.clone()))
                .bind(("chunk_index", chunk.chunk_index))
                .bind(("embedding", chunk.embedding.clone()))
                .bind(("page_number", chunk.page_number))
                .bind(("chunk_type", chunk_type_str))
                .await
                .map_err(|e| Error::Storage(format!("存储分块失败: {}", e)))?;
        }

        // 返回所有原始 id
        Ok(chunks.into_iter().map(|c| c.id).collect())
    }

    /// 根据 ID 获取文献
    pub async fn get_paper(&self, id: &str) -> Result<Option<Paper>> {
        let uuid = id.split(':').last().unwrap_or(id);
        let result: Option<serde_json::Value> = self
            .client
            .select(("paper", uuid))
            .await
            .map_err(|e| Error::Storage(format!("获取文献失败: {}", e)))?;

        match result {
            Some(v) => {
                let mut paper: Paper = serde_json::from_value(v)
                    .map_err(|e| Error::Storage(format!("反序列化文献失败: {}", e)))?;

                // 重建完整的 id
                if !paper.id.contains(':') {
                    paper.id = format!("paper:{}", paper.id);
                }
                Ok(Some(paper))
            }
            None => Ok(None),
        }
    }

    /// 根据关键词搜索文献
    pub async fn search_by_keyword(&self, keyword: &str, limit: u32) -> Result<Vec<PaperInfo>> {
        let query = r#"
            SELECT id, title, authors, year, journal, created_at
            FROM paper
            WHERE title CONTAINS $keyword OR abstract_text CONTAINS $keyword
            LIMIT $limit
        "#;

        let mut response = self
            .client
            .query(query)
            .bind(("keyword", keyword.to_string()))
            .bind(("limit", limit))
            .await
            .map_err(|e| Error::Storage(format!("搜索失败: {}", e)))?;

        let result: Vec<serde_json::Value> = response
            .take(0)
            .map_err(|e| Error::Storage(format!("解析结果失败: {}", e)))?;

        let papers: Vec<PaperInfo> = result
            .into_iter()
            .filter_map(|v| serde_json::from_value(v).ok())
            .collect();

        Ok(papers)
    }

    /// 按标签搜索文献
    pub async fn search_by_tag(&self, tag: &str, limit: u32) -> Result<Vec<PaperInfo>> {
        let query = format!(
            "SELECT id, title, authors, year, journal, created_at FROM paper WHERE $tag IN tags LIMIT {}",
            limit
        );

        let mut response = self
            .client
            .query(&query)
            .bind(("tag", tag.to_string()))
            .await
            .map_err(|e| Error::Storage(format!("按标签搜索失败: {}", e)))?;

        let result: Vec<serde_json::Value> = response
            .take(0)
            .map_err(|e| Error::Storage(format!("解析结果失败: {}", e)))?;

        let papers: Vec<PaperInfo> = result
            .into_iter()
            .filter_map(|v| serde_json::from_value(v).ok())
            .collect();

        Ok(papers)
    }

    /// 获取文献的所有分块
    pub async fn get_chunks(&self, paper_id: &str) -> Result<Vec<Chunk>> {
        // 提取 UUID 部分
        let paper_uuid = paper_id.split(':').last().unwrap_or(paper_id).to_string();

        let mut response = self
            .client
            .query("SELECT * FROM chunk WHERE paper_id = $paper_id ORDER BY chunk_index")
            .bind(("paper_id", paper_uuid))
            .await
            .map_err(|e| Error::Storage(format!("获取分块失败: {}", e)))?;

        let result: Vec<serde_json::Value> = response
            .take(0)
            .map_err(|e| Error::Storage(format!("解析结果失败: {}", e)))?;

        let mut chunks = Vec::new();
        for v in result {
            if let Ok(mut chunk) = serde_json::from_value::<Chunk>(v) {
                // 重建完整的 paper_id
                if !chunk.paper_id.contains(':') {
                    chunk.paper_id = format!("paper:{}", chunk.paper_id);
                }
                chunks.push(chunk);
            }
        }

        Ok(chunks)
    }

    /// 向量相似度搜索
    pub async fn search_similar(
        &self,
        embedding: &[f32],
        limit: u32,
        threshold: f32,
    ) -> Result<Vec<SearchResult>> {
        // 获取所有分块后计算余弦相似度
        // 生产环境应使用 SurrealDB 的向量扩展或专门的向量数据库

        let mut response = self
            .client
            .query("SELECT * FROM chunk WHERE embedding IS NOT NULL LIMIT 100")
            .await
            .map_err(|e| Error::Storage(format!("获取分块失败: {}", e)))?;

        let chunks_result: Vec<serde_json::Value> = response
            .take(0)
            .map_err(|e| Error::Storage(format!("解析结果失败: {}", e)))?;

        let mut chunks = Vec::new();
        for v in chunks_result {
            if let Ok(mut chunk) = serde_json::from_value::<Chunk>(v) {
                // 重建完整的 paper_id
                if !chunk.paper_id.contains(':') {
                    chunk.paper_id = format!("paper:{}", chunk.paper_id);
                }
                chunks.push(chunk);
            }
        }

        let mut results = Vec::new();

        for chunk in chunks {
            if let Some(chunk_embedding) = &chunk.embedding {
                let similarity = cosine_similarity(embedding, chunk_embedding);

                if similarity >= threshold {
                    // 获取关联的文献信息
                    if let Ok(Some(paper)) = self.get_paper(&chunk.paper_id).await {
                        results.push(SearchResult {
                            paper: PaperInfo::from(paper),
                            chunk: chunk.into(),
                            similarity,
                        });
                    }
                }
            }
        }

        // 按相似度排序
        results.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap());

        // 限制结果数量
        results.truncate(limit as usize);

        Ok(results)
    }

    /// 删除文献及其所有分块
    pub async fn delete_paper(&self, id: &str) -> Result<()> {
        // 提取 UUID 部分
        let uuid = id.split(':').last().unwrap_or(id).to_string();

        // 删除关联的分块
        self.client
            .query("DELETE FROM chunk WHERE paper_id = $id")
            .bind(("id", uuid))
            .await
            .map_err(|e| Error::Storage(format!("删除分块失败: {}", e)))?;

        // 删除文献
        let paper_uuid = id.split(':').last().unwrap_or(id);
        let _: Option<serde_json::Value> = self
            .client
            .delete(("paper", paper_uuid))
            .await
            .map_err(|e| Error::Storage(format!("删除文献失败: {}", e)))?;

        Ok(())
    }

    /// 列出所有文献
    pub async fn list_papers(&self, limit: u32) -> Result<Vec<PaperInfo>> {
        // 简化版本：只返回标题和基本信息的硬编码结果用于测试
        // TODO: 需要处理 SurrealDB 3.x 的 Thing 类型序列化

        // 使用原始 SQL 查询并手动处理
        let sql = "SELECT title FROM paper LIMIT $limit";
        let mut response = self
            .client
            .query(sql)
            .bind(("limit", limit))
            .await
            .map_err(|e| Error::Storage(format!("列出文献失败: {}", e)))?;

        // 获取结果
        let result: Vec<serde_json::Value> = response
            .take(0)
            .map_err(|e| Error::Storage(format!("解析结果失败: {}", e)))?;

        let mut papers = Vec::new();
        let mut counter = 0;
        for v in result {
            if let Some(title) = v.get("title").and_then(|t| t.as_str()) {
                papers.push(PaperInfo {
                    id: format!("paper:{}", counter),
                    title: title.to_string(),
                    authors: vec!["Unknown".to_string()],
                    year: None,
                    journal: None,
                    created_at: None,
                });
                counter += 1;
            }
        }

        Ok(papers)
    }

    /// 更新文献标签
    pub async fn update_tags(&self, id: &str, tags: Vec<String>) -> Result<()> {
        let uuid = id.split(':').last().unwrap_or(id);
        let _: Option<serde_json::Value> = self
            .client
            .update(("paper", uuid))
            .merge(json!({ "tags": tags }))
            .await
            .map_err(|e| Error::Storage(format!("更新标签失败: {}", e)))?;

        Ok(())
    }

    /// 获取数据库统计信息
    pub async fn stats(&self) -> Result<(u64, u64)> {
        let mut paper_response = self
            .client
            .query("SELECT count() AS count FROM paper")
            .await
            .map_err(|e| Error::Storage(format!("获取文献统计失败: {}", e)))?;

        let mut chunk_response = self
            .client
            .query("SELECT count() AS count FROM chunk")
            .await
            .map_err(|e| Error::Storage(format!("获取分块统计失败: {}", e)))?;

        let paper_results: Vec<serde_json::Value> = paper_response
            .take(0)
            .map_err(|e| Error::Storage(format!("解析文献统计失败: {}", e)))?;

        let chunk_results: Vec<serde_json::Value> = chunk_response
            .take(0)
            .map_err(|e| Error::Storage(format!("解析分块统计失败: {}", e)))?;

        let paper_count = paper_results.first()
            .and_then(|v| v.get("count"))
            .and_then(|c| c.as_u64())
            .unwrap_or(0);

        let chunk_count = chunk_results.first()
            .and_then(|v| v.get("count"))
            .and_then(|c| c.as_u64())
            .unwrap_or(0);

        Ok((paper_count, chunk_count))
    }
}

/// 计算余弦相似度
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot_product / (norm_a * norm_b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![1.0, 2.0, 3.0];

        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 0.001);

        let c = vec![-1.0, -2.0, -3.0];
        assert!((cosine_similarity(&a, &c) - (-1.0)).abs() < 0.001);

        let d = vec![0.0, 1.0, 0.0];
        let expected = 2.0 / (14.0_f32.sqrt());  // dot=2, |a|=sqrt(14), |d|=1
        assert!((cosine_similarity(&a, &d) - expected).abs() < 0.001);
    }
}
