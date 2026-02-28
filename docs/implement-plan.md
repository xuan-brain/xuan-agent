# XuanAgent 开发实施计划

根据《科研AI Agent（XuanAgent）需求说明书》，制定以下详细开发计划。本计划遵循"**小步快跑、增量交付、优先复用**"的原则，专注于 Agent 后端和 MCP 服务器的核心功能实现。

---

## 一、项目总体规划

### 1.1 项目定位

**XuanAgent 是一个 Rust 库（lib crate）**，可以嵌入到其他软件项目中，为科研人员提供 AI 辅助的文献管理和学术写作能力。

**核心特点**：
- 📚 **可嵌入**：作为 lib crate 集成到其他应用程序
- 🚀 **高性能**：基于 Rust + Rig 框架，内存安全且高效
- 🔌 **标准化**：采用 MCP 协议，生态兼容性强
- 💾 **一体化存储**：使用 SurrealDB 统一管理向量和结构化数据

### 1.2 开发方法论

- **模型**：采用 **Scrum 敏捷开发**模式，以2周为一个 Sprint（迭代周期）
- **优先级原则**：
  1. **MCP First**：优先集成成熟开源 MCP 服务器（如文献管理 MCP，以 Zotero 为示例）
  2. **Core Stability**：优先确保核心 lib 的稳定与性能
  3. **API First**：提供清晰的 API，便于其他项目集成

### 1.3 里程碑概览

| 阶段 | 时间跨度 | 核心目标 | 关键交付物 |
| :--- | :--- | :--- | :--- |
| **P0: 核心框架** | 第1-2月 | 搭建 lib 核心架构，实现基础对话能力 | lib crate、MCP Host、CLI 工具 |
| **P1: 文献智能处理** | 第3-5月 | 实现文献深度理解与知识库 | 文献阅读 MCP、SurrealDB 集成、向量检索、基础 Web 界面 |
| **P2: 学术写作辅助** | 第6-8月 | 实现从文献到写作的流程 | 写作 MCP、论文/基金撰写工具、完善 Web 界面 |
| **P3: 生态与优化** | 第9-12月 | 性能优化与开发者生态 | 性能优化、完整文档、SDK、Web 界面优化 |

---

## 二、项目结构设计

### 2.1 Workspace 结构

```
xuan-agent/
├── Cargo.toml                 # Workspace 配置
├── .env                       # AI Provider 配置（测试用）
├── .env.example               # 配置模板
│
├── crates/
│   ├── xuan-agent/           # 核心 lib crate
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs        # 库入口
│   │       ├── agent/        # Agent 核心逻辑
│   │       ├── mcp/          # MCP 协议实现
│   │       ├── rag/          # RAG 检索增强
│   │       ├── storage/      # SurrealDB 集成
│   │       └── integrations/ # 第三方集成
│   │
│   ├── xuan-agent-cli/       # CLI 应用（bin crate）
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── main.rs       # 命令行入口
│   │
│   ├── xuan-agent-web/       # Web 应用（bin crate，优先级低于 cli）
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── main.rs       # Web 服务器入口
│   │
│   ├── mcp-host/             # MCP Host 实现
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs
│   │
│   └── mcp-servers/          # MCP Server 实现
│       ├── mcp-server-reading/
│       └── mcp-server-writing/
│
├── tests/                    # 集成测试
├── docs/                     # 文档
└── examples/                 # 使用示例
```

### 2.2 核心 Crate 说明

#### xuan-agent (lib)
- **职责**：核心库，提供 Agent 的所有功能
- **特点**：可嵌入到其他项目中
- **API**：提供清晰的公共 API
- **依赖**：最小化外部依赖

#### xuan-agent-cli (bin)
- **职责**：独立的命令行工具
- **用途**：开发测试、独立运行、演示
- **依赖**：依赖 xuan-agent lib
- **优先级**：高

#### xuan-agent-web (bin)
- **职责**：独立的 Web 应用
- **用途**：提供 Web 界面，可视化交互
- **依赖**：依赖 xuan-agent lib
- **优先级**：中（低于 CLI）
- **技术栈**：Axum + 简单的 HTML/CSS/JavaScript

---

## 三、详细迭代执行计划

### 第一阶段：P0 核心框架（第1-2个月）

**目标**：搭建 lib 核心架构，实现基础对话能力，提供 CLI 工具

#### Sprint 1 (Week 1-2): 项目初始化与核心架构

