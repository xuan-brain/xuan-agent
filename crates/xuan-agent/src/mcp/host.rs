//! MCP Host 实现
//!
//! 支持两种传输方式：
//! - Stdio: 通过标准输入/输出与子进程通信
//! - HTTP: 通过 Streamable HTTP 协议与 HTTP MCP 服务器通信

use crate::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::process::{ChildStdin, ChildStdout};

/// MCP 传输类型
#[derive(Debug, Clone)]
pub enum McpTransport {
    /// Stdio 传输 (子进程)
    Stdio { command: String },
    /// HTTP 传输 (Streamable HTTP)
    Http { url: String },
}

/// MCP Host - 管理多个 MCP Server 的连接
pub struct McpHost {
    servers: HashMap<String, McpServerConnection>,
    next_id: u64,
    http_client: Client,
}

/// MCP Server 连接 (枚举支持多种传输方式)
enum McpServerConnection {
    Stdio(StdioConnection),
    Http(HttpConnection),
}

/// Stdio 连接
struct StdioConnection {
    name: String,
    stdin: ChildStdin,
    stdout_reader: BufReader<ChildStdout>,
}

/// HTTP 连接
struct HttpConnection {
    name: String,
    url: String,
}

/// JSON-RPC 请求
#[derive(Debug, Serialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: Value,
    id: u64,
}

/// JSON-RPC 响应
#[derive(Debug, Deserialize)]
struct JsonRpcResponse {
    result: Option<Value>,
    error: Option<JsonRpcError>,
}

/// JSON-RPC 错误
#[derive(Debug, Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
}

/// 解析 JSON-RPC 响应
fn parse_jsonrpc_response(response_str: &str) -> Result<Value> {
    let response: JsonRpcResponse = serde_json::from_str(response_str)
        .map_err(|e| crate::Error::Mcp(format!("解析响应失败: {}", e)))?;

    if let Some(error) = response.error {
        Err(crate::Error::Mcp(format!(
            "MCP 错误: {} - {}",
            error.code, error.message
        )))
    } else {
        Ok(response.result.unwrap_or(Value::Null))
    }
}

impl McpHost {
    /// 创建新的 MCP Host
    pub fn new() -> Self {
        Self {
            servers: HashMap::new(),
            next_id: 1,
            http_client: Client::new(),
        }
    }

    /// 添加 Stdio MCP Server
    pub async fn add_stdio_server(&mut self, name: &str, command: &str) -> Result<()> {
        // 解析命令和参数
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return Err(crate::Error::Mcp("命令为空".to_string()));
        }

        let mut cmd = Command::new(parts[0]);
        for arg in &parts[1..] {
            cmd.arg(arg);
        }

        let mut child = cmd
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| crate::Error::Mcp(format!("启动服务器失败: {}", e)))?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| crate::Error::Mcp("无法获取 stdin".to_string()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| crate::Error::Mcp("无法获取 stdout".to_string()))?;

        let conn = McpServerConnection::Stdio(StdioConnection {
            name: name.to_string(),
            stdin,
            stdout_reader: BufReader::new(stdout),
        });

        self.servers.insert(name.to_string(), conn);
        Ok(())
    }

    /// 添加 HTTP MCP Server (Streamable HTTP)
    pub async fn add_http_server(&mut self, name: &str, url: &str) -> Result<()> {
        // 验证 URL 格式
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(crate::Error::Mcp(format!("无效的 URL: {}", url)));
        }

        let conn = McpServerConnection::Http(HttpConnection {
            name: name.to_string(),
            url: url.to_string(),
        });

        self.servers.insert(name.to_string(), conn);
        Ok(())
    }

    /// 启动 MCP Server (兼容旧接口，默认使用 Stdio)
    pub async fn start_server(&mut self, name: &str, command: &str) -> Result<()> {
        self.add_stdio_server(name, command).await
    }

    /// 调用 MCP Tool
    pub async fn call_tool(&mut self, server: &str, tool: &str, params: Value) -> Result<Value> {
        let id = self.next_id;
        self.next_id += 1;

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "tools/call".to_string(),
            params: serde_json::json!({
                "name": tool,
                "arguments": params,
            }),
            id,
        };

        // 发送请求并获取响应
        let response_str = match self.servers.get_mut(server) {
            Some(McpServerConnection::Stdio(conn)) => {
                call_stdio(conn, &request).await?
            }
            Some(McpServerConnection::Http(conn)) => {
                call_http(&self.http_client, conn, &request).await?
            }
            None => return Err(crate::Error::Mcp(format!("服务器不存在: {}", server))),
        };

        parse_jsonrpc_response(&response_str)
    }

    /// 列出可用工具
    pub async fn list_tools(&mut self, server: &str) -> Result<Vec<Value>> {
        let id = self.next_id;
        self.next_id += 1;

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "tools/list".to_string(),
            params: serde_json::json!({}),
            id,
        };

        let response_str = match self.servers.get_mut(server) {
            Some(McpServerConnection::Stdio(conn)) => {
                call_stdio(conn, &request).await?
            }
            Some(McpServerConnection::Http(conn)) => {
                call_http(&self.http_client, conn, &request).await?
            }
            None => return Err(crate::Error::Mcp(format!("服务器不存在: {}", server))),
        };

        let result = parse_jsonrpc_response(&response_str)?;

        let tools = result
            .get("tools")
            .and_then(|t| t.as_array())
            .ok_or_else(|| crate::Error::Mcp("响应格式错误".to_string()))?;

        Ok(tools.clone())
    }

    /// 获取所有服务器列表
    pub fn list_servers(&self) -> Vec<&str> {
        self.servers.keys().map(|k| k.as_str()).collect()
    }

    /// 获取服务器传输类型
    pub fn get_server_transport(&self, server: &str) -> Option<String> {
        self.servers.get(server).map(|conn| match conn {
            McpServerConnection::Stdio(_) => "stdio".to_string(),
            McpServerConnection::Http(_) => "http".to_string(),
        })
    }
}

