# P0 阶段实施计划：核心框架搭建

根据《XuanAgent 开发实施计划》，本文档详细规划 P0 阶段（第1-2个月）的实施细节。本阶段的核心目标是**搭建 lib 核心架构，实现基础对话能力，提供 CLI 工具**。

---

## 一、阶段目标与验收标准

### 核心目标

搭建基于 Rust + Rig 的 **XuanAgent 库**，实现 MCP Host 功能，成功集成开源的文献管理 MCP（以 Zotero MCP 为示例），完成基础的对话能力演示。

**说明**：XuanAgent 是通用的科研助手，可以与多种文献管理工具集成。Zotero 仅作为开发测试的示例实现，用户可以根据需要集成其他文献管理工具（如 EndNote、Mendeley、Papers 等）。

**关键特点**：
- 📚 **核心是 lib**：可以嵌入到其他软件项目中
- 🛠️ **提供 CLI 工具**：用于开发测试和独立运行
- 💾 **统一配置管理**：通过 .env 文件配置 AI Provider
- 🔌 **MCP 协议支持**：实现标准的 MCP Host

### 验收标准

#### 1. 环境验收
- ✅ 能够编译整个 workspace（lib + cli）
- ✅ 能够运行 CLI 工具并进行基础对话
- ✅ 能够通过 .env 文件配置 AI Provider
- ✅ 能够启动并连接文献管理 MCP 服务（以 Zotero 为示例）

#### 2. 功能验收
- ✅ **基础对话**：
  ```
  $ xuan-agent-cli chat "你好"
  你好！我是 XuanAgent，你的科研助手。
  ```
  
- ✅ **文献检索**：
  ```
  $ xuan-agent-cli chat "帮我找关于 Rust 的论文"
  我为你找到了 3 篇相关文献...
  ```

#### 3. 代码验收
- ✅ lib crate 通过单元测试
- ✅ CLI 工具能够正常运行
- ✅ 配置加载正确（从 .env 文件）
- ✅ MCP Host 能够连接外部 MCP 服务器

---

## 二、Sprint 1 详细计划（第 1-2 周）：项目初始化与核心架构

### 目标
完成后端项目初始化，搭建 workspace 结构，实现 Rig Agent 基础对话能力，配置 .env 支持。

### 1. 任务分解

| 任务ID | 任务描述 | 负责模块 | 关键产出 | 预计时间 |
| :--- | :--- | :--- | :--- | :--- |
| **T1.1** | **Rust Workspace 初始化** | Project Setup | `Cargo.toml` (workspace), 目录结构 | 0.5 天 |
| **T1.2** | **创建 xuan-agent lib crate** | `xuan-agent` | `lib.rs`, 基础结构体定义 | 1 天 |
| **T1.3** | **创建 xuan-agent-cli bin crate** | `xuan-agent-cli` | `main.rs`, 基础 CLI 框架 | 0.5 天 |
| **T1.4** | **集成 Rig 框架** | `xuan-agent` | LLM Provider 集成，基础对话能力 | 2 天 |
| **T1.5** | **配置管理实现** | `xuan-agent` | `.env` 文件读取，`Config` 结构体 | 1 天 |
| **T1.6** | **编写单元测试** | `xuan-agent` | 基础功能的单元测试 | 1 天 |

### 2. 技术实现细节

#### 目录结构设计
```text
xuan-agent/
├── Cargo.toml                 # Workspace 配置
├── .env                       # AI Provider 配置（测试用）
├── .env.example               # 配置模板
├── .gitignore
│
├── crates/
│   ├── xuan-agent/           # 核心 lib crate
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs        # 库入口
│   │       ├── agent.rs      # Agent 核心逻辑
│   │       ├── config.rs     # 配置管理
│   │       └── error.rs      # 错误定义
│   │
│   └── xuan-agent-cli/       # CLI 应用（bin crate）
│       ├── Cargo.toml
│       └── src/
│           └── main.rs       # 命令行入口
│
├── tests/                    # 集成测试
│   └── integration_test.rs
│
└── docs/                     # 文档
    ├── README.md
    └── plan-p0.md
```

#### Workspace 配置
```toml
# Cargo.toml
[workspace]
members = [
    "crates/xuan-agent",
    "crates/xuan-agent-cli",
]
resolver = "2"

[workspace.dependencies]
rig = "0.5"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
anyhow = "1"
thiserror = "1"
dotenvy = "0.15"
tracing = "0.1"
tracing-subscriber = "0.3"
```

