//! mcp-server-reading - XuanAgent 文献阅读 MCP 服务器
//!
//! 提供 PDF 解析、文献摘要、对比分析等功能

use mcp_server_reading::McpReadingServer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("mcp_server_reading=debug".parse()?),
        )
        .init();

    tracing::info!("Starting XuanAgent MCP Reading Server...");

    let server = McpReadingServer::new();

    // 运行 stdio 模式（MCP 标准通信方式）
    server.run_stdio().await?;

    Ok(())
}
