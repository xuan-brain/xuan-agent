# XuanAgent 项目开发指南

> 基于文档的 AI 辅助开发上下文文件

---

## 项目定位

**XuanAgent** 是一个基于 Rust 的科研 AI Agent **库 (lib crate)**，提供文献管理、学术写作辅助和科研知识管理功能。

### 核心设计理念
- 📦 **核心是库** - 可嵌入到其他应用中，而非独立应用
- 🛠️ **提供 CLI 工具** - 用于开发测试和独立运行
- 🌐 **Web 界面可选** - 规划中，优先级低于 CLI
- 🔌 **标准化协议** - 采用 MCP (Model Context Protocol)
- 💾 **一体化存储** - SurrealDB 统一管理向量和结构化数据

### 产品愿景
为科研人员提供从**文献发现 → 阅读理解 → 知识管理 → 学术写作**的一站式智能辅助平台。

---

## 技术架构

### 核心技术栈

| 组件 | 技术选型 | 版本 |
|:-----|:---------|:-----|
| 语言 | Rust | 2024 edition |
| LLM 框架 | Rig | 0.5 |
| 协议 | MCP | Model Context Protocol |
| 数据库 | SurrealDB | latest |
| PDF 处理 | pdfium | - |
| Web 框架 | Axum | 0.7+ |
| 异步运行时 | Tokio | 1.x |

### 项目结构 (完整版)

```
xuan-agent/
├── Cargo.toml                  # Workspace 配置
├── .env                        # AI Provider 配置 (JSON 格式)
├── .env.example                # 配置模板
│
├── crates/
│   ├── xuan-agent/             # 核心 lib crate ★
│   │   └── src/
│   │       ├── lib.rs          # 库入口，导出公共 API
│   │       ├── agent.rs        # Agent 核心逻辑
│   │       ├── config.rs       # 配置管理
│   │       ├── error.rs        # 错误类型定义
│   │       │
│   │       ├── storage/        # 存储层
│   │       │   ├── mod.rs
│   │       │   ├── surrealdb.rs    # SurrealDB 集成
│   │       │   └── schema.rs       # 数据模型
│   │       │
│   │       ├── rag/            # 检索增强生成
│   │       │   ├── mod.rs
│   │       │   ├── embedding.rs   # 向量化服务
│   │       │   └── retrieval.rs   # 检索服务
│   │       │
│   │       ├── models/         # 数据模型
│   │       │   ├── paper.rs       # 文献模型
│   │       │   └── chunk.rs       # 文献切片模型
│   │       │
│   │       ├── classification/ # 分类系统
│   │       │   ├── tagging.rs     # 自动标签
│   │       │   └── classifier.rs  # 文献分类
│   │       │
│   │       └── citation/       # 引用分析
│   │           └── analyzer.rs   # 引用网络分析
│   │
│   ├── xuan-agent-cli/         # CLI 工具 (bin crate) ★
│   │   └── src/main.rs
│   │
│   └── xuan-agent-web/         # Web 应用 (bin crate)
│       └── src/
│           ├── main.rs         # Web 服务器入口
│           ├── routes/         # API 路由
│           └── static/         # 静态资源
│
├── mcp-host/                   # MCP Host 实现
│   └── src/lib.rs
│
├── mcp-servers/                # MCP Server 实现
│   ├── mcp-server-reading/     # 文献阅读 MCP ★
│   │   └── src/
│   │       ├── main.rs
│   │       ├── pdf.rs
│   │       ├── chunk.rs
│   │       ├── meta.rs
│   │       └── tools.rs
│   │
│   └── mcp-server-writing/     # 学术写作 MCP
│       └── src/
│           ├── main.rs
│           └── tools.rs
│
├── tests/                      # 集成测试
│   └── integration_test.rs
│
├── docs/                       # 项目文档 ★
│   ├── requirements-specification.md   # 完整需求说明
│   ├── implement-plan.md              # 详细实施计划
│   ├── plan-p0.md                     # P0 阶段详细计划
│   └── plan-p1.md                     # P1 阶段详细计划
│
└── examples/                   # 使用示例
    ├── basic_usage.rs
    └── mcp_integration.rs
```