#### 核心 lib API 设计
```rust
// crates/xuan-agent/src/lib.rs

//! XuanAgent - 科研 AI Agent 库
//!
//! # Example
//!
//! ```rust
//! use xuan_agent::{XuanAgent, Config};
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     // 从 .env 加载配置
//!     let config = Config::from_env()?;
//!     
//!     // 创建 Agent 实例
//!     let mut agent = XuanAgent::new(config).await?;
//!     
//!     // 对话
//!     let response = agent.chat("你好").await?;
//!     println!("{}", response);
//!     
//!     Ok(())
//! }
//! ```

pub mod agent;
pub mod config;
pub mod error;

pub use agent::XuanAgent;
pub use config::Config;
pub use error::{Result, Error};
```

#### Agent 核心结构
```rust
// crates/xuan-agent/src/agent.rs

use crate::{Config, Result};
use rig::{completion::Chat, providers::openai};

/// XuanAgent 核心结构
pub struct XuanAgent {
    config: Config,
    llm: Box<dyn Chat>,
}

impl XuanAgent {
    /// 创建新的 Agent 实例
    pub async fn new(config: Config) -> Result<Self> {
        // 初始化 LLM Provider
        let llm = match &config.openai_api_key {
            Some(key) => {
                openai::Client::new(key)
                    .agent(config.openai_model.as_deref().unwrap_or("gpt-4"))
            }
            None => return Err(Error::Config("OpenAI API key not found".into())),
        };
        
        Ok(Self { config, llm: Box::new(llm) })
    }
    
    /// 对话
    pub async fn chat(&mut self, message: &str) -> Result<String> {
        // 实现基础对话逻辑
        let response = self.llm.chat(message, vec![]).await?;
        Ok(response)
    }
    
    /// 检查 Agent 是否就绪
    pub fn is_ready(&self) -> bool {
        true
    }
}
```

#### 配置管理
```rust
// crates/xuan-agent/src/config.rs

use serde::Deserialize;
use std::fs;

/// XuanAgent 配置
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// AI Provider 配置
    pub ai_provider: AiProviderConfig,
    
    /// 数据库配置
    pub db: DbConfig,
}

/// AI Provider 配置
#[derive(Debug, Clone, Deserialize)]
pub struct AiProviderConfig {
    pub provider: String,
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub description: Option<String>,
}

/// 数据库配置
#[derive(Debug, Clone, Deserialize)]
pub struct DbConfig {
    pub connect: String,
    pub user: String,
    pub pass: String,
    pub namespace: String,
    pub database: String,
}

impl Config {
    /// 从 JSON 格式的 .env 文件加载配置
    pub fn from_env() -> crate::Result<Self> {
        // 读取 JSON 格式的 .env 文件
        let content = fs::read_to_string(".env")
            .map_err(|e| crate::Error::Config(format!("Failed to read .env: {}", e)))?;
        
        let config: Config = serde_json::from_str(&content)
            .map_err(|e| crate::Error::Config(format!("Failed to parse .env: {}", e)))?;
        
        Ok(config)
    }
    
    /// 获取 LLM 配置
    pub fn get_llm_config(&self) -> (&str, &str, &str) {
        (&self.ai_provider.provider, &self.ai_provider.base_url, &self.ai_provider.model)
    }
    
    /// 获取数据库连接 URL
    pub fn get_db_url(&self) -> String {
        format!("{}/db/{}", self.db.connect, self.db.database)
    }
}
```

#### CLI 工具实现
```rust
// crates/xuan-agent-cli/src/main.rs

