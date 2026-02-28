# P1 阶段实施计划：文献智能处理

根据《XuanAgent 开发实施计划》，本文档详细规划 P1 阶段（第3-5个月）的实施细节。本阶段的核心目标是**集成 SurrealDB，实现文献深度理解与向量检索，完成文献阅读 MCP 开发，提供基础 Web 界面**。

---

## 一、阶段目标与验收标准

### 核心目标

在 P0 核心框架的基础上，实现完整的文献智能处理能力，包括向量检索、文献深度分析、自动标签系统和基础 Web 界面。

**关键特点**：
- 💾 **SurrealDB 集成**：统一的向量 + 结构化数据存储
- 🔍 **智能检索**：基于向量的语义检索
- 📄 **PDF 深度解析**：结构化提取文献内容
- 🏷️ **自动分类**：AI 驱动的标签和分类系统
- 🌐 **Web 界面**：基础的浏览器交互界面

### 验收标准

#### 1. 环境验收
- ✅ SurrealDB 服务正常运行并连接
- ✅ 能够存储和检索文献元数据
- ✅ 向量嵌入功能正常工作
- ✅ PDF 解析功能正常

#### 2. 功能验收
- ✅ **文献存储**：
  ```
  $ xuan-agent-cli import ./paper.pdf
  正在解析 PDF...
  文献已保存: "Deep Learning for CV" (2024)
  向量化完成，共 156 个切片
  ```

- ✅ **语义检索**：
  ```
  $ xuan-agent-cli search "深度学习在计算机视觉中的应用"
  找到 5 篇相关文献（相似度 > 0.75）:
  1. "Deep Learning for CV" - 相似度: 0.92
  ...
  ```

- ✅ **文献摘要**：
  ```
  $ xuan-agent-cli summarize paper_id_123
  📄 文献摘要
  标题: Deep Learning for CV
  作者: Zhang, San; Li, Si
  年份: 2024
  期刊: CVPR

  🎯 研究问题:
  本文提出了一种新的深度学习架构...

  🔬 主要方法:
  - 使用 Transformer 架构
  - 引入注意力机制优化

  📊 关键结果:
  - 在 ImageNet 上准确率达到 95.2%
  - 训练速度提升 40%
  ```

- ✅ **Web 界面**：
  ```
  $ cargo run --bin xuan-agent-web
  Server running at http://localhost:3000
  ```
  - 可通过浏览器访问
  - 支持聊天交互
  - 支持文献搜索

#### 3. 代码验收
- ✅ SurrealDB 存储模块通过单元测试
- ✅ PDF 解析模块通过测试
- ✅ 向量检索功能正常
- ✅ 文献阅读 MCP 完整实现
- ✅ Web 服务器可正常运行

---

## 二、Sprint 3 详细计划（第 5-6 周）：SurrealDB 集成与向量存储

### 目标
集成 SurrealDB，设计数据库 Schema，实现文献元数据和向量嵌入的存储与检索。

### 1. 任务分解

| 任务ID | 任务描述 | 负责模块 | 关键产出 | 预计时间 |
| :--- | :--- | :--- | :--- | :--- |
| **T3.1** | **SurrealDB SDK 集成** | `xuan-agent/storage` | SDK 集成，连接管理 | 0.5 天 |
| **T3.2** | **数据库 Schema 设计** | `xuan-agent/storage` | SurrealDB 表结构定义 | 1 天 |
| **T3.3** | **文献元数据存储** | `xuan-agent/storage` | 存储 API 实现 | 1.5 天 |
| **T3.4** | **向量嵌入存储** | `xuan-agent/rag` | 向量化模块，嵌入存储 | 2 天 |
| **T3.5** | **向量检索实现** | `xuan-agent/rag` | 相似度搜索，排序 | 1.5 天 |
| **T3.6** | **存储模块测试** | `tests/` | 单元测试，集成测试 | 1 天 |

### 2. 技术实现细节

#### 目录结构
```
crates/xuan-agent/src/
├── storage/
│   ├── mod.rs
│   ├── surrealdb.rs      # SurrealDB 集成
│   └── schema.rs         # 数据模型定义
├── rag/
│   ├── mod.rs
│   ├── embedding.rs      # 向量化模块
│   └── retrieval.rs      # 检索模块
└── models/
    ├── paper.rs          # 文献模型
    └── chunk.rs          # 文献切片模型
```

