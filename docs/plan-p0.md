这是针对 **P0 阶段（基础设施与原型）** 的详细实施计划。本阶段的核心目标是**跑通最小可行性闭环**：用户通过自然语言与 Agent 对话，Agent 能够调用本地 Zotero 文献库并返回准确的检索结果。
---
## P0 阶段实施计划：基础设施与原型
### 一、 阶段目标与验收标准
**核心目标**：
搭建基于 Rust + Rig 的 Agent 后端骨架，实现 MCP Host 功能，成功集成开源的 Zotero MCP，完成“对话即检索”的 MVP（最小可用产品）演示。
**验收标准**：
1.  **环境验收**：能够一键启动后端服务、前端界面以及 Zotero MCP 服务。
2.  **功能验收**：
    -   用户输入：“帮我找一下关于‘深度学习在药物发现中的应用’的论文”。
    -   Agent 成功识别意图，调用 Zotero MCP。
    -   返回包含标题、作者、年份的文献列表，并在前端展示。
3.  **代码验收**：核心模块通过单元测试，API 接口通过 Postman/cURL 测试。
---
### 二、 Sprint 1 详细计划（第 1-2 周）：核心骨架搭建
**目标**：完成后端项目初始化，实现 Rig Agent 基础对话能力，跑通 MCP 协议通信。
#### 1. 任务分解
| 任务ID | 任务描述 | 负责模块 | 关键产出 |
| :--- | :--- | :--- | :--- |
| **T1.1** | **Rust Workspace 初始化** | Project Setup | `Cargo.toml` (workspace), 目录结构 (`crates/core`, `crates/mcp-host` 等) |
| **T1.2** | **Rig Agent 核心封装** | `core-agent` | `AgentBuilder`, `ChatRequest`, `ChatResponse` 结构体，支持 OpenAI API 调用 |
| **T1.3** | **MCP Host 协议实现** | `mcp-host` | 实现 `McpClient` trait，封装 JSON-RPC 2.0 消息解析 |
| **T1.4** | **Stdio 传输层实现** | `mcp-host` | 实现通过标准输入/输出与子进程通信的能力（用于连接 Zotero MCP） |
| **T1.5** | **配置管理** | `config` | 支持 `.env` 文件读取 (API Keys, Zotero 路径等) |
#### 2. 技术实现细节
**目录结构设计**：
```text
sci-agent/
├── Cargo.toml
├── .env.example
├── crates/
│   ├── core-agent/         # Agent 核心逻辑
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   └── agent.rs    # Rig Agent 封装
│   │   └── Cargo.toml
│   └── mcp-host/           # MCP 客户端
│       ├── src/
│       │   ├── lib.rs
│       │   ├── protocol.rs # JSON-RPC 定义
│       │   └── transport.rs # 进程通信实现
│       └── Cargo.toml
└── src/
    └── main.rs             # 主入口
```
**MCP Host 关键逻辑 (Rust伪代码)**：
```rust
// crates/mcp-host/src/transport.rs
use tokio::process::{Child, Command};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
pub struct StdioTransport {
    process: Child,
}
impl StdioTransport {
    pub async fn start(command: &str) -> Self {
        let mut child = Command::new(command)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to start MCP server");
        Self { process: child }
    }
    pub async fn send_request(&mut self, request: JsonRpcRequest) -> JsonRpcResponse {
        // 序列化请求，写入 stdin，读取 stdout，反序列化响应
        // ... 具体实现
    }
}
```
---
### 三、 Sprint 2 详细计划（第 3-4 周）：文献检索闭环
**目标**：集成 Zotero MCP，实现 Agent 工具调用，开发简单前端界面，完成端到端演示。
#### 1. 任务分解
| 任务ID | 任务描述 | 负责模块 | 关键产出 |
| :--- | :--- | :--- | :--- |
| **T2.1** | **集成 Zotero MCP** | `integrations` | 启动 `zotero-mcp` 服务，配置自动发现 |
| **T2.2** | **Rig Tool 适配器开发** | `core-agent` | 实现 `McpToolWrapper`，将 MCP Tool 映射为 Rig Tool |
| **T2.3** | **意图识别与规划** | `core-agent` | 优化 System Prompt，让 Agent 知道何时调用 `zotero_search` |
| **T2.4** | **Web API 开发** | `api` | 使用 Axum 搭建后端，提供 `/api/chat` 接口 |
| **T2.5** | **前端 UI 开发** | `web` | 初始化 React 项目，实现最简聊天界面 |
#### 2. 技术实现细节
**Rig Tool 适配器设计**：
我们需要将 MCP Server 暴露的工具动态注册到 Rig Agent 中。
```rust
// crates/core-agent/src/tools/mcp_adapter.rs
use rig::completion::Tool;
use serde_json::Value;
pub struct McpToolAdapter {
    name: String,
    description: String,
    parameters: Value, // JSON Schema
    mcp_client: Arc<McpClient>,
}
impl Tool for McpToolAdapter {
    // 实现 Rig 的 call 方法，内部调用 mcp_client.call_tool
    async fn call(&self, input: Value) -> Result<Value, ToolError> {
        self.mcp_client.call_tool(&self.name, input).await
    }
}
```
**API 接口设计**：
- **POST** `/api/chat`
    - **Request**: `{ "message": "帮我找关于Rust的论文", "session_id": "..." }`
    - **Response**: 
        - **Stream (SSE)**: 为了后续体验，建议直接实现流式响应。
        - **Event Types**: 
            - `thought`: Agent 思考过程 ("正在检索 Zotero...")
            - `tool_call`: 显示调用的工具参数
            - `text`: 最终给用户的自然语言回复
            - `data`: 结构化的文献数据