**任务**：
- [ ] 初始化 Rust workspace
- [ ] 创建 `xuan-agent` lib crate
- [ ] 创建 `xuan-agent-cli` bin crate
- [ ] 配置 `.env` 文件支持
- [ ] 集成 Rig 框架

**核心功能**：
```rust
// xuan-agent/src/lib.rs
pub struct XuanAgent {
    // Agent 核心结构
}

impl XuanAgent {
    pub fn new(config: Config) -> Result<Self>;
    pub async fn chat(&mut self, message: &str) -> Result<String>;
    pub async fn add_mcp_server(&mut self, server: Box<dyn McpServer>);
}
```

**配置管理**：
```rust
// 从 JSON 格式的 .env 读取配置
use serde::Deserialize;
use std::fs;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub ai_provider: AiProviderConfig,
    pub db: DbConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AiProviderConfig {
    pub provider: String,
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DbConfig {
    pub connect: String,
    pub user: String,
    pub pass: String,
    pub namespace: String,
    pub database: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        // 读取 JSON 格式的 .env 文件
        let content = fs::read_to_string(".env")
            .map_err(|e| Error::Config(format!("Failed to read .env: {}", e)))?;
        
        let config: Config = serde_json::from_str(&content)
            .map_err(|e| Error::Config(format!("Failed to parse .env: {}", e)))?;
        
        Ok(config)
    }
    
    pub fn get_llm_config(&self) -> (&str, &str, &str) {
        (&self.ai_provider.provider, &self.ai_provider.base_url, &self.ai_provider.model)
    }
    
    pub fn get_db_url(&self) -> String {
        format!("{}/db/{}", self.db.connect, self.db.database)
    }
}
```

**交付物**：
- ✅ 可编译的 workspace 结构
- ✅ 基础的 lib crate 框架
- ✅ 简单的 CLI 工具（能输出 "Hello, XuanAgent!"）
- ✅ `.env.example` 配置模板

#### Sprint 2 (Week 3-4): MCP Host 实现

**任务**：
- [ ] 实现 MCP Host 基础协议（JSON-RPC 2.0）
- [ ] 实现进程间通信（Stdio transport）
- [ ] 集成开源文献管理 MCP（以 Zotero MCP 为示例）
- [ ] 实现基础的 Agent 对话能力

**核心功能**：
```rust
// mcp-host/src/lib.rs
pub struct McpHost {
    servers: HashMap<String, Box<dyn McpServer>>,
}

impl McpHost {
    pub async fn start_server(&mut self, name: &str, command: &str);
    pub async fn call_tool(&self, server: &str, tool: &str, params: Value) -> Result<Value>;
}
```

**CLI 工具**：
```bash
# 基础对话
$ xuan-agent-cli chat "你好"

# 连接文献管理工具（以 Zotero 为例）
$ xuan-agent-cli --library-path ~/Zotero chat "帮我找关于 Rust 的论文"
```

**说明**：Zotero 仅作为开发测试的示例实现，XuanAgent 可以与多种文献管理工具（如 EndNote、Mendeley、Papers 等）集成。

**交付物**：
- ✅ **v0.1.0 Alpha**: 能与文献管理 MCP 通信的 CLI 工具（以 Zotero 为示例）
- ✅ 基础的对话能力
- ✅ MCP Host 框架

---

### 第二阶段：P1 文献智能处理（第3-5个月）

**目标**：集成 SurrealDB，实现文献深度理解与向量检索

#### Sprint 3 (Week 5-6): SurrealDB 集成与向量存储

**任务**：
- [ ] 集成 SurrealDB Rust SDK
- [ ] 设计数据库 Schema
- [ ] 实现文献元数据存储
- [ ] 实现向量嵌入存储

**数据库设计**：
```sql
-- SurrealDB Schema
DEFINE TABLE paper;
DEFINE FIELD title ON paper TYPE string;
DEFINE FIELD abstract ON paper TYPE string;
DEFINE FIELD embedding ON paper TYPE array<float>;
DEFINE FIELD metadata ON paper TYPE object;
DEFINE INDEX paper_embedding_idx ON paper VECTOR(embedding, 1536);
```

**核心功能**：
```rust
// xuan-agent/src/storage/surrealdb.rs
pub struct SurrealDBStorage {
    db: Surreal<Any>,
}

impl SurrealDBStorage {
    pub async fn store_paper(&self, paper: &Paper) -> Result<()>;
    pub async fn search_similar(&self, embedding: &[f32], limit: usize) -> Result<Vec<Paper>>;
}
```