#### SurrealDB Schema 设计
```sql
-- 文献表
DEFINE TABLE paper SCHEMAFULL;
DEFINE FIELD title ON paper TYPE string;
DEFINE FIELD abstract ON paper TYPE string;
DEFINE FIELD authors ON paper TYPE array<string>;
DEFINE FIELD year ON paper TYPE int;
DEFINE FIELD journal ON paper TYPE string;
DEFINE FIELD doi ON paper TYPE string;
DEFINE FIELD file_path ON paper TYPE string;
DEFINE FIELD tags ON paper TYPE array<string>;
DEFINE FIELD created_at ON paper TYPE datetime DEFAULT time::now();

-- 文献切片表
DEFINE TABLE chunk SCHEMAFULL;
DEFINE FIELD paper_id ON paper TYPE string;
DEFINE FIELD content ON chunk TYPE string;
DEFINE FIELD chunk_index ON chunk TYPE int;
DEFINE FIELD embedding ON chunk TYPE array<float>;
DEFINE FIELD page_number ON chunk TYPE int;

-- 向量索引
DEFINE INDEX chunk_embedding_idx ON chunk VECTOR(embedding, 1536);
```

#### 文献模型
```rust
// crates/xuan-agent/src/models/paper.rs

use serde::{Deserialize, Serialize};
use surrealdb::sql::Datetime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paper {
    pub id: Option<String>,
    pub title: String,
    pub abstract: String,
    pub authors: Vec<String>,
    pub year: Option<i32>,
    pub journal: Option<String>,
    pub doi: Option<String>,
    pub file_path: Option<String>,
    pub tags: Vec<String>,
    pub created_at: Option<Datetime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    pub id: Option<String>,
    pub paper_id: String,
    pub content: String,
    pub chunk_index: i32,
    pub embedding: Option<Vec<f32>>,
    pub page_number: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub paper: Paper,
    pub chunk: Option<Chunk>,
    pub similarity: f64,
}
```

#### SurrealDB 存储实现
```rust
// crates/xuan-agent/src/storage/surrealdb.rs

use surrealdb::{Surreal, Any};
use crate::{Config, Result, models::{Paper, Chunk, SearchResult}};
use crate::error::Error;

pub struct SurrealDBStorage {
    db: Surreal<Any>,
}

impl SurrealDBStorage {
    /// 从配置创建连接
    pub async fn connect(config: &crate::config::DbConfig) -> Result<Self> {
        let db = Surreal::new::<Any>(&config.connect).await
            .map_err(|e| Error::Storage(format!("连接失败: {}", e)))?;

        // 认证
        db.signin(surrealdb::opt::Auth:: {
            namespace: &config.namespace,
            database: &config.database,
            username: &config.user,
            password: &config.pass,
        }).await
            .map_err(|e| Error::Storage(format!("认证失败: {}", e)))?;

        Ok(Self { db })
    }

    /// 存储文献
    pub async fn store_paper(&self, paper: Paper) -> Result<String> {
        let created: Vec<Paper> = self.db
            .create("paper")
            .content(paper)
            .await
            .map_err(|e| Error::Storage(format!("存储文献失败: {}", e)))?;

        Ok(created[0].id.as_ref().unwrap().clone())
    }

    /// 存储文献切片
    pub async fn store_chunks(&self, chunks: Vec<Chunk>) -> Result<()> {
        for chunk in chunks {
            self.db
                .create("chunk")
                .content(chunk)
                .await
                .map_err(|e| Error::Storage(format!("存储切片失败: {}", e)))?;
        }
        Ok(())
    }

    /// 向量检索
    pub async fn search_similar(
        &self,
        embedding: &[f32],
        limit: usize,
        threshold: f64,
    ) -> Result<Vec<SearchResult>> {
        let query = format!(
            "SELECT * FROM chunk WHERE embedding <|> $embedding < $threshold ORDER BY embedding <|> $embedding LIMIT $limit"
        );

        let mut result = self.db
            .query(&query)
            .bind(("embedding", embedding.to_vec()))
            .bind(("threshold", threshold))
            .bind(("limit", limit))
            .await
            .map_err(|e| Error::Storage(format!("检索失败: {}", e)))?;

        let chunks: Vec<Chunk> = result.take(0)
            .map_err(|e| Error::Storage(format!("解析结果失败: {}", e)))?;

        // TODO: 获取关联的文献信息
        let results = chunks.into_iter().map(|chunk| SearchResult {
            paper: Paper::default(), // 需要从数据库获取
            chunk: Some(chunk),
            similarity: 0.0, // 需要计算
        }).collect();

        Ok(results)
    }

    /// 按关键词检索
    pub async fn search_by_keyword(&self, keyword: &str, limit: usize) -> Result<Vec<Paper>> {
        let papers: Vec<Paper> = self.db
            .query("SELECT * FROM paper WHERE title CONTAINS $keyword OR @string::join(authors, ', ') CONTAINS $keyword LIMIT $limit")
            .bind(("keyword", keyword))
            .bind(("limit", limit))
            .await
            .map_err(|e| Error::Storage(format!("关键词检索失败: {}", e)))?
            .take(0)
            .map_err(|e| Error::Storage(format!("解析结果失败: {}", e)))?;

        Ok(papers)
    }
}
```