use xuan_agent::{XuanAgent, Config};
use std::io::{self, BufRead, Write};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();
    
    // 加载配置
    let config = Config::from_env()?;
    
    // 创建 Agent
    let mut agent = XuanAgent::new(config).await?;
    
    println!("XuanAgent CLI v0.1.0");
    println!("输入 'quit' 退出，输入 'help' 查看帮助\n");
    
    // 交互式对话循环
    let stdin = io::stdin();
    print!("> ");
    io::stdout().flush()?;
    
    for line in stdin.lock().lines() {
        let input = line?;
        
        if input == "quit" {
            break;
        }
        
        if input == "help" {
            println!("命令:");
            println!("  quit  - 退出程序");
            println!("  help  - 显示帮助");
            println!("  其他  - 与 Agent 对话");
            print!("> ");
            io::stdout().flush()?;
            continue;
        }
        
        // 发送消息给 Agent
        match agent.chat(&input).await {
            Ok(response) => {
                println!("{}\n", response);
            }
            Err(e) => {
                eprintln!("错误: {}\n", e);
            }
        }
        
        print!("> ");
        io::stdout().flush()?;
    }
    
    println!("再见！");
    Ok(())
}
```

### 3. .env 配置文件

项目使用 **JSON 格式**的 .env 文件进行配置。

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

**其他 AI Provider 配置示例**：

OpenAI：
```json
{
  "ai_provider": {
    "provider": "openai",
    "base_url": "https://api.openai.com/v1",
    "api_key": "sk-xxxxx",
    "model": "gpt-4"
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

Anthropic：
```json
{
  "ai_provider": {
    "provider": "anthropic",
    "base_url": "https://api.anthropic.com",
    "api_key": "sk-ant-xxxxx",
    "model": "claude-3-opus-20240229"
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

本地模型（Ollama）：
```json
{
  "ai_provider": {
    "provider": "ollama",
    "base_url": "http://localhost:11434",
    "api_key": "",
    "model": "llama2"
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

### 4. 交付物

- ✅ 可编译的 workspace 结构
- ✅ 基础的 lib crate 框架
- ✅ 简单的 CLI 工具（能输出 "Hello, XuanAgent!"）
- ✅ `.env.example` 配置模板
- ✅ 基础的单元测试
- ✅ 项目 README 文档

---

## 三、Sprint 2 详细计划（第 3-4 周）：MCP Host 实现

### 目标
实现 MCP Host 基础协议，集成文献管理 MCP（以 Zotero MCP 为示例），实现 Agent 工具调用，完成端到端的文献检索演示。

### 1. 任务分解

| 任务ID | 任务描述 | 负责模块 | 关键产出 | 预计时间 |
| :--- | :--- | :--- | :--- | :--- |
| **T2.1** | **MCP Host 协议实现** | `mcp-host` | JSON-RPC 2.0 消息解析 | 2 天 |
| **T2.2** | **Stdio 传输层实现** | `mcp-host` | 进程间通信实现 | 1 天 |
| **T2.3** | **集成文献管理 MCP** | `integrations` | 启动和连接文献管理 MCP（以 Zotero 为示例） | 1 天 |
| **T2.4** | **Rig Tool 适配器** | `xuan-agent` | MCP Tool 到 Rig Tool 的映射 | 2 天 |
| **T2.5** | **意图识别优化** | `xuan-agent` | System Prompt 优化 | 1 天 |
| **T2.6** | **集成测试** | `tests` | 端到端测试 | 1 天 |

### 2. 技术实现细节

#### MCP Host 结构
```rust
// crates/mcp-host/src/lib.rs

use serde_json::Value;
use std::collections::HashMap;
use tokio::process::{Child, Command};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// MCP Host - 管理多个 MCP Server 的连接
pub struct McpHost {
    servers: HashMap<String, McpServerConnection>,
}

/// MCP Server 连接
struct McpServerConnection {
    name: String,
    process: Child,
}

impl McpHost {
    pub fn new() -> Self {
        Self {
            servers: HashMap::new(),
        }
    }
    
    /// 启动 MCP Server
    pub async fn start_server(&mut self, name: &str, command: &str) -> Result<()> {
        let mut child = Command::new(command)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        
        let server = McpServerConnection {
            name: name.to_string(),
            process: child,
        };
        
        self.servers.insert(name.to_string(), server);
        Ok(())
    }
    
    /// 调用 MCP Tool
    pub async fn call_tool(
        &mut self,
        server: &str,
        tool: &str,
        params: Value,
    ) -> Result<Value> {
        let server_conn = self.servers.get_mut(server)
            .ok_or(Error::ServerNotFound)?;
        
        // 构造 JSON-RPC 请求
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "tools/call".to_string(),
            params: json!({
                "name": tool,
                "arguments": params,
            }),
            id: 1,
        };
        
        // 发送请求并接收响应
        let response = server_conn.send_request(request).await?;
        Ok(response.result)
    }
}
```

#### Rig Tool 适配器
```rust
// crates/xuan-agent/src/tools/mcp_adapter.rs

use rig::completion::Tool;
use serde_json::Value;
use std::sync::Arc;

/// MCP Tool 适配器 - 将 MCP Tool 映射为 Rig Tool
pub struct McpToolAdapter {
    name: String,
    description: String,
    parameters: Value,
    mcp_host: Arc<tokio::sync::Mutex<McpHost>>,
    server_name: String,
}

impl McpToolAdapter {
    pub fn new(
        name: String,
        description: String,
        parameters: Value,
        mcp_host: Arc<tokio::sync::Mutex<McpHost>>,
        server_name: String,
    ) -> Self {
        Self {
            name,
            description,
            parameters,
            mcp_host,
            server_name,
        }
    }
}

impl Tool for McpToolAdapter {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    fn parameters(&self) -> Value {
        self.parameters.clone()
    }
    
    async fn call(&self, input: Value) -> Result<Value, rig::completion::ToolError> {
        let mut host = self.mcp_host.lock().await;
        host.call_tool(&self.server_name, &self.name, input)
            .await
            .map_err(|e| rig::completion::ToolError::CallError(e.to_string()))
    }
}
```

#### System Prompt 优化
```rust
// crates/xuan-agent/src/agent.rs

const SYSTEM_PROMPT: &str = r#"
你是一个专业的科研助手 XuanAgent。

你的职责是：
1. 帮助用户检索和管理学术文献
2. 辅助用户进行学术写作
3. 回答科研相关的问题

你拥有以下工具：
- library_search: 搜索文献库（支持 Zotero、EndNote、Mendeley 等多种文献管理工具）
- library_add: 添加新文献到文献库
- library_export: 导出文献列表

使用规则：
1. 当用户要求搜索文献时，使用 library_search 工具
2. 如果没有找到结果，明确告知用户，不要编造
3. 提供准确、有用的科研建议
4. 用专业但友好的语气回复

现在，请帮助用户解决他们的科研问题。
"#;
```

### 3. CLI 增强功能

```bash
# 文献检索
$ xuan-agent-cli chat "帮我找关于 Rust 的论文"
正在搜索文献库...
找到 3 篇相关文献：
1. "Rust Programming Language" (2023)
2. "Systems Programming in Rust" (2022)
3. "Memory Safety in Rust" (2021)

# 会话管理
$ xuan-agent-cli --session my-research chat "继续上次的话题"

# 调试模式
$ xuan-agent-cli --debug chat "测试消息"
[DEBUG] Loading config from .env
[DEBUG] Connecting to Library MCP
[DEBUG] Sending message to LLM
```

### 4. 交付物

- ✅ **v0.1.0 Alpha**: 能与文献管理 MCP 通信的 CLI 工具（以 Zotero 为示例）
- ✅ MCP Host 基础框架
- ✅ 基础的对话能力
- ✅ 工具调用能力
- ✅ 集成测试通过

---

## 四、技术风险与应对预案

| 风险点 | 可能的影响 | 应对预案 |
| :--- | :--- | :--- |
| **文献管理 MCP 兼容性** | 现有开源文献管理 MCP 可能与最新版本不兼容 | **Plan A**: 寻找 Fork 版本或替代方案；<br>**Plan B**: 自己实现 MCP Server，支持多种文献管理工具（Zotero、EndNote、Mendeley 等）。 |
| **Rig 框架限制** | Rig 对 Tool 的定义可能与 MCP Schema 有差异 | 在 `McpToolAdapter` 层做 Schema 转换映射。 |
| **LLM 输出不稳定** | Agent 在没有结果时编造文献 | Prompt 强制要求"若无搜索结果，必须明确告知，不可编造"，并在 Tool Result 中加入元数据校验。 |
| **配置管理复杂** | 多个 AI Provider 的配置管理复杂 | 提供清晰的 .env.example 模板，实现配置验证和友好的错误提示。 |
| **异步运行时问题** | Tokio 异步运行时与 MCP 进程通信的集成 | 使用 `tokio::process` 进行异步进程管理，确保正确的生命周期管理。 |

---

## 五、开发环境准备清单

### 必需软件

1. **Rust**: `rustup` (latest stable)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   rustc --version  # 确保版本 >= 1.70
   ```

2. **Node.js**: v18+ (用于运行文献管理 MCP，以 Zotero MCP 为示例)
   ```bash
   # 使用 nvm 安装
   nvm install 18
   nvm use 18
   node --version
   ```

3. **文献管理工具**: 桌面客户端（可选，以 Zotero 为示例）
   - Zotero: 安装 Better BibTeX 插件（推荐）
   - 或其他文献管理工具: EndNote、Mendeley、Papers 等
   - 记录文献管理工具的数据目录路径

4. **IDE**: VS Code (推荐)
   - 插件: rust-analyzer
   - 插件: CodeLLDB
   - 插件: Even Better TOML

### 项目初始化

```bash
# 1. 创建项目目录
mkdir xuan-agent
cd xuan-agent

# 2. 初始化 workspace
cat > Cargo.toml << EOF
[workspace]
members = [
    "crates/xuan-agent",
    "crates/xuan-agent-cli",
]
resolver = "2"
EOF

# 3. 创建 lib crate
cargo new --lib crates/xuan-agent

# 4. 创建 CLI crate
cargo new crates/xuan-agent-cli

# 5. 创建配置文件
cp .env.example .env
# 编辑 .env，填入你的 API keys

# 6. 验证编译
cargo build
cargo test
```

---

## 六、演示流程脚本

### Sprint 2 结束时的演示场景

#### 场景 1: 基础对话
```bash
$ cargo run --bin xuan-agent-cli
XuanAgent CLI v0.1.0
输入 'quit' 退出，输入 'help' 查看帮助

> 你好
你好！我是 XuanAgent，你的科研助手。我可以帮你：
- 检索和管理文献
- 辅助学术写作
- 回答科研问题

有什么我可以帮你的吗？

> quit
再见！
```

#### 场景 2: 文献检索
```bash
$ cargo run --bin xuan-agent-cli
XuanAgent CLI v0.1.0

> 帮我查一下关于'蛋白质折叠'的最新论文
正在搜索文献库...

我为你找到了 3 篇相关文献：

1. **AlphaFold2: Using AI to predict protein structure**
   - 作者: Jumper, J., et al.
   - 年份: 2021
   - 期刊: Nature
   
2. **Deep learning for protein folding**
   - 作者: Baek, M., et al.
   - 年份: 2021
   - 期刊: Science
   
3. **Computational methods for protein structure prediction**
   - 作者: Pearce, R., et al.
   - 年份: 2022
   - 期刊: Nature Methods

需要我详细介绍其中某篇文献吗？

> quit
再见！
```

#### 场景 3: 错误处理
```bash
$ cargo run --bin xuan-agent-cli
XuanAgent CLI v0.1.0

> 帮我找关于'不存在的主题xyz123'的论文
正在搜索文献库...

抱歉，我没有找到关于'不存在的主题xyz123'的相关文献。

建议：
1. 尝试使用更通用的关键词
2. 检查拼写是否正确
3. 确认文献库中是否有相关文献

> quit
再见！
```

---

## 七、下一步行动

### 立即启动 P0-Sprint 1

#### Day 1: 项目初始化
```bash
# 1. 创建项目
mkdir xuan-agent && cd xuan-agent

# 2. 初始化 workspace
cargo init --name xuan-agent

# 3. 创建 crates
cargo new --lib crates/xuan-agent
cargo new crates/xuan-agent-cli

# 4. 配置 workspace
# 编辑 Cargo.toml

# 5. 创建配置文件
cp .env.example .env

# 6. 验证编译
cargo build
```

#### Day 2-3: 核心库实现
- 实现 `XuanAgent` 结构体
- 集成 Rig 框架
- 实现配置管理

#### Day 4-5: CLI 工具实现
- 实现交互式 CLI
- 添加基础命令
- 实现对话循环

#### Day 6-7: 测试与文档
- 编写单元测试
- 编写 README
- 准备 Sprint 1 演示

### 成功标准
- ✅ 能够编译和运行
- ✅ 能够进行基础对话
- ✅ 配置管理正常工作
- ✅ 基础测试通过
- ✅ 文档完整

---

**预期成果**：在 2 周内完成基础框架搭建，能够运行简单的对话 Demo，为后续 MCP 集成和功能扩展奠定坚实基础。