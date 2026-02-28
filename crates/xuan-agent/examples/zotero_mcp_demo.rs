//! Zotero MCP 集成演示
//!
//! 演示如何使用 XuanAgent 通过 Streamable HTTP 协议与 Zotero MCP 通信
//!
//! 前置条件:
//! 1. 安装 Zotero 7+
//! 2. 安装 Zotero MCP 插件: https://github.com/cookjohn/zotero-mcp/releases
//! 3. 在 Zotero 中启用 MCP 服务器 (默认端口: 23120)

use xuan_agent::Config;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("xuan_agent=debug".parse().unwrap()),
        )
        .init();

    println!("=== XuanAgent Zotero MCP 集成演示 ===\n");

    // 配置 Zotero MCP 服务器地址
    // 默认地址: http://127.0.0.1:23120/mcp
    // 可在 Zotero -> Preferences -> Zotero MCP Plugin 中配置
    let zotero_url = std::env::var("ZOTERO_MCP_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:23120/mcp".to_string());

    println!("Zotero MCP 服务器: {}\n", zotero_url);

    // 创建 Agent
    let config = Config::from_env().unwrap_or_else(|_| {
        // 如果没有 .env 文件，使用默认配置
        Config {
            ai_provider: xuan_agent::AiProviderConfig {
                provider: "test".to_string(),
                base_url: "http://test.com".to_string(),
                api_key: "test-key".to_string(),
                model: "test-model".to_string(),
                description: None,
            },
            db: xuan_agent::DbConfig {
                connect: "http://localhost:8000".to_string(),
                user: "root".to_string(),
                pass: "secret".to_string(),
                namespace: "test".to_string(),
                database: "test".to_string(),
            },
            system_prompt: None,
        }
    });

    let mut agent = xuan_agent::XuanAgent::new(config).await?;

    // 添加 Zotero HTTP MCP 服务器
    println!("1. 连接 Zotero MCP 服务器...");
    match agent.add_http_mcp_server("zotero", &zotero_url).await {
        Ok(_) => println!("   ✓ Zotero MCP 已连接\n"),
        Err(e) => {
            println!("   ✗ 连接失败: {}\n", e);
            println!("请确保:");
            println!("   1. Zotero 正在运行");
            println!("   2. Zotero MCP 插件已安装");
            println!("   3. MCP 服务器已启用 (Preferences -> Zotero MCP Plugin)");
            return Err(e.into());
        }
    }

    // 演示 1: 列出可用工具
    println!("2. 列出可用的 Zotero MCP 工具...");
    match agent.mcp_host_mut().list_tools("zotero").await {
        Ok(tools) => {
            println!("   可用工具:");
            for tool in tools {
                if let Some(name) = tool.get("name").and_then(|n| n.as_str()) {
                    if let Some(desc) = tool.get("description").and_then(|d| d.as_str()) {
                        println!("   - {}: {}", name, desc);
                    }
                }
            }
            println!();
        }
        Err(e) => println!("   ✗ 获取工具列表失败: {}\n", e),
    }

    // 演示 2: 搜索文献库
    println!("3. 搜索 Zotero 文献库...");
    match agent.mcp_host_mut().call_tool(
        "zotero",
        "search_library",
        json!({"q": "machine learning", "limit": 5})
    ).await {
        Ok(result) => {
            println!("   搜索结果:\n{}\n", serde_json::to_string_pretty(&result).unwrap_or_else(|_| "N/A".to_string()));
        }
        Err(e) => println!("   ✗ 搜索失败: {}\n", e),
    }

    // 演示 3: 按年份筛选
    println!("4. 按年份筛选文献 (2023-2024)...");
    match agent.mcp_host_mut().call_tool(
        "zotero",
        "search_library",
        json!({"year": "2023", "limit": 3})
    ).await {
        Ok(result) => {
            if let Some(items) = result.get("items").and_then(|i| i.as_array()) {
                println!("   找到 {} 篇文献:", items.len());
                for item in items.iter().take(3) {
                    let title = item.get("title")
                        .and_then(|t| t.as_str())
                        .unwrap_or("N/A");
                    let creators = item.get("creators")
                        .and_then(|c| c.as_array());
                    let authors = creators
                        .map(|c| {
                            c.iter()
                                .filter_map(|cr| cr.get("firstName").and_then(|f| f.as_str())
                                    .zip(cr.get("lastName").and_then(|l| l.as_str())))
                                .map(|(f, l)| format!("{} {}", f, l))
                                .collect::<Vec<_>>()
                                .join(", ")
                        })
                        .unwrap_or_else(|| "N/A".to_string());
                    println!("   - {} ({})", title, authors);
                }
            }
            println!();
        }
        Err(e) => println!("   ✗ 筛选失败: {}\n", e),
    }

    // 演示 4: 获取文献详情
    println!("5. 获取文献详情...");
    // 先搜索一篇文献获取其 key
    match agent.mcp_host_mut().call_tool(
        "zotero",
        "search_library",
        json!({"q": "", "limit": 1})
    ).await {
        Ok(search_result) => {
            if let Some(items) = search_result.get("items").and_then(|i| i.as_array()) {
                if let Some(first_item) = items.first() {
                    if let Some(key) = first_item.get("key").and_then(|k| k.as_str()) {
                        println!("   获取文献 (key: {})...", key);
                        match agent.mcp_host_mut().call_tool(
                            "zotero",
                            "get_item_details",
                            json!({"itemKey": key})
                        ).await {
                            Ok(details) => {
                                let title = details.get("title")
                                    .and_then(|t| t.as_str())
                                    .unwrap_or("N/A");
                                let abstract_note = details.get("abstractNote")
                                    .and_then(|a| a.as_str())
                                    .unwrap_or("N/A");
                                println!("   标题: {}", title);
                                println!("   摘要: {}", abstract_note);
                                println!();
                            }
                            Err(e) => println!("   ✗ 获取详情失败: {}\n", e),
                        }
                    }
                }
            }
        }
        Err(e) => println!("   ✗ 搜索失败: {}\n", e),
    }

    // 演示 5: 按标签搜索
    println!("6. 按标签搜索文献...");
    match agent.mcp_host_mut().call_tool(
        "zotero",
        "search_library",
        json!({"tag": ["ai", "machine-learning"], "limit": 3})
    ).await {
        Ok(result) => {
            if let Some(items) = result.get("items").and_then(|i| i.as_array()) {
                println!("   找到 {} 篇标签匹配的文献", items.len());
                for item in items.iter().take(3) {
                    let title = item.get("title")
                        .and_then(|t| t.as_str())
                        .unwrap_or("N/A");
                    let tags = item.get("tags")
                        .and_then(|t| t.as_array())
                        .map(|ts| ts.iter()
                            .filter_map(|tag| tag.get("tag").and_then(|t| t.as_str()))
                            .collect::<Vec<_>>()
                            .join(", "))
                        .unwrap_or_else(|| "N/A".to_string());
                    println!("   - {} [标签: {}]", title, tags);
                }
            }
            println!();
        }
        Err(e) => println!("   ✗ 标签搜索失败: {}\n", e),
    }

    // 演示 6: 通过 DOI 查找文献
    println!("7. 通过 DOI 查找文献...");
    match agent.mcp_host_mut().call_tool(
        "zotero",
        "find_item_by_identifier",
        json!({"doi": "10.1038/nature14539"})
    ).await {
        Ok(result) => {
            println!("   查找结果:\n{}\n", serde_json::to_string_pretty(&result).unwrap_or_else(|_| "N/A".to_string()));
        }
        Err(e) => println!("   ✗ DOI 查找失败: {}\n", e),
    }

    println!("=== 演示完成 ===");
    println!("\n提示:");
    println!("- 修改 ZOTERO_MCP_URL 环境变量可自定义 Zotero MCP 服务器地址");
    println!("- 更多工具请参考: https://github.com/cookjohn/zotero-mcp");

    Ok(())
}