/// 通过 Stdio 调用 MCP 工具
async fn call_stdio(conn: &mut StdioConnection, request: &JsonRpcRequest) -> Result<String> {
    let request_json = serde_json::to_string(request)
        .map_err(|e| crate::Error::Mcp(format!("序列化请求失败: {}", e)))?;

    // 发送请求
    conn.stdin
        .write_all(request_json.as_bytes())
        .await
        .map_err(|e| crate::Error::Mcp(format!("发送请求失败: {}", e)))?;
    conn.stdin
        .write_all(b"\n")
        .await
        .map_err(|e| crate::Error::Mcp(format!("发送换行失败: {}", e)))?;

    // 读取响应
    let mut response_str = String::new();
    conn.stdout_reader
        .read_line(&mut response_str)
        .await
        .map_err(|e| crate::Error::Mcp(format!("读取响应失败: {}", e)))?;

    Ok(response_str)
}

/// 通过 HTTP 调用 MCP 工具
async fn call_http(client: &Client, conn: &HttpConnection, request: &JsonRpcRequest) -> Result<String> {
    let request_body = serde_json::to_vec(request)
        .map_err(|e| crate::Error::Mcp(format!("序列化请求失败: {}", e)))?;

    let response = client
        .post(&conn.url)
        .header("Content-Type", "application/json")
        .body(request_body)
        .send()
        .await
        .map_err(|e| crate::Error::Mcp(format!("HTTP 请求失败: {}", e)))?;

    let status = response.status();
    let response_bytes = response
        .bytes()
        .await
        .map_err(|e| crate::Error::Mcp(format!("读取响应失败: {}", e)))?;

    if !status.is_success() {
        return Err(crate::Error::Mcp(format!(
            "HTTP 错误: {} - {}",
            status.as_u16(),
            String::from_utf8_lossy(&response_bytes)
        )));
    }

    String::from_utf8(response_bytes.to_vec())
        .map_err(|e| crate::Error::Mcp(format!("响应解码失败: {}", e)))
}

impl Default for McpHost {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_host_creation() {
        let host = McpHost::new();
        assert!(host.list_servers().is_empty());
    }

    #[test]
    fn test_json_rpc_request_serialization() {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "tools/call".to_string(),
            params: serde_json::json!({"name": "test_tool", "arguments": {}}),
            id: 1,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"method\":\"tools/call\""));
        assert!(json.contains("\"id\":1"));
    }

    #[tokio::test]
    async fn test_add_http_server() {
        let mut host = McpHost::new();

        let result = host.add_http_server("test", "http://127.0.0.1:23120/mcp").await;
        assert!(result.is_ok());
        assert_eq!(host.list_servers(), vec!["test"]);
        assert_eq!(host.get_server_transport("test"), Some("http".to_string()));
    }

    #[tokio::test]
    async fn test_add_http_server_invalid_url() {
        let mut host = McpHost::new();

        let result = host.add_http_server("test", "invalid-url").await;
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_jsonrpc_response_success() {
        let json = r#"{"result":{"value":"test"},"id":1}"#;
        let result = parse_jsonrpc_response(json);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().get("value").unwrap().as_str(), Some("test"));
    }

    #[test]
    fn test_parse_jsonrpc_response_error() {
        let json = r#"{"error":{"code":-1,"message":"test error"},"id":1}"#;
        let result = parse_jsonrpc_response(json);
        assert!(result.is_err());
    }
}
