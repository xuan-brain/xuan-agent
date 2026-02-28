use std::io::{self, BufRead, Write};

use xuan_agent::{Config, XuanAgent};
use xuan_agent::tools::Tool;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("xuan_agent=debug".parse()?),
        )
        .init();

    // 加载配置
    let config = Config::from_env()?;

    // 创建 Agent
    let mut agent = XuanAgent::new(config).await?;

    // 注册测试工具
    agent.register_tool(Tool {
        name: "echo".to_string(),
        description: "回显输入的消息".to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "message": {
                    "type": "string",
                    "description": "要回显的消息"
                }
            },
            "required": ["message"]
        }),
    });

    println!("XuanAgent CLI v0.1.0");
    println!("输入 'quit' 退出，输入 'help' 查看帮助\n");

    // 交互式对话循环
    let stdin = io::stdin();
    print!("> ");
    io::stdout().flush()?;

    for line in stdin.lock().lines() {
        let input = line?;

        if input.is_empty() {
            print!("> ");
            io::stdout().flush()?;
            continue;
        }

        // 解析命令
        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() {
            print!("> ");
            io::stdout().flush()?;
            continue;
        }

        let command = parts[0];

        match command {
            "quit" | "exit" | "q" => {
                println!("再见！");
                break;
            }
            "help" | "h" => {
                print_help();
                print_mcp_help();
                print!("> ");
                io::stdout().flush()?;
                continue;
            }
            "mcp" => {
                // MCP 命令
                if parts.len() < 3 {
                    println!("用法: mcp <server> <tool> [args...]");
                } else {
                    let server = parts[1];
                    let tool = parts[2];
                    let args = if parts.len() > 3 {
                        serde_json::json!(parts[3..].join(" "))
                    } else {
                        serde_json::json!({})
                    };

                    match agent.mcp_host_mut().call_tool(server, tool, args).await {
                        Ok(result) => {
                            println!("工具响应: {}\n", serde_json::to_string_pretty(&result).unwrap_or_else(|_| "N/A".to_string()));
                        }
                        Err(e) => {
                            eprintln!("工具调用失败: {}\n", e);
                        }
                    }
                }
            }
            "tools" => {
                // 列出可用工具
                let tools = agent.tools().list_tools();
                println!("可用工具:");
                for tool in tools {
                    println!("  - {}: {}", tool.name, tool.description);
                }
                println!();
            }
            _ => {
                // 发送消息给 Agent
                match agent.chat(&input).await {
                    Ok(response) => {
                        println!("{}\n", response);
                    }
                    Err(e) => {
                        eprintln!("错误: {}\n", e);
                    }
                }
            }
        }

        print!("> ");
        io::stdout().flush()?;
    }

    Ok(())
}

fn print_help() {
    println!("基本命令:");
    println!("  quit/exit/q - 退出程序");
    println!("  help/h      - 显示帮助");
    println!();
    println!("对话:");
    println!("  <message>   - 与 Agent 对话");
    println!();
}

fn print_mcp_help() {
    println!("MCP 命令:");
    println!("  mcp <server> <tool> [args...] - 调用 MCP 工具");
    println!("  tools                       - 列出可用工具");
    println!();
}
