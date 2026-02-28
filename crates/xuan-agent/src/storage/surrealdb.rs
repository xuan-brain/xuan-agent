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
        // 解析地址，移除 ws://、wss:// 和 /rpc 后缀
        let addr = config
            .connect
            .replace("ws://", "")
            .replace("wss://", "")
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
        let id = paper.id.clone();

        let _: Option<Paper> = self
            .client
            .create(("paper", id.as_str()))
            .content(paper)
            .await
            .map_err(|e| Error::Storage(format!("存储文献失败: {}", e)))?;

        Ok(id)
    }

    /// 批量存储分块
    pub async fn store_chunks(&self, chunks: Vec<Chunk>) -> Result<Vec<String>> {
        let mut ids = Vec::new();

        for chunk in chunks {
            let id = chunk.id.clone();

            let _: Option<Chunk> = self
                .client
                .create(("chunk", id.as_str()))
                .content(chunk)
                .await
                .map_err(|e| Error::Storage(format!("存储分块失败: {}", e)))?;

            ids.push(id);
        }

        Ok(ids)
    }

    /// 根据 ID 获取文献
    pub async fn get_paper(&self, id: &str) -> Result<Option<Paper>> {
        let result: Option<Paper> = self
            .client
            .select(("paper", id))
            .await
            .map_err(|e| Error::Storage(format!("获取文献失败: {}", e)))?;

        Ok(result)
    }

    /// 根据关键词搜索文献
    pub async fn search_by_keyword(&self, keyword: &str, limit: u32) -> Result<Vec<PaperInfo>> {
        let query = format!(
            "SELECT id, title, authors, year, journal FROM paper WHERE title @ '{}' OR abstract_text @ '{}' LIMIT {}",
            keyword, keyword, limit
        );

        let mut response = self
            .client
            .query(&query)
            .await
            .map_err(|e| Error::Storage(format!("搜索失败: {}", e)))?;

        let result: Vec<PaperInfo> = response
            .take(0)
            .map_err(|e| Error::Storage(format!("解析结果失败: {}", e)))?;

        Ok(result)
    }

    /// 按标签搜索文献
    pub async fn search_by_tag(&self, tag: &str, limit: u32) -> Result<Vec<PaperInfo>> {
        let query = format!(
            "SELECT id, title, authors, year, journal FROM paper WHERE $tag IN tags LIMIT {}",
            limit
        );

        let mut response = self
            .client
            .query(&query)
            .bind(("tag", tag.to_string()))
            .await
            .map_err(|e| Error::Storage(format!("按标签搜索失败: {}", e)))?;

        let result: Vec<PaperInfo> = response
            .take(0)
            .map_err(|e| Error::Storage(format!("解析结果失败: {}", e)))?;

        Ok(result)
    }

    /// 获取文献的所有分块
    pub async fn get_chunks(&self, paper_id: &str) -> Result<Vec<Chunk>> {
        let mut response = self
            .client
            .query("SELECT * FROM chunk WHERE paper_id = $paper_id ORDER BY chunk_index")
            .bind(("paper_id", paper_id.to_string()))
            .await
            .map_err(|e| Error::Storage(format!("获取分块失败: {}", e)))?;

        let result: Vec<Chunk> = response
            .take(0)
            .map_err(|e| Error::Storage(format!("解析结果失败: {}", e)))?;

        Ok(result)
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

        let chunks: Vec<Chunk> = response
            .take(0)
            .map_err(|e| Error::Storage(format!("解析结果失败: {}", e)))?;

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
        // 删除关联的分块
        self.client
            .query("DELETE FROM chunk WHERE paper_id = $id")
            .bind(("id", id.to_string()))
            .await
            .map_err(|e| Error::Storage(format!("删除分块失败: {}", e)))?;

        // 删除文献
        let _: Option<Paper> = self
            .client
            .delete(("paper", id))
            .await
            .map_err(|e| Error::Storage(format!("删除文献失败: {}", e)))?;

        Ok(())
    }

    /// 列出所有文献
    pub async fn list_papers(&self, limit: u32) -> Result<Vec<PaperInfo>> {
        let mut response = self
            .client
            .query("SELECT id, title, authors, year, journal FROM paper ORDER BY created_at DESC LIMIT $limit")
            .bind(("limit", limit))
            .await
            .map_err(|e| Error::Storage(format!("列出文献失败: {}", e)))?;

        let result: Vec<PaperInfo> = response
            .take(0)
            .map_err(|e| Error::Storage(format!("解析结果失败: {}", e)))?;

        Ok(result)
    }

    /// 更新文献标签
    pub async fn update_tags(&self, id: &str, tags: Vec<String>) -> Result<()> {
        let _: Option<Paper> = self
            .client
            .update(("paper", id))
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

        let paper_results: Vec<CountResult> = paper_response
            .take(0)
            .map_err(|e| Error::Storage(format!("解析文献统计失败: {}", e)))?;

        let chunk_results: Vec<CountResult> = chunk_response
            .take(0)
            .map_err(|e| Error::Storage(format!("解析分块统计失败: {}", e)))?;

        let paper_count = paper_results.first().map(|r| r.count).unwrap_or(0);
        let chunk_count = chunk_results.first().map(|r| r.count).unwrap_or(0);

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
