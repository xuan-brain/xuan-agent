use std::path::Path;

use chrono::Utc;
use clap::{Parser, Subcommand};
use xuan_agent::storage::{Chunk, ChunkType, EmbeddingService, Paper, SurrealDBStorage};
use xuan_agent::{Config, PdfParser, PaperChunker, XuanAgent};

#[derive(Parser)]
#[command(name = "xuan-agent-cli")]
#[command(about = "XuanAgent - 科研 AI 助手命令行工具", long_about = None)]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 启动交互式对话模式
    Chat,
    /// 数据库操作
    Db {
        #[command(subcommand)]
        db_command: DbCommands,
    },
    /// 导入文献
    Import {
        /// PDF 文件路径
        file: String,
    },
    /// 搜索文献
    Search {
        /// 搜索关键词
        query: String,
        /// 返回结果数量
        #[arg(short, long, default_value_t = 10)]
        limit: u32,
    },
    /// 列出文献
    List {
        /// 返回结果数量
        #[arg(short, long, default_value_t = 20)]
        limit: u32,
    },
}

#[derive(Subcommand)]
enum DbCommands {
    /// 初始化数据库表结构
    Init,
    /// 显示数据库统计信息
    Stats,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("xuan_agent=debug".parse()?),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Chat => run_chat().await,
        Commands::Db { db_command } => run_db_command(db_command).await,
        Commands::Import { file } => run_import(file).await,
        Commands::Search { query, limit } => run_search(query, limit).await,
        Commands::List { limit } => run_list(limit).await,
    }
}

async fn run_chat() -> anyhow::Result<()> {
    use std::io::{BufRead, Write};

    let config = Config::from_env()?;
    let mut agent = XuanAgent::new(config).await?;

    println!("XuanAgent CLI v0.1.0 - 交互式对话模式");
    println!("输入 'quit' 退出，输入 'help' 查看帮助\n");

    let stdin = std::io::stdin();
    print!("> ");
    std::io::stdout().flush()?;

    for line in stdin.lock().lines() {
        let input = line?;

        if input.is_empty() {
            print!("> ");
            std::io::stdout().flush()?;
            continue;
        }

        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() {
            print!("> ");
            std::io::stdout().flush()?;
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
                print!("> ");
                std::io::stdout().flush()?;
                continue;
            }
            _ => match agent.chat(&input).await {
                Ok(response) => {
                    println!("{}\n", response);
                }
                Err(e) => {
                    eprintln!("错误: {}\n", e);
                }
            },
        }

        print!("> ");
        std::io::stdout().flush()?;
    }

    Ok(())
}

async fn run_db_command(command: DbCommands) -> anyhow::Result<()> {
    let config = Config::from_env()?;
    let db_config = config.db;

    match command {
        DbCommands::Init => {
            println!("正在连接 SurrealDB: {}", db_config.connect);
            let storage = SurrealDBStorage::connect(&db_config).await?;

            println!("正在创建表结构...");
            storage.init_schema().await?;

            println!("✅ 数据库初始化完成！");
        }
        DbCommands::Stats => {
            let storage = SurrealDBStorage::connect(&db_config).await?;

            let (paper_count, chunk_count) = storage.stats().await?;

            println!("📊 数据库统计:");
            println!("  文献数量: {}", paper_count);
            println!("  分块数量: {}", chunk_count);
        }
    }

    Ok(())
}