#### 向量化模块
```rust
// crates/xuan-agent/src/rag/embedding.rs

use crate::{Result, Error};
use crate::config::Config;

pub struct EmbeddingService {
    // 使用配置的 LLM Provider 进行向量化
    config: Config,
}

impl EmbeddingService {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// 生成文本的向量嵌入
    pub async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        // 调用 LLM API 获取嵌入
        // 这里需要根据配置的 provider 选择不同的实现
        let (provider, base_url, model) = self.config.get_llm_config();

        match provider {
            "openai" => self.embed_openai(base_url, model, text).await,
            "siliconflow" => self.embed_openai_compatible(base_url, model, text).await,
            "ollama" => self.embed_ollama(base_url, model, text).await,
            _ => Err(Error::Config(format!("不支持的 provider: {}", provider))),
        }
    }

    /// 批量生成嵌入
    pub async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let mut embeddings = Vec::new();
        for text in texts {
            embeddings.push(self.embed(text).await?);
        }
        Ok(embeddings)
    }
}
```

### 3. CLI 命令增强
```bash
# 数据库初始化
$ xuan-agent-cli db init
正在连接 SurrealDB...
创建表结构...
完成！

# 导入文献
$ xuan-agent-cli import ./papers/paper.pdf
正在解析 PDF...
提取元数据...
生成向量嵌入...
存储到数据库...
完成！文献 ID: paper:abc123

# 检索文献
$ xuan-agent-cli search "深度学习"
找到 5 篇相关文献:
1. "深度学习入门" - 相似度: 0.89
2. "深度学习实战" - 相似度: 0.85
...
```

### 4. 交付物
- ✅ SurrealDB 连接和认证
- ✅ 文献和切片的数据模型
- ✅ 基础存储 API (增删改查)
- ✅ 向量检索功能
- ✅ 关键词检索功能
- ✅ 单元测试和集成测试

---

## 三、Sprint 4 详细计划（第 7-8 周）：文献阅读 MCP 开发

### 目标
开发 `mcp-server-reading`，实现 PDF 解析、文献切片、向量化，提供完整的文献阅读 MCP 工具集。

### 1. 任务分解

| 任务ID | 任务描述 | 负责模块 | 关键产出 | 预计时间 |
| :--- | :--- | :--- | :--- | :--- |
| **T4.1** | **MCP Server 框架搭建** | `mcp-server-reading` | Server 基础结构 | 1 天 |
| **T4.2** | **PDF 解析实现** | `mcp-server-reading/pdf` | PDF 文本提取，结构解析 | 2 天 |
| **T4.3** | **文献切片算法** | `mcp-server-reading/chunk` | 智能切片实现 | 1.5 天 |
| **T4.4** | **MCP Tools 实现** | `mcp-server-reading` | 文献检索、摘要、对比工具 | 2 天 |
| **T4.5** | **元数据提取** | `mcp-server-reading/meta` | 标题、作者、摘要提取 | 1 天 |
| **T4.6** | **测试与文档** | `mcp-server-reading` | 工具测试，使用文档 | 1.5 天 |

### 2. 技术实现细节

#### MCP Server 结构
```
mcp-servers/mcp-server-reading/
├── Cargo.toml
└── src/
    ├── main.rs           # MCP Server 入口
    ├── pdf.rs            # PDF 解析
    ├── chunk.rs          # 文献切片
    ├── meta.rs           # 元数据提取
    ├── tools.rs          # MCP 工具定义
    └── storage.rs        # 存储接口
```

#### MCP Server 入口
```rust
// mcp-servers/mcp-server-reading/src/main.rs

use mcp_server::{Server, ServerBuilder};
use mcp_server::handler::{ToolHandler, tools_handler};

#[tokio::main]
async fn main() -> Result<()> {
    let mut server = ServerBuilder::new("mcp-server-reading")
        .version("0.1.0")
        .build();

    // 注册工具
    server.add_tool("summarize_paper", summarize_paper_handler());
    server.add_tool("search_papers", search_papers_handler());
    server.add_tool("compare_papers", compare_papers_handler());
    server.add_tool("extract_entities", extract_entities_handler());

    // 启动服务器 (stdio)
    server.run_stdio().await?;

    Ok(())
}
```

