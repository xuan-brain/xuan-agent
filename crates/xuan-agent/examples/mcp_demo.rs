//! MCP Host 演示程序

use std::path::PathBuf;
use xuan_agent::mcp::McpHost;
use xuan_agent::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== XuanAgent MCP Host 演示 ===\n");

    // 创建 MCP Host
    let mut host = McpHost::new();

    // 获取项目根目录（从示例文件向上两级）
    let mut server_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    server_path.pop(); // examples
    server_path.pop(); // xuan-agent
    server_path.push("tests");
    server_path.push("mcp-servers");
    server_path.push("test_server.py");

    let server_str = format!("python3 {}", server_path.display());

    // 启动测试服务器
    println!("正在启动测试 MCP 服务器...");
    println!("服务器路径: {}", server_str);

    let server_result = host.start_server("test", &server_str).await;

    if server_result.is_err() {
        eprintln!("无法启动测试服务器: {}", server_result.unwrap_err());
        eprintln!("请确保 Python 3 已安装");
        return Ok(());
    }

    // 给服务器一点时间启动
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    println!("测试服务器已启动\n");

    // 获取服务器列表
    println!("已连接的服务器: {:?}", host.list_servers());
    println!();

    // 调用 echo 工具
    println!("正在调用 echo 工具...");
    let response = host
        .call_tool("test", "echo", serde_json::json!({"message": "Hello from XuanAgent!"}))
        .await;

    match response {
        Ok(result) => {
            println!("工具响应: {}", serde_json::to_string_pretty(&result).unwrap());
        }
        Err(e) => {
            eprintln!("工具调用失败: {}", e);
        }
    }

    println!("\n演示完成！");
    Ok(())
}
