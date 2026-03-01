//! MCP 工具实现
//!
//! 实现文献阅读相关的 MCP 工具

use crate::chunk::Chunker;
use crate::pdf::{PdfDocument, PdfParser};
use jsonrpc_core::{Params, Result};
use serde_json::{json, Value};

/// 列出可用工具
pub fn list_tools(_params: Params) -> Result<Value> {
    let tools = json!([
        {
            "name": "literature_search",
            "description": "在文献库中搜索相关文献，支持关键词和语义搜索",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "搜索查询，支持关键词或自然语言描述"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "返回结果数量限制，默认 10",
                        "default": 10
                    },
                    "search_type": {
                        "type": "string",
                        "description": "搜索类型：keyword（关键词）或 semantic（语义），默认 semantic",
                        "enum": ["keyword", "semantic"],
                        "default": "semantic"
                    }
                },
                "required": ["query"]
            }
        },
        {
            "name": "summarize_paper",
            "description": "生成文献的结构化摘要，包括研究问题、方法、结果、创新点",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "PDF 文件路径"
                    },
                    "detail_level": {
                        "type": "string",
                        "description": "摘要详细程度：brief（简洁）、standard（标准）、detailed（详细）",
                        "enum": ["brief", "standard", "detailed"],
                        "default": "standard"
                    }
                },
                "required": ["file_path"]
            }
        },
        {
            "name": "compare_papers",
            "description": "对比多篇文献的方法、数据集、结果等",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "file_paths": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "PDF 文件路径列表"
                    },
                    "aspects": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "对比维度，如 methods、datasets、results、innovations 等",
                        "default": ["methods", "results", "innovations"]
                    }
                },
                "required": ["file_paths"]
            }
        },
        {
            "name": "extract_key_entities",
            "description": "从文献中提取关键实体，如术语、人名、机构名、数据集名等",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "PDF 文件路径"
                    },
                    "entity_types": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "实体类型：terms（术语）、methods（方法）、datasets（数据集）、all（全部）",
                        "default": ["all"]
                    }
                },
                "required": ["file_path"]
            }
        },
        {
            "name": "get_paper_fulltext",
            "description": "获取文献的完整文本内容，可选择是否包含结构化信息",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "PDF 文件路径"
                    },
                    "include_structure": {
                        "type": "boolean",
                        "description": "是否包含章节结构信息，默认 false",
                        "default": false
                    },
                    "output_format": {
                        "type": "string",
                        "description": "输出格式：text（纯文本）、markdown（Markdown）、json（结构化 JSON）",
                        "enum": ["text", "markdown", "json"],
                        "default": "text"
                    }
                },
                "required": ["file_path"]
            }
        }
    ]);

    Ok(json!({ "tools": tools }))
}

/// 调用工具
pub fn call_tool(params: Params) -> Result<Value> {
    let params_obj = params.parse::<Value>().unwrap();
    let tool_name = params_obj
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let arguments = params_obj.get("arguments").cloned().unwrap_or(json!({}));

    match tool_name {
        "literature_search" => {
            let args = vec![arguments];
            literature_search(Params::Array(args))
        }
        "summarize_paper" => {
            let args = vec![arguments];
            summarize_paper(Params::Array(args))
        }
        "compare_papers" => {
            let args = vec![arguments];
            compare_papers(Params::Array(args))
        }
        "extract_key_entities" => {
            let args = vec![arguments];
            extract_key_entities(Params::Array(args))
        }
        "get_paper_fulltext" => {
            let args = vec![arguments];
            get_paper_fulltext(Params::Array(args))
        }
        _ => Ok(json!({
            "error": format!("Unknown tool: {}", tool_name)
        })),
    }
}