async fn run_import(file: String) -> anyhow::Result<()> {
    let path = Path::new(&file);
    if !path.exists() {
        anyhow::bail!("文件不存在: {}", file);
    }

    let config = Config::from_env()?;
    let db_config = config.db.clone();

    println!("正在连接数据库...");
    let storage = SurrealDBStorage::connect(&db_config).await?;

    println!("正在解析文件: {}", file);

    // 解析 PDF
    let doc = PdfParser::parse(&file)?;
    println!("✅ PDF 解析成功 ({} 页)", doc.page_count);

    // 提取标题（优先使用文件名）
    let title = PdfParser::extract_title_from_filename(&file)
        .unwrap_or_else(|| "Unknown Title".to_string());

    // 简单摘要：取前500字符
    let abstract_text = if doc.text.len() > 500 {
        format!("{}...", doc.text.chars().take(500).collect::<String>())
    } else {
        doc.text.clone()
    };

    let paper = Paper {
        id: format!("paper:{}", uuid::Uuid::new_v4()),
        title,
        abstract_text,
        authors: vec!["Unknown".to_string()],
        year: None,
        journal: None,
        doi: None,
        file_path: Some(file.clone()),
        tags: vec!["imported".to_string()],
        created_at: Utc::now(),
    };

    // 存储文献
    let paper_id = storage.store_paper(paper.clone()).await?;
    println!("✅ 文献已保存: \"{}\" (ID: {})", paper.title, paper_id);

    // 文献分块
    println!("正在分块文献内容...");
    let chunker = PaperChunker::default();
    let text_chunks = chunker.chunk_by_paragraph(&doc.text);
    println!("✅ 生成 {} 个分块", text_chunks.len());

    // 创建向量化服务
    println!("正在初始化向量化服务...");
    let embedding_service = EmbeddingService::new(config.ai_provider.clone());
    println!("✅ 向量化服务就绪 (维度: {})", embedding_service.embedding_dim);

    // 存储分块（带向量化）
    println!("正在向量化分块并存储...");
    let mut chunks = Vec::new();
    let mut embedded_count = 0;

    for (idx, content) in text_chunks.iter().enumerate() {
        // 自动检测分块类型
        let chunk_type = ChunkType::from_text(content);

        // 生成嵌入向量
        let embedding = match embedding_service.embed(content).await {
            Ok(emb) => {
                embedded_count += 1;
                Some(emb)
            }
            Err(e) => {
                eprintln!("  警告: 分块 {} 向量化失败: {}", idx + 1, e);
                None
            }
        };

        chunks.push(Chunk {
            id: format!("chunk:{}", uuid::Uuid::new_v4()),
            paper_id: paper_id.clone(),
            content: content.clone(),
            chunk_index: idx,
            embedding,
            page_number: None,
            chunk_type,
        });

        // 显示进度（每10个分块显示一次）
        if (idx + 1) % 10 == 0 {
            println!("  已处理 {}/{} 分块", idx + 1, text_chunks.len());
        }
    }

    storage.store_chunks(chunks).await?;
    println!("✅ 已存储 {} 个分块 ({} 个已向量化)", text_chunks.len(), embedded_count);

    println!("\n📊 导入完成:");
    println!("   标题: {}", paper.title);
    println!("   页数: {}", doc.page_count);
    println!("   分块: {}", text_chunks.len());
    println!("   向量化: {}/{}", embedded_count, text_chunks.len());

    Ok(())
}

async fn run_search(query: String, limit: u32) -> anyhow::Result<()> {
    let config = Config::from_env()?;
    let db_config = config.db;

    let storage = SurrealDBStorage::connect(&db_config).await?;

    println!("🔍 搜索: \"{}\"", query);

    let results = storage.search_by_keyword(&query, limit).await?;

    if results.is_empty() {
        println!("未找到相关文献");
    } else {
        println!("找到 {} 篇相关文献:", results.len());
        for (i, paper) in results.iter().enumerate() {
            println!(
                "  {}. \"{}\" ({})",
                i + 1,
                paper.title,
                paper.year.unwrap_or(0)
            );
            if let Some(journal) = &paper.journal {
                println!("     期刊: {}", journal);
            }
            println!();
        }
    }

    Ok(())
}

async fn run_list(limit: u32) -> anyhow::Result<()> {
    let config = Config::from_env()?;
    let db_config = config.db;

    let storage = SurrealDBStorage::connect(&db_config).await?;

    println!("📚 文献列表 (最新 {} 篇):", limit);

    let papers = storage.list_papers(limit).await?;

    if papers.is_empty() {
        println!("暂无文献");
    } else {
        for (i, paper) in papers.iter().enumerate() {
            println!(
                "  {}. \"{}\" - {} ({})",
                i + 1,
                paper.title,
                paper.authors.join(", "),
                paper.year.unwrap_or(0)
            );
        }
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