---

## 配置管理

### .env 文件格式 (JSON)

项目使用 **JSON 格式**的 `.env` 配置文件，支持多种 AI Provider：

```json
{
  "ai_provider": {
    "provider": "siliconflow",
    "base_url": "https://api.siliconflow.cn/v1",
    "api_key": "sk-xxxxx",
    "model": "deepseek-ai/DeepSeek-V3",
    "description": "硅基流动 DeepSeek 配置"
  },
  "db": {
    "connect": "http://127.0.0.1:8000",
    "user": "root",
    "pass": "secret",
    "namespace": "agent",
    "database": "dev"
  }
}
```

### 支持的 AI Provider

| Provider | base_url | 说明 |
|:---------|:---------|:-----|
| OpenAI | `https://api.openai.com/v1` | 官方 OpenAI API |
| Anthropic | `https://api.anthropic.com` | Claude 系列 |
| 硅基流动 | `https://api.siliconflow.cn/v1` | 国内 API，支持 DeepSeek |
| Ollama | `http://localhost:11434` | 本地模型 |

### 配置加载代码

```rust
use xuan_agent::Config;

// 从 .env 加载配置
let config = Config::from_env()?;

// 获取 LLM 配置
let (provider, base_url, model) = config.get_llm_config();

// 获取数据库 URL
let db_url = config.get_db_url();
```

---

## 核心 API 设计

### XuanAgent 库 API

```rust
use xuan_agent::{XuanAgent, Config};

#[tokio::main]
async fn main() -> Result<()> {
    // 从 .env 加载配置
    let config = Config::from_env()?;

    // 创建 Agent 实例
    let mut agent = XuanAgent::new(config).await?;

    // 对话
    let response = agent.chat("你好").await?;
    println!("{}", response);

    // 添加 MCP 服务器
    agent.add_mcp_server("library", ZoteroMcp::new()).await?;

    // 检索文献
    let papers = agent.search_papers("深度学习", 10).await?;

    Ok(())
}
```

### 主要方法

| 方法 | 说明 |
|:-----|:-----|
| `XuanAgent::new(config)` | 创建 Agent 实例 |
| `agent.chat(message)` | 发送消息，获取响应 |
| `agent.add_mcp_server(name, server)` | 注册 MCP 服务器 |
| `agent.search_papers(query, limit)` | 检索文献 |
| `agent.import_paper(path)` | 导入 PDF 文献 |

---

## MCP 集成架构

### MCP 协议说明

MCP (Model Context Protocol) 是 AI 应用与数据源之间的标准化通信协议。

### 文献管理集成

**重要**: XuanAgent 设计为**通用**科研助手，支持多种文献管理工具：

| 工具 | 状态 | 说明 |
|:-----|:-----|:-----|
| Zotero | ✅ 示例实现 | 开发测试用，已集成 |
| EndNote | 🔄 规划中 | 通过 BibTeX 格式 |
| Mendeley | 🔄 规划中 | 通过 API |
| Papers | 🔄 规划中 | 待实现 |

### MCP 服务器

#### 1. mcp-server-reading (文献阅读)

**职责**: 文献检索、阅读、理解、标注

**核心工具**:
- `literature_search` - 多源文献检索
- `get_paper_fulltext` - 获取文献全文
- `summarize_paper` - 生成结构化摘要
- `compare_papers` - 多篇文献对比分析
- `extract_key_entities` - 提取关键实体

#### 2. mcp-server-writing (学术写作)

**职责**: 文本生成、修改、评审

**核心工具**:
- `draft_section` - 根据大纲生成章节
- `improve_text` - 文本润色与改写
- `check_logic` - 逻辑连贯性检查
- `generate_references` - 参考文献生成
- `review_proposal` - 基金申请书评审

---

## 开发路线图

### 阶段总览

