//! 文献管理 MCP 演示
//!
//! 演示如何使用 XuanAgent 集成文献管理功能

use xuan_agent::{Config, XuanAgent};
use serde_json::json;

#[tokio::main]
async fn main() -> xuan_agent::Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("xuan_agent=debug".parse().unwrap()),
        )
        .init();

    println!("=== XuanAgent 文献管理 MCP 演示 ===\n");

    // 加载配置
    let config = Config::from_env()?;
    let mut agent = XuanAgent::new(config).await?;

    // 启动文献管理 MCP 服务器
    println!("1. 启动文献管理 MCP 服务器...");
    agent.add_mcp_server(
        "literature",
        "python3 tests/mcp-servers/literature_server.py"
    ).await?;
    println!("   ✓ 文献管理服务器已启动\n");

    // 等待服务器初始化
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // 演示 1: 搜索文献
    println!("2. 搜索文献 (关键词: 'transformer')...");
    match agent.mcp_host_mut().call_tool(
        "literature",
        "search_literature",
        json!({"query": "transformer", "limit": 5})
    ).await {
        Ok(result) => {
            if let Some(content) = result.get("content") {
                if let Some(items) = content.as_array() {
                    if let Some(text) = items.first().and_then(|t| t.get("text")) {
                        println!("   搜索结果:\n{}\n", text);
                    }
                }
            }
        }
        Err(e) => println!("   ✗ 搜索失败: {}\n", e),
    }

    // 演示 2: 获取文献详情
    println!("3. 获取文献详情 (ID: lit001)...");
    match agent.mcp_host_mut().call_tool(
        "literature",
        "get_literature",
        json!({"id": "lit001"})
    ).await {
        Ok(result) => {
            if let Some(content) = result.get("content") {
                if let Some(items) = content.as_array() {
                    if let Some(text) = items.first().and_then(|t| t.get("text")) {
                        println!("   文献详情:\n{}\n", text);
                    }
                }
            }
        }
        Err(e) => println!("   ✗ 获取失败: {}\n", e),
    }

    // 演示 3: 列出文献集合
    println!("4. 列出文献集合...");
    match agent.mcp_host_mut().call_tool(
        "literature",
        "list_collections",
        json!({})
    ).await {
        Ok(result) => {
            if let Some(content) = result.get("content") {
                if let Some(items) = content.as_array() {
                    if let Some(text) = items.first().and_then(|t| t.get("text")) {
                        println!("   集合列表:\n{}\n", text);
                    }
                }
            }
        }
        Err(e) => println!("   ✗ 列出失败: {}\n", e),
    }

    // 演示 4: 导出引用 (BibTeX)
    println!("5. 导出引用 (BibTeX 格式)...");
    match agent.mcp_host_mut().call_tool(
        "literature",
        "export_citation",
        json!({"id": "lit002", "format": "bibtex"})
    ).await {
        Ok(result) => {
            if let Some(content) = result.get("content") {
                if let Some(items) = content.as_array() {
                    if let Some(text) = items.first().and_then(|t| t.get("text")) {
                        println!("   引用:\n{}\n", text);
                    }
                }
            }
        }
        Err(e) => println!("   ✗ 导出失败: {}\n", e),
    }

    // 演示 5: 添加新文献
    println!("6. 添加新文献...");
    match agent.mcp_host_mut().call_tool(
        "literature",
        "add_literature",
        json!({
            "title": "GPT-4 Technical Report",
            "authors": ["OpenAI"],
            "year": 2023,
            "venue": "arXiv",
            "tags": ["llm", "gpt"]
        })
    ).await {
        Ok(result) => {
            if let Some(content) = result.get("content") {
                if let Some(items) = content.as_array() {
                    if let Some(text) = items.first().and_then(|t| t.get("text")) {
                        println!("   {}\n", text);
                    }
                }
            }
        }
        Err(e) => println!("   ✗ 添加失败: {}\n", e),
    }

    // 演示 6: 使用 Agent 对话
    println!("7. 与 Agent 对话...");
    match agent.chat("请帮我搜索关于 BERT 的文献").await {
        Ok(response) => {
            println!("   Agent 回复:\n   {}\n", response);
        }
        Err(e) => println!("   ✗ 对话失败: {}\n", e),
    }

    println!("=== 演示完成 ===");

    Ok(())
}