/// 文献搜索工具
pub fn literature_search(params: Params) -> Result<Value> {
    let args = params.parse::<Value>().unwrap();

    let query = args
        .get("query")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let limit = args
        .get("limit")
        .and_then(|v| v.as_u64())
        .unwrap_or(10) as usize;

    let _search_type = args
        .get("search_type")
        .and_then(|v| v.as_str())
        .unwrap_or("semantic");

    // TODO: 实际实现需要连接到数据库或使用本地索引
    // 这里返回模拟数据
    let results = json!([
        {
            "id": "paper:001",
            "title": format!("Related paper for '{}'", query),
            "authors": ["Author A", "Author B"],
            "year": 2024,
            "abstract": format!("This paper discusses topics related to '{}'", query),
            "similarity": 0.92
        },
        {
            "id": "paper:002",
            "title": format!("Another relevant paper on '{}'", query),
            "authors": ["Author C", "Author D"],
            "year": 2023,
            "abstract": format!("Another study about '{}'", query),
            "similarity": 0.87
        }
    ]);

    Ok(json!({
        "query": query,
        "results": results,
        "count": 2,
        "message": format!("Found 2 papers matching '{}'", query)
    }))
}

/// 文献摘要工具
pub fn summarize_paper(params: Params) -> Result<Value> {
    let args = params.parse::<Value>().unwrap();

    let file_path = args
        .get("file_path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| jsonrpc_core::Error::invalid_params("Missing file_path"))?;

    let detail_level = args
        .get("detail_level")
        .and_then(|v| v.as_str())
        .unwrap_or("standard");

    // 解析 PDF
    match PdfParser::parse(file_path) {
        Ok(doc) => {
            // 提取基本信息
            let title = PdfParser::extract_title_from_filename(file_path)
                .unwrap_or_else(|| "Unknown".to_string());

            // 生成摘要（这里使用简单的文本提取）
            let summary = json!({
                "title": title,
                "file_path": file_path,
                "page_count": doc.page_count,
                "detail_level": detail_level,
                "summary": {
                    "research_question": "需要通过 LLM 分析提取",
                    "methods": ["需要通过 LLM 分析提取"],
                    "key_results": ["需要通过 LLM 分析提取"],
                    "innovations": ["需要通过 LLM 分析提取"],
                    "limitations": ["需要通过 LLM 分析提取"]
                },
                "sections": doc.structure.sections.iter()
                    .map(|s| json!({
                        "type": s.section_type,
                        "title": s.title,
                        "content_preview": s.content.chars().take(200).collect::<String>()
                    }))
                    .collect::<Vec<_>>()
            });

            Ok(summary)
        }
        Err(e) => Ok(json!({
            "error": format!("Failed to parse PDF: {}", e),
            "file_path": file_path
        })),
    }
}

/// 对比文献工具
pub fn compare_papers(params: Params) -> Result<Value> {
    let args = params.parse::<Value>().unwrap();

    let file_paths = args
        .get("file_paths")
        .and_then(|v| v.as_array())
        .ok_or_else(|| jsonrpc_core::Error::invalid_params("Missing file_paths"))?;

    let aspects = args
        .get("aspects")
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.iter().map(|v| v.as_str()).collect::<Option<Vec<_>>>())
        .unwrap_or(vec!["methods", "results", "innovations"]);

    // 解析所有 PDF
    let mut papers_info = Vec::new();
    let mut paper_titles = Vec::new();

    for path in file_paths {
        if let Some(path_str) = path.as_str() {
            match PdfParser::parse(path_str) {
                Ok(doc) => {
                    let title = PdfParser::extract_title_from_filename(path_str)
                        .unwrap_or_else(|| "Unknown".to_string());
                    paper_titles.push(title.clone());

                    papers_info.push(json!({
                        "file_path": path_str,
                        "title": title,
                        "page_count": doc.page_count,
                        "section_count": doc.structure.sections.len()
                    }));
                }
                Err(e) => {
                    papers_info.push(json!({
                        "file_path": path_str,
                        "error": format!("Failed to parse: {}", e)
                    }));
                }
            }
        }
    }

    // 生成对比结果
    let comparison = json!({
        "papers": papers_info,
        "aspects": aspects,
        "comparison_table": {
            "methods": format!("方法对比: {}", paper_titles.join(" vs ")),
            "results": format!("结果对比: {}", paper_titles.join(" vs ")),
            "innovations": format!("创新点对比: {}", paper_titles.join(" vs "))
        },
        "note": "完整对比需要 LLM 分析"
    });

    Ok(comparison)
}