**使用配置连接**：
```rust
// 从配置创建存储实例
let config = Config::from_env()?;
let storage = SurrealDBStorage::connect(&config.db).await?;
```

**JSON 配置示例**：
```json
{
  "db": {
    "connect": "http://127.0.0.1:8000",
    "user": "root",
    "pass": "secret",
    "namespace": "agent",
    "database": "dev"
  }
}
```

#### Sprint 4 (Week 7-8): 文献阅读 MCP 开发

**任务**：
- [ ] 开发 `mcp-server-reading`
- [ ] 实现 PDF 解析（使用 pdfium）
- [ ] 实现文献切片与向量化
- [ ] 实现语义检索

**MCP Tools**：
```rust
// mcp-servers/mcp-server-reading/src/lib.rs
pub fn summarize_paper(pdf_path: &str) -> Result<Summary>;
pub fn search_papers(query: &str, limit: usize) -> Result<Vec<Paper>>;
pub fn compare_papers(paper_ids: &[String]) -> Result<Comparison>;
```

#### Sprint 5 (Week 9-10): 知识管理自动化

**任务**：
- [ ] 实现自动标签系统
- [ ] 实现文献分类
- [ ] 实现引用关系分析
- [ ] 优化向量检索性能

#### Sprint 5-2 (Week 10-11): 基础 Web 界面

**任务**：
+- [ ] 创建 `xuan-agent-web` crate
+- [ ] 集成 Axum web 框架
+- [ ] 实现基础的聊天 API
+- [ ] 实现简单的 Web UI（HTML + JavaScript）

**Web API 设计**：
+```rust
+// xuan-agent-web/src/main.rs
+
+use axum::{Router, Json, extract::State};
+use serde::{Deserialize, Serialize};
+
+#[derive(Deserialize)]
+struct ChatRequest {
+    message: String,
+    session_id: Option<String>,
+}
+
+#[derive(Serialize)]
+struct ChatResponse {
+    response: String,
+    session_id: String,
+}
+
+async fn chat_handler(
+    State(agent): State<XuanAgent>,
+    Json(req): Json<ChatRequest>,
+) -> Json<ChatResponse> {
+    let response = agent.chat(&req.message).await.unwrap();
+    Json(ChatResponse {
+        response,
+        session_id: req.session_id.unwrap_or_default(),
+    })
+}
+
+#[tokio::main]
+async fn main() {
+    let config = Config::from_env().unwrap();
+    let agent = XuanAgent::new(config).await.unwrap();
+    
+    let app = Router::new()
+        .route("/api/chat", post(chat_handler))
+        .route("/", get(|| async { include_str!("../static/index.html") }))
+        .with_state(agent);
+    
+    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
+    axum::serve(listener, app).await.unwrap();
+}
+```

+**Web UI 示例**：
+```html
+<!-- xuan-agent-web/static/index.html -->
+<!DOCTYPE html>
+<html>
+<head>
+    <meta charset="UTF-8">
+    <title>XuanAgent Web</title>
+    <style>
+        body { font-family: Arial, sans-serif; max-width: 800px; margin: 0 auto; padding: 20px; }
+        #chat { height: 400px; border: 1px solid #ccc; overflow-y: scroll; padding: 10px; }
+        #input { width: 70%; padding: 10px; }
+        #send { width: 25%; padding: 10px; }
+    </style>
+</head>
+<body>
+    <h1>XuanAgent Web</h1>
+    <div id="chat"></div>
+    <input type="text" id="input" placeholder="输入消息...">
+    <button id="send">发送</button>
+    
+    <script>
+        const chat = document.getElementById('chat');
+        const input = document.getElementById('input');
+        const send = document.getElementById('send');
+        
+        send.onclick = async () => {
+            const message = input.value;
+            const response = await fetch('/api/chat', {
+                method: 'POST',
+                headers: {'Content-Type': 'application/json'},
+                body: JSON.stringify({message})
+            });
+            const data = await response.json();
+            chat.innerHTML += `<p><strong>You:</strong> ${message}</p>`;
+            chat.innerHTML += `<p><strong>Agent:</strong> ${data.response}</p>`;
+            input.value = '';
+        };
+    </script>
+</body>
+</html>
+```