**前端组件设计**：
- `App.tsx`: 主布局。
- `ChatWindow.tsx`: 消息列表 + 输入框。
- `MessageItem.tsx`: 渲染不同类型的消息（支持 Markdown 渲染文献卡片）。
---
### 四、 技术风险与应对预案
| 风险点 | 可能的影响 | 应对预案 |
| :--- | :--- | :--- |
| **Zotero MCP 兼容性** | 现有开源 Zotero MCP 可能与最新 Zotero 版本不兼容 | **Plan A**: 寻找 Fork 版本；<br>**Plan B**: 自己写一个极简的 MCP Server，直接读 Zotero 的 SQLite 数据库 (`zotero.sqlite`)。 |
| **Rig 框架限制** | Rig 对 Tool 的定义可能与 MCP Schema 有差异 | 在 `McpToolAdapter` 层做 Schema 转换映射。 |
| **LLM 幻觉** | Agent 在没有结果时编造文献 | Prompt 强制要求“若无搜索结果，必须明确告知，不可编造”，并在 Tool Result 中加入元数据校验。 |
---
### 五、 开发环境准备清单
**开发者机器需提前安装**：
1.  **Rust**: `rustup` (latest stable)
2.  **Node.js**: v18+ (用于运行 Zotero MCP 和前端)
3.  **Zotero**: 桌面客户端，并安装 Better BibTeX 插件（推荐）
4.  **IDE**: VS Code (推荐插件: rust-analyzer, CodeLLDB)
**项目配置文件**：
创建 `.env` 文件：
```ini
# LLM Configuration
OPENAI_API_KEY=sk-xxxxx
OPENAI_MODEL=gpt-4o
# Zotero MCP Configuration
ZOTERO_MCP_CMD=node /path/to/zotero-mcp/dist/index.js # 或 Python 命令
```
---
### 六、 演示流程脚本
**Sprint 2 结束时的演示场景**：
1.  **启动**：运行 `cargo run`，后端启动并自动拉起 Zotero MCP 进程。
2.  **交互**：
    - 用户在前端输入：“*你好，我是研究计算生物学的。*”
    - Agent 回复：“*你好！我是你的科研助手。我可以帮你检索文献或辅助写作。*”
    - 用户输入：“*帮我查一下我库里关于‘蛋白质折叠’的最新论文，只要是 2023 年以后的。*”
3.  **后台逻辑**：
    - Agent 分析意图 -> 决定调用 `zotero_search` -> 构造参数 `{"query": "蛋白质折叠", "year": ">2023"}`。
    - MCP Host 发送 JSON-RPC 请求给 Zotero MCP。
    - Zotero MCP 返回 5 条结果。
4.  **结果展示**：
    - 前端界面显示 Agent 回复：“*我为你找到了 5 篇相关文献，列表如下：*”
    - 下方展示 5 张文献卡片，包含标题、摘要摘要、DOI 链接。
---
**下一步行动**：建议立即执行 **T1.1**，搭建 Rust Workspace，并定义好 `mcp-host` 的基础 trait 接口。这将是整个系统的基石。
