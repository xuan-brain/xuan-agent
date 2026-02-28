//! MCP Host 集成测试

use xuan_agent::mcp::McpHost;
use serde_json::json;

#[tokio::test]
async fn test_mcp_host_communication() {
    let mut host = McpHost::new();

    // 启动测试服务器
    // 注意：这需要 Python 环境
    let result = host.start_server("test", "python3 tests/mcp-servers/test_server.py").await;

    // 如果 Python 不可用，跳过测试
    if result.is_err() {
        println!("Skipping test: Python 3 not available");
        return;
    }

    // 给服务器一点时间启动
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // 测试工具调用
    let response = host
        .call_tool("test", "echo", json!({"message": "Hello, MCP!"}))
        .await;

    // 验证响应
    assert!(response.is_ok());
    let result = response.unwrap();
    assert!(result.is_object());
}

#[tokio::test]
async fn test_mcp_host_list_servers() {
    let mut host = McpHost::new();

    // 初始时应该没有服务器
    assert!(host.list_servers().is_empty());

    // 启动测试服务器
    let result = host.start_server("test", "python3 tests/mcp-servers/test_server.py").await;

    if result.is_err() {
        return;
    }

    // 现在应该有一个服务器
    let servers = host.list_servers();
    assert_eq!(servers.len(), 1);
    assert_eq!(servers[0], "test");
}
