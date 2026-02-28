//! MCP Host 实现

use crate::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::process::{ChildStdin, ChildStdout};

/// MCP Host - 管理多个 MCP Server 的连接
pub struct McpHost {
    servers: HashMap<String, McpServerConnection>,
    next_id: u64,
}

/// MCP Server 连接
pub struct McpServerConnection {
    name: String,
    stdin: ChildStdin,
    stdout_reader: BufReader<ChildStdout>,
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

impl McpHost {
    /// 创建新的 MCP Host
    pub fn new() -> Self {
        Self {
            servers: HashMap::new(),
            next_id: 1,
        }
    }

    /// 启动 MCP Server
    pub async fn start_server(&mut self, name: &str, command: &str) -> Result<()> {
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

        let server = McpServerConnection {
            name: name.to_string(),
            stdin,
            stdout_reader: BufReader::new(stdout),
        };

        self.servers.insert(name.to_string(), server);
        Ok(())
    }

    /// 调用 MCP Tool
    pub async fn call_tool(&mut self, server: &str, tool: &str, params: Value) -> Result<Value> {
        let server_conn = self
            .servers
            .get_mut(server)
            .ok_or_else(|| crate::Error::Mcp(format!("服务器不存在: {}", server)))?;

        // 构造 JSON-RPC 请求
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

        let request_json = serde_json::to_string(&request)
            .map_err(|e| crate::Error::Mcp(format!("序列化请求失败: {}", e)))?;

        // 发送请求
        server_conn
            .stdin
            .write_all(request_json.as_bytes())
            .await
            .map_err(|e| crate::Error::Mcp(format!("发送请求失败: {}", e)))?;
        server_conn
            .stdin
            .write_all(b"\n")
            .await
            .map_err(|e| crate::Error::Mcp(format!("发送换行失败: {}", e)))?;

        // 读取响应
        let mut response_str = String::new();
        server_conn
            .stdout_reader
            .read_line(&mut response_str)
            .await
            .map_err(|e| crate::Error::Mcp(format!("读取响应失败: {}", e)))?;

        let response: JsonRpcResponse = serde_json::from_str(&response_str)
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

    /// 获取所有服务器列表
    pub fn list_servers(&self) -> Vec<&str> {
        self.servers.keys().map(|k| k.as_str()).collect()
    }
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
}
