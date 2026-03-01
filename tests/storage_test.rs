//! 存储模块集成测试
//!
//! 注意：这些测试需要 SurrealDB 服务运行在 localhost:8000

use chrono::Utc;
use xuan_agent::config::DbConfig;
use xuan_agent::storage::{Chunk, ChunkType, Paper, SurrealDBStorage};

/// 创建测试用的数据库配置
fn test_db_config() -> DbConfig {
    DbConfig {
        connect: "ws://127.0.0.1:8000".to_string(),
        user: "root".to_string(),
        pass: "secret".to_string(),
        namespace: "test".to_string(),
        database: "test".to_string(),
    }
}

/// 创建测试文献
fn create_test_paper(title: &str) -> Paper {
    Paper {
        id: format!("paper:{}", uuid::Uuid::new_v4()),
        title: title.to_string(),
        abstract_text: format!("This is a test paper about {}", title),
        authors: vec!["Test Author".to_string()],
        year: Some(2024),
        journal: Some("Test Journal".to_string()),
        doi: None,
        file_path: None,
        tags: vec!["test".to_string(), "sample".to_string()],
        created_at: Utc::now(),
    }
}

/// 创建测试分块
fn create_test_chunks(paper_id: &str, count: usize) -> Vec<Chunk> {
    (0..count)
        .map(|i| Chunk {
            id: format!("chunk:{}", uuid::Uuid::new_v4()),
            paper_id: paper_id.to_string(),
            content: format!("This is chunk {} with some test content.", i),
            chunk_index: i,
            embedding: Some(vec![0.1_f32; 768]), // 假嵌入向量
            page_number: Some(i as u32),
            chunk_type: ChunkType::Other,
        })
        .collect()
}