/// 提取关键实体工具
pub fn extract_key_entities(params: Params) -> Result<Value> {
    let args = params.parse::<Value>().unwrap();

    let file_path = args
        .get("file_path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| jsonrpc_core::Error::invalid_params("Missing file_path"))?;

    let entity_types = args
        .get("entity_types")
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.iter().map(|v| v.as_str()).collect::<Option<Vec<_>>>())
        .unwrap_or(vec!["all"]);

    // 解析 PDF
    match PdfParser::parse(file_path) {
        Ok(doc) => {
            // 使用分块器提取文本块
            let chunker = Chunker::default();
            let chunks = chunker.chunk_by_paragraph(&doc.text);

            // 简单的关键词提取（实际应该使用 NLP 模型）
            let mut terms = std::collections::HashSet::new();

            for chunk in chunks.iter().take(10) {
                // 提取大写单词作为潜在的术语
                for word in chunk.content.split_whitespace() {
                    if word.len() > 3 && word.chars().next().map_or(false, |c| c.is_uppercase()) {
                        terms.insert(word.to_string());
                    }
                }
            }

            let entities = json!({
                "file_path": file_path,
                "entity_types": entity_types,
                "entities": {
                    "terms": terms.into_iter().collect::<Vec<_>>(),
                    "methods": ["需要通过 NLP 模型提取"],
                    "datasets": ["需要通过 NLP 模型提取"],
                    "people": ["需要通过 NER 模型提取"],
                    "institutions": ["需要通过 NER 模型提取"]
                },
                "chunk_count": chunks.len()
            });

            Ok(entities)
        }
        Err(e) => Ok(json!({
            "error": format!("Failed to parse PDF: {}", e),
            "file_path": file_path
        })),
    }
}

/// 获取文献全文工具
pub fn get_paper_fulltext(params: Params) -> Result<Value> {
    let args = params.parse::<Value>().unwrap();

    let file_path = args
        .get("file_path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| jsonrpc_core::Error::invalid_params("Missing file_path"))?;

    let include_structure = args
        .get("include_structure")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let output_format = args
        .get("output_format")
        .and_then(|v| v.as_str())
        .unwrap_or("text");

    // 解析 PDF
    match PdfParser::parse(file_path) {
        Ok(doc) => {
            let title = PdfParser::extract_title_from_filename(file_path)
                .unwrap_or_else(|| "Unknown".to_string());

            match output_format {
                "text" => {
                    Ok(json!({
                        "file_path": file_path,
                        "title": title,
                        "page_count": doc.page_count,
                        "text": doc.text,
                        "structure": if include_structure { Some(doc.structure) } else { None }
                    }))
                }
                "markdown" => {
                    let mut markdown = format!("# {}\n\n", title);
                    for section in &doc.structure.sections {
                        markdown.push_str(&format!("## {}\n\n", section.title));
                        markdown.push_str(&section.content);
                        markdown.push_str("\n\n");
                    }

                    Ok(json!({
                        "file_path": file_path,
                        "title": title,
                        "page_count": doc.page_count,
                        "markdown": markdown
                    }))
                }
                "json" => {
                    Ok(json!({
                        "file_path": file_path,
                        "title": title,
                        "page_count": doc.page_count,
                        "structure": doc.structure,
                        "full_text": doc.text
                    }))
                }
                _ => {
                    Ok(json!({
                        "error": format!("Unknown output format: {}", output_format)
                    }))
                }
            }
        }
        Err(e) => Ok(json!({
            "error": format!("Failed to parse PDF: {}", e),
            "file_path": file_path
        })),
    }
}
