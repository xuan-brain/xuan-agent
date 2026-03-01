//! mcp-server-reading - XuanAgent 文献阅读 MCP 服务器
//!
//! 提供文献检索、解析、摘要和对比功能的 MCP 服务器

pub mod pdf;
pub mod chunk;
pub mod tools;

use anyhow::Result;
use jsonrpc_core::IoHandler;
use std::io::{BufRead, BufReader, Write};

/// MCP 服务器
pub struct McpReadingServer {
    io_handler: IoHandler,
}

impl McpReadingServer {
    /// 创建新的 MCP 服务器
    pub fn new() -> Self {
        let mut io_handler = IoHandler::new();

        // 注册同步方法
        io_handler.add_sync_method("tools/list", tools::list_tools);
        io_handler.add_sync_method("tools/call", tools::call_tool);

        // 注册文献阅读工具
        io_handler.add_sync_method("literature_search", tools::literature_search);
        io_handler.add_sync_method("summarize_paper", tools::summarize_paper);
        io_handler.add_sync_method("compare_papers", tools::compare_papers);
        io_handler.add_sync_method("extract_key_entities", tools::extract_key_entities);
        io_handler.add_sync_method("get_paper_fulltext", tools::get_paper_fulltext);

        Self { io_handler }
    }

    /// 运行 stdio 模式（用于 MCP 客户端连接）
    pub async fn run_stdio(&self) -> Result<()> {
        let stdin = std::io::stdin();
        let stdout = std::io::stdout();
        let mut reader = BufReader::new(stdin);
        let mut writer = stdout;

        loop {
            // 读取 JSON-RPC 请求
            let mut line = String::new();
            reader.read_line(&mut line)?;

            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            // 处理请求
            let response = self.io_handler.handle_request_sync(line);

            // 发送响应
            if let Some(resp) = response {
                writeln!(writer, "{}", resp)?;
                writer.flush()?;
            }
        }
    }
}

impl Default for McpReadingServer {
    fn default() -> Self {
        Self::new()
    }
}