#### PDF 解析
```rust
// mcp-servers/mcp-server-reading/src/pdf.rs

use pdfium::PdfDocument;

pub struct PdfParser {
    // PDF 解析配置
}

impl PdfParser {
    /// 解析 PDF 文件
    pub fn parse(path: &str) -> Result<ParsedDocument> {
        let document = PdfDocument::load_from_file(path)?;

        // 提取文本内容
        let text = Self::extract_text(&document)?;

        // 提取结构信息（章节、段落）
        let structure = Self::extract_structure(&document)?;

        Ok(ParsedDocument {
            text,
            structure,
            page_count: document.page_count(),
        })
    }

    /// 提取文本内容
    fn extract_text(document: &PdfDocument) -> Result<String> {
        let mut text = String::new();
        for page in document.pages() {
            text.push_str(&page.text()?);
            text.push('\n');
        }
        Ok(text)
    }

    /// 提取结构信息
    fn extract_structure(document: &PdfDocument) -> Result<DocumentStructure> {
        // 识别标题、段落、表格等
        // ...
    }
}
```

#### 智能切片算法
```rust
// mcp-servers/mcp-server-reading/src/chunk.rs

use super::{ParsedDocument, Chunk};

pub struct Chunker {
    max_chunk_size: usize,
    overlap: usize,
}

impl Chunker {
    pub fn new(max_chunk_size: usize, overlap: usize) -> Self {
        Self { max_chunk_size, overlap }
    }

    /// 按章节切片
    pub fn chunk_by_section(&self, doc: &ParsedDocument) -> Vec<Chunk> {
        let mut chunks = Vec::new();

        for section in &doc.structure.sections {
            // 按段落切片
            let section_chunks = self.chunk_by_paragraph(&section.content);
            chunks.extend(section_chunks);
        }

        chunks
    }

    /// 按段落切片（保留上下文重叠）
    pub fn chunk_by_paragraph(&self, text: &str) -> Vec<Chunk> {
        let paragraphs: Vec<&str> = text.split("\n\n").collect();
        let mut chunks = Vec::new();
        let mut current_chunk = String::new();
        let mut chunk_index = 0;

        for para in paragraphs {
            if current_chunk.len() + para.len() > self.max_chunk_size {
                if !current_chunk.is_empty() {
                    chunks.push(Chunk {
                        content: current_chunk.clone(),
                        chunk_index,
                        ..Default::default()
                    });
                    chunk_index += 1;
                }
                current_chunk = para.to_string();
            } else {
                if !current_chunk.is_empty() {
                    current_chunk.push_str("\n\n");
                }
                current_chunk.push_str(para);
            }
        }

        if !current_chunk.is_empty() {
            chunks.push(Chunk {
                content: current_chunk,
                chunk_index,
                ..Default::default()
            });
        }

        chunks
    }
}
```

#### MCP Tools 实现
```rust
// mcp-servers/mcp-server-reading/src/tools.rs

use serde_json::{json, Value};

/// summarize_paper 工具
pub fn summarize_paper_handler() -> ToolHandler {
    ToolHandler::new("summarize_paper")
        .description("生成文献的结构化摘要，包括研究问题、方法、结果、创新点")
        .parameter("file_path", "string", "PDF 文件路径")
        .handler(|params| async move {
            let file_path = params.get("file_path")
                .and_then(|v| v.as_str())
                .ok_or("缺少 file_path 参数")?;

            // 解析 PDF
            let doc = PdfParser::parse(file_path)?;

            // 提取元数据
            let meta = MetadataExtractor::extract(&doc)?;

            // 调用 LLM 生成摘要
            let summary = LlmService::generate_summary(&doc, &meta).await?;

            Ok(json!({
                "title": meta.title,
                "authors": meta.authors,
                "abstract": meta.abstract_text,
                "summary": summary,
            }))
        })
}

/// search_papers 工具
pub fn search_papers_handler() -> ToolHandler {
    ToolHandler::new("search_papers")
        .description("在文献库中搜索相关文献")
        .parameter("query", "string", "搜索查询")
        .parameter("limit", "integer", "返回数量限制")
        .handler(|params| async move {
            let query = params.get("query").and_then(|v| v.as_str()).unwrap_or("");
            let limit = params.get("limit").and_then(|v| v.as_i64()).unwrap_or(10) as usize;

            // 向量检索
            let storage = Storage::get().await?;
            let embedding = EmbeddingService::embed(query).await?;
            let results = storage.search_similar(&embedding, limit, 0.7).await?;

            Ok(json!({
                "query": query,
                "results": results,
            }))
        })
}

/// compare_papers 工具
pub fn compare_papers_handler() -> ToolHandler {
    ToolHandler::new("compare_papers")
        .description("对比多篇文献的方法、数据集、结果")
        .parameter("paper_ids", "array", "文献 ID 列表")
        .handler(|params| async move {
            let paper_ids = params.get("paper_ids")
                .and_then(|v| v.as_array())
                .ok_or("缺少 paper_ids 参数")?;

            let storage = Storage::get().await?;
            let mut papers = Vec::new();

            for id in paper_ids {
                if let Some(id_str) = id.as_str() {
                    if let Ok(paper) = storage.get_paper(id_str).await {
                        papers.push(paper);
                    }
                }
            }

            // 生成对比表
            let comparison = ComparisonGenerator::generate(&papers).await?;

            Ok(json!({
                "papers": papers,
                "comparison": comparison,
            }))
        })
}
```

