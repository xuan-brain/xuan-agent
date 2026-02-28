# XuanAgent

科研 AI Agent 库，提供文献管理、学术写作辅助和科研知识管理功能。

## 状态

**当前阶段**: P0 - 完成 ✅
- P0-Sprint 1: 项目框架 ✅
- P0-Sprint 2: 基础功能完善 ✅
- 系统提示词优化 ✅
- 文献管理 MCP 集成 ✅

## 项目结构

```
xuan-agent/
├── crates/
│   ├── xuan-agent/         # 核心 lib crate
│   │   ├── agent.rs        # Agent 核心
│   │   ├── config.rs       # 配置管理 (含 System Prompt)
│   │   ├── error.rs        # 错误类型
│   │   ├── llm.rs          # LLM 服务
│   │   ├── tools.rs         # 工具系统
│   │   └── mcp/            # MCP 协议
│   │       └── host.rs     # MCP Host
│   ├── xuan-agent-cli/     # CLI 工具
│   └── examples/
│       ├── mcp_demo.rs           # 基本 MCP 演示
│       └── literature_mcp_demo.rs # 文献管理演示
├── tests/
│   └── mcp-servers/
│       ├── test_server.py         # 测试 MCP 服务器
│       └── literature_server.py   # 文献管理 MCP 服务器
├── docs/                   # 项目文档
└── [配置文件]
```

## 快速开始

### 1. 配置环境

```bash
cp .env.example .env
# 编辑 .env，填入你的 API keys
```

**注意**: `.env` 文件使用 JSON 格式，确保格式正确。

### 环境要求

- Rust 1.70+
- Python 3 (用于运行 MCP 服务器测试)
- 可选：有效的 LLM API 密钥 (用于测试聊天功能)

### 2. 运行 CLI

```bash
# 基本对话
cargo run --bin xuan-agent-cli

# 查看可用工具
$ tools
# 或在交互界面输入: tools

# 调用 MCP 工具
$ mcp <server> <tool> [args...]

# 示例
$ mcp test echo {"message": "Hello"}
```

### 3. MCP 演示

```bash
# 基本 MCP 演示
cargo run -p xuan-agent --example mcp_demo

# 文献管理 MCP 演示
cargo run -p xuan-agent --example literature_mcp_demo
```

### 4. 测试

```bash
cargo test
```

## 作为库使用

```rust
use xuan_agent::{Config, XuanAgent};

#[tokio::main]
async fn main() -> xuan_agent::Result<()> {
    let config = Config::from_env()?;
    let mut agent = XuanAgent::new(config).await?;

    // 聊天
    let response = agent.chat("你好").await?;
    println!("{}", response);

    // 添加 MCP 服务器
    agent.add_mcp_server("test", "python3 tests/mcp-servers/test_server.py").await?;

    // 调用 MCP 工具
    let result = agent.mcp_host_mut()
        .call_tool("test", "echo", serde_json::json!({"message": "Hello"}))
        .await?;

    Ok(())
}
```

## 开发计划

详见 `docs/plan-p0.md`

### P0-Sprint 1 ✅
- [x] Workspace 结构
- [x] 配置管理 (JSON .env)
- [x] 错误处理
- [x] CLI 工具
- [x] 单元测试 (5 个通过)

### P0-Sprint 2 ✅
- [x] LLM 服务 (HTTP API 调用)
- [x] MCP Host 实现
- [x] JSON-RPC 2.0 通信
- [x] 测试 MCP 服务器
- [x] 工具系统
- [x] CLI 增强功能 (mcp, tools 命令)

### 待完成 (P0-Sprint 2 剩余)
- [x] 文献管理 MCP 集成 ✅
- [x] System Prompt 优化 ✅

### P0 阶段完成情况

**P0 阶段已完成！** 以下是已实现的功能：

1. **核心框架**
   - Workspace 结构 (lib + bin crates)
   - 配置管理 (JSON .env 支持)
   - 错误处理 (thiserror)
   - 单元测试 (7 个通过)

2. **LLM 集成**
   - HTTP API 调用
   - 可配置 System Prompt
   - 科研助手默认提示词
   - 工具信息注入支持

3. **MCP 协议**
   - MCP Host 实现
   - JSON-RPC 2.0 通信
   - 多服务器管理
   - 测试服务器

4. **文献管理** (演示)
   - 搜索文献
   - 获取文献详情
   - 添加新文献
   - 导出引用 (BibTeX/APA)
   - 列出文献集合

5. **CLI 工具**
   - 交互式对话
   - MCP 工具调用
   - 工具列表查看

## CLI 命令参考

| 命令 | 说明 |
|:-----|:-----|
| `quit/exit/q` | 退出程序 |
| `help/h` | 显示帮助 |
| `tools` | 列出可用工具 |
| `mcp <server> <tool> [args]` | 调用 MCP 工具 |
| `<message>` | 与 Agent 对话 |