+**使用方式**：
+```bash
+# 启动 Web 服务器
+$ cargo run --bin xuan-agent-web
+
+# 访问 http://localhost:3000
+```

+**交付物**：
+- ✅ 基础的 Web 服务器
+- ✅ 简单的聊天界面
+- ✅ REST API 接口

+---

**交付物**：
- ✅ **v0.5.0 Beta**: 完整的文献管理与检索能力
- ✅ SurrealDB 完整集成
- ✅ 文献阅读 MCP 稳定版本
- ✅ 基础的 Web 界面

---

### 第三阶段：P2 学术写作辅助（第6-8个月）

**目标**：实现学术写作 MCP，打通"读-写"链路

#### Sprint 6 (Week 11-12): 学术写作 MCP 核心

**任务**：
- [ ] 开发 `mcp-server-writing`
- [ ] 实现论文段落生成
- [ ] 实现文本润色
- [ ] 实现逻辑检查

**MCP Tools**：
```rust
// mcp-servers/mcp-server-writing/src/lib.rs
pub fn draft_section(topic: &str, references: &[String]) -> Result<String>;
pub fn improve_text(text: &str, style: &str) -> Result<String>;
pub fn check_logic(text: &str) -> Result<LogicCheck>;
```

#### Sprint 7 (Week 13-14): 写作工作流实现

**任务**：
- [ ] 实现基金申请书撰写流程
- [ ] 实现综述撰写流程
- [ ] 实现引用管理
- [ ] 实现格式化输出

**CLI 示例**：
```bash
# 基金申请书撰写
$ xuan-agent-cli write nsfc --topic "深度学习在药物发现中的应用"

# 综述撰写
$ xuan-agent-cli write review --papers paper1.pdf,paper2.pdf
```

#### Sprint 7-2 (Week 15-16): Web 界面增强

**任务**：
+- [ ] 添加文献列表展示
+- [ ] 添加文献详情页面
+- [ ] 优化聊天界面
+- [ ] 添加会话管理

**Web 功能**：
+```bash
+# 查看文献列表
+GET /api/papers?query=rust

+# 查看文献详情
+GET /api/papers/{id}

+# 搜索文献
+POST /api/search
+{"query": "深度学习", "limit": 10}
+```

+**交付物**：
+- ✅ 完善的 Web 界面
+- ✅ 文献管理功能
+- ✅ 会话管理功能

+---

**交付物**：
- ✅ **v0.8.0 RC**: 支持论文与基金撰写的完整功能
- ✅ 学术写作 MCP 稳定版本
- ✅ 完善的 Web 界面

---

### 第四阶段：P3 生态与优化（第9-12个月）

**目标**：性能优化、完善文档、构建开发者生态

#### Sprint 8-9 (Week 17-20): 性能优化

**任务**：
- [ ] PDF 解析性能优化（并发、零拷贝）
- [ ] 向量检索性能优化
- [ ] 内存使用优化
- [ ] 缓存策略实现

#### Sprint 10-11 (Week 21-24): 文档与生态

**任务**：
+- [ ] 编写完整的 API 文档
+- [ ] 编写开发者指南
+- [ ] 提供使用示例
+- [ ] 发布到 crates.io
+- [ ] 优化 Web 界面性能
+- [ ] 添加 Web 界面的响应式设计

**交付物**：
- ✅ **v1.0.0 Release**: 正式稳定版
- ✅ 完整的文档和示例
- ✅ 发布到 crates.io
- ✅ 优化的 Web 界面

---

## 四、技术架构要点

### 4.1 核心 API 设计

```rust
// xuan-agent/src/lib.rs

/// XuanAgent 核心库
/// 
/// # Example
/// 
/// ```rust
/// use xuan_agent::{XuanAgent, Config};
/// 
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     // 从 .env 加载配置
///     let config = Config::from_env()?;
///     
///     // 创建 Agent 实例
///     let mut agent = XuanAgent::new(config).await?;
///     
///     // 添加 MCP 服务器
///     // 添加文献管理 MCP 服务器（以 Zotero 为示例）
///     agent.add_mcp_server("library", ZoteroMcp::new()).await?;
///     
///     // 对话
///     let response = agent.chat("帮我找关于 Rust 的论文").await?;
///     println!("{}", response);
///     
///     Ok(())
/// }
/// ```

pub struct XuanAgent {
    config: Config,
    mcp_host: McpHost,
    storage: SurrealDBStorage,
    llm: Box<dyn LLMProvider>,
}