### 3. MCP 协议示例

#### 客户端调用
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "summarize_paper",
    "arguments": {
      "file_path": "/path/to/paper.pdf"
    }
  },
  "id": 1
}
```

#### 服务器响应
```json
{
  "jsonrpc": "2.0",
  "result": {
    "title": "Deep Learning for Computer Vision",
    "authors": ["Zhang, San", "Li, Si"],
    "abstract": "本文提出了一种新的深度学习架构...",
    "summary": {
      "research_question": "如何提高深度学习模型在计算机视觉任务中的性能？",
      "methods": ["Transformer架构", "注意力机制优化"],
      "key_results": ["ImageNet准确率95.2%", "训练速度提升40%"],
      "innovations": ["新的注意力机制", "轻量化设计"],
      "limitations": ["需要大量训练数据", "推理速度较慢"]
    }
  },
  "id": 1
}
```

### 4. 交付物
- ✅ `mcp-server-reading` 完整实现
- ✅ PDF 解析功能
- ✅ 智能切片算法
- ✅ 4 个核心 MCP 工具
- ✅ 工具测试
- ✅ 使用文档

---

## 四、Sprint 5 详细计划（第 9-10 周）：知识管理自动化

### 目标
实现自动标签系统、文献分类、引用关系分析，优化向量检索性能。

### 1. 任务分解

| 任务ID | 任务描述 | 负责模块 | 关键产出 | 预计时间 |
| :--- | :--- | :--- | :--- | :--- |
| **T5.1** | **自动标签系统** | `xuan-agent/classification` | 标签生成，置信度评分 | 2 天 |
| **T5.2** | **文献分类** | `xuan-agent/classification` | 按研究领域、类型分类 | 1.5 天 |
| **T5.3** | **引用关系分析** | `xuan-agent/citation` | 引用网络构建 | 2 天 |
| **T5.4** | **检索性能优化** | `xuan-agent/rag` | 缓存，混合检索 | 1.5 天 |
| **T5.5** | **Re-rank 实现** | `xuan-agent/rag` | 结果重排序 | 1 天 |
| **T5.6** | **测试与优化** | `tests/` | 性能测试，压力测试 | 2 天 |

### 2. 技术实现细节

#### 自动标签系统
```rust
// crates/xuan-agent/src/classification/tagging.rs

use crate::{Result, LlmService};

pub struct TaggingService {
    llm: LlmService,
}