```
┌─────────────────────────────────────────────────────────────────┐
│                    XuanAgent 开发路线图                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  P0: 核心框架 (月 1-2)                                          │
│  ├── Sprint 1: 项目初始化 + Rig 集成                             │
│  └── Sprint 2: MCP Host + 文献管理集成                          │
│                                                                 │
│  P1: 文献智能处理 (月 3-5) ★ 当前重点                           │
│  ├── Sprint 3: SurrealDB 集成 + 向量存储                         │
│  ├── Sprint 4: 文献阅读 MCP 开发                                │
│  ├── Sprint 5: 知识管理自动化                                   │
│  └── Sprint 5-2: 基础 Web 界面                                 │
│                                                                 │
│  P2: 学术写作辅助 (月 6-8)                                      │
│  ├── Sprint 6: 学术写作 MCP                                     │
│  └── Sprint 7: 写作工作流实现                                   │
│                                                                 │
│  P3: 生态与优化 (月 9-12)                                       │
│  ├── Sprint 8-9: 性能优化                                       │
│  └── Sprint 10-11: 文档与生态                                   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### P0 阶段: 核心框架 (月 1-2)

**目标**: 基础架构 + CLI 工具 + MCP Host

| Sprint | 时间 | 核心任务 | 交付物 |
|:-------|:-----|:---------|:-------|
| Sprint 1 | 第 1-2 周 | 项目初始化、Rig 集成、配置管理 | lib crate 框架、基础对话 |
| Sprint 2 | 第 3-4 周 | MCP Host 实现、文献管理集成 | v0.1.0 Alpha |

**验收标准**:
```bash
$ xuan-agent-cli chat "你好"
你好！我是 XuanAgent，你的科研助手。

$ xuan-agent-cli chat "帮我找关于 Rust 的论文"
为你找到了 3 篇相关文献：
1. "Rust Programming Language" (2023)
```

### P1 阶段: 文献智能处理 (月 3-5)

**目标**: SurrealDB 集成 + 文献阅读 MCP + 智能检索 + Web 界面

**详细计划**: 参见 `docs/plan-p1.md`

| Sprint | 时间 | 核心任务 | 交付物 |
|:-------|:-----|:---------|:-------|
| Sprint 3 | 第 5-6 周 | SurrealDB 集成、向量存储 | 文献存储、向量检索 |
| Sprint 4 | 第 7-8 周 | 文献阅读 MCP 开发 | PDF 解析、智能切片 |
| Sprint 5 | 第 9-10 周 | 知识管理自动化 | 自动标签、引用分析 |
| Sprint 5-2 | 第 10-11 周 | 基础 Web 界面 | Axum 服务器、聊天 UI |

**验收标准**:
```bash
$ xuan-agent-cli import ./paper.pdf
文献已保存: "Deep Learning for CV" (2024)
向量化完成，共 156 个切片

$ xuan-agent-cli search "深度学习"
找到 5 篇相关文献（相似度 > 0.75）

$ cargo run --bin xuan-agent-web
Server running at http://localhost:3000
```

### P2 阶段: 学术写作辅助 (月 6-8)

**目标**: 写作 MCP + 论文/基金撰写工具

### P3 阶段: 生态与优化 (月 9-12)

**目标**: 性能优化 + 文档 + SDK

---

## 数据模型设计

### SurrealDB Schema

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
DEFINE FIELD paper_id ON chunk TYPE string;
DEFINE FIELD content ON chunk TYPE string;
DEFINE FIELD chunk_index ON chunk TYPE int;
DEFINE FIELD embedding ON chunk TYPE array<float>;
DEFINE FIELD page_number ON chunk TYPE int;

-- 向量索引
DEFINE INDEX chunk_embedding_idx ON chunk VECTOR(embedding, 1536);
```

### Rust 数据模型

```rust
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

---

## CLI 工具命令

### 基础命令

```bash
# 对话
$ xuan-agent-cli chat "你好"

# 导入文献
$ xuan-agent-cli import ./paper.pdf

# 检索文献
$ xuan-agent-cli search "深度学习"

# 文献摘要
$ xuan-agent-cli summarize paper_id_123

# 自动标签
$ xuan-agent-cli tag paper_id_123

# 引用网络
$ xuan-agent-cli citation paper_id_123