impl XuanAgent {
    /// 创建新的 Agent 实例
    pub async fn new(config: Config) -> Result<Self>;
    
    /// 添加 MCP 服务器
    pub async fn add_mcp_server(&mut self, name: &str, server: Box<dyn McpServer>);
    
    /// 对话
    pub async fn chat(&mut self, message: &str) -> Result<String>;
    
    /// 检索文献
    pub async fn search_papers(&self, query: &str, limit: usize) -> Result<Vec<Paper>>;
    
    /// 生成文本
    pub async fn generate_text(&self, prompt: &str, context: &str) -> Result<String>;
}
```

### 4.2 SurrealDB 集成

```rust
// xuan-agent/src/storage/surrealdb.rs

use surrealdb::{Surreal, Any};

pub struct SurrealDBStorage {
    db: Surreal<Any>,
}

impl SurrealDBStorage {
    /// 连接到 SurrealDB（使用配置）
    pub async fn connect(config: &DbConfig) -> Result<Self> {
        let db = Surreal::new<Any>(&config.connect).await?;
        db.use_ns(&config.namespace).use_db(&config.database).await?;
        Ok(Self { db })
    }
    
    /// 使用简单参数连接（向后兼容）
    pub async fn connect_simple(connect: &str, namespace: &str, database: &str) -> Result<Self> {
        let db = Surreal::new<Any>(connect).await?;
        db.use_ns(namespace).use_db(database).await?;
        Ok(Self { db })
    }
    
    /// 存储文献
    pub async fn store_paper(&self, paper: Paper) -> Result<()> {
        let _: Vec<Paper> = self.db
            .create("paper")
            .content(paper)
            .await?;
        Ok(())
    }
    