impl TaggingService {
    /// 为文献生成标签
    pub async fn generate_tags(&self, paper: &Paper) -> Result<Vec<Tag>> {
        let prompt = format!(
            "请为以下文献生成 3-5 个标签，每个标签包含名称和置信度（0-1）：

标题: {}
摘要: {}
作者: {}

请以 JSON 格式返回：{{\"tags\": [{{\"name\": \"标签名\", \"confidence\": 0.95}}]}}",
            paper.title, paper.abstract, paper.authors.join(", ")
        );

        let response = self.llm.complete(&prompt).await?;
        let result: TagResponse = serde_json::from_str(&response)?;

        Ok(result.tags)
    }

    /// 按研究领域分类
    pub async fn classify_by_field(&self, paper: &Paper) -> Result<String> {
        let fields = vec![
            "计算机科学", "生物学", "医学", "物理学",
            "化学", "工程学", "数学", "经济学"
        ];

        let prompt = format!(
            "请判断以下文献属于哪个研究领域：{}

标题: {}
摘要: {}

可选领域: {}

请只返回领域名称。",
            paper.title, paper.abstract, fields.join(", ")
        );

        let field = self.llm.complete(&prompt).await?;
        Ok(field.trim().to_string())
    }

    /// 按研究类型分类
    pub async fn classify_by_type(&self, paper: &Paper) -> Result<ResearchType> {
        // 方法论文、应用论文、综述论文等
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub name: String,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResearchType {
    Methodology,  // 方法学
    Application,  // 应用
    Survey,       // 综述
    CaseStudy,    // 案例研究
}
```

#### 引用关系分析
```rust
// crates/xuan-agent/src/citation/analyzer.rs

use crate::{Result, models::Paper};

pub struct CitationAnalyzer {
    storage: Arc<SurrealDBStorage>,
}

impl CitationAnalyzer {
    /// 提取文献中的引用
    pub async fn extract_citations(&self, paper: &Paper) -> Result<Vec<Citation>> {
        let mut citations = Vec::new();

        // 从参考文献列表中提取
        for (idx, ref_text) in paper.references.iter().enumerate() {
            let citation = self.parse_reference(ref_text)?;
            citations.push(Citation {
                id: format!("citation:{}", idx),
                source_paper_id: paper.id.clone().unwrap(),
                ..citation
            });
        }

        Ok(citations)
    }

    /// 构建引用网络
    pub async fn build_citation_graph(&self, paper_id: &str) -> Result<CitationGraph> {
        // 获取引用的文献
        let citations = self.storage.get_citations(paper_id).await?;

        // 获取引用该文献的文献
        let cited_by = self.storage.get_cited_by(paper_id).await?;

        Ok(CitationGraph {
            paper_id: paper_id.to_string(),
            cites: citations,
            cited_by,
        })
    }

    /// 查找相关文献（基于引用关系）
    pub async fn find_related_papers(&self, paper_id: &str, limit: usize) -> Result<Vec<Paper>> {
        // 1. 共同引用的文献
        // 2. 被共同引用的文献
        // 3. 引用链中的文献
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Citation {
    pub id: String,
    pub source_paper_id: String,
    pub title: String,
    pub authors: Vec<String>,
    pub year: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitationGraph {
    pub paper_id: String,
    pub cites: Vec<Citation>,     // 该文献引用的
    pub cited_by: Vec<Citation>,  // 引用该文献的
}
```

#### 检索性能优化
```rust
// crates/xuan-agent/src/rag/retrieval.rs

use crate::{Result, models::{Paper, SearchResult}};

pub struct RetrievalService {
    storage: Arc<SurrealDBStorage>,
    embedding: Arc<EmbeddingService>,
    cache: Arc<Mutex<LruCache<String, Vec<SearchResult>>>>,
}

impl RetrievalService {
    /// 混合检索（向量 + 关键词）
    pub async fn hybrid_search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        // 并行执行两种检索
        let (vector_results, keyword_results) = tokio::join!(
            self.vector_search(query, limit * 2),
            self.keyword_search(query, limit * 2)
        );

        let vector_results = vector_results?;
        let keyword_results = keyword_results?;

        // 合并和重排序
        let merged = Self::merge_results(vector_results, keyword_results);
        let reranked = self.rerank(query, merged).await?;

        Ok(reranked.into_iter().take(limit).collect())
    }

    /// Re-rank 检索结果
    async fn rerank(&self, query: &str, results: Vec<SearchResult>) -> Result<Vec<SearchResult>> {
        // 使用 Re-rank 模型重新排序
        // 可以使用 CrossEncoder 等模型
    }

    /// 缓存检索结果
    async fn cached_search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        let cache_key = format!("{}:{}", query, limit);

        // 检查缓存
        {
            let mut cache = self.cache.lock().await;
            if let Some(results) = cache.get(&cache_key) {
                return Ok(results.clone());
            }
        }

        // 执行检索
        let results = self.hybrid_search(query, limit).await?;

        // 更新缓存
        {
            let mut cache = self.cache.lock().await;
            cache.put(cache_key, results.clone());
        }

        Ok(results)
    }
}
```

### 3. CLI 命令
```bash
# 自动标签
$ xuan-agent-cli tag paper_id_123
生成标签中...
标签: 深度学习 (置信度: 0.95)
标签: 计算机视觉 (置信度: 0.92)
标签: Transformer (置信度: 0.88)

# 查看引用网络
$ xuan-agent-cli citation paper_id_123
引用网络:
├─ 引用了 (5 篇):
│  └─ "Attention Is All You Need" (2017)
├─ 被引用 (12 篇):
│  └─ "Improved Transformer" (2024)
└─ 相关文献 (8 篇):
   └─ "Vision Transformer" (2021)

# 批量分类
$ xuan-agent-cli classify --all
正在分类 234 篇文献...
完成！已更新标签。
```

### 4. 交付物
- ✅ 自动标签系统
- ✅ 文献分类功能
- ✅ 引用关系分析
- ✅ 检索性能优化
- ✅ Re-rank 功能
- ✅ 性能测试报告

---

## 五、Sprint 5-2 详细计划（第 10-11 周）：基础 Web 界面

### 目标
创建 `xuan-agent-web` crate，实现基础的 Web 服务器和简单的聊天界面。

### 1. 任务分解

| 任务ID | 任务描述 | 负责模块 | 关键产出 | 预计时间 |
| :--- | :--- | :--- | :--- | :--- |
| **T5.2.1** | **Web crate 创建** | `xuan-agent-web` | 项目结构，Axum 集成 | 0.5 天 |
| **T5.2.2** | **API 路由实现** | `xuan-agent-web` | 聊天、搜索 API | 1 天 |
| **T5.2.3** | **静态资源服务** | `xuan-agent-web` | HTML/CSS/JS | 1 天 |
| **T5.2.4** | **WebSocket 支持** | `xuan-agent-web` | 实时聊天 | 1.5 天 |
| **T5.2.5** | **会话管理** | `xuan-agent-web` | 会话存储 | 1 天 |
| **T5.2.6** | **测试与部署** | `xuan-agent-web` | 功能测试 | 0.5 天 |

### 2. 技术实现细节

#### Web 项目结构
```
crates/xuan-agent-web/
├── Cargo.toml
└── src/
    ├── main.rs           # 服务器入口
    ├── routes/           # API 路由
    │   ├── chat.rs       # 聊天 API
    │   └── search.rs     # 搜索 API
    └── static/           # 静态资源
        ├── index.html
        ├── styles.css
        └── app.js
```

#### Web 服务器入口
```rust
// crates/xuan-agent-web/src/main.rs

use axum::{Router, Json, extract::State};
use axum::routing::{get, post};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::net::TcpListener;
use xuan_agent::XuanAgent;

#[derive(Clone)]
struct AppState {
    agent: Arc<tokio::sync::Mutex<XuanAgent>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    // 加载配置
    let config = xuan_agent::Config::from_env()?;

    // 创建 Agent
    let agent = XuanAgent::new(config).await?;
    let state = AppState {
        agent: Arc::new(tokio::sync::Mutex::new(agent)),
    };

    // 构建路由
    let app = Router::new()
        // API 路由
        .route("/api/chat", post(chat_handler))
        .route("/api/search", post(search_handler))
        .route("/api/papers", get(list_papers_handler))
        // 静态文件
        .route("/", get(index_handler))
        .route("/assets/*file", get(assets_handler))
        .with_state(state);

    // 启动服务器
    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    println!("🚀 Server running at http://localhost:3000");

    axum::serve(listener, app).await?;

    Ok(())
}

// 聊天 API
async fn chat_handler(
    State(state): State<AppState>,
    Json(req): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, AppError> {
    let mut agent = state.agent.lock().await;
    let response = agent.chat(&req.message).await?;

    Ok(Json(ChatResponse {
        response,
        session_id: req.session_id.unwrap_or_default(),
    }))
}

// 首页
async fn index_handler() -> &'static str {
    include_str!("static/index.html")
}

// 错误处理
struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.0.to_string()).into_response()
    }
}
```

#### 前端界面
```html
<!-- crates/xuan-agent-web/src/static/index.html -->

<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>XuanAgent - 科研 AI 助手</title>
    <link rel="stylesheet" href="/assets/styles.css">
</head>
<body>
    <div class="container">
        <header>
            <h1>🔬 XuanAgent</h1>
            <p>你的科研 AI 助手</p>
        </header>

        <main>
            <div id="chat-container">
                <div id="messages"></div>
            </div>

            <div class="input-area">
                <textarea id="user-input" placeholder="输入你的问题..."></textarea>
                <button id="send-btn">发送</button>
            </div>
        </main>
    </div>

    <script src="/assets/app.js"></script>
</body>
</html>
```

```css
/* crates/xuan-agent-web/src/static/styles.css */

* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

body {
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    min-height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
}

.container {
    width: 90%;
    max-width: 800px;
    background: white;
    border-radius: 16px;
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
    overflow: hidden;
}

header {
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    color: white;
    padding: 20px;
    text-align: center;
}

#chat-container {
    height: 500px;
    overflow-y: auto;
    padding: 20px;
}

.message {
    margin-bottom: 16px;
    padding: 12px 16px;
    border-radius: 8px;
    max-width: 80%;
}

.message.user {
    background: #667eea;
    color: white;
    margin-left: auto;
}

.message.assistant {
    background: #f3f4f6;
    color: #1f2937;
}

.input-area {
    display: flex;
    padding: 16px;
    gap: 12px;
    border-top: 1px solid #e5e7eb;
}

#user-input {
    flex: 1;
    padding: 12px;
    border: 1px solid #d1d5db;
    border-radius: 8px;
    resize: none;
    font-family: inherit;
}