#[tokio::test]
async fn test_storage_connection() {
    let config = test_db_config();

    // 尝试连接
    let result = SurrealDBStorage::connect(&config).await;

    // 如果 SurrealDB 未运行，跳过测试
    if result.is_err() {
        println!("Skipping test: SurrealDB not available at ws://127.0.0.1:8000");
        return;
    }

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_schema_initialization() {
    let config = test_db_config();
    let storage = match SurrealDBStorage::connect(&config).await {
        Ok(s) => s,
        Err(_) => {
            println!("Skipping test: SurrealDB not available");
            return;
        }
    };

    // 初始化 schema
    let result = storage.init_schema().await;
    assert!(result.is_ok(), "Schema initialization failed");
}

#[tokio::test]
async fn test_paper_crud() {
    let config = test_db_config();
    let storage = match SurrealDBStorage::connect(&config).await {
        Ok(s) => s,
        Err(_) => {
            println!("Skipping test: SurrealDB not available");
            return;
        }
    };

    // 初始化 schema
    let _ = storage.init_schema().await;

    // 创建测试文献
    let paper = create_test_paper("Test Paper CRUD");
    let paper_id = paper.id.clone();

    // 存储文献
    let stored_id = storage.store_paper(paper).await;
    assert!(stored_id.is_ok(), "Failed to store paper");
    assert_eq!(stored_id.unwrap(), paper_id);

    // 获取文献
    let retrieved = storage.get_paper(&paper_id).await;
    assert!(retrieved.is_ok(), "Failed to retrieve paper");
    let retrieved_paper = retrieved.unwrap();
    assert!(retrieved_paper.is_some(), "Paper not found");
    assert_eq!(retrieved_paper.unwrap().title, "Test Paper CRUD");

    // 搜索文献
    let search_results = storage.search_by_keyword("Test", 10).await;
    assert!(search_results.is_ok(), "Search failed");
    let results = search_results.unwrap();
    assert!(!results.is_empty(), "No search results found");

    // 清理
    let _ = storage.delete_paper(&paper_id).await;
}

#[tokio::test]
async fn test_chunk_operations() {
    let config = test_db_config();
    let storage = match SurrealDBStorage::connect(&config).await {
        Ok(s) => s,
        Err(_) => {
            println!("Skipping test: SurrealDB not available");
            return;
        }
    };

    // 初始化 schema
    let _ = storage.init_schema().await;

    // 创建并存储文献
    let paper = create_test_paper("Test Paper Chunks");
    let paper_id = storage.store_paper(paper).await.unwrap();

    // 创建测试分块
    let chunks = create_test_chunks(&paper_id, 5);

    // 存储分块
    let chunk_ids = storage.store_chunks(chunks).await;
    assert!(chunk_ids.is_ok(), "Failed to store chunks");
    assert_eq!(chunk_ids.unwrap().len(), 5);

    // 获取分块
    let retrieved_chunks = storage.get_chunks(&paper_id).await;
    assert!(retrieved_chunks.is_ok(), "Failed to retrieve chunks");
    let chunks = retrieved_chunks.unwrap();
    assert_eq!(chunks.len(), 5);

    // 清理
    let _ = storage.delete_paper(&paper_id).await;
}

#[tokio::test]
async fn test_vector_search() {
    let config = test_db_config();
    let storage = match SurrealDBStorage::connect(&config).await {
        Ok(s) => s,
        Err(_) => {
            println!("Skipping test: SurrealDB not available");
            return;
        }
    };

    // 初始化 schema
    let _ = storage.init_schema().await;

    // 创建并存储文献
    let paper = create_test_paper("Vector Search Test");
    let paper_id = storage.store_paper(paper).await.unwrap();

    // 创建带嵌入的分块
    let mut chunks = create_test_chunks(&paper_id, 3);

    // 第一个分块使用特定向量
    chunks[0].embedding = Some(vec![1.0_f32; 768]);
    chunks[0].content = "machine learning artificial intelligence".to_string();

    // 第二个分块使用不同的向量
    chunks[1].embedding = Some(vec![0.5_f32; 768]);
    chunks[1].content = "quantum physics entanglement".to_string();

    // 第三个分块使用另一个向量
    chunks[2].embedding = Some(vec![0.8_f32; 768]);
    chunks[2].content = "deep learning neural networks".to_string();

    // 存储分块
    let _ = storage.store_chunks(chunks).await;

    // 使用类似第一个分块的向量搜索
    let query_vector = vec![0.95_f32; 768];
    let search_results = storage.search_similar(&query_vector, 5, 0.5).await;

    assert!(search_results.is_ok(), "Vector search failed");
    let results = search_results.unwrap();
    assert!(!results.is_empty(), "No vector search results found");

    // 第一个结果应该最相似（向量最接近）
    if !results.is_empty() {
        assert!(results[0].similarity > 0.5, "Similarity too low");
    }

    // 清理
    let _ = storage.delete_paper(&paper_id).await;
}

#[tokio::test]
async fn test_list_papers() {
    let config = test_db_config();
    let storage = match SurrealDBStorage::connect(&config).await {
        Ok(s) => s,
        Err(_) => {
            println!("Skipping test: SurrealDB not available");
            return;
        }
    };

    // 初始化 schema
    let _ = storage.init_schema().await;

    // 创建多个测试文献
    let mut paper_ids = Vec::new();
    for i in 0..3 {
        let paper = create_test_paper(&format!("List Test Paper {}", i));
        let id = storage.store_paper(paper).await.unwrap();
        paper_ids.push(id);
    }

    // 列出文献
    let papers = storage.list_papers(10).await;
    assert!(papers.is_ok(), "Failed to list papers");
    let papers = papers.unwrap();
    assert!(papers.len() >= 3, "Not enough papers listed");

    // 清理
    for id in paper_ids {
        let _ = storage.delete_paper(&id).await;
    }
}

#[tokio::test]
async fn test_stats() {
    let config = test_db_config();
    let storage = match SurrealDBStorage::connect(&config).await {
        Ok(s) => s,
        Err(_) => {
            println!("Skipping test: SurrealDB not available");
            return;
        }
    };

    // 初始化 schema
    let _ = storage.init_schema().await;

    // 获取初始统计
    let (initial_papers, initial_chunks) = storage.stats().await.unwrap();

    // 创建文献和分块
    let paper = create_test_paper("Stats Test Paper");
    let paper_id = storage.store_paper(paper).await.unwrap();
    let chunks = create_test_chunks(&paper_id, 5);
    let _ = storage.store_chunks(chunks).await;

    // 获取更新后的统计
    let (new_papers, new_chunks) = storage.stats().await.unwrap();

    assert!(new_papers >= initial_papers + 1, "Paper count didn't increase");
    assert!(new_chunks >= initial_chunks + 5, "Chunk count didn't increase");

    // 清理
    let _ = storage.delete_paper(&paper_id).await;
}

#[tokio::test]
async fn test_tag_operations() {
    let config = test_db_config();
    let storage = match SurrealDBStorage::connect(&config).await {
        Ok(s) => s,
        Err(_) => {
            println!("Skipping test: SurrealDB not available");
            return;
        }
    };

    // 初始化 schema
    let _ = storage.init_schema().await;

    // 创建文献
    let mut paper = create_test_paper("Tag Test Paper");
    paper.tags = vec!["initial".to_string()];
    let paper_id = storage.store_paper(paper).await.unwrap();

    // 更新标签
    let new_tags = vec!["updated".to_string(), "test".to_string(), "tag".to_string()];
    let result = storage.update_tags(&paper_id, new_tags.clone()).await;
    assert!(result.is_ok(), "Failed to update tags");

    // 验证标签已更新
    let retrieved = storage.get_paper(&paper_id).await.unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().tags, new_tags);

    // 按标签搜索
    let search_results = storage.search_by_tag("test", 10).await;
    assert!(search_results.is_ok());
    assert!(!search_results.unwrap().is_empty());

    // 清理
    let _ = storage.delete_paper(&paper_id).await;
}