    /// 向量检索
    pub async fn search_similar(&self, embedding: &[f32], limit: usize) -> Result<Vec<Paper>> {
        let result: Vec<Paper> = self.db
            .query("SELECT * FROM paper WHERE embedding <|> $embedding < $limit")
            .bind(("embedding", embedding.to_vec()))
            .bind(("limit", limit as i64))
            .await?
            .take(0)?;
        Ok(result)
    }
}
```

### 4.3 .env 配置

项目使用 **JSON 格式**的 .env 文件进行配置，支持多种 AI Provider 和 SurrealDB 数据库配置。

#### 配置文件格式

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

#### 多种 AI Provider 配置示例

**OpenAI 配置**：
```json
{
  "ai_provider": {
    "provider": "openai",
    "base_url": "https://api.openai.com/v1",
    "api_key": "sk-xxxxx",
    "model": "gpt-4",
    "description": "OpenAI GPT-4 配置"
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

**Anthropic 配置**：
```json
{
  "ai_provider": {
    "provider": "anthropic",
    "base_url": "https://api.anthropic.com",
    "api_key": "sk-ant-xxxxx",
    "model": "claude-3-opus-20240229",
    "description": "Anthropic Claude 配置"
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

**本地模型（Ollama）配置**：
```json
{
  "ai_provider": {
    "provider": "ollama",
    "base_url": "http://localhost:11434",
    "api_key": "",
    "model": "llama2",
    "description": "本地 Ollama 模型配置"
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

#### SurrealDB 配置说明

- **connect**: SurrealDB 服务器地址（如 `http://127.0.0.1:8000`）
- **user**: 数据库用户名
- **pass**: 数据库密码
- **namespace**: 命名空间（用于隔离不同环境）
- **database**: 数据库名称

**不同环境的配置**：

开发环境：
```json
{
  "db": {
    "connect": "http://127.0.0.1:8000",
    "user": "root",
    "pass": "secret",
    "namespace": "agent",
    "database": "dev"
  }
}
```

生产环境：
```json
{
  "db": {
    "connect": "https://surreal.example.com:8000",
    "user": "prod_user",
    "pass": "secure_password",
    "namespace": "agent",
    "database": "production"
  }
}
```

#### 配置加载

配置会在应用启动时自动从 `.env` 文件加载：

```rust
// 加载配置
let config = Config::from_env()?;

// 使用配置
println!("AI Provider: {}", config.ai_provider.provider);
println!("Database: {}", config.db.database);
```

#### 安全建议

1. **不要提交 .env 文件**到版本控制系统
2. 使用 `.env.example` 作为配置模板
3. 生产环境使用强密码和安全的 API Key
4. 定期更换 API Key 和数据库密码

---

## 五、关键技术难点

### 5.1 PDF 解析质量

**挑战**：PDF 格式复杂，简单文本提取会丢失结构信息

**解决方案**：
- 使用 `pdfium` 提取布局信息
- 保留段落、标题、表格等结构
- 实现智能切片算法

**预研时间**：P0 阶段

### 5.2 向量检索性能

**挑战**：大规模文献库的向量检索效率

**解决方案**：
- 利用 SurrealDB 的向量索引
- 实现混合检索（关键词 + 向量）
- 引入 Re-rank 模型

**预研时间**：P1 阶段

### 5.3 写作连贯性

**挑战**：生成的文本需要逻辑连贯

**解决方案**：
- 在写作 MCP 中维护文档上下文
- 实现多轮对话管理
- 引入逻辑检查机制

**预研时间**：P2 阶段

---

## 六、资源配置

### 6.1 团队角色（最小配置）

- **Rust 后端开发 x 2**：负责核心框架、MCP 开发
- **算法工程师 x 1**（兼职）：负责 RAG 调优、Prompt Engineering

### 6.2 技术选型确认

| 技术组件 | 选型方案 | 选型理由 |
|----------|----------|----------|
| **核心框架** | Rust + Rig | 高性能、内存安全，Rig 提供 LLM 统一抽象 |
| **协议层** | MCP | 官方标准协议，生态丰富 |
| **数据库** | SurrealDB | 支持向量索引，多模态存储，嵌入式部署 |
| **PDF 引擎** | pdfium | 高性能 PDF 渲染与文本提取 |
| **LLM 接口** | Rig | 统一的 LLM 接口，支持多种 Provider |
| **CLI 框架** | clap | Rust 生态标准的 CLI 框架 |

---

## 七、测试策略

### 7.1 单元测试

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

### 7.2 集成测试

```rust
// tests/integration_test.rs

#[tokio::test]
async fn test_full_workflow() {
    // 加载 .env 配置
    let config = Config::from_env().expect("Failed to load .env");
    
    // 创建 Agent
    let mut agent = XuanAgent::new(config).await.expect("Failed to create agent");
    
    // 测试对话
    let response = agent.chat("测试消息").await.expect("Chat failed");
    assert!(!response.is_empty());
    
    // 测试文献检索
    let papers = agent.search_papers("test", 5).await.expect("Search failed");
    assert!(papers.len() <= 5);
}
```

### 7.3 性能测试

```rust
#[bench]
fn bench_vector_search(b: &mut test::Bencher) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let storage = runtime.block_on(SurrealDBStorage::connect("memory")).unwrap();
    
    b.iter(|| {
        runtime.block_on(storage.search_similar(&[0.0; 1536], 10))
    });
}
```

---

## 八、风险监控与应对

### 8.1 MCP 协议变动风险

**监控**：
- 订阅 MCP 官方 GitHub 更新日志
- 定期检查协议版本兼容性

**应对**：
- 在 `mcp-host` 层做好抽象封装
- 保持协议版本的向后兼容

### 8.2 LLM 输出不稳定

**监控**：
- 建立测试集，定期测试生成质量
- 监控 API 调用成功率和延迟

**应对**：
- 优化 Prompt 工程
- 实现重试机制
- 提供本地模型备选方案

### 8.3 SurrealDB 性能问题

**监控**：
- 监控数据库查询性能
- 定期进行压力测试

**应对**：
- 优化索引策略
- 实现查询缓存
- 考虑数据分片

---

## 九、下一步行动

### 立即启动 P0-Sprint 1

1. **初始化项目**：
   ```bash
   cargo new xuan-agent --lib
   cd xuan-agent
   cargo new --lib crates/xuan-agent
   cargo new crates/xuan-agent-cli
   ```

3. **配置 workspace**：
   ```toml
   # Cargo.toml
   [workspace]
   members = [
       "crates/xuan-agent",
       "crates/xuan-agent-cli",
       "crates/xuan-agent-web",  # Web 界面（优先级较低）
   ]
   ```

3. **创建 .env 文件**：
   ```bash
   cp .env.example .env
   # 编辑 .env，填入 API keys
   ```

4. **实现第一个 Demo**：
   ```
   User -> CLI -> XuanAgent lib -> "Hello, XuanAgent!"
   ```

---

**预期成果**：在 2 周内完成基础框架搭建，能够运行简单的对话 Demo，为后续功能开发奠定坚实基础。