#send-btn {
    padding: 12px 24px;
    background: #667eea;
    color: white;
    border: none;
    border-radius: 8px;
    cursor: pointer;
}

#send-btn:hover {
    background: #5a67d8;
}
```

```javascript
// crates/xuan-agent-web/src/static/app.js

const messagesContainer = document.getElementById('messages');
const userInput = document.getElementById('user-input');
const sendBtn = document.getElementById('send-btn');

// 发送消息
async function sendMessage() {
    const message = userInput.value.trim();
    if (!message) return;

    // 添加用户消息
    addMessage(message, 'user');
    userInput.value = '';

    // 发送到服务器
    try {
        const response = await fetch('/api/chat', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({ message }),
        });

        const data = await response.json();

        // 添加助手回复
        addMessage(data.response, 'assistant');
    } catch (error) {
        console.error('Error:', error);
        addMessage('抱歉，发生了错误。请稍后重试。', 'assistant');
    }
}

// 添加消息到聊天界面
function addMessage(text, role) {
    const messageDiv = document.createElement('div');
    messageDiv.className = `message ${role}`;
    messageDiv.textContent = text;
    messagesContainer.appendChild(messageDiv);
    messagesContainer.scrollTop = messagesContainer.scrollHeight;
}

// 事件监听
sendBtn.addEventListener('click', sendMessage);
userInput.addEventListener('keypress', (e) => {
    if (e.key === 'Enter' && !e.shiftKey) {
        e.preventDefault();
        sendMessage();
    }
});