# 数据库初始化
$ xuan-agent-cli db init
```

### 高级选项

```bash
# 调试模式
$ xuan-agent-cli --debug chat "测试消息"

# 会话管理
$ xuan-agent-cli --session my-research chat "继续"

# 指定文献库路径
$ xuan-agent-cli --library-path ~/Zotero chat "搜索"
```

---

## 代码规范

### 命名约定

| 类型 | 约定 | 示例 |
|:-----|:-----|:-----|
| 结构体 | PascalCase | `XuanAgent`, `SurrealDBStorage` |
| 函数 | snake_case | `search_papers`, `get_llm_config` |
| 常量 | SCREAMING_SNAKE_CASE | `MAX_CHUNK_SIZE` |
| 模块 | snake_case | `mod surrealdb`, `mod embedding` |

### 错误处理

```rust
// 使用 thiserror 定义错误类型
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("配置错误: {0}")]
    Config(String),

    #[error("MCP 错误: {0}")]
    Mcp(String),

    #[error("存储错误: {0}")]
    Storage(String),

    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
```

### 测试规范

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_agent_creation() {
        let config = Config::from_env().unwrap();
        let agent = XuanAgent::new(config).await.unwrap();
        assert!(agent.is_ready());
    }

    #[tokio::test]
    async fn test_paper_storage() {
        let storage = SurrealDBStorage::connect("memory").await.unwrap();
        let paper = Paper::new_test();
        storage.store_paper(paper).await.unwrap();
    }
}
```

---

## 性能指标

### 检索性能

| 指标 | 目标值 |
|:-----|:-------|
| 向量检索响应时间 | < 2 秒 (1000 篇文献) |
| 关键词检索响应时间 | < 1 秒 |
| PDF 解析时间 | < 5 秒 (20 页) |
| 自动标签生成 | < 10 秒 |
| Web 界面响应 | < 500ms |

### 存储性能

| 指标 | 目标值 |
|:-----|:-------|
| 单用户文献容量 | 100,000+ 篇 |
| 向量维度 | 1536 (OpenAI) |
| 切片大小 | 500-1000 字 |
| 重叠率 | 10-20% |

---

## 开发环境准备

### 必需软件

```bash
# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustc --version  # >= 1.70

# Node.js (用于运行 MCP 服务器)
nvm install 18
nvm use 18

# SurrealDB
docker run -d -p 8000:8000 \
  -e SURREALDB_USER=root \
  -e SURREALDB_PASS=secret \
  surrealdb/surrealdb:latest \
  start --log trace --user root --pass secret
```

### 项目初始化

```bash
# 1. 克隆项目
git clone https://github.com/your-org/xuan-agent.git
cd xuan-agent

# 2. 复制配置文件
cp .env.example .env
# 编辑 .env，填入 API keys

# 3. 验证编译
cargo build
cargo test

# 4. 运行 CLI
cargo run --bin xuan-agent-cli
```

---

## 参考文档

### 项目文档

| 文档 | 说明 |
|:-----|:-----|
| `docs/requirements-specification.md` | 完整需求说明 |
| `docs/implement-plan.md` | 详细实施计划 |
| `docs/plan-p0.md` | P0 阶段详细计划 |
| `docs/plan-p1.md` | P1 阶段详细计划（文献智能处理） |

### 外部参考

| 资源 | 链接 |
|:-----|:-----|
| Rig 框架 | https://rig.rs/docs |
| MCP 协议 | https://modelcontextprotocol.io/specification |
| SurrealDB | https://surrealdb.com/docs |
| Axum | https://docs.rs/axum |

---

## 下一步行动

### 当前阶段: P0-Sprint 1

1. 初始化 workspace 结构
2. 创建包含核心类型的 lib crate
3. 实现 Config 支持 JSON .env
4. 集成 Rig 进行 LLM 调用
5. 构建基础 CLI 工具
6. 使用真实 AI Provider 测试

### 下阶段准备: P1

- 安装 SurrealDB
- 准备 PDF 测试文献
- 研究 pdfium 库文档

---

**最后更新**: 2025-03-01
**项目阶段**: P0 - 核心框架
**当前重点**: 核心 lib + CLI + MCP Host