// 欢迎消息
addMessage('你好！我是 XuanAgent，你的科研 AI 助手。有什么可以帮你的吗？', 'assistant');
```

### 3. 运行方式
```bash
# 启动 Web 服务器
$ cargo run --bin xuan-agent-web
🚀 Server running at http://localhost:3000

# 访问浏览器
# 打开 http://localhost:3000
```

### 4. 交付物
- ✅ `xuan-agent-web` crate
- ✅ REST API 接口
- ✅ 聊天界面
- ✅ 基础样式
- ✅ 会话管理
- ✅ 功能测试

---

## 六、P1 阶段总体交付物

### 功能交付
- ✅ **v0.5.0 Beta**: 完整的文献管理与检索能力
- ✅ SurrealDB 完整集成
- ✅ 文献阅读 MCP 稳定版本
- ✅ 基础 Web 界面

### 技术交付
- ✅ 向量检索系统
- ✅ PDF 解析模块
- ✅ 自动标签系统
- ✅ 引用关系分析
- ✅ 检索性能优化
- ✅ Web 服务器

### 文档交付
- ✅ API 文档
- ✅ MCP 协议文档
- ✅ 用户使用手册
- ✅ 性能测试报告

---

## 七、技术风险与应对预案

| 风险点 | 可能的影响 | 应对预案 |
| :--- | :--- | :--- |
| **SurrealDB 兼容性** | SurrealDB 版本升级导致 API 不兼容 | 锁定版本号，做好版本兼容测试 |
| **PDF 解析质量** | 复杂 PDF 格式解析失败 | 提供多种解析器备选，记录失败日志 |
| **向量检索性能** | 大规模文献检索响应慢 | 实现分页、缓存、索引优化 |
| **嵌入 API 限制** | API 配额不足或限流 | 实现嵌入缓存，支持本地模型 |
| **LLM 输出不稳定** | 标签和分类结果不一致 | 设置温度参数，人工审核机制 |

---

## 八、性能指标

### 检索性能
| 指标 | 目标值 |
| :--- | :--- |
| 向量检索响应时间 | < 2 秒 (1000 篇文献) |
| 关键词检索响应时间 | < 1 秒 |
| PDF 解析时间 | < 5 秒 (20 页 PDF) |
| 自动标签生成 | < 10 秒 |
| Web 界面响应 | < 500ms |

### 存储性能
| 指标 | 目标值 |
| :--- | :--- |
| 单用户文献容量 | 100,000+ 篇 |
| 向量维度 | 1536 (OpenAI) |
| 切片大小 | 500-1000 字 |
| 重叠率 | 10-20% |

---

## 九、下一步行动

### P1-Sprint 3 启动检查清单

```bash
# 1. 安装 SurrealDB
docker run -d -p 8000:8000 \
  -e SURREALDB_USER=root \
  -e SURREALDB_PASS=secret \
  surrealdb/surrealdb:latest \
  start --log trace --user root --pass secret

# 2. 验证连接
curl -http://localhost:8000/sql

# 3. 创建数据库结构
# (通过代码或 SQL 脚本)

# 4. 运行测试
cargo test --package xuan-agent

# 5. 导入测试文献
cargo run --bin xuan-agent-cli -- import test.pdf
```

---

**预期成果**：在 4 周内完成 SurrealDB 集成、文献阅读 MCP 开发、知识管理功能，实现完整的文献智能处理能力，为 P2 阶段的学术写作辅助奠定基础。

**最后更新**: 2025-03-01
**项目阶段**: P1 - 文献智能处理
**预计完成**: 第 5 个月末
